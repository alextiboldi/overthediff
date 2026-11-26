//! Home's sessions tree (E06-S1-T3): the New branch, with an entry per session type, then the
//! Saved branch of named sessions and the Auto-saved branch of recent unnamed ones.
//!
//! The app hands the shell the saved and auto-saved sessions ([`Shell::set_saved_sessions`],
//! [`Shell::set_auto_saved_sessions`]) whenever they change, and the tree draws them as they stand
//! each frame, so a session saved while Home shows appears at once. Double-clicking an entry, or
//! Enter on it, opens it: a New entry as an empty session of its type, a saved or auto-saved one in
//! its type's view with its specs and settings restored ([`Shell::restore_session`]). Up and Down
//! move the selection, and focus with it; Tab reaches each item too.

use std::collections::BTreeMap;

use egui::accesskit::{Live, Role};
use egui::{Id, Key, Modifiers, Ui};
use otd_core::command::{Args, Command, CommandError, Kind, Outcome, Param};
use otd_core::i18n::{fmt, t};

use crate::{HOME, NO_SESSION_TYPES, Session, SessionError, Shell};

// The tree's text: English keys, drawn only through `t`.
/// The tree's name.
pub const SESSIONS: &str = "Sessions";
pub const NEW: &str = "New";
pub const SAVED: &str = "Saved";
pub const AUTO_SAVED: &str = "Auto-saved";
/// The command that opens a saved session by name.
pub const OPEN_SAVED_SESSION: &str = "Open Saved Session";
const NO_SAVED_SESSION: &str = "There is no saved session named \"{name}\".";

/// The sessions the tree lists, and its selection.
#[derive(Default)]
pub(crate) struct Home {
    saved: BTreeMap<String, Session>,
    /// Newest first.
    auto_saved: Vec<Session>,
    /// The selected row.
    selected: usize,
    /// Focus moves to the selected row on the next draw.
    focus: bool,
    /// The rows drawn last frame, where Up and Down move the selection.
    items: Vec<Id>,
    /// Why the last open failed, until the next open.
    error: Option<String>,
}

/// What a row stands for.
enum Target {
    Branch,
    New(&'static str),
    Saved(String),
    AutoSaved(usize),
}

impl Shell {
    /// Replaces the named sessions the Saved branch lists, by name.
    pub fn set_saved_sessions(&mut self, saved: BTreeMap<String, Session>) {
        self.home.saved = saved;
    }

    /// Replaces the recent unnamed sessions the Auto-saved branch lists, newest first.
    pub fn set_auto_saved_sessions(&mut self, auto_saved: Vec<Session>) {
        self.home.auto_saved = auto_saved;
    }

    /// Opens `session` in a new tab, in its type's view, with its specs and settings.
    pub fn restore_session(&mut self, session: Session) -> Result<(), SessionError> {
        let new = self.session_type(&session.type_id).ok_or(SessionError::UnknownType)?.new;
        let mut view = new();
        *view.session().ok_or(SessionError::NoSession)? = session;
        self.open(view);
        Ok(())
    }

    /// Opens the named session `name`, as double-clicking it on Home does.
    fn open_saved_session(&mut self, name: &str) -> Result<(), CommandError> {
        let session = self.home.saved.get(name).cloned().ok_or_else(|| CommandError::Failed(fmt(t(NO_SAVED_SESSION), &[("name", name)])))?;
        self.restore_session(session).map_err(|e| CommandError::Failed(e.to_string()))
    }

    /// The tree's rows, in order: each one's level, name and target.
    fn home_rows(&self) -> Vec<(usize, String, Target)> {
        let mut rows = vec![(1, t(NEW).to_owned(), Target::Branch)];
        rows.extend(self.session_types.iter().map(|s| (2, t(s.name).to_owned(), Target::New(s.id))));
        rows.push((1, t(SAVED).to_owned(), Target::Branch));
        rows.extend(self.home.saved.keys().map(|name| (2, name.clone(), Target::Saved(name.clone()))));
        rows.push((1, t(AUTO_SAVED).to_owned(), Target::Branch));
        // Listed by their specs, since they have no name (*inferred*, E06's behavioral contract); by
        // their type's name when they have none, so no row is nameless.
        let specs = |s: &Session| {
            let specs = s.specs.iter().filter(|spec| !spec.is_empty()).cloned().collect::<Vec<_>>();
            if specs.is_empty() {
                self.session_type(&s.type_id).map_or_else(|| s.type_id.clone(), |ty| t(ty.name).to_owned())
            } else {
                specs.join(" — ")
            }
        };
        rows.extend(self.home.auto_saved.iter().enumerate().map(|(i, s)| (2, specs(s), Target::AutoSaved(i))));
        rows
    }

    /// Draws Home: its heading and the sessions tree.
    pub(crate) fn home_ui(&mut self, ui: &mut Ui) {
        ui.heading(t(HOME));
        if self.session_types.is_empty() {
            ui.label(t(NO_SESSION_TYPES));
        }
        let rows = self.home_rows();
        let home = &mut self.home;
        home.selected = home.selected.min(rows.len() - 1);
        let focused = ui.ctx().memory(|m| m.focused());
        if let Some(at) = focused.and_then(|f| home.items.iter().position(|item| *item == f)) {
            // The selection follows focus, Tab's too.
            home.selected = at;
            let (up, down) = ui.input_mut(|i| (i.consume_key(Modifiers::NONE, Key::ArrowUp), i.consume_key(Modifiers::NONE, Key::ArrowDown)));
            let to = if up { at.checked_sub(1) } else { Some(at + 1).filter(|to| down && *to < rows.len()) };
            if let Some(to) = to {
                home.selected = to;
                home.focus = true;
            }
        }
        let mut open = None;
        ui.scope(|ui| {
            ui.ctx().accesskit_node_builder(ui.unique_id(), |node| {
                node.set_role(Role::Tree);
                node.set_label(t(SESSIONS));
            });
            home.items.clear();
            for (index, (level, name, target)) in rows.into_iter().enumerate() {
                let selected = index == home.selected;
                let response = ui
                    .push_id(index, |ui| {
                        ui.horizontal(|ui| {
                            ui.add_space(ui.spacing().indent * (level - 1) as f32);
                            ui.selectable_label(selected, name)
                        })
                        .inner
                    })
                    .inner;
                let branch = matches!(target, Target::Branch);
                ui.ctx().accesskit_node_builder(response.id, |node| {
                    node.set_role(Role::TreeItem);
                    node.clear_toggled();
                    node.set_selected(selected);
                    node.set_level(level);
                    if branch {
                        // ponytail: branches are always open; collapsing them comes when a branch
                        // grows long enough to need it.
                        node.set_expanded(true);
                    }
                });
                if selected && home.focus {
                    response.request_focus();
                }
                // The tree, not egui's arrow-key focus moves, takes Up and Down while a row has focus.
                let keeps_arrows = egui::EventFilter { vertical_arrows: true, ..Default::default() };
                ui.memory_mut(|m| m.set_focus_lock_filter(response.id, keeps_arrows));
                if response.clicked() {
                    home.selected = index;
                }
                let enter = response.has_focus() && ui.input(|i| i.key_pressed(Key::Enter));
                if (response.double_clicked() || enter) && !branch {
                    open = Some(target);
                }
                home.items.push(response.id);
            }
            home.focus = false;
        });
        if let Some(error) = &home.error {
            let label = ui.label(error);
            // Announced, since focus stays on the row.
            ui.ctx().accesskit_node_builder(label.id, |node| node.set_live(Live::Polite));
        }
        let Some(target) = open else { return };
        let result = match target {
            Target::Branch => Ok(()),
            Target::New(type_id) => self.new_session(type_id).map_err(|e| CommandError::Failed(e.to_string())),
            Target::Saved(name) => self.open_saved_session(&name),
            Target::AutoSaved(index) => match self.home.auto_saved.get(index).cloned() {
                Some(session) => self.restore_session(session).map_err(|e| CommandError::Failed(e.to_string())),
                None => Ok(()),
            },
        };
        self.home.error = result.err().map(|e| e.to_string());
    }
}

/// Open Saved Session, with the session's name: what double-clicking it on Home does, for
/// automation. Not in a menu, which reaches the same through Open Session and Home.
pub(crate) fn commands() -> Vec<Command<Shell>> {
    const NAME: &[Param] = &[Param::required("name", Kind::Text)];
    vec![
        Command::typed("otd.open_saved_session", OPEN_SAVED_SESSION, NAME, |shell: &mut Shell, args: &Args| -> Outcome {
            shell.open_saved_session(args.text("name").unwrap_or_default()).map(|()| None)
        })
        .icon("folder-open"),
    ]
}
