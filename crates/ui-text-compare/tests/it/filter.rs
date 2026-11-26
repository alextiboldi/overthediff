//! The display filters (E07-S3-T2), run by id as the View menu and toolbar run them, and read back
//! through the accessibility tree.

use std::collections::BTreeMap;

use egui::accesskit::Role;
use egui::{Key, Modifiers, Rect, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_core::command::Mark;
use otd_text_compare::Pos;
use otd_ui::{Shell, View as _};
use otd_ui_text_compare::{LEFT, RIGHT, SESSION_TYPE, TextCompareView, commands};

const ALL: &str = "text-compare.show_all";
const DIFFERENCES: &str = "text-compare.show_differences";
const SAME: &str = "text-compare.show_same";
const CONTEXT: &str = "text-compare.show_context";
const NONE: &str = "text-compare.show_none";
const FILTERS: [&str; 5] = [ALL, DIFFERENCES, SAME, CONTEXT, NONE];

/// Lines 1 to 40 on each side, the same but for lines 10 and 30, so every row has a line on both
/// sides and the differences sit between long runs of matching lines. The window shows every row.
fn harness(folder: &tempfile::TempDir) -> Harness<'static, Shell> {
    harness_of(folder, |side| (1..=40).map(|n| if n == 10 || n == 30 { format!("{side} {n}\n") } else { format!("line {n}\n") }).collect())
}

/// A tab comparing `text("left")` with `text("right")`.
fn harness_of(folder: &tempfile::TempDir, text: impl Fn(&str) -> String) -> Harness<'static, Shell> {
    let mut view = TextCompareView::new();
    if let Some(session) = view.session() {
        session.specs = [("left.txt", text("left")), ("right.txt", text("right"))]
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

fn body(harness: &Harness<'_, Shell>, side: &str) -> Rect {
    harness.get_by(|node| node.role() == Role::MultilineTextInput && node.label().as_deref() == Some(side)).rect()
}

/// Line number (from 1) to where its text is drawn in the pane named `side`, for each line drawn.
fn drawn(harness: &Harness<'_, Shell>, side: &str) -> BTreeMap<usize, f32> {
    let pane = body(harness, side);
    let own = if side == LEFT { "left" } else { "right" };
    (1..=40)
        .filter_map(|n| {
            [format!("line {n}"), format!("{own} {n}")].iter().find_map(|label| {
                harness
                    .query_all_by_label(label)
                    .filter(|node| node.accesskit_node().role() == Role::Label)
                    .map(|node| node.rect())
                    .find(|r| pane.contains(r.center()))
                    .map(|r| (n, r.top()))
            })
        })
        .collect()
}

fn caret(harness: &Harness<'_, Shell>, side: usize) -> Option<usize> {
    view(harness).pane(side).map(|pane| pane.selection.head.line)
}

/// Focuses the left editor by Tab and moves its caret down to line index `line`.
fn caret_to(harness: &mut Harness<'_, Shell>, line: usize) {
    for _ in 0..40 {
        let focused = harness.query_by(|node| node.is_focused()).map(|node| (node.accesskit_node().role(), node.accesskit_node().label()));
        if focused.is_some_and(|(role, name)| role == Role::MultilineTextInput && name.as_deref() == Some(LEFT)) {
            break;
        }
        harness.key_press(Key::Tab);
        harness.run();
    }
    for _ in 0..line {
        harness.key_press_modifiers(Modifiers::NONE, Key::ArrowDown);
        harness.run();
    }
    assert_eq!(caret(harness, 0), Some(line), "the caret reached line index {line}");
}

#[test]
fn every_filter_shows_the_same_rows_in_both_panes() {
    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder);
    let mut shown = BTreeMap::new();
    for id in FILTERS {
        run(&mut harness, id);
        let left = drawn(&harness, LEFT);
        assert_eq!(left, drawn(&harness, RIGHT), "under {id} both panes draw the same lines on the same rows");
        shown.insert(id, left.into_keys().collect::<Vec<_>>());
    }
    let all: Vec<usize> = (1..=40).collect();
    assert_eq!(shown[ALL], all, "every line");
    assert_eq!(shown[DIFFERENCES], [10, 30], "the different lines only");
    assert_eq!(shown[SAME], all.iter().copied().filter(|n| *n != 10 && *n != 30).collect::<Vec<_>>(), "the matching lines only");
    let context = &shown[CONTEXT];
    assert!(context.contains(&9) && context.contains(&11) && !context.contains(&20), "the differences and a few lines round them: {context:?}");
    assert!(shown[DIFFERENCES].iter().all(|n| context.contains(n)), "with every difference");
    assert!(shown[NONE].is_empty(), "no lines");
}

#[test]
fn the_active_filter_is_the_checked_one() {
    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder);
    for id in FILTERS {
        run(&mut harness, id);
        let shell = harness.state();
        for command in shell.commands().iter().filter(|command| FILTERS.contains(&command.id)) {
            assert_eq!((command.state)(shell).mark, Some(Mark::Radio(command.id == id)), "{} after {id}", command.id);
        }
    }
}

#[test]
fn switching_filters_keeps_the_caret_on_its_line_while_that_line_is_shown() {
    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder);
    // Line 10, a different line, which every filter but Matching and None shows.
    caret_to(&mut harness, 9);
    for id in [DIFFERENCES, CONTEXT, ALL, DIFFERENCES] {
        run(&mut harness, id);
        assert_eq!(caret(&harness, 0), Some(9), "under {id}");
        assert!(drawn(&harness, LEFT).contains_key(&10), "and its line is drawn under {id}");
    }
}

#[test]
fn a_caret_on_a_line_the_filter_hides_moves_to_a_shown_line_of_its_pane() {
    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder);
    caret_to(&mut harness, 0);
    run(&mut harness, DIFFERENCES);
    for side in [0, 1] {
        let line = caret(&harness, side).expect("a caret");
        assert!(line == 9 || line == 29, "side {side}'s caret is on a shown line, not {line}");
    }
    run(&mut harness, SAME);
    let line = caret(&harness, 0).expect("a caret");
    assert!(line != 9 && drawn(&harness, LEFT).contains_key(&(line + 1)), "the caret left the hidden line for a drawn one, line index {line}");
}

#[test]
fn the_arrows_skip_the_lines_a_filter_hides() {
    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder);
    caret_to(&mut harness, 9);
    run(&mut harness, DIFFERENCES);
    harness.key_press_modifiers(Modifiers::NONE, Key::ArrowDown);
    harness.run();
    assert_eq!(caret(&harness, 0), Some(29), "down to the next shown line");
    harness.key_press_modifiers(Modifiers::NONE, Key::ArrowDown);
    harness.run();
    assert_eq!(caret(&harness, 0), Some(40), "down past the hidden rest to the place after the final line break, which always shows");
    harness.key_press_modifiers(Modifiers::NONE, Key::ArrowUp);
    harness.key_press_modifiers(Modifiers::NONE, Key::ArrowUp);
    harness.run();
    assert_eq!(caret(&harness, 0), Some(9), "up to the shown lines before");
}

/// Crossing hidden lines sideways lands at the shown line's edge the way the caret went; up and
/// down keep the column the caret had, not the hidden line's.
#[test]
fn crossing_hidden_lines_lands_on_the_right_column() {
    let folder = tempfile::tempdir().unwrap();
    // Lines 0 and 3 differ; lines 1 and 2, a short line and a blank one, match and hide.
    let mut harness = harness_of(&folder, |side| format!("{side} a long different line\nab\n\n{side} x\ntail\n"));
    caret_to(&mut harness, 3);
    run(&mut harness, DIFFERENCES);
    let at = |harness: &Harness<'_, Shell>| view(harness).pane(0).map(|pane| pane.selection.head);
    let end_of_first = view(&harness).pane(0).map(|pane| pane.line_end(0));
    harness.key_press_modifiers(Modifiers::NONE, Key::ArrowLeft);
    harness.run();
    assert_eq!(at(&harness), end_of_first, "left from a line's start lands at the end of the shown line above");
    harness.key_press_modifiers(Modifiers::NONE, Key::ArrowRight);
    harness.run();
    assert_eq!(at(&harness), Some(Pos { line: 3, column: 0 }), "right from a line's end lands at the start of the shown line below");
    harness.key_press_modifiers(Modifiers::NONE, Key::ArrowRight);
    harness.key_press_modifiers(Modifiers::NONE, Key::ArrowUp);
    harness.run();
    assert_eq!(at(&harness), Some(Pos { line: 0, column: 1 }), "up keeps the caret's column across a blank hidden line");
}

#[test]
fn select_all_selects_from_the_first_shown_line_to_the_last() {
    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder);
    caret_to(&mut harness, 9);
    run(&mut harness, DIFFERENCES);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness.run();
    let pane = view(&harness).pane(0).expect("a pane");
    let (start, end) = pane.selection.range();
    assert_eq!((start.line, end.line), (9, 29), "by key, from the first shown line to the last, not the hidden lines round them");
    assert_eq!(end, pane.line_end(29), "to the end of the last shown line");
    run(&mut harness, NONE);
    run(&mut harness, "text-compare.select_all");
    assert!(view(&harness).pane(0).expect("a pane").selection.is_empty(), "by command, with nothing shown, nothing");
    run(&mut harness, ALL);
    run(&mut harness, "text-compare.select_all");
    let pane = view(&harness).pane(0).expect("a pane");
    assert_eq!(pane.selection.range(), (Pos::default(), pane.end()), "with every line shown, the whole text");
}

/// With no final line break there is no place after it, so nothing past the last shown line.
#[test]
fn without_a_final_line_break_the_caret_stays_on_shown_lines() {
    let folder = tempfile::tempdir().unwrap();
    let text = |side: &str| -> String {
        (1..=40).map(|n| if n == 10 || n == 30 { format!("{side} {n}") } else { format!("line {n}") }).collect::<Vec<_>>().join("\n")
    };
    let mut harness = harness_of(&folder, text);
    caret_to(&mut harness, 38);
    run(&mut harness, DIFFERENCES);
    assert_eq!(caret(&harness, 0), Some(29), "a hidden caret line moves to the nearest shown line");
    harness.key_press_modifiers(Modifiers::NONE, Key::ArrowDown);
    harness.run();
    assert_eq!(caret(&harness, 0), Some(29), "and no further, with nothing shown below");
}

/// Select All right after the panes are compared again, before they are drawn, still follows the
/// filter.
#[test]
fn select_all_follows_the_filter_before_the_next_draw() {
    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder);
    run(&mut harness, DIFFERENCES);
    let view = harness.state_mut().view_mut::<TextCompareView>().expect("a Text Compare tab is active");
    view.undo();
    view.select_all();
    let (start, end) = view.pane(0).expect("a pane").selection.range();
    assert_eq!((start.line, end.line), (9, 29), "from the first shown line to the last");
}

fn view<'a>(harness: &'a Harness<'_, Shell>) -> &'a TextCompareView {
    harness.state().view::<TextCompareView>().expect("a Text Compare tab is active")
}
