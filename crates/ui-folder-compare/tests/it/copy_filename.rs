//! Copy Filename (E08-S7-T5), run as the Edit menu runs it, after selecting by clicks.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use egui::{Modifiers, OutputCommand};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use otd_ui::Shell;
use otd_ui_folder_compare::{FolderCompare, ON_LEFT, ON_RIGHT};

use crate::view::{fixture, harness, on, settle};

const COPY_FILENAME: &str = "folder-compare.copy_filename";

fn click(harness: &mut Harness<'_, Shell>, name: &str, modifiers: Modifiers) {
    harness.get_by_label(name).click_modifiers(modifiers);
    harness.run();
}

/// Runs Copy Filename as a menu click does and returns the lines the next draw puts on the
/// clipboard, if any.
fn copy(harness: &mut Harness<'_, Shell>) -> Option<BTreeSet<String>> {
    assert!(harness.state_mut().run(COPY_FILENAME), "Copy Filename ran");
    // One frame only: a later one has output of its own.
    harness.step();
    harness.output().platform_output.commands.iter().find_map(|command| match command {
        OutputCommand::CopyText(text) => Some(text.lines().map(str::to_owned).collect()),
        _ => None,
    })
}

fn paths(root: &Path, relative: &[&str]) -> BTreeSet<String> {
    relative.iter().map(|path| root.join(path).to_string_lossy().into_owned()).collect()
}

#[test]
fn two_selected_files_put_both_full_paths_on_the_clipboard_one_per_line() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    let mut harness = harness(FolderCompare::of_folders(&["left".into(), "right".into()], root.path()));
    assert!(!harness.state_mut().run(COPY_FILENAME), "with nothing selected there is nothing to copy");

    click(&mut harness, &on(ON_LEFT, "same.txt"), Modifiers::NONE);
    click(&mut harness, &on(ON_LEFT, "only-left.txt"), Modifiers::COMMAND);
    let expected = paths(root.path(), &["left/same.txt", "left/only-left.txt"]);
    assert_eq!(copy(&mut harness), Some(expected));
}

#[test]
fn an_item_in_an_open_folder_gets_its_folder_in_its_path_and_a_folder_its_own() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    fs::write(root.path().join("right/both/inner.txt"), "inner").unwrap();
    let mut harness = harness(FolderCompare::of_folders(&["left".into(), "right".into()], root.path()));
    assert!(harness.state_mut().run("folder-compare.expand_all"));
    settle(&mut harness);

    click(&mut harness, &on(ON_RIGHT, "inner.txt"), Modifiers::NONE);
    click(&mut harness, &on(ON_LEFT, "both"), Modifiers::NONE);
    let expected = paths(root.path(), &["right/both/inner.txt", "left/both"]);
    assert_eq!(copy(&mut harness), Some(expected), "the selection on both sides, each under its own base");
}
