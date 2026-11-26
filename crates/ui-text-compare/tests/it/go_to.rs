//! Go To (E07-S6-T1): the dialog driven by keyboard, and the hidden Go To Line run without the
//! window, read back through the view and the accessibility tree.

use egui::accesskit::Role;
use egui::{Event, Key, Modifiers, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_core::command;
use otd_core::session::Value;
use otd_text_compare::Pos;
use otd_ui::{Shell, View as _};
use otd_ui_text_compare::{GO_TO, LEFT, LINE, NOT_A_NUMBER, SESSION_TYPE, TextCompareView, commands};

/// The shell with one Text Compare tab comparing two 300-line files that differ only at the end,
/// so line N of each sits on one row, and the folder holding them.
fn harness() -> (Harness<'static, Shell>, tempfile::TempDir) {
    let folder = tempfile::tempdir().unwrap();
    let body: String = (1..=300).map(|n| format!("line {n}\n")).collect();
    let mut view = TextCompareView::new();
    view.session().expect("a session").specs = [("left.txt", "left"), ("right.txt", "right")]
        .map(|(name, end)| {
            let path = folder.path().join(name);
            std::fs::write(&path, format!("{body}{end}\n")).unwrap();
            path.display().to_string()
        })
        .to_vec();
    let mut shell = Shell::new(vec![SESSION_TYPE]).with_commands(commands());
    shell.open(Box::new(view));
    let mut harness = Harness::builder().with_size(vec2(900.0, 600.0)).build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), shell);
    harness.run();
    (harness, folder)
}

fn caret(harness: &Harness<'_, Shell>) -> Pos {
    harness.state().view::<TextCompareView>().and_then(|view| view.pane(0)).expect("the left pane").selection.head
}

fn go_to_line(harness: &mut Harness<'_, Shell>, line: f64, column: f64) {
    let args = [("line", Value::Number(line)), ("column", Value::Number(column))];
    let result = command::run(&commands(), harness.state_mut(), "text-compare.go_to_line", &args, false);
    assert_eq!(result, Ok(None), "Go To Line runs without the window");
    harness.run();
    harness.run();
}

fn dialog(harness: &Harness<'_, Shell>) -> bool {
    harness.query_by(|node| node.role() == Role::Dialog && node.label().as_deref() == Some(GO_TO)).is_some()
}

fn focused(harness: &Harness<'_, Shell>) -> Option<(Role, String)> {
    let node = harness.query_by(|node| node.is_focused())?;
    let node = node.accesskit_node();
    Some((node.role(), node.label().unwrap_or_default()))
}

/// The vertical centre of each pane's line node named `label`, for the panes that draw it.
fn rows(harness: &Harness<'_, Shell>, label: &str) -> Vec<f32> {
    harness.query_all_by_label(label).filter(|node| node.accesskit_node().role() == Role::Label).map(|node| node.rect().center().y).collect()
}

#[test]
fn go_to_120_5_puts_the_caret_there_in_the_active_pane_and_both_panes_show_its_line() {
    let (mut harness, _folder) = harness();
    assert!(rows(&harness, "line 120").is_empty(), "line 120 starts out of view");

    go_to_line(&mut harness, 120.0, 5.0);
    assert_eq!(caret(&harness), Pos { line: 119, column: 4 }, "line 120, column 5, counted from 1");
    let rows = rows(&harness, "line 120");
    assert_eq!(rows.len(), 2, "both panes show line 120");
    assert_eq!(rows[0], rows[1], "on one row");
    assert_eq!(focused(&harness), Some((Role::MultilineTextInput, LEFT.to_owned())), "the pane has focus, with its caret there");
}

#[test]
fn a_line_or_column_out_of_range_goes_to_the_nearest_valid_position() {
    let (mut harness, _folder) = harness();
    go_to_line(&mut harness, 3.0, 999.0);
    assert_eq!(caret(&harness), Pos { line: 2, column: "line 3".len() }, "past a line's end: its end");
    go_to_line(&mut harness, -4.0, 0.0);
    assert_eq!(caret(&harness), Pos::default(), "before the start: the start");
    go_to_line(&mut harness, 1e12, 1.0);
    let end = caret(&harness);
    assert!(end.line >= 300 && rows(&harness, "line 300").len() == 2, "past the last line: the last, shown in both panes");
}

#[test]
fn the_dialog_takes_a_line_and_column_by_keyboard_refuses_text_and_esc_gives_focus_back() {
    let (mut harness, _folder) = harness();
    let open = |harness: &mut Harness<'_, Shell>| {
        assert!(harness.state_mut().run("text-compare.go_to"), "Go To runs from its menu");
        harness.run();
        harness.run();
        assert!(dialog(harness), "a dialog named Go To");
        assert_eq!(focused(harness), Some((Role::TextInput, LINE.to_owned())), "focus is in its Line field");
    };
    let key = |harness: &mut Harness<'_, Shell>, key: Key| {
        harness.key_press_modifiers(Modifiers::NONE, key);
        harness.run();
        harness.run();
    };

    // Esc closes it and nothing moves.
    let before = focused(&harness);
    open(&mut harness);
    key(&mut harness, Key::Escape);
    assert!(!dialog(&harness), "Esc closes it");
    assert_eq!((focused(&harness), caret(&harness)), (before, Pos::default()), "focus goes back and the caret stays");

    // Text is refused, and the dialog says why.
    open(&mut harness);
    harness.event(Event::Text("ten".to_owned()));
    key(&mut harness, Key::Enter);
    assert!(harness.query_by_label(NOT_A_NUMBER).is_some(), "it says the line must be a number");
    assert_eq!(caret(&harness), Pos::default());
    assert_eq!(focused(&harness), Some((Role::TextInput, LINE.to_owned())), "focus is back in the Line field");

    // Corrected in place: a line, Tab, a column, Enter.
    for _ in 0.."ten".len() {
        key(&mut harness, Key::Backspace);
    }
    harness.event(Event::Text("120".to_owned()));
    key(&mut harness, Key::Tab);
    harness.event(Event::Text("5".to_owned()));
    key(&mut harness, Key::Enter);
    assert!(!dialog(&harness), "Enter goes there and closes it");
    assert_eq!(caret(&harness), Pos { line: 119, column: 4 });
    assert_eq!(focused(&harness), Some((Role::MultilineTextInput, LEFT.to_owned())), "the pane has focus");
}
