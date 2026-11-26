//! The Folder Compare view in the real shell, opened as `otd A B` opens it, read back through the
//! accessibility tree and driven by keyboard. The fixture trees are made in a temporary folder.

use std::fs;
use std::path::Path;
use std::time::Duration;

use egui::accesskit::{Live, Role};
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_core::i18n::fmt;
use otd_ui::Shell;
use otd_ui_folder_compare::{FolderCompare, NOT_ON_LEFT, NOT_ON_RIGHT, ON_LEFT, ON_RIGHT, session_type};

/// `left` and `right` under `root`: a file and a folder on both sides, and an orphan on each.
fn fixture(root: &Path) {
    for (side, orphan) in [("left", "only-left.txt"), ("right", "only-right.txt")] {
        fs::create_dir_all(root.join(side).join("both")).unwrap();
        fs::write(root.join(side).join("same.txt"), "same").unwrap();
        fs::write(root.join(side).join(orphan), side).unwrap();
    }
}

/// The shell showing `view`, once its scans have nothing left to list.
fn harness(view: FolderCompare) -> Harness<'static, Shell> {
    let mut shell = Shell::new(vec![session_type()]);
    shell.open(Box::new(view));
    let mut harness = Harness::builder().build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), shell);
    settle(&mut harness);
    harness
}

/// Draws until the view's scans have nothing left to list.
fn settle(harness: &mut Harness<'_, Shell>) {
    // The scans run on worker threads; the limit only stops a test that would hang.
    for _ in 0..1000 {
        harness.run();
        if !harness.state().view::<FolderCompare>().is_some_and(FolderCompare::scanning) {
            harness.run();
            return;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("the scans never finished");
}

/// Each row's cell names, in row order.
fn rows(harness: &Harness<'_, Shell>) -> Vec<Vec<String>> {
    harness
        .query_all_by_role(Role::Row)
        .map(|row| row.query_all_by_role(Role::Cell).filter_map(|cell| cell.accesskit_node().label()).collect())
        .collect()
}

fn on(template: &str, name: &str) -> String {
    fmt(template, &[("name", name)])
}

fn focused(harness: &Harness<'_, Shell>) -> Option<(Role, String)> {
    let node = harness.query_by(|node| node.is_focused())?;
    let node = node.accesskit_node();
    Some((node.role(), node.label().or_else(|| node.value()).unwrap_or_default()))
}

fn press(harness: &mut Harness<'_, Shell>, key: Key) {
    harness.key_press_modifiers(Modifiers::NONE, key);
    harness.run();
}

fn tab_to(harness: &mut Harness<'_, Shell>, role: Role, name: &str) {
    for _ in 0..40 {
        if focused(harness).is_some_and(|(r, label)| r == role && label == name) {
            return;
        }
        press(harness, Key::Tab);
    }
    panic!("Tab never reached {name:?}");
}

#[test]
fn two_folders_from_the_command_line_fill_the_path_edits_and_their_items_align_by_name_with_gaps_for_orphans() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    // Relative, as typed in a terminal: they resolve against the launch's working folder.
    let harness = harness(FolderCompare::of_folders(&["left".into(), "right".into()], root.path()));

    let paths: Vec<String> = harness.query_all_by_role(Role::TextInput).filter_map(|edit| edit.accesskit_node().value()).collect();
    for side in ["left", "right"] {
        assert!(paths.contains(&root.path().join(side).to_string_lossy().into_owned()), "{side} is in its path edit: {paths:?}");
    }

    let rows = rows(&harness);
    let row_of = |cell: &str| rows.iter().find(|row| row.iter().any(|c| c == cell)).unwrap_or_else(|| panic!("no row has {cell:?}: {rows:?}"));
    for name in ["same.txt", "both"] {
        assert!(row_of(&on(ON_LEFT, name)).contains(&on(ON_RIGHT, name)), "{name} and its partner share a row");
    }
    assert!(row_of(&on(ON_LEFT, "only-left.txt")).contains(&NOT_ON_RIGHT.to_owned()), "a left orphan faces a gap");
    assert!(row_of(&on(ON_RIGHT, "only-right.txt")).contains(&NOT_ON_LEFT.to_owned()), "a right orphan faces a gap");
    assert_eq!(rows.len(), 4, "one row per name");
}

#[test]
fn tab_stops_once_in_the_rows_and_the_arrows_move_focus_between_rows_and_sides() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    let mut harness = harness(FolderCompare::of_folders(&["left".into(), "right".into()], root.path()));
    let rows = rows(&harness);

    for _ in 0..40 {
        if focused(&harness).is_some_and(|(role, _)| role == Role::Cell) {
            break;
        }
        press(&mut harness, Key::Tab);
    }
    assert_eq!(focused(&harness).map(|(_, name)| name).as_ref(), Some(&rows[0][0]), "Tab reaches the first row's left name");

    press(&mut harness, Key::ArrowDown);
    let second_left = rows[1].iter().find(|c| c.ends_with(&on(ON_LEFT, "")) || **c == NOT_ON_LEFT).cloned();
    assert_eq!(focused(&harness).map(|(_, name)| name), second_left, "Down moves to the next row, same side");

    press(&mut harness, Key::ArrowRight);
    let second_right = rows[1].iter().find(|c| c.ends_with(&on(ON_RIGHT, "")) || **c == NOT_ON_RIGHT).cloned();
    assert_eq!(focused(&harness).map(|(_, name)| name), second_right, "Right moves to the other side's cell of that row");

    press(&mut harness, Key::Tab);
    assert!(focused(&harness).is_none_or(|(role, _)| role != Role::Cell), "the next Tab leaves the rows");
}

#[test]
fn a_folder_that_cant_be_listed_says_why_and_shows_no_rows() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    let harness = harness(FolderCompare::of_folders(&["missing".into(), "right".into()], root.path()));
    let announced = harness.query_all_by(|node| node.live() == Live::Polite && node.role() == Role::Label).count();
    assert!(announced > 0, "the error is shown and announced");
    assert!(rows(&harness).is_empty(), "nothing is compared against a folder that isn't there");
}

#[test]
fn folders_typed_into_the_path_edits_are_compared_once_each_edit_is_left() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    let mut harness = harness(FolderCompare::new());
    for (label, side) in [(otd_ui_folder_compare::LEFT, "left"), (otd_ui_folder_compare::RIGHT, "right")] {
        tab_to(&mut harness, Role::TextInput, label);
        harness.get_by(|node| node.is_focused()).type_text(&root.path().join(side).to_string_lossy());
        harness.run();
        assert!(rows(&harness).is_empty(), "nothing is compared while a path is being typed");
        press(&mut harness, Key::Tab);
    }
    settle(&mut harness);
    let rows = rows(&harness);
    assert!(rows.iter().any(|row| row.contains(&on(ON_LEFT, "same.txt")) && row.contains(&on(ON_RIGHT, "same.txt"))), "{rows:?}");
}
