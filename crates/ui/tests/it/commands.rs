//! Registered commands in the window: menus and toolbar per view type, disabled, hidden-by-default,
//! dynamic-caption and toggle commands, and application-level placement, with test views and
//! commands.

use std::cell::Cell;

use craft_menu::{ItemKind, MenuBar, Node};
use egui::accesskit::{Role, Toggled};
use egui::os::OperatingSystem;
use egui::{Key as EguiKey, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_ui::command::{Command, Key, Mark, Mods, State};
use otd_ui::{APP_NAME, SessionType, Shell, View};

const ALPHA: &str = "alpha";
const BETA: &str = "beta";
const ALPHA_ACTION: &str = "Alpha Action";
const BETA_ACTION: &str = "Beta Action";
const LOCKED: &str = "Locked Action";
const HIDDEN: &str = "Copy Across";
const SHOW: &str = "Show Details";
const HIDE: &str = "Hide Details";
const OPTIONS: &str = "Test Options";
const RULER: &str = "Ruler";

thread_local! {
    /// How many times a test command ran. Each test runs on a thread of its own.
    static RUNS: Cell<u32> = const { Cell::new(0) };
}

fn runs() -> u32 {
    RUNS.with(Cell::get)
}

fn record(_: &mut Shell) {
    RUNS.with(|runs| runs.set(runs.get() + 1));
}

struct TestView {
    view_type: &'static str,
    details: bool,
}

impl View for TestView {
    fn title(&self) -> String {
        self.view_type.to_owned()
    }

    fn view_type(&self) -> &'static str {
        self.view_type
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label(self.view_type);
    }
}

const EDITOR: &str = "editor";

/// A view with a text field.
struct EditorView {
    text: String,
}

impl View for EditorView {
    fn title(&self) -> String {
        EDITOR.to_owned()
    }

    fn view_type(&self) -> &'static str {
        EDITOR
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let label = ui.label("Name");
        ui.text_edit_singleline(&mut self.text).labelled_by(label.id);
    }
}

fn toggle_details(shell: &mut Shell) {
    if let Some(view) = shell.view_mut::<TestView>() {
        view.details = !view.details;
    }
}

fn commands() -> Vec<Command<Shell>> {
    vec![
        Command::new("test.alpha", ALPHA_ACTION, record).views(&[ALPHA]).menu(&["Tools"]).toolbar(),
        Command::new("test.beta", BETA_ACTION, record).views(&[BETA]).menu(&["Tools"]).toolbar(),
        Command::new("test.locked", LOCKED, record)
            .views(&[ALPHA])
            .menu(&["Tools"])
            .toolbar()
            .shortcut(Mods::CMD, Key::K)
            .state(|_| State::enabled(false)),
        // Hidden by default, like Copy to Right in the catalogue.
        Command::new("test.hidden", HIDDEN, record).views(&[ALPHA]).shortcut(Mods::CMD, Key::R),
        Command::new("test.details", SHOW, toggle_details).views(&[ALPHA]).icon("plus").menu(&["View"]).toolbar().state(|shell: &Shell| {
            let details = shell.view::<TestView>().is_some_and(|view| view.details);
            State { caption: details.then(|| HIDE.to_owned()), icon: details.then_some("x"), ..State::default() }
        }),
        // A toggle, like Word Wrap.
        Command::new("test.ruler", RULER, toggle_details)
            .views(&[ALPHA])
            .menu(&["View"])
            .toolbar()
            .state(|shell: &Shell| State { mark: Some(Mark::Check(shell.view::<TestView>().is_some_and(|view| view.details))), ..State::default() }),
        Command::new("test.options", OPTIONS, record).application_menu(&["Tools"]),
        // A plain-key shortcut, like Del in a manager list.
        Command::new("test.delete", "Delete Item", record).views(&[ALPHA, EDITOR]).shortcut(Mods::NONE, Key::Delete),
    ]
}

/// A shell on `os` with tabs Home, Alpha and Beta, Beta selected.
fn harness_on(os: OperatingSystem) -> Harness<'static, Shell> {
    let session_types =
        vec![SessionType { id: "unused", name: "Unused", kinds: &[], sides: &[], new: || Box::new(TestView { view_type: ALPHA, details: false }) }];
    let mut shell = Shell::new(session_types).with_commands(commands());
    for view_type in [ALPHA, BETA] {
        shell.open(Box::new(TestView { view_type, details: false }));
    }
    let mut harness = Harness::builder().with_os(os).build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), shell);
    harness.run();
    harness
}

fn press(harness: &mut Harness<'_, Shell>, modifiers: Modifiers, key: EguiKey) {
    harness.key_press_modifiers(modifiers, key);
    harness.run();
}

/// Ctrl+Shift+Tab: the tab before.
fn previous_tab(harness: &mut Harness<'_, Shell>) {
    press(harness, Modifiers::CTRL | Modifiers::SHIFT, EguiKey::Tab);
}

/// Every item of the menu titled `title`, at any depth: its label and whether it is enabled.
fn items(bar: &MenuBar, title: &str) -> Vec<(String, bool)> {
    fn walk(nodes: &[Node], out: &mut Vec<(String, bool)>) {
        for node in nodes {
            match node {
                Node::Item(item) => out.push((item.label.clone(), item.enabled)),
                Node::Submenu { children, .. } => walk(children, out),
                _ => {}
            }
        }
    }
    let mut out = Vec::new();
    for menu in bar.menus.iter().filter(|menu| menu.title == title) {
        walk(&menu.children, &mut out);
    }
    out
}

fn labels(bar: &MenuBar, title: &str) -> Vec<String> {
    items(bar, title).into_iter().map(|(label, _)| label).collect()
}

fn menus(harness: &Harness<'_, Shell>) -> MenuBar {
    harness.state().menu_layout(harness.ctx.os()).bar
}

/// The toolbar's buttons: name and whether they are disabled.
fn toolbar(harness: &Harness<'_, Shell>) -> Vec<(String, bool)> {
    harness
        .query_all_by(|node| node.role() == Role::Button && node.parent().is_some_and(|parent| parent.role() == Role::Toolbar))
        .map(|button| {
            let node = button.accesskit_node();
            (node.label().unwrap_or_default(), node.is_disabled())
        })
        .collect()
}

fn on_toolbar(harness: &Harness<'_, Shell>, name: &str) -> bool {
    toolbar(harness).iter().any(|(label, _)| label == name)
}

#[test]
fn switching_to_another_view_type_swaps_the_menus_and_the_toolbar() {
    let mut harness = harness_on(OperatingSystem::Windows);
    let tools = |harness: &Harness<'_, Shell>| labels(&menus(harness), "Tools");

    assert!(tools(&harness).contains(&BETA_ACTION.to_owned()) && !tools(&harness).contains(&ALPHA_ACTION.to_owned()));
    assert!(on_toolbar(&harness, BETA_ACTION) && !on_toolbar(&harness, ALPHA_ACTION));

    previous_tab(&mut harness);
    assert!(tools(&harness).contains(&ALPHA_ACTION.to_owned()) && !tools(&harness).contains(&BETA_ACTION.to_owned()));
    assert!(on_toolbar(&harness, ALPHA_ACTION) && !on_toolbar(&harness, BETA_ACTION));

    previous_tab(&mut harness);
    assert!(!tools(&harness).iter().any(|label| label == ALPHA_ACTION || label == BETA_ACTION), "Home has neither");
    assert!(toolbar(&harness).is_empty(), "nor a toolbar");
}

#[test]
fn a_disabled_command_is_greyed_out_and_doesnt_run() {
    let mut harness = harness_on(OperatingSystem::Windows);
    previous_tab(&mut harness);
    assert!(items(&menus(&harness), "Tools").contains(&(LOCKED.to_owned(), false)), "greyed in the menu");
    assert!(toolbar(&harness).contains(&(LOCKED.to_owned(), true)), "and on the toolbar");

    press(&mut harness, Modifiers::COMMAND, EguiKey::K);
    assert!(!harness.state_mut().run("test.locked"));
    assert_eq!(runs(), 0, "neither its shortcut nor its id runs it");
}

#[test]
fn a_hidden_by_default_command_has_no_menu_entry_and_runs_from_its_shortcut() {
    for os in [OperatingSystem::Mac, OperatingSystem::Windows, OperatingSystem::Nix] {
        let mut harness = harness_on(os);
        previous_tab(&mut harness);
        let bar = menus(&harness);
        assert!(bar.menus.iter().all(|menu| !labels(&bar, &menu.title).contains(&HIDDEN.to_owned())), "no menu entry on {os:?}");

        let before = runs();
        press(&mut harness, Modifiers::COMMAND, EguiKey::R);
        assert!(runs() > before, "its shortcut runs it on {os:?}");
    }
}

#[test]
fn a_view_types_shortcut_does_nothing_in_another_view_type() {
    let mut harness = harness_on(OperatingSystem::Windows);
    press(&mut harness, Modifiers::COMMAND, EguiKey::R);
    assert_eq!(runs(), 0, "Ctrl+R belongs to the alpha view type, not beta");
    previous_tab(&mut harness);
    previous_tab(&mut harness);
    press(&mut harness, Modifiers::COMMAND, EguiKey::R);
    assert_eq!(runs(), 0, "nor Home");
}

#[test]
fn a_dynamic_caption_and_icon_follow_the_commands_state_in_the_menu_and_on_the_toolbar() {
    let mut harness = harness_on(OperatingSystem::Windows);
    previous_tab(&mut harness);
    assert!(labels(&menus(&harness), "View").contains(&SHOW.to_owned()) && on_toolbar(&harness, SHOW));

    for _ in 0..20 {
        if harness.query_by(|node| node.is_focused()).is_some_and(|node| node.accesskit_node().label().as_deref() == Some(SHOW)) {
            break;
        }
        press(&mut harness, Modifiers::NONE, EguiKey::Tab);
    }
    press(&mut harness, Modifiers::NONE, EguiKey::Enter);
    assert!(labels(&menus(&harness), "View").contains(&HIDE.to_owned()), "the menu shows the new caption");
    assert!(on_toolbar(&harness, HIDE) && !on_toolbar(&harness, SHOW), "and so does the toolbar");
}

#[test]
fn a_toggles_state_shows_as_a_check_item_in_the_menu_and_a_toggled_toolbar_button() {
    let mut harness = harness_on(OperatingSystem::Windows);
    previous_tab(&mut harness);
    let state = |harness: &Harness<'_, Shell>| {
        let kind = menus(harness).menus.iter().filter(|menu| menu.title == "View").flat_map(|menu| &menu.children).find_map(|node| match node {
            Node::Item(item) if item.label == RULER => Some(item.kind),
            _ => None,
        });
        let button = harness.get_by(|node| node.role() == Role::Button && node.label().as_deref() == Some(RULER));
        (kind, button.accesskit_node().toggled())
    };
    assert_eq!(state(&harness), (Some(ItemKind::Check(false)), Some(Toggled::False)), "off");

    for _ in 0..20 {
        if harness.query_by(|node| node.is_focused()).is_some_and(|node| node.accesskit_node().label().as_deref() == Some(RULER)) {
            break;
        }
        press(&mut harness, Modifiers::NONE, EguiKey::Tab);
    }
    press(&mut harness, Modifiers::NONE, EguiKey::Enter);
    assert_eq!(state(&harness), (Some(ItemKind::Check(true)), Some(Toggled::True)), "on, after the button runs it from the keyboard");
}

#[test]
fn application_level_commands_sit_in_the_application_menu_on_macos_and_in_tools_elsewhere() {
    for (os, in_app_menu) in [(OperatingSystem::Mac, true), (OperatingSystem::Windows, false), (OperatingSystem::Nix, false)] {
        let bar = menus(&harness_on(os));
        let options = OPTIONS.to_owned();
        assert_eq!(labels(&bar, APP_NAME).contains(&options), in_app_menu, "application menu on {os:?}");
        assert_eq!(labels(&bar, "Tools").contains(&options), !in_app_menu, "Tools on {os:?}");
    }
}

#[test]
fn the_in_window_menu_bar_runs_a_command_from_the_keyboard() {
    let mut harness = harness_on(OperatingSystem::Windows);
    previous_tab(&mut harness);
    // Alt+T opens Tools; its first item is Alpha Action.
    press(&mut harness, Modifiers::ALT, EguiKey::T);
    assert!(harness.query_all_by_label(ALPHA_ACTION).any(|item| item.accesskit_node().role() == Role::MenuItem), "Tools is open");
    press(&mut harness, Modifiers::NONE, EguiKey::Enter);
    assert_eq!(runs(), 1);
}

#[test]
fn a_plain_key_shortcut_runs_while_a_control_has_focus_but_not_while_a_text_field_has_the_keys() {
    let mut harness = harness_on(OperatingSystem::Windows);
    previous_tab(&mut harness);
    assert!(harness.query_by(|node| node.is_focused()).is_some(), "the selected tab has focus");
    press(&mut harness, Modifiers::NONE, EguiKey::Delete);
    assert_eq!(runs(), 1, "Del runs with a tab focused");

    harness.state_mut().open(Box::new(EditorView { text: "abc".to_owned() }));
    harness.run();
    for _ in 0..20 {
        if harness.query_by(|node| node.is_focused()).is_some_and(|node| node.accesskit_node().role() == Role::TextInput) {
            break;
        }
        press(&mut harness, Modifiers::NONE, EguiKey::Tab);
    }
    press(&mut harness, Modifiers::NONE, EguiKey::Delete);
    assert_eq!(runs(), 1, "Del goes to the text field");
}
