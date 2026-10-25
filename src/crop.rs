//! Finding the scanned image on each page and cropping the page to the paper it shows.

use anyhow::{Context, Result, bail};
use kurbo::{Affine, Rect};
use lopdf::content::Content;
use lopdf::{Dictionary, Document, Object, ObjectId};
use rayon::prelude::*;

use crate::detect::{FracRect, find_paper};
use crate::image_decode::decode_image;

/// A page counts as a picture page if one image covers at least this fraction of it.
const MIN_IMAGE_COVERAGE: f64 = 0.5;
/// Crops that would trim less than this many points from every side are not worth applying.
const MIN_TRIM: f64 = 1.0;
/// Maximum nesting depth when following form XObjects.
const MAX_FORM_DEPTH: usize = 8;

/// What happened to a single page.
#[derive(Debug)]
pub enum PageOutcome {
    Cropped { from: Rect, to: Rect },
    NotAPicture,
    NoPaperFound,
    NothingToCrop,
    Failed(anyhow::Error),
}

/// Read a PDF rectangle array such as `/MediaBox`.
fn rect_from_object(doc: &Document, obj: &Object) -> Option<Rect> {
    let [x0, y0, x1, y1] = numbers(doc, array(doc, obj)?)?;
    Some(Rect::new(x0, y0, x1, y1).abs())
}

fn rect_to_object(rect: Rect) -> Object {
    Object::Array(
        [rect.x0, rect.y0, rect.x1, rect.y1]
            .into_iter()
            .map(|v| Object::Real(v as f32))
            .collect(),
    )
}

/// The array `obj` refers to, resolving an indirect reference.
fn array<'a>(doc: &'a Document, obj: &'a Object) -> Option<&'a [Object]> {
    let (_, obj) = doc.dereference(obj).ok()?;
    obj.as_array().ok().map(Vec::as_slice)
}

/// Exactly `N` numbers, resolving indirect references.
fn numbers<const N: usize>(doc: &Document, items: &[Object]) -> Option<[f64; N]> {
    let nums: Vec<f64> = items
        .iter()
        .map(|o| {
            doc.dereference(o)
                .ok()
                .and_then(|(_, o)| o.as_float().ok())
                .map(f64::from)
        })
        .collect::<Option<_>>()?;
    nums.try_into().ok()
}

/// An image drawn on a page, together with the matrix mapping the unit square onto the page.
struct PlacedImage {
    id: ObjectId,
    ctm: Affine,
}

/// Crop every picture page of `doc` in place and return the outcome for each page number.
pub fn crop_document(doc: &mut Document) -> Vec<(u32, PageOutcome)> {
    let pages: Vec<(u32, ObjectId)> = doc.get_pages().into_iter().collect();
    let outcomes: Vec<(u32, ObjectId, PageOutcome)> = pages
        .par_iter()
        .map(|&(number, id)| {
            (
                number,
                id,
                analyze_page(doc, id).unwrap_or_else(PageOutcome::Failed),
            )
        })
        .collect();

    outcomes
        .into_iter()
        .map(|(number, id, outcome)| {
            if let PageOutcome::Cropped { to, .. } = &outcome
                && let Err(e) = apply_crop(doc, id, *to)
            {
                return (number, PageOutcome::Failed(e));
            }
            (number, outcome)
        })
        .collect()
}

fn analyze_page(doc: &Document, page_id: ObjectId) -> Result<PageOutcome> {
    let media_box = inherited(doc, page_id, b"MediaBox")
        .and_then(|o| rect_from_object(doc, o))
        .context("page has no valid /MediaBox")?;
    let visible = inherited(doc, page_id, b"CropBox")
        .and_then(|o| rect_from_object(doc, o))
        .map_or(media_box, |crop| crop.intersect(media_box));
    if visible.area() <= 0.0 {
        bail!("page has an empty visible area");
    }

    let images = page_images(doc, page_id)?;
    let largest = images
        .into_iter()
        .map(|img| {
            let area = img
                .ctm
                .transform_rect_bbox(Rect::new(0.0, 0.0, 1.0, 1.0))
                .intersect(visible)
                .area();
            (img, area)
        })
        .max_by(|a, b| a.1.total_cmp(&b.1));
    let Some((image, area)) = largest else {
        return Ok(PageOutcome::NotAPicture);
    };
    if area < MIN_IMAGE_COVERAGE * visible.area() {
        return Ok(PageOutcome::NotAPicture);
    }

    let stream = doc.get_object(image.id)?.as_stream()?;
    let bitmap = decode_image(doc, stream)?;
    let Some(FracRect { x0, y0, x1, y1 }) = find_paper(&bitmap) else {
        return Ok(PageOutcome::NoPaperFound);
    };

    // Image row 0 is at the top of the unit square, i.e. at v = 1.
    let paper = image
        .ctm
        .transform_rect_bbox(Rect::new(x0, 1.0 - y1, x1, 1.0 - y0))
        .intersect(visible);
    let trimmed = [
        paper.x0 - visible.x0,
        paper.y0 - visible.y0,
        visible.x1 - paper.x1,
        visible.y1 - paper.y1,
    ];
    if paper.area() <= 0.0 || trimmed.iter().all(|&t| t < MIN_TRIM) {
        return Ok(PageOutcome::NothingToCrop);
    }
    Ok(PageOutcome::Cropped {
        from: visible,
        to: paper,
    })
}

fn apply_crop(doc: &mut Document, page_id: ObjectId, rect: Rect) -> Result<()> {
    let page = doc.get_object_mut(page_id)?.as_dict_mut()?;
    page.set("MediaBox", rect_to_object(rect));
    page.set("CropBox", rect_to_object(rect));
    // These must lie within the media box; drop them rather than risk invalid values.
    page.remove(b"BleedBox");
    page.remove(b"TrimBox");
    page.remove(b"ArtBox");
    Ok(())
}

/// Look up a page attribute, following the page tree upwards for inheritable keys.
fn inherited<'a>(doc: &'a Document, page_id: ObjectId, key: &[u8]) -> Option<&'a Object> {
    let mut node = doc.get_dictionary(page_id).ok()?;
    for _ in 0..64 {
        if let Ok(value) = node.get(key) {
            return Some(value);
        }
        let parent = node.get(b"Parent").and_then(Object::as_reference).ok()?;
        node = doc.get_dictionary(parent).ok()?;
    }
    None
}

/// All images drawn by the page's content streams, including those inside form XObjects.
fn page_images(doc: &Document, page_id: ObjectId) -> Result<Vec<PlacedImage>> {
    let (inline, ids) = doc.get_page_resources(page_id)?;
    let resources: Vec<&Dictionary> = inline
        .into_iter()
        .chain(ids.into_iter().filter_map(|id| doc.get_dictionary(id).ok()))
        .collect();
    let content = doc.get_page_content(page_id);
    let mut images = Vec::new();
    collect_images(doc, &content, &resources, Affine::IDENTITY, 0, &mut images)?;
    Ok(images)
}

fn collect_images(
    doc: &Document,
    content: &[u8],
    resources: &[&Dictionary],
    base: Affine,
    depth: usize,
    out: &mut Vec<PlacedImage>,
) -> Result<()> {
    let content = Content::decode(content).context("failed to parse content stream")?;
    let mut ctm = base;
    let mut stack = Vec::new();
    for op in &content.operations {
        match op.operator.as_str() {
            "q" => stack.push(ctm),
            "Q" => ctm = stack.pop().unwrap_or(base),
            "cm" => {
                // The new matrix applies before the current one.
                if let Some(m) = numbers(doc, &op.operands).map(Affine::new) {
                    ctm *= m;
                }
            }
            "Do" => {
                let Some(name) = op.operands.first().and_then(|o| o.as_name().ok()) else {
                    continue;
                };
                let Some(id) = lookup_xobject(doc, resources, name) else {
                    continue;
                };
                let Ok(stream) = doc.get_object(id).and_then(Object::as_stream) else {
                    continue;
                };
                match stream.dict.get(b"Subtype").and_then(Object::as_name) {
                    Ok(b"Image") => out.push(PlacedImage { id, ctm }),
                    Ok(b"Form") if depth < MAX_FORM_DEPTH => {
                        let matrix = stream
                            .dict
                            .get(b"Matrix")
                            .ok()
                            .and_then(|m| numbers(doc, array(doc, m)?))
                            .map_or(Affine::IDENTITY, Affine::new);
                        // Forms without their own resources inherit those of the caller.
                        let own = stream
                            .dict
                            .get(b"Resources")
                            .ok()
                            .and_then(|r| doc.dereference(r).ok())
                            .and_then(|(_, r)| r.as_dict().ok());
                        let form_resources: Vec<&Dictionary> = match own {
                            Some(dict) => vec![dict],
                            None => resources.to_vec(),
                        };
                        let form_content = stream
                            .decompressed_content()
                            .unwrap_or_else(|_| stream.content.clone());
                        collect_images(
                            doc,
                            &form_content,
                            &form_resources,
                            ctm * matrix,
                            depth + 1,
                            out,
                        )?;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn lookup_xobject(doc: &Document, resources: &[&Dictionary], name: &[u8]) -> Option<ObjectId> {
    resources.iter().find_map(|res| {
        let (_, xobjects) = doc.dereference(res.get(b"XObject").ok()?).ok()?;
        xobjects
            .as_dict()
            .ok()?
            .get(name)
            .and_then(Object::as_reference)
            .ok()
    })
}
