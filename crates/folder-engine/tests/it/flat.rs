//! Flattened alignment: the files of two whole trees pair by name whatever subfolder they are in,
//! where per-folder alignment of the same trees does not pair them.

use std::path::{Path, PathBuf};

use otd_folder_engine::{Aligned, Placed, align, align_flat};
use otd_source::{DataSource, ItemKind, Links, MemorySource};

use crate::fixture::{at, caps};

fn tree(paths: &[&str]) -> MemorySource {
    paths.iter().fold(MemorySource::new(caps(true)), |source, path| source.file(path, *path, at(1)))
}

/// Every item of the tree with its folder, as a finished scan would hold them.
fn placed(source: &MemorySource, folder: &Path) -> Vec<Placed> {
    let items = source.list(folder, Links::Report).expect("the folder lists");
    let below = items.iter().filter(|item| item.kind == ItemKind::Folder).flat_map(|item| placed(source, &folder.join(&item.name)));
    let below: Vec<_> = below.collect();
    items.into_iter().map(|item| Placed { folder: folder.to_path_buf(), item }).chain(below).collect()
}

/// The (left path, right path) of every row that has both sides, through per-folder alignment
/// that descends into folders present on both sides.
fn paired_per_folder(left: &MemorySource, right: &MemorySource, folder: &Path) -> Vec<(PathBuf, PathBuf)> {
    let rows = align(left.list(folder, Links::Report).expect("left lists"), right.list(folder, Links::Report).expect("right lists"), true);
    rows.into_iter()
        .flat_map(|row| match (row.left, row.right) {
            (Some(l), Some(r)) if l.kind == ItemKind::Folder && r.kind == ItemKind::Folder => paired_per_folder(left, right, &folder.join(&l.name)),
            (Some(l), Some(r)) => vec![(folder.join(l.name), folder.join(r.name))],
            _ => Vec::new(),
        })
        .collect()
}

/// Each row's (left path, right path).
fn paths(rows: &[Aligned<Placed>]) -> Vec<(Option<PathBuf>, Option<PathBuf>)> {
    let path = |side: &Option<Placed>| side.as_ref().map(|p| p.folder.join(&p.item.name));
    rows.iter().map(|row| (path(&row.left), path(&row.right))).collect()
}

fn some(path: &str) -> Option<PathBuf> {
    Some(PathBuf::from(path))
}

#[test]
fn a_file_aligns_with_its_namesake_in_another_subfolder_when_flattened_and_not_otherwise() {
    let (left, right) = (tree(&["a/x.txt"]), tree(&["b/x.txt"]));
    let flat = align_flat(placed(&left, Path::new("")), placed(&right, Path::new("")), true);
    assert_eq!(paths(&flat), [(some("a/x.txt"), some("b/x.txt"))], "folders are not rows when flattened");
    assert!(paired_per_folder(&left, &right, Path::new("")).is_empty());
}

/// `b/x.txt` is on both sides, so it pairs with itself, and the earlier `a/x.txt` takes `d/x.txt`.
#[test]
fn among_files_of_one_name_the_same_path_pairs_first_then_folder_order_and_the_rest_are_orphans() {
    let left = placed(&tree(&["c/x.txt", "b/x.txt", "a/x.txt", "z.txt"]), Path::new(""));
    let right = placed(&tree(&["d/x.txt", "b/x.txt"]), Path::new(""));
    let rows = align_flat(left.clone(), right.clone(), true);
    let expected = [(some("b/x.txt"), some("b/x.txt")), (some("a/x.txt"), some("d/x.txt")), (some("c/x.txt"), None), (some("z.txt"), None)];
    assert_eq!(paths(&rows), expected);
    let reversed = |side: &[Placed]| side.iter().rev().cloned().collect::<Vec<_>>();
    assert_eq!(align_flat(reversed(&left), reversed(&right), true), rows);
}
