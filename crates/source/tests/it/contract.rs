//! The contract suite against the in-memory reference source, and against broken fakes that
//! show each case guards what its name says.

use std::collections::BTreeSet;
use std::io::Read;
use std::path::Path;
use std::time::{Duration, SystemTime};

use otd_source::suite::{self, CaseResult, Outcome};
use otd_source::{Capabilities, DataSource, Delete, ErrorKind, Item, Links, MemorySource, Permissions, SourceError};

/// Between them, the two configurations give every capability flag both values. The fixture
/// sets `read_only`.
const CONFIGURATIONS: [Capabilities; 2] = [
    Capabilities {
        read_only: false,
        case_sensitive: true,
        recycle_bin: true,
        timestamp_resolution: Duration::ZERO,
        windows_attributes: false,
        unix_permissions: true,
    },
    Capabilities {
        read_only: false,
        case_sensitive: false,
        recycle_bin: false,
        timestamp_resolution: Duration::from_secs(2),
        windows_attributes: true,
        unix_permissions: false,
    },
];

fn configured(fixture: &MemorySource, caps: Capabilities) -> MemorySource {
    let read_only = fixture.capabilities().read_only;
    fixture.clone().with_capabilities(Capabilities { read_only, ..caps })
}

#[test]
fn the_memory_source_passes_every_contract_case_in_every_configuration() {
    for caps in CONFIGURATIONS {
        let results = suite::run(&mut |fixture: &MemorySource| Some(Box::new(configured(fixture, caps)) as Box<dyn DataSource>));
        let not_passed: Vec<&CaseResult> = results.iter().filter(|r| r.outcome != Outcome::Passed).collect();
        assert!(not_passed.is_empty(), "with {caps:?}: {not_passed:#?}");
    }
}

#[derive(Debug, Clone, Copy)]
enum Flaw {
    /// A listing fails as a whole when one item carries an error.
    AbortsListingOnABadItem,
    /// A read-only source refuses writes as an I/O error, not access denied.
    RefusesReadOnlyWritesAsIo,
    /// Says names are case-sensitive on a source that matches them regardless of case.
    MisreportsCaseSensitivity,
    /// Accepts a new modified time and keeps the old one.
    IgnoresSetModified,
    /// Can't list inside an expandable container.
    WontListContainers,
    /// Follows links even when asked to report them.
    AlwaysFollowsLinks,
    /// Reports a CRC that doesn't match the content.
    ReportsAWrongCrc,
}

struct Flawed {
    inner: MemorySource,
    flaw: Flaw,
}

impl DataSource for Flawed {
    fn capabilities(&self) -> Capabilities {
        let caps = self.inner.capabilities();
        match self.flaw {
            Flaw::MisreportsCaseSensitivity => Capabilities { case_sensitive: true, ..caps },
            _ => caps,
        }
    }

    fn list(&self, folder: &Path, links: Links) -> Result<Vec<Item>, SourceError> {
        match self.flaw {
            Flaw::WontListContainers if self.inner.stat(folder, Links::Follow).is_ok_and(|item| item.container) => {
                Err(SourceError::new(ErrorKind::Unsupported, "no containers"))
            }
            Flaw::AlwaysFollowsLinks => self.inner.list(folder, Links::Follow),
            Flaw::AbortsListingOnABadItem => {
                let items = self.inner.list(folder, links)?;
                match items.iter().find_map(|item| item.error.clone()) {
                    Some(error) => Err(error),
                    None => Ok(items),
                }
            }
            _ => self.inner.list(folder, links),
        }
    }

    fn stat(&self, path: &Path, links: Links) -> Result<Item, SourceError> {
        let item = self.inner.stat(path, links)?;
        match self.flaw {
            Flaw::ReportsAWrongCrc => Ok(Item { crc32: item.crc32.map(|crc| crc ^ 1), ..item }),
            _ => Ok(item),
        }
    }

    fn open_read(&self, path: &Path) -> Result<Box<dyn Read + Send>, SourceError> {
        self.inner.open_read(path)
    }

    fn write(&self, path: &Path, content: &mut dyn Read) -> Result<(), SourceError> {
        let result = self.inner.write(path, content);
        match (self.flaw, result) {
            (Flaw::RefusesReadOnlyWritesAsIo, Err(e)) if e.kind == ErrorKind::AccessDenied => Err(SourceError { kind: ErrorKind::Io, ..e }),
            (_, result) => result,
        }
    }

    fn create_folder(&self, path: &Path) -> Result<(), SourceError> {
        self.inner.create_folder(path)
    }

    fn delete(&self, path: &Path, how: Delete) -> Result<(), SourceError> {
        self.inner.delete(path, how)
    }

    fn rename(&self, path: &Path, new_name: &str) -> Result<(), SourceError> {
        self.inner.rename(path, new_name)
    }

    fn set_modified(&self, path: &Path, time: SystemTime) -> Result<(), SourceError> {
        match self.flaw {
            // Sets the time the item already has, so it still refuses where a real set would.
            Flaw::IgnoresSetModified => {
                let old = self.inner.stat(path, Links::Report)?.modified.unwrap_or(SystemTime::UNIX_EPOCH);
                self.inner.set_modified(path, old)
            }
            _ => self.inner.set_modified(path, time),
        }
    }

    fn set_permissions(&self, path: &Path, permissions: Permissions) -> Result<(), SourceError> {
        self.inner.set_permissions(path, permissions)
    }
}

#[test]
fn each_broken_fake_fails_exactly_the_cases_that_guard_its_flaw() {
    let insensitive = CONFIGURATIONS[1];
    let expectations: [(Flaw, &[&str]); 7] = [
        (
            Flaw::AbortsListingOnABadItem,
            &[
                "a_link_loop_gives_that_item_an_error_and_the_listing_still_returns",
                "an_unreadable_item_carries_its_error_and_the_rest_of_the_listing_returns",
            ],
        ),
        (Flaw::RefusesReadOnlyWritesAsIo, &["a_read_only_source_refuses_every_write_and_changes_nothing"]),
        (Flaw::MisreportsCaseSensitivity, &["names_match_regardless_of_case_only_on_a_case_insensitive_source"]),
        (
            Flaw::IgnoresSetModified,
            &["a_modified_time_set_reads_back_within_the_timestamp_resolution", "a_copy_keeps_content_and_modified_time_within_and_across_sources"],
        ),
        (Flaw::WontListContainers, &["a_container_lists_its_children_like_a_folder"]),
        (Flaw::AlwaysFollowsLinks, &["links_report_their_target_unless_followed"]),
        (Flaw::ReportsAWrongCrc, &["a_source_provided_crc_matches_the_content"]),
    ];
    for (flaw, expected) in expectations {
        let results =
            suite::run(&mut |fixture: &MemorySource| Some(Box::new(Flawed { inner: configured(fixture, insensitive), flaw }) as Box<dyn DataSource>));
        let failed: BTreeSet<&str> = results.iter().filter(|r| matches!(r.outcome, Outcome::Failed(_))).map(|r| r.name).collect();
        assert_eq!(failed, expected.iter().copied().collect::<BTreeSet<_>>(), "{flaw:?}: {results:#?}");
        assert!(results.iter().all(|r| !matches!(r.outcome, Outcome::Skipped(_))), "{flaw:?} skipped a case: {results:#?}");
    }
}

#[test]
fn copying_a_folder_into_itself_copies_what_was_there_and_ends() {
    let source = MemorySource::new(CONFIGURATIONS[0]).file("dir/a.txt", "a", SystemTime::UNIX_EPOCH);
    otd_source::copy(&source, Path::new("dir"), &source, Path::new("dir/inner")).expect("the copy succeeds");
    let copied: Vec<String> = source.list(Path::new("dir/inner"), Links::Report).expect("the copy lists").into_iter().map(|item| item.name).collect();
    assert_eq!(copied, ["a.txt"]);
}
