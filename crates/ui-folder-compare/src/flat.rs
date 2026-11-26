//! Flat List (E08-S2-T4): the files of both trees in one list, aligned by name whatever subfolder
//! each is in (the folder engine's `align_flat`), each side's name showing the folder it sits in.
//!
//! The mode lists every folder on both sides, as Expand All does, but leaves the tree's open
//! folders alone, so turning it off brings back the tree as it was. Until the other side is fully
//! listed, a file with no partner is unknown rather than an orphan, as in the tree. A folder that
//! couldn't be listed gets a row of its own, with its error, since its files can't show.
//!
//! In a flat row each item's `name` is its path from its side's base, so the name cell shows the
//! folder, and the base joined with it is the file. A row's key (see [`Shown::key`]) is the left
//! item's path, or the right's for a right-only row.

use std::collections::BTreeSet;
use std::path::PathBuf;

use otd_core::command::{Command, Mark, State};
use otd_folder_engine::{Aligned, Alignment, Placed, Row, ScanState, Status, align_flat, case_sensitive, quick_test};
use otd_source::{Item, ItemKind};
use otd_ui::Shell;

use crate::expand::{Expansion, flatten};
use crate::{CRITERIA, FolderCompare, ID, Scan, Shown};

// English key, drawn only through `t` (the menu's caption).
pub const FLAT_LIST: &str = "Flat List";

/// The mode, and its rows while it is on.
#[derive(Debug, Default)]
pub(crate) struct Flat {
    pub(crate) on: bool,
    rows: Vec<Row>,
    /// Each row's key, by index.
    keys: Vec<PathBuf>,
}

impl Flat {
    /// The rows the counts and the result go by: the flat ones while the mode is on.
    pub(crate) fn rows<'a>(&'a self, tree: &'a [Row]) -> &'a [Row] {
        if self.on { &self.rows } else { tree }
    }
}

/// The rows drawn, in order: the flat rows while the mode is on, otherwise the tree's base rows
/// and those of each open folder.
pub(crate) fn shown<'a>(rows: &'a [Row], flat: &'a Flat, expansion: &Expansion) -> Vec<Shown<'a>> {
    if !flat.on {
        return flatten(rows, &|key| expansion.is_open(key));
    }
    flat.rows.iter().zip(&flat.keys).map(|(row, key)| Shown { row, depth: 0, key: key.clone(), folders: [None, None] }).collect()
}

impl FolderCompare {
    /// Whether the view shows the flat list rather than the tree.
    pub fn is_flat(&self) -> bool {
        self.flat.on
    }

    /// Turns the flat list on or off.
    pub fn toggle_flat(&mut self) {
        self.flat.on = !self.flat.on;
        // The other mode's rows have other keys, and a selection out of sight would be acted on unseen.
        self.selected = Default::default();
        self.anchor = [None, None];
        // The current row is gone; focus follows the current cell to the nearest row the mode shows.
        self.refocus |= self.grid_focus;
        self.update_flat();
    }

    /// While the mode is on: asks for every folder not yet listed, and works out the flat rows from
    /// what is. Called after each batch of listings, since a folder's subfolders are known only once
    /// it is listed.
    pub(crate) fn update_flat(&mut self) {
        self.flat.rows.clear();
        self.flat.keys.clear();
        if !self.flat.on {
            return;
        }
        // As in the tree: a side that can't be listed leaves nothing to compare.
        let ([Some(left), Some(right)], [None, None]) = (&mut self.scans, &self.errors) else { return };
        let case = case_sensitive(&left.capabilities, &right.capabilities);
        let (left, right) = (left.walk(), right.walk());
        let mut rows: Vec<(PathBuf, Row)> = Vec::new();
        for aligned in align_flat(left.files, right.files, Alignment::new(case)) {
            let key = aligned.left.as_ref().or(aligned.right.as_ref()).map(path).unwrap_or_default();
            let item = |placed: Option<Placed>| placed.map(|placed| Item { name: path(&placed).to_string_lossy().into_owned(), ..placed.item });
            let (left_item, right_item) = (item(aligned.left), item(aligned.right));
            let pair = Aligned { name: aligned.name, left: left_item, right: right_item };
            let status = match quick_test(&pair, &CRITERIA) {
                Status::LeftOrphan if !right.complete => Status::Unknown,
                Status::RightOrphan if !left.complete => Status::Unknown,
                status => status,
            };
            rows.push((key, row(pair, status)));
        }
        for (at, unread) in [left.unread, right.unread].into_iter().enumerate() {
            for (placed, error) in unread {
                let (name, key) = (placed.item.name.clone(), path(&placed));
                let item = Some(Item { name: key.to_string_lossy().into_owned(), ..placed.item });
                let (left, right) = if at == 0 { (item, None) } else { (None, item) };
                rows.push((key, row(Aligned { name, left, right }, Status::Error(error))));
            }
        }
        // By name, so files of one name stay together; stable, so the alignment's order holds among them.
        rows.sort_by(|a, b| a.1.name.cmp(&b.1.name));
        let mut used = BTreeSet::new();
        for (mut key, row) in rows {
            // ponytail: a path holds one item per side, so keys only clash when one side's unreadable
            // folder has the path of the other side's file; a suffix keeps them apart.
            while !used.insert(key.clone()) {
                key = PathBuf::from(format!("{}#", key.to_string_lossy()));
            }
            self.flat.keys.push(key);
            self.flat.rows.push(row);
        }
    }
}

fn row(pair: Aligned, status: Status) -> Row {
    Row {
        name: pair.name,
        left: pair.left,
        right: pair.right,
        status,
        left_hidden: false,
        right_hidden: false,
        scan: ScanState::Done,
        children: Vec::new(),
    }
}

/// An item's path from its side's base.
fn path(placed: &Placed) -> PathBuf {
    placed.folder.join(&placed.item.name)
}

/// One side's tree as far as it is listed.
struct Walked {
    files: Vec<Placed>,
    /// Folders that couldn't be listed, with why.
    unread: Vec<(Placed, otd_source::SourceError)>,
    /// Every folder is listed, so a file missing here really is missing.
    complete: bool,
}

impl Scan {
    /// Collects the side's listed files, and asks for every folder not listed yet.
    fn walk(&mut self) -> Walked {
        let mut walked = Walked { files: Vec::new(), unread: Vec::new(), complete: true };
        // Each folder with the item its parent listed it as; the base has none.
        let mut folders: Vec<(PathBuf, Option<Item>)> = vec![(PathBuf::new(), None)];
        while let Some((dir, item)) = folders.pop() {
            let found = self.scan.folder(&dir);
            if found.as_ref().is_none_or(|found| found.state != ScanState::Done) {
                walked.complete = false;
                // A folder already waiting stays where it is in the queue.
                if found.as_ref().is_none_or(|found| found.state == ScanState::Unscanned) {
                    self.scan.open(&dir);
                    self.idle = false;
                    self.pending = self.scan.pending();
                }
            }
            // A folder waiting to be listed again (Refresh) keeps its old items until it is, as the tree does.
            let Some(found) = found else { continue };
            if let Some(error) = found.error {
                walked.complete = false;
                // The base's own error is the view's, shown above its pane.
                if let (Some(item), Some(parent)) = (item, dir.parent()) {
                    walked.unread.push((Placed { folder: parent.to_path_buf(), item }, error));
                }
                continue;
            }
            for child in found.items {
                if child.kind == ItemKind::Folder {
                    folders.push((dir.join(&child.name), Some(child)));
                } else {
                    walked.files.push(Placed { folder: dir.clone(), item: child });
                }
            }
        }
        walked
    }
}

/// Flat List, in the View menu of a Folder Compare, checked while on.
pub(crate) fn commands() -> Vec<Command<Shell>> {
    vec![
        Command::new("folder-compare.flat_list", FLAT_LIST, |shell: &mut Shell| {
            if let Some(view) = shell.view_mut::<FolderCompare>() {
                view.toggle_flat();
            }
        })
        .state(|shell| match shell.view::<FolderCompare>() {
            Some(view) => State { mark: Some(Mark::Check(view.flat.on)), ..State::default() },
            None => State::enabled(false),
        })
        .views(&[ID])
        .menu(&["View"]),
    ]
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use otd_folder_engine::TreeScan;
    use otd_source::{DataSource, LocalSource};

    use super::*;

    /// A scan of `dir` that lists only when the test steps it.
    fn scan(dir: &std::path::Path) -> Scan {
        let source = LocalSource::open(dir.to_str().unwrap()).unwrap();
        let capabilities = source.capabilities();
        let (scan, events) = TreeScan::new(Arc::new(source), "", false);
        Scan { scan, events, capabilities, idle: false, done: 0, pending: 1 }
    }

    fn status(view: &FolderCompare, name: &str) -> Option<Status> {
        view.shown().into_iter().find(|shown| shown.row.name == name).map(|shown| shown.row.status.clone())
    }

    #[test]
    fn a_file_with_no_partner_is_unknown_until_the_other_side_is_fully_listed() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("left")).unwrap();
        std::fs::write(root.path().join("left/x.txt"), "x").unwrap();
        std::fs::create_dir_all(root.path().join("right/sub")).unwrap();
        let mut view = FolderCompare::new();
        view.scans = [Some(scan(&root.path().join("left"))), Some(scan(&root.path().join("right")))];
        let step = |view: &FolderCompare, side: usize| while view.scans[side].as_ref().is_some_and(|scan| scan.scan.step()) {};
        step(&view, 0);
        step(&view, 1);
        view.toggle_flat();
        assert_eq!(status(&view, "x.txt"), Some(Status::Unknown), "right/sub isn't listed yet");
        step(&view, 1);
        view.update_flat();
        assert_eq!(status(&view, "x.txt"), Some(Status::LeftOrphan));
    }

    #[test]
    fn a_refresh_keeps_the_rows_of_folders_waiting_to_be_listed_again() {
        let root = tempfile::tempdir().unwrap();
        for side in ["left", "right"] {
            std::fs::create_dir_all(root.path().join(side).join("sub")).unwrap();
            std::fs::write(root.path().join(side).join("sub/x.txt"), "x").unwrap();
        }
        let mut view = FolderCompare::new();
        view.scans = [Some(scan(&root.path().join("left"))), Some(scan(&root.path().join("right")))];
        let step = |view: &FolderCompare| {
            for scan in view.scans.iter().flatten() {
                while scan.scan.step() {}
            }
        };
        view.toggle_flat();
        // The base, then `sub`, which the flat list asks for once the base shows it.
        for _ in 0..2 {
            step(&view);
            view.update_flat();
        }
        assert_eq!(status(&view, "x.txt"), Some(Status::Same), "everything listed");
        view.refresh();
        // One of the two folders listed again so far; the other still waits, with its old items.
        for scan in view.scans.iter().flatten() {
            scan.scan.step();
        }
        view.update_flat();
        assert_eq!(status(&view, "x.txt"), Some(Status::Same));
    }
}
