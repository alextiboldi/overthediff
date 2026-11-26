//! Read-only panes and the reason each shows, read back through the accessibility tree.

use egui::accesskit::Live;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_core::task::End;
use otd_text_compare::ReadOnly;
use otd_ui::View as _;
use otd_ui_text_compare::{COMPARISON_CANCELLED, TextCompareView};

fn harness(specs: Vec<String>) -> Harness<'static, TextCompareView> {
    let mut view = TextCompareView::new();
    if let Some(session) = view.session() {
        session.specs = specs;
    }
    let mut harness = Harness::builder().build_ui_state(|ui, view: &mut TextCompareView| view.ui(ui), view);
    harness.run();
    harness
}

#[test]
fn a_cancelled_comparison_leaves_both_panes_read_only_and_says_so_until_one_completes() {
    let folder = tempfile::tempdir().unwrap();
    let specs: Vec<String> = ["left.txt", "right.txt"]
        .into_iter()
        .map(|name| {
            let path = folder.path().join(name);
            std::fs::write(&path, name).unwrap();
            path.display().to_string()
        })
        .collect();
    let mut harness = harness(specs);
    assert_eq!((harness.state().read_only(0), harness.state().read_only(1)), (None, None), "a temporary folder is writable");

    harness.state_mut().comparison_ended(End::Cancelled);
    harness.run();
    for side in 0..2 {
        assert_eq!(harness.state().read_only(side), Some(ReadOnly::ComparisonCancelled));
    }
    let reasons = harness.query_all_by_label(COMPARISON_CANCELLED).collect::<Vec<_>>();
    assert_eq!(reasons.len(), 2, "each pane says why");
    assert!(reasons.iter().all(|node| node.accesskit_node().live() != Live::Off), "the change is announced, since focus is elsewhere");
    drop(reasons);

    harness.state_mut().comparison_ended(End::Completed);
    harness.run();
    assert_eq!((harness.state().read_only(0), harness.state().read_only(1)), (None, None));
    assert!(harness.query_by_label(COMPARISON_CANCELLED).is_none());
}

/// macOS mounts its system volume read-only, so a file there is on a read-only location.
#[cfg(target_os = "macos")]
#[test]
fn a_file_on_a_read_only_volume_opens_read_only_and_says_why() {
    use otd_ui_text_compare::LOCATION;

    let harness = harness(vec!["/System/Library/CoreServices/SystemVersion.plist".to_owned(), String::new()]);
    assert_eq!(harness.state().read_only(0), Some(ReadOnly::Location));
    assert!(harness.query_by_label(LOCATION).is_some());

    // A completed comparison doesn't lift it.
    let mut harness = harness;
    harness.state_mut().comparison_ended(End::Completed);
    harness.run();
    assert_eq!(harness.state().read_only(0), Some(ReadOnly::Location));
}
