//! The local provider: files on this machine's disks, and network shares the OS has mounted.
//!
//! Listing and stat are built (E04-S1-T2). Reading and changing files come with E04-S1-T3 and
//! E04-S1-T4; until then those methods refuse with [`ErrorKind::Unsupported`].

use std::fs::{self, Metadata};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::memory::segments;
use crate::{Attributes, Capabilities, DataSource, Delete, ErrorKind, Item, ItemKind, Links, Permissions, SourceError};

/// A folder on a local disk or a mounted share, as a source. Contract paths resolve below it.
#[derive(Debug, Clone)]
pub struct LocalSource {
    root: PathBuf,
}

impl LocalSource {
    /// A source rooted at `root`, a folder path in the OS's own syntax.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// The OS path for a contract path. `..` and absolute paths are not found.
    fn path(&self, path: &Path) -> Result<PathBuf, SourceError> {
        let mut full = self.root.clone();
        full.extend(segments(path)?);
        Ok(full)
    }
}

fn os_error(path: &Path, e: &io::Error) -> SourceError {
    let kind = match e.kind() {
        io::ErrorKind::NotFound => ErrorKind::NotFound,
        io::ErrorKind::PermissionDenied => ErrorKind::AccessDenied,
        _ => ErrorKind::Io,
    };
    SourceError::new(kind, format!("{}: {e}", path.display()))
}

fn not_yet() -> SourceError {
    SourceError::new(ErrorKind::Unsupported, "the local provider can't change or read files yet")
}

/// An item from its unfollowed metadata. When following, a link reports its target's metadata,
/// or keeps its own and carries the error when the target can't be reached (a loop, say).
fn item(name: String, path: &Path, own: Metadata, links: Links) -> Item {
    let mut error = None;
    let mut link_target = None;
    let mut meta = own;
    if meta.is_symlink() {
        match fs::read_link(path) {
            Ok(target) => link_target = Some(target.to_string_lossy().into_owned()),
            Err(e) => error = Some(os_error(path, &e)),
        }
        if links == Links::Follow {
            match fs::metadata(path) {
                Ok(target) => meta = target,
                Err(e) => error = Some(os_error(path, &e)),
            }
        }
    }
    let kind = if meta.is_symlink() {
        ItemKind::Link
    } else if meta.is_dir() {
        ItemKind::Folder
    } else {
        ItemKind::File
    };
    Item {
        name,
        kind,
        size: if kind == ItemKind::File { meta.len() } else { 0 },
        modified: meta.modified().ok(),
        created: meta.created().ok(),
        attributes: attributes(&meta),
        link_target,
        crc32: None,
        container: false,
        error,
    }
}

// ponytail: one user and group lookup per item; cache them by id if large scans show the cost.
#[cfg(unix)]
fn attributes(meta: &Metadata) -> Option<Attributes> {
    use std::os::unix::fs::MetadataExt;
    // An id with no name (a deleted account, a share's foreign user) reports the number.
    let owner = uzers::get_user_by_uid(meta.uid()).map_or_else(|| meta.uid().to_string(), |u| u.name().to_string_lossy().into_owned());
    let group = uzers::get_group_by_gid(meta.gid()).map_or_else(|| meta.gid().to_string(), |g| g.name().to_string_lossy().into_owned());
    Some(Attributes::Unix { mode: meta.mode(), owner, group })
}

#[cfg(windows)]
fn attributes(meta: &Metadata) -> Option<Attributes> {
    use std::os::windows::fs::MetadataExt;
    Some(Attributes::Windows(meta.file_attributes()))
}

#[cfg(not(any(unix, windows)))]
fn attributes(_: &Metadata) -> Option<Attributes> {
    None
}

impl DataSource for LocalSource {
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            read_only: false,
            // ponytail: the OS's usual default (APFS and NTFS ignore case, Linux file systems
            // don't); E04-S1-T5 detects it, and the timestamp resolution, per volume.
            case_sensitive: cfg!(not(any(windows, target_os = "macos"))),
            recycle_bin: false,
            timestamp_resolution: Duration::ZERO,
            windows_attributes: false,
            unix_permissions: false,
        }
    }

    fn list(&self, folder: &Path, links: Links) -> Result<Vec<Item>, SourceError> {
        let full = self.path(folder)?;
        let mut items = Vec::new();
        // ponytail: an entry the OS can't read ends the listing, and names that aren't UTF-8 are
        // reported lossily; E04-S1-T6 gives each bad item its own error.
        for entry in fs::read_dir(&full).map_err(|e| os_error(&full, &e))? {
            let entry = entry.map_err(|e| os_error(&full, &e))?;
            let path = entry.path();
            let own = fs::symlink_metadata(&path).map_err(|e| os_error(&path, &e))?;
            items.push(item(entry.file_name().to_string_lossy().into_owned(), &path, own, links));
        }
        Ok(items)
    }

    fn stat(&self, path: &Path, links: Links) -> Result<Item, SourceError> {
        let full = self.path(path)?;
        let own = fs::symlink_metadata(&full).map_err(|e| os_error(&full, &e))?;
        let name = path.file_name().map_or_else(String::new, |name| name.to_string_lossy().into_owned());
        Ok(item(name, &full, own, links))
    }

    fn open_read(&self, _: &Path) -> Result<Box<dyn Read + Send>, SourceError> {
        Err(not_yet())
    }

    fn write(&self, _: &Path, _: &mut dyn Read) -> Result<(), SourceError> {
        Err(not_yet())
    }

    fn create_folder(&self, _: &Path) -> Result<(), SourceError> {
        Err(not_yet())
    }

    fn delete(&self, _: &Path, _: Delete) -> Result<(), SourceError> {
        Err(not_yet())
    }

    fn rename(&self, _: &Path, _: &str) -> Result<(), SourceError> {
        Err(not_yet())
    }

    fn set_modified(&self, _: &Path, _: SystemTime) -> Result<(), SourceError> {
        Err(not_yet())
    }

    fn set_permissions(&self, _: &Path, _: Permissions) -> Result<(), SourceError> {
        Err(not_yet())
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn owner_and_group_are_reported_by_name() {
        let dir = tempfile::TempDir::new().unwrap();
        fs::write(dir.path().join("mine.txt"), "m").unwrap();
        let item = LocalSource::new(dir.path()).stat(Path::new("mine.txt"), Links::Report).unwrap();
        let Some(Attributes::Unix { owner, group, .. }) = item.attributes else { panic!("{item:?}") };
        assert_eq!(Some(owner.as_str()), uzers::get_current_username().as_deref().and_then(|n| n.to_str()));
        assert!(group.parse::<u32>().is_err(), "group {group} is a number, not a name");
    }
}
