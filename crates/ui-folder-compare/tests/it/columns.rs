//! The column chooser, run as *View > Columns* runs it and from a header's right-click menu.

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_core::i18n::t;
use otd_ui::Shell;
use otd_ui_folder_compare::{FolderCompare, SIZE};

use crate::view::{fixture, harness, rows, settle};

/// How many column headers, across both panes, read `caption`.
fn headers(harness: &Harness<'_, Shell>, caption: &str) -> usize {
    harness.query_all_by_role(Role::ColumnHeader).filter(|node| node.accesskit_node().label().as_deref() == Some(t(caption))).count()
}

fn cells(harness: &Harness<'_, Shell>) -> usize {
    rows(harness).iter().map(Vec::len).sum()
}

fn compared() -> (tempfile::TempDir, Harness<'static, Shell>) {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    let harness = harness(FolderCompare::of_folders(&["left".into(), "right".into()], root.path()));
    (root, harness)
}

#[test]
fn hiding_size_removes_it_from_both_panes_and_a_refresh_keeps_it_hidden() {
    let (_root, mut harness) = compared();
    assert_eq!(headers(&harness, SIZE), 2, "one Size header per pane");
    let before = cells(&harness);

    assert!(harness.state_mut().run("folder-compare.column.size"), "the Size item ran");
    harness.run();
    assert_eq!(headers(&harness, SIZE), 0, "no pane has a Size header");
    assert!(cells(&harness) < before, "and the rows lose their Size cells");

    for refresh in ["folder-compare.refresh", "folder-compare.full_refresh", "folder-compare.swap_sides"] {
        assert!(harness.state_mut().run(refresh), "{refresh} ran");
        settle(&mut harness);
        assert_eq!(headers(&harness, SIZE), 0, "Size stays hidden after {refresh}");
    }
}

#[test]
fn a_header_s_right_click_menu_hides_a_column() {
    let (_root, mut harness) = compared();
    harness.get_all_by_role(Role::ColumnHeader).next().expect("a header").click_secondary();
    harness.run();

    harness.get_by(|node| node.role() == Role::CheckBox && node.label().as_deref() == Some(t(SIZE))).click();
    harness.run();
    assert_eq!(headers(&harness, SIZE), 0, "no pane has a Size header");
}
