//! Quick tests and the status model, on `MemorySource` fixtures aligned the way the engine aligns
//! them.

use std::path::Path;
use std::time::{Duration, SystemTime};

use otd_folder_engine::{Aligned, Criteria, Status, align, quick_test};
use otd_source::{DataSource, ErrorKind, Links, MemorySource};

use crate::fixture::{at, caps};

const BOTH: Criteria = Criteria { size: true, modified: true, tolerance: Duration::from_secs(2) };

fn ms(millis: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_millis(millis)
}

/// The one aligned row of two roots.
fn row(left: &MemorySource, right: &MemorySource) -> Aligned {
    let list = |source: &MemorySource| source.list(Path::new(""), Links::Report).expect("the root lists");
    let mut rows = align(list(left), list(right), true);
    assert_eq!(rows.len(), 1, "{rows:#?}");
    rows.remove(0)
}

/// A root holding one file `f` of `size` bytes, modified at `modified`.
fn file(size: usize, modified: SystemTime) -> MemorySource {
    MemorySource::new(caps(true)).file("f", vec![b'x'; size], modified)
}

fn status(left: (usize, SystemTime), right: (usize, SystemTime), criteria: &Criteria) -> Status {
    quick_test(&row(&file(left.0, left.1), &file(right.0, right.1)), criteria)
}

/// (left size, left time, right size, right time, tolerance, expected), times in ms. Each time
/// case runs in both directions.
const TABLE: &[(usize, u64, usize, u64, u64, Status)] = &[
    (5, 10_000, 5, 10_000, 2_000, Status::Same),
    (5, 10_000, 6, 10_000, 2_000, Status::Different),
    (5, 11_999, 5, 10_000, 2_000, Status::Same),
    (5, 12_000, 5, 10_000, 2_000, Status::Same),
    (5, 12_001, 5, 10_000, 2_000, Status::LeftNewer),
    (5, 10_000, 5, 12_000, 2_000, Status::Same),
    (5, 10_000, 5, 12_001, 2_000, Status::RightNewer),
    (5, 10_000, 5, 10_000, 0, Status::Same),
    (5, 10_001, 5, 10_000, 0, Status::LeftNewer),
    (5, 10_000, 5, 10_001, 0, Status::RightNewer),
    (5, 12_000, 9, 10_000, 2_000, Status::Different),
    (5, 12_001, 9, 10_000, 2_000, Status::LeftNewer),
    (9, 10_000, 5, 12_001, 2_000, Status::RightNewer),
];

#[test]
fn size_and_time_deltas_against_the_tolerance_give_the_expected_status_including_exactly_at_it() {
    for (index, (left_size, left_time, right_size, right_time, tolerance, expected)) in TABLE.iter().enumerate() {
        let criteria = Criteria { tolerance: Duration::from_millis(*tolerance), ..BOTH };
        let got = status((*left_size, ms(*left_time)), (*right_size, ms(*right_time)), &criteria);
        assert_eq!(&got, expected, "table row {index}");
    }
}

#[test]
fn a_criterion_that_is_off_never_decides_the_status() {
    let differ_in_both = ((5, at(100)), (9, at(10)));
    let only = |size, modified| status(differ_in_both.0, differ_in_both.1, &Criteria { size, modified, ..BOTH });
    assert_eq!(only(true, true), Status::LeftNewer);
    assert_eq!(only(true, false), Status::Different);
    assert_eq!(only(false, true), Status::LeftNewer);
    assert_eq!(only(false, false), Status::Same);
    let differ_in_time_only = status((5, at(100)), (5, at(10)), &Criteria { modified: false, ..BOTH });
    assert_eq!(differ_in_time_only, Status::Same);
}

#[test]
fn an_item_on_one_side_only_is_an_orphan_of_that_side() {
    let empty = MemorySource::new(caps(true));
    assert_eq!(quick_test(&row(&file(1, at(1)), &empty), &BOTH), Status::LeftOrphan);
    assert_eq!(quick_test(&row(&empty, &file(1, at(1))), &BOTH), Status::RightOrphan);
}

/// Unreadable takes precedence over orphan and over the quick tests, and keeps the message.
#[test]
fn an_unreadable_item_makes_the_row_an_error_that_keeps_the_sources_message() {
    let locked = file(1, at(1)).unreadable("f");
    let message = locked.list(Path::new(""), Links::Report).expect("the root lists")[0].error.clone().expect("f carries its error").message;
    let empty = MemorySource::new(caps(true));
    for (left, right) in [(&locked, &file(1, at(1))), (&file(1, at(1)), &locked), (&locked, &empty), (&empty, &locked)] {
        match quick_test(&row(left, right), &BOTH) {
            Status::Error(error) => assert!(error.kind == ErrorKind::AccessDenied && error.message == message, "{error:?}"),
            other => panic!("expected an error, got {other:?}"),
        }
    }
}

#[test]
fn a_file_and_a_folder_with_the_same_name_are_different() {
    let folder = MemorySource::new(caps(true)).folder("f", at(1));
    assert_eq!(quick_test(&row(&file(0, at(1)), &folder), &BOTH), Status::Different);
    assert_eq!(quick_test(&row(&folder, &file(0, at(1))), &BOTH), Status::Different);
}

/// A folder's result comes from its contents (aggregation), never from its own timestamp.
#[test]
fn a_folders_own_timestamp_never_makes_it_newer() {
    let folder = |modified| MemorySource::new(caps(true)).folder("f", modified);
    assert_eq!(quick_test(&row(&folder(at(1_000)), &folder(at(1))), &BOTH), Status::Unknown);
    assert_eq!(quick_test(&row(&folder(at(1)), &folder(at(1_000))), &BOTH), Status::Unknown);
}
