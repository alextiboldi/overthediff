//! The main window's shell (E01-S1-T3): a tab strip, the Home launcher tab and the status bar.
//!
//! Features plug in two ways. A [`SessionType`] in the app's session-type list
//! (`apps/otd/src/session_types.rs`) gets a button on Home, which opens its empty [`View`] in a new
//! tab. A launch-argument handler opens a view with [`Shell::open`]. The shell draws the status
//! bar from the active view's [`Status`] (the Q25 field model in E01-S1); views never draw it.
//! The session model, the Session Settings dialog and the session commands are in [`session`].
//! Registered commands fill the menus, the toolbar and the shortcuts ([`commands`]).
//! Background tasks show their progress and end-of-task summary through [`show_task`].
//! A view with unsaved changes says so with [`View::unsaved`] and saves with [`View::save`]; Close
//! Tab and Exit then ask Save / Don't Save / Cancel first (`unsaved.rs`).
//! A feature takes a typed or browsed path with a [`PathEdit`], which keeps the field's recent paths.
//! Features contribute program options to the Options dialog's pages ([`OptionSpec`], [`Shell::with_options`]).
//! The Appearance page's theme choice ([`theme`]) sets the active colour palette every frame.

mod tasks;

pub use tasks::{CANCEL, CANCELLED, CLOSE, FINISHED_WITH_ERRORS, show_task};

pub mod colour;
pub mod theme;

pub mod path_edit;
pub use path_edit::{Browse, PathEdit, PathEdits};

mod commands;
mod unsaved;

pub use unsaved::{CANT_SAVE, DONT_SAVE, NOT_SAVED, SAVE, SAVE_CHANGES, UNSAVED_LOST, save_first_later};

use std::any::Any;
use std::time::Duration;

pub use commands::{APP_NAME, CLEAR_SESSION, CLOSE_CURRENT_TAB, EXIT, NEW_SESSION, NEXT_TAB, OPEN_SESSION, PREVIOUS_TAB, keymap};
pub use otd_core::command;

use egui::accesskit::{Live, Role};
use egui::{Response, Sense, Stroke, TextStyle, TextWrapMode, Ui, Vec2, WidgetInfo, WidgetText};
use otd_core::command::{Command, HOME_VIEW};
use otd_core::i18n::{fmt, measure, t};

mod options;
pub use options::{APPLY, CHOOSE_A_PAGE, DuplicateOption, NO_OPTIONS, OPTION_PAGES, OPTIONS, OptionSpec, PREVIEW, option};

mod session;
mod session_defaults;
pub use session::{
    DESCRIPTION, DuplicateSetting, INPUTS, Kind, OK, SCOPE, SESSION_SETTINGS, Session, SessionError, Setting, THIS_VIEW_ONLY,
    UPDATE_SESSION_DEFAULTS, Value, edit_value,
};

// The shell's own text: English keys, drawn only through `t` (E01-S1-T13). The menus' and the tab
// commands' text is in `commands.rs` (`MENUS` and the commands' captions), also drawn through `t`.
pub const HOME: &str = "Home";
pub const NEW_TAB: &str = "New Tab";
/// The close button's name; `{tab}` is the tab's title.
pub const CLOSE_TAB: &str = "Close Tab {tab}";
pub const NO_SESSION_TYPES: &str = "No session types are available yet.";
pub const INSERT: &str = "Insert";
pub const OVERWRITE: &str = "Overwrite";
pub const LOAD_TIME: &str = "Load time";
/// A status-bar field's text.
const FIELD: &str = "{label}: {value}";
/// The load time's unit; `{n}` is the number of seconds.
const SECONDS: &str = "{n} s";

/// What a tab shows: a session of some type, or a test view.
pub trait View: Any {
    /// The tab's caption, in the interface language (`otd_core::i18n::t`).
    fn title(&self) -> String;
    /// The view type, which decides the menus, toolbar and shortcuts shown with it: the id that
    /// commands name in `Views::Only`, by convention the id of the session type that opens it.
    /// The default, an empty id, gets only the commands for every view type.
    fn view_type(&self) -> &'static str {
        ""
    }
    /// The fields this view publishes to the status bar. Read every frame.
    fn status(&self) -> Status {
        Status::default()
    }
    /// The status bar's insert/overwrite field was activated. Only views that publish
    /// [`Status::insert`] get this.
    fn toggle_insert(&mut self) {}
    /// The session this view shows, which Session Settings edits and Clear Session replaces. `None`
    /// for a view that shows no session.
    fn session(&mut self) -> Option<&mut Session> {
        None
    }
    /// Whether the view has changes that closing it would lose. Closing its tab or exiting then
    /// asks Save / Don't Save / Cancel first.
    fn unsaved(&self) -> bool {
        false
    }
    /// Saves the view's changes, or says in the error, already looked up (`otd_core::i18n::t`), why it
    /// couldn't; the view then stays open. Called only when [`View::unsaved`] is true.
    // ponytail: synchronous, as Text Compare's save of files it already has paths for is; a save
    // that needs a file picker or a background task comes with the first view that needs one.
    fn save(&mut self) -> Result<(), String> {
        Err(t(CANT_SAVE).to_owned())
    }
    fn ui(&mut self, ui: &mut Ui);
}

/// A kind of session Home can start, such as Text Compare.
pub struct SessionType {
    /// A stable id, unique among session types, independent of the translated name.
    pub id: &'static str,
    /// The Home button's caption, in English: the shell looks it up (`otd_core::i18n::t`).
    pub name: &'static str,
    /// The file-format kinds it opens. [`Shell::session_type_for`] looks a kind up.
    pub kinds: &'static [Kind],
    /// Each side's caption in English, such as Left and Right, which the shell looks up: one input
    /// per side on the Inputs tab.
    pub sides: &'static [&'static str],
    /// An empty session of this type. Its [`View::session`] has this type's id; the shell gives it
    /// one input per side and each contributed setting's default.
    pub new: fn() -> Box<dyn View>,
}

/// One status-bar field. Its accessible name starts with its label. The view looks its label and
/// value up (`otd_core::i18n::t`); the shell draws them as given.
pub struct Field {
    /// A stable id, unique within the view.
    pub id: &'static str,
    pub label: String,
    pub value: String,
}

/// A view's status-bar fields. Every one is optional, and the shell shows them in this order.
#[derive(Default)]
pub struct Status {
    /// The overall result in the view's own words, announced when it changes.
    pub result: Option<Field>,
    /// The view's own fields, in its order.
    pub fields: Vec<Field>,
    /// `Some(true)` for insert, `Some(false)` for overwrite; only views that edit set it.
    pub insert: Option<bool>,
    /// How long the last load and comparison took. Not announced.
    pub load_time: Option<Duration>,
}

/// A tab's contents; `None` is Home.
type Tab = Option<Box<dyn View>>;

fn title(tab: &Tab) -> String {
    tab.as_ref().map_or_else(|| t(HOME).to_owned(), |view| view.title())
}

/// The main window's contents. Launch-argument handlers act on it (`otd_launch::Handler<Shell>`).
pub struct Shell {
    session_types: Vec<SessionType>,
    tabs: Vec<Tab>,
    active: usize,
    /// The active tab changed by a command, so its tab in the strip takes focus on the next draw.
    focus_active: bool,
    commands: Vec<Command<Shell>>,
    dispatcher: craft_keymap::egui::Dispatcher,
    /// Whether the window draws its own menu bar; not while a native one shows the menus.
    in_window_menu: bool,
    tasks: tasks::Tasks,
    /// The options features contribute to Session Settings tabs.
    settings: Vec<Setting>,
    /// Each session type's stored defaults, by type id ([`Shell::load_session_defaults`]).
    session_defaults: std::collections::BTreeMap<String, otd_settings::session_defaults::Rules>,
    /// The open Session Settings dialog.
    dialog: Option<session::Dialog>,
    /// Sessions whose settings Session Settings made their type's defaults, for the app to store.
    new_defaults: Vec<Session>,
    /// The open unsaved-changes prompt.
    prompt: Option<unsaved::Prompt>,
    /// Every unsaved view was answered for, so the window may close.
    exit_confirmed: bool,
    /// The Exit command ran; the next draw asks to close the window.
    exit_requested: bool,
    /// The widget that takes focus on the next draw, back from a cancelled prompt; the selected tab
    /// when that widget is gone.
    refocus: Option<egui::Id>,
    /// The program options features contribute, their values and the Options dialog.
    options: options::Options,
    /// The sessions of views closed since the app last took them ([`Shell::take_closed_sessions`]).
    closed: Vec<Session>,
}

impl Shell {
    /// A shell with one Home tab.
    pub fn new(session_types: Vec<SessionType>) -> Self {
        let commands = commands::shell_commands(&session_types);
        let mut shell = Self {
            session_types,
            tabs: Vec::new(),
            active: 0,
            focus_active: false,
            commands,
            dispatcher: Default::default(),
            in_window_menu: true,
            tasks: tasks::Tasks::default(),
            settings: Vec::new(),
            session_defaults: Default::default(),
            dialog: None,
            new_defaults: Vec::new(),
            prompt: None,
            exit_confirmed: false,
            exit_requested: false,
            refocus: None,
            options: options::Options::default(),
            closed: Vec::new(),
        };
        shell.push(None);
        shell.focus_active = false;
        shell
    }

    /// Whether one of the shell's dialogs (Session Settings, Options or an unsaved-changes prompt) is
    /// open, which keeps the window behind it out of reach.
    fn dialog_open(&self) -> bool {
        self.dialog.is_some() || self.options.dialog.is_some() || self.prompt.is_some()
    }

    /// The active view type: [`HOME_VIEW`] on Home.
    pub fn view_type(&self) -> &'static str {
        self.tabs[self.active].as_ref().map_or(HOME_VIEW, |view| view.view_type())
    }

    /// The active view, if it is a `V`: how a view type's commands reach their view.
    pub fn view<V: View>(&self) -> Option<&V> {
        let view: &dyn Any = self.tabs[self.active].as_deref()?;
        view.downcast_ref()
    }

    /// The active view, if it is a `V`, to change.
    pub fn view_mut<V: View>(&mut self) -> Option<&mut V> {
        let view: &mut dyn Any = self.tabs[self.active].as_deref_mut()?;
        view.downcast_mut()
    }

    /// Shows `view` in a new tab and selects it. Dropping the view is how it learns it closed.
    pub fn open(&mut self, view: Box<dyn View>) {
        self.push(Some(view));
    }

    /// Opens Home in a new tab.
    pub fn new_tab(&mut self) {
        self.push(None);
    }

    /// Closes the active tab and selects the one after it, or else the one before. Closing the last
    /// tab leaves Home. A view with unsaved changes asks first.
    pub fn close_tab(&mut self) {
        self.request_close(self.active);
    }

    /// Closes the tab at `index` without asking. Closing a tab other than the active one keeps the active tab.
    fn close(&mut self, index: usize) {
        // The dialog edits a tab by index, so it goes with any close.
        self.dialog = None;
        if let Some(session) = self.tabs.remove(index).as_mut().and_then(|view| view.session()) {
            self.closed.push(session.clone());
        }
        if index < self.active {
            self.active -= 1;
        }
        if self.tabs.is_empty() {
            self.tabs.push(None);
        }
        self.select(self.active.min(self.tabs.len() - 1));
    }

    /// Selects the next tab, wrapping round.
    pub fn next_tab(&mut self) {
        self.select((self.active + 1) % self.tabs.len());
    }

    /// Selects the previous tab, wrapping round.
    pub fn previous_tab(&mut self) {
        self.select((self.active + self.tabs.len() - 1) % self.tabs.len());
    }

    fn select(&mut self, index: usize) {
        self.active = index;
        self.focus_active = true;
    }

    fn push(&mut self, mut view: Option<Box<dyn View>>) {
        if let Some(view) = &mut view {
            self.prepare(view.as_mut());
        }
        self.tabs.push(view);
        self.select(self.tabs.len() - 1);
    }

    /// Draws the window: menu bar, tab strip, toolbar, the active tab's contents and the status
    /// bar.
    pub fn ui(&mut self, ui: &mut Ui) {
        // ponytail: views and roles recolour this frame; the chrome a frame later, since `ui`
        // already holds its style, and the restyle requests that frame.
        colour::sync_system(ui.ctx());
        self.options.publish(ui.ctx());
        theme::apply(ui.ctx());
        let keymap = keymap(&self.commands, ui.ctx().os().into());
        self.shortcuts(ui, &keymap);
        self.menu_bar(ui, &keymap);
        egui::Panel::top("otd-tab-strip").show(ui, |ui| self.tab_strip(ui));
        self.toolbar(ui, &keymap);
        // Laid out by hand rather than as a bottom panel, so the bar comes after the contents in
        // the Tab order, as it does on screen.
        let bar_height = ui.spacing().interact_size.y + 2.0 * ui.spacing().item_spacing.y;
        let rect = ui.available_rect_before_wrap();
        let (contents, bar) = rect.split_top_bottom_at_y(rect.bottom() - bar_height);
        ui.scope_builder(egui::UiBuilder::new().max_rect(contents), |ui| {
            egui::Frame::central_panel(ui.style()).show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                self.contents(ui);
            });
        });
        ui.scope_builder(egui::UiBuilder::new().max_rect(bar), |ui| {
            ui.separator();
            ui.horizontal(|ui| self.status_bar(ui));
        });
        self.dialog_ui(ui.ctx());
        self.options_ui(ui.ctx());
        self.tasks.ui(ui.ctx());
        if std::mem::take(&mut self.exit_requested) && self.exit() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
        // After the contents are drawn, so egui doesn't drop it as a widget nobody drew yet.
        if let Some(id) = self.refocus.take() {
            // A widget that is gone, such as the menu row that ran Close Tab, can't take focus back:
            // the selected tab takes it on the next draw (rule 21).
            if ui.ctx().read_response(id).is_some() {
                ui.memory_mut(|memory| memory.request_focus(id));
            } else {
                self.focus_active = true;
            }
        }
        self.save_first_queued(ui.ctx());
        // Last, so it is the topmost modal.
        self.prompt_ui(ui.ctx());
        if self.focus_active {
            // A click changed the active tab after the strip was drawn: draw again to move focus.
            ui.ctx().request_repaint();
        }
    }

    fn tab_strip(&mut self, ui: &mut Ui) {
        enum Action {
            Select(usize),
            Close(usize),
            New,
        }
        let mut action = None;
        ui.horizontal(|ui| {
            ui.ctx().accesskit_node_builder(ui.unique_id(), |node| node.set_role(Role::TabList));
            let count = self.tabs.len();
            for (index, tab) in self.tabs.iter().enumerate() {
                let selected = index == self.active;
                let title = title(tab);
                // No `push_id` scope: its container would sit between the tab and the tab list.
                let response = ui.selectable_label(selected, &title);
                // The tab role and selected state, as craft-ui's `tabs::Tab` sets them.
                // ponytail: not craft-ui itself, which would add its other dependencies for these
                // lines; the app now has craft-i18n, so switching is a refactor of its own.
                ui.ctx().accesskit_node_builder(response.id, |node| {
                    node.set_role(Role::Tab);
                    node.clear_toggled();
                    node.set_selected(selected);
                    // "2 of 3", which tells apart two tabs with the same title, such as two Homes.
                    node.set_position_in_set(index + 1);
                    node.set_size_of_set(count);
                });
                // Not while a dialog or prompt is open: a view opened behind it doesn't take its focus.
                if selected && self.focus_active && !self.dialog_open() {
                    response.request_focus();
                }
                if response.clicked() {
                    action = Some(Action::Select(index));
                }
                if icon_button(ui, &fmt(t(CLOSE_TAB), &[("tab", &title)]), &LUCIDE_X).clicked() {
                    action = Some(Action::Close(index));
                }
            }
            if icon_button(ui, t(NEW_TAB), &LUCIDE_PLUS).clicked() {
                action = Some(Action::New);
            }
        });
        self.focus_active = false;
        match action {
            Some(Action::Select(index)) => self.active = index,
            Some(Action::Close(index)) => self.request_close(index),
            Some(Action::New) => self.new_tab(),
            None => {}
        }
    }

    fn contents(&mut self, ui: &mut Ui) {
        if let Some(view) = &mut self.tabs[self.active] {
            view.ui(ui);
            return;
        }
        ui.heading(t(HOME));
        if self.session_types.is_empty() {
            ui.label(t(NO_SESSION_TYPES));
        }
        let mut chosen = None;
        for session_type in &self.session_types {
            if ui.push_id(session_type.id, |ui| ui.button(t(session_type.name))).inner.clicked() {
                chosen = Some(session_type.new);
            }
        }
        if let Some(new) = chosen {
            self.open(new());
        }
    }

    fn status_bar(&mut self, ui: &mut Ui) {
        // Home publishes no fields, so its bar is empty.
        let Some(view) = &mut self.tabs[self.active] else { return };
        let status = view.status();
        if let Some(result) = &status.result {
            let response = field(ui, result);
            ui.ctx().accesskit_node_builder(response.id, |node| node.set_live(Live::Polite));
        }
        for own in &status.fields {
            field(ui, own);
        }
        if let Some(insert) = status.insert {
            ui.separator();
            if ui.button(t(if insert { INSERT } else { OVERWRITE })).clicked() {
                view.toggle_insert();
            }
        }
        if let Some(load_time) = status.load_time {
            let value = measure(load_time.as_secs_f64(), 2, SECONDS);
            field(ui, &Field { id: "load-time", label: t(LOAD_TIME).to_owned(), value });
        }
    }
}

fn field(ui: &mut Ui, field: &Field) -> Response {
    ui.separator();
    ui.push_id(field.id, |ui| ui.label(fmt(t(FIELD), &[("label", &field.label), ("value", &field.value)]))).inner
}

/// Lucide's `x` and `plus` icons (ISC; both from Feather, MIT; see ATTRIBUTION.md): their
/// `<path>` lines as segments on the icons' 24-unit grid, drawn with their stroke width of 2.
const LUCIDE_X: [[(f32, f32); 2]; 2] = [[(18.0, 6.0), (6.0, 18.0)], [(6.0, 6.0), (18.0, 18.0)]];
const LUCIDE_PLUS: [[(f32, f32); 2]; 2] = [[(5.0, 12.0), (19.0, 12.0)], [(12.0, 5.0), (12.0, 19.0)]];
/// Lucide's `sliders-horizontal` (ISC; see ATTRIBUTION.md), for the Session Settings button.
const LUCIDE_SLIDERS_HORIZONTAL: [[(f32, f32); 2]; 9] = [
    [(10.0, 5.0), (3.0, 5.0)],
    [(12.0, 19.0), (3.0, 19.0)],
    [(14.0, 3.0), (14.0, 7.0)],
    [(16.0, 17.0), (16.0, 21.0)],
    [(21.0, 12.0), (12.0, 12.0)],
    [(21.0, 19.0), (16.0, 19.0)],
    [(21.0, 5.0), (14.0, 5.0)],
    [(8.0, 10.0), (8.0, 14.0)],
    [(8.0, 12.0), (3.0, 12.0)],
];

/// Lucide's `arrow-up-right` (ISC; from Feather, MIT; see ATTRIBUTION.md), Folder Compare's link glyph.
const LUCIDE_ARROW_UP_RIGHT: [[(f32, f32); 2]; 3] = [[(7.0, 7.0), (17.0, 7.0)], [(17.0, 7.0), (17.0, 17.0)], [(7.0, 17.0), (17.0, 7.0)]];

/// A Lucide icon line by line, by its Lucide name. A command whose icon isn't here shows its
/// caption alone.
fn lucide(name: &str) -> Option<&'static [[(f32, f32); 2]]> {
    match name {
        "x" => Some(&LUCIDE_X),
        "plus" => Some(&LUCIDE_PLUS),
        "sliders-horizontal" => Some(&LUCIDE_SLIDERS_HORIZONTAL),
        "arrow-up-right" => Some(&LUCIDE_ARROW_UP_RIGHT),
        _ => None,
    }
}

const ICON: f32 = 16.0;

/// Paints the Lucide icon `name`, `size` points wide, centred on `center`, for a view's own glyphs.
/// Paints nothing when `otd_ui`'s table doesn't have it.
pub fn paint_lucide(ui: &Ui, name: &str, center: egui::Pos2, size: f32, color: egui::Color32) {
    if let Some(icon) = lucide(name) {
        paint_icon_sized(ui, center, size, icon, color);
    }
}

/// Paints a Lucide line icon, `ICON` points wide, centred on `center`.
fn paint_icon(ui: &Ui, center: egui::Pos2, icon: &[[(f32, f32); 2]], color: egui::Color32) {
    paint_icon_sized(ui, center, ICON, icon, color);
}

fn paint_icon_sized(ui: &Ui, center: egui::Pos2, size: f32, icon: &[[(f32, f32); 2]], color: egui::Color32) {
    let scale = size / 24.0;
    let origin = center - Vec2::splat(size / 2.0);
    let at = |(x, y): (f32, f32)| origin + Vec2::new(x, y) * scale;
    let stroke = Stroke::new(2.0 * scale, color);
    for &[from, to] in icon {
        ui.painter().line_segment([at(from), at(to)], stroke);
        // Lucide's round line caps.
        for end in [at(from), at(to)] {
            ui.painter().circle_filled(end, stroke.width / 2.0, stroke.color);
        }
    }
}

/// The background and focus ring of a drawn button.
fn paint_button(ui: &Ui, rect: egui::Rect, response: &Response) {
    let visuals = ui.style().interact(response);
    if response.hovered() || response.has_focus() || response.is_pointer_button_down_on() {
        ui.painter().rect(rect, visuals.corner_radius, visuals.weak_bg_fill, visuals.bg_stroke, egui::StrokeKind::Inside);
    }
    if response.has_focus() {
        ui.painter().rect_stroke(rect, visuals.corner_radius, ui.visuals().selection.stroke, egui::StrokeKind::Inside);
    }
}

/// A 24-point button showing a Lucide line icon, named `name` for screen readers and tooltips.
fn icon_button(ui: &mut Ui, name: &str, icon: &[[(f32, f32); 2]]) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(24.0), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), name));
    paint_button(ui, rect, &response);
    paint_icon(ui, rect.center(), icon, ui.style().interact(&response).fg_stroke.color);
    response.on_hover_text(name)
}

/// A toolbar button: the command's icon, when there is one, then its caption, which is also its
/// name for screen readers. A disabled one is greyed, can't take focus and doesn't click.
fn tool_button(ui: &mut Ui, caption: &str, icon: Option<&[[(f32, f32); 2]]>, enabled: bool) -> Response {
    let galley = WidgetText::from(caption).into_galley(ui, Some(TextWrapMode::Extend), f32::INFINITY, TextStyle::Button);
    let padding = ui.spacing().button_padding;
    let icon_width = if icon.is_some() { ICON + ui.spacing().icon_spacing } else { 0.0 };
    let size = Vec2::new(2.0 * padding.x + icon_width + galley.size().x, (2.0 * padding.y + galley.size().y).max(24.0));
    let (rect, response) = ui.allocate_exact_size(size, if enabled { Sense::click() } else { Sense::hover() });
    response.widget_info(|| WidgetInfo::labeled(Role::Button, enabled, caption));
    paint_button(ui, rect, &response);
    // A hover-only response gets egui's non-interactive, greyed colours.
    let color = ui.style().interact(&response).fg_stroke.color;
    let left = rect.left() + padding.x;
    if let Some(icon) = icon {
        paint_icon(ui, egui::pos2(left + ICON / 2.0, rect.center().y), icon, color);
    }
    let text_at = egui::pos2(left + icon_width, rect.center().y - galley.size().y / 2.0);
    ui.painter().galley(text_at, galley, color);
    response
}
