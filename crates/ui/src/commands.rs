//! Registered commands in the window (E01-S1-T4): the menus, the toolbar and the shortcuts of the
//! active view type, all running commands by id through [`Shell::run`].
//!
//! A feature declares its [`Command`]s (otd-core) and adds them to the app's command list
//! (`apps/otd/src/commands.rs`), one line each. Its view reports its type ([`crate::View::view_type`]),
//! and its commands name the view types they apply to. Each frame the shell lays the menus out for
//! the platform with craft-menu (an in-window bar, or the native bar on macOS that the app drives
//! through [`Shell::menu_layout`]), draws a toolbar row of the commands that have a button, and
//! matches key presses against a craft-keymap keymap scoped by view type.

use craft_keymap::{Active, Bindable, Binding, CommandRef, Keymap, Lookup, Scope, Scopes};
use craft_menu::{BarStyle, Gate, ItemKind, ItemRole, Layout, LayoutOptions, MenuBarUi, MenuRole, Row, Target};
use egui::accesskit::Role;
use egui::os::OperatingSystem;
use egui::{Key as EguiKey, Modifiers, Ui};
use otd_core::command::{Chord, Command, CommandError, Key, Mark, Mods, Outcome, Placement, Platform, Views};

use otd_core::i18n::t;

use crate::{NEW_TAB, OPTIONS, SESSION_SETTINGS, SessionError, SessionType, Shell};

/// The application's name: the macOS application menu's title.
pub const APP_NAME: &str = "OverTheDiff";
/// The Close Tab command's caption (the tab strip's close buttons are named per tab).
pub const CLOSE_CURRENT_TAB: &str = "Close Tab";
pub const NEXT_TAB: &str = "Next Tab";
pub const PREVIOUS_TAB: &str = "Previous Tab";
/// The Exit command's caption. The menus show it as the platform's quit item: Exit on Windows,
/// Quit on Linux and Quit OverTheDiff in the macOS application menu (craft-menu's labels).
pub const EXIT: &str = "Exit";
/// Exit's id, the menus' quit item.
const EXIT_ID: &str = "otd.exit";
/// The Session menu's submenu with an entry per session type.
pub const NEW_SESSION: &str = "New Session";
pub const OPEN_SESSION: &str = "Open Session";
pub const CLEAR_SESSION: &str = "Clear Session";
const SESSION: &[&str] = &["Session"];

/// The top-level menus in bar order, with the role each has in the platform layouts. A command's
/// menu path starts with one of them; a menu not listed here follows them, in registration order.
const MENUS: [(&str, MenuRole); 8] = [
    ("Session", MenuRole::File),
    // A view's own files, such as Text Compare's Open File (E07-S1).
    ("File", MenuRole::Other),
    ("Edit", MenuRole::Edit),
    // A view's own search commands, such as Text Compare's Go To (E07-S6).
    ("Search", MenuRole::Other),
    ("View", MenuRole::View),
    ("Tools", MenuRole::Other),
    ("Window", MenuRole::Window),
    ("Help", MenuRole::Help),
];

/// A session command's result: a [`SessionError`] is a failure, not a success.
fn outcome(result: Result<(), SessionError>) -> Outcome {
    result.map(|()| None).map_err(|error| CommandError::Failed(error.to_string()))
}

/// The shell's own commands: the session commands, the tab commands of the keymap conventions
/// (Q24 in E01-S1) and Options, in Session menu order. They act on the window, so they need it.
/// New Session has an entry per type in `session_types`; Clear Session and Session Settings
/// apply to the views of those types.
pub(crate) fn shell_commands(session_types: &[SessionType]) -> Vec<Command<Shell>> {
    const MAC: &[Platform] = &[Platform::Mac];
    // ponytail: leaked once per shell, since command ids and view lists are `&'static`; a few bytes
    // per session type.
    let session_views: &'static [&'static str] = Box::leak(session_types.iter().map(|t| t.id).collect());
    let new_sessions = session_types.iter().map(|session_type| {
        let id: &'static str = Box::leak(format!("otd.new_session.{}", session_type.id).into_boxed_str());
        // A registered type, so it can't be unknown.
        Command::for_item(id, session_type.name, session_type.id, |shell: &mut Shell, type_id| {
            let _ = shell.new_session(type_id);
        })
        .icon("file-plus")
        .menu(&["Session", NEW_SESSION])
    });
    let several_tabs = |shell: &Shell| otd_core::command::State::enabled(shell.tabs.len() > 1);
    let commands = new_sessions.chain([
        Command::new("otd.new_tab", NEW_TAB, Shell::new_tab).icon("plus").menu(SESSION).shortcut(Mods::CMD, Key::T),
        Command::new("otd.open_session", OPEN_SESSION, Shell::open_session).icon("house").menu(SESSION).shortcut(Mods::CMD, Key::O),
        Command::typed("otd.clear_session", CLEAR_SESSION, &[], |shell: &mut Shell, _| outcome(shell.clear_session()))
            .views(session_views)
            .icon("eraser")
            .menu(SESSION),
        // The Session menu's Session Settings and the toolbar's Rules button: both named by the
        // dialog they open (docs/feature-names.md).
        Command::typed("otd.session_settings", SESSION_SETTINGS, &[], |shell: &mut Shell, _| outcome(shell.session_settings()))
            .views(session_views)
            .icon("sliders-horizontal")
            .menu(SESSION)
            .toolbar(),
        Command::new("otd.close_tab", CLOSE_CURRENT_TAB, Shell::close_tab).icon("x").menu(SESSION).shortcut(Mods::CMD, Key::W).shortcut_on(
            &[Platform::Windows],
            Mods::CTRL,
            Key::F4,
        ),
        Command::new("otd.next_tab", NEXT_TAB, Shell::next_tab).menu(&["Window"]).state(several_tabs).shortcut(Mods::CTRL, Key::Tab).shortcut_on(
            MAC,
            Mods { shift: true, ..Mods::CMD },
            Key::CloseBracket,
        ),
        Command::new("otd.previous_tab", PREVIOUS_TAB, Shell::previous_tab)
            .menu(&["Window"])
            .state(several_tabs)
            .shortcut(Mods { shift: true, ..Mods::CTRL }, Key::Tab)
            .shortcut_on(MAC, Mods { shift: true, ..Mods::CMD }, Key::OpenBracket),
        // Q24: Ctrl+Q on Linux; Cmd+Q is the macOS application menu's (craft-menu gives the quit
        // item it), and Windows has none: Alt+F4 closes the window, which exits the same way.
        Command::new(EXIT_ID, EXIT, Shell::request_exit).menu(SESSION).shortcut_on(&[Platform::Linux], Mods::CMD, Key::Q),
        // Q24: Cmd+, in the macOS application menu; no default elsewhere.
        Command::new("otd.options", OPTIONS, Shell::open_options).application_menu(&["Tools"]).shortcut_on(MAC, Mods::CMD, Key::Comma),
    ]);
    commands.map(Command::needs_window).collect()
}

/// The default keymap of `commands` on `platform`. A command's shortcuts work in each of its
/// view types, and an application-wide command's in all of them; the OS's reserved chords are
/// never bound.
// ponytail: built every frame, which is cheap for a few hundred commands; keep one once E22-S1's
// user overrides need it kept.
pub fn keymap(commands: &[Command<Shell>], platform: Platform) -> Keymap {
    let catalog = commands
        .iter()
        .map(|command| {
            let scopes: Vec<Scope> = match command.views {
                Views::All => vec![Scope::app()],
                Views::Only(types) => types.iter().filter_map(|t| Scope::new(*t)).collect(),
            };
            let home = scopes.first().cloned().unwrap_or_else(Scope::app);
            let mut bindable = Bindable::new(command.id, command.caption, home);
            for shortcut in command.shortcuts.iter().filter(|s| s.applies_on(platform)) {
                for scope in &scopes {
                    bindable = bindable.with_default(Binding::in_scope(shortcut.chord, scope.clone()));
                }
            }
            bindable
        })
        .collect();
    // Without fallthrough, another view type's shortcut never runs in this one.
    let scopes = Scopes::new().with_panel_fallthrough(false);
    Keymap::new(catalog, scopes, platform).with_reserved(craft_keymap::reserved::system(platform))
}

/// craft-menu's own English labels (Exit, Window, ⋯), through the lookup.
fn translate(english: &'static str) -> String {
    t(english).to_owned()
}

/// The menu layout of the platform `os` stands for.
fn target(os: OperatingSystem) -> Target {
    match os {
        OperatingSystem::Mac | OperatingSystem::IOS => Target::Mac,
        OperatingSystem::Windows => Target::Windows,
        _ => Target::Linux,
    }
}

impl Shell {
    /// Adds a feature's commands. A command whose id is already registered is left out.
    pub fn with_commands(mut self, commands: Vec<Command<Shell>>) -> Self {
        for command in commands {
            if !self.commands.iter().any(|c| c.id == command.id) {
                self.commands.push(command);
            }
        }
        self
    }

    /// Every registered command, the shell's own first.
    pub fn commands(&self) -> &[Command<Shell>] {
        &self.commands
    }

    /// The menus have a native bar (macOS), so the window draws no menu bar of its own.
    pub fn hide_in_window_menu(&mut self) {
        self.in_window_menu = false;
    }

    /// Runs the command `id`, with no arguments, if it applies to the active view type and is
    /// enabled, and no shell dialog (Session Settings, Options or an unsaved-changes prompt) is open
    /// (a native menu click reaches here past them). Returns whether it ran without an error.
    pub fn run(&mut self, id: &str) -> bool {
        if self.dialog_open() {
            return false;
        }
        let view = self.view_type();
        let Some(command) = self.commands.iter().find(|c| c.id == id && c.applies_to(view)) else { return false };
        // The window is open: the shell is what it draws.
        match command.bind(self, &[], true) {
            Ok(call) => call.run(self).is_ok(),
            Err(_) => false,
        }
    }

    fn enabled(&self, id: &str) -> bool {
        let view = self.view_type();
        self.commands.iter().find(|c| c.id == id && c.applies_to(view)).is_some_and(|c| (c.state)(self).enabled)
    }

    /// The menus of the active view type, laid out for the platform `os` stands for: what the
    /// in-window bar draws, and what the app hands the native bar on macOS.
    pub fn menu_layout(&self, os: OperatingSystem) -> Layout {
        self.layout_with(&keymap(&self.commands, Platform::from(os)), target(os))
    }

    fn layout_with(&self, keymap: &Keymap, target: Target) -> Layout {
        let mac = target == Target::Mac;
        // Menu titles, captions and the layout's own labels are English keys, shown through `t`.
        let mut tops: Vec<&str> = mac.then_some(APP_NAME).into_iter().chain(MENUS.iter().map(|(title, _)| t(title))).collect();
        let mut rows = Vec::new();
        let view = self.view_type();
        let mut commands: Vec<_> = self.commands.iter().filter(|c| c.applies_to(view)).collect();
        // Application-level commands go last in their menu, where Windows and Linux apps put
        // Tools > Options.
        commands.sort_by_key(|c| matches!(c.placement, Placement::Application(_)));
        for command in commands {
            let path: &[&str] = match command.placement {
                Placement::Hidden => continue,
                Placement::Application(_) if mac => &[APP_NAME],
                Placement::Menu(path) | Placement::Application(path) => path,
            };
            let Some(top) = path.first().map(|top| if *top == APP_NAME { APP_NAME } else { t(top) }) else { continue };
            if !tops.contains(&top) {
                tops.push(top);
            }
            let state = (command.state)(self);
            rows.push(Row {
                path: path.iter().map(|part| if *part == APP_NAME { APP_NAME } else { t(part) }.to_owned()).collect(),
                command: CommandRef::new(command.id),
                label: state.caption.unwrap_or_else(|| t(command.caption).to_owned()),
                shortcut: None,
                enabled: state.enabled,
                kind: match state.mark {
                    None => ItemKind::Plain,
                    Some(Mark::Check(on)) => ItemKind::Check(on),
                    Some(Mark::Radio(on)) => ItemKind::Radio(on),
                },
            });
        }
        let mut bar = craft_menu::from_items(&tops, &rows);
        bar.tag_menu(APP_NAME, MenuRole::App);
        // Exit is the platform's quit item, in place of craft-menu's own.
        bar.tag_item(&CommandRef::new(EXIT_ID), ItemRole::Quit);
        for (title, role) in MENUS {
            bar.tag_menu(t(title), role);
        }
        craft_menu::apply_keymap(&mut bar, keymap);
        craft_menu::layout(&bar, target, &LayoutOptions { label: &translate, ..LayoutOptions::new(APP_NAME) })
    }

    /// Runs the commands this frame's key presses stand for, before any widget reads the keys.
    /// While the in-window menu bar has the keyboard, the keys are the bar's.
    pub(crate) fn shortcuts(&mut self, ui: &Ui, keymap: &Keymap) {
        let ctx = ui.ctx().clone();
        // Idempotent: egui adds a plugin type once.
        ctx.add_plugin(CtrlTab);
        let ctrl_tabs = ctx.data_mut(|data| data.remove_temp::<Vec<Modifiers>>(egui::Id::unique(CTRL_TAB))).unwrap_or_default();
        // A modal dialog (last frame's top modal layer) keeps the window behind it out of reach, and so
        // does a shell dialog (Session Settings, Options or an unsaved-changes prompt) from the frame
        // it opens. Ctrl+Tab presses are dropped, not kept for after it.
        if self.dialog_open() || ctx.memory(|memory| memory.top_modal_layer().is_some()) {
            return;
        }
        craft_menu::bar::leave_for_shortcut(&ctx);
        if craft_menu::bar::is_open(&ctx) || craft_menu::bar::is_focused(&ctx) {
            return;
        }
        let platform = keymap.platform();
        let view = self.view_type();
        let active = Active { panel: Some(view), contexts: &[], typing: ctx.text_edit_focused() };
        let mut ids: Vec<String> = ctrl_tabs
            .iter()
            .filter_map(|held| match keymap.lookup(&Chord::new(Mods::from_egui(held, platform), Key::Tab), false, &active) {
                Lookup::Run(command) => Some(command.id),
                _ => None,
            })
            .collect();
        let mut dispatcher = std::mem::take(&mut self.dispatcher);
        let hits = ctx.input_mut(|input| dispatcher.dispatch_filtered(input, keymap, &active, |hit| self.enabled(&hit.command.id)));
        self.dispatcher = dispatcher;
        ids.extend(hits.into_iter().map(|hit| hit.command.id));
        for id in ids {
            self.run(&id);
        }
    }

    /// The in-window menu bar, unless the app shows a native one.
    pub(crate) fn menu_bar(&mut self, ui: &mut Ui, keymap: &Keymap) {
        if !self.in_window_menu {
            return;
        }
        let target = target(ui.ctx().os());
        let layout = self.layout_with(keymap, target);
        let gate = Gate { text_editing: ui.ctx().text_edit_focused(), ..Gate::default() };
        // ponytail: marks egui's default fonts have (they lack ✓ and ●), until the app has fonts of
        // its own.
        let style = BarStyle { check_mark: "√".to_owned(), radio_mark: "•".to_owned(), ..BarStyle::from_style(ui.style()) };
        let chosen = egui::Panel::top("otd-menu-bar")
            .show(ui, |ui| MenuBarUi::new(&layout.bar).target(target).gate(gate).style(style).label(&translate).show(ui))
            .inner;
        for command in chosen {
            self.run(&command.id);
        }
    }

    /// The toolbar of the active view type: a button per command that has one, in registration
    /// order. No row when there are none.
    pub(crate) fn toolbar(&mut self, ui: &mut Ui, keymap: &Keymap) {
        let view = self.view_type();
        let platform = keymap.platform();
        let buttons: Vec<_> = self
            .commands
            .iter()
            .filter(|c| c.toolbar && c.applies_to(view))
            .map(|c| {
                let state = (c.state)(self);
                let shortcut = keymap.bindings(&CommandRef::new(c.id)).first().map(|b| b.trigger.display(platform));
                (c.id, state.caption.unwrap_or_else(|| t(c.caption).to_owned()), state.icon.or(c.icon), state.enabled, shortcut)
            })
            .collect();
        if buttons.is_empty() {
            return;
        }
        let mut chosen = None;
        egui::Panel::top("otd-toolbar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.ctx().accesskit_node_builder(ui.unique_id(), |node| node.set_role(Role::Toolbar));
                for (id, caption, icon, enabled, shortcut) in &buttons {
                    let response = crate::tool_button(ui, caption, icon.and_then(crate::lucide), *enabled);
                    if let Some(shortcut) = shortcut {
                        ui.ctx().accesskit_node_builder(response.id, |node| node.set_keyboard_shortcut(shortcut.clone()));
                    }
                    if response.clicked() {
                        chosen = Some(*id);
                    }
                }
            });
        });
        if let Some(id) = chosen {
            self.run(id);
        }
    }
}

const CTRL_TAB: &str = "otd-ctrl-tab";

/// Takes Ctrl+Tab and Ctrl+Shift+Tab out of the input before egui's focus handling sees them, and
/// leaves their modifiers for [`Shell::shortcuts`]. egui counts Ctrl+Shift+Tab as Shift+Tab on
/// macOS (`Modifiers::shift_only` ignores Ctrl), so it would also move focus back one widget.
struct CtrlTab;

impl egui::Plugin for CtrlTab {
    fn debug_name(&self) -> &'static str {
        CTRL_TAB
    }

    fn input_hook(&mut self, ctx: &egui::Context, input: &mut egui::RawInput) {
        let mut presses = Vec::new();
        input.events.retain(|event| match event {
            egui::Event::Key { key: EguiKey::Tab, pressed, modifiers, .. } if modifiers.ctrl && !modifiers.alt => {
                if *pressed {
                    presses.push(*modifiers);
                }
                false
            }
            _ => true,
        });
        if !presses.is_empty() {
            ctx.data_mut(|data| data.insert_temp(egui::Id::unique(CTRL_TAB), presses));
        }
    }
}
