//! The Side-by-Side and Top and Bottom layouts (E07-S9-T3), switched as the View menu switches them and
//! read back through the accessibility tree.

use egui::accesskit::Role;
use egui::{Key, Modifiers, Rect, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_ui::{Shell, View as _};
use otd_ui_text_compare::{LEFT, RIGHT, SESSION_TYPE, TextCompareView, commands};

/// The shell with Text Compare's commands, comparing two 300-line files that differ only in their
/// first line, so both panes have every row. The window is tall enough for stacked panes to show
/// some rows below their headers.
fn harness(folder: &tempfile::TempDir) -> Harness<'static, Shell> {
    let body: String = (1..300).map(|n| format!("line {n}\n")).collect();
    let mut view = TextCompareView::new();
    if let Some(session) = view.session() {
        session.specs = [("left.txt", format!("left\n{body}")), ("right.txt", format!("right\n{body}"))]
            .map(|(name, text)| {
                let path = folder.path().join(name);
                std::fs::write(&path, text).unwrap();
                path.display().to_string()
            })
            .to_vec();
    }
    let mut shell = Shell::new(vec![SESSION_TYPE]).with_commands(commands());
    shell.open(Box::new(view));
    let mut harness = Harness::builder().with_size(vec2(900.0, 1200.0)).build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), shell);
    harness.run();
    harness
}

fn run(harness: &mut Harness<'_, Shell>, id: &str) {
    assert!(harness.state_mut().run(id), "{id} ran");
    harness.run();
}

/// The body of the pane named `side`: its editor's rect.
fn body(harness: &Harness<'_, Shell>, side: &str) -> Rect {
    harness.get_by(|node| node.role() == Role::MultilineTextInput && node.label().as_deref() == Some(side)).rect()
}

/// Where the line `label` is drawn in the pane named `side`, if its row's middle is in view.
fn line(harness: &Harness<'_, Shell>, label: &str, side: &str) -> Option<Rect> {
    let pane = body(harness, side);
    harness
        .query_all_by_label(label)
        .filter(|node| node.accesskit_node().role() == Role::Label)
        .map(|node| node.rect())
        .find(|r| pane.contains(r.center()))
}

/// The highest-numbered "line N" drawn in the left pane: its last row in view.
fn last_shown(harness: &Harness<'_, Shell>) -> usize {
    (1..300).rev().find(|n| line(harness, &format!("line {n}"), LEFT).is_some()).expect("some line is shown")
}

fn focused(harness: &Harness<'_, Shell>) -> Option<(Role, String)> {
    let node = harness.query_by(|node| node.is_focused())?;
    let node = node.accesskit_node();
    Some((node.role(), node.label().unwrap_or_default()))
}

#[test]
fn switching_to_top_and_bottom_keeps_the_caret_line_in_view_and_the_editor_focused() {
    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder);
    for _ in 0..40 {
        if focused(&harness).is_some_and(|(role, name)| role == Role::MultilineTextInput && name == LEFT) {
            break;
        }
        harness.key_press(Key::Tab);
        harness.run();
    }
    // The caret down to the last row in view side by side, which the half-height pane can't show
    // without scrolling.
    let bottom = last_shown(&harness);
    for _ in 0..bottom {
        harness.key_press_modifiers(Modifiers::NONE, Key::ArrowDown);
        harness.run();
    }
    let caret = format!("line {bottom}");
    assert_eq!(view(&harness).pane(0).map(|pane| pane.selection.head.line), Some(bottom));

    run(&mut harness, "text-compare.over_under");
    assert!(body(&harness, LEFT).bottom() <= body(&harness, RIGHT).top(), "the left pane is above the right one");
    assert!(line(&harness, "line 1", LEFT).is_none(), "the half-height pane scrolled to reach the caret");
    assert!(line(&harness, &caret, LEFT).is_some(), "the caret's line is still in view");
    assert_eq!(focused(&harness), Some((Role::MultilineTextInput, LEFT.to_owned())), "the editor keeps focus");
    assert_eq!(view(&harness).pane(0).map(|pane| pane.selection.head.line), Some(bottom), "and its caret");

    run(&mut harness, "text-compare.side_by_side");
    assert!(body(&harness, LEFT).right() <= body(&harness, RIGHT).left(), "the left pane is beside the right one again");
    assert!(line(&harness, &caret, LEFT).is_some(), "with the caret's line still in view");
}

#[test]
fn top_and_bottom_panes_still_scroll_together() {
    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder);
    run(&mut harness, "text-compare.over_under");
    let offset = |harness: &Harness<'_, Shell>, side: &str| {
        line(harness, "line 20", side).map(|r| r.top() - body(harness, side).top()).unwrap_or_else(|| panic!("line 20 is shown on the {side}"))
    };
    for side in [LEFT, RIGHT] {
        let before = offset(&harness, side);
        let node = harness
            .query_all_by_label("line 20")
            .find(|node| node.accesskit_node().role() == Role::Label && body(&harness, side).contains(node.rect().center()))
            .expect("line 20 is shown");
        node.scroll_down();
        harness.run();
        assert!(offset(&harness, side) < before, "scrolling the {side} pane moves its lines up");
        assert_eq!(offset(&harness, LEFT), offset(&harness, RIGHT), "both panes stay on one row after scrolling the {side} pane");
    }
}

#[test]
fn top_and_bottom_tab_order_follows_the_screen() {
    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder);
    run(&mut harness, "text-compare.over_under");
    // Tab from the first control to the bottom editor: no stop may sit above the one before it, so
    // the top pane's path and editor both come before the bottom pane's path.
    let mut previous = f32::NEG_INFINITY;
    for _ in 0..40 {
        harness.key_press(Key::Tab);
        harness.run();
        let Some(node) = harness.query_by(|node| node.is_focused()) else { continue };
        let top = node.rect().top();
        assert!(top >= previous, "{:?} at {top} comes after a control at {previous}", focused(&harness));
        previous = top;
        if focused(&harness) == Some((Role::MultilineTextInput, RIGHT.to_owned())) {
            return;
        }
    }
    panic!("Tab never reached the {RIGHT} editor");
}

fn view<'a>(harness: &'a Harness<'_, Shell>) -> &'a TextCompareView {
    harness.state().view::<TextCompareView>().expect("a Text Compare tab is active")
}
