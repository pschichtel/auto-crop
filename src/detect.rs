//! Locating the scanned sheet of paper inside a scanner image.
//!
//! The paper may touch some edges of the scan, so the most common border colour is not
//! necessarily the scanner background, and the lid itself may show several tones. Instead,
//! the significant border colours are collected and the darkest k of them are tried as
//! background together, starting with all of them:
//!
//! 1. Flood fill from the border through every pixel that is close to one of those colours.
//! 2. Everything the flood did not reach is a candidate; the largest connected
//!    candidate region is taken to be the paper.
//! 3. The result is accepted if that region is brighter than the background (paper is
//!    assumed to be lighter than the scanner lid) and actually leaves something to crop.

use std::collections::VecDeque;

use image::{RgbImage, imageops};

/// Longest side of the downscaled working image. Detection does not need full resolution.
const WORK_SIZE: u32 = 1000;
/// Width of the border strip used to estimate the background, as a fraction of the shorter side.
const BORDER_FRACTION: f64 = 0.01;
/// A border colour must make up at least this fraction of the border to be a background candidate.
const MIN_CLUSTER_SHARE: f64 = 0.03;
/// Maximum number of background candidates to try.
const MAX_CANDIDATES: usize = 4;
/// Border pixels within this distance of a cluster's centre belong to the cluster.
const CLUSTER_RADIUS: u8 = 24;
/// Erosion radius (in working pixels) used to detach thin lines and specks from the paper.
const ERODE_RADIUS: usize = 2;
/// A row/column at the paper's edge needs this share of paper-bright pixels to be kept.
const EDGE_PAPER_SHARE: f64 = 0.5;
/// The paper must cover at least this fraction of the image to be accepted.
const MIN_PAPER_AREA: f64 = 0.05;

/// An axis-aligned rectangle in image space, expressed as fractions of the image
/// width/height. `y` grows downwards, i.e. row 0 of the image is `y = 0`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FracRect {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

impl FracRect {
    const FULL: FracRect = FracRect {
        x0: 0.0,
        y0: 0.0,
        x1: 1.0,
        y1: 1.0,
    };
}

/// A candidate background colour and the tolerance used to match it.
struct Background {
    color: [u8; 3],
    tolerance: u8,
}

/// Find the bounding box of the scanned paper.
///
/// Returns [`FracRect::FULL`] if the paper fills the whole image and `None` if no
/// plausible paper was found.
pub fn find_paper(img: &RgbImage) -> Option<FracRect> {
    let (w, h) = img.dimensions();
    if w < 8 || h < 8 {
        return None;
    }

    let scale = (WORK_SIZE as f64 / w.max(h) as f64).min(1.0);
    let sw = ((w as f64 * scale).round() as u32).max(1);
    let sh = ((h as f64 * scale).round() as u32).max(1);
    let small = if scale < 1.0 {
        imageops::resize(img, sw, sh, imageops::FilterType::Triangle)
    } else {
        img.clone()
    };
    // A light blur suppresses scanner noise and compression artifacts.
    let small = imageops::blur(&small, 1.0);

    // Lids are rarely uniform (shading, scanner padding), so several border colours may
    // be background at once. Try the k darkest together, from the most to the fewest; the
    // brightest candidate is often the paper itself touching the border, which fails the
    // brightness check and is dropped in the next round.
    let candidates = background_candidates(&small);
    let mut fills_image = false;
    for k in (1..=candidates.len()).rev() {
        match paper_against(&small, &candidates[..k]) {
            Some(rect) if rect == FracRect::FULL => fills_image = true,
            Some(rect) => return Some(refine_edges(img, rect, &candidates[..k], scale)),
            None => {}
        }
    }
    fills_image.then_some(FracRect::FULL)
}

/// Snap each edge of the coarse rectangle found on the working image to the exact paper
/// edge in the full resolution image.
///
/// Near every coarse edge, each row/column is scored by the share of its pixels that look
/// like paper, i.e. are brighter than every background tone. The outermost line reaching
/// [`EDGE_PAPER_SHARE`] becomes the new edge. This drops lid strips and the dark shadow
/// that the paper's edge casts, which the coarse pass cannot resolve.
fn refine_edges(
    img: &RgbImage,
    coarse: FracRect,
    backgrounds: &[Background],
    scale: f64,
) -> FracRect {
    let (w, h) = (img.width() as usize, img.height() as usize);
    let threshold = backgrounds
        .iter()
        .map(|bg| luma(bg.color) + bg.tolerance as f64)
        .fold(0.0, f64::max);
    let is_paper = |x: usize, y: usize| luma(img.get_pixel(x as u32, y as u32).0) > threshold;

    let window = ((ERODE_RADIUS + 4) as f64 / scale).ceil() as usize;
    let to_px = |f: f64, len: usize| ((f * len as f64).round() as usize).min(len);
    let (mut x0, mut x1) = (to_px(coarse.x0, w), to_px(coarse.x1, w));
    let (mut y0, mut y1) = (to_px(coarse.y0, h), to_px(coarse.y1, h));

    // Share of paper pixels in column `x` between rows `y0..y1`, or row `y` between `x0..x1`.
    let column_share = |x: usize, y0: usize, y1: usize| {
        (y0..y1).filter(|&y| is_paper(x, y)).count() as f64 / (y1 - y0).max(1) as f64
    };
    let row_share = |y: usize, x0: usize, x1: usize| {
        (x0..x1).filter(|&x| is_paper(x, y)).count() as f64 / (x1 - x0).max(1) as f64
    };
    let paper_column = |x: &usize| column_share(*x, y0, y1) >= EDGE_PAPER_SHARE;
    let new_x0 = (x0.saturating_sub(window)..(x0 + window).min(x1)).find(paper_column);
    let new_x1 = (x1.saturating_sub(window).max(x0)..(x1 + window).min(w))
        .rev()
        .find(paper_column);
    if let (Some(a), Some(b)) = (new_x0, new_x1) {
        (x0, x1) = (a, b + 1);
    }

    let paper_row = |y: &usize| row_share(*y, x0, x1) >= EDGE_PAPER_SHARE;
    let new_y0 = (y0.saturating_sub(window)..(y0 + window).min(y1)).find(paper_row);
    let new_y1 = (y1.saturating_sub(window).max(y0)..(y1 + window).min(h))
        .rev()
        .find(paper_row);
    if let (Some(a), Some(b)) = (new_y0, new_y1) {
        (y0, y1) = (a, b + 1);
    }

    FracRect {
        x0: x0 as f64 / w as f64,
        y0: y0 as f64 / h as f64,
        x1: x1 as f64 / w as f64,
        y1: y1 as f64 / h as f64,
    }
}

/// Locate the paper assuming the colours in `backgrounds` make up the scanner background.
fn paper_against(img: &RgbImage, backgrounds: &[Background]) -> Option<FracRect> {
    let (w, h) = (img.width() as usize, img.height() as usize);
    let is_bg = |x: usize, y: usize| {
        let p = img.get_pixel(x as u32, y as u32).0;
        backgrounds
            .iter()
            .any(|bg| distance(p, bg.color) <= bg.tolerance)
    };

    // Flood fill the background from every matching border pixel.
    let mut background = vec![false; w * h];
    let mut queue = VecDeque::new();
    for (x, y) in border_coords(w, h, 1) {
        if !background[y * w + x] && is_bg(x, y) {
            background[y * w + x] = true;
            queue.push_back((x, y));
        }
    }
    while let Some((x, y)) = queue.pop_front() {
        for (nx, ny) in neighbours(x, y, w, h) {
            let i = ny * w + nx;
            if !background[i] && is_bg(nx, ny) {
                background[i] = true;
                queue.push_back((nx, ny));
            }
        }
    }

    let foreground: Vec<bool> = background.iter().map(|b| !b).collect();
    let foreground = erode(&foreground, w, h, ERODE_RADIUS);
    let component = largest_component(&foreground, img)?;
    if (component.area as f64) < MIN_PAPER_AREA * (w * h) as f64 {
        return None;
    }
    let brightest_bg = backgrounds
        .iter()
        .map(|bg| luma(bg.color))
        .fold(0.0, f64::max);
    if component.mean_luma <= brightest_bg {
        return None;
    }

    // Undo the erosion and add one pixel of slack so the paper edge is never cut off.
    let [x0, y0, x1, y1] = component.bbox;
    let pad = ERODE_RADIUS + 1;
    let x0 = x0.saturating_sub(pad);
    let y0 = y0.saturating_sub(pad);
    let x1 = (x1 + 1 + pad).min(w);
    let y1 = (y1 + 1 + pad).min(h);

    Some(FracRect {
        x0: x0 as f64 / w as f64,
        y0: y0 as f64 / h as f64,
        x1: x1 as f64 / w as f64,
        y1: y1 as f64 / h as f64,
    })
}

/// The significant colours along the image border, darkest first, each with a matching
/// tolerance derived from its noise level.
fn background_candidates(img: &RgbImage) -> Vec<Background> {
    let (w, h) = (img.width() as usize, img.height() as usize);
    let strip = ((w.min(h) as f64 * BORDER_FRACTION).ceil() as usize).max(2);
    let mut pixels: Vec<[u8; 3]> = border_coords(w, h, strip)
        .map(|(x, y)| img.get_pixel(x as u32, y as u32).0)
        .collect();
    let min_share = (pixels.len() as f64 * MIN_CLUSTER_SHARE).max(1.0) as usize;

    // Repeatedly take the mode of a coarse colour histogram (5 bits per channel) and
    // remove the pixels around it.
    let bin = |p: [u8; 3]| {
        ((p[0] as usize >> 3) << 10) | ((p[1] as usize >> 3) << 5) | (p[2] as usize >> 3)
    };
    let mut candidates = Vec::new();
    while candidates.len() < MAX_CANDIDATES && !pixels.is_empty() {
        let mut histogram = vec![0u32; 1 << 15];
        for &p in &pixels {
            histogram[bin(p)] += 1;
        }
        let Some(mode) = (0..histogram.len()).max_by_key(|&i| histogram[i]) else {
            break;
        };
        let center = [
            ((mode >> 10) << 3) as u8 + 4,
            (((mode >> 5) & 31) << 3) as u8 + 4,
            ((mode & 31) << 3) as u8 + 4,
        ];
        let (near, rest): (Vec<[u8; 3]>, Vec<[u8; 3]>) = pixels
            .iter()
            .partition(|&&p| distance(p, center) <= CLUSTER_RADIUS);
        pixels = rest;
        if near.len() < min_share {
            break;
        }

        // Refine: mean of the border pixels near the mode.
        let mut sum = [0u64; 3];
        for p in &near {
            for c in 0..3 {
                sum[c] += p[c] as u64;
            }
        }
        let n = near.len() as u64;
        let color = [(sum[0] / n) as u8, (sum[1] / n) as u8, (sum[2] / n) as u8];

        // Noise level: median distance of the cluster's pixels to its mean.
        let mut dists: Vec<u8> = near.iter().map(|&p| distance(p, color)).collect();
        dists.sort_unstable();
        let noise = dists[dists.len() / 2] as f64;
        let tolerance = (12.0 + 4.0 * noise).clamp(20.0, 64.0) as u8;
        candidates.push(Background { color, tolerance });
    }

    candidates.sort_by(|a, b| luma(a.color).total_cmp(&luma(b.color)));
    candidates
}

fn luma(c: [u8; 3]) -> f64 {
    0.299 * c[0] as f64 + 0.587 * c[1] as f64 + 0.114 * c[2] as f64
}

/// Chebyshev distance between two colours.
fn distance(a: [u8; 3], b: [u8; 3]) -> u8 {
    (0..3).map(|c| a[c].abs_diff(b[c])).max().unwrap_or(0)
}

/// All coordinates within `strip` pixels of the image border, each visited exactly once.
fn border_coords(w: usize, h: usize, strip: usize) -> impl Iterator<Item = (usize, usize)> {
    let strip = strip.min(w / 2).min(h / 2).max(1);
    (0..h).flat_map(move |y| {
        let full_row = y < strip || y >= h - strip;
        (0..w)
            .filter(move |&x| full_row || x < strip || x >= w - strip)
            .map(move |x| (x, y))
    })
}

fn neighbours(x: usize, y: usize, w: usize, h: usize) -> impl Iterator<Item = (usize, usize)> {
    let candidates = [
        (x.wrapping_sub(1), y),
        (x + 1, y),
        (x, y.wrapping_sub(1)),
        (x, y + 1),
    ];
    candidates
        .into_iter()
        .filter(move |&(nx, ny)| nx < w && ny < h)
}

/// Binary erosion with a square structuring element, done as two separable passes.
fn erode(mask: &[bool], w: usize, h: usize, radius: usize) -> Vec<bool> {
    let pass = |src: &[bool], horizontal: bool| -> Vec<bool> {
        let mut dst = vec![false; w * h];
        for y in 0..h {
            for x in 0..w {
                let (pos, len) = if horizontal { (x, w) } else { (y, h) };
                let lo = pos.saturating_sub(radius);
                let hi = (pos + radius).min(len - 1);
                dst[y * w + x] = (lo..=hi).all(|p| {
                    if horizontal {
                        src[y * w + p]
                    } else {
                        src[p * w + x]
                    }
                });
            }
        }
        dst
    };
    pass(&pass(mask, true), false)
}

/// A connected region of the foreground mask.
struct Component {
    area: usize,
    /// Inclusive bounding box `[x0, y0, x1, y1]`.
    bbox: [usize; 4],
    /// Mean luma of the region's pixels.
    mean_luma: f64,
}

/// The largest 4-connected region of `true` pixels in `mask`, which has the size of `img`.
fn largest_component(mask: &[bool], img: &RgbImage) -> Option<Component> {
    let (w, h) = (img.width() as usize, img.height() as usize);
    let mut seen = vec![false; w * h];
    let mut best: Option<Component> = None;
    let mut queue = VecDeque::new();
    for start in 0..w * h {
        if !mask[start] || seen[start] {
            continue;
        }
        seen[start] = true;
        queue.push_back((start % w, start / w));
        let mut area = 0;
        let mut luma_sum = 0.0;
        let mut bbox = [usize::MAX, usize::MAX, 0, 0];
        while let Some((x, y)) = queue.pop_front() {
            area += 1;
            luma_sum += luma(img.get_pixel(x as u32, y as u32).0);
            bbox = [
                bbox[0].min(x),
                bbox[1].min(y),
                bbox[2].max(x),
                bbox[3].max(y),
            ];
            for (nx, ny) in neighbours(x, y, w, h) {
                let i = ny * w + nx;
                if mask[i] && !seen[i] {
                    seen[i] = true;
                    queue.push_back((nx, ny));
                }
            }
        }
        if best.as_ref().is_none_or(|b| area > b.area) {
            best = Some(Component {
                area,
                bbox,
                mean_luma: luma_sum / area as f64,
            });
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgb;

    fn synthetic_scan(bg: [u8; 3], paper: [u8; 3]) -> RgbImage {
        let mut img = RgbImage::from_pixel(400, 600, Rgb(bg));
        for y in 100..500 {
            for x in 50..300 {
                img.put_pixel(x, y, Rgb(paper));
            }
        }
        // Some "text" on the paper and a speck of dust on the background.
        for y in (150..450).step_by(20) {
            for x in 80..270 {
                img.put_pixel(x, y, Rgb([20, 20, 20]));
            }
        }
        img.put_pixel(370, 30, Rgb(paper));
        img
    }

    fn assert_close(found: FracRect, expected: FracRect) {
        let tol = 0.02;
        assert!(
            (found.x0 - expected.x0).abs() < tol
                && (found.y0 - expected.y0).abs() < tol
                && (found.x1 - expected.x1).abs() < tol
                && (found.y1 - expected.y1).abs() < tol,
            "found {found:?}, expected {expected:?}"
        );
    }

    const EXPECTED: FracRect = FracRect {
        x0: 50.0 / 400.0,
        y0: 100.0 / 600.0,
        x1: 300.0 / 400.0,
        y1: 500.0 / 600.0,
    };

    #[test]
    fn dark_background() {
        let found = find_paper(&synthetic_scan([30, 30, 30], [245, 245, 240])).unwrap();
        assert_close(found, EXPECTED);
    }

    #[test]
    fn light_background() {
        let found = find_paper(&synthetic_scan([200, 200, 200], [250, 250, 250])).unwrap();
        assert_close(found, EXPECTED);
    }

    #[test]
    fn uniform_image_has_no_paper() {
        assert_eq!(
            find_paper(&RgbImage::from_pixel(300, 300, Rgb([128, 128, 128]))),
            None
        );
    }

    #[test]
    fn paper_touching_three_edges() {
        // White paper over most of the scan, grey lid visible only at the bottom.
        let mut img = RgbImage::from_pixel(500, 800, Rgb([128, 128, 128]));
        for y in 0..650 {
            for x in 0..500 {
                img.put_pixel(x, y, Rgb([252, 252, 252]));
            }
        }
        let found = find_paper(&img).unwrap();
        assert_close(
            found,
            FracRect {
                x0: 0.0,
                y0: 0.0,
                x1: 1.0,
                y1: 650.0 / 800.0,
            },
        );
    }

    #[test]
    fn borderless_page_is_not_cropped_to_its_content() {
        // The paper fills the scan; a dark photo on it must not be mistaken for the paper.
        let mut img = RgbImage::from_pixel(500, 700, Rgb([250, 250, 250]));
        for y in 200..500 {
            for x in 100..400 {
                img.put_pixel(x, y, Rgb([40, 60, 50]));
            }
        }
        assert!(matches!(find_paper(&img), None | Some(FracRect::FULL)));
    }

    #[test]
    fn lid_with_several_tones() {
        // Like a real scan: paper touching the top, a flat grey padding area at the bottom
        // and shaded lid strips of a lighter grey on both sides.
        let mut img = RgbImage::from_pixel(510, 840, Rgb([128, 128, 128]));
        for y in 0..700 {
            for x in 0..510 {
                let v = match x {
                    0..8 => 165 - x as u8 * 3,
                    502.. => 141 + (x - 502) as u8 * 3,
                    _ => 254,
                };
                img.put_pixel(x, y, Rgb([v, v, v]));
            }
        }
        let found = find_paper(&img).unwrap();
        assert_close(
            found,
            FracRect {
                x0: 8.0 / 510.0,
                y0: 0.0,
                x1: 502.0 / 510.0,
                y1: 700.0 / 840.0,
            },
        );
    }
}
