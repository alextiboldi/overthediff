//! Name filters through the visibility hook (E08-S3-T1): the catalogue's A.1 examples over two
//! identical scanned `MemorySource` trees, read back as the items left visible on each side.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use otd_folder_engine::{Criteria, NameFilter, Row, TreeScan, TreeSide, compare_trees, resolve_selection};
use otd_source::{DataSource, ItemKind, MemorySource};

use crate::fixture::{at, caps};

const CRITERIA: Criteria = Criteria { size: true, modified: true, tolerance: Duration::from_secs(2) };

/// Paths below the base folder; folders end in `/`. It nests `Windows` in `Windows` and `Backup` in
/// `Source`, and puts a `Source` and a `Windows` in a folder that no example names.
const TREE: &[&str] = &[
    "a.pas",
    "a.dcu",
    "readme.txt",
    "Backup/old.pas",
    "Backup/notes.txt",
    "Source/main.pas",
    "Source/main.dfm",
    "Source/main.dcu",
    "Source/Backup/main.pas",
    "Windows/win.txt",
    "Windows/win.ini",
    "Windows/Windows/deep.txt",
    "Windows/System/sys.txt",
    "lib/util.pas",
    "lib/util.ddp",
    "lib/Source/lib.dpr",
    "lib/Windows/lib.txt",
];

const FOLDERS: &[&str] =
    &["Backup/", "Source/", "Source/Backup/", "Windows/", "Windows/Windows/", "Windows/System/", "lib/", "lib/Source/", "lib/Windows/"];

fn tree() -> MemorySource {
    TREE.iter().fold(MemorySource::new(caps(true)), |source, path| source.file(path, "x", at(1)))
}

/// `source` scanned all the way down, or only its base folder.
fn scan(source: MemorySource, deep: bool) -> TreeScan {
    let (scan, _events) = TreeScan::new(Arc::new(source), "", deep);
    while scan.step() {}
    scan
}

fn filtered(left: MemorySource, right: MemorySource, masks: &str, deep: bool) -> Vec<Row> {
    let case = [left.capabilities().case_sensitive, right.capabilities().case_sensitive];
    let filter = NameFilter::new(masks, case).unwrap();
    let mut rows =
        compare_trees(&scan(left, deep), &scan(right, deep), &CRITERIA, case[0] && case[1], &|side, path, item| filter.visible(side, path, item));
    filter.hide_empty_folders(&mut rows);
    rows
}

/// The items visible on `side`, as `TREE` writes them.
fn visible(rows: &[Row], side: TreeSide) -> BTreeSet<String> {
    fn walk(rows: &[Row], side: TreeSide, parent: &Path, out: &mut BTreeSet<String>) {
        for row in rows {
            let (item, hidden) = match side {
                TreeSide::Left => (&row.left, row.left_hidden),
                TreeSide::Right => (&row.right, row.right_hidden),
            };
            let Some(item) = item.as_ref().filter(|_| !hidden) else { continue };
            let path = parent.join(&item.name);
            let folder = item.kind == ItemKind::Folder;
            out.insert(format!("{}{}", path.to_string_lossy().replace('\\', "/"), if folder { "/" } else { "" }));
            walk(&row.children, side, &path, out);
        }
    }
    let mut out = BTreeSet::new();
    walk(rows, side, &PathBuf::new(), &mut out);
    out
}

enum Want {
    /// These files and every folder.
    AllFolders(&'static [&'static str]),
    Only(&'static [&'static str]),
}

const EXAMPLES: &[(&str, Want)] = &[
    (
        "*.pas;*.dfm;*.dpr",
        Want::AllFolders(&[
            "a.pas",
            "Backup/old.pas",
            "Source/main.pas",
            "Source/main.dfm",
            "Source/Backup/main.pas",
            "lib/util.pas",
            "lib/Source/lib.dpr",
        ]),
    ),
    // The base folder's files stay, and `lib/` shows because it leads to `lib/Source/`.
    (
        "Source\\",
        Want::Only(&[
            "a.pas",
            "a.dcu",
            "readme.txt",
            "Source/",
            "Source/main.pas",
            "Source/main.dfm",
            "Source/main.dcu",
            "Source/Backup/",
            "Source/Backup/main.pas",
            "lib/",
            "lib/Source/",
            "lib/Source/lib.dpr",
        ]),
    ),
    (
        "-*.dcu;-*.ddp",
        Want::AllFolders(&[
            "a.pas",
            "readme.txt",
            "Backup/old.pas",
            "Backup/notes.txt",
            "Source/main.pas",
            "Source/main.dfm",
            "Source/Backup/main.pas",
            "Windows/win.txt",
            "Windows/win.ini",
            "Windows/Windows/deep.txt",
            "Windows/System/sys.txt",
            "lib/util.pas",
            "lib/Source/lib.dpr",
            "lib/Windows/lib.txt",
        ]),
    ),
    (
        "-Backup\\",
        Want::Only(&[
            "a.pas",
            "a.dcu",
            "readme.txt",
            "Source/",
            "Source/main.pas",
            "Source/main.dfm",
            "Source/main.dcu",
            "Windows/",
            "Windows/win.txt",
            "Windows/win.ini",
            "Windows/Windows/",
            "Windows/Windows/deep.txt",
            "Windows/System/",
            "Windows/System/sys.txt",
            "lib/",
            "lib/util.pas",
            "lib/util.ddp",
            "lib/Source/",
            "lib/Source/lib.dpr",
            "lib/Windows/",
            "lib/Windows/lib.txt",
        ]),
    ),
    // `.txt` files directly in any `Windows` folder, not in its subfolder `System`.
    ("Windows\\*.txt", Want::AllFolders(&["Windows/win.txt", "Windows/Windows/deep.txt", "lib/Windows/lib.txt"])),
    ("...\\Windows\\*.txt", Want::AllFolders(&["Windows/win.txt", "Windows/Windows/deep.txt", "lib/Windows/lib.txt"])),
    (".\\Windows\\*.txt", Want::AllFolders(&["Windows/win.txt"])),
];

#[test]
fn each_catalogue_example_leaves_exactly_its_items_visible_on_both_sides() {
    let failures: Vec<String> = EXAMPLES
        .iter()
        .flat_map(|(masks, want)| {
            let want: BTreeSet<String> = match want {
                Want::AllFolders(files) => FOLDERS.iter().chain(files.iter()).map(|s| s.to_string()).collect(),
                Want::Only(items) => items.iter().map(|s| s.to_string()).collect(),
            };
            let rows = filtered(tree(), tree(), masks, true);
            [TreeSide::Left, TreeSide::Right].into_iter().filter_map(move |side| {
                let got = visible(&rows, side);
                (got != want).then(|| {
                    format!(
                        "{masks:?} {side:?}: missing {:?}, extra {:?}",
                        want.difference(&got).collect::<Vec<_>>(),
                        got.difference(&want).collect::<Vec<_>>()
                    )
                })
            })
        })
        .collect();
    assert!(failures.is_empty(), "mask lists that leave the wrong items visible:\n{}", failures.join("\n"));
}

#[test]
fn a_folder_shown_only_for_its_contents_stays_visible_until_they_are_listed() {
    let rows = filtered(tree(), tree(), "Source\\", false);
    let shown = visible(&rows, TreeSide::Left);
    assert!(shown.contains("lib/") && shown.contains("Windows/"), "unlisted folders may hold a Source folder: {shown:?}");
}

#[test]
fn a_folder_shown_only_for_its_contents_hides_on_the_side_that_shows_nothing_in_it() {
    let left = MemorySource::new(caps(true)).file("P/Other/Source/x.txt", "x", at(1));
    let right = MemorySource::new(caps(true)).file("P/Other/junk.txt", "x", at(1));
    let rows = filtered(left, right, "Source\\", true);
    assert!(visible(&rows, TreeSide::Left).contains("P/Other/"), "the left leads to Source");
    let resolved = resolve_selection(&rows, TreeSide::Right, &[Path::new("P")]);
    assert!(resolved.items.is_empty(), "the right holds only a filtered file: {:?}", resolved.items);
}
