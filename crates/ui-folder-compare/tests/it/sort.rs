//! Sorting by a column: a click on its header, or *View > Sort By* run as the menu runs it.

use std::fs;
use std::path::Path;

use egui::Key;
use egui::accesskit::{Role, SortDirection};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_core::i18n::t;
use otd_ui::Shell;
use otd_ui_folder_compare::{FolderCompare, ON_LEFT, SIZE};

use crate::view::{focused, harness, on, press, rows, settle};

/// The same three files at the top and in `sub`, on both sides, with sizes that put them in a
/// different order on each side, and neither in name order.
fn fixture(root: &Path) -> Harness<'static, Shell> {
    for (side, sizes) in [("left", [3, 2, 1]), ("right", [2, 3, 1])] {
        for dir in [root.join(side), root.join(side).join("sub")] {
            fs::create_dir_all(&dir).unwrap();
            for (name, size) in ["big.txt", "mid.txt", "small.txt"].into_iter().zip(sizes) {
                fs::write(dir.join(name), "x".repeat(size)).unwrap();
            }
        }
    }
    let mut harness = harness(FolderCompare::of_folders(&["left".into(), "right".into()], root));
    run(&mut harness, "folder-compare.expand_all");
    harness
}

fn run(harness: &mut Harness<'_, Shell>, id: &str) {
    assert!(harness.state_mut().run(id), "{id} ran");
    settle(harness);
}

/// The files shown at `depth`, in row order, as each row's left name, checking on the way that
/// every row's cells name one item on both sides.
fn files(harness: &Harness<'_, Shell>, depth: usize) -> Vec<String> {
    let view = harness.state().view::<FolderCompare>().expect("a Folder Compare tab");
    let shown: Vec<String> =
        view.shown().iter().filter(|shown| shown.depth == depth && !shown.is_folder()).map(|shown| shown.row.name.clone()).collect();
    for name in &shown {
        let row = rows(harness).into_iter().find(|cells| cells.contains(&on(ON_LEFT, name))).expect("the row is drawn");
        assert!(row.contains(&on(otd_ui_folder_compare::ON_RIGHT, name)), "{name}'s partner shares its row: {row:?}");
    }
    shown
}

/// The Size headers, left then right: their accessible names and sort directions.
fn size_headers(harness: &Harness<'_, Shell>) -> Vec<(String, Option<SortDirection>)> {
    harness
        .query_all_by_role(Role::ColumnHeader)
        .map(|header| (header.accesskit_node().label().unwrap_or_default(), header.accesskit_node().sort_direction()))
        .filter(|(name, _)| name.starts_with(t(SIZE)))
        .collect()
}

fn click_left_size(harness: &mut Harness<'_, Shell>) {
    let header =
        harness.query_all_by_role(Role::ColumnHeader).find(|header| header.accesskit_node().label().is_some_and(|name| name.starts_with(t(SIZE))));
    header.expect("the left pane has a Size header").click();
    harness.run();
}

#[test]
fn a_size_header_click_sorts_files_in_each_folder_ascending_then_descending_and_partners_stay_on_one_row() {
    let root = tempfile::tempdir().unwrap();
    let mut harness = fixture(root.path());
    let by_name = files(&harness, 0);

    click_left_size(&mut harness);
    let ascending = files(&harness, 0);
    assert_eq!(ascending, ["small.txt", "mid.txt", "big.txt"], "by the left sizes, smallest first");
    assert_ne!(ascending, by_name, "the order changed");
    assert_eq!(files(&harness, 1), ascending, "the files inside a folder sort too");
    assert_eq!(size_headers(&harness)[0].1, Some(SortDirection::Ascending), "the header says how it sorts");

    click_left_size(&mut harness);
    let descending = files(&harness, 0);
    assert_eq!(descending, ascending.iter().rev().cloned().collect::<Vec<_>>(), "a second click reverses the order");
    assert_eq!(files(&harness, 1), descending, "inside the folder too");
    assert_eq!(size_headers(&harness)[0].1, Some(SortDirection::Descending));
}

#[test]
fn sort_by_from_the_menu_sorts_by_the_current_cells_side_and_a_full_refresh_keeps_the_order() {
    let root = tempfile::tempdir().unwrap();
    let mut harness = fixture(root.path());
    for _ in 0..40 {
        if focused(&harness).is_some_and(|(role, _)| role == Role::Cell) {
            break;
        }
        press(&mut harness, Key::Tab);
    }
    press(&mut harness, Key::ArrowRight);

    run(&mut harness, "folder-compare.sort.size");
    let sorted = files(&harness, 0);
    assert_eq!(sorted, ["small.txt", "big.txt", "mid.txt"], "by the right sizes, smallest first");
    let headers: Vec<_> = size_headers(&harness).into_iter().map(|(_, direction)| direction).collect();
    assert_eq!(headers, [None, Some(SortDirection::Ascending)], "the right header is the sorted one");

    run(&mut harness, "folder-compare.full_refresh");
    run(&mut harness, "folder-compare.expand_all");
    assert_eq!(files(&harness, 0), sorted, "a full refresh keeps the order");
    assert_eq!(files(&harness, 1), sorted);
}
