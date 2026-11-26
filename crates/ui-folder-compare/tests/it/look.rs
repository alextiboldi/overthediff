//! How each side of a row looks: its colour, spot and glyphs, and a folder's aggregated cues, on
//! rows the folder engine compares from two `MemorySource` trees.

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use otd_folder_engine::{Alignment, Criteria, DEFAULT_TOLERANCE, Row, TreeScan, TreeSide, compare_trees};
use otd_source::{Capabilities, MemorySource};
use otd_ui::colour::ColourRole;
use otd_ui_folder_compare::look::{Cue, DIFFERENT, Look, ORPHAN, SAME, UNKNOWN, look};

const CRITERIA: Criteria = Criteria {
    size: true,
    modified: true,
    tolerance: DEFAULT_TOLERANCE,
    filename_case: false,
    unix_permissions: false,
    unix_owner: false,
    unix_group: false,
};

pub(crate) fn at(seconds: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)
}

pub(crate) fn source() -> MemorySource {
    MemorySource::new(Capabilities {
        read_only: false,
        case_sensitive: true,
        recycle_bin: false,
        timestamp_resolution: Duration::ZERO,
        windows_attributes: false,
        unix_permissions: false,
    })
}

/// Both trees compared, scanned to the end unless `background` is off.
pub(crate) fn rows(left: MemorySource, right: MemorySource, background: bool) -> Vec<Row> {
    let scan = |source| {
        let (scan, _events) = TreeScan::new(Arc::new(source), "", background);
        while scan.step() {}
        scan
    };
    compare_trees(&scan(left), &scan(right), &CRITERIA, Alignment::new(true), &|_, _, _| true)
}

fn row<'a>(rows: &'a [Row], name: &str) -> &'a Row {
    rows.iter().find(|row| row.name == name).unwrap_or_else(|| panic!("no row {name}: {rows:#?}"))
}

fn sides(row: &Row) -> [Option<Look>; 2] {
    [look(row, TreeSide::Left), look(row, TreeSide::Right)]
}

fn colour(look: &Option<Look>) -> Option<&'static str> {
    look.as_ref().map(|look| look.colour().id)
}

fn ids(roles: &[&ColourRole]) -> Vec<&'static str> {
    roles.iter().map(|role| role.id).collect()
}

/// Same, newer against older, a size difference within the tolerance, and an orphan on each side.
fn files() -> (MemorySource, MemorySource) {
    let left =
        source().file("same.txt", "x", at(100)).file("new.txt", "x", at(500)).file("size.txt", "xx", at(101)).file("only-left.txt", "x", at(1));
    let right =
        source().file("same.txt", "x", at(100)).file("new.txt", "x", at(100)).file("size.txt", "x", at(100)).file("only-right.txt", "x", at(1));
    (left, right)
}

#[test]
fn each_item_takes_its_status_colour_and_its_spot_shows_the_same() {
    let (left, right) = files();
    let rows = rows(left, right, true);
    let expected = [
        ("same.txt", [Some(SAME.id), Some(SAME.id)]),
        ("new.txt", [Some(DIFFERENT.id), Some(UNKNOWN.id)]),
        ("size.txt", [Some(DIFFERENT.id), Some(DIFFERENT.id)]),
        ("only-left.txt", [Some(ORPHAN.id), None]),
        ("only-right.txt", [None, Some(ORPHAN.id)]),
    ];
    for (name, colours) in expected {
        let looks = sides(row(&rows, name));
        assert_eq!(looks.each_ref().map(colour), colours, "{name}");
        for look in looks.iter().flatten() {
            assert_eq!(ids(&look.spot()), [look.colour().id], "{name}: one colour, in the spot too");
        }
    }
}

#[test]
fn every_status_says_in_words_what_its_colour_shows() {
    let (left, right) = files();
    let rows = rows(left, right, true);
    let mut seen: Vec<(Cue, String)> = Vec::new();
    for look in rows.iter().flat_map(sides).flatten() {
        let text = look.description();
        assert!(!text.is_empty(), "{look:?}");
        if let Some((cue, other)) = seen.iter().find(|(cue, other)| (*cue == look.cues[0]) != (*other == text)) {
            panic!("{cue:?} reads {other:?} but {:?} reads {text:?}", look.cues[0]);
        }
        seen.push((look.cues[0], text));
    }
}

#[test]
fn a_link_carries_the_arrow_glyph_and_a_file_does_not() {
    let left = source().file("target.txt", "x", at(1)).link("shortcut", "target.txt", at(1));
    let right = source().file("target.txt", "x", at(1)).link("shortcut", "target.txt", at(1));
    let rows = rows(left, right, true);
    assert!(sides(row(&rows, "shortcut")).iter().all(|look| look.as_ref().is_some_and(|look| look.link)));
    assert!(sides(row(&rows, "target.txt")).iter().all(|look| look.as_ref().is_some_and(|look| !look.link)));
}

#[test]
fn a_folder_that_cant_be_opened_carries_the_red_x_on_its_side_only() {
    let left = source().file("locked/a.txt", "x", at(1)).unreadable("locked");
    let right = source().file("locked/a.txt", "x", at(1));
    let [left, right] = sides(row(&rows(left, right, true), "locked"));
    let (left, right) = (left.expect("on the left"), right.expect("on the right"));
    assert!(left.error.is_some() && left.colour().id == DIFFERENT.id, "{left:?}");
    assert!(left.description().contains(left.error.as_deref().unwrap_or_default()), "the message is in its text: {left:?}");
    assert!(right.error.is_none() && right.colour().id == UNKNOWN.id, "its partner, compared with nothing, is unknown: {right:?}");
}

#[test]
fn a_folder_holding_a_newer_file_and_an_orphan_shows_both_cues_at_once_with_red_first() {
    let left = source().folder("d", at(1)).file("d/new.txt", "x", at(500)).file("d/only.txt", "x", at(1)).file("d/same.txt", "x", at(1));
    let right = source().folder("d", at(1)).file("d/new.txt", "x", at(100)).file("d/same.txt", "x", at(1));
    let [left, right] = sides(row(&rows(left, right, true), "d"));
    let (left, right) = (left.expect("on the left"), right.expect("on the right"));
    assert_eq!(ids(&left.spot()), [DIFFERENT.id, ORPHAN.id], "newer and orphan, the name in red: {left:?}");
    assert_eq!(left.colour().id, DIFFERENT.id);
    assert_eq!(ids(&right.spot()), [ORPHAN.id, UNKNOWN.id], "the other side holds the older file: {right:?}");
    assert_ne!(left.description(), right.description());
}

#[test]
fn a_folder_with_the_same_contents_is_same_whatever_its_own_timestamp() {
    let left = source().folder("d", at(1)).file("d/a.txt", "x", at(1)).folder("empty", at(1));
    let right = source().folder("d", at(9_000)).file("d/a.txt", "x", at(1)).folder("empty", at(9_000));
    let rows = rows(left, right, true);
    for name in ["d", "empty"] {
        assert_eq!(sides(row(&rows, name)).each_ref().map(colour), [Some(SAME.id); 2], "{name}");
    }
}

#[test]
fn a_cue_deep_inside_reaches_every_folder_above_it() {
    let left = source().file("a/b/c/new.txt", "x", at(500));
    let right = source().file("a/b/c/new.txt", "x", at(1));
    assert_eq!(sides(row(&rows(left, right, true), "a")).each_ref().map(colour), [Some(DIFFERENT.id), Some(UNKNOWN.id)]);
}

#[test]
fn a_folder_pair_not_listed_yet_is_unknown_not_same() {
    let left = source().file("d/new.txt", "x", at(500));
    let right = source().file("d/new.txt", "x", at(1));
    // Background scanning off: only the base folders are listed, so `d` has no children yet.
    let d = row(&rows(left, right, false), "d").clone();
    assert!(d.children.is_empty(), "{d:#?}");
    assert_eq!(sides(&d).each_ref().map(colour), [Some(UNKNOWN.id); 2]);
}
