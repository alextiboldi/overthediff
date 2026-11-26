//! The Ext, CRC and folder Size values (E08-S6-T3) in the real shell, read back through the
//! accessibility tree. The fixture trees are made in a temporary folder.

use std::fs;
use std::path::Path;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_ui::Shell;
use otd_ui_folder_compare::{COLUMNS, CRC, EXT, FolderCompare, ON_LEFT, SIZE};

use crate::view::{harness, on, settle};

/// On both sides: `d/a.txt`, `d/sub/c.txt` and `f.txt`, whose content is the CRC-32 check input.
fn fixture(root: &Path) -> Harness<'static, Shell> {
    for side in ["left", "right"] {
        let base = root.join(side);
        fs::create_dir_all(base.join("d/sub")).unwrap();
        fs::write(base.join("d/a.txt"), "abc").unwrap();
        fs::write(base.join("d/sub/c.txt"), "ij").unwrap();
        fs::write(base.join("f.txt"), "123456789").unwrap();
    }
    harness(FolderCompare::of_folders(&["left".into(), "right".into()], root))
}

fn run(harness: &mut Harness<'_, Shell>, id: &str) {
    assert!(harness.state_mut().run(id), "{id} ran");
    settle(harness);
}

/// The text of the left `caption` cell in the row of the left item `name`, blank cells included;
/// `None` when no row shows it.
fn left_cell(harness: &Harness<'_, Shell>, name: &str, caption: &str) -> Option<String> {
    let column = COLUMNS.iter().position(|column| column.caption == caption)?;
    harness.query_all_by_role(Role::Row).find_map(|row| {
        let cells: Vec<String> = row.query_all_by_role(Role::Cell).map(|cell| cell.accesskit_node().label().unwrap_or_default()).collect();
        let at = cells.iter().position(|cell| *cell == on(ON_LEFT, name))?;
        cells.get(at + 1 + column).cloned()
    })
}

fn number(text: &str) -> Option<u64> {
    text.chars().filter(char::is_ascii_digit).collect::<String>().parse().ok()
}

#[test]
fn a_folder_s_size_is_blank_until_everything_below_it_is_listed_then_the_sum_of_its_files() {
    let root = tempfile::tempdir().unwrap();
    let mut harness = fixture(root.path());
    assert_eq!(left_cell(&harness, "d", SIZE).as_deref(), Some(""), "d is not listed yet");

    // The first row, d, is current until focus moves.
    run(&mut harness, "folder-compare.open_folder");
    assert!(left_cell(&harness, "a.txt", SIZE).is_some(), "d is open");
    assert_eq!(left_cell(&harness, "d", SIZE).as_deref(), Some(""), "d/sub is not listed yet");

    run(&mut harness, "folder-compare.expand_all");
    let files = ["a.txt", "c.txt"].map(|name| left_cell(&harness, name, SIZE).as_deref().and_then(number));
    let size = left_cell(&harness, "d", SIZE).as_deref().and_then(number);
    assert_eq!(size, files.into_iter().sum(), "d's size is the sum of {files:?}");
}

#[test]
fn a_file_shows_its_extension_and_the_crc_32_of_its_content() {
    let root = tempfile::tempdir().unwrap();
    let harness = fixture(root.path());
    assert_eq!(left_cell(&harness, "f.txt", EXT).as_deref(), Some("txt"));
    // The CRC-32 check value: the CRC of `123456789` (ISO-HDLC, the zip and PNG CRC).
    assert_eq!(left_cell(&harness, "f.txt", CRC).as_deref(), Some("CBF43926"));
    assert_eq!(left_cell(&harness, "d", CRC).as_deref(), Some(""), "a folder has no CRC");
}
