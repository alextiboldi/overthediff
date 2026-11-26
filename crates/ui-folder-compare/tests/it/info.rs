//! Folder Compare Info's counts, the status bar's result, and the Info and Legend dialogs, run as
//! the menus run them and read back through the view, the command results and the accessibility
//! tree.

use std::fs;
use std::path::Path;

use egui::accesskit::Role;
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_core::command;
use otd_core::i18n::t;
use otd_core::session::Value;
use otd_ui::{CLOSE, Shell, View as _};
use otd_ui_folder_compare::look::{IS_DIFFERENT, IS_SAME};
use otd_ui_folder_compare::{FOLDER_COMPARE_INFO, FolderCompare, Info, LEGEND, LEGEND_ENTRIES, NOT_FULLY_COMPARED, commands, session_type};

use crate::look::{at, rows, source};
use crate::view::{fixture, settle};

fn harness(view: FolderCompare) -> Harness<'static, Shell> {
    let mut shell = Shell::new(vec![session_type()]).with_commands(commands());
    shell.open(Box::new(view));
    let mut harness = Harness::builder().build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), shell);
    settle(&mut harness);
    harness
}

fn view<'a>(harness: &'a Harness<'_, Shell>) -> &'a FolderCompare {
    harness.state().view::<FolderCompare>().expect("a Folder Compare tab")
}

fn result(harness: &Harness<'_, Shell>) -> Option<String> {
    view(harness).status().result.map(|field| field.value)
}

fn focused(harness: &Harness<'_, Shell>) -> Option<(Role, String)> {
    let node = harness.query_by(|node| node.is_focused())?;
    let node = node.accesskit_node();
    Some((node.role(), node.label().unwrap_or_default()))
}

/// Left and right folders under `root`, each holding `same.txt`, and `sub` too when asked.
fn same_folders(root: &Path, sub: bool) {
    for side in ["left", "right"] {
        fs::create_dir_all(root.join(side)).unwrap();
        fs::write(root.join(side).join("same.txt"), "same").unwrap();
        if sub {
            fs::create_dir_all(root.join(side).join("sub")).unwrap();
        }
    }
}

#[test]
fn info_counts_each_item_by_its_status_with_a_listed_folder_pair_counted_only_through_its_contents() {
    let trees = || {
        let left = source()
            .file("same.txt", "x", at(100))
            .file("new.txt", "x", at(500))
            .file("size.txt", "xx", at(101))
            .file("only-left.txt", "x", at(1))
            .file("sub/same.txt", "x", at(1))
            .file("sub/only-left.txt", "x", at(1));
        let right = source()
            .file("same.txt", "x", at(100))
            .file("new.txt", "x", at(100))
            .file("size.txt", "x", at(100))
            .file("only-right.txt", "x", at(1))
            .file("sub/same.txt", "x", at(1));
        (left, right)
    };
    let counts =
        |info: Info| [info.same, info.different, info.left_newer, info.right_newer, info.left_orphans, info.right_orphans, info.unknown, info.errors];

    let (left, right) = trees();
    assert_eq!(counts(Info::of(&rows(left, right, true))), [2, 1, 1, 0, 2, 1, 0, 0], "sub is its two items");
    // Not listed, the folder pair is one item not compared yet, and nothing inside it counts.
    let (left, right) = trees();
    assert_eq!(counts(Info::of(&rows(left, right, false))), [1, 1, 1, 0, 1, 1, 1, 0]);
}

#[test]
fn info_gives_the_views_counts_to_the_window_and_scripts() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    let mut harness = harness(FolderCompare::of_folders(&["left".into(), "right".into()], root.path()));
    let info = view(&harness).info().expect("a comparison");
    assert_eq!((info.same, info.left_orphans, info.right_orphans, info.unknown), (1, 1, 1, 1), "same.txt, the orphans and unlisted both/");

    let text = command::run(&commands(), harness.state_mut(), "folder-compare.info", &[], false);
    assert_eq!(text, Ok(Some(Value::Text(info.lines().join("\n")))));
}

#[test]
fn the_status_bar_says_same_only_when_everything_compared_is_the_same() {
    type Case = (fn(&Path), Option<&'static str>);
    let cases: [Case; 4] = [
        (|root| same_folders(root, false), Some(IS_SAME)),
        // The subfolders aren't listed, so it can't say they are the same.
        (|root| same_folders(root, true), Some(NOT_FULLY_COMPARED)),
        (fixture, Some(IS_DIFFERENT)),
        // One base folder is missing: nothing was compared.
        (|root| fs::create_dir_all(root.join("left")).unwrap(), None),
    ];
    for (make, expected) in cases {
        let root = tempfile::tempdir().unwrap();
        make(root.path());
        let harness = harness(FolderCompare::of_folders(&["left".into(), "right".into()], root.path()));
        assert_eq!(result(&harness), expected.map(|key| t(key).to_owned()));
    }
}

#[test]
fn info_and_the_legend_take_focus_list_their_content_and_give_focus_back_on_esc_or_close() {
    let legend: Vec<String> = LEGEND_ENTRIES.iter().map(|entry| t(entry.text).to_owned()).collect();
    for (id, title) in [("folder-compare.info", FOLDER_COMPARE_INFO), ("folder-compare.legend", LEGEND)] {
        for close in [Key::Escape, Key::Enter] {
            let root = tempfile::tempdir().unwrap();
            fixture(root.path());
            let mut harness = harness(FolderCompare::of_folders(&["left".into(), "right".into()], root.path()));
            for _ in 0..40 {
                if focused(&harness).is_some_and(|(role, _)| role == Role::Cell) {
                    break;
                }
                harness.key_press_modifiers(Modifiers::NONE, Key::Tab);
                harness.run();
            }
            let before = focused(&harness);
            assert!(before.as_ref().is_some_and(|(role, _)| *role == Role::Cell), "Tab reaches the rows");

            assert!(harness.state_mut().run(id), "{id} ran");
            harness.run();
            harness.run();
            let dialog = harness.query_by(|node| node.role() == Role::Dialog && node.label().as_deref() == Some(t(title)));
            assert!(dialog.is_some(), "{title} opens, named by its title");
            assert_eq!(focused(&harness), Some((Role::Button, t(CLOSE).to_owned())), "focus moves into {title}");
            let content = if title == LEGEND { legend.clone() } else { view(&harness).info().expect("a comparison").lines() };
            for line in content {
                assert!(harness.query_by_label(&line).is_some(), "{title} lists {line:?}");
            }

            harness.key_press_modifiers(Modifiers::NONE, close);
            harness.run();
            harness.run();
            assert!(harness.query_by_label(t(title)).is_none(), "{close:?} closes {title}");
            assert_eq!(focused(&harness), before, "{close:?} gives focus back from {title}");
        }
    }
}

#[test]
fn the_status_bar_counts_the_base_folders_as_still_to_list_until_their_listings_are_in() {
    // Each scan, from scratch or again, is read before a draw takes in any listing, so it is still
    // running whatever the worker threads have done meanwhile.
    let progress = |harness: &Harness<'_, Shell>| -> Vec<usize> {
        let status = view(harness).status();
        let field = status.fields.iter().find(|field| field.id == "scan").expect("a scan field while scanning");
        field.value.split_whitespace().filter_map(|word| word.parse().ok()).collect()
    };
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    let mut harness = harness(FolderCompare::of_folders(&["left".into(), "right".into()], root.path()));
    for id in ["folder-compare.full_refresh", "folder-compare.refresh"] {
        assert!(harness.state_mut().run(id), "{id} ran");
        let [done, total] = progress(&harness)[..] else { panic!("{id}: done and total") };
        assert!(done < total, "{id}: {done} of {total}, the base folders are still to list");
        settle(&mut harness);
        assert!(view(&harness).status().fields.iter().all(|field| field.id != "scan"), "{id}: no scan field once listed");
    }
}
