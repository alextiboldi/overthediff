//! Select All and Select All Files (E08-S7-T2), in a Folder Compare's Edit menu.
//!
//! Both select on each side the items of the rows [`FolderCompare::shown`] draws, so a row inside a
//! closed folder is never selected: selecting a folder already stands for its visible contents
//! (`resolve_selection`). Select All Files leaves out each side's folders, judged by that side's own
//! item, so a row with a file on the left and a folder on the right selects only the left. A link
//! counts as a file, as a folder is only what `ItemKind::Folder` says, the rule expansion opens by
//! (*unspecified*, decided here).

use otd_core::command::{Command, Key, Mods, State};
use otd_source::{Item, ItemKind};
use otd_ui::Shell;

use crate::{FolderCompare, ID};

// English keys, drawn only through `t` (the menus' captions).
pub const SELECT_ALL: &str = "Select All";
pub const SELECT_ALL_FILES: &str = "Select All Files";

/// Select All on the platform's chord (Q24), Select All Files with none. Both wait while a path
/// edit has the keys, which takes Cmd/Ctrl+A for its own text.
pub(crate) fn commands() -> Vec<Command<Shell>> {
    vec![
        Command::new("folder-compare.select_all", SELECT_ALL, |shell| with(shell, FolderCompare::select_all))
            .icon("list-checks")
            .shortcut(Mods::CMD, Key::A),
        Command::new("folder-compare.select_all_files", SELECT_ALL_FILES, |shell| with(shell, FolderCompare::select_all_files)).icon("file-check"),
    ]
    .into_iter()
    .map(|command| {
        command.views(&[ID]).menu(&["Edit"]).state(|shell| State::enabled(shell.view::<FolderCompare>().is_some_and(|view| !view.rows.is_empty())))
    })
    .collect()
}

fn with(shell: &mut Shell, run: fn(&mut FolderCompare)) {
    if let Some(view) = shell.view_mut::<FolderCompare>() {
        run(view);
    }
}

impl FolderCompare {
    /// Select All: every shown item, on both sides.
    pub fn select_all(&mut self) {
        self.select_shown(|_| true);
    }

    /// Select All Files: every shown item that isn't a folder, on both sides.
    pub fn select_all_files(&mut self) {
        self.select_shown(|item| item.kind != ItemKind::Folder);
    }

    fn select_shown(&mut self, take: fn(&Item) -> bool) {
        let shown = self.shown();
        let selected = [0, 1].map(|side| {
            shown.iter().filter(|shown| [&shown.row.left, &shown.row.right][side].as_ref().is_some_and(take)).map(|shown| shown.key.clone()).collect()
        });
        self.selected = selected;
    }
}
