//! Word Wrap, run as the menus and the toolbar run it, read back through the accessibility tree:
//! where each pane's lines, continuation rows and line numbers sit.

use egui::accesskit::Role;
use egui::{Rect, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_ui::{Shell, View as _};
use otd_ui_text_compare::{SESSION_TYPE, TextCompareView, commands};

/// The shell with Text Compare's commands, comparing `left` and `right` written in a temporary
/// folder, and that folder.
fn harness(left: &str, right: &str) -> (Harness<'static, Shell>, tempfile::TempDir) {
    let folder = tempfile::tempdir().unwrap();
    let specs = [("left.txt", left), ("right.txt", right)].map(|(name, text)| {
        let path = folder.path().join(name);
        std::fs::write(&path, text).unwrap();
        path.display().to_string()
    });
    let mut view = TextCompareView::new();
    view.session().expect("a session").specs = specs.to_vec();
    let mut shell = Shell::new(vec![SESSION_TYPE]).with_commands(commands());
    shell.open(Box::new(view));
    let mut harness = Harness::builder().with_size(vec2(900.0, 600.0)).build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), shell);
    harness.run();
    (harness, folder)
}

/// Whether `node` is inside pane `pane`'s body, the group named for its side.
fn in_pane(node: &egui_kittest::Node<'_>, pane: usize) -> bool {
    let name = ["Left", "Right"][pane];
    let mut at = node.parent();
    while let Some(node) = at {
        if node.accesskit_node().role() == Role::Group && node.accesskit_node().label().as_deref() == Some(name) {
            return true;
        }
        at = node.parent();
    }
    false
}

/// The rect of the node named `label` in pane `pane`.
fn find(harness: &Harness<'_, Shell>, label: &str, pane: usize) -> Rect {
    harness.query_all_by_label(label).find(|node| in_pane(node, pane)).unwrap_or_else(|| panic!("{label} is shown in pane {pane}")).rect()
}

/// The tops of the rows that pane `pane`'s text runs sit on, between `top` and `bottom`.
fn run_rows(harness: &Harness<'_, Shell>, pane: usize, rect: Rect) -> Vec<f32> {
    let mut tops: Vec<f32> = harness
        .query_all_by(|node| node.role() == Role::TextRun)
        .filter(|node| in_pane(node, pane) && rect.y_range().contains(node.rect().center().y))
        .map(|node| node.rect().top())
        .collect();
    tops.sort_by(f32::total_cmp);
    tops.dedup();
    tops
}

#[test]
fn a_long_changed_line_wraps_in_both_panes_and_the_next_line_starts_on_one_row_in_both() {
    // A 300-character changed line, and a shorter version on the right, so the panes wrap it to
    // different numbers of rows.
    let left = format!("first\n{}\nafter\n", "left words ".repeat(28).trim_end().chars().take(300).collect::<String>());
    let right = format!("first\n{}\nafter\n", "right words ".repeat(15).trim_end());
    let long = [left.lines().nth(1).unwrap().to_owned(), right.lines().nth(1).unwrap().to_owned()];
    assert_eq!(long[0].chars().count(), 300);
    let (mut harness, _folder) = harness(&left, &right);
    let unwrapped = [0, 1].map(|pane| find(&harness, "after", pane).top());
    let unwrapped_width = [0, 1].map(|pane| find(&harness, &long[pane], pane).width());

    assert!(harness.state_mut().run("text-compare.word_wrap"), "Word Wrap ran");
    harness.run();

    let after = [0, 1].map(|pane| find(&harness, "after", pane).top());
    assert_eq!(after[0], after[1], "the line after the wrapped one starts on the same row in both panes");
    assert!(after[0] > unwrapped[0] && after[1] > unwrapped[1], "below where it sat unwrapped");
    let rows = [0, 1].map(|pane| run_rows(&harness, pane, find(&harness, &long[pane], pane)).len());
    assert!(rows[0] > rows[1] && rows[1] > 1, "both panes wrap the line, the longer to more rows: {rows:?}");
    for pane in [0, 1] {
        let line = find(&harness, &long[pane], pane);
        assert!(line.width() < unwrapped_width[pane], "pane {pane}'s line is narrower than unwrapped");
        let numbers: Vec<Rect> = harness.query_all_by_label("2").filter(|node| in_pane(node, pane)).map(|node| node.rect()).collect();
        assert_eq!(numbers.len(), 1, "pane {pane}'s line number shows once");
        assert!((numbers[0].top() - line.top()).abs() < 1.0, "on the line's first row");
    }

    assert!(harness.state_mut().run("text-compare.word_wrap"), "Word Wrap ran again");
    harness.run();
    assert_eq!([0, 1].map(|pane| find(&harness, "after", pane).top()), unwrapped, "turned off, the rows are one line high again");
}

/// The shell comparing 80 long lines, with "left" and "right" above them, and the top of the
/// panes' first row.
fn long_lines() -> (Harness<'static, Shell>, tempfile::TempDir, f32) {
    // Long lines, so wrapping moves every row.
    let long = "word ".repeat(60).trim_end().to_owned();
    let text: String = (1..=80).map(|n| format!("line {n} {long}\n")).collect();
    let (harness, folder) = harness(&format!("left\n{text}"), &format!("right\n{text}"));
    let top = find(&harness, "left", 0).top();
    (harness, folder, top)
}

/// The line on the panes' top row, at `top`, in each pane.
fn top_line(harness: &Harness<'_, Shell>, top: f32) -> [Option<String>; 2] {
    [0, 1].map(|pane| {
        harness
            .query_all_by(|node| node.role() == Role::Label && node.value().is_some_and(|text| text.starts_with("line ")))
            .find(|node| in_pane(node, pane) && node.rect().y_range().contains(top + 1.0))
            .and_then(|node| node.accesskit_node().value())
    })
}

/// Scrolls the panes down a few steps and returns the line then on their top row.
fn scroll_down(harness: &mut Harness<'_, Shell>, top: f32) -> [Option<String>; 2] {
    for _ in 0..5 {
        let run = harness.query_all_by(|node| node.role() == Role::TextRun).find(|node| in_pane(node, 0));
        run.expect("a line is shown").scroll_down();
        harness.run();
    }
    let before = top_line(harness, top);
    assert!(before[0].as_ref().is_some_and(|line| line.starts_with("line ")) && before[0] == before[1], "scrolled down: {before:?}");
    before
}

#[test]
fn turning_word_wrap_on_keeps_the_line_at_the_top_of_the_panes_there() {
    let (mut harness, _folder, top) = long_lines();
    let before = scroll_down(&mut harness, top);

    assert!(harness.state_mut().run("text-compare.word_wrap"), "Word Wrap ran");
    harness.run();
    harness.run();
    assert_eq!(top_line(&harness, top), before, "the same line is still on the top row of both panes");
}

#[test]
fn recompare_and_swap_with_word_wrap_on_keep_the_line_at_the_top_of_the_panes_there() {
    let (mut harness, _folder, top) = long_lines();
    assert!(harness.state_mut().run("text-compare.word_wrap"), "Word Wrap ran");
    harness.run();
    let before = scroll_down(&mut harness, top);

    for command in ["text-compare.recompare_files", "text-compare.swap_sides"] {
        assert!(harness.state_mut().run(command), "{command} ran");
        harness.run();
        harness.run();
        assert_eq!(top_line(&harness, top), before, "after {command}, the same line is on the top row of both panes");
    }
}
