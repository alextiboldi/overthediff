//! The aligned comparison, read back through the accessibility tree: where each pane's lines and
//! line numbers sit, and how scrolling one pane moves the other.

use egui::accesskit::Role;
use egui::{Event, Modifiers, MouseWheelSource, MouseWheelUnit, Rect, TouchPhase, vec2};
use egui_kittest::kittest::{NodeT as _, Queryable as _, by};
use egui_kittest::{Harness, Node};
use otd_ui::View as _;
use otd_ui_text_compare::TextCompareView;

/// The view comparing `left` and `right`, written to files in `folder`, laid out once.
fn compare<'a>(folder: &tempfile::TempDir, left: &str, right: &str) -> Harness<'a, TextCompareView> {
    compare_named(folder, ["left.txt", "right.txt"], [left, right])
}

fn compare_named<'a>(folder: &tempfile::TempDir, names: [&str; 2], texts: [&str; 2]) -> Harness<'a, TextCompareView> {
    let paths = [(names[0], texts[0]), (names[1], texts[1])].map(|(name, text)| {
        let path = folder.path().join(name);
        std::fs::write(&path, text).unwrap();
        path.display().to_string()
    });
    let mut view = TextCompareView::new();
    if let Some(session) = view.session() {
        session.specs = paths.to_vec();
    }
    let mut harness = Harness::builder().with_size(vec2(900.0, 600.0)).build_ui_state(|ui, view: &mut TextCompareView| view.ui(ui), view);
    harness.run();
    harness
}

/// Which pane a node is in, read from the tree as a screen reader would: the side its body group is
/// named for, 0 for Left and 1 for Right.
fn pane_of(node: &Node<'_>) -> Option<usize> {
    let mut at = node.parent();
    while let Some(node) = at {
        if node.accesskit_node().role() == Role::Group {
            match node.accesskit_node().label().as_deref() {
                Some("Left") => return Some(0),
                Some("Right") => return Some(1),
                _ => {}
            }
        }
        at = node.parent();
    }
    None
}

/// The rect of the node named `label` in pane `pane`, if it is shown.
fn find(harness: &Harness<'_, TextCompareView>, label: &str, pane: usize) -> Option<Rect> {
    harness.query_all_by_label(label).find(|node| pane_of(node) == Some(pane)).map(|node| node.rect())
}

fn row(harness: &Harness<'_, TextCompareView>, label: &str, pane: usize) -> f32 {
    find(harness, label, pane).unwrap_or_else(|| panic!("{label} is shown in pane {pane}")).center().y
}

fn description(node: &Node<'_>) -> Option<String> {
    node.accesskit_node().description()
}

#[test]
fn matching_lines_share_rows_and_gaps_without_numbers_sit_opposite_inserted_and_deleted_lines() {
    let folder = tempfile::tempdir().unwrap();
    let left = "one\ntwo\nthree\ndeleted\nfour\nfive\nsix\nchanged left\nseven\neight\nnine\nten\neleven\n\ntwelve\n";
    let right = "one\ntwo\nthree\nfour\nfive\nsix\nchanged right\nseven\neight\nnine\ninserted\nten\neleven\n\ntwelve\n";
    // A name long enough to wrap the left header's path, so the headers differ in height.
    let long = format!("{}.txt", "a-left-file-name-long-enough-to-wrap-".repeat(4));
    let harness = compare_named(&folder, [&long, "right.txt"], [left, right]);

    for same in ["one", "three", "four", "six", "seven", "nine", "ten", "twelve"] {
        assert_eq!(row(&harness, same, 0), row(&harness, same, 1), "{same} sits on one row in both panes");
    }
    assert_eq!(row(&harness, "changed left", 0), row(&harness, "changed right", 1), "the changed line pairs with its other version");

    // Each orphan sits between its neighbours' rows, with nothing on its row in the other pane: a gap.
    for (orphan, pane, before, after) in [("deleted", 0, "three", "four"), ("inserted", 1, "nine", "ten")] {
        let y = row(&harness, orphan, pane);
        assert!(row(&harness, before, 1 - pane) < y && y < row(&harness, after, 1 - pane), "{orphan} has a row of its own");
        let opposite: Vec<String> = harness
            .query_all(by().predicate(|node| node.label().is_some()))
            .filter(|node| pane_of(node) == Some(1 - pane) && (node.rect().center().y - y).abs() < 1.0)
            .filter_map(|node| node.accesskit_node().label())
            .collect();
        assert!(opposite.is_empty(), "the gap opposite {orphan} has no line number or text: {opposite:?}");
    }

    // Line numbers are each file's own: the gap rows don't count.
    assert_eq!(find(&harness, "4", 0).map(|r| r.center().y), Some(row(&harness, "deleted", 0)));
    assert_eq!(find(&harness, "4", 1).map(|r| r.center().y), Some(row(&harness, "four", 1)));

    // A blank line is named too, and has a size, in each pane.
    for pane in [0, 1] {
        let blank = find(&harness, "Blank line", pane).unwrap_or_else(|| panic!("pane {pane}'s blank line has a node"));
        assert!(blank.width() > 0.0 && blank.center().y > row(&harness, "eleven", pane), "the blank line sits after eleven");
    }

    // How each line differs is said in text, not only in colour.
    let said =
        |label: &str, pane: usize| harness.query_all_by_label(label).find(|node| pane_of(node) == Some(pane)).and_then(|node| description(&node));
    assert_eq!(said("deleted", 0).as_deref(), Some("Deleted"));
    assert_eq!(said("inserted", 1).as_deref(), Some("Inserted"));
    assert_eq!(said("changed left", 0).as_deref(), Some("Changed"));
    assert_eq!(said("one", 0), None);
}

/// Two files of `lines` lines that differ only in their first line, so both have every row.
fn long_pair(lines: usize) -> (String, String) {
    let body: String = (1..lines).map(|n| format!("line {n}\n")).collect();
    (format!("left\n{body}"), format!("right\n{body}"))
}

#[test]
fn scrolling_either_pane_scrolls_both() {
    let folder = tempfile::tempdir().unwrap();
    let (left, right) = long_pair(300);
    let mut harness = compare(&folder, &left, &right);
    // A line well inside both viewports, so it is still shown after one scroll step.
    let line = "line 20";
    for pane in [0, 1] {
        let before = row(&harness, line, pane);
        let node = harness.query_all_by_label(line).find(|node| pane_of(node) == Some(pane)).unwrap();
        node.scroll_down();
        harness.run();
        let (left, right) = (row(&harness, line, 0), row(&harness, line, 1));
        assert!(left < before, "scrolling pane {pane} moves its lines up");
        assert_eq!(left, right, "both panes stay on one row after scrolling pane {pane}");
    }
}

#[test]
fn a_pane_scrolled_sideways_keeps_its_gutter_in_view() {
    let folder = tempfile::tempdir().unwrap();
    let long = format!("deleted {}", "and long ".repeat(40));
    let mut harness = compare(&folder, &format!("one\n{long}\ntwo\n"), "one\ntwo\n");
    let number = |harness: &Harness<'_, TextCompareView>| find(harness, "2", 0).expect("line 2's number is shown");
    let (text_before, number_before) = (find(&harness, &long, 0).unwrap(), number(&harness));

    // A sideways wheel over the line, as a trackpad gives.
    harness.hover_at(text_before.left_center() + vec2(20.0, 0.0));
    harness.event(Event::MouseWheel {
        unit: MouseWheelUnit::Point,
        delta: vec2(-200.0, 0.0),
        phase: TouchPhase::Move,
        source: MouseWheelSource::Unknown,
        modifiers: Modifiers::NONE,
    });
    harness.run();

    assert!(find(&harness, &long, 0).unwrap().left() < text_before.left(), "the line scrolls sideways");
    assert_eq!(number(&harness).left(), number_before.left(), "its number stays at the pane's left edge");
}
