//! Loading another file over a pane's unsaved edits (E07-S1-T9): the path edit and *Open File* ask
//! Save / Don't Save / Cancel first, on a real edit, read back through the files, the view and the
//! accessibility tree.

use std::path::{Path, PathBuf};

use egui::accesskit::Role;
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_core::session::Value;
use otd_ui::path_edit::{Answer, Request};
use otd_ui::{CANCEL, DONT_SAVE, SAVE};
use otd_ui_text_compare::{LEFT, RIGHT, commands};

use crate::open::Window;
use crate::save::{edit_left, harness, press, prompt, tab_to, view};

fn other(folder: &Path) -> String {
    let path = folder.join("other.txt");
    std::fs::write(&path, "other\n").unwrap();
    path.display().to_string()
}

/// The left pane's path, the first line of its text and whether it is unsaved.
fn left(harness: &Harness<'_, Window>) -> Option<(String, String, bool)> {
    view(harness).pane(0).map(|pane| (pane.path.clone(), pane.lines[0].text.clone(), pane.unsaved))
}

fn path_edit_text(harness: &Harness<'_, Window>, side: &str) -> String {
    let edit = harness.query_all_by_role(Role::TextInput).find(|node| node.accesskit_node().label().as_deref() == Some(side)).unwrap();
    edit.accesskit_node().value().unwrap_or_default()
}

fn answer(harness: &mut Harness<'_, Window>, answer: &str) {
    tab_to(harness, Role::Button, answer);
    press(harness, Modifiers::NONE, Key::Enter);
}

fn open_file(harness: &mut Harness<'_, Window>, side: &str, path: &str) {
    let commands = commands();
    let command = commands.iter().find(|c| c.id == "text-compare.open_file").unwrap();
    let shell = &mut harness.state_mut().shell;
    let args = [("side", Value::Choice(side.to_owned())), ("path", Value::Path(path.to_owned()))];
    command.bind(shell, &args, true).unwrap().run(shell).unwrap();
    harness.run();
}

#[test]
fn a_path_typed_over_an_edited_pane_asks_and_cancel_keeps_the_edits_and_the_old_file() {
    for choice in [CANCEL, DONT_SAVE, SAVE] {
        let folder = tempfile::tempdir().unwrap();
        let (mut harness, left_path) = harness(folder.path(), b"one\n", b"one\n");
        let old = left_path.display().to_string();
        let other = other(folder.path());
        edit_left(&mut harness, "X");
        tab_to(&mut harness, Role::TextInput, LEFT);
        press(&mut harness, Modifiers::COMMAND, Key::A);
        harness.get_by(|node| node.is_focused()).type_text(&other);
        harness.run();
        press(&mut harness, Modifiers::NONE, Key::Enter);
        assert!(prompt(&harness).is_some(), "{choice}: the typed path asks before it loses the edit");
        assert_eq!(left(&harness), Some((old.clone(), "Xone".to_owned(), true)), "{choice}: nothing loads while it asks");

        answer(&mut harness, choice);
        harness.run();
        assert_eq!(prompt(&harness), None, "{choice}");
        let focused = harness.query_by(|node| node.is_focused()).map(|node| (node.accesskit_node().role(), node.accesskit_node().label()));
        assert_eq!(focused, Some((Role::TextInput, Some(LEFT.to_owned()))), "{choice}: focus goes back to the path edit");
        let (loaded, saved): (_, &[u8]) = match choice {
            CANCEL => (Some((old.clone(), "Xone".to_owned(), true)), b"one\n"),
            DONT_SAVE => (Some((other.clone(), "other".to_owned(), false)), b"one\n"),
            _ => (Some((other.clone(), "other".to_owned(), false)), b"Xone\n"),
        };
        assert_eq!(left(&harness), loaded, "{choice}");
        assert_eq!(std::fs::read(&left_path).unwrap(), saved, "{choice}");
        let shown = if choice == CANCEL { &old } else { &other };
        assert_eq!(&path_edit_text(&harness, LEFT), shown, "{choice}: the path edit shows the pane's file");
    }
}

#[test]
fn open_file_over_an_edited_pane_asks_whether_given_a_path_or_browsing_but_not_over_a_clean_one() {
    let folder = tempfile::tempdir().unwrap();
    let (mut harness, left_path) = harness(folder.path(), b"one\n", b"one\n");
    let old = left_path.display().to_string();
    let other = other(folder.path());
    edit_left(&mut harness, "X");
    let kept = Some((old, "Xone".to_owned(), true));

    open_file(&mut harness, LEFT, &other);
    assert!(prompt(&harness).is_some(), "Open File with a path asks");
    answer(&mut harness, CANCEL);
    assert_eq!(left(&harness), kept, "Cancel keeps the edits and the old file");

    // Browsing: the picked file comes back through the path edit.
    let picked = PathBuf::from(&other);
    harness.state_mut().edits.as_mut().unwrap().set_override(Some(Box::new(move |_: &Request| Answer::Paths(vec![picked.clone()]))));
    tab_to(&mut harness, Role::MultilineTextInput, LEFT);
    assert!(harness.state_mut().shell.run("text-compare.open_file"));
    for _ in 0..3 {
        harness.run();
    }
    assert!(prompt(&harness).is_some(), "a browsed file asks");
    answer(&mut harness, CANCEL);
    assert_eq!(left(&harness), kept, "Cancel keeps the edits and the old file");

    // The right pane has no edits of its own, so it loads without asking.
    open_file(&mut harness, RIGHT, &other);
    harness.run();
    assert_eq!(prompt(&harness), None, "a clean pane loads without asking");
    assert_eq!(view(&harness).pane(1).map(|pane| pane.path.as_str()), Some(other.as_str()));
    assert_eq!(left(&harness), kept, "the edited pane is untouched");
}
