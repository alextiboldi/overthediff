//! The File Formats manager, driven by keyboard and read back through the accessibility tree.

use egui::accesskit::{Live, Role};
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_format::{Case, Format, Kind};
use otd_ui::formats::FILE_FORMATS;
use otd_ui::{HOME, Shell};

const ANY: &str = "Any file";
const NOTES: &str = "Notes";

/// A shell whose formats overlap on `a.txt`: `*.*` above `*.txt`, so the first wins.
fn shell() -> Shell {
    let mut shell = Shell::new(Vec::new());
    let formats = shell.formats_mut();
    formats.register_kind(Kind::Text);
    for (name, masks) in [(ANY, "*.*"), (NOTES, "*.txt")] {
        formats.register(Format::new(name, Kind::Text, masks).unwrap()).unwrap();
    }
    shell
}

/// A harness with the manager opened by its command, as the menus open it.
fn manager() -> Harness<'static, Shell> {
    let mut harness = Harness::builder().build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), shell());
    harness.run();
    assert!(harness.state_mut().run("otd.file_formats"));
    harness.run();
    harness
}

fn press(harness: &mut Harness<'_, Shell>, key: Key) {
    harness.key_press_modifiers(Modifiers::NONE, key);
    harness.run();
}

fn focused(harness: &Harness<'_, Shell>) -> Option<(Role, String)> {
    let node = harness.query_by(|node| node.is_focused())?;
    let node = node.accesskit_node();
    Some((node.role(), node.label().or_else(|| node.value()).unwrap_or_default()))
}

fn resolved(harness: &Harness<'_, Shell>) -> Option<String> {
    harness.state().formats().resolve("a.txt", Case::Sensitive).map(|f| f.name.clone())
}

#[test]
fn moving_a_format_above_an_overlapping_one_makes_it_win_and_keeps_focus_on_it() {
    let mut harness = manager();
    assert_eq!(resolved(&harness).as_deref(), Some(ANY));
    assert_eq!(focused(&harness), Some((Role::CheckBox, ANY.to_owned())), "the top format has focus");

    // Tab selects the second row; Move Up comes after the list.
    press(&mut harness, Key::Tab);
    assert_eq!(focused(&harness), Some((Role::CheckBox, NOTES.to_owned())));
    press(&mut harness, Key::Tab);
    assert_eq!(focused(&harness).map(|(role, _)| role), Some(Role::Button));
    press(&mut harness, Key::Enter);
    harness.run();

    assert_eq!(resolved(&harness).as_deref(), Some(NOTES), "resolution follows the new order at once");
    let rows: Vec<_> = harness.query_all_by_role(Role::Row).filter_map(|row| row.accesskit_node().label()).collect();
    assert_eq!(rows, [NOTES, ANY], "the list shows the new order");
    // At the top its Move Up is disabled, so focus goes to the moved row rather than being lost.
    assert_eq!(focused(&harness), Some((Role::CheckBox, NOTES.to_owned())));
    let announced: Vec<_> = harness.query_all_by(|node| node.live() == Live::Polite).filter_map(|node| node.accesskit_node().value()).collect();
    assert!(announced.iter().any(|text| text.contains(NOTES)), "the move is announced: {announced:?}");
}

#[test]
fn esc_closes_the_manager_and_gives_focus_back_to_the_window() {
    let mut harness = manager();
    let open = |harness: &Harness<'_, Shell>| {
        harness.query_by(|node| node.role() == Role::Dialog && node.label().as_deref() == Some(FILE_FORMATS)).is_some()
    };
    assert!(open(&harness));

    press(&mut harness, Key::Escape);
    harness.run();
    assert!(!open(&harness));
    assert_eq!(focused(&harness), Some((Role::Tab, HOME.to_owned())));
}
