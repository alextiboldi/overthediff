//! Copy Filename (E08-S7-T5): *Edit > Copy Filename* puts the full paths of the items selected on
//! either side on the clipboard, one per line (*inferred* separator, confirmed by the operator on
//! 2026-11-25).
//!
//! *Unspecified*, decided here: a selected folder gives its own path, not its contents'; the left
//! side's paths come before the right side's, each in the order the rows show; lines are joined
//! with `\n` on every platform, with none after the last. Each path is the side's base folder, as
//! it was scanned, joined with the item's path in that side's own spelling.

use std::path::PathBuf;

use otd_core::command::{Command, State};
use otd_ui::Shell;

use crate::{FolderCompare, ID};

// English key, drawn only through `t` (the menu's caption).
pub const COPY_FILENAME: &str = "Copy Filename";

/// Copy Filename. It reaches the clipboard through the window, so it needs one.
// ponytail: no default shortcut; Q24 names none for it.
pub(crate) fn commands() -> Vec<Command<Shell>> {
    vec![
        Command::new("folder-compare.copy_filename", COPY_FILENAME, |shell: &mut Shell| {
            if let Some(view) = shell.view_mut::<FolderCompare>() {
                view.copy_filename();
            }
        })
        .icon("clipboard-copy")
        .menu(&["Edit"])
        .needs_window()
        .state(|shell: &Shell| State::enabled(shell.view::<FolderCompare>().is_some_and(|view| !view.selected_paths().is_empty())))
        .views(&[ID]),
    ]
}

impl FolderCompare {
    /// The full paths of the selected items shown: the left side's, then the right side's.
    pub fn selected_paths(&self) -> Vec<PathBuf> {
        let Some(specs) = &self.scanned else { return Vec::new() };
        let shown = self.shown();
        let mut paths = Vec::new();
        for side in 0..2 {
            // As `rescan` opened it, so `~/…` and `smb://…` give the folder that was listed.
            let Some(base) = specs.get(side).and_then(|spec| otd_source::resolve(spec.trim()).ok()) else { continue };
            let base = std::path::absolute(&base).unwrap_or(base);
            // Each depth's folder on this side, in its own spelling: the rows come in tree order, so a
            // row's parent is the last row shown one level up. Flat rows are all at depth 0, under the base.
            let mut parents = vec![Some(PathBuf::new())];
            for row in &shown {
                parents.truncate(row.depth + 1);
                let parent = parents.get(row.depth).cloned().flatten();
                parents.push(row.folders[side].clone());
                let item = if side == 0 { &row.row.left } else { &row.row.right };
                if let (Some(item), Some(parent)) = (item, parent)
                    && self.selected[side].contains(&row.key)
                {
                    paths.push(base.join(parent).join(&item.name));
                }
            }
        }
        paths
    }

    /// Copy Filename: the selected items' full paths go to the clipboard on the next draw.
    pub fn copy_filename(&mut self) {
        let paths = self.selected_paths();
        if !paths.is_empty() {
            self.clipboard = Some(paths.iter().map(|path| path.to_string_lossy()).collect::<Vec<_>>().join("\n"));
        }
    }
}
