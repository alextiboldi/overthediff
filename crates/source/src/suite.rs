//! The contract test suite. It defines the data-source contract, and every provider runs it.
//!
//! [`run`] takes a harness: given a fixture, described as a [`MemorySource`], the harness returns a
//! fresh source of its own kind holding that tree, by whatever means it has (files in a temporary
//! folder, a built archive). It keeps its own capabilities except [`Capabilities::read_only`],
//! which it takes from the fixture. It returns `None` when it can't hold the fixture (links,
//! unreadable items, containers or read-only media it can't make), and that case is skipped with
//! the reason.
//!
//! Cases never panic: each returns an [`Outcome`], so a caller asserts on the results. Write
//! cases skip on a read-only source, so a provider that only reads passes the rest.

use std::io::{Cursor, Read};
use std::path::Path;
use std::time::{Duration, SystemTime};

use crate::{Attributes, Capabilities, DataSource, Delete, ErrorKind, Item, ItemKind, Links, MemorySource, Permissions, SourceError, copy};

/// Builds a source holding a fixture's tree, or `None` when this provider can't.
pub type Harness<'a> = dyn FnMut(&MemorySource) -> Option<Box<dyn DataSource>> + 'a;

/// Which operations a case uses, so a provider can say which groups it passes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    /// Listing and stat.
    Listing,
    /// Reading content.
    Read,
    /// Every change to a source.
    Write,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Passed,
    Failed(String),
    Skipped(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseResult {
    pub name: &'static str,
    pub group: Group,
    pub outcome: Outcome,
}

/// Runs every case against sources the harness builds.
pub fn run(harness: &mut Harness<'_>) -> Vec<CaseResult> {
    CASES
        .iter()
        .map(|&(name, group, case)| CaseResult {
            name,
            group,
            outcome: match case(harness) {
                Ok(()) => Outcome::Passed,
                Err(Stop::Fail(why)) => Outcome::Failed(why),
                Err(Stop::Skip(why)) => Outcome::Skipped(why),
            },
        })
        .collect()
}

type Case = fn(&mut Harness<'_>) -> Result<(), Stop>;

const CASES: &[(&str, Group, Case)] = &[
    ("a_listing_reports_each_child_with_its_name_kind_size_and_time", Group::Listing, listing),
    ("stat_agrees_with_the_listing_and_a_missing_path_is_not_found", Group::Listing, stat),
    ("links_report_their_target_unless_followed", Group::Listing, links),
    ("a_link_loop_gives_that_item_an_error_and_the_listing_still_returns", Group::Listing, link_loop),
    ("an_unreadable_item_carries_its_error_and_the_rest_of_the_listing_returns", Group::Listing, unreadable),
    ("a_container_lists_its_children_like_a_folder", Group::Listing, container),
    ("a_source_provided_crc_matches_the_content", Group::Listing, crc),
    ("names_match_regardless_of_case_only_on_a_case_insensitive_source", Group::Listing, case_sensitivity),
    ("content_reads_back_and_a_missing_file_is_not_found", Group::Read, read),
    ("written_content_reads_back_and_a_replacement_changes_it", Group::Write, write),
    ("a_created_folder_lists_as_an_empty_folder", Group::Write, create_folder),
    ("a_renamed_item_keeps_its_content_under_the_new_name", Group::Write, rename),
    ("a_permanent_delete_removes_a_file_and_a_folder_with_its_contents", Group::Write, delete),
    ("a_recycle_bin_delete_works_only_where_the_source_has_one", Group::Write, recycle_bin),
    ("a_modified_time_set_reads_back_within_the_timestamp_resolution", Group::Write, set_modified),
    ("attributes_and_permissions_are_kept_only_where_the_source_supports_them", Group::Write, set_permissions),
    ("a_read_only_source_refuses_every_write_and_changes_nothing", Group::Write, read_only),
    ("a_copy_keeps_content_and_modified_time_within_and_across_sources", Group::Write, copies),
];

/// The standard CRC-32 check value: the CRC of the ASCII digits `123456789`.
const CHECK_CRC: u32 = 0xCBF4_3926;

enum Stop {
    Fail(String),
    Skip(String),
}

impl From<SourceError> for Stop {
    fn from(e: SourceError) -> Self {
        Stop::Fail(format!("unexpected error: {e:?}"))
    }
}

fn at(seconds: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)
}

const T1: u64 = 1_600_000_000;
const T2: u64 = 1_600_000_100;

/// A fixture: writable, and case-sensitive so it can hold any names.
fn fixture() -> MemorySource {
    MemorySource::new(Capabilities {
        read_only: false,
        case_sensitive: true,
        recycle_bin: false,
        timestamp_resolution: Duration::ZERO,
        windows_attributes: false,
        unix_permissions: false,
    })
}

fn build(harness: &mut Harness<'_>, fixture: &MemorySource, needs: &str) -> Result<Box<dyn DataSource>, Stop> {
    harness(fixture).ok_or_else(|| Stop::Skip(format!("the harness can't build a source with {needs}")))
}

/// A source for a write case, which skips when the provider is read-only.
fn writable(harness: &mut Harness<'_>, fixture: &MemorySource) -> Result<Box<dyn DataSource>, Stop> {
    let source = build(harness, fixture, "files and folders")?;
    if source.capabilities().read_only {
        return Err(Stop::Skip("the source is read-only".to_owned()));
    }
    Ok(source)
}

fn ensure(holds: bool, what: impl FnOnce() -> String) -> Result<(), Stop> {
    if holds { Ok(()) } else { Err(Stop::Fail(what())) }
}

fn expect_error<T>(result: Result<T, SourceError>, kind: ErrorKind, what: &str) -> Result<(), Stop> {
    match result {
        Err(e) if e.kind == kind => Ok(()),
        Err(e) => Err(Stop::Fail(format!("{what}: expected {kind:?}, got {e:?}"))),
        Ok(_) => Err(Stop::Fail(format!("{what}: expected {kind:?}, but it succeeded"))),
    }
}

fn find<'a>(items: &'a [Item], name: &str) -> Result<&'a Item, Stop> {
    items.iter().find(|item| item.name == name).ok_or_else(|| Stop::Fail(format!("{name} is missing from the listing")))
}

fn names(items: &[Item]) -> Vec<&str> {
    let mut names: Vec<&str> = items.iter().map(|item| item.name.as_str()).collect();
    names.sort_unstable();
    names
}

fn read_all(source: &dyn DataSource, path: &str) -> Result<Vec<u8>, Stop> {
    let mut content = Vec::new();
    source.open_read(Path::new(path))?.read_to_end(&mut content).map_err(|e| Stop::Fail(format!("reading {path}: {e}")))?;
    Ok(content)
}

/// Whether a reported time is `expected` as the source can store it.
fn within_resolution(source: &dyn DataSource, reported: Option<SystemTime>, expected: SystemTime) -> bool {
    let resolution = source.capabilities().timestamp_resolution;
    reported.is_some_and(|time| time.duration_since(expected).or_else(|_| expected.duration_since(time)).is_ok_and(|gap| gap <= resolution))
}

fn listing(harness: &mut Harness<'_>) -> Result<(), Stop> {
    let spec = fixture().file("a.txt", "hello", at(T1)).folder("sub", at(T2)).file("sub/b.txt", "x", at(T2));
    let source = build(harness, &spec, "files and folders")?;
    let root = source.list(Path::new(""), Links::Report)?;
    ensure(names(&root) == ["a.txt", "sub"], || format!("the root lists {:?}", names(&root)))?;
    let file = find(&root, "a.txt")?;
    ensure(file.kind == ItemKind::File && file.size == 5 && file.error.is_none(), || format!("a.txt is reported as {file:?}"))?;
    ensure(within_resolution(&*source, file.modified, at(T1)), || format!("a.txt's time is {:?}", file.modified))?;
    let folder = find(&root, "sub")?;
    ensure(folder.kind == ItemKind::Folder, || format!("sub is reported as {folder:?}"))?;
    let inner = source.list(Path::new("sub"), Links::Report)?;
    ensure(names(&inner) == ["b.txt"], || format!("sub lists {:?}", names(&inner)))
}

fn stat(harness: &mut Harness<'_>) -> Result<(), Stop> {
    let spec = fixture().file("sub/b.txt", "four", at(T1));
    let source = build(harness, &spec, "files and folders")?;
    let listed = source.list(Path::new("sub"), Links::Report)?;
    let listed = find(&listed, "b.txt")?;
    let stated = source.stat(Path::new("sub/b.txt"), Links::Report)?;
    ensure((stated.kind, stated.size, stated.modified) == (listed.kind, listed.size, listed.modified), || {
        format!("stat reports {stated:?}, the listing {listed:?}")
    })?;
    expect_error(source.stat(Path::new("sub/missing.txt"), Links::Report), ErrorKind::NotFound, "stat of a missing file")?;
    expect_error(source.list(Path::new("missing"), Links::Report), ErrorKind::NotFound, "listing a missing folder")?;
    expect_error(source.stat(Path::new("../sub"), Links::Report), ErrorKind::NotFound, "stat of a path outside the source")
}

fn links(harness: &mut Harness<'_>) -> Result<(), Stop> {
    let spec = fixture().file("file.txt", "abc", at(T1)).file("dir/inside.txt", "12345", at(T2)).link("to-file", "file.txt", at(T2)).link(
        "to-dir",
        "dir",
        at(T1),
    );
    let source = build(harness, &spec, "links")?;
    let reported = source.list(Path::new(""), Links::Report)?;
    for (name, target) in [("to-file", "file.txt"), ("to-dir", "dir")] {
        let link = find(&reported, name)?;
        ensure(link.kind == ItemKind::Link && link.link_target.as_deref() == Some(target), || format!("{name} is reported as {link:?}"))?;
    }
    let followed = source.list(Path::new(""), Links::Follow)?;
    let file = find(&followed, "to-file")?;
    ensure(file.kind == ItemKind::File && file.size == 3 && within_resolution(&*source, file.modified, at(T1)), || {
        format!("followed, to-file is reported as {file:?}")
    })?;
    let folder = find(&followed, "to-dir")?;
    ensure(folder.kind == ItemKind::Folder, || format!("followed, to-dir is reported as {folder:?}"))?;
    let stated = source.stat(Path::new("to-dir"), Links::Report)?;
    ensure(stated.kind == ItemKind::Link, || format!("stat reports to-dir as {stated:?}"))?;
    let through = source.list(Path::new("to-dir"), Links::Report)?;
    ensure(names(&through) == ["inside.txt"], || format!("listing through to-dir gives {:?}", names(&through)))
}

fn link_loop(harness: &mut Harness<'_>) -> Result<(), Stop> {
    let spec = fixture().link("loop", "loop", at(T1)).file("ok.txt", "fine", at(T1));
    let source = build(harness, &spec, "links")?;
    let items = source.list(Path::new(""), Links::Follow)?;
    let looped = find(&items, "loop")?;
    ensure(looped.error.is_some(), || format!("the looping link is reported as {looped:?}"))?;
    let ok = find(&items, "ok.txt")?;
    ensure(ok.error.is_none() && ok.kind == ItemKind::File, || format!("ok.txt is reported as {ok:?}"))
}

fn unreadable(harness: &mut Harness<'_>) -> Result<(), Stop> {
    let spec = fixture().file("a.txt", "a", at(T1)).file("locked/secret.txt", "s", at(T1)).unreadable("locked").file("z.txt", "z", at(T1));
    let source = build(harness, &spec, "unreadable items")?;
    let items = source.list(Path::new(""), Links::Report)?;
    ensure(names(&items) == ["a.txt", "locked", "z.txt"], || format!("the listing holds {:?}", names(&items)))?;
    let locked = find(&items, "locked")?;
    ensure(locked.error.as_ref().is_some_and(|e| e.kind == ErrorKind::AccessDenied), || format!("locked is reported as {locked:?}"))?;
    for name in ["a.txt", "z.txt"] {
        let item = find(&items, name)?;
        ensure(item.error.is_none(), || format!("{name} carries {:?}", item.error))?;
    }
    expect_error(source.list(Path::new("locked"), Links::Report), ErrorKind::AccessDenied, "listing the unreadable folder")
}

fn container(harness: &mut Harness<'_>) -> Result<(), Stop> {
    let spec = fixture().container("pack.zip", "archive bytes", at(T1)).file("pack.zip/inner/a.txt", "inside", at(T2));
    let source = build(harness, &spec, "expandable containers")?;
    let pack = source.stat(Path::new("pack.zip"), Links::Report)?;
    ensure(pack.kind == ItemKind::File && pack.container, || format!("pack.zip is reported as {pack:?}"))?;
    let children = source.list(Path::new("pack.zip"), Links::Report)?;
    ensure(names(&children) == ["inner"] && find(&children, "inner")?.kind == ItemKind::Folder, || format!("pack.zip lists {children:?}"))?;
    let inner = source.list(Path::new("pack.zip/inner"), Links::Report)?;
    let file = find(&inner, "a.txt")?;
    ensure(file.kind == ItemKind::File && file.size == 6, || format!("a.txt in the container is reported as {file:?}"))?;
    let content = read_all(&*source, "pack.zip/inner/a.txt")?;
    ensure(content == b"inside", || format!("a.txt in the container reads {content:?}"))
}

fn crc(harness: &mut Harness<'_>) -> Result<(), Stop> {
    let spec = fixture().file("check.txt", "123456789", at(T1)).crc32("check.txt", CHECK_CRC);
    let source = build(harness, &spec, "files")?;
    let item = source.stat(Path::new("check.txt"), Links::Report)?;
    ensure(item.crc32.is_none_or(|crc| crc == CHECK_CRC), || format!("check.txt reports CRC {:08x?}", item.crc32))
}

fn case_sensitivity(harness: &mut Harness<'_>) -> Result<(), Stop> {
    let spec = fixture().file("Readme.txt", "r", at(T1));
    let source = build(harness, &spec, "files")?;
    let other_case = source.stat(Path::new("README.TXT"), Links::Report);
    if source.capabilities().case_sensitive {
        expect_error(other_case, ErrorKind::NotFound, "stat of README.TXT on a case-sensitive source")
    } else {
        let item = other_case?;
        ensure(item.kind == ItemKind::File && item.size == 1, || format!("README.TXT is reported as {item:?}"))
    }
}

fn read(harness: &mut Harness<'_>) -> Result<(), Stop> {
    let spec = fixture().file("a.txt", "hello", at(T1)).folder("dir", at(T1));
    let source = build(harness, &spec, "files and folders")?;
    let content = read_all(&*source, "a.txt")?;
    ensure(content == b"hello", || format!("a.txt reads {content:?}"))?;
    expect_error(source.open_read(Path::new("missing.txt")), ErrorKind::NotFound, "reading a missing file")?;
    ensure(source.open_read(Path::new("dir")).is_err(), || "reading a folder as a file succeeded".to_owned())
}

fn write(harness: &mut Harness<'_>) -> Result<(), Stop> {
    let spec = fixture().file("a.txt", "old", at(T1));
    let source = writable(harness, &spec)?;
    source.write(Path::new("new.txt"), &mut Cursor::new("fresh"))?;
    let fresh = read_all(&*source, "new.txt")?;
    ensure(fresh == b"fresh", || format!("new.txt reads {fresh:?}"))?;
    source.write(Path::new("a.txt"), &mut Cursor::new("replaced"))?;
    let replaced = read_all(&*source, "a.txt")?;
    ensure(replaced == b"replaced", || format!("a.txt reads {replaced:?} after its replacement"))?;
    let item = source.stat(Path::new("a.txt"), Links::Report)?;
    ensure(item.size == 8, || format!("after its replacement, a.txt is reported as {item:?}"))?;
    expect_error(source.write(Path::new("missing/x.txt"), &mut Cursor::new("x")), ErrorKind::NotFound, "writing into a missing folder")
}

fn create_folder(harness: &mut Harness<'_>) -> Result<(), Stop> {
    let source = writable(harness, &fixture().file("a.txt", "a", at(T1)))?;
    source.create_folder(Path::new("made"))?;
    let made = source.stat(Path::new("made"), Links::Report)?;
    ensure(made.kind == ItemKind::Folder, || format!("made is reported as {made:?}"))?;
    let children = source.list(Path::new("made"), Links::Report)?;
    ensure(children.is_empty(), || format!("the new folder lists {children:?}"))?;
    source.create_folder(Path::new("made")).map_err(|e| Stop::Fail(format!("creating an existing folder again: {e:?}")))?;
    expect_error(source.create_folder(Path::new("missing/inner")), ErrorKind::NotFound, "creating a folder in a missing folder")
}

fn rename(harness: &mut Harness<'_>) -> Result<(), Stop> {
    let spec = fixture().file("a.txt", "hello", at(T1)).file("dir/f.txt", "f", at(T1));
    let source = writable(harness, &spec)?;
    source.rename(Path::new("a.txt"), "b.txt")?;
    expect_error(source.stat(Path::new("a.txt"), Links::Report), ErrorKind::NotFound, "stat of the old name")?;
    let content = read_all(&*source, "b.txt")?;
    ensure(content == b"hello", || format!("b.txt reads {content:?}"))?;
    source.rename(Path::new("dir"), "dir2")?;
    let moved = source.list(Path::new("dir2"), Links::Report)?;
    ensure(names(&moved) == ["f.txt"], || format!("the renamed folder lists {:?}", names(&moved)))?;
    expect_error(source.rename(Path::new("missing.txt"), "x.txt"), ErrorKind::NotFound, "renaming a missing file")
}

fn delete(harness: &mut Harness<'_>) -> Result<(), Stop> {
    let spec = fixture().file("a.txt", "a", at(T1)).file("dir/sub/g.txt", "g", at(T1)).file("dir/f.txt", "f", at(T1)).file("keep.txt", "k", at(T1));
    let source = writable(harness, &spec)?;
    source.delete(Path::new("a.txt"), Delete::Permanently)?;
    source.delete(Path::new("dir"), Delete::Permanently)?;
    let left = source.list(Path::new(""), Links::Report)?;
    ensure(names(&left) == ["keep.txt"], || format!("after the deletes the root lists {:?}", names(&left)))?;
    expect_error(source.delete(Path::new("a.txt"), Delete::Permanently), ErrorKind::NotFound, "deleting a missing file")
}

fn recycle_bin(harness: &mut Harness<'_>) -> Result<(), Stop> {
    let source = writable(harness, &fixture().file("a.txt", "a", at(T1)))?;
    let result = source.delete(Path::new("a.txt"), Delete::ToRecycleBin);
    let still_there = source.stat(Path::new("a.txt"), Links::Report).is_ok();
    if source.capabilities().recycle_bin {
        result?;
        ensure(!still_there, || "a.txt is still there after its delete to the recycle bin".to_owned())
    } else {
        expect_error(result, ErrorKind::Unsupported, "a recycle-bin delete without a recycle bin")?;
        ensure(still_there, || "a refused recycle-bin delete removed a.txt".to_owned())
    }
}

fn set_modified(harness: &mut Harness<'_>) -> Result<(), Stop> {
    let source = writable(harness, &fixture().file("a.txt", "a", at(T1)))?;
    let time = at(T2 + 1) + Duration::from_nanos(456_789_000);
    source.set_modified(Path::new("a.txt"), time)?;
    let item = source.stat(Path::new("a.txt"), Links::Report)?;
    ensure(within_resolution(&*source, item.modified, time), || format!("a.txt's time is {:?} after setting {time:?}", item.modified))
}

fn set_permissions(harness: &mut Harness<'_>) -> Result<(), Stop> {
    const HIDDEN: u32 = 0x2;
    let source = writable(harness, &fixture().file("a.txt", "a", at(T1)))?;
    let caps = source.capabilities();
    let windows = source.set_permissions(Path::new("a.txt"), Permissions::Windows(HIDDEN));
    if caps.windows_attributes {
        windows?;
        let attributes = source.stat(Path::new("a.txt"), Links::Report)?.attributes;
        ensure(matches!(attributes, Some(Attributes::Windows(codes)) if codes & HIDDEN == HIDDEN), || {
            format!("after setting hidden, a.txt reports {attributes:?}")
        })?;
    } else {
        expect_error(windows, ErrorKind::Unsupported, "setting Windows attributes without support")?;
    }
    let unix = source.set_permissions(Path::new("a.txt"), Permissions::UnixMode(0o640));
    if caps.unix_permissions {
        unix?;
        let attributes = source.stat(Path::new("a.txt"), Links::Report)?.attributes;
        ensure(matches!(attributes, Some(Attributes::Unix { mode, .. }) if mode & 0o777 == 0o640), || {
            format!("after setting mode 640, a.txt reports {attributes:?}")
        })
    } else {
        expect_error(unix, ErrorKind::Unsupported, "setting Unix permissions without support")
    }
}

/// Everything a write could change in the read-only fixture: both listings and a.txt's content.
type Snapshot = (Vec<Item>, Vec<Item>, Vec<u8>);

fn snapshot(source: &dyn DataSource) -> Result<Snapshot, Stop> {
    Ok((source.list(Path::new(""), Links::Report)?, source.list(Path::new("dir"), Links::Report)?, read_all(source, "a.txt")?))
}

fn read_only(harness: &mut Harness<'_>) -> Result<(), Stop> {
    let spec = fixture().file("a.txt", "a", at(T1)).file("dir/f.txt", "f", at(T1));
    let caps = spec.capabilities();
    let source = build(harness, &spec.with_capabilities(Capabilities { read_only: true, ..caps }), "read-only media")?;
    ensure(source.capabilities().read_only, || "the source for a read-only fixture doesn't report read-only".to_owned())?;
    let before = snapshot(&*source)?;
    let (a, dir) = (Path::new("a.txt"), Path::new("dir"));
    let attempts: [(&str, Result<(), SourceError>); 10] = [
        ("writing a new file", source.write(Path::new("new.txt"), &mut Cursor::new("n"))),
        ("replacing a file", source.write(a, &mut Cursor::new("changed"))),
        ("creating a folder", source.create_folder(Path::new("made"))),
        ("deleting a file", source.delete(a, Delete::Permanently)),
        ("deleting a folder to the recycle bin", source.delete(dir, Delete::ToRecycleBin)),
        ("renaming", source.rename(a, "b.txt")),
        ("setting the modified time", source.set_modified(a, at(T2))),
        ("setting Windows attributes", source.set_permissions(a, Permissions::Windows(0x2))),
        ("setting Unix permissions", source.set_permissions(a, Permissions::UnixMode(0o600))),
        ("copying into it", copy(&*source, a, &*source, Path::new("copy.txt"))),
    ];
    for (what, result) in attempts {
        match result {
            Err(e) if e.kind == ErrorKind::AccessDenied && !e.message.is_empty() => {}
            other => return Err(Stop::Fail(format!("{what} on a read-only source gave {other:?}, not access denied with a message"))),
        }
    }
    ensure(snapshot(&*source)? == before, || "a refused write changed the read-only source".to_owned())
}

fn copies(harness: &mut Harness<'_>) -> Result<(), Stop> {
    let spec = fixture().file("a.txt", "hello", at(T1)).file("dir/f.txt", "inner", at(T2));
    let source = writable(harness, &spec)?;
    copy(&*source, Path::new("a.txt"), &*source, Path::new("a-copy.txt"))?;
    let copied = source.stat(Path::new("a-copy.txt"), Links::Report)?;
    ensure(read_all(&*source, "a-copy.txt")? == b"hello", || "a-copy.txt doesn't read like a.txt".to_owned())?;
    ensure(within_resolution(&*source, copied.modified, at(T1)), || format!("the copy's time is {:?}", copied.modified))?;
    let other = writable(harness, &fixture())?;
    copy(&*source, Path::new("dir"), &*other, Path::new("dir"))?;
    let inner = other.stat(Path::new("dir/f.txt"), Links::Report)?;
    ensure(read_all(&*other, "dir/f.txt")? == b"inner", || "dir/f.txt doesn't read like the original".to_owned())?;
    ensure(within_resolution(&*other, inner.modified, at(T2)), || format!("the copied dir/f.txt's time is {:?}", inner.modified))
}
