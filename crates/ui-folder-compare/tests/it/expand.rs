//! The tree expansion commands (E08-S1-T3) in the real shell, run as a menu runs them, with the
//! rows read back through the accessibility tree. The fixture trees are made in a temporary folder.

use std::fs;
use std::path::Path;

use egui::Key;
use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_ui::Shell;
use otd_ui_folder_compare::{FolderCompare, NOT_ON_RIGHT, ON_LEFT, ON_RIGHT};

use crate::view::{focused, harness, on, press, rows, settle};

/// On both sides: `a/b/c/deep.txt` three folders down, a sibling folder `s` with `child.txt`, and
/// `top.txt`; on the left only, the folder `only` with `inner.txt`.
fn fixture(root: &Path) -> Harness<'static, Shell> {
    for side in ["left", "right"] {
        let base = root.join(side);
        fs::create_dir_all(base.join("a/b/c")).unwrap();
        fs::write(base.join("a/b/c/deep.txt"), "deep").unwrap();
        fs::create_dir_all(base.join("s")).unwrap();
        fs::write(base.join("s/child.txt"), "child").unwrap();
        fs::write(base.join("top.txt"), "top").unwrap();
    }
    fs::create_dir_all(root.join("left/only")).unwrap();
    fs::write(root.join("left/only/inner.txt"), "inner").unwrap();
    harness(FolderCompare::of_folders(&["left".into(), "right".into()], root))
}

/// Runs a command as the menus do, then waits for the listings it asked for.
fn run(harness: &mut Harness<'_, Shell>, id: &str) {
    assert!(harness.state_mut().run(id), "{id} ran");
    settle(harness);
}

/// Moves focus onto the left name cell of the row named `name`.
fn focus(harness: &mut Harness<'_, Shell>, name: &str) {
    for _ in 0..40 {
        if focused(harness).is_some_and(|(role, _)| role == Role::Cell) {
            break;
        }
        press(harness, Key::Tab);
    }
    let target = on(ON_LEFT, name);
    for key in [Key::ArrowDown; 20].into_iter().chain([Key::ArrowUp; 20]) {
        if focused(harness).is_some_and(|(_, label)| label == target) {
            return;
        }
        press(harness, key);
    }
    panic!("the arrows never reached {target:?}");
}

fn shows(harness: &Harness<'_, Shell>, cell: &str) -> bool {
    rows(harness).iter().any(|row| row.iter().any(|c| c == cell))
}

/// The row named `name`'s tree level and, for a folder, whether it is open.
fn row_state(harness: &Harness<'_, Shell>, name: &str) -> (Option<usize>, Option<bool>) {
    let row = harness.get_by(|node| node.role() == Role::Row && node.label().as_deref() == Some(name));
    let node = row.accesskit_node();
    (node.level(), node.data().is_expanded())
}

#[test]
fn open_subfolders_opens_the_folder_and_every_folder_below_it_listing_them_as_needed_but_not_its_siblings() {
    let root = tempfile::tempdir().unwrap();
    let mut harness = fixture(root.path());
    assert!(!shows(&harness, &on(ON_LEFT, "deep.txt")), "nothing below the base folders shows before expansion");

    focus(&mut harness, "a");
    run(&mut harness, "folder-compare.open_subfolders");

    for side in [ON_LEFT, ON_RIGHT] {
        assert!(shows(&harness, &on(side, "deep.txt")), "three folders down, on both sides");
    }
    assert!(!shows(&harness, &on(ON_LEFT, "child.txt")), "a sibling folder stays closed");
    for folder in ["a", "b", "c"] {
        assert_eq!(row_state(&harness, folder).1, Some(true), "{folder} reads as open");
    }
    assert_eq!(row_state(&harness, "s").1, Some(false), "a closed folder reads as closed");
    assert!(row_state(&harness, "deep.txt").0 > row_state(&harness, "a").0, "a row's level is deeper than its folder's");
    assert_eq!(row_state(&harness, "deep.txt").1, None, "a file has no open state");
}

#[test]
fn expand_all_opens_every_folder_and_collapse_all_closes_them_all() {
    let root = tempfile::tempdir().unwrap();
    let mut harness = fixture(root.path());
    // By row names: a folder's Size, blank before, shows once Expand All has listed it.
    let names = |harness: &Harness<'_, Shell>| -> Vec<String> {
        harness.query_all_by_role(Role::Row).filter_map(|row| row.accesskit_node().label()).collect()
    };
    let closed = names(&harness);

    run(&mut harness, "folder-compare.expand_all");
    for cell in [on(ON_LEFT, "deep.txt"), on(ON_RIGHT, "child.txt"), on(ON_LEFT, "inner.txt")] {
        assert!(shows(&harness, &cell), "{cell} shows");
    }
    let inner = rows(&harness).into_iter().find(|row| row.contains(&on(ON_LEFT, "inner.txt")));
    assert!(inner.is_some_and(|row| row.contains(&NOT_ON_RIGHT.to_owned())), "an orphan folder opens to orphans facing gaps");

    run(&mut harness, "folder-compare.collapse_all");
    assert_eq!(names(&harness), closed, "back to the base folders' rows");
    assert!(!harness.state_mut().run("folder-compare.collapse_all"), "with nothing open, Collapse All is disabled");
}

#[test]
fn close_folder_keeps_the_folders_open_inside_it_and_close_subfolders_closes_them_too() {
    let root = tempfile::tempdir().unwrap();
    let mut harness = fixture(root.path());
    focus(&mut harness, "a");
    run(&mut harness, "folder-compare.open_subfolders");

    run(&mut harness, "folder-compare.close_folder");
    assert!(!shows(&harness, &on(ON_LEFT, "b")), "Close Folder hides the folder's rows");
    run(&mut harness, "folder-compare.open_folder");
    assert!(shows(&harness, &on(ON_LEFT, "deep.txt")), "opened again, the folders inside are still open");

    run(&mut harness, "folder-compare.close_subfolders");
    run(&mut harness, "folder-compare.open_folder");
    assert!(shows(&harness, &on(ON_LEFT, "b")), "Open Folder opens one level");
    assert!(!shows(&harness, &on(ON_LEFT, "c")), "after Close Subfolders, the folders inside are closed");
}

#[test]
fn focus_stays_on_its_row_as_folders_open_above_it_and_moves_to_the_open_folder_above_when_a_close_hides_it() {
    let root = tempfile::tempdir().unwrap();
    let mut harness = fixture(root.path());
    focus(&mut harness, "top.txt");
    run(&mut harness, "folder-compare.expand_all");
    assert_eq!(focused(&harness).map(|(_, name)| name), Some(on(ON_LEFT, "top.txt")), "focus stays on its row");

    focus(&mut harness, "deep.txt");
    run(&mut harness, "folder-compare.collapse_all");
    assert_eq!(focused(&harness).map(|(_, name)| name), Some(on(ON_LEFT, "a")), "focus moves to the shown folder holding it");
}

#[test]
fn a_folder_command_on_a_file_row_is_refused() {
    let root = tempfile::tempdir().unwrap();
    let mut harness = fixture(root.path());
    focus(&mut harness, "top.txt");
    for id in ["folder-compare.open_folder", "folder-compare.open_subfolders", "folder-compare.close_subfolders"] {
        assert!(!harness.state_mut().run(id), "{id} is disabled on a file");
    }
}

#[test]
fn a_click_in_an_open_folder_selects_that_row_by_its_path_through_the_folders() {
    let root = tempfile::tempdir().unwrap();
    let mut harness = fixture(root.path());
    focus(&mut harness, "s");
    run(&mut harness, "folder-compare.open_folder");

    harness.get_by_label(&on(ON_LEFT, "child.txt")).click();
    harness.run();
    let view = harness.state().view::<FolderCompare>().expect("a Folder Compare tab");
    assert_eq!(view.selected(0), [Path::new("s/child.txt")], "the child's path from the base, on its side only");
    assert!(view.selected(1).is_empty());
}

#[test]
fn a_close_that_hides_a_selected_row_drops_it_from_the_selection() {
    let root = tempfile::tempdir().unwrap();
    let mut harness = fixture(root.path());
    run(&mut harness, "folder-compare.expand_all");
    harness.get_by_label(&on(ON_LEFT, "child.txt")).click();
    harness.run();

    run(&mut harness, "folder-compare.collapse_all");
    let view = harness.state().view::<FolderCompare>().expect("a Folder Compare tab");
    assert!(view.selected(0).is_empty(), "no command may act on a row nobody can see");
}

#[test]
fn refresh_selection_on_a_row_in_an_open_folder_lists_that_folder_again() {
    let root = tempfile::tempdir().unwrap();
    let mut harness = fixture(root.path());
    focus(&mut harness, "s");
    run(&mut harness, "folder-compare.open_folder");
    focus(&mut harness, "child.txt");
    let status = |harness: &Harness<'_, Shell>| {
        let view = harness.state().view::<FolderCompare>().expect("a Folder Compare tab");
        view.shown().iter().find(|shown| shown.row.name == "child.txt").map(|shown| shown.row.status.clone())
    };
    let before = status(&harness);

    // Longer, so its size differs.
    fs::write(root.path().join("right/s/child.txt"), "no longer the same child").unwrap();
    run(&mut harness, "folder-compare.refresh_selection");
    assert_ne!(status(&harness), before, "the row's status follows the file");
}
