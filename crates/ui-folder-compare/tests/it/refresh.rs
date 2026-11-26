//! Refresh and Swap Sides, run as the menus run them, after the folders change on disk.

use std::fs;

use egui::accesskit::{Live, Role};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_core::i18n::t;
use otd_folder_engine::Status;
use otd_ui::Shell;
use otd_ui_folder_compare::{FolderCompare, LEFT, NOT_ON_RIGHT, ON_LEFT, RESULT, commands, session_type};

use crate::view::{fixture, on, rows, settle};

/// The shell with Folder Compare's commands, showing `view` once its scans are done.
fn harness(view: FolderCompare) -> Harness<'static, Shell> {
    let mut shell = Shell::new(vec![session_type()]).with_commands(commands());
    shell.open(Box::new(view));
    let mut harness = Harness::builder().build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), shell);
    settle(&mut harness);
    harness
}

fn run(harness: &mut Harness<'_, Shell>, id: &str) {
    assert!(harness.state_mut().run(id), "{id} ran");
    settle(harness);
}

fn same_txt(harness: &Harness<'_, Shell>) -> (Status, Vec<String>) {
    let view = harness.state().view::<FolderCompare>().expect("a Folder Compare tab");
    let shown = view.shown();
    let row = shown.iter().find(|shown| shown.row.name == "same.txt").expect("same.txt has a row").row;
    let cells = rows(harness).into_iter().find(|cells| cells.contains(&on(ON_LEFT, "same.txt"))).expect("same.txt is drawn");
    (row.status.clone(), cells)
}

#[test]
fn refresh_shows_the_new_status_of_a_file_changed_on_disk() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    let mut harness = harness(FolderCompare::of_folders(&["left".into(), "right".into()], root.path()));
    let before = same_txt(&harness);
    assert_eq!(before.0, Status::Same);

    // Longer, so its size differs: a rewrite of the same size is within the time tolerance.
    fs::write(root.path().join("right").join("same.txt"), "no longer the same").unwrap();
    harness.run();
    assert_eq!(same_txt(&harness), before, "nothing changes until refreshed");

    run(&mut harness, "folder-compare.refresh");
    let after = same_txt(&harness);
    assert_ne!(after.0, Status::Same, "the row's status follows the file");
    assert_ne!(after.1, before.1, "and so do its cells");
}

#[test]
fn refresh_shows_a_base_folder_that_could_not_be_listed_once_it_can() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    let mut harness = harness(FolderCompare::of_folders(&["missing".into(), "right".into()], root.path()));
    assert!(rows(&harness).is_empty());

    fs::create_dir(root.path().join("missing")).unwrap();
    run(&mut harness, "folder-compare.refresh");
    assert!(!rows(&harness).is_empty(), "the rows show");
    // The status bar's result is announced too; only the error's announcement should be gone.
    let result = t(RESULT);
    let announced = harness
        .query_all_by(|node| node.live() == Live::Polite && node.role() == Role::Label && !node.value().is_some_and(|text| text.starts_with(result)))
        .count();
    assert_eq!(announced, 0, "and the error is gone");
}

#[test]
fn swap_sides_puts_the_right_folder_on_the_left_and_compares_again() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    let mut harness = harness(FolderCompare::of_folders(&["left".into(), "right".into()], root.path()));

    run(&mut harness, "folder-compare.swap_sides");
    let left_edit = harness.get_by(|node| node.role() == Role::TextInput && node.label().as_deref() == Some(LEFT)).accesskit_node().value();
    assert_eq!(left_edit, Some(root.path().join("right").to_string_lossy().into_owned()), "the left path edit holds the right folder");
    let rows = rows(&harness);
    assert!(
        rows.iter().any(|row| row.contains(&on(ON_LEFT, "only-right.txt")) && row.contains(&NOT_ON_RIGHT.to_owned())),
        "the right folder's orphan is on the left, facing a gap: {rows:?}"
    );
}
