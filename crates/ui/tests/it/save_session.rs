//! Save Session and Save Session As (E06-S1-T2), driven by keyboard and read back through the
//! accessibility tree and the settings store, which is a temporary folder of each test's own.

use egui::accesskit::{Live, Role};
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_settings::{Location, Store, sessions};
use otd_ui::{REPLACE, SAVE_SESSION_AS, Session, SessionType, Shell, View};

const ALPHA: &str = "Alpha Compare";

struct SessionView {
    session: Session,
}

impl View for SessionView {
    fn title(&self) -> String {
        ALPHA.to_owned()
    }

    fn view_type(&self) -> &'static str {
        "alpha"
    }

    fn session(&mut self) -> Option<&mut Session> {
        Some(&mut self.session)
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label(ALPHA);
    }
}

struct State {
    shell: Shell,
    store: Store,
}

/// A shell with an Alpha session open, whose inputs are `specs`, saving into `location`.
fn harness(location: Location, specs: &[&str]) -> Harness<'static, State> {
    let types = vec![SessionType {
        id: "alpha",
        name: ALPHA,
        kinds: &[],
        sides: &["Left", "Right"],
        new: || Box::new(SessionView { session: Session::new("alpha") }),
    }];
    let mut shell = Shell::new(types);
    let mut session = Session::new("alpha");
    session.specs = specs.iter().map(|s| (*s).to_owned()).collect();
    shell.open(Box::new(SessionView { session }));
    let (store, _) = otd_settings::open(location);
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut State| {
            state.shell.ui(ui);
            state.shell.sessions_ui(ui.ctx(), &state.store);
        },
        State { shell, store },
    );
    harness.run();
    harness
}

fn run(harness: &mut Harness<'_, State>, id: &str) {
    assert!(harness.state_mut().shell.run(id), "{id} runs");
    harness.run();
}

fn press(harness: &mut Harness<'_, State>, key: Key) {
    harness.key_press_modifiers(Modifiers::NONE, key);
    harness.run();
}

fn focused(harness: &Harness<'_, State>) -> Option<(Role, String)> {
    let node = harness.query_by(|node| node.is_focused())?;
    let node = node.accesskit_node();
    Some((node.role(), node.label().unwrap_or_default()))
}

/// Types `name` into the focused name edit, over what it holds, and presses Enter.
fn enter_name(harness: &mut Harness<'_, State>, name: &str) {
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness.get_by(|node| node.is_focused()).type_text(name);
    harness.run();
    press(harness, Key::Enter);
}

fn saved(harness: &Harness<'_, State>) -> std::collections::BTreeMap<String, Session> {
    let (saved, warnings) = sessions::load(&harness.state().store, &[]);
    assert_eq!(warnings, Vec::<String>::new());
    saved
}

fn dialog_open(harness: &Harness<'_, State>, name: &str) -> bool {
    harness.query_by(|node| matches!(node.role(), Role::Dialog | Role::AlertDialog) && node.label().is_some_and(|l| l.starts_with(name))).is_some()
}

#[test]
fn save_session_on_an_unnamed_session_asks_for_a_name_and_saves_under_it() {
    let dir = tempfile::tempdir().unwrap();
    let mut harness = harness(Location::Dir(dir.path().to_path_buf()), &["a.txt", "b.txt"]);
    run(&mut harness, "otd.save_session");
    assert!(dialog_open(&harness, SAVE_SESSION_AS), "an unnamed session asks for a name");
    assert_eq!(focused(&harness).map(|(role, _)| role), Some(Role::TextInput), "focus is in the name");
    enter_name(&mut harness, "Weekly");

    assert!(!dialog_open(&harness, SAVE_SESSION_AS));
    let saved = saved(&harness);
    assert_eq!(saved.get("Weekly").map(|s| s.specs.clone()), Some(vec!["a.txt".to_owned(), "b.txt".to_owned()]));
    assert_eq!(focused(&harness).map(|(role, _)| role), Some(Role::Tab), "focus goes back to the window");
}

#[test]
fn save_session_on_a_named_session_saves_in_place_without_asking_and_says_so() {
    let dir = tempfile::tempdir().unwrap();
    let mut harness = harness(Location::Dir(dir.path().to_path_buf()), &["a.txt", "b.txt"]);
    run(&mut harness, "otd.save_session_as");
    enter_name(&mut harness, "Weekly");

    harness.state_mut().shell.view_mut::<SessionView>().unwrap().session.specs[0] = "c.txt".to_owned();
    run(&mut harness, "otd.save_session");
    assert!(!dialog_open(&harness, SAVE_SESSION_AS), "a named session saves without a dialog");
    assert_eq!(saved(&harness).get("Weekly").map(|s| s.specs[0].clone()).as_deref(), Some("c.txt"));
    let announced = harness
        .query_all_by(|node| node.live() == Live::Polite)
        .filter_map(|node| node.accesskit_node().label().or(node.accesskit_node().value()))
        .collect::<Vec<_>>();
    assert!(announced.iter().any(|text| text.contains("Weekly")), "the save is announced: {announced:?}");
}

#[test]
fn saving_under_a_name_already_saved_asks_first_and_cancel_keeps_the_saved_session() {
    let dir = tempfile::tempdir().unwrap();
    let mut harness = harness(Location::Dir(dir.path().to_path_buf()), &["new.txt"]);
    let mut other = Session::new("alpha");
    other.specs = vec!["old.txt".to_owned()];
    sessions::save(&harness.state().store, "Weekly", &other).unwrap();

    run(&mut harness, "otd.save_session_as");
    enter_name(&mut harness, "Weekly");
    assert!(dialog_open(&harness, "A session named \"Weekly\""), "it asks before replacing");
    assert_eq!(focused(&harness).map(|(_, name)| name).as_deref(), Some(otd_ui::CANCEL), "the safe answer has focus");
    press(&mut harness, Key::Enter);
    assert!(dialog_open(&harness, SAVE_SESSION_AS), "Cancel goes back to the name");
    assert_eq!(saved(&harness).get("Weekly").map(|s| s.specs[0].clone()).as_deref(), Some("old.txt"), "Cancel replaces nothing");

    press(&mut harness, Key::Enter);
    harness.get_by_label(REPLACE).click();
    harness.run();
    assert_eq!(saved(&harness).get("Weekly").map(|s| s.specs[0].clone()).as_deref(), Some("new.txt"), "Replace replaces it");
}

#[test]
fn a_save_that_fails_keeps_the_name_dialog_open_and_announces_why() {
    // Settings kept in memory are never written.
    let mut harness = harness(Location::Memory, &["a.txt"]);
    run(&mut harness, "otd.save_session");
    enter_name(&mut harness, "Weekly");
    assert!(dialog_open(&harness, SAVE_SESSION_AS));
    let errors = harness
        .query_all_by(|node| node.live() == Live::Assertive)
        .filter_map(|node| node.accesskit_node().label().or(node.accesskit_node().value()))
        .collect::<Vec<_>>();
    assert!(errors.iter().any(|text| text.contains("Weekly")), "the reason names the session: {errors:?}");
}

#[test]
fn esc_or_cancel_in_the_name_dialog_saves_nothing_and_focus_goes_back_to_the_tab() {
    let dir = tempfile::tempdir().unwrap();
    let mut harness = harness(Location::Dir(dir.path().to_path_buf()), &["a.txt"]);
    run(&mut harness, "otd.save_session_as");
    harness.get_by(|node| node.is_focused()).type_text("Weekly");
    harness.run();
    press(&mut harness, Key::Escape);
    assert!(!dialog_open(&harness, SAVE_SESSION_AS), "Esc closes the name dialog");
    assert!(saved(&harness).is_empty(), "Esc saves nothing");
    assert_eq!(focused(&harness).map(|(role, _)| role), Some(Role::Tab), "focus goes back to the tab after Esc");

    run(&mut harness, "otd.save_session_as");
    harness.get_by(|node| node.is_focused()).type_text("Weekly");
    harness.run();
    harness.get_by_label(otd_ui::CANCEL).click();
    harness.run();
    assert!(!dialog_open(&harness, SAVE_SESSION_AS), "Cancel closes the name dialog");
    assert!(saved(&harness).is_empty(), "Cancel saves nothing");
    assert_eq!(focused(&harness).map(|(role, _)| role), Some(Role::Tab), "focus goes back to the tab after Cancel");
}

#[test]
fn esc_on_the_replace_question_goes_back_to_the_name_and_replaces_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let mut harness = harness(Location::Dir(dir.path().to_path_buf()), &["new.txt"]);
    let mut other = Session::new("alpha");
    other.specs = vec!["old.txt".to_owned()];
    sessions::save(&harness.state().store, "Weekly", &other).unwrap();

    run(&mut harness, "otd.save_session_as");
    enter_name(&mut harness, "Weekly");
    assert!(dialog_open(&harness, "A session named \"Weekly\""));
    press(&mut harness, Key::Escape);
    assert!(!dialog_open(&harness, "A session named \"Weekly\""), "Esc closes the question");
    assert!(dialog_open(&harness, SAVE_SESSION_AS), "Esc goes back to the name");
    assert_eq!(saved(&harness).get("Weekly").map(|s| s.specs[0].clone()).as_deref(), Some("old.txt"), "Esc replaces nothing");
}
