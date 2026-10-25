mod crop;
mod detect;
mod image_decode;
mod simple_scan;

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use kurbo::Rect;
use lopdf::Document;

use crate::crop::{PageOutcome, crop_document};
use crate::simple_scan::Invocation;

/// Crop every scanned page of a PDF to the sheet of paper it shows.
///
/// Pages that do not consist of a scanned image are copied unchanged. The input and output
/// may be the same file.
///
/// Can also be used directly as a postprocessing script in GNOME Document Scanner
/// (simple-scan); its calling convention is detected automatically.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// PDF file to read
    input: PathBuf,
    /// PDF file to write
    output: PathBuf,
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
    crop_file(&args.input, &args.output, None)
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

/// Write the document to a temporary file next to `path` and move it into place, so an
/// existing file (possibly the input itself) is never left half written.
fn save_replacing(doc: &mut Document, path: &Path) -> Result<()> {
    let name = path.file_name().context("output path has no file name")?;
    let mut tmp_name = OsString::from(".");
    tmp_name.push(name);
    tmp_name.push(".auto-crop-tmp");
    let tmp = path.with_file_name(tmp_name);

    let result = doc
        .save(&tmp)
        .map_err(anyhow::Error::from)
        .and_then(|_| std::fs::rename(&tmp, path).map_err(anyhow::Error::from));
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

/// Format a rectangle the way PDF writes boxes: `[llx lly urx ury]`.
fn pdf_box(r: Rect) -> String {
    format!("[{:.1} {:.1} {:.1} {:.1}]", r.x0, r.y0, r.x1, r.y1)
}
