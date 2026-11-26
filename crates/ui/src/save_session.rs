//! Save Session and Save Session As (E06-S1-T2): saving the active view's session as a named
//! session in the settings store (`otd_settings::sessions`).
//!
//! Save Session As asks for a name in a dialog, and asks again before replacing a session already
//! saved under that name (Cancel goes back to the name). Save Session saves a named session in
//! place, and is Save Session As for an unnamed one. A failed save reopens the name dialog with the
//! reason, announced. A save announces itself, since it changes nothing on screen.
//!
//! The commands only open the dialog or mark the save as due; [`Shell::sessions_ui`], which the
//! app calls with its settings store after [`Shell::ui`], draws the dialog and writes.

use egui::accesskit::{Live, Role};
use egui::{Context, Id, Key, Modifiers, TextEdit};
use otd_core::i18n::{fmt, t};
use otd_settings::{Store, sessions};

use crate::{CANCEL, SAVE, SessionError, Shell};

// The dialogs' text: English keys, drawn only through `t`.
pub const SAVE_SESSION: &str = "Save Session";
pub const SAVE_SESSION_AS: &str = "Save Session As";
pub const SESSION_NAME: &str = "Name";
/// The overwrite question; `{name}` is the name typed.
pub const REPLACE_SESSION: &str = "A session named \"{name}\" is already saved. Replace it?";
pub const REPLACE: &str = "Replace";
/// The announcement after a save; `{name}` is the session's name.
pub const SESSION_SAVED: &str = "Saved the session \"{name}\".";

/// A save under way, of the session in the tab `tab`.
pub(crate) struct Saving {
    /// Tabs only open at the end, and closing one drops this, so the index stays put.
    tab: usize,
    name: String,
    step: Step,
    /// Why the last attempt didn't save, shown in the name dialog.
    error: Option<String>,
    /// Focus moves into the dialog on its next draw.
    focus: bool,
    /// A dialog was shown, so focus goes back to the window once the save is done.
    dialog: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum Step {
    /// The name dialog is open.
    Name,
    /// The name is taken: asking whether to replace it.
    Replace,
    /// Writes on the next [`Shell::sessions_ui`]: Save Session on a named session, or confirmed.
    Write,
}

/// The last save's announcement, and the tab it was for: it goes when another tab is selected or
/// any key or button is pressed.
pub(crate) struct Saved {
    tab: usize,
    text: String,
    /// It stays the frame it is first shown, whose input caused it.
    shown: bool,
}

impl Shell {
    /// Save Session: saves the active view's named session in place, or asks for a name, as
    /// [`Shell::save_session_as`], when it has none.
    pub fn save_session(&mut self) -> Result<(), SessionError> {
        match self.active_session().ok_or(SessionError::NoSession)?.name.clone() {
            Some(name) => self.saving = Some(Saving { tab: self.active, name, step: Step::Write, error: None, focus: false, dialog: false }),
            None => self.save_session_as()?,
        }
        Ok(())
    }

    /// Save Session As: asks for a name, starting from the session's own, then saves under it.
    pub fn save_session_as(&mut self) -> Result<(), SessionError> {
        let name = self.active_session().ok_or(SessionError::NoSession)?.name.clone().unwrap_or_default();
        self.saving = Some(Saving { tab: self.active, name, step: Step::Name, error: None, focus: true, dialog: true });
        self.saved = None;
        Ok(())
    }

    /// Draws the Save Session As dialog and the save announcement, and saves into `store` what
    /// Save Session or the dialog asked for. Call it each frame after [`Shell::ui`].
    pub fn sessions_ui(&mut self, ctx: &Context, store: &Store) {
        self.saved_ui(ctx);
        let Some(saving) = &mut self.saving else { return };
        match saving.step {
            Step::Name => match name_dialog(ctx, saving) {
                Some(true) if sessions::exists(store, &saving.name) => {
                    saving.step = Step::Replace;
                    saving.focus = true;
                }
                Some(true) => saving.step = Step::Write,
                Some(false) => self.close_saving(ctx),
                None => {}
            },
            Step::Replace => match replace_dialog(ctx, saving) {
                Some(true) => saving.step = Step::Write,
                Some(false) => {
                    saving.step = Step::Name;
                    saving.focus = true;
                }
                None => {}
            },
            Step::Write => {}
        }
        if self.saving.as_ref().is_some_and(|s| s.step == Step::Write) {
            self.write(ctx, store);
        }
    }

    /// Saves the session as the dialog or Save Session left it; a failure reopens the name dialog
    /// with the reason.
    fn write(&mut self, ctx: &Context, store: &Store) {
        let Some(saving) = &self.saving else { return };
        let (tab, name) = (saving.tab, saving.name.clone());
        let Some(session) = self.tabs.get_mut(tab).and_then(|t| t.as_mut()?.session()) else {
            self.saving = None;
            return;
        };
        match sessions::save(store, &name, session) {
            Ok(()) => {
                session.name = Some(name.clone());
                self.close_saving(ctx);
                self.saved = Some(Saved { tab, text: fmt(t(SESSION_SAVED), &[("name", &name)]), shown: false });
            }
            Err(error) => {
                if let Some(saving) = &mut self.saving {
                    saving.error = Some(error);
                    saving.step = Step::Name;
                    saving.focus = true;
                    saving.dialog = true;
                }
            }
        }
        ctx.request_repaint();
    }

    fn close_saving(&mut self, ctx: &Context) {
        // A Save Session with no dialog leaves focus where it was.
        if self.saving.take().is_some_and(|s| s.dialog) {
            self.focus_active = true;
        }
        ctx.request_repaint();
    }

    /// The save announcement, at the bottom of the window over its contents.
    fn saved_ui(&mut self, ctx: &Context) {
        let Some(saved) = &mut self.saved else { return };
        let pressed = ctx.input(|i| {
            i.events.iter().any(|e| matches!(e, egui::Event::Key { pressed: true, .. } | egui::Event::PointerButton { pressed: true, .. }))
        });
        if saved.tab != self.active || (saved.shown && pressed) {
            self.saved = None;
            return;
        }
        saved.shown = true;
        // Above the status bar, which is as tall as a control and its spacing.
        let spacing = ctx.global_style().spacing.clone();
        let above = 2.0 * (spacing.interact_size.y + 2.0 * spacing.item_spacing.y);
        egui::Area::new(Id::unique("otd-session-saved")).anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -above)).interactable(false).show(
            ctx,
            |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    let label = ui.label(&saved.text);
                    ui.ctx().accesskit_node_builder(label.id, |node| node.set_live(Live::Polite));
                });
            },
        );
    }
}

/// The name dialog: `Some(true)` on Save or Enter, `Some(false)` on Cancel or Esc.
fn name_dialog(ctx: &Context, saving: &mut Saving) -> Option<bool> {
    let id = Id::unique("otd-save-session-as");
    let focus = std::mem::take(&mut saving.focus);
    let modal = egui::Modal::new(id).show(ctx, |ui| {
        ui.ctx().accesskit_node_builder(id, |node| node.set_label(t(SAVE_SESSION_AS)));
        ui.heading(t(SAVE_SESSION_AS));
        let edit = ui
            .horizontal(|ui| {
                let label = ui.label(t(SESSION_NAME));
                ui.add(TextEdit::singleline(&mut saving.name)).labelled_by(label.id)
            })
            .inner;
        if focus {
            edit.request_focus();
        }
        if let Some(error) = &saving.error {
            let label = ui.label(error);
            ui.ctx().accesskit_node_builder(label.id, |node| node.set_live(Live::Assertive));
        }
        let enter = edit.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
        let mut answer = enter.then_some(true);
        ui.horizontal(|ui| {
            if ui.button(t(SAVE)).clicked() {
                answer = Some(true);
            }
            if ui.button(t(CANCEL)).clicked() {
                answer = Some(false);
            }
        });
        answer
    });
    // Esc is Cancel; a click outside does nothing, so a stray click keeps what was typed.
    let escape = modal.is_top_modal && !modal.any_popup_open && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape));
    if escape { Some(false) } else { modal.inner }
}

/// Asks before replacing the session saved under the name: `Some(true)` on Replace, `Some(false)`
/// on Cancel, Esc or a click outside. Cancel, the safe answer, takes focus.
fn replace_dialog(ctx: &Context, saving: &mut Saving) -> Option<bool> {
    let question = fmt(t(REPLACE_SESSION), &[("name", &saving.name)]);
    let focus = std::mem::take(&mut saving.focus);
    let modal = egui::Modal::new(Id::unique("otd-replace-session")).show(ctx, |ui| {
        ui.ctx().accesskit_node_builder(ui.unique_id(), |node| {
            node.set_role(Role::AlertDialog);
            node.set_label(question.as_str());
            node.set_modal();
        });
        let heading = ui.label(&question);
        ui.ctx().accesskit_node_builder(heading.id, |node| node.set_live(Live::Assertive));
        let mut answer = None;
        ui.horizontal(|ui| {
            if ui.button(t(REPLACE)).clicked() {
                answer = Some(true);
            }
            let cancel = ui.button(t(CANCEL));
            if focus {
                cancel.request_focus();
            }
            if cancel.clicked() {
                answer = Some(false);
            }
        });
        answer
    });
    if modal.inner.is_none() && modal.should_close() { Some(false) } else { modal.inner }
}
