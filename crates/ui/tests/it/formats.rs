//! The File Formats manager, driven by keyboard and read back through the accessibility tree.

use egui::accesskit::{Live, Role};
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_format::{Case, Format, Kind};
use otd_ui::command::{CommandError, Outcome, Value};
use otd_ui::formats::{FILE_FORMATS, MOVE_DOWN, MOVE_UP};
use otd_ui::{CLOSE, HOME, Shell};

const ANY: &str = "Any file";
const NOTES: &str = "Notes";
const LOGS: &str = "Logs";

/// A shell whose formats overlap on `a.txt`: `*.*` above `*.txt`, so the first wins. `*.log`, last,
/// matches neither.
fn shell() -> Shell {
    let mut shell = Shell::new(Vec::new());
    let formats = shell.formats_mut();
    formats.register_kind(Kind::Text);
    for (name, masks) in [(ANY, "*.*"), (NOTES, "*.txt"), (LOGS, "*.log")] {
        formats.register(Format::new(name, Kind::Text, masks).unwrap()).unwrap();
    }
    shell
}

/// A harness with the manager opened by its command, as the menus open it.
fn manager() -> Harness<'static, Shell> {
    manager_of(shell())
}

fn manager_of(shell: Shell) -> Harness<'static, Shell> {
    let mut harness = Harness::builder().build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), shell);
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
    resolve(harness.state())
}

fn resolve(shell: &Shell) -> Option<String> {
    shell.formats().resolve("a.txt", Case::Sensitive).map(|f| f.name.clone())
}

fn rows(harness: &Harness<'_, Shell>) -> Vec<String> {
    harness.query_all_by_role(Role::Row).filter_map(|row| row.accesskit_node().label()).collect()
}

fn button(label: &str) -> Option<(Role, String)> {
    Some((Role::Button, label.to_owned()))
}

#[test]
fn moving_a_format_above_an_overlapping_one_makes_it_win_and_keeps_focus_on_it() {
    let mut harness = manager();
    assert_eq!(resolved(&harness).as_deref(), Some(ANY));
    assert_eq!(focused(&harness), Some((Role::CheckBox, ANY.to_owned())), "the top format has focus");

    // Down selects the second row; Tab leaves the list for Move Up.
    press(&mut harness, Key::ArrowDown);
    assert_eq!(focused(&harness), Some((Role::CheckBox, NOTES.to_owned())));
    press(&mut harness, Key::Tab);
    assert_eq!(focused(&harness), button(MOVE_UP));
    press(&mut harness, Key::Enter);
    harness.run();

    assert_eq!(resolved(&harness).as_deref(), Some(NOTES), "resolution follows the new order at once");
    assert_eq!(rows(&harness), [NOTES, ANY, LOGS], "the list shows the new order");
    // At the top its Move Up is disabled, so focus goes to the moved row rather than being lost.
    assert_eq!(focused(&harness), Some((Role::CheckBox, NOTES.to_owned())));
    let announced: Vec<_> = harness.query_all_by(|node| node.live() == Live::Polite).filter_map(|node| node.accesskit_node().value()).collect();
    assert!(announced.iter().any(|text| text.contains(NOTES)), "the move is announced: {announced:?}");
}

#[test]
fn tab_reaches_the_buttons_without_changing_the_selection() {
    let mut harness = manager();
    // The list is one Tab stop: from the top row, Tab goes past the other rows to Move Down (Move
    // Up is disabled at the top), which moves the top row.
    press(&mut harness, Key::Tab);
    assert_eq!(focused(&harness), button(MOVE_DOWN));
    press(&mut harness, Key::Enter);
    assert_eq!(rows(&harness), [NOTES, ANY, LOGS]);

    // Shift+Tab from the buttons comes back to the selected row, not the last one.
    harness.key_press_modifiers(Modifiers::SHIFT, Key::Tab);
    harness.run();
    assert_eq!(focused(&harness), button(MOVE_UP));
    harness.key_press_modifiers(Modifiers::SHIFT, Key::Tab);
    harness.run();
    assert_eq!(focused(&harness), Some((Role::CheckBox, ANY.to_owned())));
}

#[test]
fn the_list_is_one_stop_that_arrows_move_within() {
    let mut harness = manager();
    let row = |name: &str| Some((Role::CheckBox, name.to_owned()));
    for _ in 0..3 {
        press(&mut harness, Key::ArrowDown);
    }
    assert_eq!(focused(&harness), row(LOGS), "Down stops at the bottom row");
    press(&mut harness, Key::ArrowUp);
    assert_eq!(focused(&harness), row(NOTES));

    // Shift+Tab leaves the list rather than walking up it; Tab wraps back in at the selected row.
    harness.key_press_modifiers(Modifiers::SHIFT, Key::Tab);
    harness.run();
    assert_eq!(focused(&harness), button(CLOSE));
    press(&mut harness, Key::Tab);
    assert_eq!(focused(&harness), row(NOTES));
}

#[test]
fn moving_a_format_to_the_bottom_gives_focus_to_its_row() {
    let mut harness = manager();
    press(&mut harness, Key::ArrowDown);
    press(&mut harness, Key::Tab);
    press(&mut harness, Key::Tab);
    assert_eq!(focused(&harness), button(MOVE_DOWN));
    press(&mut harness, Key::Enter);

    assert_eq!(rows(&harness), [ANY, LOGS, NOTES]);
    // At the bottom its Move Down is disabled, so focus goes to the moved row.
    assert_eq!(focused(&harness), Some((Role::CheckBox, NOTES.to_owned())));
}

#[test]
fn space_on_a_row_disables_its_format_so_it_stops_matching() {
    let mut harness = manager();
    assert_eq!(resolved(&harness).as_deref(), Some(ANY));
    press(&mut harness, Key::Space);
    assert_eq!(resolved(&harness).as_deref(), Some(NOTES), "the disabled format above no longer wins");
    let any = harness.query_by(|node| node.role() == Role::CheckBox && node.label().as_deref() == Some(ANY)).unwrap();
    assert_eq!(any.accesskit_node().toggled(), Some(egui::accesskit::Toggled::False));
}

#[test]
fn an_empty_list_gives_focus_to_close() {
    let harness = manager_of(Shell::new(Vec::new()));
    assert_eq!(focused(&harness), button(CLOSE));
}

#[test]
fn the_commands_move_and_enable_a_format_by_name() {
    let mut shell = shell();
    fn run(shell: &mut Shell, id: &str, args: &[(&str, Value)]) -> Outcome {
        let command = shell.commands().iter().find(|c| c.id == id).unwrap();
        command.bind(shell, args, false)?.run(shell)
    }
    let name = |name: &str| ("format", Value::Text(name.to_owned()));

    assert!(run(&mut shell, "otd.move_format_up", &[name(NOTES)]).is_ok());
    assert_eq!(resolve(&shell).as_deref(), Some(NOTES));
    assert!(run(&mut shell, "otd.move_format_down", &[name(NOTES)]).is_ok());
    assert_eq!(resolve(&shell).as_deref(), Some(ANY));
    assert!(run(&mut shell, "otd.enable_format", &[name(ANY), ("enabled", Value::Bool(false))]).is_ok());
    assert_eq!(resolve(&shell).as_deref(), Some(NOTES));

    let missing = "No such format";
    for (id, args) in [("otd.move_format_up", vec![name(missing)]), ("otd.enable_format", vec![name(missing), ("enabled", Value::Bool(true))])] {
        match run(&mut shell, id, &args) {
            Err(CommandError::Failed(message)) => assert!(message.contains(missing), "{id}: {message}"),
            other => panic!("{id} should fail on an unknown format: {other:?}"),
        }
    }
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
