//! Quick tests and the status model (E03-S2-T4): one aligned row's status from the two items'
//! metadata alone, without reading content.

use std::time::{Duration, SystemTime};

use otd_source::{Attributes, Item, ItemKind, SourceError};

use crate::Aligned;

/// The default modified-time tolerance (E08-S1-T1): 2 seconds, so a copy on a FAT volume or in a
/// zip archive, which keep times to 2 seconds, still compares same. Times exactly this far apart
/// count as the same.
pub const DEFAULT_TOLERANCE: Duration = Duration::from_secs(2);

/// Which quick tests run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Criteria {
    /// Different sizes make a pair different.
    pub size: bool,
    /// Modified times further apart than `tolerance` make the later one newer.
    pub modified: bool,
    /// Times this close or closer count as the same.
    pub tolerance: Duration,
    /// Names that differ only in capitalisation (`Readme.md` against `README.md`, paired by a
    /// case-insensitive alignment) make a pair different.
    pub filename_case: bool,
    /// Different Unix permissions (user, group and other rwx, and the sticky bit) make a pair
    /// different. The Unix tests only look at items whose source reports a Unix mode on both sides.
    pub unix_permissions: bool,
    /// A different Unix owner makes a pair different. The owner and group compare as the source
    /// reports them: a name where the system has one for the ID, otherwise the ID itself.
    pub unix_owner: bool,
    /// A different Unix group makes a pair different.
    pub unix_group: bool,
}

/// The mode bits the Permissions test compares: rwx for user, group and other, and sticky.
/// Set-user-ID, set-group-ID and the file-type bits are left out, as the story lists them.
const PERMISSION_BITS: u32 = 0o1777;

/// A row's status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Same,
    Different,
    LeftNewer,
    RightNewer,
    LeftOrphan,
    RightOrphan,
    /// Not compared yet.
    Unknown,
    /// An item couldn't be read. The error keeps the source's message.
    Error(SourceError),
}

/// The status the quick tests give one aligned row.
///
/// - An unreadable item on either side makes the row an error, orphan or not: the story's error
///   status is "unreadable item". With both sides unreadable it keeps the left's error (each item
///   still carries its own). *Unspecified.*
/// - An item on one side only is an orphan of that side.
/// - A file against a folder (or a link) with the same name is different.
/// - Two folders are unknown: a folder's own size and time never decide its status, and its
///   result comes from its contents, which these tests don't look at.
/// - Otherwise a time gap past the tolerance makes the later side newer, even when the sizes
///   differ too, since newer is also a difference and keeps the direction. *Unspecified.* Then
///   different sizes, names that differ only in case, or different Unix permissions, owner or
///   group, each where its test is on, make the row different, and anything else is same. Two
///   links compare like two files, by the metadata the listing reported; their targets aren't compared. *Unspecified.*
pub fn quick_test(row: &Aligned, criteria: &Criteria) -> Status {
    if let Some(error) = [&row.left, &row.right].into_iter().flatten().find_map(|item| item.error.clone()) {
        return Status::Error(error);
    }
    let (left, right) = match (&row.left, &row.right) {
        (Some(left), Some(right)) => (left, right),
        (Some(_), None) => return Status::LeftOrphan,
        (None, Some(_)) => return Status::RightOrphan,
        (None, None) => return Status::Unknown,
    };
    if left.kind != right.kind {
        return Status::Different;
    }
    if left.kind == ItemKind::Folder {
        return Status::Unknown;
    }
    if criteria.modified
        && let (Some(l), Some(r)) = (left.modified, right.modified)
        && let Some(newer) = newer(l, r, criteria.tolerance)
    {
        return newer;
    }
    if (criteria.size && left.size != right.size)
        || (criteria.filename_case && differ_only_in_case(&left.name, &right.name))
        || unix_differs(left, right, criteria)
    {
        return Status::Different;
    }
    Status::Same
}

fn differ_only_in_case(left: &str, right: &str) -> bool {
    // ponytail: Unicode lower-casing, as alignment uses, not the OS's own case table.
    left != right && left.to_lowercase() == right.to_lowercase()
}

/// Whether a Unix test that is on sees a difference. A side without a Unix mode (a Windows
/// source, or one that reports no attributes) gives the Unix tests nothing to compare, so they
/// don't make the pair different. *Unspecified.*
fn unix_differs(left: &Item, right: &Item, criteria: &Criteria) -> bool {
    let (Some(Attributes::Unix { mode: lm, owner: lo, group: lg }), Some(Attributes::Unix { mode: rm, owner: ro, group: rg })) =
        (&left.attributes, &right.attributes)
    else {
        return false;
    };
    (criteria.unix_permissions && lm & PERMISSION_BITS != rm & PERMISSION_BITS)
        || (criteria.unix_owner && lo != ro)
        || (criteria.unix_group && lg != rg)
}

/// Which side is newer when the times are further apart than `tolerance`.
fn newer(left: SystemTime, right: SystemTime, tolerance: Duration) -> Option<Status> {
    match left.duration_since(right) {
        Ok(gap) if gap > tolerance => Some(Status::LeftNewer),
        Ok(_) => None,
        Err(err) if err.duration() > tolerance => Some(Status::RightNewer),
        Err(_) => None,
    }
}
