//! Select All and Select All Files, run as the Edit menu and the keyboard run them.

use std::fs;
use std::path::{Path, PathBuf};

use egui::accesskit::Role;
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_ui::Shell;
use otd_ui_folder_compare::{FolderCompare, NOT_ON_LEFT, NOT_ON_RIGHT, ON_LEFT};

use crate::view::{fixture, harness, on, settle};

/// The view fixture with a file inside the `both` folder on each side, hidden while it is closed.
fn folders() -> (tempfile::TempDir, Harness<'static, Shell>) {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    for side in ["left", "right"] {
        fs::write(root.path().join(side).join("both").join("inner.txt"), side).unwrap();
    }
    let harness = harness(FolderCompare::of_folders(&["left".into(), "right".into()], root.path()));
    (root, harness)
}

fn run(harness: &mut Harness<'_, Shell>, id: &str) {
    assert!(harness.state_mut().run(id), "{id} ran");
    settle(harness);
}

fn selected(harness: &Harness<'_, Shell>, side: usize) -> Vec<PathBuf> {
    harness.state().view::<FolderCompare>().expect("a Folder Compare tab").selected(side).into_iter().map(Path::to_path_buf).collect()
}

/// The shown rows' keys that have an item on `side`.
fn shown_items(harness: &Harness<'_, Shell>, side: usize) -> Vec<PathBuf> {
    let view = harness.state().view::<FolderCompare>().expect("a Folder Compare tab");
    view.shown().into_iter().filter(|shown| [&shown.row.left, &shown.row.right][side].is_some()).map(|shown| shown.key).collect()
}

#[test]
fn select_all_selects_every_shown_item_on_both_sides_and_never_a_hidden_row() {
    let (_root, mut harness) = folders();
    let inner = Path::new("both").join("inner.txt");
    // Opened, so `both` is listed and its file is a row, then closed, so that row is hidden.
    run(&mut harness, "folder-compare.expand_all");
    run(&mut harness, "folder-compare.collapse_all");

    run(&mut harness, "folder-compare.select_all");
    for side in 0..2 {
        assert_eq!(selected(&harness, side), shown_items(&harness, side), "side {side}: every shown item");
        assert!(!selected(&harness, side).contains(&inner), "side {side}: the closed folder's file stays unselected");
    }

    run(&mut harness, "folder-compare.expand_all");
    run(&mut harness, "folder-compare.select_all");
    assert!(selected(&harness, 0).contains(&inner), "once its folder is open, the file is selected too");
}

#[test]
fn select_all_files_leaves_folders_unselected() {
    let (_root, mut harness) = folders();
    run(&mut harness, "folder-compare.expand_all");

    run(&mut harness, "folder-compare.select_all_files");
    for side in 0..2 {
        let selected = selected(&harness, side);
        assert!(!selected.contains(&PathBuf::from("both")), "side {side}: the folder stays unselected");
        assert!(selected.contains(&Path::new("both").join("inner.txt")), "side {side}: the file inside it is selected");
        assert!(selected.contains(&PathBuf::from("same.txt")), "side {side}: a top-level file is selected");
    }
}

#[test]
fn cmd_a_in_the_grid_selects_every_shown_item_and_the_cells_say_so() {
    let (_root, mut harness) = folders();
    harness.get_by_label(&on(ON_LEFT, "same.txt")).click();
    harness.run();

    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness.run();
    let is_item = |name: &str| name != NOT_ON_LEFT && name != NOT_ON_RIGHT && !name.is_empty();
    let cells =
        harness.query_all_by(|node| node.role() == Role::Cell && node.label().is_some_and(|name| is_item(&name)) && node.is_selected().is_some());
    let (selected, unselected): (Vec<_>, Vec<_>) = cells.partition(|cell| cell.accesskit_node().is_selected() == Some(true));
    assert!(selected.len() > 1, "more than the clicked item is selected");
    assert!(
        unselected.is_empty(),
        "no shown item is left unselected: {:?}",
        unselected.iter().map(|c| c.accesskit_node().label()).collect::<Vec<_>>()
    );
}
