//! Tree expansion (E08-S1-T3): which folders are open, the rows that makes [`FolderCompare::shown`],
//! and the commands that open and close them.
//!
//! A folder is open by its row's key, the path of `Row::name`s from the base folders (the paths
//! `resolve_selection` takes), so a row stays open while the rows around it change. Opening a
//! folder asks each side's scan to list that side's folder, in its own spelling; its rows show
//! once the listing arrives. Open Subfolders and Expand All also keep a root whose subtree goes on
//! opening as listings arrive, until nothing below it is left to list.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use otd_core::command::{Command, State};
use otd_folder_engine::{Row, ScanState};
use otd_source::{Item, ItemKind};
use otd_ui::Shell;

use crate::{FolderCompare, ID};

// English keys, drawn only through `t`.
pub const OPEN_FOLDER: &str = "Open Folder";
pub const CLOSE_FOLDER: &str = "Close Folder";
pub const OPEN_SUBFOLDERS: &str = "Open Subfolders";
pub const CLOSE_SUBFOLDERS: &str = "Close Subfolders";
pub const EXPAND_ALL: &str = "Expand All";
pub const COLLAPSE_ALL: &str = "Collapse All";

/// A row drawn, with where it sits in the tree.
#[derive(Debug, Clone)]
pub struct Shown<'a> {
    pub row: &'a Row,
    /// 0 for a row of the base folders.
    pub depth: usize,
    /// The path of `Row::name`s from the base folders.
    pub key: PathBuf,
    /// Each side's folder of this row, relative to that side's base in that side's spelling;
    /// `None` where that side's item isn't a folder.
    pub folders: [Option<PathBuf>; 2],
}

impl Shown<'_> {
    /// Whether either side's item is a folder, so the row can open.
    pub fn is_folder(&self) -> bool {
        self.folders.iter().any(Option::is_some)
    }
}

/// The open folders.
#[derive(Debug, Default)]
pub(crate) struct Expansion {
    open: BTreeSet<PathBuf>,
    /// Roots whose subtrees go on opening as their listings arrive: Open Subfolders' folder, or the
    /// empty path (every row) for Expand All.
    deep: Vec<PathBuf>,
}

impl Expansion {
    pub(crate) fn is_open(&self, key: &Path) -> bool {
        self.open.contains(key)
    }

    /// Stops opening below and above `key`, so a late listing doesn't open what was just closed.
    fn stop_deep(&mut self, key: &Path) {
        self.deep.retain(|root| !(root.starts_with(key) || key.starts_with(root)));
    }
}

/// `rows` in tree order, going into the folders `open` says are open.
pub(crate) fn flatten<'a>(rows: &'a [Row], open: &dyn Fn(&Path) -> bool) -> Vec<Shown<'a>> {
    let mut shown = Vec::new();
    let base = Some(PathBuf::new());
    walk(rows, Path::new(""), [&base, &base], 0, open, &mut shown);
    shown
}

// ponytail: recursive, as deep as the tree, like `compare_trees`.
fn walk<'a>(rows: &'a [Row], parent: &Path, dirs: [&Option<PathBuf>; 2], depth: usize, open: &dyn Fn(&Path) -> bool, shown: &mut Vec<Shown<'a>>) {
    for row in rows {
        let key = parent.join(&row.name);
        // Each side descends by its own item's name, which may differ in case.
        let folder =
            |dir: &Option<PathBuf>, item: &Option<Item>| Some(dir.as_ref()?.join(&item.as_ref().filter(|i| i.kind == ItemKind::Folder)?.name));
        let folders = [folder(dirs[0], &row.left), folder(dirs[1], &row.right)];
        let shown_row = Shown { row, depth, key, folders };
        let descend = shown_row.is_folder() && open(&shown_row.key);
        let (key, folders) = (shown_row.key.clone(), shown_row.folders.clone());
        shown.push(shown_row);
        if descend {
            walk(&row.children, &key, [&folders[0], &folders[1]], depth + 1, open, shown);
        }
    }
}

/// Where the row `key` is in `shown`, or the nearest shown folder above it once a close hides it;
/// the first row when neither is shown.
pub(crate) fn current(shown: &[Shown<'_>], key: &Path) -> usize {
    key.ancestors().find_map(|key| shown.iter().position(|shown| shown.key == key)).unwrap_or(0)
}

impl FolderCompare {
    /// The rows drawn, in order: the base folders' rows and the rows of every open folder.
    pub fn shown(&self) -> Vec<Shown<'_>> {
        flatten(&self.rows, &|key| self.expansion.is_open(key))
    }

    /// The row the expansion commands act on: the current cell's row, on either side.
    fn current_folder(&self) -> Option<Shown<'_>> {
        let shown = self.shown();
        let at = current(&shown, &self.current.0);
        shown.into_iter().nth(at).filter(Shown::is_folder)
    }

    fn current_is_open(&self) -> Option<bool> {
        self.current_folder().map(|shown| self.expansion.is_open(&shown.key))
    }

    /// Opens the current folder.
    pub fn open_folder(&mut self) {
        let Some(shown) = self.current_folder() else { return };
        let (key, folders) = (shown.key, shown.folders);
        self.expansion.open.insert(key);
        self.list(&folders);
    }

    /// Closes the current folder. The folders open inside it stay open, to show when it opens again.
    pub fn close_folder(&mut self) {
        let Some(key) = self.current_folder().map(|shown| shown.key) else { return };
        self.expansion.stop_deep(&key);
        self.expansion.open.remove(&key);
    }

    /// Opens the current folder and every folder below it, listing them as needed.
    pub fn open_subfolders(&mut self) {
        let Some(key) = self.current_folder().map(|shown| shown.key) else { return };
        self.expansion.deep.push(key);
        self.open_deep();
    }

    /// Closes the current folder and every folder below it.
    pub fn close_subfolders(&mut self) {
        let Some(key) = self.current_folder().map(|shown| shown.key) else { return };
        self.expansion.stop_deep(&key);
        self.expansion.open.retain(|open| !open.starts_with(&key));
    }

    /// Opens every folder, listing them as needed.
    pub fn expand_all(&mut self) {
        self.expansion.deep = vec![PathBuf::new()];
        self.open_deep();
    }

    /// Closes every folder.
    pub fn collapse_all(&mut self) {
        self.expansion = Expansion::default();
    }

    /// Opens every folder under the deep roots that the scans have reported, asking for the
    /// unlisted ones; a root is done once everything below it is listed. Called again after each
    /// batch of listings, since a folder's subfolders are known only once it is listed.
    pub(crate) fn open_deep(&mut self) {
        if self.expansion.deep.is_empty() {
            return;
        }
        let deep = &self.expansion.deep;
        let under = |key: &Path| deep.iter().any(|root| key.starts_with(root));
        // ponytail: walks the whole compared tree each batch; fine while `update` compares it all
        // again per batch anyway.
        let mut opened = Vec::new();
        let mut unfinished = BTreeSet::new();
        for shown in flatten(&self.rows, &|_| true).into_iter().filter(|shown| shown.is_folder() && under(&shown.key)) {
            if shown.row.scan != ScanState::Done {
                unfinished.extend(deep.iter().filter(|root| shown.key.starts_with(root)).cloned());
            }
            opened.push((shown.key, shown.folders));
        }
        self.expansion.deep.retain(|root| unfinished.contains(root));
        for (key, folders) in opened {
            self.expansion.open.insert(key);
            self.list(&folders);
        }
    }

    /// Asks each side's scan to list its folder of a row, unless it already is listed.
    fn list(&mut self, folders: &[Option<PathBuf>; 2]) {
        for (scan, folder) in self.scans.iter_mut().zip(folders) {
            let (Some(scan), Some(folder)) = (scan, folder) else { continue };
            // A folder not yet listed sends an event once it is, which `update` waits for; one
            // already listed sends none, so it mustn't make the view wait.
            if scan.scan.folder(folder).is_none_or(|found| found.state != ScanState::Done) {
                scan.scan.open(folder);
                scan.idle = false;
                // For the status bar, until the listing's event brings the scan's own counts.
                scan.pending = scan.scan.pending();
            }
        }
    }
}

/// The expansion commands, in the View menu of a Folder Compare. Each acts on the current cell's
/// row from either side, and is disabled where it has nothing to do.
pub(crate) fn commands() -> Vec<Command<Shell>> {
    vec![
        Command::new("folder-compare.open_folder", OPEN_FOLDER, |shell| with(shell, FolderCompare::open_folder))
            .state(|shell| enabled(shell, |view| view.current_is_open() == Some(false))),
        Command::new("folder-compare.close_folder", CLOSE_FOLDER, |shell| with(shell, FolderCompare::close_folder))
            .state(|shell| enabled(shell, |view| view.current_is_open() == Some(true))),
        Command::new("folder-compare.open_subfolders", OPEN_SUBFOLDERS, |shell| with(shell, FolderCompare::open_subfolders))
            .state(|shell| enabled(shell, |view| view.current_folder().is_some())),
        Command::new("folder-compare.close_subfolders", CLOSE_SUBFOLDERS, |shell| with(shell, FolderCompare::close_subfolders))
            .state(|shell| enabled(shell, |view| view.current_folder().is_some())),
        Command::new("folder-compare.expand_all", EXPAND_ALL, |shell| with(shell, FolderCompare::expand_all))
            .state(|shell| enabled(shell, |view| !view.rows.is_empty())),
        Command::new("folder-compare.collapse_all", COLLAPSE_ALL, |shell| with(shell, FolderCompare::collapse_all))
            .state(|shell| enabled(shell, |view| !view.expansion.open.is_empty() || !view.expansion.deep.is_empty())),
    ]
    .into_iter()
    .map(|command| command.views(&[ID]).menu(&["View"]))
    .collect()
}

fn with(shell: &mut Shell, run: fn(&mut FolderCompare)) {
    if let Some(view) = shell.view_mut::<FolderCompare>() {
        run(view);
    }
}

fn enabled(shell: &Shell, when: fn(&FolderCompare) -> bool) -> State {
    State::enabled(shell.view::<FolderCompare>().is_some_and(when))
}
