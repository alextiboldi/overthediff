//! Display filters: the row status × filter visibility table (E08-S2-T1).

use otd_folder_engine::{DisplayFilter as F, Status};
use otd_source::{ErrorKind, SourceError};

/// Columns, in order: same, left newer, right newer, different, left orphan, right orphan, then
/// the two the story leaves open: unknown and error. `x` shows the row, `.` hides it.
const TABLE: &[(F, &str)] = &[
    (F::All, "xxxxxxxx"),
    (F::Differences, ".xxxxxxx"),
    (F::Same, "x......."),
    (F::NoOrphans, "xxxx..xx"),
    (F::DifferencesNoOrphans, ".xxx..xx"),
    (F::Orphans, "....xx.."),
    (F::LeftNewer, ".x......"),
    (F::RightNewer, "..x....."),
    (F::LeftNewerAndLeftOrphans, ".x..x..."),
    (F::RightNewerAndRightOrphans, "..x..x.."),
    (F::LeftOrphans, "....x..."),
    (F::RightOrphans, ".....x.."),
    (F::None, "........"),
];

#[test]
fn every_filter_shows_exactly_the_specified_statuses() {
    let statuses = [
        Status::Same,
        Status::LeftNewer,
        Status::RightNewer,
        Status::Different,
        Status::LeftOrphan,
        Status::RightOrphan,
        Status::Unknown,
        Status::Error(SourceError::new(ErrorKind::AccessDenied, "locked")),
    ];
    assert_eq!(F::ALL.to_vec(), TABLE.iter().map(|(filter, _)| *filter).collect::<Vec<_>>(), "ALL lists every filter once, in table order");
    for (filter, cells) in TABLE {
        for (status, cell) in statuses.iter().zip(cells.chars()) {
            assert_eq!(filter.shows(status), cell == 'x', "{filter:?} on {status:?}");
        }
    }
}
