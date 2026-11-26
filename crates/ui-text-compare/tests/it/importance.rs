//! The Session Settings dialog's Importance tab, driven by keyboard through the real shell: a
//! changed checkbox compares the open view again at once, and edits keep comparing by it.

use egui::accesskit::Role;
use egui::{Event, Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_ui::{OK, Shell, View as _};
use otd_ui_text_compare::importance::{ELEMENTS, IMPORTANCE_TAB, LINE_ENDINGS, session_settings};
use otd_ui_text_compare::session::{IMPORTANT, SAME, UNIMPORTANT_ONLY};
use otd_ui_text_compare::{LEFT, SESSION_TYPE, TextCompareView, commands};

fn press(harness: &mut Harness<'_, Shell>, key: Key) {
    harness.key_press_modifiers(Modifiers::NONE, key);
    harness.run();
}

fn tab_to(harness: &mut Harness<'_, Shell>, role: Role, name: &str) {
    for _ in 0..60 {
        let focused = harness.query_by(|node| node.is_focused()).map(|node| {
            let node = node.accesskit_node();
            (node.role(), node.label().unwrap_or_default())
        });
        if focused.is_some_and(|(r, label)| r == role && label == name) {
            return;
        }
        press(harness, Key::Tab);
    }
    panic!("Tab never reached {name:?}");
}

/// The shell with one Text Compare tab comparing the files `left` and `right`, and their folder.
fn harness(left: &str, right: &str) -> (Harness<'static, Shell>, tempfile::TempDir) {
    let folder = tempfile::tempdir().unwrap();
    let specs = [("left.txt", left), ("right.txt", right)].map(|(name, text)| {
        let path = folder.path().join(name);
        std::fs::write(&path, text).unwrap();
        path.display().to_string()
    });
    let mut view = TextCompareView::new();
    view.session().expect("a session").specs = specs.to_vec();
    let mut shell = Shell::new(vec![SESSION_TYPE]).with_commands(commands()).with_settings(session_settings()).unwrap();
    shell.open(Box::new(view));
    let mut harness = Harness::builder().build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), shell);
    harness.run();
    (harness, folder)
}

fn result(harness: &Harness<'_, Shell>) -> Option<String> {
    harness.state().view::<TextCompareView>().expect("a Text Compare tab").status().result.map(|field| field.value)
}

/// Ticks or unticks the checkbox `label` on the Importance tab by keyboard, then presses OK.
fn toggle(harness: &mut Harness<'_, Shell>, label: &str) {
    harness.state_mut().session_settings().unwrap();
    harness.run();
    tab_to(harness, Role::Tab, IMPORTANCE_TAB);
    press(harness, Key::Enter);
    tab_to(harness, Role::CheckBox, label);
    press(harness, Key::Space);
    tab_to(harness, Role::Button, OK);
    press(harness, Key::Enter);
}

#[test]
fn unticking_leading_whitespace_recompares_the_view_and_its_edits_at_once() {
    let (mut harness, _folder) = harness("  a\n", "a\n");
    assert_eq!(result(&harness), Some(IMPORTANT.to_owned()), "indentation matters by default");

    toggle(&mut harness, ELEMENTS[0].1);
    assert_eq!(result(&harness), Some(UNIMPORTANT_ONLY.to_owned()), "and not once Leading whitespace is unticked");

    // More indentation, typed: the edit is compared by the same rules.
    tab_to(&mut harness, Role::MultilineTextInput, LEFT);
    harness.event(Event::Text(" ".to_owned()));
    harness.run();
    assert_eq!(harness.state().view::<TextCompareView>().unwrap().pane(0).unwrap().lines[0].text, "   a", "the space was typed");
    assert_eq!(result(&harness), Some(UNIMPORTANT_ONLY.to_owned()));
}

#[test]
fn ticking_compare_line_endings_makes_a_crlf_file_differ_from_its_lf_copy() {
    let (mut harness, _folder) = harness("a\r\nb\r\n", "a\nb\n");
    assert_eq!(result(&harness), Some(SAME.to_owned()), "line endings are ignored by default");

    toggle(&mut harness, LINE_ENDINGS.1);
    assert_eq!(result(&harness), Some(IMPORTANT.to_owned()));
}
