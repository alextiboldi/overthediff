//! Expandable containers: a file until opened, then scanned, listed and aligned like a folder.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use otd_folder_engine::{Alignment, Criteria, Folder, ScanState, TreeScan, align, quick_test};
use otd_source::{Capabilities, Item, ItemKind, MemorySource};

fn path(names: &[&str]) -> PathBuf {
    names.iter().collect()
}

fn source() -> MemorySource {
    MemorySource::new(Capabilities {
        read_only: false,
        case_sensitive: true,
        recycle_bin: false,
        timestamp_resolution: Duration::ZERO,
        windows_attributes: false,
        unix_permissions: false,
    })
}

fn at(seconds: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)
}

/// The same children under the folder `dir` and the container `pack.zip` (a container nested in
/// each), and a container at the top that nobody opens.
fn tree() -> Arc<MemorySource> {
    let mut spec = source()
        .container("pack.zip", "archive bytes", at(7))
        .container("closed.zip", "zip", at(1))
        .file("closed.zip/hidden.txt", "h", at(1))
        .folder("dir", at(6));
    for top in ["dir", "pack.zip"] {
        spec = spec
            .file(&format!("{top}/a.txt"), "a", at(2))
            .file(&format!("{top}/sub/deep/b.txt"), "b", at(3))
            .container(&format!("{top}/inner.zip"), "inner", at(4))
            .file(&format!("{top}/inner.zip/c.txt"), "c", at(5));
    }
    Arc::new(spec)
}

fn item<'a>(folder: &'a Folder, name: &str) -> &'a Item {
    folder.items.iter().find(|item| item.name == name).unwrap_or_else(|| panic!("{name} is listed in {folder:?}"))
}

/// Names, kinds and container flags of a listing, sorted, and the scan states of its children.
fn shape(scan: &TreeScan, folder: &Path) -> Vec<(String, ItemKind, bool, Option<ScanState>)> {
    let listing = scan.folder(folder).expect("the folder was listed");
    let mut shape: Vec<_> =
        listing.items.iter().map(|i| (i.name.clone(), i.kind, i.container, scan.folder(&folder.join(&i.name)).map(|f| f.state))).collect();
    shape.sort_by(|a, b| a.0.cmp(&b.0));
    shape
}

#[test]
fn an_opened_container_scans_and_lists_like_a_folder_with_the_same_children() {
    for background in [true, false] {
        let (scan, events) = TreeScan::new(tree(), "", background);
        scan.open(Path::new("pack.zip"));
        scan.open(Path::new("dir"));
        while scan.step() {}
        assert_eq!(shape(&scan, Path::new("pack.zip")), shape(&scan, Path::new("dir")), "background {background}");
        if background {
            let deep = scan.folder(&path(&["pack.zip", "sub", "deep"])).expect("background scanning went on below the container");
            assert_eq!(deep.state, ScanState::Done);
            assert_eq!(events.try_iter().last().map(|e| e.pending), Some(0));
        }
        let inner = scan.folder(Path::new("pack.zip")).expect("the container was listed");
        assert_eq!(item(&inner, "inner.zip").kind, ItemKind::File, "a container inside it stays a file until it is opened");
    }
}

#[test]
fn once_opened_the_parent_lists_a_container_as_a_folder_whichever_was_listed_first() {
    // Opened before the base is listed, and opened after it.
    for open_first in [true, false] {
        let (scan, _events) = TreeScan::new(tree(), "", false);
        if open_first {
            scan.open(Path::new("pack.zip"));
        }
        while scan.step() {}
        scan.open(Path::new("pack.zip"));
        while scan.step() {}
        let base = scan.folder(Path::new("")).expect("the base was listed");
        let pack = item(&base, "pack.zip");
        assert!(pack.kind == ItemKind::Folder && pack.container, "open first {open_first}: {pack:?}");
        assert_eq!(item(&base, "closed.zip").kind, ItemKind::File, "only the opened container changes");
    }
}

#[test]
fn an_expanded_container_aligns_with_a_folder_and_its_children_pair_up_like_a_folders() {
    let left = Arc::new(source().container("pack", "zip", at(1)).file("pack/a.txt", "a", at(1)).file("pack/b.txt", "b", at(1)));
    let right = Arc::new(source().file("pack/b.txt", "b", at(1)).file("pack/c.txt", "c", at(1)));
    let (left, _) = TreeScan::new(left, "", false);
    let (right, _) = TreeScan::new(right, "", false);
    let pair = |folder: &str| {
        let items = |scan: &TreeScan| scan.folder(Path::new(folder)).map(|f| f.items).unwrap_or_default();
        align(items(&left), items(&right), Alignment::new(true))
    };
    let kinds = |rows: Vec<otd_folder_engine::Aligned>| -> Vec<_> {
        rows.into_iter().map(|row| (row.name, row.left.map(|i| i.kind), row.right.map(|i| i.kind))).collect()
    };
    for scan in [&left, &right] {
        while scan.step() {}
    }
    assert_eq!(kinds(pair("")), [("pack".to_owned(), Some(ItemKind::File), Some(ItemKind::Folder))], "unopened, a file against a folder");
    for scan in [&left, &right] {
        scan.open(Path::new("pack"));
        while scan.step() {}
    }
    assert_eq!(kinds(pair("")), [("pack".to_owned(), Some(ItemKind::Folder), Some(ItemKind::Folder))]);
    let file = Some(ItemKind::File);
    assert_eq!(
        kinds(pair("pack")),
        [("a.txt".to_owned(), file, None), ("b.txt".to_owned(), file, file), ("c.txt".to_owned(), None, file)],
        "the container's children pair with the folder's and leave orphans"
    );
}

#[test]
fn an_unopened_container_pair_gets_a_files_status_and_an_expanded_one_a_folders() {
    let criteria = Criteria {
        size: true,
        modified: true,
        tolerance: Duration::from_secs(2),
        filename_case: false,
        unix_permissions: false,
        unix_owner: false,
        unix_group: false,
    };
    // Right-side content and time against the left's "ab" at 10 s: different size, newer, within tolerance.
    let mut seen = Vec::new();
    for (content, time) in [("abc", 10), ("ab", 20), ("ab", 11)] {
        let side = |content: &str, time: u64| {
            let spec =
                source().container("pack", content, at(time)).file("pack/x.txt", "x", at(1)).file("plain", content, at(time)).folder("dir", at(time));
            TreeScan::new(Arc::new(spec), "", false).0
        };
        let (left, right) = (side("ab", 10), side(content, time));
        let status = |name: &str| {
            let items = |scan: &TreeScan| scan.folder(Path::new("")).map(|f| f.items).unwrap_or_default();
            let row = align(items(&left), items(&right), Alignment::new(true)).into_iter().find(|row| row.name == name).expect("the row is aligned");
            quick_test(&row, &criteria)
        };
        for scan in [&left, &right] {
            while scan.step() {}
        }
        assert_eq!(status("pack"), status("plain"), "unopened, {content} at {time}");
        seen.push(status("pack"));
        for scan in [&left, &right] {
            scan.open(Path::new("pack"));
            while scan.step() {}
        }
        assert_eq!(status("pack"), status("dir"), "expanded, {content} at {time}");
    }
    seen.dedup();
    assert_eq!(seen.len(), 3, "each case gets its own status: {seen:?}");
}
