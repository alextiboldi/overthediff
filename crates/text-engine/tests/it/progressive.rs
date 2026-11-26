//! Progressive results and cancellation: rows handed out while the comparison works, and a stop
//! that the engine obeys at once. Work is counted in polls of the caller's step, never timed.

use std::ops::ControlFlow;

use otd_text_engine::{Line, LineEnding, Options, Row, compare, compare_progressive};

fn lines(texts: impl IntoIterator<Item = String>) -> Vec<Line> {
    texts.into_iter().map(|text| Line { text, ending: LineEnding::Lf }).collect()
}

/// `n` numbered lines, and a copy with a line changed, one deleted and one inserted every 50 lines,
/// and one more line at its end.
fn scattered(n: usize) -> (Vec<Line>, Vec<Line>) {
    let left = lines((0..n).map(|k| format!("line {k}")));
    let mut right = lines((0..n).filter(|k| k % 50 != 20).flat_map(|k| match k % 50 {
        10 => vec![format!("changed {k}")],
        30 => vec![format!("line {k}"), format!("inserted {k}")],
        _ => vec![format!("line {k}")],
    }));
    right.extend(lines(["appended".to_owned()]));
    (left, right)
}

/// Every batch the step is handed, in order, comparing to the end.
fn batches(left: &[Line], right: &[Line]) -> Vec<Vec<Row>> {
    let mut out = Vec::new();
    let done = compare_progressive(left, right, Options::default(), |batch| {
        out.push(batch.to_vec());
        ControlFlow::<()>::Continue(())
    });
    assert!(done.is_continue());
    out
}

#[test]
fn the_batches_together_are_exactly_the_rows_of_a_full_comparison() {
    let (left, right) = scattered(2_000);
    let handed_out: Vec<Row> = batches(&left, &right).concat();
    assert_eq!(handed_out, compare(&left, &right).rows);
}

#[test]
fn equal_inputs_hand_out_their_first_rows_before_the_last() {
    let (left, _) = scattered(2_000);
    let batches = batches(&left, &left);
    let first = batches.iter().position(|b| !b.is_empty());
    let last = batches.iter().rposition(|b| !b.is_empty());
    assert!(first < last, "every row came in one batch");
}

#[test]
fn a_search_through_inputs_with_nothing_in_common_polls_for_every_line_before_any_row() {
    let left = lines((0..2_000).map(|k| format!("left {k}")));
    let right = lines((0..2_000).map(|k| format!("right {k}")));
    let polls_before_rows = batches(&left, &right).iter().take_while(|b| b.is_empty()).count();
    assert!(polls_before_rows >= left.len(), "{polls_before_rows} polls for {} lines", left.len());
}

#[test]
fn a_break_stops_the_comparison_without_another_poll_and_returns_the_break() {
    let (left, right) = scattered(2_000);
    let total = batches(&left, &right).len();
    let stop_at = total / 2;
    let mut polls = 0;
    let result = compare_progressive(&left, &right, Options::default(), |_| {
        polls += 1;
        if polls == stop_at { ControlFlow::Break("stop") } else { ControlFlow::Continue(()) }
    });
    assert_eq!(result, ControlFlow::Break("stop"));
    assert_eq!(polls, stop_at);
}

#[test]
fn twice_the_lines_take_about_twice_the_polls() {
    // The step counter behind E03-S1-T6's budgets: work that grows faster than the input, such as
    // a search rescanning the whole text at every change, would miss them on large inputs.
    let polls = |n| {
        let (left, right) = scattered(n);
        batches(&left, &right).len()
    };
    let (small, large) = (polls(4_000), polls(8_000));
    assert!(large <= small * 2 + small / 4, "{small} polls for 4,000 lines, {large} for 8,000");
}
