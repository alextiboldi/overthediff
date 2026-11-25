//! Content tests and how their result meets the quick tests', on `MemorySource` fixtures.

use std::path::Path;
use std::time::Duration;

use otd_folder_engine::{Aligned, Compared, ContentOptions, ContentResult, ContentTest, Criteria, Status, align, compare};
use otd_source::{DataSource, Delete, ErrorKind, Links, MemorySource};

use crate::fixture::{at, caps};

const BOTH: Criteria = Criteria { size: true, modified: true, tolerance: Duration::from_secs(2) };
const TESTS: [ContentTest; 2] = [ContentTest::Crc, ContentTest::Binary];

/// The CRC-32 check value: the CRC of `123456789` (ISO-HDLC, the zip and PNG CRC).
const CHECK: u32 = 0xCBF4_3926;

fn root(file: &str, content: impl Into<Vec<u8>>, modified: u64) -> MemorySource {
    MemorySource::new(caps(true)).file(file, content, at(modified))
}

fn rows(left: &MemorySource, right: &MemorySource) -> Vec<Aligned> {
    let list = |source: &MemorySource| source.list(Path::new(""), Links::Report).expect("the root lists");
    align(list(left), list(right), true)
}

fn run(row: &Aligned, left: &MemorySource, right: &MemorySource, options: ContentOptions) -> Compared {
    compare(row, &BOTH, &options, (left, Path::new("")), (right, Path::new("")))
}

/// The one row of two roots, compared.
fn one(left: &MemorySource, right: &MemorySource, options: ContentOptions) -> Compared {
    let rows = rows(left, right);
    assert_eq!(rows.len(), 1, "{rows:#?}");
    run(&rows[0], left, right, options)
}

fn with(test: ContentTest) -> ContentOptions {
    ContentOptions { test: Some(test), ..ContentOptions::default() }
}

#[test]
fn equal_content_with_different_times_is_same_with_override_on_and_newer_with_it_off() {
    let (left, right) = (root("f", "same bytes", 100), root("f", "same bytes", 10));
    for test in TESTS {
        let on = one(&left, &right, with(test));
        assert_eq!(on, Compared { status: Status::Same, content: Some(ContentResult::BinarySame) }, "{test:?}");
        let off = one(&left, &right, ContentOptions { override_quick_results: false, ..with(test) });
        assert_eq!(off, Compared { status: Status::LeftNewer, content: Some(ContentResult::BinarySame) }, "{test:?}");
    }
}

/// The files are deleted after listing, so any read would make the row an error.
#[test]
fn a_crc_the_source_provides_is_used_as_is_without_reading_the_content() {
    let left = root("f", "abc", 1).crc32("f", 7);
    let right = root("f", "xyz", 1).crc32("f", 7);
    let rows = rows(&left, &right);
    for source in [&left, &right] {
        source.delete(Path::new("f"), Delete::Permanently).expect("f deletes");
    }
    let got = run(&rows[0], &left, &right, with(ContentTest::Crc));
    assert_eq!(got, Compared { status: Status::Same, content: Some(ContentResult::BinarySame) });
}

#[test]
fn without_a_source_crc_the_content_s_crc_32_is_computed_and_meets_a_provided_one() {
    let provided = root("f", "not read!", 1).crc32("f", CHECK);
    let content = |text| one(&provided, &root("f", text, 1), with(ContentTest::Crc)).content;
    assert_eq!(content("123456789"), Some(ContentResult::BinarySame));
    assert_eq!(content("123456780"), Some(ContentResult::BinaryDifferences));
}

/// Large enough that the comparison takes several reads, with the only difference in the last
/// byte. The binary test ignores provided CRCs, which here agree.
#[test]
fn binary_compares_every_byte_and_ignores_provided_crcs() {
    let bytes = vec![7u8; 200_000];
    let mut last_differs = bytes.clone();
    if let Some(byte) = last_differs.last_mut() {
        *byte = 8;
    }
    let content = |right: Vec<u8>| {
        let (left, right) = (root("f", bytes.clone(), 1).crc32("f", 1), root("f", right, 1).crc32("f", 1));
        one(&left, &right, with(ContentTest::Binary)).content
    };
    assert_eq!(content(bytes.clone()), Some(ContentResult::BinarySame));
    assert_eq!(content(last_differs), Some(ContentResult::BinaryDifferences));
}

/// Same size and time, different bytes: the quick tests call it same.
#[test]
fn skip_if_quick_same_runs_content_tests_only_for_pairs_the_quick_tests_call_different() {
    let (left, right) = (root("f", "abc", 1), root("f", "xyz", 1));
    let (newer_left, newer_right) = (root("f", "abc", 100), root("f", "abc", 1));
    for test in TESTS {
        let skip = ContentOptions { skip_if_quick_same: true, ..with(test) };
        assert_eq!(one(&left, &right, skip), Compared { status: Status::Same, content: None }, "{test:?}");
        let run = one(&left, &right, with(test));
        assert_eq!(run, Compared { status: Status::Different, content: Some(ContentResult::BinaryDifferences) }, "{test:?}");
        assert_eq!(one(&newer_left, &newer_right, skip).content, Some(ContentResult::BinarySame), "{test:?}");
    }
}

#[test]
fn a_content_mismatch_keeps_the_newer_side() {
    let (left, right) = (root("f", "abc", 1), root("f", "xyz", 100));
    for test in TESTS {
        assert_eq!(one(&left, &right, with(test)).status, Status::RightNewer, "{test:?}");
    }
}

/// Deleted after listing, so the read fails where the listing didn't.
#[test]
fn a_read_that_fails_makes_the_row_an_error_with_no_content_result() {
    let (left, right) = (root("f", "abc", 1), root("f", "abc", 1));
    let rows = rows(&left, &right);
    right.delete(Path::new("f"), Delete::Permanently).expect("f deletes");
    for test in TESTS {
        match run(&rows[0], &left, &right, with(test)) {
            Compared { status: Status::Error(error), content: None } => assert_eq!(error.kind, ErrorKind::NotFound, "{test:?}"),
            other => panic!("{test:?}: expected an error, got {other:?}"),
        }
    }
}

#[test]
fn only_two_files_get_a_content_result() {
    let file = root("f", "abc", 1);
    let folder = MemorySource::new(caps(true)).folder("f", at(1));
    let empty = MemorySource::new(caps(true));
    for (left, right) in [(&file, &empty), (&empty, &file), (&folder, &folder), (&file, &folder)] {
        assert_eq!(one(left, right, with(ContentTest::Binary)).content, None);
    }
}
