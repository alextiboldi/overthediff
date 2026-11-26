//! The view's name filter (E08-S3-T1) in the real shell, with the rows read back through the
//! accessibility tree.

use otd_ui_folder_compare::{FolderCompare, ON_LEFT, ON_RIGHT};

use crate::view::{fixture, harness, on, rows, settle};

#[test]
fn a_name_filter_hides_the_excluded_rows_and_one_that_does_not_compile_keeps_the_previous_filter() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    let mut harness = harness(FolderCompare::of_folders(&["left".into(), "right".into()], root.path()));
    let names = |harness: &egui_kittest::Harness<'_, otd_ui::Shell>| rows(harness).concat();
    let unfiltered = names(&harness);

    let view = harness.state_mut().view_mut::<FolderCompare>().unwrap();
    view.set_name_filter("-only-*.txt;-both\\").unwrap();
    settle(&mut harness);
    let filtered = names(&harness);
    assert!(filtered.contains(&on(ON_LEFT, "same.txt")) && filtered.contains(&on(ON_RIGHT, "same.txt")), "{filtered:?}");
    assert!(filtered.len() < unfiltered.len() && !filtered.iter().any(|cell| cell.contains("only-") || cell.contains("both")), "{filtered:?}");

    let view = harness.state_mut().view_mut::<FolderCompare>().unwrap();
    assert!(view.set_name_filter("[").is_err());
    assert_eq!(view.name_filter(), "-only-*.txt;-both\\");
    settle(&mut harness);
    assert_eq!(names(&harness), filtered, "the rows stay filtered as before");
}
