//! The local provider: files on this machine's disks, and network shares the OS has mounted.
//!
//! Listing and stat (E04-S1-T2); reading, writing, creating folders, renaming and deleting,
//! permanently or to the OS recycle bin or trash (E04-S1-T3); and setting modified times, Windows
//! attributes and Unix permissions (E04-S1-T4).

use std::fs::{self, Metadata};
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;

use crate::memory::segments;
use crate::volume;
use crate::{Attributes, Capabilities, DataSource, Delete, ErrorKind, Item, ItemKind, Links, Permissions, SourceError};

/// A folder on a local disk or a mounted share, as a source. Contract paths resolve below it.
#[derive(Debug, Clone)]
pub struct LocalSource {
    root: PathBuf,
    /// The root's volume's, found once (E04-S1-T5).
    capabilities: Capabilities,
}

impl LocalSource {
    /// A source rooted at `root`, a folder path in the OS's own syntax.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let capabilities = Capabilities { recycle_bin: !on_network_share(&root), ..volume::capabilities(&root) };
        Self { capabilities, root }
    }

    /// A source rooted at a path as people write it: an OS path, `~/…`, or
    /// `smb://[user@]server/share/…` for a share the OS has mounted.
    pub fn open(path: &str) -> Result<Self, SourceError> {
        Ok(Self::new(volume::resolve(path)?))
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

pub(crate) fn os_error(path: &Path, e: &io::Error) -> SourceError {
    let kind = match e.kind() {
        io::ErrorKind::NotFound => ErrorKind::NotFound,
        io::ErrorKind::PermissionDenied => ErrorKind::AccessDenied,
        io::ErrorKind::Unsupported => ErrorKind::Unsupported,
        _ => ErrorKind::Io,
    };
    SourceError::new(kind, format!("{}: {e}", path.display()))
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
    // lstat on a folder needs only the parent's permission, so a folder that can't be listed
    // looks fine until something tries. ponytail: one extra open per folder; files aren't
    // probed (opening one can block on a pipe or download a cloud placeholder), so a file that
    // can't be read reports its error when it is opened.
    if kind == ItemKind::Folder && error.is_none() {
        error = fs::read_dir(path).err().map(|e| os_error(path, &e));
    }
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

/// An entry whose metadata the OS refused: its kind if the folder entry records one, and the error.
fn unreadable(name: String, entry: &fs::DirEntry, error: SourceError) -> Item {
    let kind = match entry.file_type() {
        Ok(t) if t.is_symlink() => ItemKind::Link,
        Ok(t) if t.is_dir() => ItemKind::Folder,
        _ => ItemKind::File,
    };
    Item {
        name,
        kind,
        size: 0,
        modified: None,
        created: None,
        attributes: None,
        link_target: None,
        crc32: None,
        container: false,
        error: Some(error),
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
    /// FUSE, which serves desktop `smb://` mounts (GVFS), sshfs and rclone, and also local disks
    /// through `fuseblk` (ntfs-3g, exFAT).
    const FUSE: u32 = 0x6573_5546;
    // The magic numbers are 32-bit; the field is wider on 64-bit targets.
    match rustix::fs::statfs(path).map(|info| info.f_type as u32) {
        Ok(FUSE) => {
            use std::os::unix::fs::MetadataExt;
            // ponytail: every FUSE mount but a block device counts as a share, so a local FUSE
            // overlay (bindfs, encfs) gets no recycle bin either; name them if that matters.
            let fstype = fs::metadata(path).ok().and_then(|meta| {
                let mountinfo = fs::read_to_string("/proc/self/mountinfo").ok()?;
                mount_fstype(&mountinfo, meta.dev()).map(str::to_owned)
            });
            fstype.as_deref() != Some("fuseblk")
        }
        Ok(magic) => NETWORK.contains(&magic),
        Err(_) => true,
    }
}

/// The file-system type `/proc/self/mountinfo` gives the mount with device `dev`: the field after
/// the ` - ` separator, matched on the third field, `major:minor`.
#[cfg(target_os = "linux")]
fn mount_fstype(mountinfo: &str, dev: u64) -> Option<&str> {
    let id = format!("{}:{}", rustix::fs::major(dev), rustix::fs::minor(dev));
    mountinfo.lines().find_map(|line| {
        let (fields, rest) = line.split_once(" - ")?;
        if fields.split(' ').nth(2)? == id { rest.split(' ').next() } else { None }
    })
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
        self.capabilities
    }

    fn list(&self, folder: &Path, links: Links) -> Result<Vec<Item>, SourceError> {
        let full = self.path(folder)?;
        let mut items = Vec::new();
        for entry in fs::read_dir(&full).map_err(|e| os_error(&full, &e))? {
            // An entry the OS can't return has no name to carry the error, and skipping it could
            // spin on a broken stream, so it ends the listing.
            let entry = entry.map_err(|e| os_error(&full, &e))?;
            let path = entry.path();
            // No contract path reaches a name that isn't Unicode, and two such names can share
            // one lossy spelling, so the item says so.
            let (name, name_error) = match entry.file_name().into_string() {
                Ok(name) => (name, None),
                Err(raw) => (
                    raw.to_string_lossy().into_owned(),
                    Some(SourceError::new(ErrorKind::Unsupported, format!("{}: the name isn't valid Unicode", path.display()))),
                ),
            };
            let mut found = match fs::symlink_metadata(&path) {
                Ok(own) => item(name, &path, own, links),
                Err(e) => unreadable(name, &entry, os_error(&path, &e)),
            };
            if name_error.is_some() {
                found.error = name_error;
            }
            items.push(found);
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
                if !self.capabilities.recycle_bin || full.parent().is_none_or(on_network_share) {
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

    /// Sets a link's own time, never its target's. The root and folders take one too, as a copy
    /// of a folder needs.
    fn set_modified(&self, path: &Path, time: SystemTime) -> Result<(), SourceError> {
        self.writable()?;
        let full = self.path(path)?;
        set_modified(&full, time).map_err(|e| os_error(&full, &e))
    }

    /// Only the kind the volume keeps (see [`Capabilities`]); the other is refused as
    /// unsupported, as is a Unix mode on FAT and exFAT, which accept one and drop it. A link
    /// is refused too, since the change would land on its target, which can be outside the source.
    fn set_permissions(&self, path: &Path, permissions: Permissions) -> Result<(), SourceError> {
        self.writable()?;
        let full = self.path(path)?;
        let kept = match permissions {
            Permissions::UnixMode(_) => self.capabilities.unix_permissions,
            Permissions::Windows(_) => self.capabilities.windows_attributes,
        };
        if !kept {
            return Err(SourceError::new(ErrorKind::Unsupported, format!("{}: the volume doesn't keep these permissions", full.display())));
        }
        let meta = fs::symlink_metadata(&full).map_err(|e| os_error(&full, &e))?;
        if meta.is_symlink() {
            return Err(SourceError::new(ErrorKind::Unsupported, format!("{}: a link's own attributes can't be set", full.display())));
        }
        set_permissions(&full, permissions).map_err(|e| os_error(&full, &e))
    }
}

/// `utimensat` without following links, keeping the access time.
#[cfg(unix)]
fn set_modified(path: &Path, time: SystemTime) -> io::Result<()> {
    use rustix::fs::{AtFlags, CWD, Timespec, Timestamps, UTIME_OMIT, utimensat};
    let out_of_range = || io::Error::new(io::ErrorKind::InvalidInput, "the time is out of range");
    // Before 1970 the seconds are negative and the nanoseconds still count forward.
    let (seconds, nanos) = match time.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(after) => (i64::try_from(after.as_secs()).map_err(|_| out_of_range())?, after.subsec_nanos()),
        Err(e) => {
            let before = e.duration();
            let seconds = i64::try_from(before.as_secs()).map_err(|_| out_of_range())?;
            match before.subsec_nanos() {
                0 => (-seconds, 0),
                n => (-seconds - 1, 1_000_000_000 - n),
            }
        }
    };
    let times = Timestamps {
        last_access: Timespec { tv_sec: 0, tv_nsec: UTIME_OMIT },
        last_modification: Timespec { tv_sec: seconds, tv_nsec: nanos.into() },
    };
    utimensat(CWD, path, &times, AtFlags::SYMLINK_NOFOLLOW).map_err(io::Error::from)
}

#[cfg(windows)]
fn set_modified(path: &Path, time: SystemTime) -> io::Result<()> {
    attribute_writer(path)?.set_modified(time)
}

/// Opened for attribute writes only, so a read-only file takes a change too; folders need backup
/// semantics, and the reparse-point flag keeps a link's change on the link. std's open adds the
/// `\\?\` prefix a path longer than `MAX_PATH` needs.
#[cfg(windows)]
fn attribute_writer(path: &Path) -> io::Result<fs::File> {
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::{FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_WRITE_ATTRIBUTES};
    fs::OpenOptions::new().access_mode(FILE_WRITE_ATTRIBUTES).custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT).open(path)
}

#[cfg(not(any(unix, windows)))]
fn set_modified(_: &Path, _: SystemTime) -> io::Result<()> {
    Err(io::Error::new(io::ErrorKind::Unsupported, "setting times isn't supported on this system"))
}

fn unsupported(what: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, format!("{what} can't be set on this system"))
}

/// The permission bits, set-id and sticky bits of a mode; the file-type bits a reported mode
/// carries are dropped.
#[cfg(unix)]
fn set_permissions(path: &Path, permissions: Permissions) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    match permissions {
        Permissions::UnixMode(mode) => fs::set_permissions(path, fs::Permissions::from_mode(mode & 0o7777)),
        Permissions::Windows(_) => Err(unsupported("Windows attributes")),
    }
}

#[cfg(windows)]
fn set_permissions(path: &Path, permissions: Permissions) -> io::Result<()> {
    match permissions {
        Permissions::Windows(codes) => windows_attributes::set(path, codes),
        Permissions::UnixMode(_) => Err(unsupported("Unix permissions")),
    }
}

#[cfg(not(any(unix, windows)))]
fn set_permissions(_: &Path, _: Permissions) -> io::Result<()> {
    Err(unsupported("attributes"))
}

/// std sets just the read-only bit, so the other attribute codes go through
/// `SetFileInformationByHandle`, on a handle std opened.
#[cfg(windows)]
#[allow(unsafe_code)] // AGENTS.md "## This repository", rule 13, names this module.
mod windows_attributes {
    use std::io;
    use std::os::windows::io::AsRawHandle;
    use std::path::Path;

    use windows_sys::Win32::Storage::FileSystem::{
        FILE_ATTRIBUTE_ARCHIVE, FILE_ATTRIBUTE_HIDDEN, FILE_ATTRIBUTE_NORMAL, FILE_ATTRIBUTE_NOT_CONTENT_INDEXED, FILE_ATTRIBUTE_OFFLINE,
        FILE_ATTRIBUTE_READONLY, FILE_ATTRIBUTE_SYSTEM, FILE_ATTRIBUTE_TEMPORARY, FILE_BASIC_INFO, FileBasicInfo, SetFileInformationByHandle,
    };

    /// The codes that can be set. The rest (folder, compressed, reparse point…) are the item's
    /// nature, not settings, and are dropped, so a reported code can be passed back.
    const SETTABLE: u32 = FILE_ATTRIBUTE_READONLY
        | FILE_ATTRIBUTE_HIDDEN
        | FILE_ATTRIBUTE_SYSTEM
        | FILE_ATTRIBUTE_ARCHIVE
        | FILE_ATTRIBUTE_TEMPORARY
        | FILE_ATTRIBUTE_OFFLINE
        | FILE_ATTRIBUTE_NOT_CONTENT_INDEXED;

    pub(super) fn set(path: &Path, codes: u32) -> io::Result<()> {
        // A zero would leave the codes as they are, so none is written `NORMAL`, which is valid
        // only alone.
        let codes = match codes & SETTABLE {
            0 => FILE_ATTRIBUTE_NORMAL,
            codes => codes,
        };
        let file = super::attribute_writer(path)?;
        // Zero times are left as they are.
        let info = FILE_BASIC_INFO { CreationTime: 0, LastAccessTime: 0, LastWriteTime: 0, ChangeTime: 0, FileAttributes: codes };
        // SAFETY: `file` keeps the handle open for the whole call, and `info` is a live local
        // whose size is passed with it; the call only reads it.
        let done =
            unsafe { SetFileInformationByHandle(file.as_raw_handle(), FileBasicInfo, (&raw const info).cast(), size_of::<FILE_BASIC_INFO>() as u32) };
        if done == 0 { Err(io::Error::last_os_error()) } else { Ok(()) }
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

    #[cfg(target_os = "linux")]
    #[test]
    fn a_mount_is_found_by_its_device() {
        let mountinfo = "22 1 8:1 / / rw - ext4 /dev/sda1 rw\n\
            40 22 0:45 / /run/user/1000/gvfs rw,nosuid - fuse.gvfsd-fuse gvfsd-fuse rw\n\
            41 22 8:17 / /media/usb rw - fuseblk /dev/sdb1 rw\n";
        assert_eq!(mount_fstype(mountinfo, rustix::fs::makedev(0, 45)), Some("fuse.gvfsd-fuse"));
        assert_eq!(mount_fstype(mountinfo, rustix::fs::makedev(8, 17)), Some("fuseblk"));
        assert_eq!(mount_fstype(mountinfo, rustix::fs::makedev(9, 9)), None);
    }

    #[test]
    fn a_child_whose_metadata_is_denied_carries_the_error_and_the_rest_still_list() {
        use std::io::Write;
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::TempDir::new().unwrap();
        let names = dir.path().join("names-only");
        fs::create_dir(&names).unwrap();
        fs::write(names.join("a.txt"), "a").unwrap();
        fs::write(names.join("b.txt"), "b").unwrap();
        // Readable but not searchable: the names list, but nothing can stat them.
        fs::set_permissions(&names, fs::Permissions::from_mode(0o444)).unwrap();
        if fs::symlink_metadata(names.join("a.txt")).is_ok() {
            fs::set_permissions(&names, fs::Permissions::from_mode(0o755)).unwrap();
            // Straight to stderr, past the test harness's capture, so a passing run still lists it.
            let _ = writeln!(std::io::stderr(), "skipped the denied-metadata case: this account stats children of an unsearchable folder");
            return;
        }
        let listed = LocalSource::new(dir.path()).list(Path::new("names-only"), Links::Report);
        fs::set_permissions(&names, fs::Permissions::from_mode(0o755)).unwrap();
        let mut items = listed.unwrap();
        items.sort_by(|a, b| a.name.cmp(&b.name));
        assert_eq!(items.iter().map(|i| i.name.as_str()).collect::<Vec<_>>(), ["a.txt", "b.txt"]);
        assert!(items.iter().all(|i| i.error.as_ref().is_some_and(|e| e.kind == ErrorKind::AccessDenied)), "{items:#?}");
    }

    // macOS file systems refuse names that aren't UTF-8, so this runs on Linux.
    #[cfg(target_os = "linux")]
    #[test]
    fn a_name_that_isnt_unicode_carries_an_error_and_the_rest_still_list() {
        use std::os::unix::ffi::OsStrExt;
        let dir = tempfile::TempDir::new().unwrap();
        fs::write(dir.path().join(std::ffi::OsStr::from_bytes(b"bad\xff")), "x").unwrap();
        fs::write(dir.path().join("good.txt"), "g").unwrap();
        let items = LocalSource::new(dir.path()).list(Path::new(""), Links::Report).unwrap();
        assert_eq!(items.len(), 2, "{items:#?}");
        for item in &items {
            assert_eq!(item.error.is_some(), item.name != "good.txt", "{item:?}");
        }
    }
}
