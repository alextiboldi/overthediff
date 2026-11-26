//! The hidden font size commands, run as a script or a user-added menu entry runs them, measured
//! through the accessibility tree.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use otd_ui::{Shell, View as _};
use otd_ui_text_compare::{SESSION_TYPE, TextCompareView, commands};

/// The heights of the line `shared` in each pane, as drawn.
fn heights(harness: &Harness<'_, Shell>) -> Vec<f32> {
    harness.query_all_by_label("shared").map(|node| node.rect().height()).collect()
}

fn run(harness: &mut Harness<'_, Shell>, id: &str) {
    assert!(harness.state_mut().run(id), "{id} ran");
    harness.run();
}

#[test]
fn two_increases_then_a_reset_return_both_panes_text_to_the_configured_size() {
    let folder = tempfile::tempdir().unwrap();
    let mut view = TextCompareView::new();
    view.session().expect("a session").specs = [("left.txt", "left\nshared\n"), ("right.txt", "right\nshared\n")]
        .map(|(name, text)| {
            let path = folder.path().join(name);
            std::fs::write(&path, text).unwrap();
            path.display().to_string()
        })
        .to_vec();
    let mut shell = Shell::new(vec![SESSION_TYPE]).with_commands(commands());
    shell.open(Box::new(view));
    let mut harness = Harness::builder().build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), shell);
    harness.run();
    let configured = heights(&harness);
    assert_eq!(configured.len(), 2, "the line is drawn in both panes");

    run(&mut harness, "text-compare.increase_font_size");
    run(&mut harness, "text-compare.increase_font_size");
    let larger = heights(&harness);
    assert!(larger.iter().zip(&configured).all(|(larger, configured)| larger > configured), "both panes' text grew: {larger:?} from {configured:?}");

    run(&mut harness, "text-compare.reset_font_size");
    assert_eq!(heights(&harness), configured, "both panes are back at the configured size");
}
