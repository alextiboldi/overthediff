//! The local provider: files on this machine's disks, and network shares the OS has mounted.
//!
//! Listing and stat (E04-S1-T2) and reading, writing, creating folders, renaming and deleting,
//! permanently or to the OS recycle bin or trash (E04-S1-T3), are built. Setting times and
//! permissions refuses with [`ErrorKind::Unsupported`] until E04-S1-T4.

use std::fs::{self, Metadata};
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::memory::segments;
use crate::{Attributes, Capabilities, DataSource, Delete, ErrorKind, Item, ItemKind, Links, Permissions, SourceError};

/// A folder on a local disk or a mounted share, as a source. Contract paths resolve below it.
#[derive(Debug, Clone)]
pub struct LocalSource {
    root: PathBuf,
    recycle_bin: bool,
}

impl LocalSource {
    /// A source rooted at `root`, a folder path in the OS's own syntax.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let recycle_bin = !on_network_share(&root);
        Self { root, recycle_bin }
    }

    /// The OS path for a contract path. `..` and absolute paths are not found.
    fn path(&self, path: &Path) -> Result<PathBuf, SourceError> {
        let mut full = self.root.clone();
        full.extend(segments(path)?);
        Ok(full)
    }

    /// The OS path for an item a change replaces, renames or deletes, which the root can't be.
    fn item_path(&self, path: &Path) -> Result<PathBuf, SourceError> {
        let full = self.path(path)?;
        if full == self.root {
            return Err(SourceError::new(ErrorKind::Unsupported, "the root of a source can't be replaced, renamed or deleted"));
        }
        Ok(full)
    }

    /// Every change checks this first, so a refused one changes nothing.
    fn writable(&self) -> Result<(), SourceError> {
        if self.capabilities().read_only {
            return Err(SourceError::new(ErrorKind::AccessDenied, format!("{} is read-only", self.root.display())));
        }
        Ok(())
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
    SourceError::new(ErrorKind::Unsupported, "the local provider can't set times or permissions yet")
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

/// Whether two names that differ only in case are one item, as a case-only rename gives. Two
/// hard links to one file are different names, so the rename is refused there too.
fn same_item(a: &Metadata, b: &Metadata, from: &Path, to: &Path) -> bool {
    let lower = |path: &Path| path.file_name().map(|name| name.to_string_lossy().to_lowercase());
    lower(from) == lower(to) && same_file(a, b)
}

#[cfg(unix)]
fn same_file(a: &Metadata, b: &Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    (a.dev(), a.ino()) == (b.dev(), b.ino())
}

/// ponytail: Windows' file index is unstable in std, so identical creation and write times and
/// size stand in for it.
#[cfg(windows)]
fn same_file(a: &Metadata, b: &Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    (a.creation_time(), a.last_write_time(), a.len()) == (b.creation_time(), b.last_write_time(), b.len())
}

/// Removes a file, or a link itself and never its target. On Windows a link to a folder,
/// junctions included, is a folder entry.
fn remove_entry(path: &Path, meta: &Metadata) -> io::Result<()> {
    #[cfg(windows)]
    {
        use std::os::windows::fs::FileTypeExt;
        if meta.file_type().is_symlink_dir() {
            return fs::remove_dir(path);
        }
    }
    #[cfg(not(windows))]
    let _ = meta;
    fs::remove_file(path)
}

/// Whether `path` is on a network share, where the OS keeps no recycle bin (Windows deletes such
/// items permanently). On macOS and Linux a path the OS can't inspect counts as one, so a
/// recycle-bin delete is refused rather than made permanent.
#[cfg(target_os = "macos")]
fn on_network_share(path: &Path) -> bool {
    /// `<sys/mount.h>`: the file system is stored on this machine.
    const MNT_LOCAL: u32 = 0x1000;
    rustix::fs::statfs(path).map_or(true, |info| info.f_flags & MNT_LOCAL == 0)
}

#[cfg(target_os = "linux")]
fn on_network_share(path: &Path) -> bool {
    /// `statfs` magic numbers (`<linux/magic.h>` and the file systems' own headers): NFS, SMB,
    /// CIFS, SMB2, AFS, Coda, NCP and 9P.
    const NETWORK: [u32; 8] = [0x6969, 0x517B, 0xFF53_4D42, 0xFE53_4D42, 0x5346_414F, 0x7375_7245, 0x564C, 0x0102_1997];
    // The magic numbers are 32-bit; the field is wider on 64-bit targets.
    let network = |info: rustix::fs::StatFs| NETWORK.contains(&(info.f_type as u32));
    rustix::fs::statfs(path).map_or(true, network)
}

/// UNC paths (`\\server\share\…`). A mapped drive is expected to resolve to one through
/// `canonicalize`, which is unverified. A path that can't be resolved is judged by its own prefix.
#[cfg(windows)]
fn on_network_share(path: &Path) -> bool {
    use std::path::Prefix;
    let resolved = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    matches!(resolved.components().next(), Some(Component::Prefix(prefix)) if matches!(prefix.kind(), Prefix::UNC(..) | Prefix::VerbatimUNC(..)))
}

/// ponytail: other systems get no recycle bin until a story targets them.
#[cfg(not(any(target_os = "macos", target_os = "linux", windows)))]
fn on_network_share(_: &Path) -> bool {
    true
}

/// Moves an item to the OS recycle bin or trash.
fn to_recycle_bin(path: &Path) -> Result<(), trash::Error> {
    #[cfg(target_os = "macos")]
    {
        use trash::macos::{DeleteMethod, TrashContextExtMacos};
        // Finder's method drives Finder by script, which asks the user for permission first.
        let mut context = trash::TrashContext::default();
        context.set_delete_method(DeleteMethod::NsFileManager);
        context.delete(path)
    }
    #[cfg(not(target_os = "macos"))]
    trash::delete(path)
}

impl DataSource for LocalSource {
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            read_only: false,
            // ponytail: the OS's usual default (APFS and NTFS ignore case, Linux file systems
            // don't); E04-S1-T5 detects it, and the timestamp resolution, per volume.
            case_sensitive: cfg!(not(any(windows, target_os = "macos"))),
            recycle_bin: self.recycle_bin,
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

    fn open_read(&self, path: &Path) -> Result<Box<dyn Read + Send>, SourceError> {
        let full = self.path(path)?;
        let file = fs::File::open(&full).map_err(|e| os_error(&full, &e))?;
        // Unix opens a folder for reading; its reads would fail later.
        if file.metadata().map_err(|e| os_error(&full, &e))?.is_dir() {
            return Err(SourceError::new(ErrorKind::Io, format!("{} is a folder", full.display())));
        }
        Ok(Box::new(file))
    }

    /// Crash-safe: the content goes to a temporary file beside the target, which then replaces it,
    /// so a failure leaves the old content whole. A replaced file keeps its permissions.
    fn write(&self, path: &Path, content: &mut dyn Read) -> Result<(), SourceError> {
        self.writable()?;
        let full = self.item_path(path)?;
        craft_atomic::atomic_write_with(&full, |file| io::copy(content, file).map(drop)).map_err(|e| os_error(&full, &e))
    }

    fn create_folder(&self, path: &Path) -> Result<(), SourceError> {
        self.writable()?;
        let full = self.path(path)?;
        match fs::create_dir(&full) {
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                if fs::metadata(&full).is_ok_and(|meta| meta.is_dir()) {
                    Ok(())
                } else {
                    Err(SourceError::new(ErrorKind::Io, format!("{}: a file is in the way", full.display())))
                }
            }
            result => result.map_err(|e| os_error(&full, &e)),
        }
    }

    fn delete(&self, path: &Path, how: Delete) -> Result<(), SourceError> {
        self.writable()?;
        let full = self.item_path(path)?;
        let meta = fs::symlink_metadata(&full).map_err(|e| os_error(&full, &e))?;
        match how {
            Delete::Permanently if meta.is_dir() => fs::remove_dir_all(&full),
            Delete::Permanently => remove_entry(&full, &meta),
            Delete::ToRecycleBin => {
                // A share can be mounted below a local root, so the item's own folder decides.
                if !self.recycle_bin || full.parent().is_none_or(on_network_share) {
                    return Err(SourceError::new(ErrorKind::Unsupported, format!("the recycle bin is unavailable for {}", full.display())));
                }
                return to_recycle_bin(&full)
                    .map_err(|e| SourceError::new(ErrorKind::Io, format!("{}: couldn't move it to the recycle bin: {e}", full.display())));
            }
        }
        .map_err(|e| os_error(&full, &e))
    }

    /// Refuses a name another item already has, but allows a change of case alone.
    fn rename(&self, path: &Path, new_name: &str) -> Result<(), SourceError> {
        self.writable()?;
        let mut parts = Path::new(new_name).components();
        if !matches!((parts.next(), parts.next()), (Some(Component::Normal(_)), None)) {
            return Err(SourceError::new(ErrorKind::Unsupported, format!("{new_name} isn't a single name")));
        }
        let from = self.item_path(path)?;
        let to = from.with_file_name(new_name);
        let meta = fs::symlink_metadata(&from).map_err(|e| os_error(&from, &e))?;
        // `symlink_metadata`, so a link at the new name counts even when its target is missing.
        if let Ok(other) = fs::symlink_metadata(&to)
            && !same_item(&meta, &other, &from, &to)
        {
            return Err(SourceError::new(ErrorKind::Io, format!("{}: an item with that name already exists", to.display())));
        }
        fs::rename(&from, &to).map_err(|e| os_error(&from, &e))
    }

    fn set_modified(&self, path: &Path, _: SystemTime) -> Result<(), SourceError> {
        self.writable()?;
        self.item_path(path)?;
        Err(not_yet())
    }

    fn set_permissions(&self, path: &Path, _: Permissions) -> Result<(), SourceError> {
        self.writable()?;
        self.item_path(path)?;
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
