//! The shell driven by keyboard, read back through the accessibility tree: tabs, Home and the
//! status bar, with two test session types.

use std::time::Duration;

use egui::accesskit::{Live, Role};
use egui::os::OperatingSystem;
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_ui::{Field, HOME, SessionType, Shell, Status, View};

const ALPHA: &str = "Alpha Compare";
const EDIT_CONTROL: &str = "Edit control";
const BETA: &str = "Beta Compare";

/// An empty test session that publishes fixed status fields.
struct TestView {
    title: &'static str,
    result: &'static str,
    field: &'static str,
}

impl View for TestView {
    fn title(&self) -> String {
        self.title.to_owned()
    }

    fn status(&self) -> Status {
        Status {
            result: Some(Field { id: "result", label: "Result".into(), value: self.result.into() }),
            fields: vec![Field { id: "own", label: "Own".into(), value: self.field.into() }],
            insert: None,
            load_time: Some(Duration::from_millis(1500)),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label(self.title);
    }
}

/// A test view that edits: one focusable control, and the insert/overwrite field.
struct EditView {
    insert: bool,
}

impl View for EditView {
    fn title(&self) -> String {
        "Edit".to_owned()
    }

    fn status(&self) -> Status {
        Status { insert: Some(self.insert), ..Status::default() }
    }

    fn toggle_insert(&mut self) {
        self.insert = !self.insert;
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let _ = ui.button(EDIT_CONTROL);
    }
}

fn session_types() -> Vec<SessionType> {
    vec![
        SessionType {
            id: "alpha",
            name: ALPHA,
            kinds: &[],
            sides: &[],
            new: || Box::new(TestView { title: ALPHA, result: "Alpha result", field: "Alpha field" }),
        },
        SessionType {
            id: "beta",
            name: BETA,
            kinds: &[],
            sides: &[],
            new: || Box::new(TestView { title: BETA, result: "Beta result", field: "Beta field" }),
        },
    ]
}

fn harness_on(os: OperatingSystem) -> Harness<'static, Shell> {
    let mut harness = Harness::builder().with_os(os).build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), Shell::new(session_types()));
    harness.run();
    harness
}

fn harness() -> Harness<'static, Shell> {
    harness_on(OperatingSystem::Mac)
}

/// A harness with tabs Home, Alpha and Beta, Beta selected.
fn three_tabs() -> Harness<'static, Shell> {
    let mut harness = harness();
    for new in session_types().into_iter().map(|t| t.new) {
        harness.state_mut().open(new());
    }
    harness.run();
    harness
}

fn press(harness: &mut Harness<'_, Shell>, modifiers: Modifiers, key: Key) {
    harness.key_press_modifiers(modifiers, key);
    harness.run();
}

/// Each tab's name and whether it is selected, in strip order.
fn tabs(harness: &Harness<'_, Shell>) -> Vec<(String, bool)> {
    harness
        .query_all_by_role(Role::Tab)
        .map(|tab| {
            let node = tab.accesskit_node();
            (node.label().unwrap_or_default(), node.is_selected() == Some(true))
        })
        .collect()
}

fn selected(harness: &Harness<'_, Shell>) -> Option<String> {
    tabs(harness).into_iter().find(|(_, selected)| *selected).map(|(name, _)| name)
}

fn focused(harness: &Harness<'_, Shell>) -> Option<(Role, String)> {
    let node = harness.query_by(|node| node.is_focused())?;
    let node = node.accesskit_node();
    Some((node.role(), node.label().or_else(|| node.value()).unwrap_or_default()))
}

/// Focus is on the selected tab in the strip.
fn focus_is_on_selected_tab(harness: &Harness<'_, Shell>) -> bool {
    focused(harness).is_some_and(|(role, name)| role == Role::Tab && Some(name) == selected(harness))
}

/// The text of every label in the window.
fn texts(harness: &Harness<'_, Shell>) -> Vec<String> {
    harness.query_all_by_role(Role::Label).filter_map(|node| node.accesskit_node().label().or_else(|| node.accesskit_node().value())).collect()
}

fn tab_to(harness: &mut Harness<'_, Shell>, role: Role, name: &str) {
    for _ in 0..20 {
        if focused(harness).is_some_and(|(r, label)| r == role && label == name) {
            return;
        }
        press(harness, Modifiers::NONE, Key::Tab);
    }
    panic!("Tab never reached {name:?}");
}

#[test]
fn each_session_type_has_a_new_entry_on_home_that_opens_it_in_a_new_focused_tab() {
    let mut harness = harness();
    for name in [ALPHA, BETA] {
        tab_to(&mut harness, Role::TreeItem, name);
        press(&mut harness, Modifiers::NONE, Key::Enter);
        assert_eq!(tabs(&harness).len(), 2, "{name} opens in a new tab");
        assert_eq!(selected(&harness).as_deref(), Some(name), "the new tab is selected");
        assert!(focus_is_on_selected_tab(&harness), "focus moves to the new tab");
        // Back to Home alone for the next type.
        press(&mut harness, Modifiers::COMMAND, Key::W);
    }
}

#[test]
fn new_tab_opens_home_in_a_new_tab_that_takes_focus() {
    let mut harness = three_tabs();
    let before = tabs(&harness).len();
    press(&mut harness, Modifiers::COMMAND, Key::T);
    assert_eq!(tabs(&harness).len(), before + 1);
    assert_eq!(selected(&harness).as_deref(), Some(HOME));
    assert!(focus_is_on_selected_tab(&harness));
}

#[test]
fn close_tab_selects_the_next_tab_or_else_the_previous_and_focus_follows() {
    let mut harness = three_tabs();
    press(&mut harness, Modifiers::CTRL | Modifiers::SHIFT, Key::Tab);
    assert_eq!(selected(&harness).as_deref(), Some(ALPHA));

    press(&mut harness, Modifiers::COMMAND, Key::W);
    assert_eq!(tabs(&harness).iter().map(|(name, _)| name.as_str()).collect::<Vec<_>>(), [HOME, BETA]);
    assert_eq!(selected(&harness).as_deref(), Some(BETA), "the tab after the closed one");
    assert!(focus_is_on_selected_tab(&harness));

    press(&mut harness, Modifiers::COMMAND, Key::W);
    assert_eq!(selected(&harness).as_deref(), Some(HOME), "with none after, the tab before");
    assert!(focus_is_on_selected_tab(&harness));
}

#[test]
fn closing_the_last_tab_leaves_home_with_focus() {
    let mut harness = harness();
    harness.state_mut().open((session_types()[0].new)());
    press(&mut harness, Modifiers::COMMAND, Key::W);
    press(&mut harness, Modifiers::COMMAND, Key::W);
    assert_eq!(tabs(&harness), [(HOME.to_owned(), true)]);
    assert!(focus_is_on_selected_tab(&harness));
}

#[test]
fn ctrl_tab_and_ctrl_shift_tab_switch_tabs_wrapping_round_and_focus_follows() {
    let mut harness = three_tabs();
    press(&mut harness, Modifiers::CTRL, Key::Tab);
    assert_eq!(selected(&harness).as_deref(), Some(HOME), "Ctrl+Tab wraps from the last tab to the first");
    assert!(focus_is_on_selected_tab(&harness), "and Ctrl+Tab doesn't also move focus on");

    press(&mut harness, Modifiers::CTRL | Modifiers::SHIFT, Key::Tab);
    assert_eq!(selected(&harness).as_deref(), Some(BETA), "Ctrl+Shift+Tab wraps back");
    assert!(focus_is_on_selected_tab(&harness));
}

#[test]
fn cmd_shift_brackets_switch_tabs_on_macos_only() {
    for (os, switches) in [(OperatingSystem::Mac, true), (OperatingSystem::Windows, false), (OperatingSystem::Nix, false)] {
        let mut harness = harness_on(os);
        for new in session_types().into_iter().map(|t| t.new) {
            harness.state_mut().open(new());
        }
        harness.run();
        press(&mut harness, Modifiers::COMMAND | Modifiers::SHIFT, Key::OpenBracket);
        let expected = if switches { ALPHA } else { BETA };
        assert_eq!(selected(&harness).as_deref(), Some(expected), "Cmd+Shift+[ on {os:?}");
        press(&mut harness, Modifiers::COMMAND | Modifiers::SHIFT, Key::CloseBracket);
        assert_eq!(selected(&harness).as_deref(), Some(BETA), "Cmd+Shift+] on {os:?}");
    }
}

#[test]
fn the_status_bar_shows_the_active_views_fields_and_switching_tabs_swaps_them() {
    let shows = |harness: &Harness<'_, Shell>, value: &str| texts(harness).iter().any(|text| text.ends_with(value));
    let mut harness = three_tabs();
    assert!(shows(&harness, "Beta result") && shows(&harness, "Beta field"));
    assert!(!shows(&harness, "Alpha result") && !shows(&harness, "Alpha field"));

    press(&mut harness, Modifiers::CTRL | Modifiers::SHIFT, Key::Tab);
    assert!(shows(&harness, "Alpha result") && shows(&harness, "Alpha field"));
    assert!(!shows(&harness, "Beta result") && !shows(&harness, "Beta field"));

    press(&mut harness, Modifiers::CTRL | Modifiers::SHIFT, Key::Tab);
    assert!(texts(&harness).iter().all(|text| !text.contains("result") && !text.contains("field")), "Home has no fields");
}

#[test]
fn each_status_field_is_named_by_its_label_and_only_the_result_is_announced() {
    let harness = three_tabs();
    let field = |label: &str| {
        let node =
            harness.get_by(|node| node.role() == Role::Label && node.label().or_else(|| node.value()).is_some_and(|text| text.starts_with(label)));
        node.accesskit_node().live()
    };
    assert_eq!(field("Result"), Live::Polite);
    assert_eq!(field("Own"), Live::Off);
    assert_eq!(field(otd_ui::LOAD_TIME), Live::Off);
}

#[test]
fn shortcuts_with_extra_alt_or_shift_do_nothing() {
    let mut harness = three_tabs();
    let before = tabs(&harness);
    for modifiers in [Modifiers::COMMAND | Modifiers::ALT, Modifiers::COMMAND | Modifiers::SHIFT] {
        press(&mut harness, modifiers, Key::W);
        press(&mut harness, modifiers, Key::T);
    }
    press(&mut harness, Modifiers::CTRL | Modifiers::ALT, Key::Tab);
    assert_eq!(tabs(&harness), before);
}

#[test]
fn tab_reaches_the_insert_field_after_the_views_contents_and_activating_it_toggles_the_view() {
    let mut harness = harness();
    harness.state_mut().open(Box::new(EditView { insert: true }));
    harness.run();
    let mut order = Vec::new();
    for _ in 0..20 {
        press(&mut harness, Modifiers::NONE, Key::Tab);
        let (_, name) = focused(&harness).unwrap_or_default();
        if name == otd_ui::INSERT {
            break;
        }
        order.push(name);
    }
    assert!(order.iter().any(|name| name == EDIT_CONTROL), "the view's control comes before the bar: {order:?}");

    let control = harness.get_by_label(EDIT_CONTROL).rect();
    let field = harness.get_by_label(otd_ui::INSERT).rect();
    let window = harness.ctx.content_rect();
    assert!(window.contains_rect(field), "the bar is inside the window: {field:?} in {window:?}");
    assert!(field.top() >= control.bottom(), "the bar is below the contents");

    press(&mut harness, Modifiers::NONE, Key::Enter);
    assert!(harness.query_by_label(otd_ui::OVERWRITE).is_some(), "the view toggled to overwrite");
}

#[test]
fn the_tab_strip_is_a_tab_list() {
    let harness = harness();
    let tab = harness.get_by_role(Role::Tab);
    assert_eq!(tab.accesskit_node().parent().map(|p| p.role()), Some(Role::TabList));
}

fn names(harness: &Harness<'_, Shell>) -> Vec<String> {
    tabs(harness).into_iter().map(|(name, _)| name).collect()
}

#[test]
fn a_close_button_closes_its_own_tab_and_keeps_the_selected_one() {
    let mut harness = three_tabs();
    tab_to(&mut harness, Role::Button, &otd_ui::CLOSE_TAB.replace("{tab}", HOME));
    press(&mut harness, Modifiers::NONE, Key::Enter);
    assert_eq!(names(&harness), [ALPHA, BETA]);
    assert_eq!(selected(&harness).as_deref(), Some(BETA), "closing another tab keeps the selected one");
    assert!(focus_is_on_selected_tab(&harness));

    tab_to(&mut harness, Role::Button, &otd_ui::CLOSE_TAB.replace("{tab}", BETA));
    press(&mut harness, Modifiers::NONE, Key::Enter);
    assert_eq!(names(&harness), [ALPHA]);
    assert!(focus_is_on_selected_tab(&harness));
}

#[test]
fn the_new_tab_button_opens_home_in_a_new_tab_that_takes_focus() {
    let mut harness = three_tabs();
    tab_to(&mut harness, Role::Button, otd_ui::NEW_TAB);
    press(&mut harness, Modifiers::NONE, Key::Enter);
    assert_eq!(names(&harness), [HOME, ALPHA, BETA, HOME]);
    assert_eq!(tabs(&harness).last(), Some(&(HOME.to_owned(), true)));
    assert!(focus_is_on_selected_tab(&harness));
}

#[test]
fn ctrl_f4_closes_the_tab_on_windows_only() {
    for (os, closes) in [(OperatingSystem::Windows, true), (OperatingSystem::Mac, false), (OperatingSystem::Nix, false)] {
        let mut harness = harness_on(os);
        harness.state_mut().open((session_types()[0].new)());
        harness.run();
        press(&mut harness, Modifiers::CTRL, Key::F4);
        assert_eq!(tabs(&harness).len(), if closes { 1 } else { 2 }, "Ctrl+F4 on {os:?}");
    }
}

#[test]
fn enter_or_space_on_a_focused_tab_selects_it() {
    for key in [Key::Enter, Key::Space] {
        let mut harness = three_tabs();
        tab_to(&mut harness, Role::Tab, ALPHA);
        assert_eq!(selected(&harness).as_deref(), Some(BETA), "focus alone doesn't select");
        press(&mut harness, Modifiers::NONE, key);
        assert_eq!(selected(&harness).as_deref(), Some(ALPHA), "{key:?} selects the focused tab");
        assert!(focus_is_on_selected_tab(&harness));
    }
}
