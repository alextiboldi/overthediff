//! Mask lists over a sample tree: the catalogue examples from story E02-S1's acceptance criteria,
//! then the rules A.1 leaves open that the story records.

use std::collections::BTreeSet;

use otd_mask::{Case, Folder, MaskList};

/// Paths below the base folder, names joined by `/` here only for reading. Folders end in `/`.
/// It nests `Windows` in `Windows` and `Backup` in `Source`, and puts a `Source` and a `Windows`
/// in a folder that no example names.
const TREE: &[&str] = &[
    "a.pas",
    "a.dcu",
    "readme.txt",
    "Backup/",
    "Backup/old.pas",
    "Backup/notes.txt",
    "Source/",
    "Source/main.pas",
    "Source/main.dfm",
    "Source/main.dcu",
    "Source/Backup/",
    "Source/Backup/main.pas",
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
];

/// What the folder view shows for `list`: included files, included folders, and the folders that
/// lead to either. The last is the caller's part of "folders containing included files are
/// included", which [`Folder::IfContainsIncluded`] hands it.
fn shown(list: &str) -> BTreeSet<&'static str> {
    let masks = MaskList::new(list, Case::Sensitive).unwrap();
    let names = |item: &str| item.trim_end_matches('/').split('/').map(str::to_owned).collect::<Vec<_>>();
    let verdict = |item: &str| item.ends_with('/').then(|| masks.folder(&names(item)));
    let direct: Vec<&str> = TREE
        .iter()
        .copied()
        .filter(|item| match verdict(item) {
            Some(folder) => folder == Folder::Included,
            None => masks.includes_file(&names(item)),
        })
        .collect();
    let leading = TREE.iter().copied().filter(|item| verdict(item) == Some(Folder::IfContainsIncluded) && direct.iter().any(|d| d.starts_with(item)));
    direct.iter().copied().chain(leading).collect()
}

fn folders() -> impl Iterator<Item = &'static str> {
    TREE.iter().copied().filter(|item| item.ends_with('/'))
}

/// Mask list, and the items it shows besides every folder or besides the folders listed.
enum Want {
    AllFolders(&'static [&'static str]),
    Only(&'static [&'static str]),
}

const EXAMPLES: &[(&str, Want)] = &[
    // The catalogue examples.
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
    // The base folder's own files stay, and `lib/` shows because it leads to `lib/Source/`.
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
    // `...` in the middle: zero or more folders between `Windows` and the file.
    ("Windows\\...\\*.txt", Want::AllFolders(&["Windows/win.txt", "Windows/Windows/deep.txt", "Windows/System/sys.txt", "lib/Windows/lib.txt"])),
    // An exclude wins over an include.
    (
        "*.pas;-Backup\\",
        Want::Only(&[
            "a.pas",
            "Source/",
            "Source/main.pas",
            "Windows/",
            "Windows/Windows/",
            "Windows/System/",
            "lib/",
            "lib/util.pas",
            "lib/Source/",
            "lib/Windows/",
        ]),
    ),
    // A file passes the file masks and an included folder both.
    ("Source\\;*.pas", Want::Only(&["a.pas", "Source/", "Source/main.pas", "Source/Backup/", "Source/Backup/main.pas", "lib/", "lib/Source/"])),
];

#[test]
fn each_example_shows_exactly_its_items_on_the_sample_tree() {
    let failures: Vec<String> = EXAMPLES
        .iter()
        .filter_map(|(list, want)| {
            let want: BTreeSet<&str> = match want {
                Want::AllFolders(files) => folders().chain(files.iter().copied()).collect(),
                Want::Only(items) => items.iter().copied().collect(),
            };
            let got = shown(list);
            (got != want).then(|| {
                format!("{list:?}: missing {:?}, extra {:?}", want.difference(&got).collect::<Vec<_>>(), got.difference(&want).collect::<Vec<_>>())
            })
        })
        .collect();
    assert!(failures.is_empty(), "mask lists that show the wrong items:\n{}", failures.join("\n"));
}
