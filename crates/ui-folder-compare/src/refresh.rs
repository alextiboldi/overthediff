//! Refresh, Full Refresh, Refresh Selection and Swap Sides (E08-S1-T6).
//!
//! Refresh lists every folder the scans have listed again, in place, so the rows and the focus stay
//! while the new listings come in. Full Refresh drops both scans and compares from scratch. Swap
//! Sides exchanges the two base folders and compares from scratch too.

use std::path::{Path, PathBuf};

use otd_core::command::{Command, State};
use otd_ui::Shell;

use crate::{FolderCompare, ID, Scan};

// English keys, drawn only through `t` (the menus' captions).
pub const REFRESH: &str = "Refresh";
pub const FULL_REFRESH: &str = "Full Refresh";
pub const REFRESH_SELECTION: &str = "Refresh Selection";
pub const SWAP_SIDES: &str = "Swap Sides";

const VIEWS: &[&str] = &[ID];

/// Refresh, Full Refresh, Refresh Selection and Swap Sides.
// ponytail: no default shortcuts. Q24 gives these none, and Cmd/Ctrl+R is Copy to Right here.
pub(crate) fn commands() -> Vec<Command<Shell>> {
    vec![
        Command::new("folder-compare.refresh", REFRESH, |shell| with_view(shell, FolderCompare::refresh)).icon("refresh-cw").menu(&["View"]),
        Command::new("folder-compare.full_refresh", FULL_REFRESH, |shell| with_view(shell, FolderCompare::full_refresh))
            .icon("refresh-ccw-dot")
            .menu(&["View"]),
        Command::new("folder-compare.refresh_selection", REFRESH_SELECTION, |shell| with_view(shell, FolderCompare::refresh_selection))
            .icon("list-restart")
            .menu(&["View"])
            .state(|shell| State::enabled(shell.view::<FolderCompare>().is_some_and(|view| !view.shown().is_empty()))),
        Command::new("folder-compare.swap_sides", SWAP_SIDES, |shell| with_view(shell, FolderCompare::swap_sides))
            .icon("arrow-left-right")
            .menu(&["Session"]),
    ]
    .into_iter()
    .chain(crate::info::commands())
    .map(|command| command.views(VIEWS))
    .collect()
}

/// The commands apply only to Folder Compare tabs, so the view is there whenever one runs.
fn with_view(shell: &mut Shell, run: fn(&mut FolderCompare)) {
    if let Some(view) = shell.view_mut::<FolderCompare>() {
        run(view);
    }
}

impl FolderCompare {
    /// Refresh: lists again every folder either scan has listed.
    pub fn refresh(&mut self) {
        self.relist(Path::new(""), true);
    }

    /// Full Refresh: scans both base folders again from scratch and compares them.
    pub fn full_refresh(&mut self) {
        self.rescan();
    }

    /// Refresh Selection: lists the focused row's folder again, for its own size and time, and the
    /// listed folders inside it when it is a folder.
    // ponytail: the focused row, not yet T5's selection (issue #100).
    pub fn refresh_selection(&mut self) {
        let shown = self.shown();
        let Some(row) = shown.get(crate::expand::current(&shown, &self.current.0)) else { return };
        // Each side by its own spelling: case-insensitive pairing can match `Docs` to `docs`. A shown
        // row's parent is shown too, and the base rows' parent is the base.
        let parents = match row.key.parent().filter(|parent| !parent.as_os_str().is_empty()) {
            Some(parent) => shown.iter().find(|shown| shown.key == parent).map(|shown| shown.folders.clone()).unwrap_or_default(),
            None => [Some(PathBuf::new()), Some(PathBuf::new())],
        };
        let folders = row.folders.clone();
        for ((scan, parent), folder) in self.scans.iter_mut().zip(parents).zip(folders) {
            let Some(scan) = scan else { continue };
            if let Some(parent) = parent {
                scan.relist(&parent, false);
            }
            if let Some(folder) = folder {
                scan.relist(&folder, true);
            }
        }
    }

    /// Swap Sides: the left base folder goes right and the right one left, then both are compared.
    pub fn swap_sides(&mut self) {
        self.session.specs.resize(self.session.specs.len().max(2), String::new());
        self.session.specs.swap(0, 1);
        self.rescan();
    }

    fn relist(&mut self, folder: &Path, below: bool) {
        for scan in self.scans.iter_mut().flatten() {
            scan.relist(folder, below);
        }
    }
}

impl Scan {
    fn relist(&mut self, folder: &Path, below: bool) {
        let queued = self.scan.refresh(folder, below);
        if queued > 0 {
            // Scanning again: the view polls until the listings are in. No event says the folders
            // went back to pending, so the counts move here, as the scan's own do.
            self.idle = false;
            self.done = self.done.saturating_sub(queued);
            self.pending += queued;
        }
    }
}
