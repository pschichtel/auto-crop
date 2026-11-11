//! Cropping each scanned page to the paper it shows.

use anyhow::Result;
use kurbo::Rect;
use lopdf::{Document, ObjectId};
use rayon::prelude::*;

use crate::detect::{FracRect, find_paper};
use crate::image_decode::decode_image;
use crate::pdf_page::{scan_image, set_page_box, visible_box};

/// Crops that would trim less than this many points from every side are not worth applying.
const MIN_TRIM: f64 = 1.0;

/// What happened to a single page.
#[derive(Debug)]
pub enum PageOutcome {
    Cropped { from: Rect, to: Rect },
    NotAPicture,
    NoPaperFound,
    NothingToCrop,
    Failed(anyhow::Error),
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
                && let Err(e) = set_page_box(doc, id, *to)
            {
                return (number, PageOutcome::Failed(e));
            }
            (number, outcome)
        })
        .collect()
}

fn analyze_page(doc: &Document, page_id: ObjectId) -> Result<PageOutcome> {
    let visible = visible_box(doc, page_id)?;
    let Some(image) = scan_image(doc, page_id, visible)? else {
        return Ok(PageOutcome::NotAPicture);
    };

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
