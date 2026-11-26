//! Finding the next or previous section and change from the cursor (E07-S2-T1).

use std::iter::successors;

use otd_text_compare::{Change, Cursor, Differences, Line, LineEnding};
use otd_text_engine::{Row, RowKind, Side, compare};

fn lines(texts: &[(&str, LineEnding)]) -> Vec<Line> {
    texts.iter().map(|&(text, ending)| Line { text: text.to_owned(), ending }).collect()
}

fn lf(texts: &[&str]) -> Vec<Line> {
    lines(&texts.iter().map(|&t| (t, LineEnding::Lf)).collect::<Vec<_>>())
}

fn differs(rows: &[Row]) -> impl Fn(usize) -> bool + '_ {
    |k| rows[k].kind != RowKind::Same
}

/// Every change *Next Difference* visits from `from`, landing each time at the change's start.
fn forwards<F: Fn(usize) -> bool>(d: &Differences<'_, F>, from: Cursor) -> Vec<Change> {
    successors(d.next(from), |c| d.next(Cursor { row: c.row, side: from.side, column: c.on(from.side).start })).collect()
}

fn backwards<F: Fn(usize) -> bool>(d: &Differences<'_, F>, from: Cursor) -> Vec<Change> {
    successors(d.previous(from), |c| d.previous(Cursor { row: c.row, side: from.side, column: c.on(from.side).start })).collect()
}

/// The text a change covers on each side.
fn covered(change: &Change, left: &[Line], right: &[Line], rows: &[Row]) -> (String, String) {
    let text = |lines: &[Line], line: Option<usize>, columns: &std::ops::Range<usize>| {
        line.map(|i| lines[i].text.chars().skip(columns.start).take(columns.len()).collect()).unwrap_or_default()
    };
    let row = rows[change.row];
    (text(left, row.left, &change.left), text(right, row.right, &change.right))
}

#[test]
fn sections_are_found_from_a_cursor_before_inside_and_after_them() {
    let (left, right) = (lf(&["a", "b", "c", "d", "e", "f", "g", "h"]), lf(&["a", "X", "c", "d", "Y", "Z", "g", "h"]));
    let rows = compare(&left, &right).rows;
    let d = Differences { rows: &rows, left: &left, right: &right, differs: differs(&rows) };
    // Before the first section, and between the two.
    assert_eq!((d.previous_section(0), d.next_section(0)), (None, Some(1..2)));
    assert_eq!((d.previous_section(2), d.next_section(2)), (Some(1..2), Some(4..6)));
    // Inside a section, on its first or a later row, both directions leave it.
    for row in [4, 5] {
        assert_eq!((d.previous_section(row), d.next_section(row)), (Some(1..2), None));
    }
    // After the last section, and after the final line break.
    for row in [7, rows.len()] {
        assert_eq!((d.previous_section(row), d.next_section(row)), (Some(4..6), None));
    }
}

#[test]
fn a_row_the_view_treats_as_same_is_skipped() {
    let (left, right) = (lf(&["a", "b", "c", "d", "e", "f", "g"]), lf(&["a", "X", "c", "d", "Y", "Z", "g"]));
    let rows = compare(&left, &right).rows;
    let d = Differences { rows: &rows, left: &left, right: &right, differs: |k: usize| rows[k].kind != RowKind::Same && k != 1 && k != 4 };
    assert_eq!((d.previous_section(3), d.next_section(0)), (None, Some(5..6)));
    let at = Cursor { row: 0, side: Side::Left, column: 0 };
    assert_eq!(forwards(&d, at).iter().map(|c| c.row).collect::<Vec<_>>(), [5]);
}

#[test]
fn a_line_with_three_changes_is_visited_change_by_change_in_order_from_either_pane() {
    // A replacement, an insertion and a deletion, after a two-byte character, so columns that
    // counted bytes would cover the wrong text.
    let left = lf(&["same", "é a = 1; let bb = 2; let cc = 3;", "same", "p"]);
    let right = lf(&["same", "é a = 9; let bxb = 2; let c = 3;", "same", "q"]);
    let rows = compare(&left, &right).rows;
    let d = Differences { rows: &rows, left: &left, right: &right, differs: differs(&rows) };
    let expected = [("1", "9"), ("", "x"), ("c", "")].map(|(l, r)| (l.to_owned(), r.to_owned()));
    for side in [Side::Left, Side::Right] {
        let ahead = forwards(&d, Cursor { row: 1, side, column: 0 });
        let in_line: Vec<_> = ahead.iter().take_while(|c| c.row == 1).map(|c| covered(c, &left, &right, &rows)).collect();
        assert_eq!(in_line, expected, "from the {side:?} pane");
        // After the line's last change, the next is in the following differing row.
        assert_eq!(ahead.get(3).map(|c| c.row), Some(3));

        let end = left[1].text.chars().count().max(right[1].text.chars().count());
        let back: Vec<_> = backwards(&d, Cursor { row: 1, side, column: end }).iter().map(|c| covered(c, &left, &right, &rows)).collect();
        assert_eq!(back, expected.iter().rev().cloned().collect::<Vec<_>>(), "back from the {side:?} pane");
    }
}

#[test]
fn an_ending_only_change_and_an_orphan_line_are_one_stop_each() {
    let left = lines(&[("a", LineEnding::Lf), ("b", LineEnding::CrLf), ("c", LineEnding::Lf)]);
    let right = lf(&["a", "b", "c", "d"]);
    let rows = compare(&left, &right).rows;
    let d = Differences { rows: &rows, left: &left, right: &right, differs: differs(&rows) };
    let at = Cursor { row: 0, side: Side::Right, column: 0 };
    assert_eq!(forwards(&d, at).iter().map(|c| c.row).collect::<Vec<_>>(), [1, 3]);
    let after = Cursor { row: rows.len(), side: Side::Right, column: 0 };
    assert_eq!(backwards(&d, after).iter().map(|c| c.row).collect::<Vec<_>>(), [3, 1]);
}
