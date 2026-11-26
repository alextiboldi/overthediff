//! Saving a pane back through the data-source layer (E07-S1-T6).

use std::io::Read;
use std::path::Path;
use std::time::{Duration, SystemTime};

use otd_format::Registry;
use otd_source::{Capabilities, DataSource, MemorySource};
use otd_text_compare::{Pane, Pos, ReadOnly, SaveError, Unencodable, load, open, save, save_as};

const PATH: &str = "file.txt";

fn source(content: &[u8]) -> MemorySource {
    let capabilities = Capabilities {
        read_only: false,
        case_sensitive: true,
        recycle_bin: false,
        timestamp_resolution: Duration::ZERO,
        windows_attributes: false,
        unix_permissions: false,
    };
    MemorySource::new(capabilities).file(PATH, content.to_vec(), SystemTime::UNIX_EPOCH)
}

fn read(source: &MemorySource) -> Vec<u8> {
    let mut bytes = Vec::new();
    source.open_read(Path::new(PATH)).unwrap().read_to_end(&mut bytes).unwrap();
    bytes
}

/// Types `text` at the start of line `line`.
fn type_at(pane: &mut Pane, line: usize, text: &str) {
    pane.move_caret(Pos { line, column: 0 }, false);
    pane.type_text(text);
}

fn utf16(bom: bool, text: &str, unit: fn(u16) -> [u8; 2]) -> Vec<u8> {
    bom.then_some(0xFEFF).into_iter().chain(text.encode_utf16()).flat_map(unit).collect()
}

#[test]
fn saving_an_edited_line_changes_only_its_bytes_in_each_encoding_and_line_ending_style() {
    let bom = |text: &str| [&[0xEF, 0xBB, 0xBF], text.as_bytes()].concat();
    // Each file, and what it must be after typing X at the start of its second line.
    let cases: [(&str, Vec<u8>, Vec<u8>, u16); 6] = [
        ("UTF-8 with BOM, Windows", bom("a\r\nb\r\nc\r\n"), bom("a\r\nXb\r\nc\r\n"), 1252),
        ("UTF-8, mixed, no final break", b"a\nb\r\nc".to_vec(), b"a\nXb\r\nc".to_vec(), 1252),
        ("UTF-16 LE with BOM, Windows", utf16(true, "a\r\nb\r\n", u16::to_le_bytes), utf16(true, "a\r\nXb\r\n", u16::to_le_bytes), 1252),
        ("UTF-16 BE, Unix", utf16(false, "aa\nbb\ncc\n", u16::to_be_bytes), utf16(false, "aa\nXbb\ncc\n", u16::to_be_bytes), 1252),
        ("Windows-1252, Mac", b"caf\xE9\rb\rc\r".to_vec(), b"caf\xE9\rXb\rc\r".to_vec(), 1252),
        // 日 and 本 in Shift_JIS.
        ("Shift_JIS, Unix", b"\x93\xFA\n\x96\x7B\n".to_vec(), b"\x93\xFA\nX\x96\x7B\n".to_vec(), 932),
    ];
    for (name, before, after, code_page) in cases {
        let source = source(&before);
        let mut pane = load(&source, Path::new(PATH), PATH, &Registry::default(), code_page).unwrap();
        assert_eq!(pane.read_only, None, "{name}: it reads without loss, so it can be saved");
        type_at(&mut pane, 1, "X");
        assert!(pane.unsaved, "{name}: the edit marks it");
        save(&mut pane, &source, Path::new(PATH)).unwrap();
        assert_eq!(read(&source), after, "{name}: only the edited line's bytes change");
        assert!(!pane.unsaved, "{name}: the save clears the mark");
        assert_eq!(pane.size, after.len() as u64, "{name}: the header shows the new size");
    }
}

#[test]
fn a_character_its_encoding_cant_hold_is_refused_and_nothing_is_written() {
    for (before, code_page) in [(b"caf\xE9\n".as_slice(), 1252), (b"\x93\xFA\n", 932)] {
        let source = source(before);
        let mut pane = load(&source, Path::new(PATH), PATH, &Registry::default(), code_page).unwrap();
        type_at(&mut pane, 0, "\u{1F600}");
        let refused = save(&mut pane, &source, Path::new(PATH));
        assert_eq!(refused, Err(SaveError::Unencodable(Unencodable { line: 0, character: '\u{1F600}' })), "Windows-{code_page}");
        assert_eq!(read(&source), before, "Windows-{code_page}: the file is as it was");
        assert!(pane.unsaved, "Windows-{code_page}: the edit is still unsaved");
    }
}

#[test]
fn a_file_with_bytes_its_encoding_cant_read_loads_read_only_and_wont_save() {
    // A byte Windows-874 doesn't assign, invalid UTF-8 after a BOM, and an odd UTF-16 tail: each
    // reads as U+FFFD, which would save as other bytes.
    let marked: Vec<u8> = [0xFF, 0xFE, b'a', 0, b'b'].to_vec();
    for (content, code_page) in [(b"a\x81b\n".to_vec(), 874), ([&[0xEF, 0xBB, 0xBF], b"a\xFFb".as_slice()].concat(), 1252), (marked, 1252)] {
        let source = source(&content);
        let mut pane = load(&source, Path::new(PATH), PATH, &Registry::default(), code_page).unwrap();
        assert_eq!(pane.read_only, Some(ReadOnly::Encoding), "{content:?}");
        assert_eq!(save(&mut pane, &source, Path::new(PATH)), Err(SaveError::Lossy));
        assert_eq!(read(&source), content, "nothing was written");
    }
}

#[test]
fn a_file_marked_read_only_opens_read_only_and_saving_it_is_refused() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("locked.txt");
    std::fs::write(&path, "one\n").unwrap();
    let set_read_only = |read_only| {
        let mut permissions = std::fs::metadata(&path).unwrap().permissions();
        permissions.set_readonly(read_only);
        std::fs::set_permissions(&path, permissions).unwrap();
    };
    set_read_only(true);
    let spec = path.display().to_string();
    let mut pane = open(&spec, &Registry::default()).unwrap();
    assert_eq!(pane.read_only, Some(ReadOnly::File));
    // Replacing the file through a temporary one would work on Unix, so the save checks first.
    pane.lines[0].text = "edited".to_owned();
    assert_eq!(save_as(&mut pane, &spec), Err(SaveError::MarkedReadOnly));
    assert_eq!(std::fs::read(&path).unwrap(), b"one\n");

    // Save As somewhere else writes it, and the pane, now that file, can be edited.
    let copy = folder.path().join("copy.txt").display().to_string();
    save_as(&mut pane, &copy).unwrap();
    assert_eq!(std::fs::read(&copy).unwrap(), b"edited\n");
    assert_eq!((pane.path.as_str(), pane.read_only), (copy.as_str(), None));
    set_read_only(false);
}
