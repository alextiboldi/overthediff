//! Unsaved-changes prompts on Close Tab and Exit (E01-S1-T10).
//!
//! A view opts in with [`View::unsaved`] and [`View::save`]. Closing its tab, or exiting, asks
//! Save / Don't Save / Cancel in a modal dialog over the window, one view at a time. Cancel aborts
//! the whole close or exit and leaves every view as it was.

use egui::accesskit::{Live, Role};
use egui::{Context, Id, ViewportCommand};

use otd_core::i18n::{fmt, t};

use crate::{CANCEL, Shell, Tab, View, title};

// The prompt's text: English keys, drawn only through `t` (E01-S1-T13).
/// The dialog's heading and name; `{title}` is the view's title.
pub const SAVE_CHANGES: &str = "Save changes to {title}?";
pub const UNSAVED_LOST: &str = "Your changes will be lost if you don't save them.";
pub const SAVE: &str = "Save";
pub const DONT_SAVE: &str = "Don't Save";
pub const CANT_SAVE: &str = "This view can't save its changes.";
pub const NOT_SAVED: &str = "The changes weren't saved.";

/// An open prompt: the tabs it still has to ask about, the one it is asking about first.
pub(crate) struct Prompt {
    /// Exit asks about every unsaved tab before the window closes; Close Tab and Clear Session
    /// about one.
    exit: bool,
    /// Clear Session: the answered view is replaced with this empty one instead of closed.
    replace: Option<fn() -> Box<dyn View>>,
    queue: Vec<usize>,
    /// Why the last Save didn't save, shown in the dialog.
    error: Option<String>,
    /// Focus moves into the dialog on its next draw.
    focus: bool,
    /// What had focus when a Close Tab prompt opened, to give it back on Cancel. Exit gives it to
    /// the selected tab, since it may have selected another.
    return_to: Option<Id>,
}

impl Prompt {
    fn new(exit: bool, queue: Vec<usize>) -> Self {
        Self { exit, replace: None, queue, error: None, focus: true, return_to: None }
    }
}

enum Answer {
    Save,
    DontSave,
    Cancel,
}

fn unsaved(tab: &Tab) -> bool {
    tab.as_ref().is_some_and(|view| view.unsaved())
}

impl Shell {
    /// Closes the tab at `index`, first asking whether to save it when its view has unsaved
    /// changes. Does nothing while a prompt is open.
    pub(crate) fn request_close(&mut self, index: usize) {
        if self.prompt.is_some() {
            return;
        }
        if unsaved(&self.tabs[index]) {
            self.active = index;
            self.prompt = Some(Prompt::new(false, vec![index]));
        } else {
            self.close(index);
        }
    }

    /// Replaces the view at `index` with `new` (Clear Session), first asking whether to save it
    /// when it has unsaved changes. Does nothing while a prompt is open.
    pub(crate) fn request_replace(&mut self, index: usize, new: fn() -> Box<dyn View>) {
        if self.prompt.is_some() {
            return;
        }
        if unsaved(&self.tabs[index]) {
            self.prompt = Some(Prompt { replace: Some(new), ..Prompt::new(false, vec![index]) });
        } else {
            self.replace(index, new);
        }
    }

    /// Asks to exit. Returns `true` when the window may close: no view has unsaved changes, or the
    /// user has answered for each one. Otherwise it asks about each unsaved view in turn, once, and
    /// when the last is answered asks the window to close again (`ViewportCommand::Close`), when
    /// this returns `true`. Cancel stops the exit and leaves every view open.
    pub fn exit(&mut self) -> bool {
        if self.exit_confirmed {
            return true;
        }
        if self.prompt.as_ref().is_some_and(|prompt| prompt.exit) {
            return false;
        }
        let queue: Vec<usize> = (0..self.tabs.len()).filter(|&index| unsaved(&self.tabs[index])).collect();
        let Some(&first) = queue.first() else { return true };
        // An exit takes over an open Close Tab prompt, whose tab is in the queue too.
        self.active = first;
        self.prompt = Some(Prompt::new(true, queue));
        false
    }

    /// The Exit command: asks about unsaved views, then closes the window ([`Shell::exit`]).
    pub(crate) fn request_exit(&mut self) {
        self.exit_requested = true;
    }

    /// Holds a window close (the close button, the OS, Alt+F4, the Exit command)
    /// open with `ViewportCommand::CancelClose` while [`Shell::exit`] asks about unsaved views, and
    /// brings the window into view to answer. Call it each frame before [`Shell::ui`], in
    /// `eframe::App::logic`, which runs while the window is minimized and `ui` doesn't.
    pub fn on_close_request(&mut self, ctx: &Context) {
        if ctx.input(|i| i.viewport().close_requested()) && !self.exit() {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(ViewportCommand::Focus);
        }
    }

    /// Draws the open prompt, if any, and acts on its answer.
    pub(crate) fn prompt_ui(&mut self, ctx: &Context) {
        let Some(prompt) = &mut self.prompt else { return };
        let index = prompt.queue[0];
        let focus = std::mem::take(&mut prompt.focus);
        if focus && !prompt.exit && prompt.return_to.is_none() {
            prompt.return_to = ctx.memory(|memory| memory.focused());
        }
        let Some(answer) = dialog(ctx, &title(&self.tabs[index]), prompt.error.as_deref(), focus) else { return };
        match answer {
            Answer::Cancel => {
                // On the next draw: focus can't leave the dialog in the frame that closes it.
                self.refocus = self.prompt.take().and_then(|prompt| prompt.return_to);
                self.focus_active = self.refocus.is_none();
                ctx.request_repaint();
            }
            Answer::Save => {
                let error = match &mut self.tabs[index] {
                    Some(view) => save(view.as_mut()),
                    None => None,
                };
                match error {
                    Some(error) => prompt.error = Some(error),
                    None => self.answered(ctx),
                }
            }
            Answer::DontSave => self.answered(ctx),
        }
    }

    /// The prompted view is saved or its changes are given up: close its tab or clear it, or ask
    /// about the next view, or exit.
    fn answered(&mut self, ctx: &Context) {
        let Some(mut prompt) = self.prompt.take() else { return };
        if !prompt.exit {
            match prompt.replace {
                Some(new) => self.replace(prompt.queue[0], new),
                None => self.close(prompt.queue[0]),
            }
            return;
        }
        prompt.queue.remove(0);
        match prompt.queue.first() {
            Some(&next) => {
                self.active = next;
                self.prompt = Some(Prompt { error: None, focus: true, ..prompt });
            }
            None => {
                self.exit_confirmed = true;
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
        }
    }
}

/// Saves `view`, or says why it didn't.
fn save(view: &mut dyn View) -> Option<String> {
    match view.save() {
        Err(error) => Some(error),
        Ok(()) if view.unsaved() => Some(t(NOT_SAVED).to_owned()),
        Ok(()) => None,
    }
}

/// The Save / Don't Save / Cancel dialog for the view titled `title`. Esc and a click outside it are
/// Cancel. Save, the default, takes focus when `focus` is set.
fn dialog(ctx: &Context, title: &str, error: Option<&str>, focus: bool) -> Option<Answer> {
    let question = fmt(t(SAVE_CHANGES), &[("title", title)]);
    let modal = egui::Modal::new(Id::unique("otd-unsaved-prompt")).show(ctx, |ui| {
        ui.ctx().accesskit_node_builder(ui.unique_id(), |node| {
            node.set_role(Role::AlertDialog);
            node.set_label(question.as_str());
            node.set_modal();
        });
        // Exit's next question replaces this one in the same dialog, with focus kept on Save, and
        // selects that view's tab behind it: announce it (rule 21).
        let heading = ui.heading(&question);
        ui.ctx().accesskit_node_builder(heading.id, |node| node.set_live(Live::Assertive));
        ui.label(t(UNSAVED_LOST));
        if let Some(error) = error {
            let response = ui.label(error);
            ui.ctx().accesskit_node_builder(response.id, |node| node.set_live(Live::Assertive));
        }
        let mut answer = None;
        ui.horizontal(|ui| {
            let save = ui.button(t(SAVE));
            if focus {
                save.request_focus();
            }
            if save.clicked() {
                answer = Some(Answer::Save);
            }
            if ui.button(t(DONT_SAVE)).clicked() {
                answer = Some(Answer::DontSave);
            }
            if ui.button(t(CANCEL)).clicked() {
                answer = Some(Answer::Cancel);
            }
        });
        answer
    });
    if modal.inner.is_none() && modal.should_close() {
        return Some(Answer::Cancel);
    }
    modal.inner
}
