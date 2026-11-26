//! The find panel (E07-S5-T2): opened and driven by keyboard, read back through the view and the
//! accessibility tree, and the hidden Find Text run without the window.

use egui::accesskit::{Live, Role};
use egui::{Event, Key, Modifiers, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_core::command;
use otd_core::session::Value;
use otd_ui::{Shell, View as _};
use otd_ui_regex::ELEMENTS_LABEL;
use std::collections::BTreeMap;

use otd_ui_text_compare::{
    CLOSE, CLOSE_AUTOMATICALLY, CURRENT_WORD, FIND, FIND_WHAT, INVALID_PATTERN, LEFT, LINES_SELECTED, NEXT, NOT_FOUND, REGULAR_EXPRESSION,
    SEARCH_HISTORY, SELECT_ALL, SESSION_TYPE, TextCompareView, commands, options,
};

const LEFT_TEXT: &str = "alpha\nTODO one\nbeta\nx TODO and TODO\ngamma\n";
const RIGHT_TEXT: &str = "TODO right\nbeta\ndelta\nlast TODO\n";

/// The shell with one Text Compare tab comparing the two texts above, with the left pane focused.
fn harness() -> (Harness<'static, Shell>, tempfile::TempDir) {
    harness_with(false)
}

/// [`harness`], with *Initialize "Text to find" to be current word* set to `current_word`.
fn harness_with(current_word: bool) -> (Harness<'static, Shell>, tempfile::TempDir) {
    let folder = tempfile::tempdir().unwrap();
    let mut view = TextCompareView::new();
    view.session().expect("a session").specs = [("left.txt", LEFT_TEXT), ("right.txt", RIGHT_TEXT)]
        .map(|(name, text)| {
            let path = folder.path().join(name);
            std::fs::write(&path, text).unwrap();
            path.display().to_string()
        })
        .to_vec();
    let saved = BTreeMap::from([(CURRENT_WORD.to_owned(), Value::Bool(current_word))]);
    let mut shell = Shell::new(vec![SESSION_TYPE]).with_commands(commands()).with_options(options(), saved).unwrap();
    shell.open(Box::new(view));
    let mut harness = Harness::builder().with_size(vec2(1000.0, 600.0)).build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), shell);
    harness.run();
    run(&mut harness, "text-compare.go_to_line", &[("line", Value::Number(1.0))]).expect("the left pane takes focus");
    (harness, folder)
}

fn run(harness: &mut Harness<'_, Shell>, id: &str, args: &[(&str, Value)]) -> command::Outcome {
    let result = command::run(&commands(), harness.state_mut(), id, args, false);
    harness.run();
    harness.run();
    result
}

fn view<'a>(harness: &'a Harness<'_, Shell>) -> &'a TextCompareView {
    harness.state().view::<TextCompareView>().expect("a Text Compare tab")
}

/// The lines of `text` that contain `needle`, by index.
fn holding(text: &str, needle: &str) -> Vec<usize> {
    text.lines().enumerate().filter(|(_, line)| line.contains(needle)).map(|(n, _)| n).collect()
}

fn key(harness: &mut Harness<'_, Shell>, modifiers: Modifiers, key: Key) {
    harness.key_press_modifiers(modifiers, key);
    harness.run();
    harness.run();
}

fn type_text(harness: &mut Harness<'_, Shell>, text: &str) {
    harness.event(Event::Text(text.to_owned()));
    harness.run();
}

fn focused(harness: &Harness<'_, Shell>) -> Option<(Role, String)> {
    let node = harness.query_by(|node| node.is_focused())?;
    let node = node.accesskit_node();
    Some((node.role(), node.label().unwrap_or_default()))
}

fn panel(harness: &Harness<'_, Shell>) -> bool {
    harness.query_by(|node| node.role() == Role::Search && node.label().as_deref() == Some(FIND)).is_some()
}

/// Opens the panel by its shortcut, from the focused pane, and checks that its box has focus.
fn open(harness: &mut Harness<'_, Shell>) {
    key(harness, Modifiers::COMMAND, Key::F);
    assert!(panel(harness), "a search region named Find");
    assert_eq!(focused(harness), Some((Role::TextInput, FIND_WHAT.to_owned())), "focus is in its Find what box");
}

fn left_selection(harness: &Harness<'_, Shell>) -> (usize, String) {
    let pane = view(harness).pane(0).expect("the left pane");
    (pane.selection.head.line, pane.selected_text())
}

/// Moves focus with Tab until the control named `label` has it.
fn tab_to(harness: &mut Harness<'_, Shell>, label: &str) {
    for _ in 0..40 {
        if focused(harness).is_some_and(|(_, name)| name == label) {
            return;
        }
        key(harness, Modifiers::NONE, Key::Tab);
    }
    panic!("Tab never reaches {label}");
}

fn find_box(harness: &Harness<'_, Shell>) -> Option<String> {
    harness.get_by(|node| node.role() == Role::TextInput && node.label().as_deref() == Some(FIND_WHAT)).accesskit_node().value()
}

fn click_checkbox(harness: &mut Harness<'_, Shell>, caption: &str) {
    harness.get_by(|node| node.role() == Role::CheckBox && node.label().as_deref() == Some(caption)).click();
    harness.run();
}

#[test]
fn select_all_for_todo_selects_exactly_the_lines_holding_it_on_the_chosen_sides() {
    let (mut harness, _folder) = harness();
    let select_all = |harness: &mut Harness<'_, Shell>, sides: &str| {
        let args = [("text", Value::Text("TODO".into())), ("sides", Value::Choice(sides.into())), ("select_all", Value::Bool(true))];
        run(harness, "text-compare.find_text", &args).expect("Find Text runs without the window");
        [0, 1].map(|side| view(harness).selected_lines(side).to_vec())
    };
    assert_eq!(select_all(&mut harness, "Left"), [holding(LEFT_TEXT, "TODO"), vec![]], "the left side only, each line once");
    assert_eq!(select_all(&mut harness, "Right"), [vec![], holding(RIGHT_TEXT, "TODO")]);
    assert_eq!(select_all(&mut harness, "Both"), [holding(LEFT_TEXT, "TODO"), holding(RIGHT_TEXT, "TODO")]);

    // The tree marks exactly those lines selected.
    let mut selected: Vec<String> = harness
        .query_all_by(|node| node.role() == Role::Label && node.is_selected() == Some(true))
        .map(|node| node.accesskit_node().value().unwrap_or_default())
        .collect();
    selected.sort();
    let mut expected: Vec<String> =
        [LEFT_TEXT, RIGHT_TEXT].iter().flat_map(|text| text.lines()).filter(|line| line.contains("TODO")).map(str::to_owned).collect();
    expected.sort();
    assert_eq!(selected, expected);
}

#[test]
fn lines_select_all_chose_go_once_their_pane_s_caret_moves() {
    let (mut harness, _folder) = harness();
    let args = [("text", Value::Text("TODO".into())), ("select_all", Value::Bool(true))];
    let lines = holding(LEFT_TEXT, "TODO").len() + holding(RIGHT_TEXT, "TODO").len();
    let said = run(&mut harness, "text-compare.find_text", &args);
    assert!(matches!(&said, Ok(Some(Value::Text(said))) if said.contains(&lines.to_string())), "it says how many lines it selected: {said:?}");
    key(&mut harness, Modifiers::NONE, Key::ArrowDown);
    assert!(view(&harness).selected_lines(0).is_empty(), "the moved pane's lines go");
    assert_eq!(view(&harness).selected_lines(1), holding(RIGHT_TEXT, "TODO"), "the other pane's stay");
}

#[test]
fn enter_in_the_find_box_selects_each_next_match_keeping_focus_there_and_esc_gives_focus_back() {
    let (mut harness, _folder) = harness();
    open(&mut harness);
    type_text(&mut harness, "TODO");
    let matches = holding(LEFT_TEXT, "TODO");
    key(&mut harness, Modifiers::NONE, Key::Enter);
    assert_eq!(left_selection(&harness), (matches[0], "TODO".to_owned()), "the first match is selected");
    assert_eq!(focused(&harness), Some((Role::TextInput, FIND_WHAT.to_owned())), "focus stays in the box");
    key(&mut harness, Modifiers::NONE, Key::Enter);
    assert!(left_selection(&harness).0 > matches[0], "Enter again finds the next one, further down");

    key(&mut harness, Modifiers::NONE, Key::Escape);
    assert!(!panel(&harness), "Esc hides the panel");
    assert_eq!(focused(&harness), Some((Role::MultilineTextInput, LEFT.to_owned())), "focus is back in the pane");
}

#[test]
fn close_automatically_hides_the_panel_once_a_search_starts_and_the_pane_takes_focus() {
    let (mut harness, _folder) = harness();
    open(&mut harness);
    click_checkbox(&mut harness, CLOSE_AUTOMATICALLY);
    harness.get_by(|node| node.role() == Role::TextInput && node.label().as_deref() == Some(FIND_WHAT)).focus();
    harness.run();
    type_text(&mut harness, "TODO");
    key(&mut harness, Modifiers::NONE, Key::Enter);
    assert!(!panel(&harness), "the panel hides");
    assert_eq!(left_selection(&harness).1, "TODO");
    assert_eq!(focused(&harness), Some((Role::MultilineTextInput, LEFT.to_owned())), "the pane with the match has focus");

    // A search that finds nothing hides it too, and says so while focus is in the pane.
    open(&mut harness);
    type_text(&mut harness, "nowhere");
    key(&mut harness, Modifiers::NONE, Key::Enter);
    assert!(!panel(&harness));
    let prefix = NOT_FOUND.split('{').next().unwrap_or_default();
    let said = harness.get_by(|node| node.role() == Role::Label && node.value().is_some_and(|text| text.starts_with(prefix)));
    assert_eq!(said.accesskit_node().live(), Live::Polite, "the miss is announced");
    let status = |harness: &Harness<'_, Shell>, prefix: &str| view(harness).status().fields.iter().any(|field| field.value.starts_with(prefix));
    assert!(status(&harness, prefix), "and shown in the status bar");

    // Select All hides it too, and the status bar says that only the highlight acts on the lines.
    open(&mut harness);
    // The box still holds both earlier searches' text.
    for _ in 0.."TODOnowhere".len() {
        key(&mut harness, Modifiers::NONE, Key::Backspace);
    }
    type_text(&mut harness, "TODO");
    tab_to(&mut harness, SELECT_ALL);
    key(&mut harness, Modifiers::NONE, Key::Space);
    assert!(!panel(&harness));
    assert!(!view(&harness).selected_lines(0).is_empty());
    assert!(status(&harness, LINES_SELECTED.split('{').next().unwrap_or_default()), "the status bar says what Select All did");
}

#[test]
fn esc_from_any_control_of_the_panel_hides_it_and_gives_focus_back() {
    let (mut harness, _folder) = harness();
    // From the menu, as a pointer user opens it.
    assert!(harness.state_mut().run("text-compare.find"), "Find runs from its menu");
    harness.run();
    harness.run();
    click_checkbox(&mut harness, REGULAR_EXPRESSION);
    harness.get_by_label(ELEMENTS_LABEL).focus();
    harness.run();
    assert_eq!(focused(&harness).map(|(_, name)| name).as_deref(), Some(ELEMENTS_LABEL));
    key(&mut harness, Modifiers::NONE, Key::Escape);
    assert!(!panel(&harness), "Esc hides the panel");
    assert_eq!(focused(&harness), Some((Role::MultilineTextInput, LEFT.to_owned())), "focus is back in the pane");
}

#[test]
fn the_find_box_keeps_earlier_searches_newest_first_and_picking_one_fills_it() {
    let (mut harness, _folder) = harness();
    open(&mut harness);
    for text in ["TODO", "beta"] {
        type_text(&mut harness, text);
        key(&mut harness, Modifiers::NONE, Key::Enter);
        for _ in 0..text.len() {
            key(&mut harness, Modifiers::NONE, Key::Backspace);
        }
    }
    harness.get_by_label(SEARCH_HISTORY).click();
    harness.run();
    let history: Vec<String> = harness
        .query_all_by(|node| node.role() == Role::Button && matches!(node.label().as_deref(), Some("TODO" | "beta")))
        .map(|node| node.accesskit_node().label().unwrap_or_default())
        .collect();
    assert_eq!(history, ["beta", "TODO"], "both searches, the newest first");
    harness.get_by(|node| node.role() == Role::Button && node.label().as_deref() == Some("TODO")).click();
    harness.run();
    let field = harness.get_by(|node| node.role() == Role::TextInput && node.label().as_deref() == Some(FIND_WHAT));
    assert_eq!(field.accesskit_node().value().as_deref(), Some("TODO"), "the chosen search is back in the box");
}

#[test]
fn with_regular_expression_on_an_element_goes_in_at_the_cursor_and_searches_as_a_pattern() {
    let (mut harness, _folder) = harness();
    open(&mut harness);
    assert!(harness.query_by_label(ELEMENTS_LABEL).is_none(), "no element picker for plain text");
    click_checkbox(&mut harness, REGULAR_EXPRESSION);
    harness.get_by(|node| node.role() == Role::TextInput && node.label().as_deref() == Some(FIND_WHAT)).focus();
    harness.run();
    type_text(&mut harness, "TODO ");
    harness.get_by_label(ELEMENTS_LABEL).click();
    harness.run();
    harness.get_by(|node| node.role() == Role::Button && node.label().is_some_and(|label| label.starts_with("Word character"))).click();
    harness.run();
    harness.run();
    key(&mut harness, Modifiers::NONE, Key::Enter);
    assert_eq!(left_selection(&harness).1, "TODO o", r"`TODO \w` found as a pattern");
}

#[test]
fn an_invalid_pattern_is_announced_and_searches_nothing() {
    let (mut harness, _folder) = harness();
    open(&mut harness);
    click_checkbox(&mut harness, REGULAR_EXPRESSION);
    harness.get_by(|node| node.role() == Role::TextInput && node.label().as_deref() == Some(FIND_WHAT)).focus();
    harness.run();
    let before = left_selection(&harness);
    type_text(&mut harness, "TODO(");
    key(&mut harness, Modifiers::NONE, Key::Enter);
    let prefix = INVALID_PATTERN.split('{').next().unwrap_or_default();
    let error = harness.get_by(|node| node.role() == Role::Label && node.value().is_some_and(|text| text.starts_with(prefix)));
    assert_eq!(error.accesskit_node().live(), Live::Polite, "the error is announced");
    assert_eq!(left_selection(&harness), before, "nothing is selected");

    let args = [("text", Value::Text("TODO(".into())), ("regex", Value::Bool(true))];
    assert!(run(&mut harness, "text-compare.find_text", &args).is_err(), "Find Text fails on it too");
}

#[test]
fn next_select_all_and_close_work_by_keyboard() {
    let (mut harness, _folder) = harness();
    open(&mut harness);
    type_text(&mut harness, "TODO");
    tab_to(&mut harness, NEXT);
    key(&mut harness, Modifiers::NONE, Key::Space);
    assert_eq!(left_selection(&harness), (holding(LEFT_TEXT, "TODO")[0], "TODO".to_owned()), "Next selects the first match");
    assert_eq!(focused(&harness).map(|(_, name)| name).as_deref(), Some(NEXT), "focus stays on Next");

    tab_to(&mut harness, SELECT_ALL);
    key(&mut harness, Modifiers::NONE, Key::Enter);
    assert_eq!(view(&harness).selected_lines(1), holding(RIGHT_TEXT, "TODO"), "Select All selects the matching lines");
    let prefix = LINES_SELECTED.split('{').next().unwrap_or_default();
    let said = harness.get_by(|node| node.role() == Role::Label && node.value().is_some_and(|text| text.starts_with(prefix)));
    assert_eq!(said.accesskit_node().live(), Live::Polite, "what it selected, and that only the highlight acts on it, is announced");

    tab_to(&mut harness, CLOSE);
    key(&mut harness, Modifiers::NONE, Key::Space);
    assert!(!panel(&harness), "Close hides the panel");
    assert_eq!(focused(&harness), Some((Role::MultilineTextInput, LEFT.to_owned())), "focus is back in the pane");
}

#[test]
fn esc_in_the_search_history_list_closes_only_the_list_and_focus_goes_back_to_its_button() {
    let (mut harness, _folder) = harness();
    open(&mut harness);
    type_text(&mut harness, "TODO");
    key(&mut harness, Modifiers::NONE, Key::Enter);
    tab_to(&mut harness, SEARCH_HISTORY);
    key(&mut harness, Modifiers::NONE, Key::Space);
    let earlier =
        |harness: &Harness<'_, Shell>| harness.query_by(|node| node.role() == Role::Button && node.label().as_deref() == Some("TODO")).is_some();
    assert!(earlier(&harness), "the list opens by keyboard");

    key(&mut harness, Modifiers::NONE, Key::Escape);
    assert!(!earlier(&harness), "Esc closes the list");
    assert!(panel(&harness), "but not the panel");
    assert_eq!(focused(&harness).map(|(_, name)| name).as_deref(), Some(SEARCH_HISTORY), "focus is back on the list's button");

    key(&mut harness, Modifiers::NONE, Key::Escape);
    assert!(!panel(&harness), "Esc again hides the panel");
    assert_eq!(focused(&harness), Some((Role::MultilineTextInput, LEFT.to_owned())));
}

#[test]
fn choosing_an_earlier_search_by_keyboard_closes_the_list_and_puts_it_in_the_box() {
    let (mut harness, _folder) = harness();
    open(&mut harness);
    for text in ["TODO", "beta"] {
        type_text(&mut harness, text);
        key(&mut harness, Modifiers::NONE, Key::Enter);
        for _ in 0..text.len() {
            key(&mut harness, Modifiers::NONE, Key::Backspace);
        }
    }
    tab_to(&mut harness, SEARCH_HISTORY);
    key(&mut harness, Modifiers::NONE, Key::Space);
    tab_to(&mut harness, "TODO");
    key(&mut harness, Modifiers::NONE, Key::Enter);
    let earlier = harness.query_all_by(|node| node.role() == Role::Button && matches!(node.label().as_deref(), Some("TODO" | "beta")));
    assert_eq!(earlier.count(), 0, "the list closes");
    assert_eq!(find_box(&harness).as_deref(), Some("TODO"), "the chosen search is in the box");
    assert_eq!(focused(&harness), Some((Role::TextInput, FIND_WHAT.to_owned())), "and the box has focus");

    key(&mut harness, Modifiers::NONE, Key::Escape);
    assert!(!panel(&harness), "one Esc hides the panel");
}

#[test]
fn find_opens_with_the_word_at_the_caret_and_find_next_repeats_the_panel_s_search() {
    let (mut harness, _folder) = harness_with(true);
    let line = holding(LEFT_TEXT, "TODO")[0];
    run(&mut harness, "text-compare.go_to_line", &[("line", Value::Number(line as f64 + 1.0))]).expect("the caret goes to the TODO line");
    open(&mut harness);
    assert_eq!(find_box(&harness).as_deref(), Some("TODO"), "the box holds the word at the caret");

    key(&mut harness, Modifiers::NONE, Key::Enter);
    key(&mut harness, Modifiers::NONE, Key::Escape);
    let first = left_selection(&harness).0;
    run(&mut harness, "text-compare.find_next", &[]).expect("Find Next has the panel's search to repeat");
    assert!(left_selection(&harness).0 > first, "it finds the next TODO, further down");
}

#[test]
fn typing_after_a_search_is_an_undo_step_of_its_own() {
    let (mut harness, _folder) = harness();
    let line = holding(LEFT_TEXT, "TODO")[0];
    run(&mut harness, "text-compare.go_to_line", &[("line", Value::Number(line as f64 + 1.0))]).expect("the caret goes to the TODO line");
    type_text(&mut harness, "z");
    open(&mut harness);
    type_text(&mut harness, "one");
    key(&mut harness, Modifiers::NONE, Key::Enter);
    key(&mut harness, Modifiers::NONE, Key::Escape);
    type_text(&mut harness, "q");
    let text = |harness: &Harness<'_, Shell>| view(harness).pane(0).expect("the left pane").lines[line].text.clone();
    assert_eq!(text(&harness), "zTODO q", "the match was typed over");
    run(&mut harness, "text-compare.undo", &[]).expect("Undo runs");
    assert_eq!(text(&harness), "zTODO one", "one Undo takes back only what was typed after the search");
}
