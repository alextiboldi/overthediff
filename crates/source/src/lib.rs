//! The data-source contract (E04's behavioral contract): one trait every provider implements, so
//! views, operations and scripts treat local disks, network shares, archives and later providers
//! the same way.
//!
//! - [`DataSource`] is the contract, and [`copy`] copies within one source or across two.
//! - [`MemorySource`] is the in-memory reference source, and the fixture other crates' tests use.
//! - [`LocalSource`] is the local provider: local disks and mounted network shares.
//! - [`suite`] is the contract test suite. Every provider runs it.
//!
//! Paths are relative to the source's root, with `/` between names (`docs/readme.txt`); the empty
//! path is the root. Each provider maps them onto its own syntax (`C:\…`, `smb://…`, a path
//! inside an archive). A path with a `..` component, even one that leads back inside, or an
//! absolute path is not found, so nothing can reach outside the root.
//!
//! ponytail: error messages are English. They go through the translation lookup once E01-S1 has one.

mod local;
mod memory;
pub mod suite;

use std::fmt;
use std::io::Read;
use std::path::{Component, Path};
use std::time::{Duration, SystemTime};

pub use local::LocalSource;
pub use memory::MemorySource;

/// What an item is. An expandable container (an archive, say) is a [`ItemKind::File`] whose
/// [`Item::container`] flag is set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    File,
    Folder,
    Link,
}

/// An item's attributes, as the source's platform reports them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Attributes {
    /// Windows attribute codes (`FILE_ATTRIBUTE_*`).
    Windows(u32),
    /// The Unix mode, owner and group.
    Unix { mode: u32, owner: String, group: String },
}

/// An attribute change: Windows attribute codes, or Unix permission bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permissions {
    Windows(u32),
    UnixMode(u32),
}

/// One file, folder or link, as a listing or [`DataSource::stat`] reports it.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub name: String,
    pub kind: ItemKind,
    /// The content length in bytes: 0 for a folder or an unfollowed link.
    pub size: u64,
    /// `None` only when [`Item::error`] stopped the source from reading it.
    pub modified: Option<SystemTime>,
    /// Where the OS records one.
    pub created: Option<SystemTime>,
    pub attributes: Option<Attributes>,
    /// A link's target, as stored. A followed link keeps it while reporting its target's metadata.
    pub link_target: Option<String>,
    /// A CRC-32 the source already holds (an archive's stored CRC), so nobody needs to read the
    /// content to get it.
    pub crc32: Option<u32>,
    /// The source can list this file's children like a folder's.
    pub container: bool,
    /// Why the source couldn't read this item. The rest of a listing still returns.
    pub error: Option<SourceError>,
}

/// What a source can do. Each source reports its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capabilities {
    /// Every write is refused before anything changes (a CD or DVD, say).
    pub read_only: bool,
    /// `Readme.txt` and `README.txt` are different names.
    pub case_sensitive: bool,
    /// [`Delete::ToRecycleBin`] works.
    pub recycle_bin: bool,
    /// The step modified times are stored in: 2 s on FAT, for example. Zero means exact.
    pub timestamp_resolution: Duration,
    /// [`Permissions::Windows`] works.
    pub windows_attributes: bool,
    /// [`Permissions::UnixMode`] works.
    pub unix_permissions: bool,
}

/// Whether a link reports itself or what it points at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Links {
    /// Report links as links, with their target.
    Report,
    /// Report the target's kind, size, times and attributes instead.
    Follow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delete {
    Permanently,
    /// Only where [`Capabilities::recycle_bin`] says so. Elsewhere the source refuses with
    /// [`ErrorKind::Unsupported`] and deletes nothing.
    ToRecycleBin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    NotFound,
    /// Also every write to a read-only source.
    AccessDenied,
    Io,
    Unsupported,
}

/// An error, attached to the item it concerns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceError {
    pub kind: ErrorKind,
    pub message: String,
}

impl SourceError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self { kind, message: message.into() }
    }
}

impl fmt::Display for SourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for SourceError {}

/// The data-source contract. Methods take `&self` so one source can serve a background scan and a
/// view at once.
///
/// Every write checks [`Capabilities::read_only`] first and changes nothing when it refuses.
/// What a rename onto an existing name, or a folder created over an existing file, does is
/// unspecified: it fails, but each provider picks the error.
pub trait DataSource: Send + Sync {
    fn capabilities(&self) -> Capabilities;

    /// The children of a folder or an expandable container, in no particular order. An item the
    /// source can't read carries its error, and the rest still return.
    fn list(&self, folder: &Path, links: Links) -> Result<Vec<Item>, SourceError>;

    fn stat(&self, path: &Path, links: Links) -> Result<Item, SourceError>;

    fn open_read(&self, path: &Path) -> Result<Box<dyn Read + Send>, SourceError>;

    /// Creates or replaces a file with `content`. The parent folder must exist.
    fn write(&self, path: &Path, content: &mut dyn Read) -> Result<(), SourceError>;

    /// Creates a folder whose parent exists. An existing folder there is fine.
    fn create_folder(&self, path: &Path) -> Result<(), SourceError>;

    /// Deletes a file, a link (not its target) or a folder with everything in it.
    fn delete(&self, path: &Path, how: Delete) -> Result<(), SourceError>;

    /// Gives an item a new name in the same folder.
    fn rename(&self, path: &Path, new_name: &str) -> Result<(), SourceError>;

    fn set_modified(&self, path: &Path, time: SystemTime) -> Result<(), SourceError>;

    /// Refused with [`ErrorKind::Unsupported`] unless the matching capability is set.
    fn set_permissions(&self, path: &Path, permissions: Permissions) -> Result<(), SourceError>;
}

/// Copies a file, or a folder with everything in it, within one source or from one to another,
/// keeping each item's modified time. A link is refused as unsupported (ponytail: copying links
/// waits for a story that needs it).
pub fn copy(from: &dyn DataSource, from_path: &Path, to: &dyn DataSource, to_path: &Path) -> Result<(), SourceError> {
    let item = from.stat(from_path, Links::Report)?;
    if let Some(error) = item.error {
        return Err(error);
    }
    match item.kind {
        ItemKind::File => to.write(to_path, &mut from.open_read(from_path)?)?,
        ItemKind::Folder => {
            // Listed before the target exists, so copying a folder into itself ends.
            let children = from.list(from_path, Links::Report)?;
            to.create_folder(to_path)?;
            for child in children {
                // A listed name could come from an archive entry such as `../x` or `/x`.
                let mut parts = Path::new(&child.name).components();
                if !matches!((parts.next(), parts.next()), (Some(Component::Normal(_)), None)) {
                    return Err(SourceError::new(ErrorKind::NotFound, format!("{} isn't a single name", child.name)));
                }
                copy(from, &from_path.join(&child.name), to, &to_path.join(&child.name))?;
            }
        }
        ItemKind::Link => return Err(SourceError::new(ErrorKind::Unsupported, "copying links isn't supported")),
    }
    match item.modified {
        Some(time) => to.set_modified(to_path, time),
        None => Ok(()),
    }
}
