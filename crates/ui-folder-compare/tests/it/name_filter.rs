//! The view's name filter (E08-S3-T1) in the real shell, with the rows read back through the
//! accessibility tree.

use egui_kittest::Harness;
use otd_ui::Shell;
use otd_ui_folder_compare::{FolderCompare, ON_LEFT, ON_RIGHT};

use crate::view::{fixture, harness, on, rows, settle};

const MASKS: &str = "-only-*.txt;-both\\";

/// The fixture's view, filtered by [`MASKS`].
fn filtered(root: &std::path::Path) -> Harness<'static, Shell> {
    fixture(root);
    let mut harness = harness(FolderCompare::of_folders(&["left".into(), "right".into()], root));
    harness.state_mut().view_mut::<FolderCompare>().unwrap().set_name_filter(MASKS).unwrap();
    settle(&mut harness);
    harness
}

#[test]
fn a_name_filter_stops_drawing_the_rows_it_excludes() {
    let root = tempfile::tempdir().unwrap();
    let cells = rows(&filtered(root.path())).concat();
    assert!(cells.contains(&on(ON_LEFT, "same.txt")) && cells.contains(&on(ON_RIGHT, "same.txt")), "{cells:?}");
    assert!(!cells.iter().any(|cell| cell.contains("only-") || cell.contains("both")), "{cells:?}");
}

#[test]
fn a_name_filter_that_does_not_compile_keeps_the_previous_one() {
    let root = tempfile::tempdir().unwrap();
    let mut harness = filtered(root.path());
    let before = rows(&harness);
    let view = harness.state_mut().view_mut::<FolderCompare>().unwrap();
    assert!(view.set_name_filter("[").is_err() && view.name_filter() == MASKS);
    settle(&mut harness);
    assert_eq!(rows(&harness), before);
}
