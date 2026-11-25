//! The contract suite's listing cases against the local provider, on files it builds in temporary
//! folders.

use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use otd_source::suite::{self, Group, Outcome};
use otd_source::{DataSource, ItemKind, Links, LocalSource, MemorySource};
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
