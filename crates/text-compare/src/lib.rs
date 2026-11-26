//! Text Compare's pane documents (story E07-S1), without a UI: a file read through the data-source
//! layer (E04), decoded with its encoding and line-ending style detected, and the facts its file
//! info header shows. The view that shows them is `otd-ui-text-compare`.
//!
//! - [`load`] reads one file from any [`DataSource`]; [`open`] reads a spec as people type it (an
//!   OS path, `~/…` or `smb://…`) through the local source.
//! - [`decode`] detects the encoding (a byte-order mark, valid UTF-8, UTF-16 without a mark, or
//!   the system code page) and splits the lines, keeping each line's ending.

mod decode;

use std::io::Read;
use std::path::Path;
use std::time::SystemTime;

pub use decode::{Decoded, Encoding, LineEndings, WESTERN, decode, system_code_page};
use otd_format::{Case, Registry};
use otd_source::{DataSource, ErrorKind, ItemKind, Links, LocalSource, SourceError};
pub use otd_text_engine::{Line, LineEnding};

/// One side's file: its lines and what its file info header shows.
#[derive(Clone, Debug, PartialEq)]
pub struct Pane {
    /// The spec as given, for the header.
    pub path: String,
    /// The file's size in bytes, as read.
    pub size: u64,
    pub modified: Option<SystemTime>,
    /// The name of the file format it resolved to.
    pub format: String,
    pub encoding: Encoding,
    pub endings: LineEndings,
    pub lines: Vec<Line>,
}

/// Reads the file at `path` in `source`, shown as `spec`. Its format is the first enabled one in
/// `formats` whose mask matches its name (compared with the source's case), else the fallback for
/// its content. Text that is neither UTF-8 nor UTF-16 decodes in `code_page`.
// ponytail: the whole file is read into memory on the caller's thread; a background load with
// progress comes when large files need one.
pub fn load(source: &dyn DataSource, path: &Path, spec: &str, formats: &Registry, code_page: u16) -> Result<Pane, SourceError> {
    let item = source.stat(path, Links::Follow)?;
    if let Some(error) = item.error {
        return Err(error);
    }
    if item.kind != ItemKind::File {
        return Err(SourceError::new(ErrorKind::Unsupported, format!("{spec} isn't a file")));
    }
    let mut bytes = Vec::new();
    source.open_read(path)?.read_to_end(&mut bytes).map_err(|e| SourceError::new(ErrorKind::Io, format!("{spec}: {e}")))?;
    let case = if source.capabilities().case_sensitive { Case::Sensitive } else { Case::Insensitive };
    let name = path.file_name().map(|name| name.to_string_lossy()).unwrap_or_default();
    let format = formats.resolve(&name, case).unwrap_or_else(|| formats.fallback(&bytes)).name.clone();
    let Decoded { encoding, endings, lines } = decode(&bytes, code_page);
    Ok(Pane {
        path: spec.to_owned(),
        size: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
        modified: item.modified,
        format,
        encoding,
        endings,
        lines,
    })
}

/// Reads the file `spec` names on this machine (an OS path, `~/…` or `smb://…` for a mounted
/// share), in the system code page when it isn't Unicode.
pub fn open(spec: &str, formats: &Registry) -> Result<Pane, SourceError> {
    let path = Path::new(spec);
    let name = path.file_name().ok_or_else(|| SourceError::new(ErrorKind::NotFound, format!("{spec} doesn't name a file")))?;
    let folder = path.parent().and_then(Path::to_str).unwrap_or_default();
    let source = LocalSource::open(if folder.is_empty() { "." } else { folder })?;
    load(&source, Path::new(name), spec, formats, system_code_page())
}
