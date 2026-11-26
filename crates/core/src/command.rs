//! Commands (E01-S1-T4): what an action is called, where the shell shows it and what runs it.
//!
//! A command runs on a context `C`, the main window's shell in the app. The shell shows each
//! command in the menus and toolbar of the view types it names ([`Views`]), binds its default
//! shortcuts ([`Shortcut`]) and asks its [`State`] every frame. Menus, the toolbar and shortcuts
//! all run a command by its id.

pub use craft_keymap::{Chord, Key, Mods, Platform};

/// The view type of the Home launcher tab, for [`Views::Only`].
pub const HOME_VIEW: &str = "home";

/// The view types a command applies to: its menu entry, toolbar button and shortcuts exist only
/// while a view of one of them is active.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Views {
    /// Every view type, Home included. Its shortcuts hold in every view type (Q24).
    All,
    /// These view types only, by the id each view reports.
    Only(&'static [&'static str]),
}

/// Where a command's menu entry goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Placement {
    /// No menu entry: the command runs from its shortcut or toolbar button (hidden by default).
    Hidden,
    /// The menu at this path, such as `["Session"]` or `["Session", "New Session"]`.
    Menu(&'static [&'static str]),
    /// An application-level command, such as Options: in the application menu on macOS, and in
    /// the menu at this path (Tools or Help) on Windows and Linux.
    Application(&'static [&'static str]),
}

/// A default shortcut, on every platform or only on some. The chord's `cmd` is Cmd on macOS and
/// Ctrl elsewhere; its `ctrl` is the Control key on macOS and Ctrl elsewhere (Q24).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shortcut {
    pub chord: Chord,
    /// The platforms it is a default on; empty for every platform.
    pub on: &'static [Platform],
}

impl Shortcut {
    /// Whether it is a default on `platform`.
    pub fn applies_on(&self, platform: Platform) -> bool {
        self.on.is_empty() || self.on.contains(&platform)
    }
}

/// A command's state for one frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// A disabled command is greyed out and doesn't run.
    pub enabled: bool,
    /// Replaces the command's caption while set (a dynamic caption), in the interface language:
    /// the command looks it up (`crate::i18n::t`).
    pub caption: Option<String>,
    /// Replaces the command's icon while set.
    pub icon: Option<&'static str>,
}

impl State {
    pub fn enabled(enabled: bool) -> State {
        State { enabled, caption: None, icon: None }
    }
}

impl Default for State {
    fn default() -> State {
        State::enabled(true)
    }
}

/// A registered command acting on `C`.
pub struct Command<C> {
    /// Stable and unique, independent of the caption: `otd.new_tab`.
    pub id: &'static str,
    /// The English caption, also the translation key.
    pub caption: &'static str,
    /// A Lucide icon name, such as `plus`.
    pub icon: Option<&'static str>,
    pub views: Views,
    pub placement: Placement,
    /// Whether it has a button on the toolbar of its view types, in registration order.
    pub toolbar: bool,
    pub shortcuts: Vec<Shortcut>,
    /// Asked every frame the command is shown, and before it runs.
    pub state: fn(&C) -> State,
    pub run: fn(&mut C),
}

impl<C> Command<C> {
    /// An always-enabled command for every view type, with no menu entry, toolbar button or
    /// shortcut yet.
    pub fn new(id: &'static str, caption: &'static str, run: fn(&mut C)) -> Command<C> {
        Command {
            id,
            caption,
            icon: None,
            views: Views::All,
            placement: Placement::Hidden,
            toolbar: false,
            shortcuts: Vec::new(),
            state: |_| State::default(),
            run,
        }
    }

    pub fn icon(mut self, icon: &'static str) -> Command<C> {
        self.icon = Some(icon);
        self
    }

    pub fn views(mut self, views: &'static [&'static str]) -> Command<C> {
        self.views = Views::Only(views);
        self
    }

    pub fn menu(mut self, path: &'static [&'static str]) -> Command<C> {
        self.placement = Placement::Menu(path);
        self
    }

    /// See [`Placement::Application`].
    pub fn application_menu(mut self, path: &'static [&'static str]) -> Command<C> {
        self.placement = Placement::Application(path);
        self
    }

    pub fn toolbar(mut self) -> Command<C> {
        self.toolbar = true;
        self
    }

    /// A default shortcut on every platform.
    pub fn shortcut(mut self, mods: Mods, key: Key) -> Command<C> {
        self.shortcuts.push(Shortcut { chord: Chord::new(mods, key), on: &[] });
        self
    }

    /// A default shortcut on `platforms` only.
    pub fn shortcut_on(mut self, platforms: &'static [Platform], mods: Mods, key: Key) -> Command<C> {
        self.shortcuts.push(Shortcut { chord: Chord::new(mods, key), on: platforms });
        self
    }

    pub fn state(mut self, state: fn(&C) -> State) -> Command<C> {
        self.state = state;
        self
    }

    /// Whether the command applies while a view of `view_type` is active.
    pub fn applies_to(&self, view_type: &str) -> bool {
        match self.views {
            Views::All => true,
            Views::Only(types) => types.contains(&view_type),
        }
    }
}
