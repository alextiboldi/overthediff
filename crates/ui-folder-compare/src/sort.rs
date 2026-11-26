//! Sorting (E08-S6-T2): a click on a column header, or *View > Sort By ▸ <column>*, sorts the rows
//! of every folder by that column, and the same again reverses the order.
//!
//! Rows sort whole, so partners stay on one row. The key is the item on the side whose header was
//! clicked (the current cell's side for the menu), and an orphan with no item there sorts by its
//! other side's item (*unspecified*, decided here). Ties keep the engine's name order, and folders
//! sort among the files by the same key. The sort is the view's own, so a Refresh, Full Refresh or
//! Swap Sides keeps it. A column added to [`COLUMNS`] sorts by its [`Column::order`] and gets its
//! menu entry without a change here.

use std::cmp::Ordering;

use egui::accesskit::SortDirection;
use egui::{Pos2, Stroke, Vec2};
use otd_core::command::Command;
use otd_folder_engine::Row;
use otd_source::Item;
use otd_ui::Shell;

use crate::{COLUMNS, FolderCompare, ID, NAME};

// English keys, drawn only through `t`.
pub const SORT_BY: &str = "Sort By";
/// A sorted column header's accessible name, after its caption.
pub const SORTED_ASCENDING: &str = "{name}, sorted ascending";
pub const SORTED_DESCENDING: &str = "{name}, sorted descending";

/// The order the rows show in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Sort {
    /// The column's caption: [`NAME`] or one of [`COLUMNS`].
    pub column: &'static str,
    /// 0 left, 1 right: whose items give the key.
    pub side: usize,
    pub descending: bool,
}

impl Default for Sort {
    /// The engine's own order.
    fn default() -> Self {
        Self { column: NAME, side: 0, descending: false }
    }
}

impl Sort {
    /// The direction `column`'s header on `side` shows, if it is the one sorted by.
    pub(crate) fn direction(&self, column: &str, side: usize) -> Option<SortDirection> {
        (self.column == column && self.side == side).then_some(if self.descending { SortDirection::Descending } else { SortDirection::Ascending })
    }

    /// Sorts `rows` and every folder's rows inside them.
    pub(crate) fn apply(&self, rows: &mut [Row]) {
        let order =
            COLUMNS.iter().find(|column| column.caption == self.column).map_or(by_name as fn(&Item, &Item) -> Ordering, |column| column.order);
        sort(rows, &|a, b| {
            let ordering = match (key(a, self.side), key(b, self.side)) {
                (Some(a), Some(b)) => order(a, b),
                _ => Ordering::Equal,
            };
            if self.descending { ordering.reverse() } else { ordering }
        });
    }
}

/// The item a row sorts by: `side`'s, or the other side's for an orphan. A row always has one.
fn key(row: &Row, side: usize) -> Option<&Item> {
    let (own, other) = if side == 0 { (&row.left, &row.right) } else { (&row.right, &row.left) };
    own.as_ref().or(other.as_ref())
}

// ponytail: recursive, as deep as the tree, like `compare_trees`.
fn sort(rows: &mut [Row], order: &dyn Fn(&Row, &Row) -> Ordering) {
    // Stable, so ties keep the name order the engine gives.
    rows.sort_by(order);
    for row in rows {
        sort(&mut row.children, order);
    }
}

fn by_name(a: &Item, b: &Item) -> Ordering {
    a.name.cmp(&b.name)
}

impl FolderCompare {
    /// Sorts by the column captioned `column` on `side`, ascending; when the rows are already sorted
    /// by it, reverses the order.
    pub fn sort_by(&mut self, column: &'static str, side: usize) {
        let descending = self.sort.column == column && self.sort.side == side && !self.sort.descending;
        self.sort = Sort { column, side, descending };
        self.sort.apply(&mut self.rows);
    }
}

/// One *View > Sort By* entry per column, Name's first: each sorts by its column on the current
/// cell's side, as a click on that side's header does. Q24 names no shortcuts.
// ponytail: no radio mark for the column sorted by, since a command's state is a plain `fn` that
// can't be told its column; the header's arrow and accessible name show it.
pub(crate) fn commands() -> Vec<Command<Shell>> {
    std::iter::once(NAME)
        .chain(COLUMNS.iter().map(|column| column.caption))
        .map(|caption| {
            // ponytail: leaked once per shell, since command ids are `&'static`; a few bytes per column.
            let id: &'static str = Box::leak(format!("folder-compare.sort.{}", caption.to_lowercase()).into_boxed_str());
            Command::for_item(id, caption, caption, |shell: &mut Shell, caption| {
                if let Some(view) = shell.view_mut::<FolderCompare>() {
                    let side = view.current.1;
                    view.sort_by(caption, side);
                }
            })
            .icon("arrow-up-down")
            .menu(&["View", SORT_BY])
            .views(&[ID])
        })
        .collect()
}

/// Lucide's `arrow-up` and `arrow-down` (ISC; both from Feather, MIT; see ATTRIBUTION.md): their
/// path points on the icons' 24-unit grid, a polyline each.
const ARROW_UP: [&[(f32, f32)]; 2] = [&[(5.0, 12.0), (12.0, 5.0), (19.0, 12.0)], &[(12.0, 19.0), (12.0, 5.0)]];
const ARROW_DOWN: [&[(f32, f32)]; 2] = [&[(12.0, 5.0), (12.0, 19.0)], &[(19.0, 12.0), (12.0, 19.0), (5.0, 12.0)]];

/// The sorted header's arrow, `size` points wide and centred on `center`, with Lucide's stroke width
/// of 2: up for ascending, down for descending.
pub(crate) fn paint_arrow(painter: &egui::Painter, center: Pos2, size: f32, direction: SortDirection, colour: egui::Color32) {
    let scale = size / 24.0;
    let at = |&(x, y): &(f32, f32)| center + Vec2::new(x - 12.0, y - 12.0) * scale;
    let lines = if direction == SortDirection::Descending { ARROW_DOWN } else { ARROW_UP };
    for line in lines {
        painter.add(egui::Shape::line(line.iter().map(at).collect(), Stroke::new(2.0 * scale, colour)));
    }
}
