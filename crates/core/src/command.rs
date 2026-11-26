//! Commands (E01-S1-T4): what an action is called, where the shell shows it and what runs it.
//!
//! A command runs on a context `C`, the main window's shell in the app. The shell shows each
//! command in the menus and toolbar of the view types it names ([`Views`]), binds its default
//! shortcuts ([`Shortcut`]) and asks its [`State`] every frame. Menus, the toolbar and shortcuts
//! all run a command by its id.
//!
//! A command can also declare typed arguments ([`Param`]) and return a result (E01-S1-T12), so a
//! script or the automation interface (E28) runs it by id without the window through [`run`]. The
//! arguments are checked against the declaration before the command runs; a command that acts on
//! the window says so with [`Command::needs_window`] and is refused while none is open.

use std::collections::BTreeMap;
use std::fmt;

pub use craft_keymap::{Chord, Key, Mods, Platform};

/// An argument's or result's value; the same value types as settings.
pub use crate::session::Value;

use crate::i18n::t;

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
    /// Shows the menu entry as a check or radio item, in this state; `None` is a plain entry.
    pub mark: Option<Mark>,
}

/// A menu entry's check mark: whether a check item is on, or whether a radio item is the chosen
/// one of the radio items next to it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    Check(bool),
    Radio(bool),
}

impl State {
    pub fn enabled(enabled: bool) -> State {
        State { enabled, caption: None, icon: None, mark: None }
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
    /// The arguments it takes, checked before it runs.
    pub params: &'static [Param],
    /// It acts on the main window, so it is refused while none is open.
    pub needs_window: bool,
    pub run: Run<C>,
}

/// What a command runs.
pub enum Run<C> {
    /// Takes no arguments and returns no value.
    Plain(fn(&mut C)),
    /// Takes the checked arguments and returns a value, or an error such as
    /// [`CommandError::Failed`].
    Typed(fn(&mut C, &Args) -> Outcome),
    /// Like `Plain`, given the item it was made for: one command per registered item, such as
    /// New Session ▸ <type> for each session type.
    Item(fn(&mut C, &'static str), &'static str),
}

// Not derived: a derive would ask `C: Copy`.
impl<C> Clone for Run<C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<C> Copy for Run<C> {}

/// What running a command gives: its value, if it returns one, or why it didn't run or failed.
pub type Outcome = Result<Option<Value>, CommandError>;

/// An argument a command declares.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Param {
    /// Stable, unique within the command: `left_path`.
    pub name: &'static str,
    pub kind: Kind,
    pub required: bool,
}

impl Param {
    pub const fn required(name: &'static str, kind: Kind) -> Param {
        Param { name, kind, required: true }
    }

    pub const fn optional(name: &'static str, kind: Kind) -> Param {
        Param { name, kind, required: false }
    }
}

/// An argument's value type and the values it allows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Bool,
    /// A number from `min` to `max`, both included.
    Number {
        min: f64,
        max: f64,
    },
    /// One of these, in English.
    Choice(&'static [&'static str]),
    Text,
    Path,
    Colour,
}

impl Kind {
    /// Whether `value`, given for the argument `name`, has this type and is in range.
    fn check(&self, name: &'static str, value: &Value) -> Result<(), CommandError> {
        let in_range = match (self, value) {
            (Kind::Bool, Value::Bool(_)) | (Kind::Text, Value::Text(_)) | (Kind::Path, Value::Path(_)) | (Kind::Colour, Value::Colour(_)) => true,
            // Written so that NaN is out of range.
            (Kind::Number { min, max }, Value::Number(n)) => *min <= *n && *n <= *max,
            (Kind::Choice(choices), Value::Choice(choice)) => choices.contains(&choice.as_str()),
            _ => return Err(CommandError::WrongType(name)),
        };
        if in_range { Ok(()) } else { Err(CommandError::OutOfRange(name)) }
    }
}

/// A command's arguments, checked against its [`Param`]s: each is declared, of its type and in
/// range, and every required one is there.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Args(BTreeMap<&'static str, Value>);

impl Args {
    /// Checks `given` against `params`.
    pub fn check(params: &'static [Param], given: &[(&str, Value)]) -> Result<Args, CommandError> {
        let mut args = BTreeMap::new();
        for (name, value) in given {
            let Some(param) = params.iter().find(|p| p.name == *name) else {
                return Err(CommandError::UnknownArgument((*name).to_owned()));
            };
            param.kind.check(param.name, value)?;
            if args.insert(param.name, value.clone()).is_some() {
                return Err(CommandError::RepeatedArgument(param.name));
            }
        }
        match params.iter().find(|p| p.required && !args.contains_key(p.name)) {
            Some(missing) => Err(CommandError::MissingArgument(missing.name)),
            None => Ok(Args(args)),
        }
    }

    /// The argument `name`, if given.
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.0.get(name)
    }

    pub fn bool(&self, name: &str) -> Option<bool> {
        match self.get(name)? {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn number(&self, name: &str) -> Option<f64> {
        match self.get(name)? {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// A text, path or choice argument.
    pub fn text(&self, name: &str) -> Option<&str> {
        match self.get(name)? {
            Value::Text(s) | Value::Path(s) | Value::Choice(s) => Some(s),
            _ => None,
        }
    }
}

/// Why a command didn't run, or how it failed. Each argument error names the argument.
#[derive(Clone, Debug, PartialEq)]
pub enum CommandError {
    /// No registered command has this id.
    UnknownCommand(String),
    /// The command acts on the main window, and none is open.
    NeedsWindow(&'static str),
    /// The command is disabled in the current state.
    Disabled(&'static str),
    MissingArgument(&'static str),
    /// The command declares no argument with this name.
    UnknownArgument(String),
    RepeatedArgument(&'static str),
    WrongType(&'static str),
    /// A number outside its range, or a choice not among its choices.
    OutOfRange(&'static str),
    /// The command ran and failed, with its message in the interface language.
    Failed(String),
}

impl CommandError {
    /// The argument an argument error names.
    pub fn argument(&self) -> Option<&str> {
        match self {
            CommandError::UnknownArgument(name) => Some(name),
            CommandError::MissingArgument(name)
            | CommandError::RepeatedArgument(name)
            | CommandError::WrongType(name)
            | CommandError::OutOfRange(name) => Some(name),
            _ => None,
        }
    }
}

impl fmt::Display for CommandError {
    /// The message in the interface language.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (template, name): (&str, &str) = match self {
            CommandError::UnknownCommand(id) => ("There is no command {name}.", id),
            CommandError::NeedsWindow(id) => ("The command {name} needs the main window, and it isn't open.", id),
            CommandError::Disabled(id) => ("The command {name} can't run now.", id),
            CommandError::MissingArgument(name) => ("The argument {name} is missing.", name),
            CommandError::UnknownArgument(name) => ("There is no argument {name}.", name),
            CommandError::RepeatedArgument(name) => ("The argument {name} is given more than once.", name),
            CommandError::WrongType(name) => ("The argument {name} has the wrong type.", name),
            CommandError::OutOfRange(name) => ("The argument {name} is out of range.", name),
            CommandError::Failed(message) => return f.write_str(message),
        };
        f.write_str(&crate::i18n::fmt(t(template), &[("name", name)]))
    }
}

impl std::error::Error for CommandError {}

/// Runs the command `id` of `commands` on `ctx` with `args`, with the main window open or not. It
/// runs only when it exists, is enabled, has the window if it needs it and its arguments check.
pub fn run<C>(commands: &[Command<C>], ctx: &mut C, id: &str, args: &[(&str, Value)], window_open: bool) -> Outcome {
    let command = commands.iter().find(|c| c.id == id).ok_or_else(|| CommandError::UnknownCommand(id.to_owned()))?;
    command.bind(ctx, args, window_open)?.run(ctx)
}

/// A command ready to run: its arguments checked, its state and window need met.
pub struct Call<C> {
    run: Run<C>,
    args: Args,
}

impl<C> Call<C> {
    pub fn run(self, ctx: &mut C) -> Outcome {
        match self.run {
            Run::Plain(run) => {
                run(ctx);
                Ok(None)
            }
            Run::Typed(run) => run(ctx, &self.args),
            Run::Item(run, item) => {
                run(ctx, item);
                Ok(None)
            }
        }
    }
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
            params: &[],
            needs_window: false,
            run: Run::Plain(run),
        }
    }

    /// A command taking the arguments `params` and returning a result, otherwise like [`new`](Self::new).
    pub fn typed(id: &'static str, caption: &'static str, params: &'static [Param], run: fn(&mut C, &Args) -> Outcome) -> Command<C> {
        Command { params, run: Run::Typed(run), ..Command::new(id, caption, |_| {}) }
    }

    /// A command made for `item`, which `run` is given, otherwise like [`new`](Self::new).
    pub fn for_item(id: &'static str, caption: &'static str, item: &'static str, run: fn(&mut C, &'static str)) -> Command<C> {
        Command { run: Run::Item(run, item), ..Command::new(id, caption, |_| {}) }
    }

    /// See [`Command::needs_window`](field@Command::needs_window).
    pub fn needs_window(mut self) -> Command<C> {
        self.needs_window = true;
        self
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

    /// Checks that it can run on `ctx` with `args`: it has the window if it needs it, it is
    /// enabled, and its arguments check. The [`Call`] doesn't borrow the command, so the command
    /// list may live in `ctx`.
    pub fn bind(&self, ctx: &C, args: &[(&str, Value)], window_open: bool) -> Result<Call<C>, CommandError> {
        if self.needs_window && !window_open {
            return Err(CommandError::NeedsWindow(self.id));
        }
        if !(self.state)(ctx).enabled {
            return Err(CommandError::Disabled(self.id));
        }
        Ok(Call { run: self.run, args: Args::check(self.params, args)? })
    }

    /// Whether the command applies while a view of `view_type` is active.
    pub fn applies_to(&self, view_type: &str) -> bool {
        match self.views {
            Views::All => true,
            Views::Only(types) => types.contains(&view_type),
        }
    }
}
