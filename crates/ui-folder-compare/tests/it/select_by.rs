//! Select Newer, Select Orphans and Invert Selection, run as the Edit menu runs them.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use egui::accesskit::Live;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use otd_core::command::{self, Value};
use otd_core::i18n::t;
use otd_ui::Shell;
use otd_ui_folder_compare::{FolderCompare, SELECTED_COUNTS, commands};

use crate::view::{fixture, harness, settle};

/// Writes `content` to `path`, modified `seconds` after the epoch.
fn write_at(path: &Path, content: &str, seconds: u64) {
    fs::write(path, content).unwrap();
    fs::File::options().write(true).open(path).unwrap().set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)).unwrap();
}

/// The view fixture, plus a file newer on each side, and one newer on the left inside the folder
/// `both`, listed and then closed again, so it is compared but not visible.
fn open(root: &Path) -> Harness<'static, Shell> {
    fixture(root);
    let (left, right) = (root.join("left"), root.join("right"));
    for (name, l, r) in
        [("newer-left.txt", 2_000_000, 1_000_000), ("newer-right.txt", 1_000_000, 2_000_000), ("both/inner.txt", 2_000_000, 1_000_000)]
    {
        write_at(&left.join(name), "x", l);
        write_at(&right.join(name), "x", r);
    }
    let mut harness = harness(FolderCompare::of_folders(&["left".into(), "right".into()], root));
    run(&mut harness, "folder-compare.expand_all");
    settle(&mut harness);
    run(&mut harness, "folder-compare.collapse_all");
    let both = view(&harness).shown().into_iter().find(|shown| shown.row.name == "both").map(|shown| shown.row.children.len());
    assert_eq!(both, Some(1), "`both` is listed, holding inner.txt");
    harness
}

fn run(harness: &mut Harness<'_, Shell>, id: &str) {
    assert!(harness.state_mut().run(id), "{id} ran");
    harness.run();
}

fn view<'a>(harness: &'a Harness<'_, Shell>) -> &'a FolderCompare {
    harness.state().view::<FolderCompare>().expect("a Folder Compare tab")
}

fn selected(harness: &Harness<'_, Shell>, side: usize) -> BTreeSet<PathBuf> {
    view(harness).selected(side).into_iter().map(Path::to_path_buf).collect()
}

fn paths(names: &[&str]) -> BTreeSet<PathBuf> {
    names.iter().map(PathBuf::from).collect()
}

#[test]
fn select_left_newer_selects_exactly_the_visible_left_newer_items() {
    let root = tempfile::tempdir().unwrap();
    let mut harness = open(root.path());
    run(&mut harness, "folder-compare.select_orphans");
    assert!(!selected(&harness, 1).is_empty(), "something selected on the right first");

    run(&mut harness, "folder-compare.select_left_newer");
    // Not `both/inner.txt`, inside the closed folder, nor `both`, which only holds it.
    assert_eq!(selected(&harness, 0), paths(&["newer-left.txt"]));
    assert!(selected(&harness, 1).is_empty(), "the selection is replaced, the right's too");
}

#[test]
fn select_orphans_selects_each_sides_own_orphans() {
    let root = tempfile::tempdir().unwrap();
    let mut harness = open(root.path());
    run(&mut harness, "folder-compare.select_orphans");
    assert_eq!([selected(&harness, 0), selected(&harness, 1)], [paths(&["only-left.txt"]), paths(&["only-right.txt"])]);
}

#[test]
fn invert_selection_flips_the_selection_within_the_visible_items() {
    let root = tempfile::tempdir().unwrap();
    let mut harness = open(root.path());
    let shown = |harness: &Harness<'_, Shell>, side: usize| -> BTreeSet<PathBuf> {
        let view = view(harness);
        let shown = view.shown();
        shown
            .iter()
            .filter(|shown| if side == 0 { shown.row.left.is_some() } else { shown.row.right.is_some() })
            .map(|shown| shown.key.clone())
            .collect()
    };
    run(&mut harness, "folder-compare.select_left_newer");
    let before = selected(&harness, 0);

    run(&mut harness, "folder-compare.invert_selection");
    let left: BTreeSet<_> = shown(&harness, 0).into_iter().filter(|key| !before.contains(key)).collect();
    assert_eq!(selected(&harness, 0), left, "the left's other visible items");
    assert!(!left.contains(&PathBuf::from("both/inner.txt")), "nothing inside the closed folder");
    assert_eq!(selected(&harness, 1), shown(&harness, 1), "every visible right item, and no gap");

    run(&mut harness, "folder-compare.invert_selection");
    assert_eq!(selected(&harness, 0), before, "inverting twice gives the selection back");
}

#[test]
fn a_selection_command_announces_the_counts_and_gives_them_to_scripts() {
    let root = tempfile::tempdir().unwrap();
    let mut harness = open(root.path());
    let text = command::run(&commands(), harness.state_mut(), "folder-compare.select_orphans", &[], false);
    harness.run();
    let Ok(Some(Value::Text(text))) = text else { panic!("the counts as text: {text:?}") };
    let prefix = t(SELECTED_COUNTS).split('{').next().unwrap_or_default();
    assert!(text.starts_with(prefix) && text.contains('1'), "{text}");
    let announced = harness.query_all_by(|node| node.live() == Live::Polite && [node.label(), node.value()].contains(&Some(text.clone())));
    assert!(announced.count() > 0, "announced, since the selection changed away from focus");
}
