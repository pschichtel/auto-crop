mod crop;
mod detect;
mod image_decode;
mod pdf_page;
mod simple_scan;
mod straighten;
mod unwarp;
mod whiten;

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use image::ImageFormat;
use kurbo::Rect;
use lopdf::Document;

use crate::crop::{PageOutcome, crop_document};
use crate::simple_scan::Invocation;
use crate::straighten::{Resized, Straightener, straighten_document, straighten_image};
use crate::unwarp::{Model, Unwarper};

/// Crop every scanned page of a PDF to the sheet of paper it shows.
///
/// Pages that do not consist of a scanned image are copied unchanged. The input and output
/// may be the same file.
///
/// Can also be used directly as a postprocessing script in GNOME Document Scanner
/// (simple-scan); its calling convention is detected automatically.
#[derive(Parser)]
#[command(
    version,
    args_conflicts_with_subcommands = true,
    subcommand_negates_reqs = true
)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
    /// PDF file to read
    #[arg(required = true)]
    input: Option<PathBuf>,
    /// PDF file to write
    #[arg(required = true)]
    output: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Command {
    /// Straighten a photographed or crooked document with a neural network (UVDoc by default).
    ///
    /// Works on image files (JPEG, PNG, TIFF, WebP) and on the scanned pages of PDFs.
    /// Rotation, perspective and curled paper are corrected and the result is cropped to
    /// the document. Unlike plain cropping, this resamples and re-encodes the image.
    ///
    /// The models are built in.
    Straighten {
        /// Image or PDF file to read
        input: PathBuf,
        /// File to write; images are encoded according to its extension
        output: PathBuf,
        /// Neural network that locates the document
        #[arg(long, value_enum, default_value_t)]
        model: Model,
        /// Also even out the lighting, so that shading and fold shadows disappear and
        /// the paper becomes white. For drawings and text on light paper; photos and
        /// large dark areas on the page get washed out
        #[arg(long)]
        whiten: bool,
    },
}

fn main() -> Result<()> {
    let raw_args: Vec<OsString> = std::env::args_os().skip(1).collect();
    if let Some(invocation) = Invocation::from_args(&raw_args) {
        if invocation.mime_type != "application/pdf" {
            eprintln!(
                "{}: not a PDF ({}), leaving it unchanged",
                invocation.file.display(),
                invocation.mime_type
            );
            return Ok(());
        }
        return crop_file(
            &invocation.file,
            &invocation.file,
            invocation.backup().as_deref(),
        );
    }

    let args = Args::parse();
    match (args.command, args.input, args.output) {
        (
            Some(Command::Straighten {
                input,
                output,
                model,
                whiten,
            }),
            ..,
        ) => straighten_file(&input, &output, model, whiten),
        (None, Some(input), Some(output)) => crop_file(&input, &output, None),
        _ => unreachable!("clap requires both paths without a subcommand"),
    }
}

/// Crop `input` into `output`. If `backup` is given, the input is copied there before
/// it gets overwritten.
fn crop_file(input: &Path, output: &Path, backup: Option<&Path>) -> Result<()> {
    let mut doc =
        Document::load(input).with_context(|| format!("failed to read {}", input.display()))?;

    let mut cropped = 0;
    for (number, outcome) in crop_document(&mut doc) {
        match outcome {
            PageOutcome::Cropped { from, to } => {
                cropped += 1;
                eprintln!(
                    "page {number}: cropped {} -> {}",
                    pdf_box(from),
                    pdf_box(to)
                );
            }
            PageOutcome::NotAPicture => eprintln!("page {number}: skipped, not a scanned page"),
            PageOutcome::NoPaperFound => eprintln!("page {number}: skipped, no paper detected"),
            PageOutcome::NothingToCrop => eprintln!("page {number}: already tightly cropped"),
            PageOutcome::Failed(e) => eprintln!("page {number}: skipped, {e:#}"),
        }
    }

    if cropped == 0 && input == output {
        eprintln!("nothing cropped, {} left unchanged", input.display());
        return Ok(());
    }
    if let Some(backup) = backup {
        std::fs::copy(input, backup)
            .with_context(|| format!("failed to keep the original as {}", backup.display()))?;
        eprintln!("original kept as {}", backup.display());
    }
    save_replacing(&mut doc, output)
        .with_context(|| format!("failed to write {}", output.display()))?;
    eprintln!("cropped {cropped} page(s), written to {}", output.display());
    Ok(())
}

/// Straighten the image or PDF `input` into `output`.
fn straighten_file(input: &Path, output: &Path, model: Model, whiten: bool) -> Result<()> {
    let data =
        std::fs::read(input).with_context(|| format!("failed to read {}", input.display()))?;
    let is_pdf = data.starts_with(b"%PDF");
    // Settle everything cheap before loading the model.
    let format = if is_pdf {
        None
    } else {
        Some(ImageFormat::from_path(output).with_context(|| {
            format!(
                "cannot tell the image format of {} from its extension",
                output.display()
            )
        })?)
    };
    let straightener = Straightener {
        unwarper: Unwarper::new(model)?,
        whiten,
    };
    let size = |(w, h): (u32, u32)| format!("{w}x{h} px");

    let Some(format) = format else {
        let mut doc = Document::load_mem(&data)
            .with_context(|| format!("failed to read {}", input.display()))?;
        let mut straightened = 0;
        for (number, outcome) in straighten_document(&straightener, &mut doc) {
            match outcome {
                straighten::PageOutcome::Straightened(Resized { from, to }) => {
                    straightened += 1;
                    eprintln!("page {number}: straightened {} -> {}", size(from), size(to));
                }
                straighten::PageOutcome::NotAPicture => {
                    eprintln!("page {number}: skipped, not a scanned page")
                }
                straighten::PageOutcome::Failed(e) => eprintln!("page {number}: skipped, {e:#}"),
            }
        }
        if straightened == 0 && input == output {
            eprintln!("nothing straightened, {} left unchanged", input.display());
            return Ok(());
        }
        save_replacing(&mut doc, output)
            .with_context(|| format!("failed to write {}", output.display()))?;
        eprintln!(
            "straightened {straightened} page(s), written to {}",
            output.display()
        );
        return Ok(());
    };

    let (encoded, Resized { from, to }) = straighten_image(&straightener, &data, format)
        .with_context(|| format!("failed to straighten {}", input.display()))?;
    write_replacing(output, |tmp| Ok(std::fs::write(tmp, &encoded)?))
        .with_context(|| format!("failed to write {}", output.display()))?;
    eprintln!(
        "straightened {} -> {}, written to {}",
        size(from),
        size(to),
        output.display()
    );
    Ok(())
}

/// Save the document without ever leaving `path` half written.
fn save_replacing(doc: &mut Document, path: &Path) -> Result<()> {
    write_replacing(path, |tmp| Ok(doc.save(tmp).map(drop)?))
}

/// Let `write` create a temporary file next to `path` and move it into place, so an
/// existing file (possibly the input itself) is never left half written.
fn write_replacing(path: &Path, write: impl FnOnce(&Path) -> Result<()>) -> Result<()> {
    let name = path.file_name().context("output path has no file name")?;
    let mut tmp_name = OsString::from(".");
    tmp_name.push(name);
    tmp_name.push(".auto-crop-tmp");
    let tmp = path.with_file_name(tmp_name);

    let result = write(&tmp).and_then(|_| Ok(std::fs::rename(&tmp, path)?));
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

/// Format a rectangle the way PDF writes boxes: `[llx lly urx ury]`.
fn pdf_box(r: Rect) -> String {
    format!("[{:.1} {:.1} {:.1} {:.1}]", r.x0, r.y0, r.x1, r.y1)
}
