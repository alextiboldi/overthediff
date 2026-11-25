//! The shell's side of background tasks (E01-S1-T8): a progress window with Cancel for each running
//! task, and an end-of-task summary listing what failed.
//!
//! A feature starts its work with `otd_core::task::Task::spawn` and hands a clone of the handle to
//! [`show_task`] once, when it starts the task, from wherever it has the window's `egui::Context`.
//! It keeps its own clone to read the [`otd_core::task::Outcome`] when the task ends. The task's
//! title and its items' messages are text the feature has already looked up
//! (`otd_core::i18n::t`); the shell draws them as given.

use std::collections::VecDeque;
use std::time::Duration;

use egui::{Context, Id};
use otd_core::i18n::{fmt, t, trn};
use otd_core::task::{End, Outcome, Task};

// English keys, drawn through `otd_core::i18n`.
pub const CANCEL: &str = "Cancel";
pub const CLOSE: &str = "Close";
pub const CANCELLED: &str = "Cancelled.";
pub const FINISHED_WITH_ERRORS: &str = "Finished with errors.";
const DONE_OF_TOTAL: &str = "{done} of {total}";
const DONE: &str = "{done} done";
const ONE_FAILED: &str = "{n} item failed:";
const MANY_FAILED: &str = "{n} items failed:";
const ITEM_ERROR: &str = "{item}: {message}";

/// Where [`show_task`] leaves tasks for the shell to pick up on its next frame.
const NEW_TASKS: &str = "otd-new-tasks";

/// Shows `task` in the main window: its progress, with Cancel if it is cancellable, and once it
/// ends cancelled or with errors, a summary of every failed item. A task that completes cleanly
/// just closes its progress window; the feature reports its own result.
///
/// Call it once per task, when the task starts, not every frame: a task already showing is
/// ignored, but one whose summary has closed would show its summary again.
pub fn show_task(ctx: &Context, task: Task) {
    ctx.data_mut(|data| data.get_temp_mut_or_default::<Vec<Task>>(Id::unique(NEW_TASKS)).push(task));
    ctx.request_repaint();
}

/// A task the shell shows, and where focus was when it started, to give it back after the summary
/// or, if focus was in its progress window, when that window closes.
#[derive(Clone)]
struct Shown {
    task: Task,
    focus_back: Option<Id>,
    /// Its Cancel button had focus last frame.
    has_focus: bool,
}

/// An ended task's summary, waiting to show or showing.
struct Summary {
    shown: Shown,
    outcome: Outcome,
    /// The dialog has been drawn, so its Close button already took focus.
    drawn: bool,
}

#[derive(Default)]
pub(crate) struct Tasks {
    running: Vec<Shown>,
    /// One summary at a time, the oldest first.
    summaries: VecDeque<Summary>,
    /// Where focus goes back to once the closed summary's modal layer has gone, a frame later:
    /// widgets behind a modal layer can't take focus.
    focus_back: Option<Id>,
}

impl Tasks {
    pub(crate) fn ui(&mut self, ctx: &Context) {
        if let Some(id) = self.focus_back.take() {
            ctx.memory_mut(|memory| memory.request_focus(id));
            ctx.request_repaint();
        }
        let new = ctx.data_mut(|data| data.remove_temp::<Vec<Task>>(Id::unique(NEW_TASKS))).unwrap_or_default();
        let focused = ctx.memory(|memory| memory.focused());
        for task in new {
            // The same handle twice would draw two windows with one id.
            let mut showing = self.running.iter().chain(self.summaries.iter().map(|summary| &summary.shown));
            if !showing.any(|shown| shown.task.id() == task.id()) {
                self.running.push(Shown { task, focus_back: focused, has_focus: false });
            }
        }

        let (summaries, focus_back) = (&mut self.summaries, &mut self.focus_back);
        self.running.retain(|shown| match shown.task.outcome() {
            None => true,
            Some(outcome) => {
                if outcome.end != End::Completed {
                    summaries.push_back(Summary { shown: shown.clone(), outcome, drawn: false });
                } else if shown.has_focus || focused.is_none() {
                    // Its window closes with focus in it, or focus was lost when Cancel was
                    // disabled: give focus back as the summary would.
                    *focus_back = shown.focus_back;
                    ctx.request_repaint();
                }
                false
            }
        });
        if !self.summaries.is_empty() {
            // A summary takes focus itself, and gives it back when it closes.
            self.focus_back = None;
        }
        for shown in &mut self.running {
            shown.has_focus = progress_window(ctx, &shown.task);
        }
        if !self.running.is_empty() {
            // ponytail: the shell polls running tasks ten times a second; a waker from the task
            // thread would repaint only on change, if this ever shows up in a profile.
            ctx.request_repaint_after(Duration::from_millis(100));
        }

        let Some(summary) = self.summaries.front_mut() else { return };
        if summary_dialog(ctx, summary) {
            let focus_back = summary.shown.focus_back;
            self.summaries.pop_front();
            // The next summary takes focus itself.
            if self.summaries.is_empty() {
                self.focus_back = focus_back;
            }
            ctx.request_repaint();
        }
    }
}

/// Draws a task's progress window; true if its Cancel button has focus.
fn progress_window(ctx: &Context, task: &Task) -> bool {
    let window = egui::Window::new(task.title()).id(Id::unique(("otd-task", task.id()))).collapsible(false).resizable(false).show(ctx, |ui| {
        let progress = task.progress();
        // No `animate` while the total is unknown: an animation repaints every frame.
        let (fraction, text) = match progress.total {
            Some(total) => {
                let text = fmt(t(DONE_OF_TOTAL), &[("done", &progress.done.to_string()), ("total", &total.to_string())]);
                (progress.done as f32 / total.max(1) as f32, text)
            }
            None => (0.0, fmt(t(DONE), &[("done", &progress.done.to_string())])),
        };
        ui.add(egui::ProgressBar::new(fraction).text(text));
        // Disabled once asked, until the task reaches its next safe point and the window closes.
        if !task.cancellable() {
            return false;
        }
        let cancel = ui.add_enabled(!task.cancel_requested(), egui::Button::new(t(CANCEL)));
        if cancel.clicked() {
            task.cancel();
        }
        cancel.has_focus()
    });
    window.and_then(|window| window.inner).unwrap_or(false)
}

/// Draws the summary dialog; true once it should close. It takes focus when it opens, and Esc or
/// Close closes it.
fn summary_dialog(ctx: &Context, summary: &mut Summary) -> bool {
    let title = summary.shown.task.title().to_owned();
    let response = egui::Modal::new(Id::unique(("otd-task-summary", summary.shown.task.id()))).show(ctx, |ui| {
        // The dialog's node is the modal's own `Ui`, an ancestor of this one: name it by the title.
        if let Some(dialog) = ui.stack().iter().find(|frame| frame.kind() == Some(egui::UiKind::Modal)) {
            ui.ctx().accesskit_node_builder(dialog.unique_id, |node| {
                node.set_label(title.as_str());
                node.set_modal();
            });
        }
        ui.heading(&title);
        ui.label(t(if summary.outcome.end == End::Cancelled { CANCELLED } else { FINISHED_WITH_ERRORS }));
        let errors = &summary.outcome.errors;
        if !errors.is_empty() {
            ui.label(trn(errors.len() as u64, ONE_FAILED, MANY_FAILED));
            egui::ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
                for error in errors {
                    ui.label(fmt(t(ITEM_ERROR), &[("item", &error.item), ("message", &error.message)]));
                }
            });
        }
        let close = ui.button(t(CLOSE));
        if !summary.drawn {
            close.request_focus();
        }
        close.clicked()
    });
    summary.drawn = true;
    response.inner || response.should_close()
}
