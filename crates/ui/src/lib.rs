//! The main window's shell (E01-S1-T3): a tab strip, the Home launcher tab and the status bar.
//!
//! Features plug in two ways. A [`SessionType`] in the app's session-type list
//! (`apps/otd/src/session_types.rs`) gets a button on Home, which opens its empty [`View`] in a new
//! tab. A launch-argument handler opens a view with [`Shell::open`]. The shell draws the status
//! bar from the active view's [`Status`] (the Q25 field model in E01-S1); views never draw it.
//! Background tasks show their progress and end-of-task summary through [`show_task`].

mod tasks;

pub use tasks::{CANCEL, CANCELLED, CLOSE, FINISHED_WITH_ERRORS, show_task};

pub mod colour;

use std::time::Duration;

use egui::accesskit::{Live, Role};
use egui::os::OperatingSystem;
use egui::{Key, Modifiers, Response, Sense, Stroke, Ui, Vec2, WidgetInfo};
use otd_core::i18n::{fmt, measure, t};

// The shell's own text: English keys, drawn only through `t` (E01-S1-T13).
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
pub trait View {
    /// The tab's caption, in the interface language (`otd_core::i18n::t`).
    fn title(&self) -> String;
    /// The fields this view publishes to the status bar. Read every frame.
    fn status(&self) -> Status {
        Status::default()
    }
    /// The status bar's insert/overwrite field was activated. Only views that publish
    /// [`Status::insert`] get this.
    fn toggle_insert(&mut self) {}
    fn ui(&mut self, ui: &mut Ui);
}

/// A kind of session Home can start, such as Text Compare.
pub struct SessionType {
    /// A stable id, unique among session types, independent of the translated name.
    pub id: &'static str,
    /// The Home button's caption, in English: the shell looks it up (`otd_core::i18n::t`).
    pub name: &'static str,
    /// An empty session of this type.
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
    tasks: tasks::Tasks,
}

impl Shell {
    /// A shell with one Home tab.
    pub fn new(session_types: Vec<SessionType>) -> Self {
        let mut shell = Self { session_types, tabs: Vec::new(), active: 0, focus_active: false, tasks: tasks::Tasks::default() };
        shell.push(None);
        shell.focus_active = false;
        shell
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
    /// tab leaves Home.
    pub fn close_tab(&mut self) {
        self.close(self.active);
    }

    /// Closes the tab at `index`. Closing a tab other than the active one keeps the active tab.
    fn close(&mut self, index: usize) {
        self.tabs.remove(index);
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

    fn push(&mut self, view: Option<Box<dyn View>>) {
        self.tabs.push(view);
        self.select(self.tabs.len() - 1);
    }

    /// Draws the window: tab strip, the active tab's contents and the status bar.
    pub fn ui(&mut self, ui: &mut Ui) {
        self.shortcuts(ui);
        egui::Panel::top("otd-tab-strip").show(ui, |ui| self.tab_strip(ui));
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
        self.tasks.ui(ui.ctx());
        if self.focus_active {
            // A click changed the active tab after the strip was drawn: draw again to move focus.
            ui.ctx().request_repaint();
        }
    }

    /// The tab shortcuts of the keymap conventions (Q24 in E01-S1).
    // ponytail: hard-wired until E01-S1-T4's command registry binds New Tab, Close Tab and the tab
    // switches as commands; it calls the same methods.
    fn shortcuts(&mut self, ui: &Ui) {
        // Idempotent: egui adds a plugin type once.
        ui.ctx().add_plugin(CtrlTab);
        let os = ui.ctx().os();
        let ctrl_tab = ui.ctx().data_mut(|data| data.remove_temp::<CtrlTabPresses>(egui::Id::unique(CTRL_TAB)));
        let CtrlTabPresses { mut previous, mut next } = ctrl_tab.unwrap_or_default();
        // A modal dialog (last frame's top modal layer) keeps the window behind it out of reach.
        if ui.ctx().memory(|memory| memory.top_modal_layer().is_some()) {
            return;
        }
        let [new, close] = ui.input_mut(|i| {
            // Exact modifiers, unlike egui's `consume_key`, which lets extra Shift and Alt through:
            // Ctrl+Alt+W is AltGr+W on Windows, and Cmd+Shift+T isn't New Tab.
            let mut pressed = |modifiers: Modifiers, keys: &[Key]| {
                let before = i.events.len();
                i.events.retain(|event| {
                    !matches!(event, egui::Event::Key { key, pressed: true, modifiers: held, .. }
                        if keys.contains(key) && held.matches_exact(modifiers))
                });
                i.events.len() < before
            };
            if os == OperatingSystem::Mac {
                let shift_cmd = Modifiers::SHIFT | Modifiers::COMMAND;
                previous |= pressed(shift_cmd, &[Key::OpenBracket, Key::OpenCurlyBracket]);
                next |= pressed(shift_cmd, &[Key::CloseBracket, Key::CloseCurlyBracket]);
            }
            let new = pressed(Modifiers::COMMAND, &[Key::T]);
            let close = pressed(Modifiers::COMMAND, &[Key::W]) || (os == OperatingSystem::Windows && pressed(Modifiers::CTRL, &[Key::F4]));
            [new, close]
        });
        if new {
            self.new_tab();
        }
        if close {
            self.close_tab();
        }
        if previous {
            self.previous_tab();
        }
        if next {
            self.next_tab();
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
                if selected && self.focus_active {
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
            Some(Action::Close(index)) => self.close(index),
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

const CTRL_TAB: &str = "otd-ctrl-tab";

/// Ctrl+Tab and Ctrl+Shift+Tab presses taken out of a frame's input by [`CtrlTab`].
#[derive(Clone, Copy, Default)]
struct CtrlTabPresses {
    previous: bool,
    next: bool,
}

/// Takes Ctrl+Tab and Ctrl+Shift+Tab out of the input before egui's focus handling sees them, and
/// leaves them for [`Shell::shortcuts`]. egui counts Ctrl+Shift+Tab as Shift+Tab on macOS
/// (`Modifiers::shift_only` ignores Ctrl), so it would also move focus back one widget.
struct CtrlTab;

impl egui::Plugin for CtrlTab {
    fn debug_name(&self) -> &'static str {
        CTRL_TAB
    }

    fn input_hook(&mut self, ctx: &egui::Context, input: &mut egui::RawInput) {
        let mut presses = CtrlTabPresses::default();
        input.events.retain(|event| match event {
            egui::Event::Key { key: Key::Tab, pressed, modifiers, .. } if modifiers.ctrl && !modifiers.alt => {
                if *pressed {
                    *(if modifiers.shift { &mut presses.previous } else { &mut presses.next }) = true;
                }
                false
            }
            _ => true,
        });
        if presses.previous || presses.next {
            ctx.data_mut(|data| data.insert_temp(egui::Id::unique(CTRL_TAB), presses));
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

/// A 24-point button showing a Lucide line icon, named `name` for screen readers and tooltips.
fn icon_button(ui: &mut Ui, name: &str, icon: &[[(f32, f32); 2]]) -> Response {
    const SIZE: f32 = 24.0;
    const ICON: f32 = 16.0;
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(SIZE), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), name));
    let visuals = ui.style().interact(&response);
    if response.hovered() || response.has_focus() || response.is_pointer_button_down_on() {
        ui.painter().rect(rect, visuals.corner_radius, visuals.weak_bg_fill, visuals.bg_stroke, egui::StrokeKind::Inside);
    }
    if response.has_focus() {
        ui.painter().rect_stroke(rect, visuals.corner_radius, ui.visuals().selection.stroke, egui::StrokeKind::Inside);
    }
    let scale = ICON / 24.0;
    let origin = rect.center() - Vec2::splat(ICON / 2.0);
    let at = |(x, y): (f32, f32)| origin + Vec2::new(x, y) * scale;
    let stroke = Stroke::new(2.0 * scale, visuals.fg_stroke.color);
    for &[from, to] in icon {
        ui.painter().line_segment([at(from), at(to)], stroke);
        // Lucide's round line caps.
        for end in [at(from), at(to)] {
            ui.painter().circle_filled(end, stroke.width / 2.0, stroke.color);
        }
    }
    response.on_hover_text(name)
}
