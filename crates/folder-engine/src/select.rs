//! Selection resolution for operations (E03-S2-T8): which items on one side a copy, move, delete
//! or other operation acts on, given the rows selected on that side.

use std::path::{Path, PathBuf};

use otd_source::{Item, ItemKind};

use crate::{Row, ScanState, Status, TreeSide};

/// What a selection resolves to on one side.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Resolved {
    /// Each item with its path relative to the side's base, in that side's own spelling, in tree
    /// order (a folder before its contents) and once each. A folder here stands for itself only:
    /// its contents that count are listed after it.
    pub items: Vec<(PathBuf, Item)>,
    /// Some folder whose contents were taken isn't listed yet, or couldn't be, on one side or the
    /// other, so `items` may lack some of what the selection means. Finish the scan and resolve
    /// again before acting on it.
    pub incomplete: bool,
}

/// Resolves the rows selected on `side`, each given as its path of [`Row::name`]s from the base,
/// against the rows [`crate::compare_trees`] returned.
///
/// - A selected file resolves to itself.
/// - A selected folder resolves to its visible contents, at every depth, not to itself.
/// - A selected folder with some of its descendants also selected resolves to those descendants
///   only. That is how an open folder plus some of its children reads, so the engine needs no
///   notion of which folders a view has open.
/// - An item hidden on `side` is never part of the result, nor is anything below it. The other
///   side's flag doesn't matter.
/// - A row with no item on `side` adds nothing there, and a path that names no row is ignored.
///
/// *Unspecified*, decided here: the rule for selected descendants applies at every depth, and
/// subfolders are part of a folder's contents, so an operation can create or remove them.
pub fn resolve_selection(rows: &[Row], side: TreeSide, selected: &[&Path]) -> Resolved {
    let mut resolved = Resolved::default();
    walk(rows, side, Path::new(""), Path::new(""), selected, &mut resolved);
    resolved
}

/// `at` is the row-name path of `rows`' folder, `path` the same folder in `side`'s spelling.
fn walk(rows: &[Row], side: TreeSide, at: &Path, path: &Path, selected: &[&Path], out: &mut Resolved) {
    for row in rows {
        let at = at.join(&row.name);
        if !selected.iter().any(|s| s.starts_with(&at)) {
            continue;
        }
        let Some(item) = visible(row, side) else { continue };
        let path = path.join(&item.name);
        if item.kind != ItemKind::Folder {
            if selected.contains(&at.as_path()) {
                out.items.push((path, item.clone()));
            }
        } else if selected.iter().any(|s| s.strip_prefix(&at).is_ok_and(|below| names_row(&row.children, below))) {
            walk(&row.children, side, &at, &path, selected, out);
        } else if selected.contains(&at.as_path()) {
            contents(row, side, &path, out);
        }
    }
}

/// Whether `below`, a non-empty row-name path, names one of `rows` or a row under them, so a
/// stale or mistyped path doesn't open its folder and empty its resolution.
fn names_row(rows: &[Row], below: &Path) -> bool {
    let mut names = below.iter();
    let Some(first) = names.next() else { return false };
    rows.iter().find(|row| *row.name == *first).is_some_and(|row| {
        let rest = names.as_path();
        rest.as_os_str().is_empty() || names_row(&row.children, rest)
    })
}

/// The visible contents of the folder `row`, at `path` on `side`.
fn contents(row: &Row, side: TreeSide, path: &Path, out: &mut Resolved) {
    if row.scan != ScanState::Done || matches!(row.status, Status::Error(_)) {
        out.incomplete = true;
    }
    for child in &row.children {
        let Some(item) = visible(child, side) else { continue };
        let path = path.join(&item.name);
        out.items.push((path.clone(), item.clone()));
        if item.kind == ItemKind::Folder {
            contents(child, side, &path, out);
        }
    }
}

fn visible(row: &Row, side: TreeSide) -> Option<&Item> {
    match side {
        TreeSide::Left => row.left.as_ref().filter(|_| !row.left_hidden),
        TreeSide::Right => row.right.as_ref().filter(|_| !row.right_hidden),
    }
}
