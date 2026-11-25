//! Background tasks (E01-S1-T8): work that runs off the UI thread, reports progress, stops at its
//! next safe point when cancelled, and ends with every per-item error it met.
//!
//! A feature starts one with [`Task::spawn`] and keeps the returned handle. In the window it passes
//! a clone to `otd_ui::show_task`, which shows progress with Cancel and the end-of-task summary;
//! without a window it calls [`Task::wait`] for the [`Outcome`].

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};

// ponytail: English literals until E01-S1-T13's translation lookup.
const STOPPED_UNEXPECTEDLY: &str = "The task stopped unexpectedly.";
const COULD_NOT_START: &str = "The task couldn't start";

/// How far a task has got. `total` is `None` while the task doesn't know it yet.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Progress {
    pub done: u64,
    pub total: Option<u64>,
}

/// One item the task couldn't process, such as a file it couldn't copy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemError {
    /// What failed, such as a relative path.
    pub item: String,
    /// Why, in words for the user.
    pub message: String,
}

/// How a task ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum End {
    /// It ran to the end and every item succeeded.
    Completed,
    /// It saw a cancel request at a safe point and stopped there.
    Cancelled,
    /// It ran to the end, but some items failed, or the task itself stopped unexpectedly.
    Failed,
}

/// A finished task's result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub end: End,
    /// Every item error, in the order the task reported them.
    pub errors: Vec<ItemError>,
    /// The last progress the task reported.
    pub progress: Progress,
}

#[derive(Default)]
struct State {
    progress: Progress,
    errors: Vec<ItemError>,
    outcome: Option<Outcome>,
}

struct Shared {
    id: u64,
    title: String,
    cancellable: bool,
    cancel_requested: AtomicBool,
    /// The work saw the cancel request at a safe point, so it stopped early.
    cancel_seen: AtomicBool,
    state: Mutex<State>,
    ended: Condvar,
}

impl Shared {
    fn state(&self) -> MutexGuard<'_, State> {
        // The lock is never held while the task's own code runs, so a poisoned lock still holds
        // whole values.
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn finish(&self, stopped_unexpectedly: Option<String>) {
        let mut state = self.state();
        if let Some(message) = stopped_unexpectedly {
            state.errors.push(ItemError { item: self.title.clone(), message });
        }
        let end = if self.cancel_seen.load(Ordering::SeqCst) {
            End::Cancelled
        } else if state.errors.is_empty() {
            End::Completed
        } else {
            End::Failed
        };
        let outcome = Outcome { end, errors: std::mem::take(&mut state.errors), progress: state.progress };
        state.outcome = Some(outcome);
        self.ended.notify_all();
    }
}

/// A handle to a background task. Clones refer to the same task.
#[derive(Clone)]
pub struct Task {
    shared: Arc<Shared>,
}

/// What the task's work reports through, on its own thread.
pub struct Worker {
    shared: Arc<Shared>,
}

impl Task {
    /// Runs `work` on a new thread and returns at once. A `cancellable` task offers Cancel; `work`
    /// stops when [`Worker::cancelled`] returns true. A panic in `work` ends the task as
    /// [`End::Failed`] instead of taking the app down.
    pub fn spawn(title: impl Into<String>, cancellable: bool, work: impl FnOnce(&Worker) + Send + 'static) -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let shared = Arc::new(Shared {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            title: title.into(),
            cancellable,
            cancel_requested: AtomicBool::new(false),
            cancel_seen: AtomicBool::new(false),
            state: Mutex::new(State::default()),
            ended: Condvar::new(),
        });
        let worker = Worker { shared: Arc::clone(&shared) };
        let started = std::thread::Builder::new().name(format!("otd-task-{}", shared.id)).spawn(move || {
            let result = catch_unwind(AssertUnwindSafe(|| work(&worker)));
            worker.shared.finish(result.err().map(|_| STOPPED_UNEXPECTEDLY.to_owned()));
        });
        if let Err(err) = started {
            shared.finish(Some(format!("{COULD_NOT_START}: {err}")));
        }
        Self { shared }
    }

    /// Unique among the tasks of this process.
    pub fn id(&self) -> u64 {
        self.shared.id
    }

    pub fn title(&self) -> &str {
        &self.shared.title
    }

    pub fn cancellable(&self) -> bool {
        self.shared.cancellable
    }

    pub fn progress(&self) -> Progress {
        self.shared.state().progress
    }

    /// Asks the task to stop at its next safe point. Does nothing for a task that isn't cancellable.
    pub fn cancel(&self) {
        if self.shared.cancellable {
            self.shared.cancel_requested.store(true, Ordering::SeqCst);
        }
    }

    pub fn cancel_requested(&self) -> bool {
        self.shared.cancel_requested.load(Ordering::SeqCst)
    }

    /// The result, once the task has ended; `None` while it runs.
    pub fn outcome(&self) -> Option<Outcome> {
        self.shared.state().outcome.clone()
    }

    /// Blocks until the task ends and returns its result.
    pub fn wait(&self) -> Outcome {
        let mut state = self.shared.state();
        loop {
            if let Some(outcome) = &state.outcome {
                return outcome.clone();
            }
            state = self.shared.ended.wait(state).unwrap_or_else(PoisonError::into_inner);
        }
    }
}

impl Worker {
    /// Reports how far the task has got.
    pub fn progress(&self, done: u64, total: Option<u64>) {
        self.shared.state().progress = Progress { done, total };
    }

    /// Records an item that failed. The task carries on; the error is listed when it ends.
    pub fn fail(&self, item: impl Into<String>, message: impl Into<String>) {
        self.shared.state().errors.push(ItemError { item: item.into(), message: message.into() });
    }

    /// The safe-point check: true once Cancel was asked for, and the work should stop now. A task
    /// that stops on it ends as [`End::Cancelled`]; one that never asks ends normally.
    pub fn cancelled(&self) -> bool {
        let cancelled = self.shared.cancel_requested.load(Ordering::SeqCst);
        if cancelled {
            self.shared.cancel_seen.store(true, Ordering::SeqCst);
        }
        cancelled
    }
}
