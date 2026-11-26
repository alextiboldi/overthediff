//! Each side's encoding override on the Comparison Settings dialog's Format tab, driven by keyboard
//! through the real shell and read back through the accessibility tree.

use egui::accesskit::Role;
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_ui::{OK, Shell, View as _};
use otd_ui_text_compare::encoding::{ENCODING_OVERRIDES, session_settings};
use otd_ui_text_compare::{SESSION_TYPE, TextCompareView, commands};

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

#[test]
fn a_windows_1252_file_detected_as_utf8_shows_its_own_characters_once_windows_1252_is_chosen() {
    let folder = tempfile::tempdir().unwrap();
    // "CafÃ©" in Windows-1252, whose bytes are also valid UTF-8, for "Café".
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
    let shown = |harness: &Harness<'_, Shell>, label: &str| harness.query_all_by_label(label).count();
    assert_eq!(shown(&harness, "CafÃ©"), 0, "None: detection reads the bytes as UTF-8");

    harness.state_mut().session_settings().unwrap();
    harness.run();
    tab_to(&mut harness, Role::Tab, "Format");
    press(&mut harness, Key::Enter);
    tab_to(&mut harness, Role::ComboBox, ENCODING_OVERRIDES[0].1);
    press(&mut harness, Key::Space);
    tab_to(&mut harness, Role::Button, "Windows-1252");
    press(&mut harness, Key::Enter);
    tab_to(&mut harness, Role::Button, OK);
    press(&mut harness, Key::Enter);

    assert_eq!(shown(&harness, "CafÃ©"), 1, "the left pane is read again in Windows-1252");
    assert_eq!(shown(&harness, "Encoding: Windows-1252"), 1, "the left header names it, and the right keeps its own");
}
