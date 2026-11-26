//! Select Newer, Select Orphans and Invert Selection (E08-S7-T4), in the Edit menu.
//!
//! Each works on the visible items only: the rows [`FolderCompare::shown`] gives (so nothing inside
//! a closed folder), on a side where the row has an item that no filter hides.
//!
//! - *Select Newer* (left, right or both) selects the items newer on that side, and *Select Orphans*
//!   the orphans on that side. A row counts by its own status, so a folder pair that only holds
//!   newer files or orphans isn't selected itself (*unspecified*, decided here: selecting it would
//!   take in everything inside). The command replaces the whole selection, so after *Select Left
//!   Newer* nothing is selected on the right (*unspecified*, decided here).
//! - *Invert Selection* selects, on each side, the visible items that weren't selected and
//!   deselects the rest, hidden ones included.
//!
//! The selection changes away from focus, so the view announces how many items each side now has
//! selected, and gives the same text to scripts.

use std::collections::BTreeSet;
use std::path::PathBuf;

use egui::Ui;
use egui::accesskit::Live;
use otd_core::command::{Args, Command, CommandError, Outcome, State, Value};
use otd_core::i18n::{fmt, t};
use otd_folder_engine::Status;
use otd_ui::Shell;

use crate::{FolderCompare, ID, Shown};

// English keys, drawn only through `t` (the menus' captions and the announcement).
pub const SELECT_LEFT_NEWER: &str = "Select Left Newer";
pub const SELECT_RIGHT_NEWER: &str = "Select Right Newer";
pub const SELECT_NEWER: &str = "Select Newer";
pub const SELECT_LEFT_ORPHANS: &str = "Select Left Orphans";
pub const SELECT_RIGHT_ORPHANS: &str = "Select Right Orphans";
pub const SELECT_ORPHANS: &str = "Select Orphans";
pub const INVERT_SELECTION: &str = "Invert Selection";
pub const SELECTED_COUNTS: &str = "Selected: {left} on the left, {right} on the right";
const NOT_FOLDER_COMPARE: &str = "The active tab isn't a Folder Compare.";

/// Which statuses a command selects, for each side; `None` leaves that side with nothing selected.
type Wanted = [Option<Status>; 2];

const LEFT_NEWER: Wanted = [Some(Status::LeftNewer), None];
const RIGHT_NEWER: Wanted = [None, Some(Status::RightNewer)];
const NEWER: Wanted = [Some(Status::LeftNewer), Some(Status::RightNewer)];
const LEFT_ORPHANS: Wanted = [Some(Status::LeftOrphan), None];
const RIGHT_ORPHANS: Wanted = [None, Some(Status::RightOrphan)];
const ORPHANS: Wanted = [Some(Status::LeftOrphan), Some(Status::RightOrphan)];

/// The seven commands, in the Edit menu of a Folder Compare, disabled while it shows no rows.
// ponytail: no default shortcuts; Q24 names none for these.
pub(crate) fn commands() -> Vec<Command<Shell>> {
    vec![
        Command::typed("folder-compare.select_left_newer", SELECT_LEFT_NEWER, &[], |shell, _| select(shell, LEFT_NEWER)),
        Command::typed("folder-compare.select_right_newer", SELECT_RIGHT_NEWER, &[], |shell, _| select(shell, RIGHT_NEWER)),
        Command::typed("folder-compare.select_newer", SELECT_NEWER, &[], |shell, _| select(shell, NEWER)),
        Command::typed("folder-compare.select_left_orphans", SELECT_LEFT_ORPHANS, &[], |shell, _| select(shell, LEFT_ORPHANS)),
        Command::typed("folder-compare.select_right_orphans", SELECT_RIGHT_ORPHANS, &[], |shell, _| select(shell, RIGHT_ORPHANS)),
        Command::typed("folder-compare.select_orphans", SELECT_ORPHANS, &[], |shell, _| select(shell, ORPHANS)),
        Command::typed("folder-compare.invert_selection", INVERT_SELECTION, &[], invert),
    ]
    .into_iter()
    .map(|command| {
        command.views(&[ID]).menu(&["Edit"]).state(|shell| State::enabled(shell.view::<FolderCompare>().is_some_and(|view| !view.shown().is_empty())))
    })
    .collect()
}

fn view(shell: &mut Shell) -> Result<&mut FolderCompare, CommandError> {
    shell.view_mut::<FolderCompare>().ok_or_else(|| CommandError::Failed(t(NOT_FOLDER_COMPARE).to_owned()))
}

fn select(shell: &mut Shell, wanted: Wanted) -> Outcome {
    let view = view(shell)?;
    view.select_status(&wanted);
    Ok(Some(Value::Text(view.announce())))
}

fn invert(shell: &mut Shell, _: &Args) -> Outcome {
    let view = view(shell)?;
    view.invert_selection();
    Ok(Some(Value::Text(view.announce())))
}

/// Whether `side` of a shown row has an item the user can see.
fn visible(shown: &Shown<'_>, side: usize) -> bool {
    let row = shown.row;
    if side == 0 { row.left.is_some() && !row.left_hidden } else { row.right.is_some() && !row.right_hidden }
}

impl FolderCompare {
    /// Selects, on each side, the visible items whose status is the one `wanted` gives that side.
    fn select_status(&mut self, wanted: &Wanted) {
        let shown = self.shown();
        let selected: [BTreeSet<PathBuf>; 2] = std::array::from_fn(|side| {
            let Some(status) = &wanted[side] else { return BTreeSet::new() };
            shown.iter().filter(|shown| visible(shown, side) && shown.row.status == *status).map(|shown| shown.key.clone()).collect()
        });
        self.set_selection(selected);
    }

    /// Selects, on each side, the visible items that weren't selected, and nothing else.
    fn invert_selection(&mut self) {
        let shown = self.shown();
        let selected: [BTreeSet<PathBuf>; 2] = std::array::from_fn(|side| {
            shown.iter().filter(|shown| visible(shown, side) && !self.selected[side].contains(&shown.key)).map(|shown| shown.key.clone()).collect()
        });
        self.set_selection(selected);
    }

    fn set_selection(&mut self, selected: [BTreeSet<PathBuf>; 2]) {
        self.selected = selected;
        // A Shift range starts afresh, at the focused cell, as after a flat-list toggle.
        self.anchor = [None, None];
    }

    /// The counts of each side's selection in words, kept to announce until the selection changes.
    fn announce(&mut self) -> String {
        let counts = self.selected.each_ref().map(BTreeSet::len);
        let text = fmt(t(SELECTED_COUNTS), &[("left", &counts[0].to_string()), ("right", &counts[1].to_string())]);
        self.selection_notice = Some((counts, text.clone()));
        text
    }

    /// The last selection command's counts, announced; gone once a click or key changes them.
    pub(crate) fn selection_notice_ui(&mut self, ui: &mut Ui) {
        // ponytail: compares counts, so a click that keeps both counts keeps the notice; a key that
        // says the selection changed would need every pick to clear it.
        if self.selection_notice.as_ref().is_some_and(|(counts, _)| *counts != self.selected.each_ref().map(BTreeSet::len)) {
            self.selection_notice = None;
        }
        let Some((_, text)) = &self.selection_notice else { return };
        let label = ui.label(text.as_str());
        ui.ctx().accesskit_node_builder(label.id, |node| node.set_live(Live::Polite));
    }
}
