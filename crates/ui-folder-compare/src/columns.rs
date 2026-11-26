//! The column chooser (E08-S6-T1): *View > Columns* and a right-click on a column header show or
//! hide each of [`COLUMNS`] in both panes at once.
//!
//! The choice is the view's own, so a Refresh, Full Refresh or Swap Sides keeps it. Name always
//! shows: its cell is each row's focus stop and holds the tree's chevron, the status spot and the
//! selection, so the chooser lists it checked and disabled (*unspecified*, decided here). A column
//! added to [`COLUMNS`] gets its chooser entry without a change here.

use std::collections::BTreeSet;

use egui::Ui;
use otd_core::command::{Command, Mark, State};
use otd_core::i18n::t;
use otd_ui::Shell;

use crate::{COLUMNS, Column, FolderCompare, ID, NAME};

// English key, drawn only through `t`: the View menu's submenu.
pub const COLUMNS_MENU: &str = "Columns";

const MENU: &[&str] = &["View", COLUMNS_MENU];

/// Each column's check mark, by its index in [`COLUMNS`]: a command's state is a plain `fn`, so it
/// can't be told which column it is for.
const MARKS: [fn(&Shell) -> State; 8] = [mark::<0>, mark::<1>, mark::<2>, mark::<3>, mark::<4>, mark::<5>, mark::<6>, mark::<7>];
const _: () = assert!(COLUMNS.len() <= MARKS.len(), "add a mark per column");

fn mark<const I: usize>(shell: &Shell) -> State {
    let on = COLUMNS.get(I).is_some_and(|column| shell.view::<FolderCompare>().is_none_or(|view| view.column_shown(column.caption)));
    State { mark: Some(Mark::Check(on)), ..State::default() }
}

/// One check item per column in *View > Columns*, Name's first and disabled. Q24 names no shortcuts.
pub(crate) fn commands() -> Vec<Command<Shell>> {
    let name = Command::new("folder-compare.column.name", NAME, |_| {}).state(|_| State {
        enabled: false,
        mark: Some(Mark::Check(true)),
        ..State::default()
    });
    let columns = COLUMNS.iter().zip(MARKS).map(|(column, mark)| {
        // ponytail: leaked once per shell, since command ids are `&'static`; a few bytes per column.
        let id: &'static str = Box::leak(format!("folder-compare.column.{}", column.caption.to_lowercase()).into_boxed_str());
        Command::for_item(id, column.caption, column.caption, |shell: &mut Shell, caption| {
            if let Some(view) = shell.view_mut::<FolderCompare>() {
                view.toggle_column(caption);
            }
        })
        .state(mark)
    });
    std::iter::once(name).chain(columns).map(|command| command.icon("columns-3").menu(MENU).views(&[ID])).collect()
}

/// The columns hidden, by caption, so a column inserted into [`COLUMNS`] doesn't shift the choice.
pub(crate) type Hidden = BTreeSet<&'static str>;

impl FolderCompare {
    /// Whether the column captioned `caption` shows in both panes.
    pub fn column_shown(&self, caption: &str) -> bool {
        !self.hidden.contains(caption)
    }

    /// Shows the column captioned `caption` if it is hidden, and hides it if it shows.
    pub fn toggle_column(&mut self, caption: &'static str) {
        if !self.hidden.remove(caption) {
            self.hidden.insert(caption);
        }
    }

    /// The columns that show after the name, in [`COLUMNS`] order.
    pub(crate) fn shown_columns(&self) -> Vec<&'static Column> {
        COLUMNS.iter().filter(|column| self.column_shown(column.caption)).collect()
    }
}

/// The header's right-click menu: the same choices as *View > Columns*.
// ponytail: toggles the view directly, as the commands do, since a view's `ui` has no shell to run
// a command by id through.
pub(crate) fn menu(ui: &mut Ui, view: &mut FolderCompare) {
    ui.add_enabled(false, egui::Checkbox::new(&mut true, t(NAME)));
    for column in COLUMNS {
        if ui.checkbox(&mut view.column_shown(column.caption), t(column.caption)).changed() {
            view.toggle_column(column.caption);
        }
    }
}
