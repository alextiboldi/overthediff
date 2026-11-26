//! Swipe selection (E08-S7-T1): press on a cell, drag, release.

use egui::Modifiers;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use otd_ui::Shell;
use otd_ui_folder_compare::FolderCompare;

use crate::view::{click, fixture, harness, items, names, rows, selected};

/// Presses on the cell named `from`, drags over the cell named `to` and releases there.
fn swipe(harness: &mut Harness<'_, Shell>, from: &str, to: &str) {
    let (from, to) = (harness.get_by_label(from).rect().center(), harness.get_by_label(to).rect().center());
    harness.drag_at(from);
    harness.run();
    // In steps, as a real pointer moves, so the drag starts on the first cell before it leaves it.
    for step in 1..=4 {
        harness.hover_at(from.lerp(to, step as f32 / 4.0));
        harness.run();
    }
    harness.drop_at(to);
    harness.run();
}

fn sorted(mut cells: Vec<String>) -> Vec<String> {
    cells.sort();
    cells
}

/// The rows of the first and the last item among `cells`, one side's name cells.
fn ends(cells: &[String]) -> (usize, usize) {
    let item = |c: &&String| items(std::slice::from_ref(*c)).len() == 1;
    (cells.iter().position(|c| item(&c)).unwrap(), cells.iter().rposition(|c| item(&c)).unwrap())
}

#[test]
fn a_swipe_from_the_left_pane_across_the_middle_selects_the_rows_on_both_sides() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    let mut harness = harness(FolderCompare::of_folders(&["left".into(), "right".into()], root.path()));
    let rows = rows(&harness);
    let (left, right) = (names(&rows, "left"), names(&rows, "right"));
    let (start, end) = (ends(&left).0, ends(&right).1);
    assert!(end > start + 1, "the swipe passes a row between its ends: {rows:?}");

    swipe(&mut harness, &left[start], &right[end]);

    let mut expected = items(&left[start..=end]);
    expected.extend(items(&right[start..=end]));
    assert_eq!(sorted(selected(&harness)), sorted(expected), "the rows from start to end, on both sides");
}

#[test]
fn a_swipe_inside_one_pane_selects_that_side_only() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    let mut harness = harness(FolderCompare::of_folders(&["left".into(), "right".into()], root.path()));
    let rows = rows(&harness);
    let (left, right) = (names(&rows, "left"), names(&rows, "right"));
    click(&mut harness, &items(&right)[0], Modifiers::NONE);
    let (start, end) = ends(&left);
    assert!(end > start, "the swipe spans more than one row: {rows:?}");

    swipe(&mut harness, &left[start], &left[end]);

    assert_eq!(sorted(selected(&harness)), sorted(items(&left[start..=end])), "the left's rows, and nothing on the right");
}
