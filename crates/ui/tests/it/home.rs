//! Home's sessions tree (E06-S1-T3), driven by keyboard and pointer and read back through the
//! accessibility tree: the New, Saved and Auto-saved branches, opening their entries, and moving
//! through the rows with Up and Down.

use std::collections::BTreeMap;

use egui::accesskit::Role;
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_ui::{AUTO_SAVED, NEW, SAVED, Session, SessionType, Setting, Shell, Value, View};

const ALPHA: &str = "Alpha Compare";
const BETA: &str = "Beta Compare";
const TAB: &str = "Comparison";
/// What the test view shows of its session.
const SHOWS: &str = "Shows";

/// A test session view that shows its specs and its one setting.
struct SessionView {
    title: &'static str,
    session: Session,
}

impl View for SessionView {
    fn title(&self) -> String {
        self.title.to_owned()
    }

    fn session(&mut self) -> Option<&mut Session> {
        Some(&mut self.session)
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("{SHOWS}: {} {:?}", self.session.specs.join("|"), self.session.rule(TAB, "ignore_case")));
    }
}

fn session_types() -> Vec<SessionType> {
    vec![
        SessionType {
            id: "alpha",
            name: ALPHA,
            kinds: &[],
            sides: &["Left", "Right"],
            new: || Box::new(SessionView { title: ALPHA, session: Session::new("alpha") }),
        },
        SessionType {
            id: "beta",
            name: BETA,
            kinds: &[],
            sides: &["Left", "Right"],
            new: || Box::new(SessionView { title: BETA, session: Session::new("beta") }),
        },
    ]
}

/// A setting a test feature contributes to beta, off by default.
fn settings() -> Vec<Setting> {
    vec![Setting { session_type: "beta", tab: TAB, id: "ignore_case", label: "Ignore case", default: Value::Bool(false), choices: &[] }]
}

/// A beta session with both specs and its setting turned on: nothing like an empty one.
fn weekly() -> Session {
    let mut session = Session::new("beta");
    session.specs = vec!["/work/left".into(), "/work/right".into()];
    session.rules.entry(TAB.into()).or_default().insert("ignore_case".into(), Value::Bool(true));
    session
}

fn harness() -> Harness<'static, Shell> {
    let shell = Shell::new(session_types()).with_settings(settings()).unwrap();
    let mut harness = Harness::builder().build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), shell);
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
    Some((node.role(), node.label().unwrap_or_default()))
}

fn tab_to_row(harness: &mut Harness<'_, Shell>, name: &str) {
    for _ in 0..20 {
        if focused(harness) == Some((Role::TreeItem, name.to_owned())) {
            return;
        }
        press(harness, Key::Tab);
    }
    panic!("Tab never reached the row {name:?}");
}

/// The tree's rows, in order: each one's name and level.
fn rows(harness: &Harness<'_, Shell>) -> Vec<(String, usize)> {
    harness
        .query_all_by_role(Role::TreeItem)
        .map(|row| {
            let node = row.accesskit_node();
            (node.label().unwrap_or_default(), node.level().unwrap_or_default())
        })
        .collect()
}

/// The selected tab's name, and what its view shows.
fn opened(harness: &Harness<'_, Shell>) -> (String, Option<String>) {
    let tab = harness.query_all_by_role(Role::Tab).find(|tab| tab.accesskit_node().is_selected() == Some(true));
    let tab = tab.and_then(|tab| tab.accesskit_node().label()).unwrap_or_default();
    let shows = harness
        .query_all_by_role(Role::Label)
        .filter_map(|n| n.accesskit_node().label().or_else(|| n.accesskit_node().value()))
        .find(|l| l.starts_with(SHOWS));
    (tab, shows)
}

#[test]
fn the_new_branch_lists_every_session_type_under_it_before_the_saved_and_auto_saved_branches() {
    let harness = harness();
    let branches: Vec<_> = rows(&harness).into_iter().filter(|(_, level)| *level == 1).map(|(name, _)| name).collect();
    assert_eq!(branches, [NEW, SAVED, AUTO_SAVED]);
    let rows = rows(&harness);
    let new = rows.iter().position(|(name, _)| name == NEW).unwrap();
    let under_new: Vec<_> = rows[new + 1..].iter().take_while(|(_, level)| *level > 1).map(|(name, _)| name.as_str()).collect();
    assert_eq!(under_new, [ALPHA, BETA], "one entry per registered session type");
}

#[test]
fn a_session_saved_while_home_shows_is_listed_at_once_and_opens_with_its_specs_and_settings() {
    let mut harness = harness();
    harness.state_mut().set_saved_sessions(BTreeMap::from([("Weekly".to_owned(), weekly())]));
    harness.run();
    let rows = rows(&harness);
    let saved = rows.iter().position(|(name, _)| name == SAVED).unwrap();
    assert_eq!(rows.get(saved + 1), Some(&("Weekly".to_owned(), 2)), "listed under Saved, without reopening Home");

    tab_to_row(&mut harness, "Weekly");
    press(&mut harness, Key::Enter);
    let (tab, shows) = opened(&harness);
    assert_eq!(tab, BETA, "in its type's view");
    let shows = shows.unwrap_or_default();
    assert!(shows.contains("/work/left|/work/right") && shows.contains("Bool(true)"), "its specs and settings: {shows}");
}

#[test]
fn double_clicking_an_auto_saved_session_listed_by_its_specs_opens_it_restored() {
    let mut harness = harness();
    harness.state_mut().set_auto_saved_sessions(vec![weekly()]);
    harness.run();
    let name = "/work/left — /work/right";
    let row = harness.get_by_role_and_label(Role::TreeItem, name).rect().center();
    harness.input_mut().events.push(egui::Event::PointerMoved(row));
    harness.step();
    // Two presses and releases in one frame: well inside egui's double-click delay.
    for pressed in [true, false, true, false] {
        let button = egui::Event::PointerButton { pos: row, button: egui::PointerButton::Primary, pressed, modifiers: Modifiers::NONE };
        harness.input_mut().events.push(button);
    }
    harness.run();
    let (tab, shows) = opened(&harness);
    assert_eq!(tab, BETA);
    assert!(shows.unwrap_or_default().contains("Bool(true)"), "its settings are restored");
}

#[test]
fn up_and_down_move_focus_one_row_at_a_time_and_stop_at_the_ends() {
    let mut harness = harness();
    tab_to_row(&mut harness, NEW);
    press(&mut harness, Key::ArrowUp);
    assert_eq!(focused(&harness), Some((Role::TreeItem, NEW.to_owned())), "Up on the first row stays");
    for expected in [ALPHA, BETA, SAVED, AUTO_SAVED, AUTO_SAVED] {
        press(&mut harness, Key::ArrowDown);
        assert_eq!(focused(&harness), Some((Role::TreeItem, expected.to_owned())));
    }
    press(&mut harness, Key::ArrowUp);
    assert_eq!(focused(&harness), Some((Role::TreeItem, SAVED.to_owned())));
    let selected: Vec<_> = harness.query_all_by_role(Role::TreeItem).filter(|r| r.accesskit_node().is_selected() == Some(true)).collect();
    assert_eq!(selected.len(), 1, "one row is selected");
    assert_eq!(selected[0].accesskit_node().label().as_deref(), Some(SAVED), "the selection follows focus");
}

#[test]
fn opening_a_saved_session_of_a_type_that_is_not_registered_says_why() {
    let mut harness = harness();
    let gone = Session { type_id: "gone".into(), ..weekly() };
    harness.state_mut().set_saved_sessions(BTreeMap::from([("Old".to_owned(), gone)]));
    harness.run();
    tab_to_row(&mut harness, "Old");
    let announced = |harness: &Harness<'_, Shell>| harness.query_all_by(|n| n.live() == egui::accesskit::Live::Polite).count();
    let before = announced(&harness);
    press(&mut harness, Key::Enter);
    assert_eq!(opened(&harness).0, otd_ui::HOME, "nothing opened");
    assert!(announced(&harness) > before, "the failure is announced");
}
