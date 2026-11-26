//! Name filters (E08-S3-T1): a session's name masks, an A.1 mask list such as `*.cs;-obj\;-bin\`,
//! fed into [`crate::compare_trees`]' visibility hook.
//!
//! [`NameFilter::visible`] is the hook: a file shows when the list includes it, a folder unless the
//! list excludes it. A folder no folder mask includes is shown only when something inside it is
//! (`otd_mask::Folder::IfContainsIncluded`), which a per-item hook can't know, so the hook lets it
//! through and [`NameFilter::hide_empty_folders`] hides it once its contents are known. Call both,
//! in that order. The flags they set are what aggregation, the view and selection resolution read,
//! so an excluded item takes no part in the comparison.
//!
//! *Unspecified*, decided here:
//! - Each side matches with its own data source's case rule.
//! - A link is matched as a file: the scan reports links without following them.
//! - A folder shown only for its contents is hidden on each side by what that side shows inside
//!   it, so selecting it there never resolves to a folder that holds only filtered items. Its row
//!   still shows while the other side's folder does.
//! - It also stays shown while its contents aren't fully known (not listed yet, or unreadable), so
//!   opening it can still find what the masks include.

use std::borrow::Cow;
use std::path::{Path, PathBuf};

use otd_mask::{Case, Folder, MaskList, MaskListError};
use otd_source::{Item, ItemKind};

use crate::{Row, ScanState, Status, TreeSide};

/// A compiled name filter, one mask list per side. The default shows everything.
#[derive(Clone, Debug, Default)]
pub struct NameFilter {
    masks: [MaskList; 2],
}

impl NameFilter {
    /// Compiles `masks` for each side, with that side's case rule (`[left, right]`). An empty list
    /// shows everything. Whether it compiles doesn't depend on case.
    pub fn new(masks: &str, case_sensitive: [bool; 2]) -> Result<Self, MaskListError> {
        let case = |sensitive| if sensitive { Case::Sensitive } else { Case::Insensitive };
        Ok(Self { masks: [MaskList::new(masks, case(case_sensitive[0]))?, MaskList::new(masks, case(case_sensitive[1]))?] })
    }

    /// The visibility hook: whether `item`, at `path` on `side`, passes the masks.
    pub fn visible(&self, side: TreeSide, path: &Path, item: &Item) -> bool {
        let masks = self.side(side);
        match item.kind {
            ItemKind::Folder => masks.folder(&names(path)) != Folder::Excluded,
            ItemKind::File | ItemKind::Link => masks.includes_file(&names(path)),
        }
    }

    /// Hides, on each side, the folders shown only for their contents that show nothing inside on
    /// that side. Works from the deepest folders up, so a chain of such folders goes at once.
    pub fn hide_empty_folders(&self, rows: &mut [Row]) {
        let base = Some(PathBuf::new());
        self.prune(rows, [&base, &base]);
    }

    fn side(&self, side: TreeSide) -> &MaskList {
        let [left, right] = &self.masks;
        match side {
            TreeSide::Left => left,
            TreeSide::Right => right,
        }
    }

    /// `dirs`: each side's folder of `rows`, in that side's spelling; `None` where it has none.
    // ponytail: recursive, as deep as the tree, like `compare_trees`.
    fn prune(&self, rows: &mut [Row], dirs: [&Option<PathBuf>; 2]) {
        for row in rows {
            let folder =
                |dir: &Option<PathBuf>, item: &Option<Item>| Some(dir.as_ref()?.join(&item.as_ref().filter(|i| i.kind == ItemKind::Folder)?.name));
            let folders = [folder(dirs[0], &row.left), folder(dirs[1], &row.right)];
            self.prune(&mut row.children, [&folders[0], &folders[1]]);
            // Contents not fully known yet may hold what the masks include.
            if row.scan != ScanState::Done || matches!(row.status, Status::Error(_)) {
                continue;
            }
            let empty = [(TreeSide::Left, 0), (TreeSide::Right, 1)].map(|(side, at)| {
                !row.children.iter().any(|child| shows(child, side))
                    && folders[at].as_ref().is_some_and(|path| self.side(side).folder(&names(path)) == Folder::IfContainsIncluded)
            });
            row.left_hidden |= empty[0];
            row.right_hidden |= empty[1];
        }
    }
}

/// Whether `row` has an item on `side` that isn't hidden there.
fn shows(row: &Row, side: TreeSide) -> bool {
    match side {
        TreeSide::Left => row.left.is_some() && !row.left_hidden,
        TreeSide::Right => row.right.is_some() && !row.right_hidden,
    }
}

/// `path`'s names from the base down. Item names are UTF-8, so nothing is lost.
fn names(path: &Path) -> Vec<Cow<'_, str>> {
    path.iter().map(|name| name.to_string_lossy()).collect()
}
