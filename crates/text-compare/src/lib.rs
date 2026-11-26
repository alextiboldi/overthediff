//! Text Compare's pane documents (story E07-S1), without a UI: a file read through the data-source
//! layer (E04), decoded with its encoding and line-ending style detected, and the facts its file
//! info header shows. The view that shows them is `otd-ui-text-compare`.
//!
//! - [`load`] reads one file from any [`DataSource`]; [`open`] reads a spec as people type it (an
//!   OS path, `~/…` or `smb://…`) through the local source.
//! - [`decode`] detects the encoding (a byte-order mark, valid UTF-8, UTF-16 without a mark, or
//!   the system code page) and splits the lines, keeping each line's ending.
//! - A [`Pane`] is edited through its [`Selection`]: typing, pasting and deleting replace it, and
//!   its [`History`] undoes and redoes each edit (E07-S1-T5).
//! - [`save`] writes a pane back through a data source, and [`save_as`] to a spec, in the pane's
//!   own encoding and line endings ([`encode`]), so only the edited bytes change (E07-S1-T6).
//! - [`visible`] works out which aligned rows a display [`Filter`] shows (E07-S3-T1).
//! - [`copy_selection`], [`copy_section`] and [`copy_lines`] work out which lines a copy to the
//!   other side takes and which target lines they replace (E07-S7-T1).

mod copy;
mod decode;
mod edit;
mod filter;

use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub use copy::{CopyLines, copy_lines, copy_section, copy_selection};
pub use decode::{Decoded, Encoding, LineEndings, Unencodable, WESTERN, decode, encode, system_code_page};
pub use edit::{History, Pos, Selection};
pub use filter::{Filter, visible};
use otd_format::{Case, Registry};
use otd_source::{Attributes, DataSource, ErrorKind, Item, ItemKind, Links, LocalSource, SourceError};
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
    /// Why the pane can't be edited, if it can't. [`load`] sets [`ReadOnly::Encoding`] whenever
    /// the file reads with loss, else [`ReadOnly::Location`] or [`ReadOnly::File`].
    pub read_only: Option<ReadOnly>,
    /// It was edited since it was loaded or saved. Every edit sets it, Undo included, and a save
    /// clears it.
    pub unsaved: bool,
    /// The caret and the selected text (E07-S1-T5).
    pub selection: Selection,
    /// The edits Undo and Redo walk through.
    pub history: History,
}

/// Why a pane is read-only. The view shows the reason in the pane.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadOnly {
    /// The data source refuses writes ([`otd_source::Capabilities::read_only`]).
    Location,
    /// The file itself is marked read-only (no write permission, or the read-only attribute).
    File,
    /// Some of the file's bytes don't decode in its encoding, so saving would change them.
    Encoding,
    /// The comparison stopped with an error.
    ComparisonFailed,
    /// The comparison was cancelled.
    ComparisonCancelled,
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
    // What decoded with a substitute (U+FFFD) or a second spelling wouldn't save as it was.
    let lossless = encode(&lines, encoding).is_ok_and(|encoded| encoded == bytes);
    // Encoding wins: it is the one reason Save File As can't lift, and save checks it.
    let read_only = if !lossless {
        Some(ReadOnly::Encoding)
    } else if source.capabilities().read_only {
        Some(ReadOnly::Location)
    } else {
        marked_read_only(&item).then_some(ReadOnly::File)
    };
    Ok(Pane {
        path: spec.to_owned(),
        size: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
        modified: item.modified,
        format,
        encoding,
        endings,
        lines,
        read_only,
        unsaved: false,
        selection: Selection::default(),
        history: History::default(),
    })
}

/// Reads the file `spec` names on this machine (an OS path, `~/…` or `smb://…` for a mounted
/// share), in the system code page when it isn't Unicode.
pub fn open(spec: &str, formats: &Registry) -> Result<Pane, SourceError> {
    let (source, name) = local(spec)?;
    load(&source, &name, spec, formats, system_code_page())
}

/// The local source of the folder `spec` names a file in, and the file's name there.
fn local(spec: &str) -> Result<(LocalSource, PathBuf), SourceError> {
    // Resolved whole, so an `smb://` spec's file name is percent-decoded like its folders.
    let path = otd_source::resolve(spec)?;
    let name = path.file_name().ok_or_else(|| SourceError::new(ErrorKind::NotFound, format!("{spec} doesn't name a file")))?;
    let folder = path.parent().filter(|folder| !folder.as_os_str().is_empty()).unwrap_or(Path::new("."));
    Ok((LocalSource::new(folder), PathBuf::from(name)))
}

/// Whether the file is marked read-only: no write permission at all on Unix, the read-only
/// attribute on Windows. Replacing a file works around both on some systems, so a save checks.
fn marked_read_only(item: &Item) -> bool {
    const FILE_ATTRIBUTE_READONLY: u32 = 1;
    match &item.attributes {
        Some(Attributes::Unix { mode, .. }) => mode & 0o222 == 0,
        Some(Attributes::Windows(attributes)) => attributes & FILE_ATTRIBUTE_READONLY != 0,
        None => false,
    }
}

/// Why a pane wasn't saved. Nothing was written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SaveError {
    /// A character its encoding can't hold.
    Unencodable(Unencodable),
    /// The pane is [`ReadOnly::Encoding`]: saving would change bytes nobody edited.
    Lossy,
    /// The file there is marked read-only.
    MarkedReadOnly,
    /// The source refused or failed the write.
    Source(SourceError),
}

/// Writes `pane` to `path` in `source` in its own encoding, byte-order mark and line endings, so
/// only edited lines' bytes change. The source writes crash-safely ([`DataSource::write`]). Then the
/// pane takes the file's new size and modified time and is no longer unsaved.
pub fn save(pane: &mut Pane, source: &dyn DataSource, path: &Path) -> Result<(), SaveError> {
    if pane.read_only == Some(ReadOnly::Encoding) {
        return Err(SaveError::Lossy);
    }
    let bytes = encode(&pane.lines, pane.encoding).map_err(SaveError::Unencodable)?;
    // A missing file is a new one; any other stat error is the write's to report.
    if source.stat(path, Links::Follow).is_ok_and(|item| marked_read_only(&item)) {
        return Err(SaveError::MarkedReadOnly);
    }
    source.write(path, &mut bytes.as_slice()).map_err(SaveError::Source)?;
    pane.size = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    pane.modified = source.stat(path, Links::Follow).ok().and_then(|item| item.modified);
    pane.unsaved = false;
    Ok(())
}

/// Saves `pane` to the file `spec` names on this machine (as [`open`] reads it): *Save File* with
/// the pane's own path, *Save File As* with another. The pane then shows that file, so one that was
/// read-only for its old location or file can be edited.
pub fn save_as(pane: &mut Pane, spec: &str) -> Result<(), SaveError> {
    let (source, name) = local(spec).map_err(SaveError::Source)?;
    save(pane, &source, &name)?;
    spec.clone_into(&mut pane.path);
    if matches!(pane.read_only, Some(ReadOnly::Location | ReadOnly::File)) {
        pane.read_only = None;
    }
    Ok(())
}
