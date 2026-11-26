//! Unsaved-changes prompts on Close Tab and Exit, driven by keyboard and read back through the
//! accessibility tree, with test views flagged as unsaved.

use std::cell::Cell;
use std::rc::Rc;

use egui::accesskit::{Live, Role};
use egui::os::OperatingSystem;
use egui::{Key, Modifiers, ViewportCommand, ViewportEvent, ViewportId};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_core::i18n::{fmt, t};
use otd_ui::{CANCEL, CLOSE_TAB, DONT_SAVE, HOME, SAVE, SAVE_CHANGES, Shell, View};

const DISK_FULL: &str = "The disk is full.";
/// A control inside each test view.
const EDIT: &str = "Edit";

/// A test document. `saves` counts the saves that worked.
struct Doc {
    title: &'static str,
    unsaved: bool,
    save_fails: bool,
    saves: Rc<Cell<u32>>,
}

impl View for Doc {
    fn title(&self) -> String {
        self.title.to_owned()
    }

    fn unsaved(&self) -> bool {
        self.unsaved
    }

    fn save(&mut self) -> Result<(), String> {
        if self.save_fails {
            return Err(DISK_FULL.to_owned());
        }
        self.unsaved = false;
        self.saves.set(self.saves.get() + 1);
        Ok(())
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let _ = ui.button(EDIT);
    }
}

fn doc(title: &'static str, unsaved: bool) -> (Box<dyn View>, Rc<Cell<u32>>) {
    let saves = Rc::default();
    (Box::new(Doc { title, unsaved, save_fails: false, saves: Rc::clone(&saves) }), saves)
}

/// The window commands a test shell has sent so far.
#[derive(Default)]
struct Sent {
    close: Cell<bool>,
    cancel_close: Cell<bool>,
}

/// A shell with Home and `views`, run as the app runs it (`on_close_request`, then `ui`), and the
/// window commands it sends.
fn watched_harness_on(os: OperatingSystem, views: Vec<Box<dyn View>>) -> (Harness<'static, Shell>, Rc<Sent>) {
    let sent = Rc::new(Sent::default());
    let watch = Rc::clone(&sent);
    let mut harness = Harness::builder().with_os(os).build_ui_state(
        move |ui, shell: &mut Shell| {
            shell.on_close_request(ui.ctx());
            shell.ui(ui);
            ui.ctx().viewport(|viewport| {
                watch.close.set(watch.close.get() || viewport.commands.contains(&ViewportCommand::Close));
                watch.cancel_close.set(watch.cancel_close.get() || viewport.commands.contains(&ViewportCommand::CancelClose));
            });
        },
        Shell::new(Vec::new()),
    );
    for view in views {
        harness.state_mut().open(view);
    }
    harness.run();
    (harness, sent)
}

/// The window is asked to close: its close button, the OS, Alt+F4, or Quit in the macOS menu.
fn close_window(harness: &mut Harness<'_, Shell>) {
    harness.input_mut().viewports.entry(ViewportId::ROOT).or_default().events.push(ViewportEvent::Close);
    harness.run();
}

fn harness_on(os: OperatingSystem, views: Vec<Box<dyn View>>) -> Harness<'static, Shell> {
    watched_harness_on(os, views).0
}

/// The prompt's name for the view titled `title`.
fn asks(title: &str) -> String {
    fmt(t(SAVE_CHANGES), &[("title", title)])
}

/// Tabs Home and an unsaved "Doc", Doc selected, and Doc's save counter.
fn one_unsaved() -> (Harness<'static, Shell>, Rc<Cell<u32>>) {
    let (view, saves) = doc("Doc", true);
    (harness_on(OperatingSystem::Mac, vec![view]), saves)
}

fn press(harness: &mut Harness<'_, Shell>, modifiers: Modifiers, key: Key) {
    harness.key_press_modifiers(modifiers, key);
    harness.run();
}

fn tab_names(harness: &Harness<'_, Shell>) -> Vec<String> {
    harness.query_all_by_role(Role::Tab).map(|tab| tab.accesskit_node().label().unwrap_or_default()).collect()
}

fn selected(harness: &Harness<'_, Shell>) -> Option<String> {
    harness.query_by(|node| node.role() == Role::Tab && node.is_selected() == Some(true)).and_then(|tab| tab.accesskit_node().label())
}

/// The open prompt's name, if a prompt is open.
fn prompt(harness: &Harness<'_, Shell>) -> Option<String> {
    harness.query_by_role(Role::AlertDialog).map(|dialog| dialog.accesskit_node().label().unwrap_or_default())
}

fn focused(harness: &Harness<'_, Shell>) -> Option<(Role, String)> {
    let node = harness.query_by(|node| node.is_focused())?;
    let node = node.accesskit_node();
    Some((node.role(), node.label().unwrap_or_default()))
}

fn focus_is_on_selected_tab(harness: &Harness<'_, Shell>) -> bool {
    focused(harness).is_some_and(|(role, name)| role == Role::Tab && Some(name) == selected(harness))
}

fn tab_to(harness: &mut Harness<'_, Shell>, name: &str) {
    for _ in 0..20 {
        if focused(harness).is_some_and(|(role, label)| role == Role::Button && label == name) {
            return;
        }
        press(harness, Modifiers::NONE, Key::Tab);
    }
    panic!("Tab never reached {name:?}");
}

/// Answers the open prompt with the button named `answer`.
fn answer(harness: &mut Harness<'_, Shell>, answer: &str) {
    tab_to(harness, answer);
    press(harness, Modifiers::NONE, Key::Enter);
}

#[test]
fn close_tab_on_an_unsaved_view_asks_save_dont_save_or_cancel_with_focus_on_save() {
    let (mut harness, _) = one_unsaved();
    press(&mut harness, Modifiers::COMMAND, Key::W);
    assert_eq!(prompt(&harness), Some(asks("Doc")));
    for button in [SAVE, DONT_SAVE, CANCEL] {
        assert!(harness.query_by(|node| node.role() == Role::Button && node.label().as_deref() == Some(button)).is_some(), "{button}");
    }
    assert_eq!(focused(&harness), Some((Role::Button, SAVE.to_owned())), "focus moves into the dialog, on its default button");
    assert_eq!(tab_names(&harness), [HOME, "Doc"], "nothing closes before the answer");
}

#[test]
fn a_clean_view_closes_without_asking() {
    let (view, _) = doc("Doc", false);
    let mut harness = harness_on(OperatingSystem::Mac, vec![view]);
    press(&mut harness, Modifiers::COMMAND, Key::W);
    assert_eq!(prompt(&harness), None);
    assert_eq!(tab_names(&harness), [HOME]);
}

#[test]
fn cancel_or_esc_keeps_the_view_open_unsaved_and_gives_focus_back_where_it_was() {
    for cancel in [Some(CANCEL), None] {
        let (mut harness, saves) = one_unsaved();
        tab_to(&mut harness, EDIT);
        press(&mut harness, Modifiers::COMMAND, Key::W);
        match cancel {
            Some(button) => answer(&mut harness, button),
            None => press(&mut harness, Modifiers::NONE, Key::Escape),
        }
        assert_eq!(prompt(&harness), None, "{cancel:?} closes the prompt");
        assert_eq!(tab_names(&harness), [HOME, "Doc"], "{cancel:?} keeps the view");
        assert_eq!(saves.get(), 0);
        assert_eq!(focused(&harness), Some((Role::Button, EDIT.to_owned())), "{cancel:?} gives focus back to the view's control");

        press(&mut harness, Modifiers::COMMAND, Key::W);
        assert!(prompt(&harness).is_some(), "the view still has its changes, so closing it asks again");
    }
}

#[test]
fn dont_save_closes_the_view_without_saving() {
    let (mut harness, saves) = one_unsaved();
    press(&mut harness, Modifiers::COMMAND, Key::W);
    answer(&mut harness, DONT_SAVE);
    assert_eq!(prompt(&harness), None);
    assert_eq!(tab_names(&harness), [HOME]);
    assert_eq!(saves.get(), 0);
    assert!(focus_is_on_selected_tab(&harness));
}

#[test]
fn save_saves_the_view_then_closes_it() {
    let (mut harness, saves) = one_unsaved();
    press(&mut harness, Modifiers::COMMAND, Key::W);
    press(&mut harness, Modifiers::NONE, Key::Enter);
    assert_eq!(saves.get(), 1);
    assert_eq!(prompt(&harness), None);
    assert_eq!(tab_names(&harness), [HOME]);
    assert!(focus_is_on_selected_tab(&harness));
}

#[test]
fn a_failed_save_keeps_the_view_and_announces_why() {
    let saves = Rc::default();
    let view = Box::new(Doc { title: "Doc", unsaved: true, save_fails: true, saves });
    let mut harness = harness_on(OperatingSystem::Mac, vec![view]);
    press(&mut harness, Modifiers::COMMAND, Key::W);
    press(&mut harness, Modifiers::NONE, Key::Enter);
    assert!(prompt(&harness).is_some(), "the prompt stays open");
    assert_eq!(tab_names(&harness), [HOME, "Doc"]);
    let error = harness.get_by(|node| node.role() == Role::Label && node.label().or_else(|| node.value()).is_some_and(|text| text == DISK_FULL));
    assert_ne!(error.accesskit_node().live(), Live::Off, "the error is announced");
}

#[test]
fn the_close_button_of_an_unsaved_background_tab_asks_about_that_tab() {
    let (first, _) = doc("First", true);
    let (second, _) = doc("Second", false);
    let mut harness = harness_on(OperatingSystem::Mac, vec![first, second]);
    tab_to(&mut harness, &fmt(t(CLOSE_TAB), &[("tab", "First")]));
    press(&mut harness, Modifiers::NONE, Key::Enter);
    assert_eq!(prompt(&harness), Some(asks("First")));
    assert_eq!(selected(&harness).as_deref(), Some("First"), "the tab asked about is shown");
    answer(&mut harness, DONT_SAVE);
    assert_eq!(tab_names(&harness), [HOME, "Second"]);
}

#[test]
fn keys_under_the_prompt_act_on_the_dialog_only() {
    let (mut harness, _) = one_unsaved();
    press(&mut harness, Modifiers::COMMAND, Key::W);
    let before = tab_names(&harness);
    press(&mut harness, Modifiers::COMMAND, Key::T);
    press(&mut harness, Modifiers::COMMAND, Key::W);
    press(&mut harness, Modifiers::CTRL, Key::Tab);
    assert_eq!(tab_names(&harness), before, "tab shortcuts wait for the answer");
    assert_eq!(selected(&harness).as_deref(), Some("Doc"));
    for _ in 0..6 {
        press(&mut harness, Modifiers::NONE, Key::Tab);
        let (_, name) = focused(&harness).unwrap_or_default();
        assert!([SAVE, DONT_SAVE, CANCEL].contains(&name.as_str()), "Tab stays in the dialog, but reached {name:?}");
    }
    press(&mut harness, Modifiers::NONE, Key::Escape);
    assert_eq!(selected(&harness).as_deref(), Some("Doc"), "no Ctrl+Tab was kept for after the prompt");
}

#[test]
fn exit_asks_once_for_each_unsaved_view_then_closes_the_window() {
    let (first, first_saves) = doc("First", true);
    let (clean, _) = doc("Clean", false);
    let (third, third_saves) = doc("Third", true);
    let (mut harness, sent) = watched_harness_on(OperatingSystem::Mac, vec![first, clean, third]);
    assert!(!harness.state_mut().exit(), "unsaved views hold the window open");
    harness.run();
    let mut asked = Vec::new();
    while let Some(name) = prompt(&harness) {
        assert!(asked.len() < 5, "the prompts never end: {asked:?}");
        assert!(name.contains(selected(&harness).unwrap_or_default().as_str()), "the view asked about is shown");
        let question = harness.get_by(|node| node.role() == Role::Label && node.label().or_else(|| node.value()).is_some_and(|text| text == name));
        assert_ne!(question.accesskit_node().live(), Live::Off, "each view's question is announced, though focus stays on Save");
        asked.push(name);
        assert!(!harness.state_mut().exit(), "asking to exit again doesn't start over");
        assert!(!sent.close.get(), "the window stays open while a view is unanswered");
        press(&mut harness, Modifiers::NONE, Key::Enter);
    }
    assert_eq!(asked, [asks("First"), asks("Third")]);
    assert_eq!((first_saves.get(), third_saves.get()), (1, 1));
    assert!(sent.close.get(), "the last answer asks the window to close again");
    assert!(harness.state_mut().exit(), "every view was answered for, so the window may close");
}

#[test]
fn cancel_during_exit_keeps_every_view_and_the_window() {
    let (first, first_saves) = doc("First", true);
    let (second, _) = doc("Second", true);
    let (mut harness, sent) = watched_harness_on(OperatingSystem::Mac, vec![first, second]);
    assert!(!harness.state_mut().exit());
    harness.run();
    answer(&mut harness, DONT_SAVE);
    assert_eq!(prompt(&harness), Some(asks("Second")));
    press(&mut harness, Modifiers::NONE, Key::Escape);
    assert_eq!(prompt(&harness), None);
    assert_eq!(tab_names(&harness), [HOME, "First", "Second"], "Don't Save discards nothing until the exit goes ahead");
    assert_eq!(first_saves.get(), 0);
    assert!(!sent.close.get());
    assert!(focus_is_on_selected_tab(&harness));
    assert!(!harness.state_mut().exit(), "the exit was cancelled, so a new one asks again");
}

#[test]
fn ctrl_q_exits_on_linux_only() {
    for (os, exits) in [(OperatingSystem::Nix, true), (OperatingSystem::Mac, false), (OperatingSystem::Windows, false)] {
        let (view, _) = doc("Doc", true);
        let (mut harness, sent) = watched_harness_on(os, vec![view]);
        press(&mut harness, Modifiers::CTRL, Key::Q);
        assert_eq!(prompt(&harness).is_some(), exits, "Ctrl+Q on {os:?} asks about the unsaved view");
        assert!(!sent.close.get(), "Ctrl+Q on {os:?} waits for the answer");

        let (view, _) = doc("Doc", false);
        let (mut harness, sent) = watched_harness_on(os, vec![view]);
        press(&mut harness, Modifiers::CTRL, Key::Q);
        assert_eq!(sent.close.get(), exits, "Ctrl+Q on {os:?} with nothing unsaved closes the window at once");
        assert_eq!(prompt(&harness), None);
    }
}

#[test]
fn closing_the_window_asks_about_unsaved_views_and_holds_it_open_until_answered() {
    let (first, first_saves) = doc("First", true);
    let (second, _) = doc("Second", true);
    let (mut harness, sent) = watched_harness_on(OperatingSystem::Windows, vec![first, second]);
    close_window(&mut harness);
    assert!(sent.cancel_close.get(), "the close waits for the answers");
    assert_eq!(prompt(&harness), Some(asks("First")));
    press(&mut harness, Modifiers::NONE, Key::Enter);
    answer(&mut harness, DONT_SAVE);
    assert_eq!(first_saves.get(), 1);
    assert!(sent.close.get(), "the last answer asks the window to close again");

    sent.cancel_close.set(false);
    // The window's own close, as that `Close` brings it.
    close_window(&mut harness);
    assert!(!sent.cancel_close.get(), "every view was answered for, so this close goes ahead");
}

#[test]
fn closing_the_window_with_nothing_unsaved_goes_ahead() {
    let (view, _) = doc("Doc", false);
    let (mut harness, sent) = watched_harness_on(OperatingSystem::Windows, vec![view]);
    close_window(&mut harness);
    assert!(!sent.cancel_close.get());
    assert_eq!(prompt(&harness), None);
}

/// Every item in `nodes`, at any depth.
fn all_items(nodes: &[craft_menu::Node]) -> Vec<&craft_menu::Item> {
    nodes
        .iter()
        .flat_map(|node| match node {
            craft_menu::Node::Item(item) => vec![item],
            craft_menu::Node::Submenu { children, .. } => all_items(children),
            _ => Vec::new(),
        })
        .collect()
}

#[test]
fn exit_is_the_one_quit_item_in_each_platforms_menus_and_asks_first() {
    for os in [OperatingSystem::Mac, OperatingSystem::Windows, OperatingSystem::Nix] {
        let (view, _) = doc("Doc", true);
        let (mut harness, sent) = watched_harness_on(os, vec![view]);
        let bar = harness.state().menu_layout(os).bar;
        let quits: Vec<_> =
            bar.menus.iter().flat_map(|menu| all_items(&menu.children)).filter(|item| item.role == Some(craft_menu::ItemRole::Quit)).collect();
        assert_eq!(quits.len(), 1, "one quit item on {os:?}");
        let id = quits[0].command.id.clone();
        assert!(harness.state_mut().run(&id), "the quit item on {os:?} is a registered command");
        harness.run();
        assert_eq!(prompt(&harness), Some(asks("Doc")), "the quit item on {os:?} asks about the unsaved view");
        assert!(!sent.close.get(), "and waits for the answer");
    }
}

#[test]
fn menu_commands_wait_under_the_prompt() {
    let (mut harness, _) = one_unsaved();
    press(&mut harness, Modifiers::COMMAND, Key::W);
    let before = tab_names(&harness);
    // A native menu click reaches `Shell::run` past the modal.
    for id in ["otd.new_tab", "otd.close_tab", "otd.previous_tab"] {
        assert!(!harness.state_mut().run(id), "{id} waits");
    }
    harness.run();
    assert_eq!(tab_names(&harness), before);
    assert_eq!(prompt(&harness), Some(asks("Doc")));
}
