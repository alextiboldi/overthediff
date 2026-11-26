//! Folder aggregation and the per-side visibility predicate, on two scanned `MemorySource` trees.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use otd_folder_engine::{Alignment, Criteria, Cues, Row, Status, TreeScan, TreeSide, compare_trees};
use otd_source::{DataSource, ErrorKind, Item, MemorySource, SourceError};

use crate::fixture::{at, caps};

const CRITERIA: Criteria = Criteria {
    size: true,
    modified: true,
    tolerance: Duration::from_secs(2),
    filename_case: false,
    unix_permissions: false,
    unix_owner: false,
    unix_group: false,
};

fn path(names: &[&str]) -> PathBuf {
    names.iter().collect()
}

fn scan(source: MemorySource, background: bool) -> TreeScan {
    let (scan, _events) = TreeScan::new(Arc::new(source), "", background);
    while scan.step() {}
    scan
}

/// Both trees fully scanned, compared with `visible` as the predicate.
fn rows(left: MemorySource, right: MemorySource, visible: &dyn Fn(TreeSide, &Path, &Item) -> bool) -> Vec<Row> {
    let case_sensitive = left.capabilities().case_sensitive && right.capabilities().case_sensitive;
    compare_trees(&scan(left, true), &scan(right, true), &CRITERIA, Alignment::new(case_sensitive), visible)
}

fn all(_: TreeSide, _: &Path, _: &Item) -> bool {
    true
}

fn row<'a>(rows: &'a [Row], name: &str) -> &'a Row {
    rows.iter().find(|row| row.name == name).unwrap_or_else(|| panic!("{name} is a row of {rows:#?}"))
}

/// `d` holds `new.txt`, newer on the left, and `only.txt`, on the left only.
fn newer_and_orphan() -> (MemorySource, MemorySource) {
    let left = MemorySource::new(caps(true)).file("d/new.txt", "x", at(100)).file("d/only.txt", "x", at(1));
    let right = MemorySource::new(caps(true)).file("d/new.txt", "x", at(1));
    (left, right)
}

#[test]
fn a_folder_holding_a_newer_file_and_an_orphan_carries_both_cues() {
    let (left, right) = newer_and_orphan();
    let cues = row(&rows(left, right, &all), "d").cues();
    assert!(cues.contains(&Status::LeftNewer) && cues.contains(&Status::LeftOrphan), "{cues:?}");
}

#[test]
fn hiding_the_newer_file_through_the_predicate_removes_its_cue_and_keeps_the_orphans() {
    let (left, right) = newer_and_orphan();
    let rows = rows(left, right, &|_, at: &Path, _: &Item| at != path(&["d", "new.txt"]));
    let d = row(&rows, "d");
    assert!(row(&d.children, "new.txt").hidden(), "hidden on both sides");
    let cues = d.cues();
    assert!(!cues.contains(&Status::LeftNewer) && cues.contains(&Status::LeftOrphan), "{cues:?}");
}

#[test]
fn a_file_hidden_on_the_left_only_stays_visible_flagged_on_that_side_and_still_counts() {
    let (left, right) = newer_and_orphan();
    let rows = rows(left, right, &|side, at: &Path, _: &Item| !(side == TreeSide::Left && at == path(&["d", "new.txt"])));
    let d = row(&rows, "d");
    let new = row(&d.children, "new.txt");
    assert!(new.left_hidden && !new.right_hidden && !new.hidden(), "{new:#?}");
    assert!(d.cues().contains(&Status::LeftNewer));
}

#[test]
fn a_cue_deep_down_reaches_every_folder_above_it() {
    let left = MemorySource::new(caps(true)).file("a/b/c/x.txt", "x", at(1));
    let right = MemorySource::new(caps(true)).file("a/b/c/x.txt", "x", at(100));
    let rows = rows(left, right, &all);
    assert!(Cues::of(&rows).contains(&Status::RightNewer));
    assert!(row(&rows, "a").cues().contains(&Status::RightNewer));
}

/// A folder's own timestamp never makes it newer: only its contents decide.
#[test]
fn a_folder_with_a_later_timestamp_and_the_same_contents_has_no_newer_cue() {
    let left = MemorySource::new(caps(true)).folder("d", at(1_000)).file("d/x.txt", "x", at(1));
    let right = MemorySource::new(caps(true)).folder("d", at(1)).file("d/x.txt", "x", at(1));
    let rows = rows(left, right, &all);
    for cues in [row(&rows, "d").cues(), Cues::of(&rows)] {
        assert!(cues.contains(&Status::Same) && !cues.contains(&Status::LeftNewer) && !cues.contains(&Status::RightNewer), "{cues:?}");
    }
}

/// On a case-insensitive pair, each side descends by its own spelling, so `Docs` and `docs`
/// compare their contents rather than look empty on one side.
#[test]
fn folders_whose_names_differ_only_in_case_compare_their_contents() {
    let left = MemorySource::new(caps(false)).file("Docs/a.txt", "x", at(1));
    let right = MemorySource::new(caps(false)).file("docs/a.txt", "x", at(1));
    let rows = rows(left, right, &all);
    let docs = row(&rows, "Docs");
    let a = row(&docs.children, "a.txt");
    assert!(a.left.is_some() && a.right.is_some() && a.status == Status::Same, "{a:#?}");
}

#[test]
fn a_folder_not_yet_scanned_is_unknown_not_same() {
    let tree = || MemorySource::new(caps(true)).file("d/x.txt", "x", at(1));
    // Background scanning off and `d` never opened: only the base is listed.
    let rows = compare_trees(&scan(tree(), false), &scan(tree(), false), &CRITERIA, Alignment::new(true), &all);
    let cues = Cues::of(&rows);
    assert!(cues.contains(&Status::Unknown) && !cues.contains(&Status::Same), "{cues:?}");
}

#[test]
fn an_unreadable_folder_gives_its_parent_an_error_cue() {
    let left = MemorySource::new(caps(true)).file("d/locked/x.txt", "x", at(1)).unreadable("d/locked");
    let right = MemorySource::new(caps(true)).file("d/locked/x.txt", "x", at(1));
    let d = row(&rows(left, right, &all), "d").cues();
    assert!(d.contains(&Status::Error(SourceError::new(ErrorKind::Io, "any error matches"))), "{d:?}");
    // Nothing is known of what the unreadable side holds, so the other side's items aren't orphans.
    assert!(d.contains(&Status::Unknown) && !d.contains(&Status::RightOrphan), "{d:?}");
}

fn no_orphans(cues: Cues) -> bool {
    !cues.contains(&Status::LeftOrphan) && !cues.contains(&Status::RightOrphan)
}

/// One side's listing of `d` isn't done yet, so what it faces deep down is unknown, not orphans.
#[test]
fn items_facing_a_folder_not_yet_listed_on_the_other_side_are_unknown_not_orphans() {
    let tree = || MemorySource::new(caps(true)).file("d/sub/x.txt", "x", at(1));
    let rows = compare_trees(&scan(tree(), true), &scan(tree(), false), &CRITERIA, Alignment::new(true), &all);
    for cues in [row(&rows, "d").cues(), Cues::of(&rows)] {
        assert!(cues.contains(&Status::Unknown) && no_orphans(cues), "{cues:?}");
    }
}

#[test]
fn items_facing_a_base_not_yet_listed_are_unknown_not_orphans() {
    let tree = || MemorySource::new(caps(true)).file("x.txt", "x", at(1));
    let (unlisted, _events) = TreeScan::new(Arc::new(tree()), "", false);
    let cues = Cues::of(&compare_trees(&scan(tree(), true), &unlisted, &CRITERIA, Alignment::new(true), &all));
    assert!(cues.contains(&Status::Unknown) && no_orphans(cues), "{cues:?}");
}

/// A folder on one side only is listed and its contents really are orphans.
#[test]
fn the_contents_of_a_listed_orphan_folder_are_orphans() {
    let left = MemorySource::new(caps(true)).file("d/x.txt", "x", at(1));
    let right = MemorySource::new(caps(true)).file("y.txt", "x", at(1));
    let rows = rows(left, right, &all);
    assert_eq!(row(&row(&rows, "d").children, "x.txt").status, Status::LeftOrphan);
}

/// A folder hidden on a side hides its contents on that side, so nothing in it counts.
#[test]
fn the_contents_of_a_hidden_folder_are_hidden_and_add_no_cues() {
    let (left, right) = newer_and_orphan();
    let rows = rows(left, right, &|_, at: &Path, _: &Item| at != Path::new("d"));
    let d = row(&rows, "d");
    assert!(d.children.iter().all(Row::hidden), "{d:#?}");
    assert!(Cues::of(&rows).is_empty());
}

/// The normalization switch pairs folders too, so a folder spelt NFD on one side and NFC on the
/// other compares its contents instead of showing two orphan trees.
#[test]
fn with_the_normalization_switch_on_an_nfd_folder_and_its_nfc_twin_compare_their_contents() {
    let left = MemorySource::new(caps(true)).file("caf\u{e9}/a.txt", "x", at(1));
    let right = MemorySource::new(caps(true)).file("cafe\u{301}/a.txt", "x", at(1));
    let alignment = Alignment { normalization: true, ..Alignment::new(true) };
    let rows = compare_trees(&scan(left, true), &scan(right, true), &CRITERIA, alignment, &all);
    let folder = row(&rows, "caf\u{e9}");
    assert_eq!(row(&folder.children, "a.txt").status, Status::Same);
}

/// `d` holds two files and `d/sub` a third, on the left only.
fn sized() -> MemorySource {
    MemorySource::new(caps(true)).file("d/a.txt", "abc", at(1)).file("d/b.txt", "defgh", at(1)).file("d/sub/c.txt", "ij", at(1))
}

#[test]
fn a_folder_s_total_size_is_the_sum_of_every_file_below_it_on_that_side() {
    let rows = rows(sized(), MemorySource::new(caps(true)), &all);
    let d = row(&rows, "d");
    let files: u64 = ["abc", "defgh", "ij"].iter().map(|text| text.len() as u64).sum();
    assert_eq!(d.total_size(TreeSide::Left), Some(files));
    assert_eq!(d.total_size(TreeSide::Right), None, "the right side has no folder here");
}

/// `d` is listed but `d/sub` is not: its size would be a partial sum.
#[test]
fn a_folder_s_total_size_is_unknown_until_every_folder_below_it_is_listed() {
    let (left, _events) = TreeScan::new(Arc::new(sized()), "", false);
    let (right, _events) = TreeScan::new(Arc::new(MemorySource::new(caps(true))), "", false);
    left.open(Path::new("d"));
    while left.step() || right.step() {}
    let total = |left: &TreeScan| row(&compare_trees(left, &right, &CRITERIA, Alignment::new(true), &all), "d").total_size(TreeSide::Left);
    assert_eq!(total(&left), None);
    left.open(&path(&["d", "sub"]));
    while left.step() {}
    assert!(total(&left).is_some());
}

#[test]
fn a_folder_with_an_unreadable_item_below_it_has_no_total_size() {
    let left = sized().unreadable("d/sub/c.txt");
    assert_eq!(row(&rows(left, MemorySource::new(caps(true)), &all), "d").total_size(TreeSide::Left), None);
}

#[test]
fn a_folder_s_total_size_leaves_out_what_is_hidden_on_that_side() {
    let all_rows = rows(sized(), MemorySource::new(caps(true)), &all);
    let hidden = rows(sized(), MemorySource::new(caps(true)), &|_, at: &Path, _: &Item| at != path(&["d", "sub"]));
    let total = |rows: &[Row]| row(rows, "d").total_size(TreeSide::Left).expect("listed");
    assert!(total(&hidden) < total(&all_rows));
}
