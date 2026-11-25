//! The local provider, on files it builds in temporary folders: the contract suite's cases, and
//! the data-loss paths the suite doesn't reach.

use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use otd_source::suite::{self, Group, Outcome};
use otd_source::{DataSource, Delete, ErrorKind, ItemKind, Links, LocalSource, MemorySource, Permissions};
use tempfile::TempDir;

/// A built fixture's folder. Dropping it unlocks the folders it locked, so the temporary folder
/// can be removed.
struct Scratch {
    locked: Vec<PathBuf>,
    /// A lock didn't stop this account listing the folder: Windows CI runs with the backup
    /// privilege on, which reads past any permission.
    lock_ignored: bool,
    _dir: TempDir,
}

impl Drop for Scratch {
    fn drop(&mut self) {
        for path in &self.locked {
            unlock(path);
        }
    }
}

/// Builds a fixture's tree on disk. `None` for what this provider can't make here: containers
/// (archives come with E04-S2), unreadable files and read-only media.
fn materialise(fixture: &MemorySource, scratch: &mut Vec<Scratch>) -> Option<Box<dyn DataSource>> {
    if fixture.capabilities().read_only {
        return None;
    }
    let dir = TempDir::new().ok()?;
    // One folder below the temporary one, as the suite asks.
    let root = dir.path().join("root");
    fs::create_dir(&root).ok()?;
    let mut built = Scratch { locked: Vec::new(), lock_ignored: false, _dir: dir };
    let made = copy_tree(fixture, Path::new(""), &root, &mut built);
    scratch.push(built);
    made?;
    Some(Box::new(LocalSource::new(root)))
}

fn copy_tree(fixture: &MemorySource, folder: &Path, to: &Path, scratch: &mut Scratch) -> Option<()> {
    for item in fixture.list(folder, Links::Report).ok()? {
        let (from, path) = (folder.join(&item.name), to.join(&item.name));
        if let Some(error) = item.error {
            // An unreadable folder is made empty, since the fixture won't list it either, then
            // locked. Unless the lock took, the case skips.
            if item.kind != ItemKind::Folder || error.kind != ErrorKind::AccessDenied {
                return None;
            }
            fs::create_dir(&path).ok()?;
            scratch.locked.push(path.clone());
            lock(&path)?;
            if fs::read_dir(&path).is_ok() {
                scratch.lock_ignored = true;
                return None;
            }
            continue;
        }
        if item.container {
            return None;
        }
        match item.kind {
            ItemKind::File => {
                let mut content = Vec::new();
                fixture.open_read(&from).ok()?.read_to_end(&mut content).ok()?;
                fs::write(&path, content).ok()?;
                File::options().write(true).open(&path).ok()?.set_modified(item.modified?).ok()?;
            }
            ItemKind::Folder => {
                fs::create_dir(&path).ok()?;
                copy_tree(fixture, &from, &path, scratch)?;
            }
            ItemKind::Link => link(&item.link_target?, &path, || fixture.stat(&from, Links::Follow).is_ok_and(|t| t.kind == ItemKind::Folder))?,
        }
    }
    Some(())
}

#[cfg(unix)]
fn link(target: &str, path: &Path, _: impl FnOnce() -> bool) -> Option<()> {
    std::os::unix::fs::symlink(target, path).ok()
}

#[cfg(windows)]
fn link(target: &str, path: &Path, to_folder: impl FnOnce() -> bool) -> Option<()> {
    // A relative target with `/` doesn't resolve on Windows.
    let target = target.replace('/', "\\");
    if to_folder() { std::os::windows::fs::symlink_dir(target, path) } else { std::os::windows::fs::symlink_file(target, path) }.ok()
}

#[cfg(unix)]
fn lock(path: &Path) -> Option<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o000)).ok()
}

#[cfg(unix)]
fn unlock(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o755));
}

/// Denies everyone listing the folder. Its owner can still change its permissions.
#[cfg(windows)]
fn lock(path: &Path) -> Option<()> {
    std::process::Command::new("icacls").arg(path).args(["/deny", "*S-1-1-0:(RD)"]).output().ok()?.status.success().then_some(())
}

#[cfg(windows)]
fn unlock(path: &Path) {
    let _ = std::process::Command::new("icacls").arg(path).args(["/remove:d", "*S-1-1-0"]).output();
}

#[test]
fn the_local_provider_passes_every_listing_case_it_can_hold() {
    let mut scratch = Vec::new();
    let results = suite::run(&mut |fixture: &MemorySource| materialise(fixture, &mut scratch));
    let listing: Vec<_> = results.iter().filter(|r| r.group == Group::Listing).collect();
    let failed: Vec<_> = listing.iter().filter(|r| matches!(r.outcome, Outcome::Failed(_))).collect();
    assert!(failed.is_empty(), "{failed:#?}");
    // Exactly these skip, so a link or a lock the OS refused to make can't skip a case unnoticed.
    let skipped: BTreeSet<&str> = listing.iter().filter(|r| matches!(r.outcome, Outcome::Skipped(_))).map(|r| r.name).collect();
    let mut expected = BTreeSet::from(["a_container_lists_its_children_like_a_folder"]);
    if scratch.iter().any(|s| s.lock_ignored) {
        // Straight to stderr, past the test harness's capture, so a passing run still lists it.
        let _ = writeln!(std::io::stderr(), "skipped the unreadable-item case: this account lists folders whatever their permissions");
        expected.insert("an_unreadable_item_carries_its_error_and_the_rest_of_the_listing_returns");
    }
    assert_eq!(skipped, expected, "{listing:#?}");
}

#[cfg(windows)]
#[test]
fn a_junction_is_reported_as_a_link_unless_followed() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join("target")).unwrap();
    let made = std::process::Command::new("cmd").args(["/C", "mklink", "/J", "junction", "target"]).current_dir(dir.path()).status().unwrap();
    assert!(made.success());
    let source = LocalSource::new(dir.path());
    let reported = source.stat(Path::new("junction"), Links::Report).unwrap();
    assert!(reported.kind == ItemKind::Link && reported.link_target.is_some(), "{reported:?}");
    let followed = source.stat(Path::new("junction"), Links::Follow).unwrap();
    assert_eq!(followed.kind, ItemKind::Folder, "{followed:?}");
}

/// The suite cases E04-S1-T3 and T4 build. Listing is E04-S1-T2's, and read-only media E04-S1-T5's.
const CASES: [&str; 10] = [
    "content_reads_back_and_a_missing_file_is_not_found",
    "written_content_reads_back_and_a_replacement_changes_it",
    "a_created_folder_lists_as_an_empty_folder",
    "a_renamed_item_keeps_its_content_under_the_new_name",
    "a_permanent_delete_removes_a_file_and_a_folder_with_its_contents",
    "a_recycle_bin_delete_works_only_where_the_source_has_one",
    "a_path_that_would_leave_the_root_is_not_found_and_changes_nothing",
    "a_modified_time_set_reads_back_within_the_timestamp_resolution",
    "attributes_and_permissions_are_kept_only_where_the_source_supports_them",
    "a_copy_keeps_content_and_modified_time_within_and_across_sources",
];

/// A source holding files and folders, one folder below a fresh temporary folder, as the suite asks.
fn scratch_root() -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("root");
    fs::create_dir(&root).unwrap();
    (dir, root)
}

fn names(source: &LocalSource, folder: &str) -> Vec<String> {
    let mut names: Vec<String> = source.list(Path::new(folder), Links::Report).unwrap().into_iter().map(|item| item.name).collect();
    names.sort_unstable();
    names
}

#[test]
fn the_local_provider_passes_the_read_and_change_cases() {
    let mut scratch = Vec::new();
    let results = suite::run(&mut |fixture: &MemorySource| materialise(fixture, &mut scratch));
    for name in CASES {
        let result = results.iter().find(|r| r.name == name);
        assert!(result.is_some_and(|r| r.outcome == Outcome::Passed), "{name}: {result:?}");
    }
}

/// Gives some bytes, then fails, like a source that drops off the network mid-copy.
struct FailsMidway(usize);

impl Read for FailsMidway {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.0 == 0 {
            return Err(io::Error::other("the source went away"));
        }
        let n = self.0.min(buf.len());
        buf[..n].fill(b'x');
        self.0 -= n;
        Ok(n)
    }
}

#[test]
fn a_write_that_fails_midway_keeps_the_old_content_and_leaves_nothing_behind() {
    let (_dir, root) = scratch_root();
    fs::write(root.join("a.txt"), "old").unwrap();
    let source = LocalSource::new(&root);
    assert!(source.write(Path::new("a.txt"), &mut FailsMidway(10_000)).is_err());
    assert_eq!(fs::read(root.join("a.txt")).unwrap(), b"old");
    assert_eq!(names(&source, ""), ["a.txt"]);
}

#[test]
fn a_rename_never_replaces_another_item() {
    let (_dir, root) = scratch_root();
    fs::write(root.join("a.txt"), "a").unwrap();
    fs::write(root.join("b.txt"), "b").unwrap();
    let source = LocalSource::new(&root);
    assert!(source.rename(Path::new("a.txt"), "b.txt").is_err());
    assert_eq!((fs::read(root.join("a.txt")).unwrap(), fs::read(root.join("b.txt")).unwrap()), (b"a".to_vec(), b"b".to_vec()));
}

#[cfg(unix)]
#[test]
fn a_rename_never_replaces_a_link_whose_target_is_missing() {
    let (_dir, root) = scratch_root();
    fs::write(root.join("a.txt"), "a").unwrap();
    std::os::unix::fs::symlink("missing", root.join("dangling")).unwrap();
    let source = LocalSource::new(&root);
    assert!(source.rename(Path::new("a.txt"), "dangling").is_err());
    assert!(fs::symlink_metadata(root.join("dangling")).unwrap().is_symlink());
    assert_eq!(fs::read(root.join("a.txt")).unwrap(), b"a");
}

#[test]
fn a_rename_can_change_only_the_case() {
    let (_dir, root) = scratch_root();
    fs::write(root.join("readme.txt"), "r").unwrap();
    let source = LocalSource::new(&root);
    source.rename(Path::new("readme.txt"), "README.txt").unwrap();
    assert_eq!(names(&source, ""), ["README.txt"]);
}

#[cfg(unix)]
fn link_to_folder(target: &Path, link: &Path) {
    std::os::unix::fs::symlink(target, link).unwrap();
}

#[cfg(windows)]
fn link_to_folder(target: &Path, link: &Path) {
    std::os::windows::fs::symlink_dir(target, link).unwrap();
}

#[test]
fn deleting_a_link_to_a_folder_keeps_the_folder_and_its_contents() {
    let (_dir, root) = scratch_root();
    fs::create_dir(root.join("target")).unwrap();
    fs::write(root.join("target/kept.txt"), "k").unwrap();
    link_to_folder(&root.join("target"), &root.join("gone"));
    link_to_folder(&root.join("target"), &root.join("binned"));
    let source = LocalSource::new(&root);
    source.delete(Path::new("gone"), Delete::Permanently).unwrap();
    if source.capabilities().recycle_bin {
        source.delete(Path::new("binned"), Delete::ToRecycleBin).unwrap();
    }
    assert!(fs::symlink_metadata(root.join("gone")).is_err());
    assert_eq!(fs::read(root.join("target/kept.txt")).unwrap(), b"k");
}

/// The AC itself: what the recycle bin holds, the OS can put back. macOS has no API that lists
/// the Trash, so there this is checked by hand.
#[cfg(any(target_os = "linux", windows))]
#[test]
fn a_file_deleted_to_the_recycle_bin_can_be_restored_from_it() {
    let (_dir, root) = scratch_root();
    // A name no other run or test uses, so only this file is restored.
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap();
    let name = format!("otd-restore-{}-{}.txt", std::process::id(), now.as_nanos());
    fs::write(root.join(&name), "restore me").unwrap();
    let source = LocalSource::new(&root);
    source.delete(Path::new(&name), Delete::ToRecycleBin).unwrap();
    assert!(fs::symlink_metadata(root.join(&name)).is_err());
    let binned: Vec<_> = trash::os_limited::list().unwrap().into_iter().filter(|item| item.name == name.as_str()).collect();
    assert_eq!(binned.len(), 1, "{binned:?}");
    trash::os_limited::restore_all(binned).unwrap();
    assert_eq!(fs::read(root.join(&name)).unwrap(), b"restore me");
}

/// `\\localhost\C$\…` reaches a local folder through the network, as a share would.
#[cfg(windows)]
#[test]
fn a_network_path_has_no_recycle_bin_and_a_recycle_bin_delete_keeps_the_file() {
    use otd_source::ErrorKind;
    let (_dir, root) = scratch_root();
    fs::write(root.join("a.txt"), "a").unwrap();
    let local = fs::canonicalize(&root).unwrap();
    let local = local.to_string_lossy();
    // `\\?\C:\Users\…` to `\\localhost\C$\Users\…`.
    let (drive, rest) = local.trim_start_matches(r"\\?\").split_once(":").unwrap();
    let share = PathBuf::from(format!(r"\\localhost\{drive}${rest}"));
    let source = LocalSource::new(&share);
    assert!(!source.capabilities().recycle_bin);
    assert!(fs::metadata(share.join("a.txt")).is_ok(), "{} isn't reachable", share.display());
    let result = source.delete(Path::new("a.txt"), Delete::ToRecycleBin);
    assert_eq!(result.map_err(|e| e.kind), Err(ErrorKind::Unsupported));
    assert!(root.join("a.txt").exists());
}

/// A disk image formatted with `file_system`, attached at a temporary mount point and detached
/// when dropped. hdiutil needs no root, so these run on every Mac, CI included. Each takes about
/// a second, most of it hdiutil's.
#[cfg(target_os = "macos")]
struct Volume {
    mount: std::path::PathBuf,
    _dir: TempDir,
}

#[cfg(target_os = "macos")]
impl Volume {
    fn attach(file_system: &str, read_only: bool) -> Self {
        let dir = TempDir::new().unwrap();
        let (image, mount) = (dir.path().join("volume.dmg"), dir.path().join("mount"));
        let mut create = hdiutil(&["create", "-size", "4m", "-fs", file_system, "-layout", "NONE"]);
        assert!(create.arg(&image).status().unwrap().success(), "hdiutil create -fs {file_system}");
        fs::create_dir(&mount).unwrap();
        let mut attach = hdiutil(&["attach", "-nobrowse", "-noverify", "-mountpoint"]);
        attach.arg(&mount).arg(&image);
        if read_only {
            attach.arg("-readonly");
        }
        assert!(attach.status().unwrap().success(), "hdiutil attach {file_system}");
        Self { mount, _dir: dir }
    }
}

#[cfg(target_os = "macos")]
fn hdiutil(args: &[&str]) -> std::process::Command {
    let mut command = std::process::Command::new("hdiutil");
    command.args(args).stdout(std::process::Stdio::null());
    command
}

#[cfg(target_os = "macos")]
impl Drop for Volume {
    fn drop(&mut self) {
        let _ = hdiutil(&["detach", "-force"]).arg(&self.mount).status();
    }
}

#[cfg(target_os = "macos")]
#[test]
fn a_read_only_fat_volume_reports_two_second_times_and_read_only() {
    let volume = Volume::attach("MS-DOS", true);
    let caps = LocalSource::new(&volume.mount).capabilities();
    assert!(caps.read_only && !caps.case_sensitive && caps.timestamp_resolution == std::time::Duration::from_secs(2), "{caps:?}");
}

/// macOS accepts a `chmod` on FAT and keeps nothing, so a mode set there must be refused rather
/// than reported done.
#[cfg(target_os = "macos")]
#[test]
fn a_fat_volume_refuses_a_unix_mode_it_would_drop() {
    let volume = Volume::attach("MS-DOS", false);
    fs::write(volume.mount.join("a.txt"), "a").unwrap();
    let source = LocalSource::new(&volume.mount);
    assert!(!source.capabilities().unix_permissions, "{:?}", source.capabilities());
    let result = source.set_permissions(Path::new("a.txt"), Permissions::UnixMode(0o644));
    assert_eq!(result.map_err(|e| e.kind), Err(ErrorKind::Unsupported));
}

#[cfg(target_os = "macos")]
#[test]
fn a_case_sensitive_volume_reports_case_sensitive_names() {
    // Empty, as a fresh sync or copy target is: there is no name in it to probe.
    let volume = Volume::attach("Case-sensitive HFS+", false);
    let caps = LocalSource::new(&volume.mount).capabilities();
    assert!(caps.case_sensitive && !caps.read_only, "{caps:?}");
}

/// Windows shares every drive as an administrative share, so a temporary folder can be reached
/// through a UNC path (`\\localhost\C$\…`) without setting a share up.
#[cfg(windows)]
#[test]
fn a_folder_reached_through_a_unc_path_lists_and_reports_its_volume() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("a.txt"), "a").unwrap();
    let local = dir.path().to_str().unwrap();
    let (drive, rest) = local.split_once(':').unwrap();
    let source = LocalSource::open(&format!(r"\\localhost\{drive}${rest}")).unwrap();
    let names: Vec<_> = source.list(Path::new(""), Links::Report).unwrap().into_iter().map(|item| item.name).collect();
    assert_eq!(names, ["a.txt"]);
    // The share's file system answered, so the resolution isn't the "exact" fallback.
    assert!(!source.capabilities().timestamp_resolution.is_zero(), "{:?}", source.capabilities());
}

fn modified(path: &Path) -> std::time::SystemTime {
    fs::symlink_metadata(path).unwrap().modified().unwrap()
}

/// The AC: after a copy from one local source to another, the OS reports the same modified time
/// for each file and folder on both sides.
#[test]
fn a_copy_between_local_sources_keeps_every_modified_time() {
    let (_from_dir, from) = scratch_root();
    let (_to_dir, to) = scratch_root();
    fs::create_dir(from.join("dir")).unwrap();
    fs::write(from.join("dir/f.txt"), "inner").unwrap();
    let past = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_500_000_000);
    for path in [from.join("dir/f.txt"), from.join("dir")] {
        LocalSource::new(&from).set_modified(path.strip_prefix(&from).unwrap(), past).unwrap();
    }
    otd_source::copy(&LocalSource::new(&from), Path::new("dir"), &LocalSource::new(&to), Path::new("dir")).unwrap();
    for name in ["dir/f.txt", "dir"] {
        assert_eq!(modified(&to.join(name)), modified(&from.join(name)), "{name}");
        assert_eq!(modified(&to.join(name)), past, "{name}");
    }
}

/// Seconds before 1970 count back while nanoseconds still count forward.
#[test]
fn a_time_before_1970_reads_back() {
    let (_dir, root) = scratch_root();
    fs::write(root.join("old.txt"), "o").unwrap();
    let time = std::time::SystemTime::UNIX_EPOCH - std::time::Duration::from_millis(1_500);
    LocalSource::new(&root).set_modified(Path::new("old.txt"), time).unwrap();
    assert_eq!(modified(&root.join("old.txt")), time);
}

/// The AC: mode 0644 set through the source is what the OS reports.
#[cfg(unix)]
#[test]
fn a_unix_mode_set_through_the_source_is_what_the_os_reports() {
    use std::os::unix::fs::PermissionsExt;
    let (_dir, root) = scratch_root();
    let file = root.join("a.txt");
    fs::write(&file, "a").unwrap();
    // A new file is often 0644 already, so start elsewhere.
    fs::set_permissions(&file, fs::Permissions::from_mode(0o600)).unwrap();
    let source = LocalSource::new(&root);
    assert!(source.capabilities().unix_permissions);
    source.set_permissions(Path::new("a.txt"), Permissions::UnixMode(0o100_644)).unwrap();
    assert_eq!(fs::metadata(&file).unwrap().permissions().mode() & 0o7777, 0o644);
}

/// The AC: read-only set through the source is what the OS reports, and a read-only file still
/// takes a modified time.
#[cfg(windows)]
#[test]
fn read_only_set_through_the_source_is_what_the_os_reports() {
    const READ_ONLY: u32 = 0x1;
    let (_dir, root) = scratch_root();
    let file = root.join("a.txt");
    fs::write(&file, "a").unwrap();
    let source = LocalSource::new(&root);
    assert!(source.capabilities().windows_attributes);
    source.set_permissions(Path::new("a.txt"), Permissions::Windows(READ_ONLY)).unwrap();
    assert!(fs::metadata(&file).unwrap().permissions().readonly());
    let past = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_500_000_000);
    source.set_modified(Path::new("a.txt"), past).unwrap();
    assert_eq!(modified(&file), past);
    // Cleared again, so the temporary folder can be removed.
    source.set_permissions(Path::new("a.txt"), Permissions::Windows(0)).unwrap();
    assert!(!fs::metadata(&file).unwrap().permissions().readonly());
}

/// An item past `MAX_PATH` (260 characters), as deep trees have, takes attributes like any other.
#[cfg(windows)]
#[test]
fn attributes_are_set_on_an_item_past_max_path() {
    const HIDDEN: u32 = 0x2;
    let (_dir, root) = scratch_root();
    let deep = Path::new(&"d".repeat(100)).join("e".repeat(100)).join("f".repeat(100)).join("a.txt");
    fs::create_dir_all(root.join(deep.parent().unwrap())).unwrap();
    fs::write(root.join(&deep), "a").unwrap();
    assert!(root.join(&deep).as_os_str().len() > 260);
    let source = LocalSource::new(&root);
    source.set_permissions(&deep, Permissions::Windows(HIDDEN)).unwrap();
    let attributes = source.stat(&deep, Links::Report).unwrap().attributes;
    assert!(matches!(attributes, Some(otd_source::Attributes::Windows(codes)) if codes & HIDDEN == HIDDEN), "{attributes:?}");
}

/// A link's target can be outside the source, so changes stop at the link.
#[cfg(unix)]
#[test]
fn setting_times_and_permissions_on_a_link_never_changes_its_target() {
    use std::os::unix::fs::PermissionsExt;
    let (_dir, root) = scratch_root();
    let outside = root.parent().unwrap().join("outside.txt");
    fs::write(&outside, "o").unwrap();
    std::os::unix::fs::symlink(&outside, root.join("link")).unwrap();
    let (time, mode) = (modified(&outside), fs::metadata(&outside).unwrap().permissions().mode());
    let source = LocalSource::new(&root);
    let past = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_500_000_000);
    source.set_modified(Path::new("link"), past).unwrap();
    assert!(source.set_permissions(Path::new("link"), Permissions::UnixMode(0o777)).is_err());
    assert_eq!(modified(&root.join("link")), past);
    assert_eq!((modified(&outside), fs::metadata(&outside).unwrap().permissions().mode()), (time, mode));
}
