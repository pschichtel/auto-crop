//! Support for running as a postprocessing script of GNOME Document Scanner (simple-scan).
//!
//! simple-scan calls the script on the file it just saved as
//!
//! ```text
//! <script> <mime type> <keep original: true|false> <file> <extra arguments>
//! ```
//!
//! where the extra arguments are passed as a single, possibly empty, argument. The
//! script's output only ends up in simple-scan's debug log and a non-zero exit status
//! merely logs a warning.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// A postprocessing request from simple-scan.
#[derive(Debug, PartialEq)]
pub struct Invocation {
    pub mime_type: String,
    pub keep_original: bool,
    pub file: PathBuf,
}

impl Invocation {
    /// Recognize simple-scan's calling convention. `args` excludes the program name.
    pub fn from_args(args: &[OsString]) -> Option<Invocation> {
        let [mime_type, keep_original, file, rest @ ..] = args else {
            return None;
        };
        if rest.len() > 1 {
            return None;
        }
        let mime_type = mime_type.to_str()?;
        let is_mime = mime_type.split_once('/').is_some_and(|(kind, sub)| {
            [kind, sub].iter().all(|part| {
                !part.is_empty()
                    && part
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || "+-.".contains(c))
            })
        });
        let keep_original = match keep_original.to_str()? {
            "true" => true,
            "false" => false,
            _ => return None,
        };
        is_mime.then(|| Invocation {
            mime_type: mime_type.to_owned(),
            keep_original,
            file: PathBuf::from(file),
        })
    }

    /// Where to keep a copy of the original if requested. The cropped document always
    /// replaces the file itself.
    pub fn backup(&self) -> Option<PathBuf> {
        self.keep_original.then(|| original_sibling(&self.file))
    }
}

/// `scan.pdf` -> `scan-original.pdf`
fn original_sibling(file: &Path) -> PathBuf {
    let stem = file.file_stem().unwrap_or_default().to_string_lossy();
    let name = match file.extension() {
        Some(ext) => format!("{stem}-original.{}", ext.to_string_lossy()),
        None => format!("{stem}-original"),
    };
    file.with_file_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<OsString> {
        list.iter().map(OsString::from).collect()
    }

    #[test]
    fn recognizes_simple_scan_arguments() {
        let expected = Invocation {
            mime_type: "application/pdf".into(),
            keep_original: true,
            file: "/home/me/Scan.pdf".into(),
        };
        let with_extra = args(&["application/pdf", "true", "/home/me/Scan.pdf", ""]);
        assert_eq!(Invocation::from_args(&with_extra), Some(expected));
        let without_extra = args(&["image/jpeg", "false", "Scan.jpg"]);
        assert!(Invocation::from_args(&without_extra).is_some());
    }

    #[test]
    fn ignores_regular_usage() {
        assert_eq!(Invocation::from_args(&args(&["in.pdf", "out.pdf"])), None);
        assert_eq!(
            Invocation::from_args(&args(&["a/b.pdf", "c.pdf", "d.pdf"])),
            None
        );
        assert_eq!(Invocation::from_args(&args(&["--help"])), None);
    }

    #[test]
    fn backup_location() {
        let mut invocation = Invocation {
            mime_type: "application/pdf".into(),
            keep_original: false,
            file: "/scans/Scan 1.pdf".into(),
        };
        assert_eq!(invocation.backup(), None);
        invocation.keep_original = true;
        assert_eq!(
            invocation.backup(),
            Some(PathBuf::from("/scans/Scan 1-original.pdf"))
        );
    }
}
