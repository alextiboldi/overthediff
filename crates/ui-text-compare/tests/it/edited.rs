//! Next Edit and Previous Edit (E07-S2-T3), driven by keyboard through the real shell and read
//! back through the accessibility tree.

use egui::accesskit::Role;
use egui::{Event, Key, Modifiers, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_ui::{Shell, View as _};
use otd_ui_text_compare::{LEFT, SESSION_TYPE, TextCompareView, commands};

fn harness(folder: &tempfile::TempDir, text: &str) -> Harness<'static, Shell> {
    let mut view = TextCompareView::new();
    if let Some(session) = view.session() {
        session.specs = ["left.txt", "right.txt"]
            .map(|name| {
                let path = folder.path().join(name);
                std::fs::write(&path, text).unwrap();
                path.display().to_string()
            })
            .to_vec();
    }
    let mut shell = Shell::new(vec![SESSION_TYPE]).with_commands(commands());
    shell.open(Box::new(view));
    let mut harness = Harness::builder().with_size(vec2(900.0, 600.0)).build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), shell);
    harness.run();
    harness
}

fn key(harness: &mut Harness<'_, Shell>, modifiers: Modifiers, key: Key) {
    harness.key_press_modifiers(modifiers, key);
    harness.run();
}

/// Moves focus with Tab to the left editor.
fn focus_left(harness: &mut Harness<'_, Shell>) {
    for _ in 0..40 {
        if harness.query_by(|node| node.is_focused()).is_some_and(|node| node.accesskit_node().role() == Role::MultilineTextInput) {
            break;
        }
        key(harness, Modifiers::NONE, Key::Tab);
    }
    assert_eq!(harness.get_by(|node| node.is_focused()).accesskit_node().label().as_deref(), Some(LEFT), "the left editor has focus");
}

fn caret_line(harness: &Harness<'_, Shell>) -> usize {
    harness.state().view::<TextCompareView>().unwrap().pane(0).unwrap().selection.head.line
}

/// How many panes draw a line node named `label`.
fn shown(harness: &Harness<'_, Shell>, label: &str) -> usize {
    harness.query_all_by_label(label).filter(|node| node.accesskit_node().role() == Role::Label).count()
}

#[test]
fn next_edit_from_line_1_visits_line_10_then_line_40_with_both_panes_showing_it() {
    let folder = tempfile::tempdir().unwrap();
    let text: String = (1..=100).map(|n| format!("line {n}\n")).collect();
    let mut harness = harness(&folder, &text);
    focus_left(&mut harness);

    // Edit lines 10 and 40, then back to line 1.
    for down in [9, 30] {
        for _ in 0..down {
            key(&mut harness, Modifiers::NONE, Key::ArrowDown);
        }
        key(&mut harness, Modifiers::NONE, Key::Home);
        harness.event(Event::Text("x".to_owned()));
        harness.run();
    }
    key(&mut harness, Modifiers::COMMAND, Key::Home);
    assert_eq!(caret_line(&harness), 0);
    assert_eq!(shown(&harness, "xline 40"), 0, "line 40 starts out of view");

    let next = Modifiers::ALT | Modifiers::SHIFT;
    key(&mut harness, next, Key::ArrowDown);
    assert_eq!(caret_line(&harness), 9, "the first Next Edit lands on line 10");
    key(&mut harness, next, Key::ArrowDown);
    assert_eq!(caret_line(&harness), 39, "the second on line 40");
    assert_eq!((shown(&harness, "xline 40"), shown(&harness, "line 40")), (1, 1), "both panes scrolled to it");
    assert!(
        harness.query_by(|node| node.is_focused()).is_some_and(|node| node.accesskit_node().label().as_deref() == Some(LEFT)),
        "focus stays in the pane"
    );
}

#[test]
fn past_the_last_edit_next_edit_is_disabled_and_its_chord_leaves_the_caret_alone() {
    let folder = tempfile::tempdir().unwrap();
    let text: String = (1..=5).map(|n| format!("line {n}\n")).collect();
    let mut harness = harness(&folder, &text);
    focus_left(&mut harness);
    key(&mut harness, Modifiers::NONE, Key::ArrowDown);
    harness.event(Event::Text("x".to_owned()));
    harness.run();
    key(&mut harness, Modifiers::NONE, Key::ArrowDown);

    assert!(!harness.state_mut().run("text-compare.next_edit"), "no edit lies further on");
    key(&mut harness, Modifiers::ALT | Modifiers::SHIFT, Key::ArrowDown);
    let pane = harness.state().view::<TextCompareView>().unwrap().pane(0).unwrap();
    assert_eq!(pane.selection.head.line, 2, "the chord doesn't move the caret");
    assert!(pane.selection.is_empty(), "or select");
}

#[test]
fn previous_edit_from_below_visits_the_later_edit_then_the_earlier_one() {
    let folder = tempfile::tempdir().unwrap();
    let text: String = (1..=10).map(|n| format!("line {n}\n")).collect();
    let mut harness = harness(&folder, &text);
    focus_left(&mut harness);

    // Edit lines 3 and 6, then go down to line 9.
    for down in [2, 3] {
        for _ in 0..down {
            key(&mut harness, Modifiers::NONE, Key::ArrowDown);
        }
        key(&mut harness, Modifiers::NONE, Key::Home);
        harness.event(Event::Text("x".to_owned()));
        harness.run();
    }
    for _ in 0..3 {
        key(&mut harness, Modifiers::NONE, Key::ArrowDown);
    }
    assert_eq!(caret_line(&harness), 8, "the caret starts below both edits");

    let previous = Modifiers::ALT | Modifiers::SHIFT;
    key(&mut harness, previous, Key::ArrowUp);
    assert_eq!(caret_line(&harness), 5, "the first Previous Edit lands on line 6");
    key(&mut harness, previous, Key::ArrowUp);
    assert_eq!(caret_line(&harness), 2, "the second on line 3");
    assert!(
        harness.query_by(|node| node.is_focused()).is_some_and(|node| node.accesskit_node().label().as_deref() == Some(LEFT)),
        "focus stays in the pane"
    );
}
