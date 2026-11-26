//! Flat List (E08-S2-T4) in the real shell, toggled as the View menu toggles it, with the rows read
//! back through the accessibility tree. The fixture trees are made in a temporary folder.

use std::fs;
use std::path::Path;

use egui::Key;
use egui::accesskit::Role;
use egui_kittest::Harness;
use otd_ui::Shell;
use otd_ui_folder_compare::{FolderCompare, ON_LEFT, ON_RIGHT};

use crate::view::{focused, harness, on, press, row_names, rows, settle};

const FLAT_LIST: &str = "folder-compare.flat_list";

/// `x.txt` in `moved` on the left and in `elsewhere` on the right; `dup.txt` in both `d` and `e`
/// on both sides; `top.txt` in both base folders.
fn fixture(root: &Path) -> Harness<'static, Shell> {
    for (side, folder) in [("left", "moved"), ("right", "elsewhere")] {
        let base = root.join(side);
        fs::create_dir_all(base.join(folder)).unwrap();
        fs::write(base.join(folder).join("x.txt"), "x").unwrap();
        for dup in ["d", "e"] {
            fs::create_dir_all(base.join(dup)).unwrap();
            fs::write(base.join(dup).join("dup.txt"), dup).unwrap();
        }
        fs::write(base.join("top.txt"), "top").unwrap();
    }
    harness(FolderCompare::of_folders(&["left".into(), "right".into()], root))
}

fn toggle(harness: &mut Harness<'_, Shell>) {
    assert!(harness.state_mut().run(FLAT_LIST), "{FLAT_LIST} ran");
    settle(harness);
}

/// A side's name cell of a file at `path` (slash-separated) from its base, as the flat list names it.
fn at(side: &str, path: &str) -> String {
    let path = path.split('/').fold(std::path::PathBuf::new(), |path, part| path.join(part));
    on(side, &path.to_string_lossy())
}

#[test]
fn same_named_files_in_different_subfolders_share_a_row_that_shows_each_sides_folder() {
    let root = tempfile::tempdir().unwrap();
    let mut harness = fixture(root.path());
    toggle(&mut harness);
    let rows = rows(&harness);
    assert!(rows.iter().any(|row| row.contains(&at(ON_LEFT, "moved/x.txt")) && row.contains(&at(ON_RIGHT, "elsewhere/x.txt"))), "{rows:?}");
    assert!(!rows.iter().flatten().any(|cell| cell == &on(ON_LEFT, "moved")), "no folder rows: {rows:?}");
}

#[test]
fn turning_the_flat_list_off_brings_back_the_tree_as_it_was_with_focus_still_in_the_rows() {
    let root = tempfile::tempdir().unwrap();
    let mut harness = fixture(root.path());
    tab_to_rows(&mut harness);
    // The first row, `d`, opened and the other folders left closed, so the tree to come back is
    // neither the base rows only nor every folder open.
    assert!(harness.state_mut().run("folder-compare.open_folder"), "open_folder ran");
    settle(&mut harness);
    let before = row_names(&harness);
    toggle(&mut harness);
    assert_ne!(row_names(&harness), before, "the flat list shows other rows");
    assert!(focused(&harness).is_some_and(|(role, _)| role == Role::Cell), "focus stays in the rows");
    toggle(&mut harness);
    assert_eq!(row_names(&harness), before);
    assert!(focused(&harness).is_some_and(|(role, _)| role == Role::Cell), "focus stays in the rows");
}

/// Tabs to the row area's current name cell.
fn tab_to_rows(harness: &mut Harness<'_, Shell>) {
    for _ in 0..40 {
        if focused(harness).is_some_and(|(role, _)| role == Role::Cell) {
            return;
        }
        press(harness, Key::Tab);
    }
    panic!("Tab never reached the rows");
}

#[test]
fn files_of_one_name_in_two_folders_on_both_sides_are_two_rows_each_paired_with_its_own_path() {
    let root = tempfile::tempdir().unwrap();
    let mut harness = fixture(root.path());
    toggle(&mut harness);
    let rows = rows(&harness);
    for dup in ["d/dup.txt", "e/dup.txt"] {
        assert!(rows.iter().any(|row| row.contains(&at(ON_LEFT, dup)) && row.contains(&at(ON_RIGHT, dup))), "{dup}: {rows:?}");
    }
    // Each row is a cell of its own for the keyboard too: Down reaches both.
    tab_to_rows(&mut harness);
    // From the top: the current cell is wherever the list's first rows put it.
    for _ in 0..rows.len() {
        press(&mut harness, Key::ArrowUp);
    }
    let mut reached = Vec::new();
    for _ in 0..rows.len() {
        reached.extend(focused(&harness).map(|(_, label)| label));
        press(&mut harness, Key::ArrowDown);
    }
    for dup in ["d/dup.txt", "e/dup.txt"] {
        assert!(reached.contains(&at(ON_LEFT, dup)), "{dup}: {reached:?}");
    }
}
