//! Sessions in the shell, driven by keyboard and read back through the accessibility tree: the
//! Session Settings dialog with test session types and options contributed by test features, and
//! the session commands.

use egui::accesskit::Role;
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_ui::{
    CANCEL, DESCRIPTION, HOME, INPUTS, Kind, OK, SCOPE, SESSION_SETTINGS, Session, SessionType, Setting, Shell, THIS_VIEW_ONLY, Value, View,
};

const ALPHA: &str = "Alpha Compare";
const BETA: &str = "Beta Compare";
const LEFT: &str = "Left";
const COMPARISON: &str = "Comparison";
const DISPLAY: &str = "Display";
const IGNORE_CASE: &str = "Ignore case";
const TAB_WIDTH: &str = "Tab width";
/// What the test view shows of its session.
const SHOWS_SPECS: &str = "Shows inputs";
const SHOWS_DESCRIPTION: &str = "About";
const SHOWS_RULE: &str = "Ignoring case";
const SHOWS_DISPLAY: &str = "Display values";

/// A test session view that shows its inputs, description and one rule.
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
        ui.label(format!("{SHOWS_SPECS}: {}", self.session.specs.join("|")));
        ui.label(format!("{SHOWS_DESCRIPTION}: {}", self.session.description));
        ui.label(format!("{SHOWS_RULE}: {:?}", self.session.rule(COMPARISON, "ignore_case")));
        ui.label(format!("{SHOWS_DISPLAY}: {:?}", self.session.rules.get(DISPLAY)));
    }
}

fn session_types() -> Vec<SessionType> {
    vec![
        SessionType {
            id: "alpha",
            name: ALPHA,
            kinds: &[Kind::Text],
            sides: &[LEFT, "Right"],
            new: || Box::new(SessionView { title: ALPHA, session: Session::new("alpha") }),
        },
        SessionType {
            id: "beta",
            name: BETA,
            kinds: &[Kind::Table, Kind::Text],
            sides: &[LEFT, "Right"],
            new: || Box::new(SessionView { title: BETA, session: Session::new("beta") }),
        },
    ]
}

fn setting(session_type: &'static str, tab: &'static str, id: &'static str, label: &'static str, default: Value) -> Setting {
    Setting { session_type, tab, id, label, default, choices: &[] }
}

/// What three test features contribute: two of them to alpha's Comparison tab, and one of every
/// value type to alpha's Display tab; and a tab for beta only.
fn settings() -> Vec<Setting> {
    let first_feature = vec![setting("alpha", COMPARISON, "ignore_case", IGNORE_CASE, Value::Bool(false))];
    let second_feature = vec![
        setting("alpha", COMPARISON, "tab_width", TAB_WIDTH, Value::Number(4.0)),
        setting("alpha", DISPLAY, "font", "Font", Value::Text("Mono".into())),
        setting("alpha", DISPLAY, "folder", "Folder", Value::Path("/tmp".into())),
        setting("alpha", DISPLAY, "accent", "Accent", Value::Colour([10, 20, 30, 255])),
        Setting { choices: &["Lines", "Words"], ..setting("alpha", DISPLAY, "unit", "Unit", Value::Choice("Lines".into())) },
    ];
    let third_feature = vec![setting("beta", "Beta only", "other", "Other", Value::Bool(true))];
    [first_feature, second_feature, third_feature].concat()
}

fn shell() -> Shell {
    Shell::new(session_types()).with_settings(settings()).unwrap()
}

/// A harness with tabs Home, Alpha and Alpha, the second Alpha selected.
fn two_alphas() -> Harness<'static, Shell> {
    let mut shell = shell();
    for _ in 0..2 {
        shell.new_session("alpha").unwrap();
    }
    let mut harness = Harness::builder().build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), shell);
    harness.run();
    harness
}

/// [`two_alphas`] with the Session Settings dialog open.
fn dialog() -> Harness<'static, Shell> {
    let mut harness = two_alphas();
    harness.state_mut().session_settings().unwrap();
    harness.run();
    harness
}

fn press<S>(harness: &mut Harness<'_, S>, modifiers: Modifiers, key: Key) {
    harness.key_press_modifiers(modifiers, key);
    harness.run();
}

fn type_text<S>(harness: &mut Harness<'_, S>, text: &str) {
    harness.get_by(|node| node.is_focused()).type_text(text);
    harness.run();
}

fn focused<S>(harness: &Harness<'_, S>) -> Option<(Role, String)> {
    let node = harness.query_by(|node| node.is_focused())?;
    let node = node.accesskit_node();
    Some((node.role(), node.label().or_else(|| node.value()).unwrap_or_default()))
}

fn tab_to<S>(harness: &mut Harness<'_, S>, role: Role, name: &str) {
    for _ in 0..30 {
        if focused(harness).is_some_and(|(r, label)| r == role && label == name) {
            return;
        }
        press(harness, Modifiers::NONE, Key::Tab);
    }
    panic!("Tab never reached {name:?}");
}

/// The text after `prefix: ` of the label the view shows it in.
fn shown(harness: &Harness<'_, Shell>, prefix: &str) -> String {
    let start = format!("{prefix}: ");
    let node = harness.get_by(|node| node.role() == Role::Label && node.label().or_else(|| node.value()).is_some_and(|t| t.starts_with(&start)));
    let node = node.accesskit_node();
    node.label().or_else(|| node.value()).unwrap_or_default()[start.len()..].to_owned()
}

fn is_open(harness: &Harness<'_, Shell>) -> bool {
    harness.query_by(|node| node.role() == Role::Dialog && node.label().as_deref() == Some(SESSION_SETTINGS)).is_some()
}

/// The dialog's tab names, in order.
fn dialog_tabs(harness: &Harness<'_, Shell>) -> Vec<String> {
    harness
        .query_all_by_role(Role::Tab)
        .filter(|tab| tab.accesskit_node().parent().is_some_and(|list| list.label().as_deref() == Some(SESSION_SETTINGS)))
        .map(|tab| tab.accesskit_node().label().unwrap_or_default())
        .collect()
}

/// The selected tab of the window's tab strip.
fn selected_window_tab(harness: &Harness<'_, Shell>) -> Option<String> {
    harness
        .query_all_by_role(Role::Tab)
        .map(|tab| tab.accesskit_node())
        .find(|tab| tab.is_selected() == Some(true) && tab.parent().is_some_and(|list| list.label().as_deref() != Some(SESSION_SETTINGS)))
        .and_then(|tab| tab.label())
}

#[test]
fn the_shell_returns_the_first_session_type_registered_for_a_kind() {
    let shell = shell();
    assert_eq!(shell.session_type_for(Kind::Text).map(|t| t.id), Some("alpha"), "alpha and beta both open Text; alpha came first");
    assert_eq!(shell.session_type_for(Kind::Table).map(|t| t.id), Some("beta"));
    assert!(shell.session_type_for(Kind::Hex).is_none());
}

#[test]
fn the_dialog_shows_specs_then_the_tabs_contributed_for_its_session_type() {
    let harness = dialog();
    assert_eq!(dialog_tabs(&harness), [INPUTS, COMPARISON, DISPLAY], "beta's tab isn't alpha's, and two features' Comparison is one tab");
}

#[test]
fn a_tab_two_features_contribute_to_holds_both_their_options() {
    let mut harness = dialog();
    tab_to(&mut harness, Role::Tab, COMPARISON);
    press(&mut harness, Modifiers::NONE, Key::Enter);
    assert!(harness.query_by_label(IGNORE_CASE).is_some(), "the first feature's option");
    assert!(harness.query_by_label(TAB_WIDTH).is_some(), "the second feature's option");
}

#[test]
fn the_specs_tab_has_an_input_per_side_and_the_description() {
    let harness = dialog();
    let inputs: Vec<_> = harness.query_all_by_role(Role::TextInput).filter_map(|node| node.accesskit_node().label()).collect();
    assert_eq!(inputs, [LEFT, "Right"]);
    assert!(harness.query_by(|node| node.role() == Role::MultilineTextInput && node.label().as_deref() == Some(DESCRIPTION)).is_some());
}

#[test]
fn ok_applies_the_changes_to_the_current_view_only() {
    let mut harness = dialog();
    tab_to(&mut harness, Role::TextInput, LEFT);
    type_text(&mut harness, "left.txt");
    tab_to(&mut harness, Role::MultilineTextInput, DESCRIPTION);
    type_text(&mut harness, "notes");
    tab_to(&mut harness, Role::Tab, COMPARISON);
    press(&mut harness, Modifiers::NONE, Key::Enter);
    tab_to(&mut harness, Role::CheckBox, IGNORE_CASE);
    press(&mut harness, Modifiers::NONE, Key::Space);
    tab_to(&mut harness, Role::Button, OK);
    press(&mut harness, Modifiers::NONE, Key::Enter);
    assert!(!is_open(&harness));
    let changed = (shown(&harness, SHOWS_SPECS), shown(&harness, SHOWS_DESCRIPTION), shown(&harness, SHOWS_RULE));
    assert_eq!(changed, ("left.txt|".to_owned(), "notes".to_owned(), "Some(Bool(true))".to_owned()), "the current view changed");

    let untouched = |harness: &Harness<'_, Shell>| (shown(harness, SHOWS_SPECS), shown(harness, SHOWS_DESCRIPTION), shown(harness, SHOWS_RULE));
    let empty = ("|".to_owned(), String::new(), "Some(Bool(false))".to_owned());
    press(&mut harness, Modifiers::CTRL | Modifiers::SHIFT, Key::Tab);
    assert_eq!(untouched(&harness), empty, "the other view of the same type didn't");
    harness.state_mut().new_session("alpha").unwrap();
    harness.run();
    assert_eq!(untouched(&harness), empty, "nor does a new session of the type");
}

#[test]
fn enter_in_a_single_line_field_is_ok() {
    let mut harness = dialog();
    tab_to(&mut harness, Role::TextInput, LEFT);
    type_text(&mut harness, "left.txt");
    press(&mut harness, Modifiers::NONE, Key::Enter);
    assert!(!is_open(&harness));
    assert_eq!(shown(&harness, SHOWS_SPECS), "left.txt|");
}

#[test]
fn esc_and_cancel_discard_the_changes_and_give_focus_back_to_the_window() {
    for close in [None, Some(CANCEL)] {
        let mut harness = dialog();
        tab_to(&mut harness, Role::MultilineTextInput, DESCRIPTION);
        type_text(&mut harness, "notes");
        match close {
            None => press(&mut harness, Modifiers::NONE, Key::Escape),
            Some(cancel) => {
                tab_to(&mut harness, Role::Button, cancel);
                press(&mut harness, Modifiers::NONE, Key::Enter);
            }
        }
        assert!(!is_open(&harness), "{close:?} closes the dialog");
        assert_eq!(shown(&harness, SHOWS_DESCRIPTION), "", "{close:?} discards the change");
        let focus = focused(&harness);
        assert!(
            focus.is_some_and(|(role, name)| role == Role::Tab && Some(name) == selected_window_tab(&harness)),
            "{close:?}: focus on the window's tab"
        );
    }
}

#[test]
fn a_click_outside_the_dialog_keeps_it_and_its_changes() {
    let mut harness = dialog();
    tab_to(&mut harness, Role::MultilineTextInput, DESCRIPTION);
    type_text(&mut harness, "notes");
    harness.get_by(|node| node.role() == Role::Tab && node.label().as_deref() == Some(HOME)).click();
    harness.run();
    assert!(is_open(&harness));
    let description = harness.get_by(|node| node.role() == Role::MultilineTextInput && node.label().as_deref() == Some(DESCRIPTION));
    assert_eq!(description.accesskit_node().value().as_deref(), Some("notes"));
}

#[test]
fn focus_moves_into_the_dialog_and_tab_never_leaves_it() {
    let mut harness = dialog();
    assert_eq!(focused(&harness), Some((Role::Tab, INPUTS.to_owned())));
    for _ in 0..25 {
        press(&mut harness, Modifiers::NONE, Key::Tab);
        let node = harness.get_by(|node| node.is_focused());
        let mut ancestor = node.accesskit_node().parent();
        while let Some(node) = ancestor.filter(|node| node.role() != Role::Dialog) {
            ancestor = node.parent();
        }
        assert!(ancestor.is_some(), "focus left the dialog for {:?}", focused(&harness));
    }
}

#[test]
fn the_windows_shortcuts_wait_while_the_dialog_is_open() {
    let mut harness = dialog();
    for key in [Key::W, Key::T] {
        press(&mut harness, Modifiers::COMMAND, key);
    }
    press(&mut harness, Modifiers::CTRL, Key::Tab);
    assert!(is_open(&harness));
    assert_eq!(harness.query_all_by_role(Role::Tab).count() - dialog_tabs(&harness).len(), 3, "the window still has its three tabs");
    assert_eq!(selected_window_tab(&harness).as_deref(), Some(ALPHA));
}

#[test]
fn clear_session_gives_an_empty_session_of_the_same_type() {
    let mut harness = dialog();
    tab_to(&mut harness, Role::TextInput, LEFT);
    type_text(&mut harness, "left.txt");
    press(&mut harness, Modifiers::NONE, Key::Enter);
    harness.state_mut().clear_session().unwrap();
    harness.run();
    assert_eq!(shown(&harness, SHOWS_SPECS), "|", "an input per side, all empty");
    assert_eq!(shown(&harness, SHOWS_RULE), "Some(Bool(false))", "the contributed defaults");
    assert_eq!(selected_window_tab(&harness).as_deref(), Some(ALPHA), "of the same type, in the same tab");
    assert_eq!(harness.query_all_by_role(Role::Tab).count(), 3);
}

#[test]
fn open_session_selects_home_or_opens_it() {
    let mut harness = two_alphas();
    harness.state_mut().open_session();
    harness.run();
    assert_eq!(selected_window_tab(&harness).as_deref(), Some(HOME));
    assert_eq!(harness.query_all_by_role(Role::Tab).count(), 3, "Home was already open");

    press(&mut harness, Modifiers::COMMAND, Key::W);
    harness.state_mut().open_session();
    harness.run();
    assert_eq!(selected_window_tab(&harness).as_deref(), Some(HOME));
    assert_eq!(harness.query_all_by_role(Role::Tab).count(), 3, "with no Home open, it opens one");
}

#[test]
fn each_value_type_gets_a_control_named_by_its_label() {
    let mut harness = dialog();
    tab_to(&mut harness, Role::Tab, DISPLAY);
    press(&mut harness, Modifiers::NONE, Key::Enter);
    for (label, role) in [("Font", Role::TextInput), ("Folder", Role::TextInput), ("Accent", Role::ColorWell), ("Unit", Role::ComboBox)] {
        let found = harness.query_by(|node| node.role() == role && node.label().is_some_and(|name| name.starts_with(label)));
        assert!(found.is_some(), "{label}: a {role:?} named by it");
    }
}

#[test]
fn the_scope_dropdown_offers_use_for_this_view_only() {
    let mut harness = dialog();
    tab_to(&mut harness, Role::ComboBox, SCOPE);
    press(&mut harness, Modifiers::NONE, Key::Space);
    press(&mut harness, Modifiers::NONE, Key::ArrowDown);
    assert_eq!(focused(&harness).map(|(_, name)| name).as_deref(), Some(THIS_VIEW_ONLY));
}

#[test]
fn a_choice_picked_by_keyboard_is_applied() {
    let mut harness = dialog();
    let before = shown(&harness, SHOWS_DISPLAY);
    tab_to(&mut harness, Role::Tab, DISPLAY);
    press(&mut harness, Modifiers::NONE, Key::Enter);
    tab_to(&mut harness, Role::ComboBox, "Unit");
    press(&mut harness, Modifiers::NONE, Key::Space);
    tab_to(&mut harness, Role::Button, "Words");
    press(&mut harness, Modifiers::NONE, Key::Enter);
    tab_to(&mut harness, Role::Button, OK);
    press(&mut harness, Modifiers::NONE, Key::Enter);
    let after = shown(&harness, SHOWS_DISPLAY);
    assert!(after != before && after.contains(r#"Choice("Words")"#), "{before} became {after}");
}

#[test]
fn a_colour_changed_by_keyboard_is_written_back() {
    let accent = setting("alpha", DISPLAY, "accent", "Accent", Value::Colour([10, 20, 30, 255]));
    let start = accent.default.clone();
    // ponytail: egui's colour picker draws its copy button with an emoji the bundled fonts lack.
    let mut harness = Harness::builder().allow_missing_glyphs().build_ui_state(
        move |ui, value: &mut Value| {
            otd_ui::edit_value(ui, &accent, value);
        },
        start.clone(),
    );
    harness.run();
    tab_to(&mut harness, Role::ColorWell, "Accent");
    press(&mut harness, Modifiers::NONE, Key::Space);
    tab_to(&mut harness, Role::SpinButton, "R");
    type_text(&mut harness, "5");
    press(&mut harness, Modifiers::NONE, Key::Enter);
    assert_ne!(*harness.state(), start);
}
