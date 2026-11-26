//! Loading a pane through the data-source layer.

use std::path::Path;
use std::time::{Duration, SystemTime};

use otd_format::{CATCH_ALL, Format, Kind, Registry};
use otd_source::{Capabilities, ErrorKind, MemorySource};
use otd_text_compare::{Encoding, LineEndings, load};

const CONTENT: &[u8] = b"caf\xE9\r\nx\r\n";

fn source(case_sensitive: bool) -> (MemorySource, SystemTime) {
    let modified = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);
    let capabilities = Capabilities {
        read_only: false,
        case_sensitive,
        recycle_bin: false,
        timestamp_resolution: Duration::ZERO,
        windows_attributes: false,
        unix_permissions: false,
    };
    let source = MemorySource::new(capabilities).folder("docs", modified).file("docs/Notes.TXT", CONTENT.to_vec(), modified);
    (source, modified)
}

fn formats() -> Registry {
    let mut formats = Registry::default();
    formats.register_kind(Kind::Text);
    formats.register(Format::new("Notes", Kind::Text, "*.txt").unwrap()).unwrap();
    formats
}

#[test]
fn a_loaded_pane_carries_what_its_header_shows() {
    let (source, modified) = source(true);
    let pane = load(&source, Path::new("docs/Notes.TXT"), "/shown/Notes.TXT", &Registry::default(), 1252).unwrap();
    assert_eq!(pane.path, "/shown/Notes.TXT");
    assert_eq!(pane.size, CONTENT.len() as u64);
    assert_eq!(pane.modified, Some(modified));
    assert_eq!(pane.format, CATCH_ALL, "no format matches, so the content decides");
    assert_eq!((pane.encoding, pane.endings), (Encoding::CodePage(1252), LineEndings::Windows));
    assert_eq!(pane.lines.first().map(|l| l.text.as_str()), Some("café"));
}

#[test]
fn the_format_mask_matches_with_the_source_s_case() {
    let path = Path::new("docs/Notes.TXT");
    let (sensitive, _) = source(true);
    assert_eq!(load(&sensitive, path, "", &formats(), 1252).unwrap().format, CATCH_ALL, "*.txt doesn't match .TXT");
    let (insensitive, _) = source(false);
    assert_eq!(load(&insensitive, path, "", &formats(), 1252).unwrap().format, "Notes");
}

#[test]
fn a_folder_or_a_missing_file_is_an_error() {
    let (source, _) = source(true);
    assert_eq!(load(&source, Path::new("docs"), "docs", &formats(), 1252).map_err(|e| e.kind), Err(ErrorKind::Unsupported));
    assert_eq!(load(&source, Path::new("gone.txt"), "gone.txt", &formats(), 1252).map_err(|e| e.kind), Err(ErrorKind::NotFound));
}
