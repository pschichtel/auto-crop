//! The `straighten` mode: unwarping image files and the scanned pages of PDFs.
//!
//! Unlike cropping, this resamples the image, so the result is re-encoded.

use std::collections::HashMap;
use std::io::Cursor;

use anyhow::{Context, Result, bail};
use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ImageFormat, RgbImage};
use kurbo::Rect;
use lopdf::{Dictionary, Document, Object, ObjectId, Stream, dictionary};

use crate::detect::find_paper;
use crate::image_decode::decode_image;
use crate::pdf_page::{PlacedImage, scan_image, set_page_box, visible_box};
use crate::unwarp::{Margins, Unwarper};
use crate::whiten::whiten;

/// Quality of the JPEGs written for straightened images.
const JPEG_QUALITY: u8 = 90;
/// Margin shown to the model beyond an image edge the paper touches, as a fraction of
/// the image size.
const MARGIN: f64 = 0.08;
/// The paper counts as touching an image edge if it comes this close, as a fraction of
/// the image size.
const TOUCHING: f64 = 0.005;
/// An image edge shows paper if this share of its pixels is within `PAPER_TOLERANCE`
/// of the paper's colour in every channel.
const PAPER_EDGE_SHARE: f64 = 0.6;
const PAPER_TOLERANCE: u8 = 48;

/// How to straighten: the model and the optional steps around it.
pub struct Straightener {
    pub unwarper: Unwarper,
    /// Even out the lighting afterwards, so that the paper becomes white.
    pub whiten: bool,
}

impl Straightener {
    /// Flatten the document shown in `img` and crop to it.
    fn flatten(&self, img: &RgbImage) -> Result<RgbImage> {
        let straight = self.unwarper.predict(img, &margins_for(img))?.unwarp(img);
        Ok(if self.whiten {
            whiten(&straight)
        } else {
            straight
        })
    }
}

/// Size of an image in pixels before and after straightening.
#[derive(Debug, Clone, Copy)]
pub struct Resized {
    pub from: (u32, u32),
    pub to: (u32, u32),
}

/// What happened to a single page.
#[derive(Debug)]
pub enum PageOutcome {
    Straightened(Resized),
    NotAPicture,
    Failed(anyhow::Error),
}

/// Straighten an encoded image file and encode the result as `format`.
pub fn straighten_image(
    straightener: &Straightener,
    data: &[u8],
    format: ImageFormat,
) -> Result<(Vec<u8>, Resized)> {
    let img = image::load_from_memory(data).context("failed to decode the image")?;
    let gray = !img.color().has_color();
    let rgb = img.into_rgb8();
    let straight = straightener.flatten(&rgb)?;
    let resized = Resized {
        from: rgb.dimensions(),
        to: straight.dimensions(),
    };

    let encoded = if format == ImageFormat::Jpeg {
        encode_jpeg(straight, gray)?
    } else {
        let img = DynamicImage::ImageRgb8(straight);
        let img = if gray {
            DynamicImage::ImageLuma8(img.into_luma8())
        } else {
            img
        };
        let mut buffer = Cursor::new(Vec::new());
        img.write_to(&mut buffer, format)
            .context("failed to encode the image")?;
        buffer.into_inner()
    };
    Ok((encoded, resized))
}

/// Margins for the model on the sides where the paper runs to the edge of the image,
/// filled with the colour of the scanner lid.
///
/// This relies on the paper detection of the crop mode, which is made for scans. On
/// photos it can report the paper as touching an edge where there is only background,
/// and a margin there makes the model include that background. A side therefore only
/// gets a margin if the pixels along that edge also look like the paper.
fn margins_for(img: &RgbImage) -> Margins {
    let Some(paper) = find_paper(img) else {
        return Margins::default();
    };
    let small = image::imageops::thumbnail(img, 200, 200);
    let (sw, sh) = small.dimensions();
    // The paper's extent on the thumbnail, as pixel ranges.
    let span = |from: f64, to: f64, len: u32| {
        let start = ((from * len as f64).floor() as u32).min(len - 1);
        start..((to * len as f64).ceil() as u32).clamp(start + 1, len)
    };
    let (xs, ys) = (span(paper.x0, paper.x1, sw), span(paper.y0, paper.y1, sh));
    let inside = |x: u32, y: u32| xs.contains(&x) && ys.contains(&y);

    // Typical colours of the paper and of everything around it.
    let paper_color = median_color(
        small
            .enumerate_pixels()
            .filter(|(x, y, _)| inside(*x, *y))
            .map(|p| p.2.0),
    );
    let lid_color = median_color(
        small
            .enumerate_pixels()
            .filter(|(x, y, _)| !inside(*x, *y))
            .map(|p| p.2.0),
    );
    let (Some(paper_color), lid_color) = (paper_color, lid_color.unwrap_or([128; 3])) else {
        return Margins::default();
    };

    let looks_like_paper = |pixels: &mut dyn Iterator<Item = (u32, u32)>| {
        let (mut total, mut similar) = (0, 0);
        for (x, y) in pixels {
            let pixel = small.get_pixel(x, y).0;
            total += 1;
            similar +=
                usize::from((0..3).all(|c| pixel[c].abs_diff(paper_color[c]) <= PAPER_TOLERANCE));
        }
        similar as f64 >= total as f64 * PAPER_EDGE_SHARE
    };
    let (w, h) = img.dimensions();
    let margin = |touching: bool, edge: &mut dyn Iterator<Item = (u32, u32)>, len: u32| {
        if touching && looks_like_paper(edge) {
            (len as f64 * MARGIN).round() as u32
        } else {
            0
        }
    };
    Margins {
        left: margin(paper.x0 < TOUCHING, &mut ys.clone().map(|y| (0, y)), w),
        top: margin(paper.y0 < TOUCHING, &mut xs.clone().map(|x| (x, 0)), h),
        right: margin(
            paper.x1 > 1.0 - TOUCHING,
            &mut ys.clone().map(|y| (sw - 1, y)),
            w,
        ),
        bottom: margin(
            paper.y1 > 1.0 - TOUCHING,
            &mut xs.clone().map(|x| (x, sh - 1)),
            h,
        ),
        color: lid_color,
    }
}

/// Per-channel median of some colours.
fn median_color(colors: impl Iterator<Item = [u8; 3]>) -> Option<[u8; 3]> {
    let colors: Vec<[u8; 3]> = colors.collect();
    if colors.is_empty() {
        return None;
    }
    Some(std::array::from_fn(|c| {
        let mut channel: Vec<u8> = colors.iter().map(|p| p[c]).collect();
        *channel.select_nth_unstable(colors.len() / 2).1
    }))
}

fn encode_jpeg(img: RgbImage, gray: bool) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let encoder = JpegEncoder::new_with_quality(&mut buffer, JPEG_QUALITY);
    let img = DynamicImage::ImageRgb8(img);
    let img = if gray {
        DynamicImage::ImageLuma8(img.into_luma8())
    } else {
        img
    };
    img.write_with_encoder(encoder)
        .context("failed to encode the image as JPEG")?;
    Ok(buffer)
}

/// Straighten every picture page of `doc` in place and return the outcome for each page number.
///
/// Pages are processed one after the other: a single 600 dpi page already takes a few
/// hundred megabytes while it is resampled, which itself uses all cores.
pub fn straighten_document(
    straightener: &Straightener,
    doc: &mut Document,
) -> Vec<(u32, PageOutcome)> {
    // Images shared between pages are straightened only once.
    let mut done: HashMap<ObjectId, Resized> = HashMap::new();
    doc.get_pages()
        .into_iter()
        .map(|(number, page_id)| {
            let outcome = straighten_page(straightener, doc, page_id, &mut done)
                .unwrap_or_else(PageOutcome::Failed);
            (number, outcome)
        })
        .collect()
}

fn straighten_page(
    straightener: &Straightener,
    doc: &mut Document,
    page_id: ObjectId,
    done: &mut HashMap<ObjectId, Resized>,
) -> Result<PageOutcome> {
    let visible = visible_box(doc, page_id)?;
    let Some(image) = scan_image(doc, page_id, visible)? else {
        return Ok(PageOutcome::NotAPicture);
    };
    // Check the placement first; it is cheap and the image must not be replaced otherwise.
    let rotated = quarter_turned(&image)?;

    let resized = match done.get(&image.id) {
        Some(resized) => *resized,
        None => {
            let bitmap = decode_image(doc, doc.get_object(image.id)?.as_stream()?)?;
            let gray = bitmap.pixels().all(|p| p[0] == p[1] && p[1] == p[2]);
            let straight = straightener.flatten(&bitmap)?;
            let resized = Resized {
                from: bitmap.dimensions(),
                to: straight.dimensions(),
            };
            let stream = jpeg_xobject(straight, gray)?;
            *doc.get_object_mut(image.id)? = Object::Stream(stream);
            done.insert(image.id, resized);
            resized
        }
    };

    fit_page_to_image(doc, page_id, &image, rotated, resized)?;
    Ok(PageOutcome::Straightened(resized))
}

/// Whether the image is placed rotated by 90° or 270°. Fails for any placement that is not
/// a multiple of a quarter turn, as the page cannot simply be resized around such an image.
fn quarter_turned(image: &PlacedImage) -> Result<bool> {
    let [a, b, c, d, _, _] = image.ctm.as_coeffs();
    let scale = a.abs().max(b.abs()).max(c.abs()).max(d.abs());
    let tiny = |v: f64| v.abs() <= scale * 1e-3;
    if tiny(b) && tiny(c) && !tiny(a) && !tiny(d) {
        Ok(false)
    } else if tiny(a) && tiny(d) && !tiny(b) && !tiny(c) {
        Ok(true)
    } else {
        bail!("the image is placed at an angle on the page");
    }
}

/// An image XObject holding `img` as JPEG.
fn jpeg_xobject(img: RgbImage, gray: bool) -> Result<Stream> {
    let (width, height) = img.dimensions();
    let dict = dictionary! {
        "Type" => "XObject",
        "Subtype" => "Image",
        "Width" => width as i64,
        "Height" => height as i64,
        "ColorSpace" => if gray { "DeviceGray" } else { "DeviceRGB" },
        "BitsPerComponent" => 8,
        "Filter" => "DCTDecode",
    };
    Ok(Stream::new(dict, encode_jpeg(img, gray)?).with_compression(false))
}

/// Resize the page after its image changed size, keeping the scan's resolution.
///
/// The page's content still draws the image into its old rectangle. Rather than editing
/// the content stream, it is wrapped in a transformation that maps the old rectangle
/// onto the new page, which also carries along anything else drawn on the page.
fn fit_page_to_image(
    doc: &mut Document,
    page_id: ObjectId,
    image: &PlacedImage,
    rotated: bool,
    resized: Resized,
) -> Result<()> {
    let old = image.ctm.transform_rect_bbox(Rect::new(0.0, 0.0, 1.0, 1.0));
    let across = resized.to.0 as f64 / resized.from.0 as f64;
    let down = resized.to.1 as f64 / resized.from.1 as f64;
    // With a quarter turn the image's width runs along the page's y axis.
    let (sx, sy) = if rotated {
        (down, across)
    } else {
        (across, down)
    };
    let new = Rect::new(0.0, 0.0, old.width() * sx, old.height() * sy);

    let transform = format!("q {sx} 0 0 {sy} {} {} cm\n", -old.x0 * sx, -old.y0 * sy);
    let prefix = doc.add_object(Stream::new(Dictionary::new(), transform.into_bytes()));
    let suffix = doc.add_object(Stream::new(Dictionary::new(), b"\nQ\n".to_vec()));
    let contents = std::iter::once(prefix)
        .chain(doc.get_page_contents(page_id))
        .chain(std::iter::once(suffix))
        .map(Object::Reference)
        .collect();
    doc.get_object_mut(page_id)?
        .as_dict_mut()?
        .set("Contents", Object::Array(contents));
    set_page_box(doc, page_id, new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgb;

    #[test]
    fn margins_where_the_paper_touches_the_edge() {
        // White paper against the top and left edge of a scan with a dark lid.
        let img = RgbImage::from_fn(500, 800, |x, y| {
            if x < 400 && y < 650 {
                Rgb([250, 250, 250])
            } else {
                Rgb([40, 40, 40])
            }
        });
        let expected = Margins {
            left: 40,
            top: 64,
            right: 0,
            bottom: 0,
            color: [40, 40, 40],
        };
        assert_eq!(margins_for(&img), expected);
    }

    #[test]
    fn no_margins_without_a_detected_paper() {
        let img = RgbImage::from_pixel(300, 300, Rgb([128, 128, 128]));
        assert_eq!(margins_for(&img), Margins::default());
    }

    #[test]
    fn no_margin_at_an_edge_that_shows_background() {
        // Light paper on a dark wooden floor, as in a photo. A narrow light streak in the
        // wood connects the paper to the right edge, which the crop detector takes for
        // paper reaching that edge.
        let img = RgbImage::from_fn(600, 800, |x, y| {
            if (60..500).contains(&x) && (40..760).contains(&y) {
                Rgb([225, 225, 228])
            } else if x >= 500 && (300..330).contains(&y) {
                Rgb([215, 170, 120])
            } else {
                Rgb([150, 95, 50])
            }
        });
        let paper = find_paper(&img).expect("the detector finds something");
        assert!(
            paper.x1 > 1.0 - TOUCHING,
            "the detector no longer misfires here: {paper:?}"
        );
        let margins = margins_for(&img);
        assert_eq!(
            (margins.left, margins.top, margins.right, margins.bottom),
            (0, 0, 0, 0)
        );
    }
}
