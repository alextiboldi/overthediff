//! Sessions in the shell (E01-S1-T7): looking a session type up by format kind, the session commands
//! (New Session, Open Session, Clear Session) and the Session Settings dialog.
//!
//! A view shows a session by returning it from [`View::session`]. The dialog edits a copy of the
//! active view's session: the Specs tab holds one input per side and the Description, and each
//! tab contributed for the session type holds its contributed options ([`Setting`], added with
//! [`Shell::with_settings`]). OK writes the copy back to that view. With the scope "Update session
//! defaults" (E06-S2-T2) OK also hands the copy to the app, which stores its settings as its type's
//! defaults ([`Shell::take_session_defaults`]); with "Use for this view only" it doesn't. Cancel
//! and Esc drop the copy, and a click outside it does nothing.

use std::fmt;

use egui::accesskit::Role;
use egui::{Color32, Context, Id, Key, Modifiers, Response, TextEdit, Ui};

use otd_core::i18n::t;
use otd_core::session::check_unique;
pub use otd_core::session::{DuplicateSetting, Session, Setting, Value};
pub use otd_format::Kind;

use crate::{Browse, CANCEL, PathEdit, SessionType, Shell, View};

// The dialog's own text: English keys, drawn only through `t`, as in lib.rs.
/// The Session Settings dialog's title, in our own words (docs/feature-names.md).
pub const SESSION_SETTINGS: &str = "Comparison Settings";
/// The tab the backlog calls Specs: the session's inputs and its description.
pub const INPUTS: &str = "Inputs";
pub const DESCRIPTION: &str = "Description";
pub const SCOPE: &str = "Scope";
pub const THIS_VIEW_ONLY: &str = "Use for this view only";
pub const UPDATE_SESSION_DEFAULTS: &str = "Update session defaults";
pub const OK: &str = "OK";
const NO_SESSION: &str = "The current tab shows no session.";
const UNKNOWN_TYPE: &str = "This session's type is not available.";

/// Why a session command did nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionError {
    /// The active tab is Home, or a view that shows no session.
    NoSession,
    /// No registered session type has the id.
    UnknownType,
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NoSession => t(NO_SESSION),
            Self::UnknownType => t(UNKNOWN_TYPE),
        })
    }
}

impl std::error::Error for SessionError {}

/// The open Session Settings dialog.
pub(crate) struct Dialog {
    /// The shell tab whose session it edits. Tabs only open at the end while it is open, so the
    /// index stays put.
    tab: usize,
    /// The copy OK writes back.
    working: Session,
    /// The contributed tabs, after Specs, in the order their first option was contributed.
    pages: Vec<&'static str>,
    /// The selected tab: 0 is Specs, then `pages`.
    page: usize,
    /// Focus moves to the selected tab on the next draw.
    focus: bool,
    /// The single-line edits drawn last frame, where Enter is OK.
    single_line: Vec<Id>,
    /// The scope "Update session defaults" is chosen; each open starts on "Use for this view only".
    update_defaults: bool,
}

impl Shell {
    /// Adds the options features contribute to Session Settings tabs. Fails when one shares its
    /// session type, tab and id with another, added now or before.
    pub fn with_settings(mut self, settings: Vec<Setting>) -> Result<Self, DuplicateSetting> {
        self.settings.extend(settings);
        check_unique(&self.settings)?;
        Ok(self)
    }

    /// The session type that opens files of `kind`: the first registered one that declares it.
    pub fn session_type_for(&self, kind: Kind) -> Option<&SessionType> {
        self.session_types.iter().find(|t| t.kinds.contains(&kind))
    }

    pub(crate) fn session_type(&self, id: &str) -> Option<&SessionType> {
        self.session_types.iter().find(|t| t.id == id)
    }

    /// New Session ▸ <type>: opens an empty session of the type `type_id` in a new tab.
    pub fn new_session(&mut self, type_id: &str) -> Result<(), SessionError> {
        let new = self.session_type(type_id).ok_or(SessionError::UnknownType)?.new;
        self.open(new());
        Ok(())
    }

    /// Open Session: selects the first Home tab, or opens Home in a new tab when none is open.
    pub fn open_session(&mut self) {
        match self.tabs.iter().position(Option::is_none) {
            Some(home) => self.select(home),
            None => self.new_tab(),
        }
    }

    /// Clear Session: replaces the active view with an empty session of the same type. A view with
    /// unsaved changes asks Save / Don't Save / Cancel first, and Cancel keeps it. Does nothing
    /// while that prompt is open.
    pub fn clear_session(&mut self) -> Result<(), SessionError> {
        let type_id = self.active_session().ok_or(SessionError::NoSession)?.type_id.clone();
        let new = self.session_type(&type_id).ok_or(SessionError::UnknownType)?.new;
        self.request_replace(self.active, new);
        Ok(())
    }

    /// Replaces the view at `index` with `new`, without asking.
    pub(crate) fn replace(&mut self, index: usize, new: fn() -> Box<dyn View>) {
        let mut view = new();
        self.prepare(view.as_mut());
        self.dialog = None;
        // The replaced view closes, so its session is auto-saved as a closed tab's is.
        if let Some(session) = self.tabs[index].replace(view).as_mut().and_then(|old| old.session()) {
            self.closed.push(session.clone());
        }
        self.select(index);
    }

    /// Session Settings (and the Rules toolbar button): opens the dialog on the active view's session.
    pub fn session_settings(&mut self) -> Result<(), SessionError> {
        let working = self.active_session().ok_or(SessionError::NoSession)?.clone();
        let mut pages = Vec::new();
        for setting in self.settings.iter().filter(|s| s.session_type == working.type_id) {
            if !pages.contains(&setting.tab) {
                pages.push(setting.tab);
            }
        }
        self.dialog = Some(Dialog { tab: self.active, working, pages, page: 0, focus: true, single_line: Vec::new(), update_defaults: false });
        Ok(())
    }

    /// The sessions OK made their type's defaults since the last call, oldest first: what the app
    /// stores (`otd_settings::session_defaults::write`). Each holds every setting of its type.
    pub fn take_session_defaults(&mut self) -> Vec<Session> {
        std::mem::take(&mut self.new_defaults)
    }

    /// The sessions of the views closed since the last call, oldest first, for auto-saving them.
    pub fn take_closed_sessions(&mut self) -> Vec<Session> {
        std::mem::take(&mut self.closed)
    }

    /// The session of each open view, in tab order: what exiting closes.
    pub fn open_sessions(&mut self) -> Vec<Session> {
        self.tabs.iter_mut().filter_map(|tab| Some(tab.as_mut()?.session()?.clone())).collect()
    }

    pub(crate) fn active_session(&mut self) -> Option<&mut Session> {
        self.tabs[self.active].as_mut()?.session()
    }

    /// Completes a view's session as an empty session of its type would be: one input per side and
    /// each contributed setting's value from its type's defaults, keeping what it already has.
    pub(crate) fn prepare(&self, view: &mut dyn View) {
        let Some(session) = view.session() else { return };
        if let Some(session_type) = self.session_type(&session.type_id) {
            session.specs.resize(session.specs.len().max(session_type.sides.len()), String::new());
        }
        self.apply_session_defaults(session);
        session.fill_defaults(&self.settings);
    }

    /// Draws the Session Settings dialog, if open, over the window.
    pub(crate) fn dialog_ui(&mut self, ctx: &Context) {
        let Some(dialog) = &mut self.dialog else { return };
        let enter = ctx.memory(|m| m.focused()).is_some_and(|id| dialog.single_line.contains(&id)) && ctx.input(|i| i.key_pressed(Key::Enter));
        let sides = self.session_types.iter().find(|t| t.id == dialog.working.type_id).map_or(&[][..], |t| t.sides);
        let settings = &self.settings;
        let mut ok = enter;
        let mut cancel = false;
        let id = Id::unique("otd-session-settings");
        let modal = egui::Modal::new(id).show(ctx, |ui| {
            // The modal's area is its Dialog node; it takes the area's id.
            ui.ctx().accesskit_node_builder(id, |node| node.set_label(t(SESSION_SETTINGS)));
            ui.heading(t(SESSION_SETTINGS));
            dialog.tab_list(ui);
            ui.separator();
            dialog.single_line.clear();
            let Dialog { working, pages, page, single_line, update_defaults, .. } = dialog;
            match page.checked_sub(1).and_then(|p| pages.get(p)) {
                None => specs(ui, sides, working, single_line),
                Some(tab) => {
                    let values = working.rules.entry((*tab).to_owned()).or_default();
                    for setting in settings.iter().filter(|s| s.session_type == working.type_id && s.tab == *tab) {
                        let value = values.entry(setting.id.to_owned()).or_insert_with(|| setting.default.clone());
                        let single = matches!(value, Value::Text(_) | Value::Path(_));
                        let response = ui.push_id(setting.id, |ui| edit_value(ui, setting, value)).inner;
                        if single {
                            single_line.push(response.id);
                        }
                    }
                }
            }
            ui.separator();
            let selected = if *update_defaults { UPDATE_SESSION_DEFAULTS } else { THIS_VIEW_ONLY };
            let scope = egui::ComboBox::new("otd-session-scope", t(SCOPE)).selected_text(t(selected)).show_ui(ui, |ui| {
                let picked = ui.selectable_value(update_defaults, false, t(THIS_VIEW_ONLY)).clicked()
                    | ui.selectable_value(update_defaults, true, t(UPDATE_SESSION_DEFAULTS)).clicked();
                // A pick by keyboard leaves egui's popup open; close it and give focus back to the box.
                if picked {
                    ui.close();
                }
                picked
            });
            if scope.inner == Some(true) {
                scope.response.request_focus();
            }
            ui.horizontal(|ui| {
                ok |= ui.button(t(OK)).clicked();
                cancel |= ui.button(t(CANCEL)).clicked();
            });
        });
        // Esc closes it as Cancel does, unless it closes a popup first. A click outside it doesn't:
        // a stray click would drop what the user typed.
        let escape = modal.is_top_modal && !modal.any_popup_open && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape));
        if !(ok || cancel || escape) {
            return;
        }
        let dialog = self.dialog.take();
        if let Some(Dialog { tab, mut working, update_defaults, .. }) = dialog.filter(|_| ok)
            && let Some(session) = self.tabs.get_mut(tab).and_then(|t| t.as_mut()?.session())
        {
            if update_defaults {
                // Complete, as storing them needs: a tab never opened in the dialog may lack some.
                working.fill_defaults(&self.settings);
                self.new_defaults.push(working.clone());
            }
            *session = working;
        }
        // Focus goes back to the window: its selected tab.
        self.focus_active = true;
    }
}

impl Dialog {
    /// Specs, then each contributed tab, as a tab list.
    fn tab_list(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.ctx().accesskit_node_builder(ui.unique_id(), |node| {
                node.set_role(Role::TabList);
                node.set_label(t(SESSION_SETTINGS));
            });
            let names = std::iter::once(INPUTS).chain(self.pages.iter().copied());
            let count = self.pages.len() + 1;
            for (index, name) in names.enumerate() {
                let selected = index == self.page;
                let response = ui.selectable_label(selected, t(name));
                ui.ctx().accesskit_node_builder(response.id, |node| {
                    node.set_role(Role::Tab);
                    node.clear_toggled();
                    node.set_selected(selected);
                    node.set_position_in_set(index + 1);
                    node.set_size_of_set(count);
                });
                if selected && self.focus {
                    response.request_focus();
                }
                if response.clicked() {
                    self.page = index;
                }
            }
        });
        self.focus = false;
    }
}

/// The Specs tab: one input per side, then the Description.
fn specs(ui: &mut Ui, sides: &[&str], session: &mut Session, single_line: &mut Vec<Id>) {
    for (index, (side, spec)) in sides.iter().zip(&mut session.specs).enumerate() {
        ui.push_id(index, |ui| single_line.push(labelled(ui, t(side), |ui| ui.add(TextEdit::singleline(spec))).id));
    }
    labelled(ui, t(DESCRIPTION), |ui| ui.add(TextEdit::multiline(&mut session.description)));
}

/// A caption, then the control `add` draws, named by the caption.
fn labelled(ui: &mut Ui, label: &str, add: impl FnOnce(&mut Ui) -> Response) -> Response {
    ui.horizontal(|ui| {
        let label = ui.label(label);
        add(ui).labelled_by(label.id)
    })
    .inner
}

/// The control that edits `value`, a value of `setting`, named by its label. Text values are
/// single-line edits, and Path values path edits whose recent paths are the setting's own. Returns
/// the editing control's response.
pub fn edit_value(ui: &mut Ui, setting: &Setting, value: &mut Value) -> Response {
    let field = format!("{}/{}/{}", setting.session_type, setting.tab, setting.id);
    edit(ui, &field, setting.id, setting.label, setting.choices, value)
}

/// [`edit_value`] for a value with the id `id`, the English label `label` and, for a choice, the
/// English `choices`: a Session Settings setting or a program option. A Path value keeps its
/// recent paths under `field`.
pub(crate) fn edit(ui: &mut Ui, field: &str, id: &str, label: &str, choices: &[&str], value: &mut Value) -> Response {
    match value {
        Value::Bool(on) => ui.checkbox(on, t(label)),
        // A finite range: the store reads back only finite numbers, and egui's default range lets a
        // typed `inf` or `nan` through.
        Value::Number(number) => labelled(ui, t(label), |ui| ui.add(egui::DragValue::new(number).range(f64::MIN..=f64::MAX))),
        Value::Choice(choice) => {
            let mut chosen = None;
            let response = egui::ComboBox::new(id, t(label))
                .selected_text(t(choice))
                .show_ui(ui, |ui| {
                    for option in choices {
                        if ui.selectable_label(choice == option, t(option)).clicked() {
                            chosen = Some(*option);
                        }
                    }
                })
                .response;
            if let Some(option) = chosen {
                *choice = option.to_owned();
            }
            response
        }
        Value::Text(text) => labelled(ui, t(label), |ui| ui.add(TextEdit::singleline(text))),
        Value::Path(path) => labelled(ui, t(label), |ui| PathEdit::new(field, path, Browse::Both).show(ui)),
        Value::Colour(rgba) => labelled(ui, t(label), |ui| {
            let mut colour = Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3]);
            let response = ui.color_edit_button_srgba(&mut colour);
            // egui names the button "Color"; the setting's label names it instead.
            ui.ctx().accesskit_node_builder(response.id, |node| node.set_label(t(label)));
            // Only on a change: the round trip through premultiplied alpha can round.
            if response.changed() {
                *rgba = colour.to_srgba_unmultiplied();
            }
            response
        }),
    }
}
