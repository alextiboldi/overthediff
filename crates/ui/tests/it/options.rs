//! The Options dialog, driven by keyboard and read back through the accessibility tree, with
//! options contributed by two test features and a test view that reads one of them.

use std::path::Path;

use egui::accesskit::Role;
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_settings::Location;
use otd_ui::{APPLY, CANCEL, CHOOSE_A_PAGE, OK, OPTIONS, OptionSpec, SessionType, Shell, Value, View};

const TEXT: &str = "Text";
const EDITING: &str = "Editing";
const DISPLAY: &str = "Display";
const WRAP: &str = "Wrap lines";
const TAB_WIDTH: &str = "Tab width";
const FONT: &str = "Font";
/// What the test view shows of the option it reads.
const SHOWS_WRAP: &str = "Wrapping";

/// A test view that shows the applied value of the first feature's option.
struct Reader;

impl View for Reader {
    fn title(&self) -> String {
        "Reader".to_owned()
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("{SHOWS_WRAP}: {:?}", otd_ui::option(ui.ctx(), "test.wrap")));
    }
}

fn option(page: &'static [&'static str], id: &'static str, label: &'static str, default: Value) -> OptionSpec {
    OptionSpec { page, id, label, default, choices: &[] }
}

/// Two test features: both contribute to Text > Editing, and the second also to Display. Nothing
/// contributes to Text itself.
fn options() -> Vec<OptionSpec> {
    let first_feature = vec![option(&[TEXT, EDITING], "test.wrap", WRAP, Value::Bool(false))];
    let second_feature = vec![
        option(&[TEXT, EDITING], "test.tab_width", TAB_WIDTH, Value::Number(4.0)),
        option(&[DISPLAY], "test.font", FONT, Value::Text("Mono".into())),
    ];
    [first_feature, second_feature].concat()
}

fn shell(saved: std::collections::BTreeMap<String, Value>) -> Shell {
    let reader = SessionType { id: "reader", name: "Reader", kinds: &[], sides: &[], new: || Box::new(Reader) };
    let mut shell = Shell::new(vec![reader]).with_options(options(), saved).unwrap();
    shell.new_session("reader").unwrap();
    shell
}

/// A harness on the reader view with the Options dialog opened by its command, as the menus and
/// its shortcut open it.
fn dialog(shell: Shell) -> Harness<'static, Shell> {
    let mut harness = Harness::builder().build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), shell);
    harness.run();
    assert!(harness.state_mut().run("otd.options"));
    harness.run();
    harness
}

fn press<S>(harness: &mut Harness<'_, S>, modifiers: Modifiers, key: Key) {
    harness.key_press_modifiers(modifiers, key);
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

fn is_open(harness: &Harness<'_, Shell>) -> bool {
    harness.query_by(|node| node.role() == Role::Dialog && node.label().as_deref() == Some(OPTIONS)).is_some()
}

/// The page tree's items, in order, with their level.
fn tree(harness: &Harness<'_, Shell>) -> Vec<(String, usize)> {
    harness
        .query_all_by_role(Role::TreeItem)
        .map(|item| item.accesskit_node())
        .map(|item| (item.label().unwrap_or_default(), item.level().unwrap_or(0)))
        .collect()
}

fn selected_page(harness: &Harness<'_, Shell>) -> Option<String> {
    harness
        .query_all_by_role(Role::TreeItem)
        .map(|item| item.accesskit_node())
        .find(|item| item.is_selected() == Some(true))
        .and_then(|item| item.label())
}

/// The applied wrap value, as the test view shows it.
fn wrapping(harness: &Harness<'_, Shell>) -> String {
    let start = format!("{SHOWS_WRAP}: ");
    let node = harness.get_by(|node| node.role() == Role::Label && node.label().or_else(|| node.value()).is_some_and(|t| t.starts_with(&start)));
    let node = node.accesskit_node();
    node.label().or_else(|| node.value()).unwrap_or_default()[start.len()..].to_owned()
}

fn wrap_checked(harness: &Harness<'_, Shell>) -> bool {
    let checkbox = harness.get_by(|node| node.role() == Role::CheckBox && node.label().as_deref() == Some(WRAP));
    checkbox.accesskit_node().toggled() == Some(egui::accesskit::Toggled::True)
}

fn toggle_wrap(harness: &mut Harness<'_, Shell>) {
    tab_to(harness, Role::CheckBox, WRAP);
    press(harness, Modifiers::NONE, Key::Space);
}

fn click(harness: &mut Harness<'_, Shell>, button: &str) {
    tab_to(harness, Role::Button, button);
    press(harness, Modifiers::NONE, Key::Enter);
}

#[test]
fn the_tree_shows_only_the_contributed_pages_and_those_above_them() {
    let harness = dialog(shell(Default::default()));
    let expected = [(TEXT.to_owned(), 1), (EDITING.to_owned(), 2), (DISPLAY.to_owned(), 1)];
    assert_eq!(tree(&harness), expected, "no page that nothing contributed to");
}

#[test]
fn the_dialog_opens_on_the_first_page_with_options_holding_every_features_options() {
    let harness = dialog(shell(Default::default()));
    assert_eq!(focused(&harness), Some((Role::TreeItem, EDITING.to_owned())), "Text has only a page under it");
    assert!(harness.query_by_label(WRAP).is_some(), "the first feature's option");
    assert!(harness.query_by_label(TAB_WIDTH).is_some(), "the second feature's option");
    assert!(harness.query_by_label(FONT).is_none(), "another page's option");
}

#[test]
fn up_and_down_move_the_selection_and_focus_through_the_tree() {
    let mut harness = dialog(shell(Default::default()));
    press(&mut harness, Modifiers::NONE, Key::ArrowDown);
    assert_eq!(focused(&harness), Some((Role::TreeItem, DISPLAY.to_owned())));
    assert_eq!(selected_page(&harness).as_deref(), Some(DISPLAY));
    assert!(harness.query_by_label(FONT).is_some() && harness.query_by_label(WRAP).is_none(), "Display's options replace Editing's");

    press(&mut harness, Modifiers::NONE, Key::ArrowDown);
    assert_eq!(focused(&harness), Some((Role::TreeItem, DISPLAY.to_owned())), "the last page stays");
    for _ in 0..3 {
        press(&mut harness, Modifiers::NONE, Key::ArrowUp);
    }
    assert_eq!(focused(&harness), Some((Role::TreeItem, TEXT.to_owned())), "the first page stays");
    let hint = otd_core::i18n::fmt(CHOOSE_A_PAGE, &[("page", TEXT)]);
    assert!(harness.query_by_label(&hint).is_some(), "a page with no options of its own points to its pages");
}

#[test]
fn apply_saves_without_closing_and_ok_saves_and_closes() {
    let mut harness = dialog(shell(Default::default()));
    toggle_wrap(&mut harness);
    assert_eq!(wrapping(&harness), "Some(Bool(false))", "nothing applies before Apply");
    click(&mut harness, APPLY);
    assert!(is_open(&harness), "Apply keeps the dialog open");
    assert_eq!(wrapping(&harness), "Some(Bool(true))");
    assert!(harness.state_mut().take_applied_options().is_some(), "Apply hands the values to the app to save");

    toggle_wrap(&mut harness);
    click(&mut harness, OK);
    assert!(!is_open(&harness), "OK closes the dialog");
    assert_eq!(wrapping(&harness), "Some(Bool(false))");
    assert!(harness.state_mut().take_applied_options().is_some(), "OK hands the values to the app to save");
}

#[test]
fn enter_in_a_single_line_option_is_ok() {
    let mut harness = dialog(shell(Default::default()));
    toggle_wrap(&mut harness);
    tab_to(&mut harness, Role::TreeItem, EDITING);
    press(&mut harness, Modifiers::NONE, Key::ArrowDown);
    tab_to(&mut harness, Role::TextInput, FONT);
    press(&mut harness, Modifiers::NONE, Key::Enter);
    assert!(!is_open(&harness));
    assert_eq!(wrapping(&harness), "Some(Bool(true))");
}

#[test]
fn cancel_and_esc_discard_the_changes_and_give_focus_back_to_the_window() {
    for close in [None, Some(CANCEL)] {
        let mut harness = dialog(shell(Default::default()));
        toggle_wrap(&mut harness);
        match close {
            None => press(&mut harness, Modifiers::NONE, Key::Escape),
            Some(cancel) => click(&mut harness, cancel),
        }
        assert!(!is_open(&harness), "{close:?} closes the dialog");
        assert_eq!(wrapping(&harness), "Some(Bool(false))", "{close:?} keeps the previous value");
        assert!(harness.state_mut().take_applied_options().is_none(), "{close:?} leaves nothing to save");
        assert_eq!(focused(&harness), Some((Role::Tab, "Reader".to_owned())), "{close:?}: focus on the window's tab");

        assert!(harness.state_mut().run("otd.options"));
        harness.run();
        assert!(!wrap_checked(&harness), "{close:?}: the dialog reopens on the previous value");
    }
}

#[test]
fn the_windows_commands_wait_while_the_dialog_is_open() {
    let mut harness = dialog(shell(Default::default()));
    press(&mut harness, Modifiers::COMMAND, Key::T);
    assert!(!harness.state_mut().run("otd.close_tab"), "a menu click runs nothing behind the dialog");
    harness.run();
    assert!(is_open(&harness));
    assert_eq!(harness.query_all_by_role(Role::Tab).count(), 2, "Home and the reader, and no new tab");
}

fn open_store(dir: &Path) -> otd_settings::Store {
    otd_settings::open(Location::Dir(dir.to_path_buf())).0
}

#[test]
fn an_applied_option_persists_across_a_restart() {
    let dir = tempfile::tempdir().unwrap();
    {
        // The app's start: the saved values, read for the contributed options.
        let mut store = open_store(dir.path());
        let saved = otd_settings::options::read(&mut store, &options());
        let mut harness = dialog(shell(saved));
        toggle_wrap(&mut harness);
        click(&mut harness, APPLY);
        // What the app does each frame, then on exit.
        let values = harness.state_mut().take_applied_options().unwrap();
        otd_settings::options::write(&mut store, &values);
        store.flush().unwrap();
    }
    let mut store = open_store(dir.path());
    let saved = otd_settings::options::read(&mut store, &options());
    let harness = dialog(shell(saved));
    assert_eq!(wrapping(&harness), "Some(Bool(true))", "the view reads the saved value");
    assert!(wrap_checked(&harness), "and the dialog shows it");
}
