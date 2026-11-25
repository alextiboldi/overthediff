//! The shell's side of background tasks (E01-S1-T8): a progress window with Cancel for each running
//! task, and an end-of-task summary listing what failed.
//!
//! A feature starts its work with `otd_core::task::Task::spawn` and hands a clone of the handle to
//! [`show_task`], from a view's `ui` or anywhere else it has the window's `egui::Context`. It keeps
//! its own clone to read the [`otd_core::task::Outcome`] when the task ends.

use std::collections::VecDeque;
use std::time::Duration;

use egui::{Context, Id};
use otd_core::task::{End, Outcome, Task};

// ponytail: English literals until E01-S1-T13's translation lookup.
pub const CANCEL: &str = "Cancel";
pub const CLOSE: &str = "Close";
pub const CANCELLED: &str = "Cancelled.";
pub const FINISHED_WITH_ERRORS: &str = "Finished with errors.";

/// Where [`show_task`] leaves tasks for the shell to pick up on its next frame.
const NEW_TASKS: &str = "otd-new-tasks";

/// Shows `task` in the main window: its progress, with Cancel if it is cancellable, and once it
/// ends cancelled or with errors, a summary of every failed item. A task that completes cleanly
/// just closes its progress window; the feature reports its own result.
pub fn show_task(ctx: &Context, task: Task) {
    ctx.data_mut(|data| data.get_temp_mut_or_default::<Vec<Task>>(Id::unique(NEW_TASKS)).push(task));
    ctx.request_repaint();
}

/// A task the shell shows, and where focus was when it started, to give it back after the summary.
#[derive(Clone)]
struct Shown {
    task: Task,
    focus_back: Option<Id>,
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
        self.running.extend(new.into_iter().map(|task| Shown { task, focus_back: focused }));

        let summaries = &mut self.summaries;
        self.running.retain(|shown| match shown.task.outcome() {
            None => true,
            Some(outcome) => {
                if outcome.end != End::Completed {
                    summaries.push_back(Summary { shown: shown.clone(), outcome, drawn: false });
                }
                false
            }
        });
        for shown in &self.running {
            progress_window(ctx, &shown.task);
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

fn progress_window(ctx: &Context, task: &Task) {
    egui::Window::new(task.title()).id(Id::unique(("otd-task", task.id()))).collapsible(false).resizable(false).show(ctx, |ui| {
        let progress = task.progress();
        // No `animate` while the total is unknown: an animation repaints every frame.
        let (fraction, text) = match progress.total {
            Some(total) => (progress.done as f32 / total.max(1) as f32, format!("{} of {total}", progress.done)),
            None => (0.0, format!("{} done", progress.done)),
        };
        ui.add(egui::ProgressBar::new(fraction).text(text));
        // Disabled once asked, until the task reaches its next safe point and the window closes.
        if task.cancellable() && ui.add_enabled(!task.cancel_requested(), egui::Button::new(CANCEL)).clicked() {
            task.cancel();
        }
    });
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
        ui.label(if summary.outcome.end == End::Cancelled { CANCELLED } else { FINISHED_WITH_ERRORS });
        let errors = &summary.outcome.errors;
        if !errors.is_empty() {
            ui.label(if errors.len() == 1 { "1 item failed:".to_owned() } else { format!("{} items failed:", errors.len()) });
            egui::ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
                for error in errors {
                    ui.label(format!("{}: {}", error.item, error.message));
                }
            });
        }
        let close = ui.button(CLOSE);
        if !summary.drawn {
            close.request_focus();
        }
        close.clicked()
    });
    summary.drawn = true;
    response.inner || response.should_close()
}
