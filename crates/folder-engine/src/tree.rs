//! The compared tree, folder aggregation and the per-side visibility predicate (E03-S2-T6).
//!
//! [`compare_trees`] aligns two scanned trees folder by folder into [`Row`]s, gives each row its quick
//! status and flags each side the visibility predicate hides. [`Row::cues`] then summarises a
//! folder's visible contents: every status found anywhere below it, several at once.
//!
//! Hidden rows stay in the tree, flagged, so later stages can still reach them. A row is left out
//! of aggregation only when it is hidden on every side it has an item on; hidden on one side
//! only, it stays visible and counts.

use std::path::{Path, PathBuf};

use otd_source::{Item, ItemKind, SourceError};

use crate::{Criteria, ScanState, Status, TreeScan, align, quick_test};

/// One side of a comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeSide {
    Left,
    Right,
}

/// The per-side visibility predicate: whether `item`, at `path` on `side` (relative to that side's
/// base, in that side's own spelling), is visible. Name masks and file filters plug in here.
pub type Visibility<'a> = dyn Fn(TreeSide, &Path, &Item) -> bool + 'a;

/// One aligned name in the compared tree.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// The left item's name when there is one, otherwise the right's.
    pub name: String,
    pub left: Option<Item>,
    pub right: Option<Item>,
    /// From the quick tests. Two folders are [`Status::Unknown`]: their result is their
    /// [`Row::cues`]. A folder that can't be listed makes the row an error with its message. An
    /// item facing a folder whose listing isn't done, or failed, is unknown, not an orphan: nothing
    /// is known yet of what that side holds.
    pub status: Status,
    /// The predicate hides the left item, or the folder it is in is hidden on the left.
    pub left_hidden: bool,
    pub right_hidden: bool,
    /// The least far scanned of the row's folders; done for a row with none.
    pub scan: ScanState,
    /// The aligned contents, when either side is a folder.
    pub children: Vec<Row>,
}

/// The set of statuses a folder's visible contents hold, such as "contains newer" plus
/// "contains orphans".
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Cues(u8);

impl Cues {
    /// The cues of these rows, as if they were one folder's contents.
    pub fn of(rows: &[Row]) -> Self {
        let mut cues = Self::default();
        for row in rows.iter().filter(|row| !row.hidden()) {
            cues.0 |= Self::of(&row.children).0;
            // A compared folder pair adds only what is below it: its own size and time never
            // count. Until both of its listings are done, it adds unknown.
            if !(row.folder_pair() && row.status == Status::Unknown && row.scan == ScanState::Done) {
                cues.0 |= bit(&row.status);
            }
        }
        cues
    }

    /// Whether some visible item below has this status. Any [`Status::Error`] matches any other.
    pub fn contains(self, status: &Status) -> bool {
        self.0 & bit(status) != 0
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}

fn bit(status: &Status) -> u8 {
    1 << match status {
        Status::Same => 0,
        Status::Different => 1,
        Status::LeftNewer => 2,
        Status::RightNewer => 3,
        Status::LeftOrphan => 4,
        Status::RightOrphan => 5,
        Status::Unknown => 6,
        Status::Error(_) => 7,
    }
}

impl Row {
    /// Hidden on every side it has an item on, so out of aggregation and selection.
    pub fn hidden(&self) -> bool {
        (self.left.is_none() || self.left_hidden) && (self.right.is_none() || self.right_hidden)
    }

    /// The folder's cues, from its visible contents. Computed on each call, so a status changed
    /// after the quick tests (by a content test, say) shows at once.
    ///
    /// ponytail: walks the whole subtree each call; cache per folder if a view needs it per frame.
    pub fn cues(&self) -> Cues {
        Cues::of(&self.children)
    }

    fn folder_pair(&self) -> bool {
        let folder = |item: &Option<Item>| item.as_ref().is_some_and(|item| item.kind == ItemKind::Folder);
        folder(&self.left) && folder(&self.right)
    }
}

/// The compared tree of two scans: the rows of their base folders, as far as each scan has got.
/// Call it again after more listings to see them. Until a base is listed, the other side's items
/// are unknown; why a base couldn't be listed is on its scan (`TreeScan::folder` of the empty path).
///
/// *Unspecified*, decided here: the contents of a folder hidden on a side are hidden on that side
/// too; a file against a folder has the folder's contents as orphans of the folder's side; an
/// orphan folder adds its orphan cue even when it is empty.
pub fn compare_trees(left: &TreeScan, right: &TreeScan, criteria: &Criteria, case_sensitive: bool, visible: &Visibility<'_>) -> Vec<Row> {
    let trees = Trees { left, right, criteria, case_sensitive, visible };
    let base = Some(Path::new(""));
    trees.rows([trees.listing(TreeSide::Left, base), trees.listing(TreeSide::Right, base)], [false, false], [false, false])
}

struct Trees<'a> {
    left: &'a TreeScan,
    right: &'a TreeScan,
    criteria: &'a Criteria,
    case_sensitive: bool,
    visible: &'a Visibility<'a>,
}

/// One side of a folder: where it is, its items, its scan state and its listing error.
struct Listing {
    dir: Option<PathBuf>,
    items: Vec<Item>,
    state: ScanState,
    error: Option<SourceError>,
}

impl Trees<'_> {
    /// `dir` is `None` when the side has no folder here.
    fn listing(&self, side: TreeSide, dir: Option<&Path>) -> Listing {
        let Some(dir) = dir else { return Listing { dir: None, items: Vec::new(), state: ScanState::Done, error: None } };
        let scan = match side {
            TreeSide::Left => self.left,
            TreeSide::Right => self.right,
        };
        match scan.folder(dir) {
            Some(folder) => Listing { dir: Some(dir.to_path_buf()), items: folder.items, state: folder.state, error: folder.error },
            None => Listing { dir: Some(dir.to_path_buf()), items: Vec::new(), state: ScanState::Unscanned, error: None },
        }
    }

    /// `unknown` marks a side whose contents here aren't known: this folder, or one above it, isn't
    /// listed yet or couldn't be. A side with no folder here (`dir` is `None`) is known to be empty.
    ///
    /// ponytail: recursive, so as deep as the tree; paths' own length limits keep that shallow.
    fn rows(&self, [left, right]: [Listing; 2], hidden: [bool; 2], unknown: [bool; 2]) -> Vec<Row> {
        let unlisted = |l: &Listing| l.dir.is_some() && (l.state != ScanState::Done || l.error.is_some());
        let unknown = [unknown[0] || unlisted(&left), unknown[1] || unlisted(&right)];
        let mut rows = Vec::new();
        for aligned in align(left.items, right.items, self.case_sensitive) {
            let mut status = match quick_test(&aligned, self.criteria) {
                Status::LeftOrphan if unknown[1] => Status::Unknown,
                Status::RightOrphan if unknown[0] => Status::Unknown,
                status => status,
            };
            // Each side descends by its own item's spelling, which may differ in case.
            let path = |dir: &Option<PathBuf>, item: &Option<Item>| Some(dir.as_ref()?.join(&item.as_ref()?.name));
            let paths = [path(&left.dir, &aligned.left), path(&right.dir, &aligned.right)];
            let flag = |side, at: usize, item: &Option<Item>| {
                hidden[at] || matches!((item, &paths[at]), (Some(item), Some(path)) if !(self.visible)(side, path, item))
            };
            let row_hidden = [flag(TreeSide::Left, 0, &aligned.left), flag(TreeSide::Right, 1, &aligned.right)];
            let folder_path =
                |at: usize, item: &Option<Item>| paths[at].as_deref().filter(|_| item.as_ref().is_some_and(|item| item.kind == ItemKind::Folder));
            let below = [self.listing(TreeSide::Left, folder_path(0, &aligned.left)), self.listing(TreeSide::Right, folder_path(1, &aligned.right))];
            if let Some(error) = below.iter().find_map(|listing| listing.error.clone())
                && !matches!(status, Status::Error(_))
            {
                status = Status::Error(error);
            }
            let scan = if below.iter().any(|l| l.state == ScanState::Unscanned) {
                ScanState::Unscanned
            } else if below.iter().any(|l| l.state == ScanState::Pending) {
                ScanState::Pending
            } else {
                ScanState::Done
            };
            let children = if below.iter().any(|l| l.dir.is_some()) { self.rows(below, row_hidden, unknown) } else { Vec::new() };
            rows.push(Row {
                name: aligned.name,
                left: aligned.left,
                right: aligned.right,
                status,
                left_hidden: row_hidden[0],
                right_hidden: row_hidden[1],
                scan,
                children,
            });
        }
        rows
    }
}
