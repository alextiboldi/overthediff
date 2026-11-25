//! Selection resolution on two scanned `MemorySource` trees.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use otd_folder_engine::{Criteria, Resolved, Row, TreeScan, TreeSide, compare_trees, resolve_selection};
use otd_source::{DataSource, Item, MemorySource};

use crate::fixture::{at, caps};

const CRITERIA: Criteria = Criteria { size: true, modified: true, tolerance: Duration::from_secs(2) };

fn scan(source: MemorySource, background: bool) -> TreeScan {
    let (scan, _events) = TreeScan::new(Arc::new(source), "", background);
    while scan.step() {}
    scan
}

fn rows(left: MemorySource, right: MemorySource, background: bool, visible: &dyn Fn(TreeSide, &Path, &Item) -> bool) -> Vec<Row> {
    let case_sensitive = left.capabilities().case_sensitive && right.capabilities().case_sensitive;
    compare_trees(&scan(left, background), &scan(right, background), &CRITERIA, case_sensitive, visible)
}

fn all(_: TreeSide, _: &Path, _: &Item) -> bool {
    true
}

fn paths(resolved: &Resolved) -> BTreeSet<PathBuf> {
    resolved.items.iter().map(|(path, _)| path.clone()).collect()
}

fn set(paths: &[&str]) -> BTreeSet<PathBuf> {
    paths.iter().map(PathBuf::from).collect()
}

/// `d` holds three files, a subfolder with a file in it and a log file; `top.txt` sits beside it.
fn tree() -> MemorySource {
    MemorySource::new(caps(true))
        .file("d/a.txt", "a", at(1))
        .file("d/b.txt", "b", at(1))
        .file("d/c.txt", "c", at(1))
        .file("d/sub/x.txt", "x", at(1))
        .file("d/build.log", "log", at(1))
        .file("top.txt", "t", at(1))
}

#[test]
fn a_closed_folder_resolves_to_all_its_contents_at_every_depth_and_not_to_itself() {
    let rows = rows(tree(), tree(), true, &all);
    let resolved = resolve_selection(&rows, TreeSide::Left, &[Path::new("d")]);
    assert_eq!(paths(&resolved), set(&["d/a.txt", "d/b.txt", "d/c.txt", "d/sub", "d/sub/x.txt", "d/build.log"]));
    assert!(!resolved.incomplete);
}

#[test]
fn an_open_folder_with_two_selected_children_resolves_to_those_children_only() {
    let rows = rows(tree(), tree(), true, &all);
    let resolved = resolve_selection(&rows, TreeSide::Left, &[Path::new("d"), Path::new("d/a.txt"), Path::new("d/sub")]);
    assert_eq!(paths(&resolved), set(&["d/a.txt", "d/sub/x.txt"]));
}

/// Hidden on both sides, `d/sub` drops out with its contents; hidden on the left only, `build.log`
/// drops out of the left set and stays in the right one.
#[test]
fn a_folder_with_hidden_items_resolves_to_what_each_side_shows() {
    let visible = |side, at: &Path, _: &Item| at != Path::new("d/sub") && !(side == TreeSide::Left && at == Path::new("d/build.log"));
    let rows = rows(tree(), tree().file("d/right-only.txt", "r", at(1)), true, &visible);
    let left = resolve_selection(&rows, TreeSide::Left, &[Path::new("d")]);
    let right = resolve_selection(&rows, TreeSide::Right, &[Path::new("d")]);
    assert_eq!(paths(&left), set(&["d/a.txt", "d/b.txt", "d/c.txt"]));
    assert_eq!(paths(&right), set(&["d/a.txt", "d/b.txt", "d/c.txt", "d/build.log", "d/right-only.txt"]));
}

#[test]
fn a_selected_folder_not_listed_yet_says_its_result_is_incomplete() {
    // Background scanning off: only the bases are listed, so `d` has no rows below it yet.
    let rows = rows(tree(), tree(), false, &all);
    let resolved = resolve_selection(&rows, TreeSide::Left, &[Path::new("d")]);
    assert!(resolved.incomplete && resolved.items.is_empty(), "{resolved:#?}");
}

/// The right side's paths use its own spelling, not the row's, which is the left's.
#[test]
fn each_side_resolves_to_paths_in_its_own_spelling() {
    let left = MemorySource::new(caps(false)).file("Docs/Read.txt", "x", at(1));
    let right = MemorySource::new(caps(false)).file("docs/READ.txt", "x", at(1));
    let rows = rows(left, right, true, &all);
    let resolved = resolve_selection(&rows, TreeSide::Right, &[Path::new("Docs")]);
    assert_eq!(paths(&resolved), set(&["docs/READ.txt"]));
}
