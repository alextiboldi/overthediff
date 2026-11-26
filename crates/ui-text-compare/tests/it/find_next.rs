//! Find Next, Find Previous and the "Initialize Text to find to be current word" option
//! (E07-S5-T3), driven by keyboard through the real shell and read back through the view and the
//! accessibility tree.

use std::collections::BTreeMap;

use egui::accesskit::{Live, Role};
use egui::os::OperatingSystem;
use egui::{Key, Modifiers, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_core::command::{self, CommandError};
use otd_core::session::Value;
use otd_text_compare::{FindOptions, Pos, Selection};
use otd_ui::{Shell, View as _};
use otd_ui_text_compare::{CURRENT_WORD, LEFT, NOT_FOUND, RIGHT, SESSION_TYPE, TextCompareView, commands, options};

/// The shell with one Text Compare tab on `left` and `right`, the option set to `current_word`, and
/// the folder holding the files.
fn harness(left: &str, right: &str, current_word: bool) -> (Harness<'static, Shell>, tempfile::TempDir) {
    let folder = tempfile::tempdir().unwrap();
    let mut view = TextCompareView::new();
    view.session().expect("a session").specs = [("left.txt", left), ("right.txt", right)]
        .map(|(name, text)| {
            let path = folder.path().join(name);
            std::fs::write(&path, text).unwrap();
            path.display().to_string()
        })
        .to_vec();
    let saved = BTreeMap::from([(CURRENT_WORD.to_owned(), Value::Bool(current_word))]);
    let mut shell = Shell::new(vec![SESSION_TYPE]).with_commands(commands()).with_options(options(), saved).unwrap();
    shell.open(Box::new(view));
    let mut harness = Harness::builder().with_size(vec2(900.0, 600.0)).build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), shell);
    harness.run();
    (harness, folder)
}

fn key(harness: &mut Harness<'_, Shell>, modifiers: Modifiers, key: Key) {
    harness.key_press_modifiers(modifiers, key);
    harness.run();
}

/// Find Next's chord (`forward`) or Find Previous's for the harness's OS (Q24): F3 and Shift+F3 off
/// macOS, Cmd+G and Cmd+Shift+G on it.
fn find_chord(harness: &mut Harness<'_, Shell>, forward: bool) {
    let shift = if forward { Modifiers::NONE } else { Modifiers::SHIFT };
    if harness.ctx.os() == OperatingSystem::Mac {
        key(harness, Modifiers::MAC_CMD | Modifiers::COMMAND | shift, Key::G)
    } else {
        key(harness, shift, Key::F3)
    }
}

fn focused_label(harness: &Harness<'_, Shell>) -> Option<String> {
    harness.query_by(|node| node.is_focused()).and_then(|node| node.accesskit_node().label())
}

/// Moves focus with Tab to the left editor.
fn focus_left(harness: &mut Harness<'_, Shell>) {
    for _ in 0..40 {
        if harness.query_by(|node| node.is_focused()).is_some_and(|node| node.accesskit_node().role() == Role::MultilineTextInput) {
            break;
        }
        key(harness, Modifiers::NONE, Key::Tab);
    }
    assert_eq!(focused_label(harness).as_deref(), Some(LEFT), "the left editor has focus");
}

fn view<'a>(harness: &'a Harness<'_, Shell>) -> &'a TextCompareView {
    harness.state().view::<TextCompareView>().expect("a Text Compare tab")
}

/// The chosen pane and its selection.
fn selected(harness: &Harness<'_, Shell>) -> (usize, Selection) {
    let view = view(harness);
    (view.chosen(), view.pane(view.chosen()).expect("a loaded pane").selection)
}

fn on(line: usize, columns: std::ops::Range<usize>) -> Selection {
    Selection { anchor: Pos { line, column: columns.start }, head: Pos { line, column: columns.end } }
}

fn search(harness: &mut Harness<'_, Shell>, text: &str, options: FindOptions) {
    harness.state_mut().view_mut::<TextCompareView>().unwrap().set_search(text, options).unwrap();
    harness.run();
}

#[test]
fn with_the_cursor_inside_foo_and_no_selection_find_opens_filled_with_foo() {
    let (mut harness, _folder) = harness("let foo = 1;\n", "let foo = 2;\n", true);
    focus_left(&mut harness);
    for _ in 0.."let fo".len() {
        key(&mut harness, Modifiers::NONE, Key::ArrowRight);
    }
    assert_eq!(view(&harness).find_text_on_open(&harness.ctx).as_deref(), Some("foo"));

    key(&mut harness, Modifiers::SHIFT, Key::ArrowRight);
    assert_eq!(view(&harness).find_text_on_open(&harness.ctx), None, "a selection leaves the find box alone");
}

#[test]
fn with_the_option_off_find_leaves_the_find_box_alone() {
    let (mut harness, _folder) = harness("let foo = 1;\n", "let foo = 2;\n", false);
    focus_left(&mut harness);
    key(&mut harness, Modifiers::NONE, Key::ArrowRight);
    assert_eq!(view(&harness).find_text_on_open(&harness.ctx), None);
}

#[test]
fn find_next_repeats_the_last_search_forwards_into_the_right_pane_then_find_previous_comes_back() {
    let (mut harness, _folder) = harness("foo\nbar\nfoo\n", "baz\nfoo\n", false);
    focus_left(&mut harness);
    search(&mut harness, "foo", FindOptions::default());

    find_chord(&mut harness, true);
    assert_eq!(selected(&harness), (0, on(0, 0..3)), "the first match, at the caret");
    find_chord(&mut harness, true);
    assert_eq!(selected(&harness), (0, on(2, 0..3)), "the next one down");
    find_chord(&mut harness, true);
    assert_eq!(selected(&harness), (1, on(1, 0..3)), "then on into the right pane, which becomes the chosen one");
    assert_eq!(focused_label(&harness).as_deref(), Some(RIGHT), "and takes focus");

    find_chord(&mut harness, false);
    assert_eq!(selected(&harness), (0, on(2, 0..3)), "Find Previous goes back to the left pane's last match");
    assert_eq!(focused_label(&harness).as_deref(), Some(LEFT));
}

#[test]
fn on_macos_command_g_and_command_shift_g_are_find_next_and_find_previous() {
    let (mut harness, _folder) = harness("foo\nfoo\n", "bar\n", false);
    harness.ctx.set_os(OperatingSystem::Mac);
    focus_left(&mut harness);
    search(&mut harness, "foo", FindOptions::default());
    find_chord(&mut harness, true);
    find_chord(&mut harness, true);
    assert_eq!(selected(&harness), (0, on(1, 0..3)), "Cmd+G twice: the second match");
    find_chord(&mut harness, false);
    assert_eq!(selected(&harness), (0, on(0, 0..3)), "Cmd+Shift+G: back to the first");
}

#[test]
fn past_the_last_match_without_wrap_find_next_says_not_found_and_leaves_the_selection() {
    let (mut harness, _folder) = harness("foo\n", "bar\n", false);
    search(&mut harness, "foo", FindOptions::default());
    let run = |harness: &mut Harness<'_, Shell>| command::run(&commands(), harness.state_mut(), "text-compare.find_next", &[], false);
    assert_eq!(run(&mut harness), Ok(None), "Find Next runs without the window");
    harness.run();
    let before = selected(&harness);

    assert!(matches!(run(&mut harness), Err(CommandError::Failed(said)) if said.contains("foo")), "automation hears why");
    harness.run();
    assert_eq!(selected(&harness), before, "the selection stays on the last match");
    let message = NOT_FOUND.replace("{text}", "foo");
    let said = harness.get_by_label(&message);
    assert_eq!(said.accesskit_node().live(), Live::Polite, "the miss is announced");
    assert!(view(&harness).status().fields.iter().any(|field| field.value.contains("foo")), "and shown in the status bar");
}

#[test]
fn with_no_search_yet_find_next_and_find_previous_are_disabled() {
    let (mut harness, _folder) = harness("foo\n", "foo\n", false);
    assert!(!harness.state_mut().run("text-compare.find_next"));
    assert!(!harness.state_mut().run("text-compare.find_previous"));
}
