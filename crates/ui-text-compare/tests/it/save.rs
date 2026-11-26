//! Saving the panes (E07-S1-T6): Save File by its shortcut from the focused editor, Save File As
//! through a stand-in for the OS save picker and without the window, a failed save, and the
//! unsaved-changes prompts on a real edit, read back through the files and the accessibility tree.

use std::path::{Path, PathBuf};

use egui::accesskit::{Live, Role};
use egui::{Event, Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_core::command;
use otd_core::i18n::{fmt, t};
use otd_core::session::Value;
use otd_text_compare::ReadOnly;
use otd_ui::path_edit::{Answer, PickerKind, Request};
use otd_ui::{CANCEL, DONT_SAVE, HOME, SAVE, SAVE_CHANGES, Shell, View as _};
use otd_ui_text_compare::{LEFT, NAME, RIGHT, SESSION_TYPE, TextCompareView, UNSAVED_CHANGES, commands};

use crate::open::{Window, harness as window_harness};

/// The window with one Text Compare tab comparing the files `left` and `right` in `folder`.
fn harness(folder: &Path, left: &[u8], right: &[u8]) -> (Harness<'static, Window>, PathBuf) {
    let [left_path, right_path] = [("left.txt", left), ("right.txt", right)].map(|(name, bytes)| {
        let path = folder.join(name);
        std::fs::write(&path, bytes).unwrap();
        path
    });
    let mut view = TextCompareView::new();
    if let Some(session) = view.session() {
        session.specs = vec![left_path.display().to_string(), right_path.display().to_string()];
    }
    let mut shell = Shell::new(vec![SESSION_TYPE]).with_commands(commands());
    shell.open(Box::new(view));
    (window_harness(shell), left_path)
}

fn view<'a>(harness: &'a Harness<'_, Window>) -> &'a TextCompareView {
    harness.state().shell.view::<TextCompareView>().expect("a Text Compare tab is active")
}

fn press(harness: &mut Harness<'_, Window>, modifiers: Modifiers, key: Key) {
    harness.key_press_modifiers(modifiers, key);
    harness.run();
}

fn focused(harness: &Harness<'_, Window>) -> Option<(Role, String)> {
    let node = harness.query_by(|node| node.is_focused())?;
    let node = node.accesskit_node();
    Some((node.role(), node.label().unwrap_or_default()))
}

/// Moves focus with Tab to the node named `name` in the role `role`.
fn tab_to(harness: &mut Harness<'_, Window>, role: Role, name: &str) {
    for _ in 0..40 {
        if focused(harness) == Some((role, name.to_owned())) {
            return;
        }
        press(harness, Modifiers::NONE, Key::Tab);
    }
    panic!("Tab never reached {name:?}");
}

/// Types `text` at the start of the left pane's first line, from its focused editor.
fn edit_left(harness: &mut Harness<'_, Window>, text: &str) {
    tab_to(harness, Role::MultilineTextInput, LEFT);
    press(harness, Modifiers::COMMAND, Key::Home);
    harness.event(Event::Text(text.to_owned()));
    harness.run();
}

fn utf16_le(text: &str) -> Vec<u8> {
    std::iter::once(0xFEFF).chain(text.encode_utf16()).flat_map(u16::to_le_bytes).collect()
}

fn tab_names(harness: &Harness<'_, Window>) -> Vec<String> {
    harness.query_all_by_role(Role::Tab).map(|tab| tab.accesskit_node().label().unwrap_or_default()).collect()
}

fn prompt(harness: &Harness<'_, Window>) -> Option<String> {
    harness.query_by_role(Role::AlertDialog).map(|dialog| dialog.accesskit_node().label().unwrap_or_default())
}

#[test]
fn save_file_from_the_editor_keeps_the_encoding_and_line_endings_so_only_the_edited_bytes_change() {
    let folder = tempfile::tempdir().unwrap();
    let (mut harness, left) = harness(folder.path(), &utf16_le("one\r\ntwo\r\n"), b"one\ntwo\n");
    assert!(harness.query_by_label(UNSAVED_CHANGES).is_none(), "a pane just loaded has nothing unsaved");

    edit_left(&mut harness, "X");
    let mark = harness.get_by_label(UNSAVED_CHANGES);
    assert_eq!(mark.accesskit_node().live(), Live::Polite, "the mark is announced");
    assert!(view(&harness).unsaved(0) && !view(&harness).unsaved(1), "only the edited pane is marked");

    press(&mut harness, Modifiers::COMMAND, Key::S);
    assert_eq!(std::fs::read(&left).unwrap(), utf16_le("Xone\r\ntwo\r\n"), "UTF-16 LE, its BOM and CRLF kept");
    assert!(harness.query_by_label(UNSAVED_CHANGES).is_none(), "the save clears the mark");
    assert_eq!(focused(&harness), Some((Role::MultilineTextInput, LEFT.to_owned())), "focus stays in the pane");
}

#[test]
fn save_file_as_asks_where_through_the_save_picker_and_the_pane_then_shows_that_file() {
    let folder = tempfile::tempdir().unwrap();
    let (mut harness, left) = harness(folder.path(), b"one\n", b"one\n");
    let copy = folder.path().join("copy.txt");
    let answer = copy.clone();
    let asked = std::rc::Rc::new(std::cell::Cell::new(None));
    let seen = std::rc::Rc::clone(&asked);
    harness.state_mut().edits.as_mut().unwrap().set_override(Some(Box::new(move |request: &Request| {
        if let Request::File(dialog) = request {
            seen.set(Some(dialog.kind));
        }
        Answer::Paths(vec![answer.clone()])
    })));
    edit_left(&mut harness, "X");

    press(&mut harness, Modifiers::COMMAND | Modifiers::SHIFT, Key::S);
    for _ in 0..3 {
        harness.run();
    }
    assert_eq!(asked.get(), Some(PickerKind::Save), "the OS save picker asks");
    assert_eq!(std::fs::read(&copy).unwrap(), b"Xone\n");
    assert_eq!(std::fs::read(&left).unwrap(), b"one\n", "the old file is left as it was");
    let shown = copy.display().to_string();
    assert_eq!(view(&harness).pane(0).map(|pane| (pane.path.as_str(), pane.unsaved)), Some((shown.as_str(), false)));
    assert!(harness.query_by_label(&format!("Path: {shown}")).is_some(), "the header shows the new file");
    assert_eq!(view(&harness).pane(0).map(|pane| pane.lines[0].text.as_str()), Some("Xone"), "it isn't read again");
}

#[test]
fn save_file_as_with_a_path_works_without_the_window() {
    let folder = tempfile::tempdir().unwrap();
    let (mut harness, _) = harness(folder.path(), b"one\n", b"two\n");
    let copy = folder.path().join("copy.txt").display().to_string();
    let args = [("side", Value::Choice(LEFT.to_owned())), ("path", Value::Path(copy.clone()))];
    for id in ["text-compare.save_file_as", "text-compare.save_file_on_file_system_as"] {
        std::fs::remove_file(&copy).ok();
        command::run(&commands(), &mut harness.state_mut().shell, id, &args, false).expect("it runs without the window");
        assert_eq!(std::fs::read(&copy).unwrap(), b"one\n", "{id}");
    }
}

#[test]
fn a_script_saves_the_side_it_names_even_when_the_chosen_pane_cant_be_saved() {
    let folder = tempfile::tempdir().unwrap();
    // The left pane, chosen, is marked read-only.
    let set_read_only = |path: &Path, read_only| {
        let mut permissions = std::fs::metadata(path).unwrap().permissions();
        permissions.set_readonly(read_only);
        std::fs::set_permissions(path, permissions).unwrap();
    };
    let (mut harness, left) = harness(folder.path(), b"one\n", b"two\n");
    set_read_only(&left, true);
    harness.state_mut().shell.view_mut::<TextCompareView>().unwrap().reload_files();
    harness.run();
    assert_eq!(view(&harness).read_only(0), Some(ReadOnly::File));
    let right = folder.path().join("right.txt");
    std::fs::write(&right, b"").unwrap();
    let copy = folder.path().join("copy.txt").display().to_string();
    let side = |side: &str| ("side", Value::Choice(side.to_owned()));
    let run = |harness: &mut Harness<'_, Window>, id: &str, args: &[(&str, Value)]| {
        command::run(&commands(), &mut harness.state_mut().shell, id, args, false)
    };
    run(&mut harness, "text-compare.save_file", &[side(RIGHT)]).expect("the right pane saves");
    assert_eq!(std::fs::read(&right).unwrap(), b"two\n");
    run(&mut harness, "text-compare.save_file_as", &[side(RIGHT), ("path", Value::Path(copy.clone()))]).expect("and saves as");
    assert_eq!(std::fs::read(&copy).unwrap(), b"two\n");
    assert!(run(&mut harness, "text-compare.save_file", &[side(LEFT)]).is_err(), "the left pane still refuses");
    set_read_only(&left, false);
}

#[test]
fn with_no_side_the_save_commands_write_the_loaded_pane_when_the_chosen_one_has_no_file() {
    let folder = tempfile::tempdir().unwrap();
    let right = folder.path().join("right.txt");
    std::fs::write(&right, b"two\n").unwrap();
    let mut tab = TextCompareView::new();
    if let Some(session) = tab.session() {
        // The left path, chosen, fails to load.
        session.specs = vec![folder.path().join("missing.txt").display().to_string(), right.display().to_string()];
    }
    let mut shell = Shell::new(vec![SESSION_TYPE]).with_commands(commands());
    shell.open(Box::new(tab));
    let mut harness = window_harness(shell);
    assert!(view(&harness).pane(0).is_none() && view(&harness).chosen() == 0);
    let copy = folder.path().join("copy.txt");
    let path = [("path", Value::Path(copy.display().to_string()))];
    command::run(&commands(), &mut harness.state_mut().shell, "text-compare.save_file_as", &path, true).expect("the right pane saves as");
    assert_eq!(std::fs::read(&copy).unwrap(), b"two\n");
    command::run(&commands(), &mut harness.state_mut().shell, "text-compare.save_file", &[], true).expect("and saves");
}

#[test]
fn a_failed_save_says_why_in_the_pane_and_leaves_it_unsaved() {
    let folder = tempfile::tempdir().unwrap();
    // Windows-1252 text, which has no byte for ł.
    let (mut harness, left) = harness(folder.path(), b"caf\xE9\n", b"cafe\n");
    edit_left(&mut harness, "\u{142}");
    let failed = command::run(&commands(), &mut harness.state_mut().shell, "text-compare.save_file", &[], true);
    let Err(command::CommandError::Failed(message)) = failed else { panic!("the command says it failed, not {failed:?}") };
    assert!(message.contains("U+0142") && message.contains("line 1"), "it names the character and its line: {message}");
    harness.run();
    assert_eq!(std::fs::read(&left).unwrap(), b"caf\xE9\n", "nothing was written");
    let error = harness.get_by_label(&message);
    assert_eq!(error.accesskit_node().live(), Live::Assertive, "the pane says why, announced");
    assert!(view(&harness).unsaved(0));
}

#[test]
fn close_tab_on_an_edited_pane_offers_save_discard_or_cancel() {
    for answer in [CANCEL, DONT_SAVE, SAVE] {
        let folder = tempfile::tempdir().unwrap();
        let (mut harness, left) = harness(folder.path(), b"one\n", b"one\n");
        edit_left(&mut harness, "X");
        press(&mut harness, Modifiers::COMMAND, Key::W);
        assert_eq!(prompt(&harness), Some(fmt(t(SAVE_CHANGES), &[("title", t(NAME))])), "{answer}: Close Tab asks");
        tab_to(&mut harness, Role::Button, answer);
        press(&mut harness, Modifiers::NONE, Key::Enter);
        assert_eq!(prompt(&harness), None, "{answer}");
        let (tabs, saved): (&[&str], &[u8]) = match answer {
            CANCEL => (&[HOME, NAME], b"one\n"),
            DONT_SAVE => (&[HOME], b"one\n"),
            _ => (&[HOME], b"Xone\n"),
        };
        assert_eq!(tab_names(&harness), tabs, "{answer}");
        assert_eq!(std::fs::read(&left).unwrap(), saved, "{answer}");
        if answer == CANCEL {
            assert!(view(&harness).unsaved(0), "Cancel keeps the edit");
        }
    }
}

#[test]
fn reload_files_on_an_edited_pane_asks_first_and_dont_save_reads_the_file_again() {
    let folder = tempfile::tempdir().unwrap();
    let (mut harness, left) = harness(folder.path(), b"one\n", b"one\n");
    edit_left(&mut harness, "X");
    assert!(harness.state_mut().shell.run("text-compare.reload_files"));
    harness.run();
    assert!(prompt(&harness).is_some(), "Reload Files asks before it loses the edit");
    tab_to(&mut harness, Role::Button, DONT_SAVE);
    press(&mut harness, Modifiers::NONE, Key::Enter);
    assert_eq!(view(&harness).pane(0).map(|pane| (pane.lines[0].text.as_str(), pane.unsaved)), Some(("one", false)));
    assert_eq!(std::fs::read(&left).unwrap(), b"one\n");
}
