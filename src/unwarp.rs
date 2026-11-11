//! Straightening a document image with the UVDoc neural network.
//!
//! [UVDoc](https://github.com/tanguymagne/UVDoc) looks at a downscaled copy of the image
//! and predicts a coarse grid of points: where a regular grid laid over the flat document
//! ends up in the image. Resampling the full resolution image along that grid undoes
//! rotation, perspective and paper curl, and crops to the document at the same time.

use std::sync::Arc;

use anyhow::{Context, Result, bail};
use image::{Rgb, RgbImage, imageops};
use rayon::prelude::*;
use tract_onnx::prelude::*;

/// The models built into the binary, as ONNX exports that output the grid. See
/// `models/README.md` for their origin and licences.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum Model {
    /// UVDoc (2023): 45x31 grid, fastest
    #[default]
    Uvdoc,
    /// SyntheticDoc (2026): UVDoc's successor with a finer grid from a larger input;
    /// crops tighter, but can trim the edge of the sheet
    Syntheticdoc,
}

impl Model {
    fn onnx(self) -> &'static [u8] {
        match self {
            Model::Uvdoc => include_bytes!("../models/UVDoc_grid.onnx"),
            Model::Syntheticdoc => include_bytes!("../models/SyntheticDoc_121x177.onnx"),
        }
    }

    /// Map an 8-bit colour value to the range the model was trained on.
    fn normalize(self, value: u8) -> f32 {
        match self {
            Model::Uvdoc => value as f32 / 255.0,
            Model::Syntheticdoc => value as f32 / 127.5 - 1.0,
        }
    }
}

/// Margins, in image pixels, to add around an image before showing it to the model.
///
/// UVDoc expects to see background all around the document. Where the document runs to
/// the edge of the image, as paper pushed against the edge of a flatbed scanner does, the
/// model otherwise looks for the document's edge further inside and cuts content off.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Margins {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    /// Colour to fill the margins with, ideally that of the background.
    pub color: [u8; 3],
}

/// A document unwarping model, ready to run.
pub struct Unwarper {
    kind: Model,
    model: Arc<TypedRunnableModel>,
    /// Input size the model expects.
    width: u32,
    height: u32,
}

impl Unwarper {
    /// Load one of the built-in models. Each maps an RGB image `[1, 3, H, W]` to a grid
    /// of normalized source coordinates `[1, 2, rows, columns]` in `-1..=1`.
    pub fn new(kind: Model) -> Result<Unwarper> {
        let model = tract_onnx::onnx()
            .model_for_read(&mut kind.onnx())
            .and_then(|m| m.into_optimized())
            .context("failed to load the built-in model")?;
        let input = model.input_fact(0)?;
        let &[1, 3, height, width] = input
            .shape
            .as_concrete()
            .context("the model's input size must be fixed")?
        else {
            bail!("unexpected model input shape {:?}", input.shape);
        };
        Ok(Unwarper {
            kind,
            model: model.into_runnable()?,
            width: width as u32,
            height: height as u32,
        })
    }

    /// Predict where the document lies in `img`, which is shown to the model with the
    /// given margins added around it.
    pub fn predict(&self, img: &RgbImage, margins: &Margins) -> Result<Grid> {
        // The model was trained on images squeezed to its input size, ignoring the aspect
        // ratio. Only this small copy is padded, never the full image.
        let canvas_width = img.width() + margins.left + margins.right;
        let canvas_height = img.height() + margins.top + margins.bottom;
        let scale =
            |v: u32, len: u32, canvas: u32| (v as f64 * len as f64 / canvas as f64).round() as u32;
        let x0 = scale(margins.left, self.width, canvas_width);
        let x1 = scale(margins.left + img.width(), self.width, canvas_width).max(x0 + 1);
        let y0 = scale(margins.top, self.height, canvas_height);
        let y1 = scale(margins.top + img.height(), self.height, canvas_height).max(y0 + 1);
        let mut small = RgbImage::from_pixel(self.width, self.height, Rgb(margins.color));
        let inner = imageops::resize(img, x1 - x0, y1 - y0, imageops::FilterType::Triangle);
        imageops::replace(&mut small, &inner, x0 as i64, y0 as i64);

        let (w, h) = (self.width as usize, self.height as usize);
        let input = tract_ndarray::Array4::from_shape_fn((1, 3, h, w), |(_, c, y, x)| {
            self.kind.normalize(small.get_pixel(x as u32, y as u32)[c])
        });
        let outputs = self.model.run(tvec!(Tensor::from(input).into()))?;
        let output = outputs[0].to_plain_array_view::<f32>()?;
        let &[1, 2, rows, columns] = output.shape() else {
            bail!("unexpected model output shape {:?}", output.shape());
        };
        if rows < 2 || columns < 2 {
            bail!("the model's grid is too small");
        }

        // Normalized coordinates follow the `align_corners` convention: -1 and 1 are the
        // centres of the first and last pixel of the padded image. Points the model puts
        // into a margin are pulled back onto the image.
        let to_px = |v: f32, canvas: u32, margin: u32, len: u32| {
            let on_canvas = (v as f64 + 1.0) / 2.0 * (canvas - 1) as f64;
            (on_canvas - margin as f64).clamp(0.0, (len - 1) as f64)
        };
        let points = (0..rows * columns)
            .map(|i| {
                let (r, c) = (i / columns, i % columns);
                [
                    to_px(
                        output[[0, 0, r, c]],
                        canvas_width,
                        margins.left,
                        img.width(),
                    ),
                    to_px(
                        output[[0, 1, r, c]],
                        canvas_height,
                        margins.top,
                        img.height(),
                    ),
                ]
            })
            .collect();
        Ok(Grid {
            rows,
            columns,
            points,
        })
    }
}

/// Where a regular grid on the flat document lies in the image, in pixel coordinates,
/// row by row from the document's top left corner.
pub struct Grid {
    rows: usize,
    columns: usize,
    points: Vec<[f64; 2]>,
}

impl Grid {
    fn point(&self, row: usize, column: usize) -> [f64; 2] {
        self.points[row * self.columns + column]
    }

    /// Size of the document in image pixels, measured along the grid lines.
    ///
    /// For a flat scan this is exact, so the result keeps the scan's resolution. For a
    /// photo it is an estimate, as perspective shortens the far side of the document.
    pub fn document_size(&self) -> (u32, u32) {
        let length = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).hypot(a[1] - b[1]);
        let width: f64 = (0..self.rows)
            .map(|r| {
                (1..self.columns)
                    .map(|c| length(self.point(r, c - 1), self.point(r, c)))
                    .sum::<f64>()
            })
            .sum::<f64>()
            / self.rows as f64;
        let height: f64 = (0..self.columns)
            .map(|c| {
                (1..self.rows)
                    .map(|r| length(self.point(r - 1, c), self.point(r, c)))
                    .sum::<f64>()
            })
            .sum::<f64>()
            / self.columns as f64;
        // Grid points are pixel centres, so n pixels span a length of n - 1.
        (
            (width.round() as u32 + 1).max(2),
            (height.round() as u32 + 1).max(2),
        )
    }

    /// The grid points of the (fractional) grid row `y`, interpolated between rows.
    fn row_at(&self, y: f64) -> Vec<[f64; 2]> {
        let (r, weights) = cubic_weights(y, self.rows);
        (0..self.columns)
            .map(|c| {
                let column = |i: usize| self.point(i, c);
                combine(
                    std::array::from_fn(|k| extended(self.rows, r + k as isize - 1, column)),
                    weights,
                )
            })
            .collect()
    }

    /// Resample `img` so that the document fills the result, flat and upright.
    pub fn unwarp(&self, img: &RgbImage) -> RgbImage {
        let (width, height) = self.document_size();
        let mut out = RgbImage::new(width, height);
        let row_len = width as usize * 3;
        out.par_chunks_mut(row_len)
            .enumerate()
            .for_each(|(v, row)| {
                // Interpolate the grid smoothly (Catmull-Rom), first between rows, then
                // along the row. Affine maps such as a plain rotation are reproduced exactly.
                let gy = v as f64 * (self.rows - 1) as f64 / (height - 1) as f64;
                let line = self.row_at(gy);
                for (u, pixel) in row.chunks_exact_mut(3).enumerate() {
                    let gx = u as f64 * (self.columns - 1) as f64 / (width - 1) as f64;
                    let (c, weights) = cubic_weights(gx, self.columns);
                    let points = std::array::from_fn(|k| {
                        extended(line.len(), c + k as isize - 1, |i| line[i])
                    });
                    let [x, y] = combine(points, weights);
                    pixel.copy_from_slice(&sample(img, x, y).0);
                }
            });
        out
    }
}

/// Catmull-Rom weights for position `t` on a line of `len` points: the index of the
/// point before `t` and the weights of the four points around it.
fn cubic_weights(t: f64, len: usize) -> (isize, [f64; 4]) {
    let i = (t.floor() as isize).clamp(0, len as isize - 2);
    let f = t - i as f64;
    let (f2, f3) = (f * f, f * f * f);
    (
        i,
        [
            (-f3 + 2.0 * f2 - f) / 2.0,
            (3.0 * f3 - 5.0 * f2 + 2.0) / 2.0,
            (-3.0 * f3 + 4.0 * f2 + f) / 2.0,
            (f3 - f2) / 2.0,
        ],
    )
}

/// Point `index` of a line of `len` points, with one virtual point of linear
/// extrapolation beyond each end.
fn extended(len: usize, index: isize, at: impl Fn(usize) -> [f64; 2]) -> [f64; 2] {
    let mirror = |a: [f64; 2], b: [f64; 2]| [2.0 * a[0] - b[0], 2.0 * a[1] - b[1]];
    if index < 0 {
        mirror(at(0), at(1))
    } else if index as usize >= len {
        mirror(at(len - 1), at(len - 2))
    } else {
        at(index as usize)
    }
}

fn combine(points: [[f64; 2]; 4], weights: [f64; 4]) -> [f64; 2] {
    let mut out = [0.0; 2];
    for (p, w) in points.iter().zip(weights) {
        out[0] += p[0] * w;
        out[1] += p[1] * w;
    }
    out
}

/// Bicubic (Catmull-Rom) sample of `img` at a fractional position, repeating edge pixels.
fn sample(img: &RgbImage, x: f64, y: f64) -> Rgb<u8> {
    let (w, h) = (img.width() as isize, img.height() as isize);
    let x = x.clamp(0.0, (w - 1) as f64);
    let y = y.clamp(0.0, (h - 1) as f64);
    let (x0, wx) = cubic_weights(x, w.max(2) as usize);
    let (y0, wy) = cubic_weights(y, h.max(2) as usize);
    let mut acc = [0.0f64; 3];
    for (j, wy) in wy.iter().enumerate() {
        let py = (y0 + j as isize - 1).clamp(0, h - 1) as u32;
        for (i, wx) in wx.iter().enumerate() {
            let px = (x0 + i as isize - 1).clamp(0, w - 1) as u32;
            let p = img.get_pixel(px, py);
            let weight = wx * wy;
            for c in 0..3 {
                acc[c] += p[c] as f64 * weight;
            }
        }
    }
    Rgb(acc.map(|v| v.round().clamp(0.0, 255.0) as u8))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A grid describing a document rotated by `angle` around the image centre.
    fn rotated_grid(
        rows: usize,
        columns: usize,
        size: (f64, f64),
        centre: (f64, f64),
        angle: f64,
    ) -> Grid {
        let (sin, cos) = angle.sin_cos();
        let points = (0..rows * columns)
            .map(|i| {
                let x = ((i % columns) as f64 / (columns - 1) as f64 - 0.5) * size.0;
                let y = ((i / columns) as f64 / (rows - 1) as f64 - 0.5) * size.1;
                [centre.0 + x * cos - y * sin, centre.1 + x * sin + y * cos]
            })
            .collect();
        Grid {
            rows,
            columns,
            points,
        }
    }

    #[test]
    fn document_size_follows_the_grid() {
        let grid = rotated_grid(45, 31, (300.0, 500.0), (400.0, 400.0), 0.3);
        assert_eq!(grid.document_size(), (301, 501));
    }

    #[test]
    fn unwarp_undoes_a_rotation() {
        // A white 200x300 "document" with a black top left quarter, rotated by 20 degrees.
        let (sin, cos) = 20f64.to_radians().sin_cos();
        let img = RgbImage::from_fn(600, 600, |x, y| {
            let (dx, dy) = (x as f64 - 300.0, y as f64 - 300.0);
            let (u, v) = (dx * cos + dy * sin, -dx * sin + dy * cos);
            if u.abs() > 100.0 || v.abs() > 150.0 {
                Rgb([128, 128, 128])
            } else if u < 0.0 && v < 0.0 {
                Rgb([0, 0, 0])
            } else {
                Rgb([255, 255, 255])
            }
        });
        let grid = rotated_grid(45, 31, (200.0, 300.0), (300.0, 300.0), 20f64.to_radians());
        let out = grid.unwarp(&img);
        assert_eq!(out.dimensions(), (201, 301));
        assert_eq!(out.get_pixel(50, 75).0, [0, 0, 0]);
        assert_eq!(out.get_pixel(150, 75).0, [255, 255, 255]);
        assert_eq!(out.get_pixel(50, 225).0, [255, 255, 255]);
        assert_eq!(out.get_pixel(150, 225).0, [255, 255, 255]);
        // No background left, apart from the antialiased outline.
        let grey = out.pixels().filter(|p| p.0 == [128, 128, 128]).count();
        assert!(grey < 2 * (201 + 301), "{grey} background pixels left");
    }

    #[test]
    fn embedded_model_flattens_a_photographed_page() {
        // A photo of a curled magazine page from the UVDoc repository.
        let img = image::open("tests/data/warped-page.jpg")
            .unwrap()
            .into_rgb8();
        assert_eq!(img.dimensions(), (277, 384));
        let grid = Unwarper::new(Model::Uvdoc)
            .unwrap()
            .predict(&img, &Margins::default())
            .unwrap();
        assert_eq!((grid.rows, grid.columns), (45, 31));
        // The page fills most of the photo; the model's estimate is stable to a few pixels.
        let (width, height) = grid.document_size();
        assert!((235..=260).contains(&width), "width {width}");
        assert!((360..=384).contains(&height), "height {height}");
        // The page's top left corner lies near the photo's, its bottom right near the photo's.
        let [x0, y0] = grid.point(0, 0);
        let [x1, y1] = grid.point(44, 30);
        assert!(x0 < 40.0 && y0 < 30.0, "top left at {x0}, {y0}");
        assert!(x1 > 240.0 && y1 > 350.0, "bottom right at {x1}, {y1}");
    }

    #[test]
    fn syntheticdoc_model_flattens_a_photographed_page() {
        let img = image::open("tests/data/warped-page.jpg")
            .unwrap()
            .into_rgb8();
        let unwarper = Unwarper::new(Model::Syntheticdoc).unwrap();
        let grid = unwarper.predict(&img, &Margins::default()).unwrap();
        assert_eq!((grid.rows, grid.columns), (121, 177));
        // This model crops tighter than UVDoc and trims the page's lower edge.
        let (width, height) = grid.document_size();
        assert!((230..=260).contains(&width), "width {width}");
        assert!((315..=384).contains(&height), "height {height}");
    }
}
