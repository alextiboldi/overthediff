//! Editing in the panes (E07-S1-T5), driven by keyboard through the real shell, so its shortcuts
//! hand the editor what a text field gets, and read back through the accessibility tree.

use egui::accesskit::Role;
use egui::os::OperatingSystem;
use egui::{Event, ImeEvent, Key, Modifiers, OutputCommand, PointerButton, Pos2, ViewportCommand, ViewportId, vec2};
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

/// The status bar's overall result.
fn result(harness: &Harness<'_, Shell>) -> Option<String> {
    view(harness).status().result.map(|field| field.value)
}

/// The left pane's lines.
fn left_lines(harness: &Harness<'_, Shell>) -> Vec<String> {
    view(harness).pane(0).unwrap().lines.iter().map(|line| line.text.clone()).collect()
}

/// Whether this frame's output puts `text` on the clipboard.
fn copied(harness: &Harness<'_, Shell>, text: &str) -> bool {
    harness.output().platform_output.commands.iter().any(|c| matches!(c, OutputCommand::CopyText(copied) if copied == text))
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
    let editor = harness.get_by(|node| node.is_focused()).accesskit_node();
    let selection = editor.text_selection();
    assert_eq!(selection.as_ref().map(|range| range.text()).as_deref(), Some("left"), "the editor publishes what is selected");
    assert!(selection.is_some_and(|range| !range.bounding_boxes().is_empty()), "and where it is on screen");
    let before = result(&harness);
    harness.event(Event::Text("right".to_owned()));
    harness.run();
    assert_ne!(result(&harness), before, "the status bar's result follows the typing");
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

#[test]
fn tab_and_shift_tab_leave_a_focused_pane() {
    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder, "one\n", "two\n");
    tab_to_editor(&mut harness, LEFT);
    let left = Some((Role::MultilineTextInput, LEFT.to_owned()));
    for modifiers in [Modifiers::NONE, Modifiers::SHIFT] {
        key(&mut harness, modifiers, Key::Tab);
        let now = focused(&harness).map(|(role, name, _)| (role, name));
        assert!(now.is_some() && now != left, "{modifiers:?}+Tab moves focus out of the pane, to {now:?}");
        tab_to_editor(&mut harness, LEFT);
    }
}

#[test]
fn redo_by_its_shortcuts_brings_back_what_undo_took() {
    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder, "a\n", "a\n");
    tab_to_editor(&mut harness, LEFT);
    harness.event(Event::Text("x".to_owned()));
    harness.run();
    key(&mut harness, Modifiers::COMMAND, Key::Z);
    assert_eq!(left_lines(&harness), ["a"]);
    key(&mut harness, Modifiers::COMMAND | Modifiers::SHIFT, Key::Z);
    assert_eq!(left_lines(&harness), ["xa"], "Shift+Command+Z redoes");

    // Windows and Linux also take Ctrl+Y.
    harness.ctx.set_os(OperatingSystem::Windows);
    let ctrl = Modifiers { ctrl: true, command: true, ..Modifiers::NONE };
    key(&mut harness, ctrl, Key::Z);
    assert_eq!(left_lines(&harness), ["a"]);
    key(&mut harness, ctrl, Key::Y);
    assert_eq!(left_lines(&harness), ["xa"], "Ctrl+Y redoes");
}

#[test]
fn the_cut_copy_and_paste_commands_reach_the_clipboard_on_the_next_draw() {
    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder, "first\nsecond\n", "first\nsecond\n");
    tab_to_editor(&mut harness, LEFT);
    key(&mut harness, Modifiers::SHIFT, Key::ArrowDown);

    // Run as a menu click runs them: the clipboard is reached when the view next draws.
    assert!(harness.state_mut().run("text-compare.copy"));
    harness.step();
    assert!(copied(&harness, "first\n"), "Copy puts the selection on the clipboard");
    harness.run();
    assert_eq!(left_lines(&harness), ["first", "second"], "and leaves it");
    assert!(harness.state_mut().run("text-compare.cut"));
    harness.step();
    assert!(copied(&harness, "first\n"), "Cut does too");
    harness.run();
    assert_eq!(left_lines(&harness), ["second"], "and deletes it");

    assert!(harness.state_mut().run("text-compare.paste"));
    harness.step();
    let asked = harness.output().viewport_output.get(&ViewportId::ROOT).is_some_and(|v| v.commands.contains(&ViewportCommand::RequestPaste));
    assert!(asked, "Paste asks the window for the clipboard's text");
    // Which the window sends as a paste event.
    harness.event(Event::Paste("first\n".to_owned()));
    harness.run();
    assert_eq!(left_lines(&harness), ["first", "second"], "the pane takes it at the caret");
}

#[test]
fn edit_delete_leaves_the_pane_alone_while_a_path_edit_has_focus() {
    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder, "one\n", "two\n");
    for _ in 0..40 {
        if focused(&harness).is_some_and(|(role, name, _)| role != Role::MultilineTextInput && name.starts_with(LEFT)) {
            break;
        }
        key(&mut harness, Modifiers::NONE, Key::Tab);
    }
    let path_edit = focused(&harness);
    assert!(path_edit.as_ref().is_some_and(|(role, _, _)| *role != Role::MultilineTextInput), "the Left path edit has focus: {path_edit:?}");

    assert!(!harness.state_mut().run("text-compare.delete"), "Delete is disabled");
    harness.run();
    assert_eq!(left_lines(&harness), ["one"], "the file is unchanged");
    assert_eq!(focused(&harness), path_edit, "and focus stays");
}

#[test]
fn an_input_methods_composition_shows_until_it_commits_into_the_focused_pane() {
    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder, "a\n", "a\n");
    assert!(harness.output().platform_output.ime.is_none(), "no input method without a focused editor");
    tab_to_editor(&mut harness, LEFT);
    assert!(harness.output().platform_output.ime.is_some(), "the focused editor turns input methods on");

    // A dead key's accent, then the letter it composes.
    harness.event(Event::Ime(ImeEvent::Preedit { text: "´".to_owned(), active_range_chars: None }));
    harness.run();
    assert_eq!(left_lines(&harness), ["a"], "a composition isn't in the text yet");
    harness.event(Event::Ime(ImeEvent::Commit("é".to_owned())));
    harness.run();
    assert_eq!(left_lines(&harness), ["éa"], "the committed text is typed at the caret");
    key(&mut harness, Modifiers::COMMAND, Key::Z);
    assert_eq!(left_lines(&harness), ["a"], "and undoes as typing");

    // A read-only pane takes no text, so it asks for no input method.
    harness.state_mut().view_mut::<TextCompareView>().unwrap().comparison_ended(End::Cancelled);
    harness.run();
    assert!(harness.output().platform_output.ime.is_none());
}

#[test]
fn recompare_files_empties_the_undo_history_that_swap_sides_keeps_and_the_status_bar_shows_the_cursor() {
    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder, "a\nbcd\n", "a\nbcd\n");
    tab_to_editor(&mut harness, LEFT);
    key(&mut harness, Modifiers::NONE, Key::ArrowDown);
    key(&mut harness, Modifiers::NONE, Key::End);
    harness.event(Event::Text("x".to_owned()));
    harness.run();
    let cursor = view(&harness).status().fields.into_iter().find(|field| field.id == "cursor").map(|field| field.value);
    assert!(cursor.as_deref().is_some_and(|at| at.contains('2') && at.contains('5')), "line 2, column 5, counted from 1: {cursor:?}");
    assert!(view(&harness).can_undo());

    // Swap Sides moves the edited pane, its undo history and all.
    for _ in 0..2 {
        assert!(harness.state_mut().run("text-compare.swap_sides"));
        harness.run();
    }
    assert!(view(&harness).can_undo(), "Swap Sides keeps the history");
    assert!(harness.state_mut().run("text-compare.recompare_files"));
    harness.run();
    assert_eq!(left_lines(&harness), ["a", "bcdx"], "the edit stays");
    assert!(!view(&harness).can_undo(), "but can't be undone");
}

/// The focused editor's published selection, as text: reading it fails if an end names a run that
/// isn't there.
fn published(harness: &Harness<'_, Shell>) -> Option<String> {
    harness.get_by(|node| node.is_focused()).accesskit_node().text_selection().map(|range| range.text())
}

#[test]
fn the_published_selection_names_drawn_text_wherever_the_caret_stands() {
    let folder = tempfile::tempdir().unwrap();
    let long = "x".repeat(300);
    let body: String = (1..300).map(|n| format!("line {n}\n")).collect();
    let mut harness = harness(&folder, &format!("a\n\n{long}\n{body}"), "b\n");
    tab_to_editor(&mut harness, LEFT);
    key(&mut harness, Modifiers::NONE, Key::ArrowDown);
    assert_eq!(published(&harness).as_deref(), Some(""), "on a blank line");
    key(&mut harness, Modifiers::NONE, Key::ArrowDown);
    key(&mut harness, Modifiers::NONE, Key::End);
    key(&mut harness, Modifiers::SHIFT, Key::ArrowLeft);
    assert_eq!(published(&harness).as_deref(), Some("x"), "past the first 255 characters of a long line");
    key(&mut harness, Modifiers::COMMAND, Key::End);
    assert_eq!(published(&harness).as_deref(), Some(""), "after the final line break");
    key(&mut harness, Modifiers::COMMAND, Key::A);
    let all = published(&harness).unwrap_or_default();
    assert!(all.ends_with("line 299\n") && !all.starts_with('a'), "from the first drawn line, with the top scrolled away: {all:?}");
    key(&mut harness, Modifiers::NONE, Key::Delete);
    assert!(view(&harness).pane(0).unwrap().lines.is_empty());
    assert_eq!(published(&harness), None, "an empty pane has no text to select in");
}
