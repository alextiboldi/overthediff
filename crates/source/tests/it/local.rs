//! The local provider, on files it builds in temporary folders: the contract suite's cases, and
//! the data-loss paths the suite doesn't reach.

use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use otd_source::suite::{self, Group, Outcome};
use otd_source::{DataSource, Delete, ItemKind, Links, LocalSource, MemorySource};
use tempfile::TempDir;

/// Builds a fixture's tree on disk. `None` for what this task can't make: containers (archives
/// come with E04-S2), unreadable items (E04-S1-T6) and read-only media.
fn materialise(fixture: &MemorySource, scratch: &mut Vec<TempDir>) -> Option<Box<dyn DataSource>> {
    if fixture.capabilities().read_only {
        return None;
    }
    let dir = TempDir::new().ok()?;
    // One folder below the temporary one, as the suite asks.
    let root = dir.path().join("root");
    fs::create_dir(&root).ok()?;
    copy_tree(fixture, Path::new(""), &root)?;
    scratch.push(dir);
    Some(Box::new(LocalSource::new(root)))
}

fn copy_tree(fixture: &MemorySource, folder: &Path, to: &Path) -> Option<()> {
    for item in fixture.list(folder, Links::Report).ok()? {
        if item.error.is_some() || item.container {
            return None;
        }
        let (from, path) = (folder.join(&item.name), to.join(&item.name));
        match item.kind {
            ItemKind::File => {
                let mut content = Vec::new();
                fixture.open_read(&from).ok()?.read_to_end(&mut content).ok()?;
                fs::write(&path, content).ok()?;
                File::options().write(true).open(&path).ok()?.set_modified(item.modified?).ok()?;
            }
            ItemKind::Folder => {
                fs::create_dir(&path).ok()?;
                copy_tree(fixture, &from, &path)?;
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

#[test]
fn the_local_provider_passes_every_listing_case_it_can_hold() {
    let mut scratch = Vec::new();
    let results = suite::run(&mut |fixture: &MemorySource| materialise(fixture, &mut scratch));
    let listing: Vec<_> = results.iter().filter(|r| r.group == Group::Listing).collect();
    let failed: Vec<_> = listing.iter().filter(|r| matches!(r.outcome, Outcome::Failed(_))).collect();
    assert!(failed.is_empty(), "{failed:#?}");
    // Exactly these skip, so a link the OS refused to make can't skip the link cases unnoticed.
    let skipped: BTreeSet<&str> = listing.iter().filter(|r| matches!(r.outcome, Outcome::Skipped(_))).map(|r| r.name).collect();
    let expected =
        BTreeSet::from(["a_container_lists_its_children_like_a_folder", "an_unreadable_item_carries_its_error_and_the_rest_of_the_listing_returns"]);
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

/// The suite cases E04-S1-T3 builds. Listing is E04-S1-T2's, times and permissions (and so copy)
/// E04-S1-T4's, and read-only media E04-S1-T5's.
const CASES: [&str; 7] = [
    "content_reads_back_and_a_missing_file_is_not_found",
    "written_content_reads_back_and_a_replacement_changes_it",
    "a_created_folder_lists_as_an_empty_folder",
    "a_renamed_item_keeps_its_content_under_the_new_name",
    "a_permanent_delete_removes_a_file_and_a_folder_with_its_contents",
    "a_recycle_bin_delete_works_only_where_the_source_has_one",
    "a_path_that_would_leave_the_root_is_not_found_and_changes_nothing",
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
