//! Editing in the panes (E07-S1-T5), driven by keyboard through the real shell, so its shortcuts
//! hand the editor what a text field gets, and read back through the accessibility tree.

use egui::accesskit::Role;
use egui::{Event, Key, Modifiers, OutputCommand, PointerButton, Pos2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_core::command::{self, Command, Mods};
use otd_core::session::Value;
use otd_core::task::End;
use otd_text_compare::Pos;
use otd_ui::{Shell, View as _};
use otd_ui_text_compare::{LEFT, SESSION_TYPE, TEXT_COMPARE, TextCompareView, commands};

/// The shell with one Text Compare tab comparing `left` and `right`, written to files in `folder`.
fn harness(folder: &tempfile::TempDir, left: &str, right: &str) -> Harness<'static, Shell> {
    harness_with(folder, left, right, Vec::new())
}

/// [`harness`], with `extra` commands beside Text Compare's.
fn harness_with(folder: &tempfile::TempDir, left: &str, right: &str, extra: Vec<Command<Shell>>) -> Harness<'static, Shell> {
    let mut view = TextCompareView::new();
    if let Some(session) = view.session() {
        session.specs = [("left.txt", left), ("right.txt", right)]
            .map(|(name, text)| {
                let path = folder.path().join(name);
                std::fs::write(&path, text).unwrap();
                path.display().to_string()
            })
            .to_vec();
    }
    let mut shell = Shell::new(vec![SESSION_TYPE]).with_commands(commands().into_iter().chain(extra).collect());
    shell.open(Box::new(view));
    let mut harness = Harness::builder().with_size(vec2(900.0, 600.0)).build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), shell);
    harness.run();
    harness
}

fn view<'a>(harness: &'a Harness<'_, Shell>) -> &'a TextCompareView {
    harness.state().view::<TextCompareView>().expect("a Text Compare tab is active")
}

fn key(harness: &mut Harness<'_, Shell>, modifiers: Modifiers, key: Key) {
    harness.key_press_modifiers(modifiers, key);
    harness.run();
}

/// The focused node's role, name and description.
fn focused(harness: &Harness<'_, Shell>) -> Option<(Role, String, Option<String>)> {
    let node = harness.query_by(|node| node.is_focused())?;
    let node = node.accesskit_node();
    Some((node.role(), node.label().unwrap_or_default(), node.description()))
}

/// Moves focus with Tab to the editor of the pane named `side`.
fn tab_to_editor(harness: &mut Harness<'_, Shell>, side: &str) {
    for _ in 0..40 {
        if focused(harness).is_some_and(|(role, name, _)| role == Role::MultilineTextInput && name == side) {
            return;
        }
        key(harness, Modifiers::NONE, Key::Tab);
    }
    panic!("Tab never reached the {side} editor");
}

/// The descriptions of the line nodes named `label`, one per pane that shows it.
fn said(harness: &Harness<'_, Shell>, label: &str) -> Vec<Option<String>> {
    harness
        .query_all_by_label(label)
        .filter(|node| node.accesskit_node().role() == Role::Label)
        .map(|node| node.accesskit_node().description())
        .collect()
}

#[test]
fn typing_the_other_sides_text_into_a_changed_line_clears_its_highlight_and_undo_brings_it_back() {
    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder, "one\nchanged left\nthree\n", "one\nchanged right\nthree\n");
    tab_to_editor(&mut harness, LEFT);
    key(&mut harness, Modifiers::NONE, Key::ArrowDown);
    key(&mut harness, Modifiers::NONE, Key::End);
    assert_eq!(focused(&harness).and_then(|(_, _, said)| said).as_deref(), Some("Changed"), "the caret's line says how it differs");

    // Select "left" and type the right side's word over it.
    for _ in 0.."left".len() {
        key(&mut harness, Modifiers::SHIFT, Key::ArrowLeft);
    }
    harness.event(Event::Text("right".to_owned()));
    harness.run();
    assert_eq!(said(&harness, "changed right"), [None, None], "both panes show the line, neither as a difference");
    assert!(said(&harness, "changed left").is_empty());
    assert_eq!(focused(&harness), Some((Role::MultilineTextInput, LEFT.to_owned(), None)), "focus stays in the pane, whose line now matches");

    // Undo by its shortcut, which the shell leaves to the focused pane.
    key(&mut harness, Modifiers::COMMAND, Key::Z);
    assert_eq!(said(&harness, "changed left"), [Some("Changed".to_owned())], "the difference is back");
    assert_eq!(said(&harness, "changed right"), [Some("Changed".to_owned())]);
    assert_eq!(focused(&harness).and_then(|(_, _, said)| said).as_deref(), Some("Changed"));
}

#[test]
fn cut_and_paste_through_the_clipboard_events_move_lines_and_the_comparison_follows() {
    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder, "first\nsecond\n", "first\nsecond\n");
    tab_to_editor(&mut harness, LEFT);
    key(&mut harness, Modifiers::SHIFT, Key::ArrowDown);

    // The platform sends Cut and Paste as events of their own, not as key presses.
    harness.event(Event::Cut);
    harness.step();
    let copied = harness.output().platform_output.commands.iter().any(|c| matches!(c, OutputCommand::CopyText(text) if text == "first\n"));
    assert!(copied, "the cut line goes to the clipboard with its line break");
    harness.run();
    assert_eq!(said(&harness, "first"), [Some("Inserted".to_owned())], "only the right pane has it now");

    // To the end, after the final line break, and paste the line there.
    key(&mut harness, Modifiers::COMMAND, Key::End);
    harness.event(Event::Paste("first\n".to_owned()));
    harness.run();
    let lines: Vec<&str> = view(&harness).pane(0).unwrap().lines.iter().map(|line| line.text.as_str()).collect();
    assert_eq!(lines, ["second", "first"], "pasted at the caret, with no blank line after it");
    assert_eq!(said(&harness, "first").len(), 2, "both panes show it again");
}

#[test]
fn the_edit_menus_commands_act_on_the_chosen_pane_and_give_it_focus() {
    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder, "same\n", "same\n");
    assert!(!view(&harness).can_undo(), "nothing to undo yet");

    // Run as a menu click runs them, with focus outside the pane.
    for id in ["text-compare.select_all", "text-compare.delete"] {
        assert!(harness.state_mut().run(id), "{id} runs");
        harness.run();
    }
    assert!(view(&harness).pane(0).unwrap().lines.iter().all(|line| line.text.is_empty()), "Select All then Delete empties the pane");
    assert_eq!(said(&harness, "same"), [Some("Inserted".to_owned())]);
    assert_eq!(focused(&harness).map(|(role, name, _)| (role, name)), Some((Role::MultilineTextInput, LEFT.to_owned())), "the pane has focus");

    assert!(harness.state_mut().run("text-compare.undo"));
    harness.run();
    assert_eq!(said(&harness, "same"), [None, None]);
    assert!(harness.state_mut().run("text-compare.redo"));
    harness.run();
    assert_eq!(said(&harness, "same"), [Some("Inserted".to_owned())]);
}

#[test]
fn a_read_only_pane_refuses_typing_and_the_edit_commands_but_still_moves_its_caret() {
    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder, "one\ntwo\n", "one\ntwo\n");
    harness.state_mut().view_mut::<TextCompareView>().unwrap().comparison_ended(End::Cancelled);
    harness.run();
    tab_to_editor(&mut harness, LEFT);
    let editor = harness.get_by(|node| node.is_focused());
    assert!(editor.accesskit_node().is_read_only(), "the pane says it is read-only");

    key(&mut harness, Modifiers::NONE, Key::ArrowDown);
    harness.event(Event::Text("x".to_owned()));
    key(&mut harness, Modifiers::NONE, Key::Delete);
    assert_eq!(view(&harness).pane(0).unwrap().selection.head.line, 1, "the caret moves");
    let lines: Vec<&str> = view(&harness).pane(0).unwrap().lines.iter().map(|line| line.text.as_str()).collect();
    assert_eq!(lines, ["one", "two"], "nothing typed or deleted");
    assert!(!harness.state_mut().run("text-compare.paste"), "Paste is disabled");
    assert!(harness.state_mut().run("text-compare.select_all"), "Select All still works");
}

#[test]
fn moving_the_caret_past_the_viewport_scrolls_both_panes_to_it() {
    let folder = tempfile::tempdir().unwrap();
    let body: String = (1..300).map(|n| format!("line {n}\n")).collect();
    let mut harness = harness(&folder, &format!("left\n{body}"), &format!("right\n{body}"));
    tab_to_editor(&mut harness, LEFT);
    assert!(said(&harness, "line 299").is_empty(), "the last line starts out of view");

    key(&mut harness, Modifiers::COMMAND, Key::End);
    let rows: Vec<f32> = harness
        .query_all_by_label("line 299")
        .filter(|node| node.accesskit_node().role() == Role::Label)
        .map(|node| node.rect().center().y)
        .collect();
    assert_eq!(rows.len(), 2, "both panes show the caret's line");
    assert_eq!(rows[0], rows[1], "on one row");
}

#[test]
fn plain_keys_type_in_the_focused_pane_instead_of_running_a_shortcut_bound_to_them() {
    let folder = tempfile::tempdir().unwrap();
    // A plain-key shortcut, such as a later Text Compare command could have.
    let plain = Command::new("test.new_tab", "New Tab", Shell::new_tab).views(&[TEXT_COMPARE]).shortcut(Mods::NONE, command::Key::Q);
    let mut harness = harness_with(&folder, "a\n", "a\n", vec![plain]);
    tab_to_editor(&mut harness, LEFT);
    let tabs = harness.query_all_by_role(Role::Tab).count();

    // A typed key arrives as its press and its text.
    harness.key_press(Key::Q);
    harness.event(Event::Text("q".to_owned()));
    harness.run();
    assert_eq!(harness.query_all_by_role(Role::Tab).count(), tabs, "the shortcut waits while the pane has focus");
    assert_eq!(view(&harness).pane(0).unwrap().lines[0].text, "qa", "the key types");
}

/// Presses and releases the primary button at `at`, with `modifiers` held.
fn click(harness: &mut Harness<'_, Shell>, at: Pos2, modifiers: Modifiers) {
    harness.event(Event::ModifiersChanged(modifiers));
    harness.hover_at(at);
    for pressed in [true, false] {
        harness.event(Event::PointerButton { pos: at, button: PointerButton::Primary, pressed, modifiers });
        harness.step();
    }
    harness.event(Event::ModifiersChanged(Modifiers::NONE));
    harness.run();
}

/// The rect of the left pane's line named `label`: the leftmost node with that name.
fn left_line(harness: &Harness<'_, Shell>, label: &str) -> egui::Rect {
    let rects = harness.query_all_by_label(label).filter(|node| node.accesskit_node().role() == Role::Label).map(|node| node.rect());
    rects.min_by(|a, b| a.left().total_cmp(&b.left())).unwrap_or_else(|| panic!("{label} is shown"))
}

#[test]
fn a_press_on_a_line_focuses_its_pane_with_the_caret_there_and_a_shift_press_selects_to_it() {
    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder, "one\ntwo\nthree\n", "one\ntwo\nthree\n");
    let two = left_line(&harness, "two");
    click(&mut harness, two.left_center() + vec2(1.0, 0.0), Modifiers::NONE);
    assert_eq!(focused(&harness).map(|(role, name, _)| (role, name)), Some((Role::MultilineTextInput, LEFT.to_owned())), "the pane takes focus");
    assert_eq!(view(&harness).pane(0).unwrap().selection.head, Pos { line: 1, column: 0 }, "the caret is where the press was");

    let three = left_line(&harness, "three");
    // Past the line's end, short of the scroll bar.
    click(&mut harness, three.left_center() + vec2(three.width() / 2.0, 0.0), Modifiers::SHIFT);
    let selection = view(&harness).pane(0).unwrap().selection;
    assert_eq!(selection.range(), (Pos { line: 1, column: 0 }, Pos { line: 2, column: 5 }), "Shift extends the selection to the press");
}

#[test]
fn insert_text_edits_the_chosen_pane_without_the_window() {
    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder, "one\n", "two\n");
    let commands = commands();
    let args = [("text", Value::Text("two\n".to_owned()))];
    command::run(&commands, harness.state_mut(), "text-compare.insert_text", &args, false).expect("it runs without the window");
    command::run(&commands, harness.state_mut(), "text-compare.delete", &[], false).expect("so does Delete");
    harness.run();
    let lines: Vec<&str> = view(&harness).pane(0).unwrap().lines.iter().map(|line| line.text.as_str()).collect();
    assert_eq!(lines, ["two", "ne"], "inserted at the caret, then the next character deleted");
    assert!(said(&harness, "two").iter().all(Option::is_none), "and compared again");
}
