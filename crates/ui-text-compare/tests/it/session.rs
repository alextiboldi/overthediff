//! The Session menu's commands, run as the menus run them, and the status bar's fields and Text
//! Compare Info, read back through the view, the command results and the accessibility tree.

use std::path::Path;

use egui::accesskit::Role;
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_core::command::{self, CommandError};
use otd_core::session::Value;
use otd_core::task::End;
use otd_ui::{CLOSE, Shell, View as _};
use otd_ui_text_compare::session::IMPORTANT;
use otd_ui_text_compare::{Info, LEFT, NO_RESULT, SESSION_TYPE, TEXT_COMPARE_INFO, TextCompareView, commands};

/// The shell with Text Compare's commands, showing the files `left` and `right` written in a
/// temporary folder, and that folder.
fn harness(left: &str, right: &str) -> (Harness<'static, Shell>, tempfile::TempDir) {
    let folder = tempfile::tempdir().unwrap();
    let mut view = TextCompareView::new();
    let specs = [("left.txt", left), ("right.txt", right)].map(|(name, text)| write(folder.path(), name, text));
    view.session().expect("a session").specs = specs.to_vec();
    let mut shell = Shell::new(vec![SESSION_TYPE]).with_commands(commands());
    shell.open(Box::new(view));
    let mut harness = Harness::builder().build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), shell);
    harness.run();
    (harness, folder)
}

fn write(folder: &Path, name: &str, text: &str) -> String {
    let path = folder.join(name);
    std::fs::write(&path, text).unwrap();
    path.display().to_string()
}

fn view<'a>(harness: &'a Harness<'_, Shell>) -> &'a TextCompareView {
    harness.state().view::<TextCompareView>().expect("a Text Compare tab is active")
}

fn info(harness: &Harness<'_, Shell>) -> Info {
    view(harness).info().expect("a completed comparison")
}

fn run(harness: &mut Harness<'_, Shell>, id: &str) {
    assert!(harness.state_mut().run(id), "{id} ran");
    harness.run();
}

fn focused(harness: &Harness<'_, Shell>) -> Option<(Role, String)> {
    let node = harness.query_by(|node| node.is_focused())?;
    let node = node.accesskit_node();
    Some((node.role(), node.label().unwrap_or_default()))
}

/// One line deleted (b) and two inserted (d, e), reading the left side as the earlier one.
const EARLIER: &str = "a\nb\nc\n";
const LATER: &str = "a\nc\nd\ne\n";

#[test]
fn swap_sides_moves_each_file_to_the_other_pane_and_compares_them_again() {
    let (mut harness, folder) = harness(EARLIER, LATER);
    let before = info(&harness);

    run(&mut harness, "text-compare.swap_sides");
    let right = folder.path().join("right.txt").display().to_string();
    assert_eq!(view(&harness).pane(0).map(|pane| pane.path.as_str()), Some(right.as_str()), "the right file is in the left pane");
    let left_edit = harness.get_by(|node| node.role() == Role::TextInput && node.label().as_deref() == Some(LEFT)).accesskit_node().value();
    assert_eq!(left_edit, Some(right), "and in the left path edit");
    let after = info(&harness);
    assert_eq!((after.inserted, after.deleted), (before.deleted, before.inserted), "what was deleted now reads as inserted");
    assert_eq!((after.left_lines, after.right_lines), (before.right_lines, before.left_lines));
}

#[test]
fn info_counts_each_panes_lines_and_its_changed_inserted_and_deleted_lines_for_the_window_and_scripts() {
    let (mut harness, _folder) = harness("same\nold\ngone\nend\n", "same\nnew\nend\nadded\n");
    let info = info(&harness);
    let view = view(&harness);
    assert_eq!((info.left_lines, info.right_lines), (view.pane(0).unwrap().lines.len(), view.pane(1).unwrap().lines.len()));
    assert_eq!((info.changed, info.inserted, info.deleted), (1, 1, 1));

    // Without the window, the command gives the same figures as text.
    let result = command::run(&commands(), harness.state_mut(), "text-compare.info", &[], false);
    assert_eq!(result, Ok(Some(Value::Text(info.lines().join("\n")))));
}

#[test]
fn after_a_cancelled_comparison_neither_the_status_bar_nor_info_shows_a_result_until_one_completes() {
    let (mut harness, _folder) = harness(EARLIER, LATER);
    let status = view(&harness).status();
    assert_eq!(status.result.map(|field| field.value), Some(IMPORTANT.to_owned()));
    assert!(status.load_time.is_some(), "loading both files and comparing them took a time");

    harness.state_mut().view_mut::<TextCompareView>().unwrap().comparison_ended(End::Cancelled);
    harness.run();
    assert!(view(&harness).status().result.is_none(), "a cancelled comparison never reads as a result");
    assert_eq!(view(&harness).info(), None);
    let result = command::run(&commands(), harness.state_mut(), "text-compare.info", &[], false);
    assert_eq!(result, Err(CommandError::Failed(NO_RESULT.to_owned())), "a script hears there is none");

    run(&mut harness, "text-compare.recompare_files");
    assert_eq!(view(&harness).status().result.map(|field| field.value), Some(IMPORTANT.to_owned()));
}

#[test]
fn the_info_dialog_takes_focus_lists_the_figures_and_gives_focus_back_on_esc_or_close() {
    for close in [Key::Escape, Key::Enter] {
        let (mut harness, _folder) = harness(EARLIER, LATER);
        let before = Some((Role::TextInput, LEFT.to_owned()));
        for _ in 0..40 {
            if focused(&harness) == before {
                break;
            }
            harness.key_press_modifiers(Modifiers::NONE, Key::Tab);
            harness.run();
        }
        assert_eq!(focused(&harness), before, "Tab reaches the left path edit");

        run(&mut harness, "text-compare.info");
        harness.run();
        let dialog = harness.query_by(|node| node.role() == Role::Dialog && node.label().as_deref() == Some(TEXT_COMPARE_INFO));
        assert!(dialog.is_some(), "the dialog opens, named by its title");
        assert_eq!(focused(&harness), Some((Role::Button, CLOSE.to_owned())), "focus moves into it");
        for line in info(&harness).lines() {
            assert!(harness.query_by_label(&line).is_some(), "it lists {line:?}");
        }

        harness.key_press_modifiers(Modifiers::NONE, close);
        harness.run();
        harness.run();
        assert!(harness.query_by_label(TEXT_COMPARE_INFO).is_none(), "{close:?} closes it");
        assert_eq!(focused(&harness), before, "{close:?} gives focus back");
    }
}
