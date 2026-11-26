//! Quick tests and the status model, on `MemorySource` fixtures aligned the way the engine aligns
//! them.

use std::path::Path;
use std::time::{Duration, SystemTime};

use otd_folder_engine::{Aligned, Alignment, Criteria, Status, align, quick_test};
use otd_source::{Attributes, DataSource, ErrorKind, Links, MemorySource};

use crate::fixture::{at, caps};

const BOTH: Criteria = Criteria {
    size: true,
    modified: true,
    tolerance: Duration::from_secs(2),
    filename_case: false,
    unix_permissions: false,
    unix_owner: false,
    unix_group: false,
    ignore_daylight_saving: false,
    ignore_time_zone: false,
};

fn ms(millis: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_millis(millis)
}

/// The one aligned row of two roots.
fn row(left: &MemorySource, right: &MemorySource) -> Aligned {
    let list = |source: &MemorySource| source.list(Path::new(""), Links::Report).expect("the root lists");
    let mut rows = align(list(left), list(right), Alignment::new(true));
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

/// (gap, tolerance, Ignore daylight saving, Ignore timezone, same), times in ms (E08-S4-T1). A
/// gap that isn't the same makes the later side newer; each case runs in both directions.
const OFFSETS: &[(u64, u64, bool, bool, bool)] = &[
    (1_000, 2_000, false, false, true),
    (3_600_000, 2_000, false, false, false),
    (3_600_000, 2_000, true, false, true),
    (3_600_000, 2_000, false, true, true),
    (7_200_000, 2_000, false, true, true),
    (7_200_000, 2_000, true, false, false),
    (36_000_000, 2_000, false, true, true),
    // The tolerance reaches only above an offset (E08-S4's decision): 3600 s to 3602 s at 2 s.
    (3_598_000, 2_000, true, false, false),
    (3_599_999, 2_000, true, false, false),
    (3_602_000, 2_000, true, false, true),
    (3_602_001, 2_000, true, false, false),
    (7_198_000, 2_000, false, true, false),
    (7_199_999, 2_000, false, true, false),
    (7_202_000, 2_000, false, true, true),
    (7_202_001, 2_000, false, true, false),
    // A gap short of a whole hour stays different with both on (E08-S4-T1).
    (3_599_000, 2_000, true, true, false),
    (1_800_000, 2_000, true, true, false),
];

#[test]
fn daylight_saving_and_time_zone_offsets_count_as_same_within_the_tolerance_and_only_when_on() {
    for (index, &(gap, tolerance, ignore_daylight_saving, ignore_time_zone, same)) in OFFSETS.iter().enumerate() {
        let criteria = Criteria { tolerance: Duration::from_millis(tolerance), ignore_daylight_saving, ignore_time_zone, ..BOTH };
        let (early, late) = (ms(10_000), ms(10_000 + gap));
        let expect = |newer| if same { Status::Same } else { newer };
        assert_eq!(status((5, late), (5, early), &criteria), expect(Status::LeftNewer), "row {index}, left later");
        assert_eq!(status((5, early), (5, late), &criteria), expect(Status::RightNewer), "row {index}, right later");
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

#[test]
fn names_that_differ_only_in_case_are_different_only_with_the_filename_case_test_on() {
    let source = |name| MemorySource::new(caps(false)).file(name, "same", at(1));
    let list = |source: &MemorySource| source.list(Path::new(""), Links::Report).expect("the root lists");
    let mut rows = align(list(&source("Readme.md")), list(&source("README.md")), Alignment::new(false));
    assert_eq!(rows.len(), 1, "a case-insensitive alignment pairs them: {rows:#?}");
    let paired = rows.remove(0);
    assert_eq!(quick_test(&paired, &BOTH), Status::Same);
    assert_eq!(quick_test(&paired, &Criteria { filename_case: true, ..BOTH }), Status::Different);
    let exact = row(&file(4, at(1)), &file(4, at(1)));
    assert_eq!(quick_test(&exact, &Criteria { filename_case: true, ..BOTH }), Status::Same);
}

#[test]
fn each_unix_metadata_difference_is_different_only_with_its_own_test_on() {
    let unix = |mode, owner: &str, group: &str| {
        let attributes = Attributes::Unix { mode, owner: owner.to_owned(), group: group.to_owned() };
        file(4, at(1)).attributes("f", attributes)
    };
    let base = unix(0o100644, "ann", "staff");
    let on = |permissions, owner, group| Criteria { unix_permissions: permissions, unix_owner: owner, unix_group: group, ..BOTH };
    let all = on(true, true, true);
    // (other side, the one test that sees it)
    let cases = [
        (unix(0o100664, "ann", "staff"), on(true, false, false)),
        (unix(0o101644, "ann", "staff"), on(true, false, false)),
        (unix(0o100644, "bob", "staff"), on(false, true, false)),
        (unix(0o100644, "ann", "wheel"), on(false, false, true)),
    ];
    for (index, (other, its_test)) in cases.iter().enumerate() {
        let pair = row(&base, other);
        assert_eq!(quick_test(&pair, &BOTH), Status::Same, "case {index} with every Unix test off");
        assert_eq!(quick_test(&pair, its_test), Status::Different, "case {index} with its test on");
        let others =
            Criteria { unix_permissions: !its_test.unix_permissions, unix_owner: !its_test.unix_owner, unix_group: !its_test.unix_group, ..BOTH };
        assert_eq!(quick_test(&pair, &others), Status::Same, "case {index} with only the other Unix tests on");
    }
    // Set-user-ID isn't one of the Permissions bits, and a side with no Unix mode gives nothing to compare.
    assert_eq!(quick_test(&row(&base, &unix(0o104644, "ann", "staff")), &all), Status::Same);
    assert_eq!(quick_test(&row(&base, &file(4, at(1))), &all), Status::Same);
}
