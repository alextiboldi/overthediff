//! The session commands on the Session menu and the toolbar (E01-S1-T14): New Session ▸ <type>,
//! Open Session, Clear Session and Session Settings, each run by keyboard from its menu path in
//! the in-window menu bar and read back through the accessibility tree.

use egui::accesskit::Role;
use egui::os::OperatingSystem;
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_ui::{CLEAR_SESSION, CLOSE_CURRENT_TAB, DONT_SAVE, HOME, NEW_SESSION, OPEN_SESSION, SESSION_SETTINGS, Session, SessionType, Shell, View};

const ALPHA: &str = "Alpha Compare";
const BETA: &str = "Beta Compare";
/// What the test view shows of its session.
const SHOWS_DESCRIPTION: &str = "About";

/// A test session view: its view type is its session type's id, as a real view's is.
struct SessionView {
    title: &'static str,
    view_type: &'static str,
    /// `None` for a view of a session type that shows no session.
    session: Option<Session>,
    unsaved: bool,
}

impl View for SessionView {
    fn title(&self) -> String {
        self.title.to_owned()
    }

    fn view_type(&self) -> &'static str {
        self.view_type
    }

    fn session(&mut self) -> Option<&mut Session> {
        self.session.as_mut()
    }

    fn unsaved(&self) -> bool {
        self.unsaved
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let description = self.session.as_ref().map_or("", |s| s.description.as_str());
        ui.label(format!("{SHOWS_DESCRIPTION}: {description}"));
    }
}

fn view(title: &'static str, view_type: &'static str) -> Box<dyn View> {
    Box::new(SessionView { title, view_type, session: Some(Session::new(view_type)), unsaved: false })
}

fn session_types() -> Vec<SessionType> {
    vec![
        SessionType { id: "alpha", name: ALPHA, kinds: &[], sides: &["Left", "Right"], new: || view(ALPHA, "alpha") },
        SessionType { id: "beta", name: BETA, kinds: &[], sides: &["Left", "Right"], new: || view(BETA, "beta") },
    ]
}

/// A shell on Windows, whose in-window menu bar has mnemonics, on Home.
fn harness() -> Harness<'static, Shell> {
    let mut harness =
        Harness::builder().with_os(OperatingSystem::Windows).build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), Shell::new(session_types()));
    harness.run();
    harness
}

fn press(harness: &mut Harness<'_, Shell>, modifiers: Modifiers, key: Key) {
    harness.key_press_modifiers(modifiers, key);
    harness.run();
}

fn focused(harness: &Harness<'_, Shell>) -> Option<(Role, String)> {
    let node = harness.query_by(|node| node.is_focused())?;
    let node = node.accesskit_node();
    Some((node.role(), node.label().unwrap_or_default()))
}

/// The Session menu's rows as the shell lays it out for the in-window bar.
fn session_rows(harness: &Harness<'_, Shell>) -> Vec<craft_menu::Node> {
    let bar = harness.state().menu_layout(OperatingSystem::Windows).bar;
    bar.menus.into_iter().find(|menu| menu.title == "Session").map(|menu| menu.children).unwrap_or_default()
}

/// Runs the item at `path` in the Session menu by keyboard: Alt+S opens the menu on its first row,
/// Down moves to each entry, Right opens a submenu on its first row and Enter runs the item. Each
/// entry is a menu item of that name in the accessibility tree.
// ponytail: counts Down presses from the layout instead of following egui's focus, which trails the
// bar's highlight by one row (alextiboldi/overthediff#72).
fn run_from_session_menu(harness: &mut Harness<'_, Shell>, path: &[&str]) {
    press(harness, Modifiers::ALT, Key::S);
    let mut rows = session_rows(harness);
    for (depth, name) in path.iter().enumerate() {
        // The arrows skip separators and disabled items.
        let reachable: Vec<&craft_menu::Node> = rows
            .iter()
            .filter(|node| match node {
                craft_menu::Node::Item(item) => item.enabled,
                craft_menu::Node::Submenu { .. } => true,
                _ => false,
            })
            .collect();
        let label = |node: &craft_menu::Node| match node {
            craft_menu::Node::Item(item) => item.label.clone(),
            craft_menu::Node::Submenu { label, .. } => label.clone(),
            _ => String::new(),
        };
        let index = reachable.iter().position(|node| label(node) == *name).unwrap_or_else(|| panic!("{name:?} is in the menu"));
        for _ in 0..index {
            press(harness, Modifiers::NONE, Key::ArrowDown);
        }
        assert!(harness.query_all_by_label(name).any(|node| node.accesskit_node().role() == Role::MenuItem), "{name:?} shows as a menu item");
        let next = match reachable[index] {
            craft_menu::Node::Submenu { children, .. } => children.clone(),
            _ => Vec::new(),
        };
        press(harness, Modifiers::NONE, if depth + 1 < path.len() { Key::ArrowRight } else { Key::Enter });
        rows = next;
    }
}

fn tab_names(harness: &Harness<'_, Shell>) -> Vec<String> {
    harness.query_all_by_role(Role::Tab).map(|tab| tab.accesskit_node().label().unwrap_or_default()).collect()
}

fn selected(harness: &Harness<'_, Shell>) -> Option<String> {
    harness.query_by(|node| node.role() == Role::Tab && node.is_selected() == Some(true)).and_then(|tab| tab.accesskit_node().label())
}

fn settings_open(harness: &Harness<'_, Shell>) -> bool {
    harness.query_by(|node| node.role() == Role::Dialog && node.label().as_deref() == Some(SESSION_SETTINGS)).is_some()
}

fn prompt_open(harness: &Harness<'_, Shell>) -> bool {
    harness.query_by_role(Role::AlertDialog).is_some()
}

/// The labels in the Session menu, at any depth, as the shell lays it out on Windows.
fn session_menu(harness: &Harness<'_, Shell>) -> Vec<String> {
    session_menu_on(harness, OperatingSystem::Windows)
}

/// The labels in the Session menu, at any depth, as the shell lays it out on `os`.
fn session_menu_on(harness: &Harness<'_, Shell>, os: OperatingSystem) -> Vec<String> {
    fn walk(nodes: &[craft_menu::Node], out: &mut Vec<String>) {
        for node in nodes {
            match node {
                craft_menu::Node::Item(item) => out.push(item.label.clone()),
                craft_menu::Node::Submenu { label, children, .. } => {
                    out.push(label.clone());
                    walk(children, out);
                }
                _ => {}
            }
        }
    }
    let bar = harness.state().menu_layout(os).bar;
    let mut out = Vec::new();
    for menu in bar.menus.iter().filter(|menu| menu.title == "Session") {
        walk(&menu.children, &mut out);
    }
    out
}

fn description(harness: &mut Harness<'_, Shell>) -> Option<String> {
    Some(harness.state_mut().view_mut::<SessionView>()?.session.as_ref()?.description.clone())
}

fn set_description(harness: &mut Harness<'_, Shell>, text: &str) {
    if let Some(session) = harness.state_mut().view_mut::<SessionView>().and_then(|view| view.session.as_mut()) {
        session.description = text.to_owned();
    }
    harness.run();
}

#[test]
fn new_session_has_an_entry_per_registered_type_and_each_opens_an_empty_session_of_it_in_a_new_tab() {
    let mut harness = harness();
    // The macOS layout is what the native menu bar shows.
    for os in [OperatingSystem::Mac, OperatingSystem::Windows, OperatingSystem::Nix] {
        let menu = session_menu_on(&harness, os);
        let at = menu.iter().position(|label| label == NEW_SESSION).unwrap_or_else(|| panic!("a New Session submenu on {os:?}"));
        assert_eq!(menu[at + 1..at + 3], [ALPHA, BETA], "one entry per registered type, in registration order, on {os:?}");
    }

    run_from_session_menu(&mut harness, &[NEW_SESSION, BETA]);
    assert_eq!(tab_names(&harness), [HOME, BETA], "a new tab");
    assert_eq!(selected(&harness).as_deref(), Some(BETA));
    assert_eq!(harness.state().view_type(), "beta");
    assert_eq!(description(&mut harness).as_deref(), Some(""), "an empty session");

    run_from_session_menu(&mut harness, &[NEW_SESSION, ALPHA]);
    assert_eq!(tab_names(&harness), [HOME, BETA, ALPHA]);
}

#[test]
fn open_session_shows_home_from_the_menu_and_its_shortcut() {
    let mut harness = harness();
    run_from_session_menu(&mut harness, &[NEW_SESSION, ALPHA]);
    run_from_session_menu(&mut harness, &[OPEN_SESSION]);
    assert_eq!(selected(&harness).as_deref(), Some(HOME), "the menu shows Home");
    assert_eq!(tab_names(&harness), [HOME, ALPHA], "the Home tab already open");

    press(&mut harness, Modifiers::CTRL, Key::Tab);
    assert_eq!(selected(&harness).as_deref(), Some(ALPHA));
    press(&mut harness, Modifiers::COMMAND, Key::O);
    assert_eq!(selected(&harness).as_deref(), Some(HOME), "and so does Ctrl+O");
}

#[test]
fn clear_session_replaces_the_view_with_an_empty_session_of_the_same_type() {
    let mut harness = harness();
    run_from_session_menu(&mut harness, &[NEW_SESSION, BETA]);
    set_description(&mut harness, "notes");

    run_from_session_menu(&mut harness, &[CLEAR_SESSION]);
    assert_eq!(tab_names(&harness), [HOME, BETA], "in the same tab");
    assert_eq!(harness.state().view_type(), "beta", "of the same type");
    assert_eq!(description(&mut harness).as_deref(), Some(""), "empty");
}

#[test]
fn clear_session_on_an_unsaved_view_asks_first_and_cancel_keeps_it() {
    let mut harness = harness();
    run_from_session_menu(&mut harness, &[NEW_SESSION, ALPHA]);
    set_description(&mut harness, "notes");
    if let Some(view) = harness.state_mut().view_mut::<SessionView>() {
        view.unsaved = true;
    }

    run_from_session_menu(&mut harness, &[CLEAR_SESSION]);
    assert!(prompt_open(&harness), "Save / Don't Save / Cancel");
    press(&mut harness, Modifiers::NONE, Key::Escape);
    assert!(!prompt_open(&harness));
    assert_eq!(description(&mut harness).as_deref(), Some("notes"), "Cancel keeps the view");
    assert_eq!(focused(&harness), Some((Role::Tab, ALPHA.to_owned())), "and focus goes to its tab");

    run_from_session_menu(&mut harness, &[CLEAR_SESSION]);
    for _ in 0..10 {
        if focused(&harness).is_some_and(|(_, label)| label == DONT_SAVE) {
            break;
        }
        press(&mut harness, Modifiers::NONE, Key::Tab);
    }
    press(&mut harness, Modifiers::NONE, Key::Enter);
    assert!(!prompt_open(&harness));
    assert_eq!(description(&mut harness).as_deref(), Some(""), "Don't Save clears it");
    assert_eq!(tab_names(&harness), [HOME, ALPHA]);
}

// The menu row that ran Close Tab is gone by Cancel, so focus goes to the tab instead (#75).
#[test]
fn close_tab_from_the_menu_on_an_unsaved_view_gives_focus_to_its_tab_on_cancel() {
    let mut harness = harness();
    run_from_session_menu(&mut harness, &[NEW_SESSION, ALPHA]);
    if let Some(view) = harness.state_mut().view_mut::<SessionView>() {
        view.unsaved = true;
    }

    run_from_session_menu(&mut harness, &[CLOSE_CURRENT_TAB]);
    assert!(prompt_open(&harness), "Save / Don't Save / Cancel");
    press(&mut harness, Modifiers::NONE, Key::Escape);
    assert!(!prompt_open(&harness));
    assert_eq!(tab_names(&harness), [HOME, ALPHA], "Cancel keeps the tab");
    assert_eq!(focused(&harness), Some((Role::Tab, ALPHA.to_owned())), "and focus goes to it");
}

#[test]
fn session_settings_opens_the_dialog_from_the_menu_and_from_its_toolbar_button() {
    let mut harness = harness();
    run_from_session_menu(&mut harness, &[NEW_SESSION, ALPHA]);
    run_from_session_menu(&mut harness, &[SESSION_SETTINGS]);
    assert!(settings_open(&harness), "the menu opens it");
    press(&mut harness, Modifiers::NONE, Key::Escape);
    assert!(!settings_open(&harness));

    for _ in 0..20 {
        if focused(&harness).is_some_and(|(role, label)| role == Role::Button && label == SESSION_SETTINGS) {
            break;
        }
        press(&mut harness, Modifiers::NONE, Key::Tab);
    }
    let button = harness.get_by(|node| node.is_focused());
    assert!(button.accesskit_node().parent().is_some_and(|parent| parent.role() == Role::Toolbar), "Tab reaches its toolbar button");
    press(&mut harness, Modifiers::NONE, Key::Enter);
    assert!(settings_open(&harness), "the toolbar button opens it");
}

#[test]
fn home_has_neither_clear_session_nor_session_settings_and_no_toolbar() {
    let mut harness = harness();
    let menu = session_menu(&harness);
    assert!(menu.contains(&OPEN_SESSION.to_owned()));
    assert!(!menu.contains(&CLEAR_SESSION.to_owned()) && !menu.contains(&SESSION_SETTINGS.to_owned()), "nothing to clear or set on Home");
    assert!(harness.query_by_role(Role::Toolbar).is_none());
    for id in ["otd.clear_session", "otd.session_settings"] {
        assert!(!harness.state_mut().run(id), "{id} doesn't run on Home");
    }
}

#[test]
fn a_session_command_on_a_view_without_a_session_reports_that_it_failed() {
    let mut harness = harness();
    harness.state_mut().open(Box::new(SessionView { title: ALPHA, view_type: "alpha", session: None, unsaved: false }));
    harness.run();
    for id in ["otd.clear_session", "otd.session_settings"] {
        assert!(!harness.state_mut().run(id), "{id} fails rather than claiming success");
    }
    harness.run();
    assert!(!settings_open(&harness));
}
