//! Display filters (E08-S2-T1): which rows a Folder Compare view shows, by row status. Unlike name
//! masks and file filters, which plug into [`crate::Visibility`] and hide items, a display filter
//! hides whole rows and needs no rescan to change. Which folders show is the folder display mode's
//! call (E08-S2-T3); this answers for a row by its own status.

use crate::Status;

/// The 13 display filters; a view has exactly one active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayFilter {
    /// Everything.
    All,
    /// Hides matches.
    Differences,
    /// Matches only.
    Same,
    /// Hides orphans.
    NoOrphans,
    /// Hides matches and orphans.
    DifferencesNoOrphans,
    /// Orphans only.
    Orphans,
    LeftNewer,
    RightNewer,
    LeftNewerAndLeftOrphans,
    RightNewerAndRightOrphans,
    LeftOrphans,
    RightOrphans,
    /// Hides every file.
    None,
}

impl DisplayFilter {
    /// Whether a row with `status` shows under this filter.
    ///
    /// *Unspecified*, decided here: a filter worded as "hides X" shows unknown and error rows, and
    /// one worded as "X only" hides them. So Show Differences and Show No Orphans keep rows not yet
    /// compared and unreadable ones in view, since neither is a known match or orphan.
    ///
    /// This answers for file rows. A folder pair's own status is [`Status::Unknown`], so the "X
    /// only" filters would hide every folder: folders follow the folder display mode instead.
    pub fn shows(self, status: &Status) -> bool {
        use Status as S;
        let orphan = matches!(status, S::LeftOrphan | S::RightOrphan);
        match self {
            Self::All => true,
            Self::Differences => *status != S::Same,
            Self::Same => *status == S::Same,
            Self::NoOrphans => !orphan,
            Self::DifferencesNoOrphans => *status != S::Same && !orphan,
            Self::Orphans => orphan,
            Self::LeftNewer => *status == S::LeftNewer,
            Self::RightNewer => *status == S::RightNewer,
            Self::LeftNewerAndLeftOrphans => matches!(status, S::LeftNewer | S::LeftOrphan),
            Self::RightNewerAndRightOrphans => matches!(status, S::RightNewer | S::RightOrphan),
            Self::LeftOrphans => *status == S::LeftOrphan,
            Self::RightOrphans => *status == S::RightOrphan,
            Self::None => false,
        }
    }
}
