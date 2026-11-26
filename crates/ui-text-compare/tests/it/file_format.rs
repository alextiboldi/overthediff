//! Each side's file-format picker on the Comparison Settings dialog's Format tab, driven by keyboard
//! through the real shell and read back through the accessibility tree.

use egui::accesskit::Role;
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_format::{Format, Kind};
use otd_ui::{OK, Shell, View as _};
use otd_ui_text_compare::{FORMAT_PICKERS, FORMAT_TAB, SESSION_TYPE, TextCompareView, commands, formats, session_settings};

fn press(harness: &mut Harness<'_, Shell>, key: Key) {
    harness.key_press_modifiers(Modifiers::NONE, key);
    harness.run();
}

fn tab_to(harness: &mut Harness<'_, Shell>, role: Role, name: &str) {
    for _ in 0..40 {
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
fn an_empty_mask_format_is_offered_and_once_chosen_the_left_header_names_it() {
    let folder = tempfile::tempdir().unwrap();
    let specs = ["left.txt", "right.txt"].map(|name| {
        let path = folder.path().join(name);
        std::fs::write(&path, "text\n").unwrap();
        path.display().to_string()
    });
    let mut formats = formats();
    formats.register(Format::new("Notes", Kind::Text, "").unwrap()).unwrap();
    let settings = session_settings(&formats);
    let mut view = TextCompareView::with_formats(formats);
    view.session().expect("a session").specs = specs.to_vec();
    let mut shell = Shell::new(vec![SESSION_TYPE]).with_commands(commands()).with_settings(settings).unwrap();
    shell.open(Box::new(view));
    let mut harness = Harness::builder().build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), shell);
    harness.run();
    let named = |harness: &Harness<'_, Shell>| harness.query_all_by_label("Format: Notes").count();
    assert_eq!(named(&harness), 0, "Detected: the empty mask matched neither file");

    harness.state_mut().session_settings().unwrap();
    harness.run();
    tab_to(&mut harness, Role::Tab, FORMAT_TAB);
    press(&mut harness, Key::Enter);
    tab_to(&mut harness, Role::ComboBox, FORMAT_PICKERS[0].1);
    press(&mut harness, Key::Space);
    tab_to(&mut harness, Role::Button, "Notes");
    press(&mut harness, Key::Enter);
    tab_to(&mut harness, Role::Button, OK);
    press(&mut harness, Key::Enter);

    assert_eq!(named(&harness), 1, "one header names the chosen format, and the other keeps its own");
    let x = |label: &str| harness.get_by_label(label).rect().center().x;
    assert!(x("Format: Notes") < x("Format: Plain text"), "the header naming it is the left one");
}
