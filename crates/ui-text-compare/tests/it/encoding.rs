//! Each side's encoding override on the Comparison Settings dialog's Format tab, driven by keyboard
//! through the real shell and read back through the accessibility tree.

use egui::accesskit::{Live, Role};
use egui::{Event, Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_ui::{OK, Shell, View as _};
use otd_ui_text_compare::encoding::{ENCODING_OVERRIDES, OVERRIDE_WAITS, session_settings};
use otd_ui_text_compare::{LEFT, SESSION_TYPE, TextCompareView, commands};

fn press(harness: &mut Harness<'_, Shell>, key: Key) {
    harness.key_press_modifiers(Modifiers::NONE, key);
    harness.run();
}

fn tab_to(harness: &mut Harness<'_, Shell>, role: Role, name: &str) {
    for _ in 0..60 {
        let focused = harness.query_by(|node| node.is_focused()).map(|node| {
            let node = node.accesskit_node();
            (node.role(), node.label().or_else(|| node.value()).unwrap_or_default())
        });
        if focused.is_some_and(|(r, label)| r == role && label == name) {
            return;
        }
        press(harness, Key::Tab);
    }
    panic!("Tab never reached {name:?}");
}

/// The shell with one Text Compare tab comparing a left file in Windows-1252 whose bytes are also
/// valid UTF-8 ("CafÃ©", read as "Café") with a plain right one.
fn harness(folder: &tempfile::TempDir) -> Harness<'static, Shell> {
    let specs = [("left.txt", &b"Caf\xC3\xA9\n"[..]), ("right.txt", b"other\n")].map(|(name, bytes)| {
        let path = folder.path().join(name);
        std::fs::write(&path, bytes).unwrap();
        path.display().to_string()
    });
    let mut view = TextCompareView::new();
    view.session().expect("a session").specs = specs.to_vec();
    let mut shell = Shell::new(vec![SESSION_TYPE]).with_commands(commands()).with_settings(session_settings()).unwrap();
    shell.open(Box::new(view));
    let mut harness = Harness::builder().build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), shell);
    harness.run();
    harness
}

/// Chooses Windows-1252 for the left side on the Session Settings dialog's Format tab, by keyboard.
fn choose_windows_1252_on_the_left(harness: &mut Harness<'_, Shell>) {
    harness.state_mut().session_settings().unwrap();
    harness.run();
    tab_to(harness, Role::Tab, "Format");
    press(harness, Key::Enter);
    tab_to(harness, Role::ComboBox, ENCODING_OVERRIDES[0].1);
    press(harness, Key::Space);
    tab_to(harness, Role::Button, "Windows-1252");
    press(harness, Key::Enter);
    tab_to(harness, Role::Button, OK);
    press(harness, Key::Enter);
}

#[test]
fn a_windows_1252_file_detected_as_utf8_shows_its_own_characters_once_windows_1252_is_chosen() {
    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder);
    let shown = |harness: &Harness<'_, Shell>, label: &str| harness.query_all_by_label(label).count();
    assert_eq!(shown(&harness, "CafÃ©"), 0, "None: detection reads the bytes as UTF-8");

    choose_windows_1252_on_the_left(&mut harness);

    assert_eq!(shown(&harness, "CafÃ©"), 1, "the left pane is read again in Windows-1252");
    assert_eq!(shown(&harness, "Encoding: Windows-1252"), 1, "the left header names it, and the right keeps its own");
    assert!(harness.query_by_label(OVERRIDE_WAITS).is_none(), "nothing waits");
}

#[test]
fn an_override_chosen_over_unsaved_edits_waits_and_the_pane_announces_it() {
    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder);
    tab_to(&mut harness, Role::MultilineTextInput, LEFT);
    harness.event(Event::Text("X".to_owned()));
    harness.run();

    choose_windows_1252_on_the_left(&mut harness);

    let notices = harness.query_all_by_label(OVERRIDE_WAITS).collect::<Vec<_>>();
    assert_eq!(notices.len(), 1, "only the edited side waits");
    assert!(notices[0].accesskit_node().live() != Live::Off, "announced, since it appears when the dialog closes");
}
