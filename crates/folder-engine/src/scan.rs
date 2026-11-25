//! Scanning one tree (E03-S2-T2): a folder is listed when it is opened, ahead of everything else,
//! and, with background scanning on, the rest of the tree is listed after it. Each listing sends a
//! [`ScanEvent`] with the progress so far.
//!
//! The scan is driven by [`TreeScan::step`], which lists one folder on the calling thread, so a
//! headless caller can run it to the end and tests can drive it one listing at a time.
//! [`TreeScan::start`] runs the same steps on a worker thread, so a view can browse while it works.
//! A comparison scans its two trees with one [`TreeScan`] each.
//!
//! [`ScanState`] is the state of a folder's own listing. Whether a whole subtree is done is for the
//! caller to derive from its folders (aggregation, E03-S2-T6).
//!
//! ponytail: links are reported, not followed (following them is E08-S11-T5), and expandable
//! containers are listed only when opened (E03-S2-T9). Rescanning is E08-S1-T6.

use std::collections::{HashMap, VecDeque};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::thread;

use otd_source::{DataSource, Item, ItemKind, Links, SourceError};

/// How far a folder's own listing has got.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanState {
    /// Known from its parent's listing, and not yet asked for: background scanning is off and
    /// nobody has opened it.
    Unscanned,
    /// Waiting to be listed, or being listed.
    Pending,
    /// Listed, or failed to list, with the error kept.
    Done,
}

/// A folder of the tree, as far as it has been scanned.
#[derive(Debug, Clone, PartialEq)]
pub struct Folder {
    pub state: ScanState,
    /// Its children, once [`ScanState::Done`]. Items the source couldn't read carry their error.
    pub items: Vec<Item>,
    /// Why the folder couldn't be listed. It is then done, with no items.
    pub error: Option<SourceError>,
}

/// Sent each time a folder's listing finishes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanEvent {
    /// The folder just listed, relative to the base.
    pub folder: PathBuf,
    /// Folders done so far.
    pub done: usize,
    /// Folders still waiting or being listed. Zero means the scan has nothing left to do until
    /// another folder is opened.
    pub pending: usize,
}

/// One tree's scan: a source and the base folder in it. Paths it takes and reports are relative to
/// the base, and the empty path is the base itself. Dropping it stops its workers after the
/// listing they are on.
pub struct TreeScan {
    shared: Arc<Shared>,
}

struct Shared {
    source: Arc<dyn DataSource>,
    base: PathBuf,
    background: bool,
    state: Mutex<State>,
    wake: Condvar,
}

struct State {
    folders: HashMap<PathBuf, Folder>,
    /// Pending folders not yet taken by a step: opened ones at the front, background ones behind.
    queue: VecDeque<PathBuf>,
    done: usize,
    pending: usize,
    events: Sender<ScanEvent>,
    stopped: bool,
}

impl TreeScan {
    /// A scan of `base` in `source`, with the base already opened. With `background` on, every
    /// folder found is queued behind the opened ones; with it off, a folder is listed only once
    /// opened. Nothing is listed until [`TreeScan::step`] or [`TreeScan::start`].
    pub fn new(source: Arc<dyn DataSource>, base: impl Into<PathBuf>, background: bool) -> (Self, Receiver<ScanEvent>) {
        let (events, receiver) = mpsc::channel();
        let state = State { folders: HashMap::new(), queue: VecDeque::new(), done: 0, pending: 0, events, stopped: false };
        let scan = Self { shared: Arc::new(Shared { source, base: base.into(), background, state: Mutex::new(state), wake: Condvar::new() }) };
        scan.open(Path::new(""));
        (scan, receiver)
    }

    /// Asks for `folder` to be listed before anything else still waiting. A folder already done,
    /// or being listed, is left alone. Any listable path can be opened, including one no listing
    /// has reported yet (an expandable container, say).
    pub fn open(&self, folder: &Path) {
        let mut state = self.shared.lock();
        let state = &mut *state;
        match state.folders.get_mut(folder).map(|f| f.state) {
            Some(ScanState::Done) => return,
            Some(ScanState::Pending) => {
                // Not in the queue means a step is listing it now.
                let Some(at) = state.queue.iter().position(|queued| queued == folder) else { return };
                state.queue.remove(at);
            }
            Some(ScanState::Unscanned) | None => {
                state.folders.insert(folder.to_path_buf(), Folder { state: ScanState::Pending, items: Vec::new(), error: None });
                state.pending += 1;
            }
        }
        state.queue.push_front(folder.to_path_buf());
        self.shared.wake.notify_all();
    }

    /// The folder as scanned so far, or `None` if no listing has reported it and nobody opened it.
    pub fn folder(&self, folder: &Path) -> Option<Folder> {
        self.shared.lock().folders.get(folder).cloned()
    }

    /// Lists the next waiting folder on this thread. Returns `false` when nothing was waiting.
    pub fn step(&self) -> bool {
        self.shared.step()
    }

    /// Starts a worker thread that runs [`TreeScan::step`] until the scan is dropped, waiting
    /// whenever nothing is pending. Each call adds one worker.
    pub fn start(&self) -> io::Result<()> {
        let shared = Arc::clone(&self.shared);
        thread::Builder::new().name("otd-tree-scan".into()).spawn(move || shared.work()).map(drop)
    }
}

impl Drop for TreeScan {
    fn drop(&mut self) {
        self.shared.lock().stopped = true;
        self.shared.wake.notify_all();
    }
}

impl Shared {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn step(&self) -> bool {
        let Some(folder) = self.lock().queue.pop_front() else { return false };
        // The lock is not held while the source lists, so opening and reading go on meanwhile.
        let (items, error) = match self.source.list(&self.base.join(&folder), Links::Report) {
            Ok(items) => (items, None),
            Err(error) => (Vec::new(), Some(error)),
        };
        let mut state = self.lock();
        let state = &mut *state;
        for item in items.iter().filter(|item| item.kind == ItemKind::Folder) {
            let child = folder.join(&item.name);
            if state.folders.contains_key(&child) {
                continue;
            }
            let found = if self.background {
                state.queue.push_back(child.clone());
                state.pending += 1;
                ScanState::Pending
            } else {
                ScanState::Unscanned
            };
            state.folders.insert(child, Folder { state: found, items: Vec::new(), error: None });
        }
        state.folders.insert(folder.clone(), Folder { state: ScanState::Done, items, error });
        state.pending = state.pending.saturating_sub(1);
        state.done += 1;
        // Nobody listening is fine: the scan's results stay readable through `folder`.
        let _ = state.events.send(ScanEvent { folder, done: state.done, pending: state.pending });
        true
    }

    fn work(&self) {
        loop {
            {
                let mut state = self.lock();
                while state.queue.is_empty() && !state.stopped {
                    state = self.wake.wait(state).unwrap_or_else(PoisonError::into_inner);
                }
                if state.stopped {
                    return;
                }
            }
            self.step();
        }
    }
}
