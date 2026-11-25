//! The engine's fixture is `otd_source::MemorySource`. These cases check it for the operations the
//! engine uses (list, stat, read and a source-provided CRC), including what the E04 contract suite
//! doesn't pin.

use std::io::Read;
use std::path::Path;
use std::time::{Duration, SystemTime};

use otd_source::suite::{self, Group, Outcome};
use otd_source::{Capabilities, DataSource, ErrorKind, Links, MemorySource};

pub(crate) fn at(seconds: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)
}

pub(crate) fn caps(case_sensitive: bool) -> Capabilities {
    Capabilities {
        read_only: false,
        case_sensitive,
        recycle_bin: false,
        timestamp_resolution: Duration::ZERO,
        windows_attributes: false,
        unix_permissions: false,
    }
}

#[test]
fn the_fixture_passes_the_contract_cases_for_listing_and_reading_on_either_case_rule() {
    for case_sensitive in [true, false] {
        let results = suite::run(&mut |spec: &MemorySource| {
            let read_only = spec.capabilities().read_only;
            Some(Box::new(spec.clone().with_capabilities(Capabilities { read_only, ..caps(case_sensitive) })) as Box<dyn DataSource>)
        });
        let used: Vec<_> = results.iter().filter(|r| r.group != Group::Write).collect();
        assert!(!used.is_empty() && used.iter().all(|r| r.outcome == Outcome::Passed), "case-sensitive {case_sensitive}: {used:#?}");
    }
}

#[test]
fn an_unreadable_file_lists_with_its_message_and_refuses_reading_while_its_neighbours_read() {
    let source = MemorySource::new(caps(true)).file("a.txt", "a", at(1)).file("locked.txt", "secret", at(1)).unreadable("locked.txt");
    let items = source.list(Path::new(""), Links::Report).expect("the root lists");
    let locked = items.iter().find(|item| item.name == "locked.txt").expect("locked.txt is listed");
    let error = locked.error.as_ref().expect("locked.txt carries its error");
    assert!(error.kind == ErrorKind::AccessDenied && !error.message.is_empty(), "{error:?}");
    let refused = source.open_read(Path::new("locked.txt")).err().map(|e| e.kind);
    assert_eq!(refused, Some(ErrorKind::AccessDenied));
    let mut content = Vec::new();
    source.open_read(Path::new("a.txt")).expect("a.txt opens").read_to_end(&mut content).expect("a.txt reads");
    assert_eq!(content, b"a");
}

/// A declared CRC that disagrees with the content lets a content test show it used the CRC
/// without reading.
#[test]
fn a_declared_crc_lists_as_declared_even_when_the_content_disagrees() {
    let wrong = 0x1234_5678;
    let source = MemorySource::new(caps(true)).file("given.txt", "123456789", at(1)).crc32("given.txt", wrong).file("plain.txt", "x", at(1));
    let items = source.list(Path::new(""), Links::Report).expect("the root lists");
    let crc = |name: &str| items.iter().find(|item| item.name == name).and_then(|item| item.crc32);
    assert_eq!((crc("given.txt"), crc("plain.txt")), (Some(wrong), None));
}
