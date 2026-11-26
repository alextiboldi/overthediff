//! Bookmarks 0–9 (E07-S6-T2): set from the Search menu's commands on the caret's line, marked in
//! the gutter, moved with their lines by typing, and jumped back to; driven by keyboard through the
//! real shell and read back through the accessibility tree.

use egui::accesskit::{Live, Role};
use egui::{Key, Modifiers, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_ui::{Shell, View as _};
use otd_ui_text_compare::{LEFT, SESSION_TYPE, TextCompareView, commands};

/// The shell with one Text Compare tab comparing `text` with itself.
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

fn run(harness: &mut Harness<'_, Shell>, id: &str) -> bool {
    let ran = harness.state_mut().run(id);
    harness.run();
    ran
}

fn caret_line(harness: &Harness<'_, Shell>) -> usize {
    harness.state().view::<TextCompareView>().unwrap().pane(0).unwrap().selection.head.line
}

fn focused_editor(harness: &Harness<'_, Shell>) -> Option<String> {
    let node = harness.query_by(|node| node.is_focused())?;
    (node.accesskit_node().role() == Role::MultilineTextInput).then(|| node.accesskit_node().label().unwrap_or_default())
}

/// The vertical centres of the label nodes named `label`: lines and gutter marks.
fn rows(harness: &Harness<'_, Shell>, label: &str) -> Vec<f32> {
    harness.query_all_by_label(label).filter(|node| node.accesskit_node().role() == Role::Label).map(|node| node.rect().center().y).collect()
}

#[test]
fn a_bookmark_moves_with_its_line_when_lines_are_inserted_above_it_and_go_to_lands_there() {
    let folder = tempfile::tempdir().unwrap();
    let text: String = (1..=60).map(|n| format!("line {n}\n")).collect();
    let mut harness = harness(&folder, &text);
    for _ in 0..40 {
        if focused_editor(&harness).as_deref() == Some(LEFT) {
            break;
        }
        key(&mut harness, Modifiers::NONE, Key::Tab);
    }
    assert_eq!(focused_editor(&harness).as_deref(), Some(LEFT));
    for _ in 1..50 {
        key(&mut harness, Modifiers::NONE, Key::ArrowDown);
    }
    assert!(!run(&mut harness, "text-compare.go_to_bookmark_2"), "Go To Bookmark 2 is disabled while it isn't set");

    assert!(run(&mut harness, "text-compare.toggle_bookmark_2"));
    let said = harness.get_by_label("Bookmark 2 set on line 50");
    assert_eq!(said.accesskit_node().live(), Live::Polite, "the toggle is announced");
    let mark = rows(&harness, "Bookmark 2");
    assert_eq!(mark.len(), 1, "one gutter mark, in the pane it was set in");
    assert!(rows(&harness, "line 50").contains(&mark[0]), "beside line 50");

    // Three lines inserted above it, by keyboard at the top of the file.
    key(&mut harness, Modifiers::COMMAND, Key::Home);
    for _ in 0..3 {
        key(&mut harness, Modifiers::NONE, Key::Enter);
    }
    assert!(caret_line(&harness) < 10, "the caret is far from the bookmark");

    assert!(run(&mut harness, "text-compare.go_to_bookmark_2"));
    assert_eq!(caret_line(&harness) + 1, 53, "Go To Bookmark 2 lands on line 53, where its line now is");
    assert_eq!(focused_editor(&harness).as_deref(), Some(LEFT), "with focus in the pane");
    let mark = rows(&harness, "Bookmark 2");
    assert!(mark.len() == 1 && rows(&harness, "line 50").contains(&mark[0]), "its mark is in view, still beside the text of line 50");

    // Toggling it on its own line clears it.
    assert!(run(&mut harness, "text-compare.toggle_bookmark_2"));
    assert!(rows(&harness, "Bookmark 2").is_empty(), "its mark is gone");
    assert!(harness.query_by_label("Bookmark 2 cleared").is_some(), "and that is announced");
    assert!(!run(&mut harness, "text-compare.go_to_bookmark_2"));
}

#[test]
fn clear_bookmarks_removes_every_bookmark_in_both_panes() {
    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder, "one\ntwo\n");
    assert!(!run(&mut harness, "text-compare.clear_bookmarks"), "nothing to clear yet");
    assert!(run(&mut harness, "text-compare.toggle_bookmark_0"));
    assert!(run(&mut harness, "text-compare.toggle_bookmark_9"));
    // The right pane, chosen by Tab from the left editor.
    key(&mut harness, Modifiers::NONE, Key::Tab);
    for _ in 0..40 {
        if focused_editor(&harness).is_some_and(|name| name != LEFT) {
            break;
        }
        key(&mut harness, Modifiers::NONE, Key::Tab);
    }
    assert!(run(&mut harness, "text-compare.toggle_bookmark_5"));
    for number in [0, 5, 9] {
        assert_eq!(harness.query_all_by_label(&format!("Bookmark {number}")).count(), 1, "bookmark {number} is marked");
    }

    assert!(run(&mut harness, "text-compare.clear_bookmarks"));
    for number in [0, 5, 9] {
        assert!(harness.query_by_label(&format!("Bookmark {number}")).is_none(), "bookmark {number} is gone");
    }
    assert!(harness.query_by_label("Bookmarks cleared").is_some(), "the clear is announced");
    assert!(!run(&mut harness, "text-compare.go_to_bookmark_0"));
}
