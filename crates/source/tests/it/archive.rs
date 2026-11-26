//! The zip provider, on archives the tests build: the contract suite's cases, listing without
//! reading entry data, checked reads, odd entry names, and damaged archives.

use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::{Datelike, Local, Timelike};
use craft_testkit::hostile::{HammerConfig, hammer};
use otd_source::suite::{self, Outcome};
use otd_source::{DataSource, ItemKind, Links, MemorySource, ZipSource};
use tempfile::TempDir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

/// A DOS time, local time as zip tools write it.
fn dos_time(time: SystemTime) -> Option<zip::DateTime> {
    let local = chrono::DateTime::<Local>::from(time);
    let narrow = |n: u32| u8::try_from(n).ok();
    zip::DateTime::from_date_and_time(
        u16::try_from(local.year()).ok()?,
        narrow(local.month())?,
        narrow(local.day())?,
        narrow(local.hour())?,
        narrow(local.minute())?,
        narrow(local.second())?,
    )
    .ok()
}

/// Writes a fixture's tree as entries. `None` for what a zip can't hold here: links, unreadable
/// items and archives inside the archive. The fixture's attributes are left out: the provider
/// reports each entry's own.
fn add_tree(fixture: &MemorySource, folder: &Path, zip: &mut ZipWriter<Cursor<Vec<u8>>>) -> Option<()> {
    for item in fixture.list(folder, Links::Report).ok()? {
        if item.error.is_some() || item.container {
            return None;
        }
        let path = folder.join(&item.name);
        let name = path.to_str()?.replace('\\', "/");
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated).last_modified_time(dos_time(item.modified?)?);
        match item.kind {
            ItemKind::File => {
                zip.start_file(name, options).ok()?;
                io::copy(&mut fixture.open_read(&path).ok()?, zip).ok()?;
            }
            ItemKind::Folder => {
                zip.add_directory(name, options).ok()?;
                add_tree(fixture, &path, zip)?;
            }
            ItemKind::Link => return None,
        }
    }
    Some(())
}

/// Writes `bytes` as an archive in a fresh temporary folder.
fn save(bytes: &[u8]) -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("archive.zip");
    fs::write(&path, bytes).unwrap();
    (dir, path)
}

fn build(fixture: &MemorySource, dirs: &mut Vec<TempDir>) -> Option<Box<dyn DataSource>> {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    add_tree(fixture, Path::new(""), &mut zip)?;
    let (dir, path) = save(&zip.finish().ok()?.into_inner());
    dirs.push(dir);
    Some(Box::new(ZipSource::open(path).ok()?))
}

#[test]
fn the_zip_provider_passes_every_case_a_read_only_archive_can_hold() {
    let mut dirs = Vec::new();
    let results = suite::run(&mut |fixture: &MemorySource| build(fixture, &mut dirs));
    let failed: Vec<_> = results.iter().filter(|r| matches!(r.outcome, Outcome::Failed(_))).collect();
    assert!(failed.is_empty(), "{failed:#?}");
    // Exactly these pass, so a fixture the harness stopped building can't skip a case unnoticed.
    // The rest skip: links, unreadable items and containers a zip doesn't hold, and every write
    // until E04-S2-T2, as the provider is read-only.
    let passed: BTreeSet<&str> = results.iter().filter(|r| r.outcome == Outcome::Passed).map(|r| r.name).collect();
    let expected = BTreeSet::from([
        "a_listing_reports_each_child_with_its_name_kind_size_and_time",
        "every_item_reports_its_attributes_and_stat_agrees",
        "stat_agrees_with_the_listing_and_a_missing_path_is_not_found",
        "a_source_provided_crc_matches_the_content",
        "names_match_regardless_of_case_only_on_a_case_insensitive_source",
        "content_reads_back_and_a_missing_file_is_not_found",
        "a_read_only_source_refuses_every_write_and_changes_nothing",
    ]);
    assert_eq!(passed, expected, "{results:#?}");
}

#[test]
fn listing_a_10000_entry_archive_reads_no_entry_data() {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for i in 0..10_000 {
        zip.start_file(format!("d{}/f{}.txt", i / 100, i % 100), options).unwrap();
        write!(zip, "entry number {i}").unwrap();
    }
    let mut bytes = zip.finish().unwrap().into_inner();
    let mut stored = zip::ZipArchive::new(Cursor::new(&bytes)).unwrap();
    let crcs: Vec<(String, u64, u32)> = (0..stored.len())
        .map(|i| {
            let entry = stored.by_index_raw(i).unwrap();
            (entry.name().unwrap().into_owned(), entry.size(), entry.crc32())
        })
        .collect();
    // Every entry's header and data go, so only the directory is left to list from.
    let directory = usize::try_from(stored.central_directory_start()).unwrap();
    bytes[..directory].fill(0);
    let (_dir, path) = save(&bytes);

    let source = ZipSource::open(path).unwrap();
    assert_eq!(source.list(Path::new(""), Links::Report).unwrap().len(), 100);
    for (name, size, crc) in &crcs {
        let (folder, file) = name.split_once('/').unwrap();
        let listed = source.list(Path::new(folder), Links::Report).unwrap();
        let item = listed.iter().find(|item| item.name == file).unwrap();
        assert!(item.kind == ItemKind::File && item.size == *size && item.crc32 == Some(*crc) && item.error.is_none(), "{name}: {item:?}");
    }
    // The data really is gone: reading an entry fails.
    assert!(source.open_read(Path::new("d0/f0.txt")).unwrap().read_to_end(&mut Vec::new()).is_err());
}

/// An archive of one entry, `big.bin`, stored uncompressed so its bytes can be found and damaged.
fn big_entry() -> (Vec<u8>, Vec<u8>) {
    let content: Vec<u8> = (0..300_000u32).map(|i| (i % 251) as u8).collect();
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    zip.start_file("big.bin", SimpleFileOptions::default().compression_method(CompressionMethod::Stored)).unwrap();
    zip.write_all(&content).unwrap();
    (zip.finish().unwrap().into_inner(), content)
}

#[test]
fn an_entry_reads_back_whole_across_many_hand_overs_and_a_damaged_one_fails() {
    let (mut bytes, content) = big_entry();
    let (_dir, path) = save(&bytes);
    let mut read = Vec::new();
    ZipSource::open(path).unwrap().open_read(Path::new("big.bin")).unwrap().read_to_end(&mut read).unwrap();
    assert!(read == content, "read {} bytes of {}", read.len(), content.len());

    // One byte near the end changes, so the CRC check fails after most of the entry has arrived.
    let at = bytes.windows(content.len()).position(|w| w == content.as_slice()).unwrap() + content.len() - 10;
    bytes[at] ^= 0xFF;
    let (_dir, path) = save(&bytes);
    let result = ZipSource::open(path).unwrap().open_read(Path::new("big.bin")).unwrap().read_to_end(&mut Vec::new());
    assert!(result.is_err(), "a damaged entry read as {result:?}");
}

#[test]
fn odd_entry_names_are_listed_where_they_lead_or_with_an_error_never_dropped() {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default();
    for name in ["../up.txt", "/rooted.txt", "back\\slash.txt", "both", "both/inner.txt"] {
        zip.start_file(name, options).unwrap();
        zip.write_all(name.as_bytes()).unwrap();
    }
    let (_dir, path) = save(&zip.finish().unwrap().into_inner());
    let source = ZipSource::open(path).unwrap();
    let root = source.list(Path::new(""), Links::Report).unwrap();
    let find = |name: &str| root.iter().find(|item| item.name == name).unwrap_or_else(|| panic!("{name} is missing from {root:#?}"));

    let up = find("../up.txt");
    assert!(up.error.is_some(), "{up:?}");
    assert!(find("rooted.txt").error.is_none());
    let mut read = String::new();
    source.open_read(Path::new("back/slash.txt")).unwrap().read_to_string(&mut read).unwrap();
    assert_eq!(read, "back\\slash.txt");
    // `both` is a file and the folder of another entry: it lists as the folder, with an error.
    let both = find("both");
    assert!(both.kind == ItemKind::Folder && both.error.is_some(), "{both:?}");
    assert_eq!(source.list(Path::new("both"), Links::Report).unwrap()[0].name, "inner.txt");
}

/// Lists every folder and reads every file, ignoring errors.
fn walk(source: &ZipSource, folder: &Path) {
    let Ok(items) = source.list(folder, Links::Report) else { return };
    for item in items {
        let path = folder.join(&item.name);
        match item.kind {
            ItemKind::Folder => walk(source, &path),
            _ => {
                let _ = source.open_read(&path).map(|mut r| r.read_to_end(&mut Vec::new()));
            }
        }
    }
}

#[test]
fn opening_listing_and_reading_a_damaged_archive_never_panics_or_hangs() {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    zip.add_directory("docs", SimpleFileOptions::default()).unwrap();
    zip.start_file("docs/a.txt", SimpleFileOptions::default().compression_method(CompressionMethod::Deflated)).unwrap();
    zip.write_all(b"deflated text, deflated text, deflated text").unwrap();
    zip.start_file("b.bin", SimpleFileOptions::default().compression_method(CompressionMethod::Stored)).unwrap();
    zip.write_all(&[1, 2, 3, 4]).unwrap();
    let seed = zip.finish().unwrap().into_inner();
    hammer(&HammerConfig::new("zip archive", 2_000), &seed, |data| {
        if let Ok(source) = ZipSource::from_reader("hostile.zip".to_owned(), Cursor::new(data.to_vec()), SystemTime::UNIX_EPOCH) {
            walk(&source, Path::new(""));
        }
    });
}
