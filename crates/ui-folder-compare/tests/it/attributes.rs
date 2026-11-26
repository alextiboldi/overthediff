//! The Attributes, Owner and Group columns, read from a real file in a temporary folder.

use std::fs;
use std::path::Path;

use otd_source::{Attributes, DataSource as _, Links, LocalSource};
use otd_ui_folder_compare::{FolderCompare, ON_LEFT};

use crate::view::{harness, on, rows};

/// The row of `a.txt`, in a pair of folders where only the left one has it.
fn row_of_a_file(set_up: impl FnOnce(&Path)) -> (Vec<String>, Option<Attributes>) {
    let root = tempfile::tempdir().unwrap();
    for side in ["left", "right"] {
        fs::create_dir(root.path().join(side)).unwrap();
    }
    let file = root.path().join("left").join("a.txt");
    fs::write(&file, "a").unwrap();
    set_up(&file);
    let reported = LocalSource::new(root.path().join("left")).stat(Path::new("a.txt"), Links::Report).unwrap().attributes;
    let harness = harness(FolderCompare::of_folders(&["left".into(), "right".into()], root.path()));
    let name = on(ON_LEFT, "a.txt");
    let row = rows(&harness).into_iter().find(|row| row.contains(&name)).expect("a.txt has a row");
    (row, reported)
}

#[cfg(unix)]
#[test]
fn a_unix_file_shows_its_mode_as_ls_does_and_its_owner_and_group_names() {
    use std::os::unix::fs::PermissionsExt as _;
    let (row, reported) = row_of_a_file(|file| fs::set_permissions(file, fs::Permissions::from_mode(0o644)).unwrap());
    let Some(Attributes::Unix { owner, group, .. }) = reported else { panic!("the local source reports {reported:?}") };
    for cell in ["-rw-r--r--", &owner, &group] {
        assert!(row.iter().any(|c| c == cell), "{cell:?} is in {row:?}");
    }
}

#[cfg(windows)]
#[test]
fn a_read_only_hidden_windows_file_shows_the_r_and_h_codes() {
    use otd_source::Permissions;
    let (row, _) = row_of_a_file(|file| {
        let source = LocalSource::new(file.parent().unwrap());
        // FILE_ATTRIBUTE_READONLY | FILE_ATTRIBUTE_HIDDEN, with ARCHIVE as a new file has it.
        source.set_permissions(Path::new("a.txt"), Permissions::Windows(0x1 | 0x2 | 0x20)).unwrap();
    });
    assert!(row.iter().any(|c| c.contains('R') && c.contains('H') && c.chars().all(|ch| ch.is_ascii_uppercase())), "R and H in {row:?}");
}
