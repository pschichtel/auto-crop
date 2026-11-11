//! Removing uneven lighting from a photographed document.
//!
//! This is a flat-field correction: estimate what the bare paper looks like at every
//! point, then divide the image by it. Shading, fold shadows and a colour cast of the
//! paper disappear and the paper turns white, while lines and text keep their colour.
//!
//! The paper is estimated as the brightest colour in each neighbourhood, which ignores
//! lines and text thinner than the neighbourhood. Dark areas larger than that, such as
//! photos on the page, would be taken for shaded paper and washed out, so this is only
//! suitable for drawings and text on light paper.

use image::{Rgb, RgbImage, imageops};
use rayon::prelude::*;

/// Longest side of the downscaled image the paper is estimated on. Lighting changes
/// slowly, so little resolution is needed.
const WORK_SIZE: u32 = 1000;
/// Radius, in working pixels, of the neighbourhood whose brightest colour is taken for
/// the paper. Must exceed half the width of the thickest strokes and filled areas.
const PAPER_RADIUS: usize = 20;
/// Blur applied to the paper estimate, in working pixels, to hide the blocky shape of
/// the neighbourhoods.
const PAPER_BLUR: f32 = 10.0;
/// The paper is never assumed darker than this, which keeps large dark areas and the
/// surroundings of the sheet from being amplified into noise.
const MIN_PAPER: f32 = 48.0;

/// Even out the lighting of `img` so that the paper becomes white.
pub fn whiten(img: &RgbImage) -> RgbImage {
    let (w, h) = img.dimensions();
    let scale = (WORK_SIZE as f64 / w.max(h) as f64).min(1.0);
    let sw = ((w as f64 * scale).round() as u32).max(1);
    let sh = ((h as f64 * scale).round() as u32).max(1);
    let small = imageops::resize(img, sw, sh, imageops::FilterType::Triangle);
    let paper = imageops::blur(&brightest_nearby(&small, PAPER_RADIUS), PAPER_BLUR);

    let mut out = RgbImage::new(w, h);
    out.par_chunks_mut(w as usize * 3)
        .enumerate()
        .for_each(|(y, row)| {
            // Position of this row on the paper estimate, aligning pixel centres.
            let py = ((y as f32 + 0.5) * sh as f32 / h as f32 - 0.5).clamp(0.0, (sh - 1) as f32);
            let (y0, fy) = (py.floor() as u32, py.fract());
            let y1 = (y0 + 1).min(sh - 1);
            for (x, pixel) in row.chunks_exact_mut(3).enumerate() {
                let px =
                    ((x as f32 + 0.5) * sw as f32 / w as f32 - 0.5).clamp(0.0, (sw - 1) as f32);
                let (x0, fx) = (px.floor() as u32, px.fract());
                let x1 = (x0 + 1).min(sw - 1);
                let source = img.get_pixel(x as u32, y as u32);
                for c in 0..3 {
                    let at = |x: u32, y: u32| paper.get_pixel(x, y)[c] as f32;
                    let top = at(x0, y0) * (1.0 - fx) + at(x1, y0) * fx;
                    let bottom = at(x0, y1) * (1.0 - fx) + at(x1, y1) * fx;
                    let paper = (top * (1.0 - fy) + bottom * fy).max(MIN_PAPER);
                    pixel[c] = (source[c] as f32 * 255.0 / paper).round().min(255.0) as u8;
                }
            }
        });
    out
}

/// Per channel, the brightest value within a square of the given radius around each pixel.
fn brightest_nearby(img: &RgbImage, radius: usize) -> RgbImage {
    let (w, h) = (img.width() as usize, img.height() as usize);
    // The square maximum separates into a pass along the rows and one along the columns.
    let pass = |src: &RgbImage, horizontal: bool| {
        RgbImage::from_fn(w as u32, h as u32, |x, y| {
            let (pos, len) = if horizontal {
                (x as usize, w)
            } else {
                (y as usize, h)
            };
            let mut best = [0u8; 3];
            for p in pos.saturating_sub(radius)..=(pos + radius).min(len - 1) {
                let pixel = if horizontal {
                    src.get_pixel(p as u32, y)
                } else {
                    src.get_pixel(x, p as u32)
                };
                for c in 0..3 {
                    best[c] = best[c].max(pixel[c]);
                }
            }
            Rgb(best)
        })
    };
    pass(&pass(img, true), false)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A drawing on cream paper that gets darker towards the right, crossed by the broad
    /// shadow of a fold, with a thin black line and a thin red line. (The edges of a
    /// shadow are only corrected partially, so the tests look at its middle.)
    fn shaded_drawing() -> RgbImage {
        RgbImage::from_fn(800, 600, |x, y| {
            let light =
                (1.0 - 0.4 * x as f32 / 800.0) * if (330..490).contains(&x) { 0.8 } else { 1.0 };
            let ink = if (100..104).contains(&y) {
                [0.1, 0.1, 0.1]
            } else if (300..304).contains(&y) {
                [0.9, 0.15, 0.1]
            } else {
                [0.95, 0.92, 0.85]
            };
            Rgb(ink.map(|v| (v * light * 255.0) as u8))
        })
    }

    #[test]
    fn paper_becomes_evenly_white() {
        let out = whiten(&shaded_drawing());
        for (x, y) in [(50, 200), (410, 200), (750, 200), (50, 500), (750, 500)] {
            let pixel = out.get_pixel(x, y).0;
            assert!(
                pixel.iter().all(|&v| v >= 245),
                "paper at {x},{y} is {pixel:?}"
            );
        }
    }

    #[test]
    fn lines_keep_their_colour() {
        let out = whiten(&shaded_drawing());
        for x in [50, 410, 750] {
            let black = out.get_pixel(x, 102).0;
            assert!(
                black.iter().all(|&v| v < 60),
                "black line at {x} is {black:?}"
            );
            let red = out.get_pixel(x, 302).0;
            assert!(
                red[0] > 200 && red[1] < 80 && red[2] < 80,
                "red line at {x} is {red:?}"
            );
        }
    }
}
