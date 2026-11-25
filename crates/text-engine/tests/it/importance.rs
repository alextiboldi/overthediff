//! Importance classification: rules, orphan lines, the overall result and statistics.

use std::ops::Range;

use otd_text_engine::{Classification, Importance, Line, LineEnding, Outcome, RowKind, Rule, classify, compare};

/// A test rule: every ASCII digit is unimportant.
struct Digits;

impl Rule for Digits {
    fn unimportant(&self, text: &str) -> Vec<Range<usize>> {
        text.char_indices().filter(|(_, c)| c.is_ascii_digit()).map(|(k, c)| k..k + c.len_utf8()).collect()
    }
}

fn side(texts: &[&str]) -> Vec<Line> {
    texts.iter().map(|t| Line { text: (*t).to_owned(), ending: LineEnding::Lf }).collect()
}

fn run(left: &[Line], right: &[Line], rules: &[&dyn Rule]) -> Classification {
    classify(&compare(left, right), left, right, rules)
}

#[test]
fn a_change_only_in_text_a_rule_marks_is_an_unimportant_difference() {
    let (left, right) = (side(&["head", "total 120 items", "tail"]), side(&["head", "total 345 items", "tail"]));
    let c = run(&left, &right, &[&Digits]);
    assert_eq!(c.outcome, Outcome::UnimportantOnly);
    assert!(c.rows.contains(&Some(Importance::Unimportant)));
}

#[test]
fn an_orphan_line_is_important_even_when_a_rule_marks_all_its_text() {
    let (short, long) = (side(&["head", "tail"]), side(&["head", "12345", "tail"]));
    for (left, right) in [(&short, &long), (&long, &short)] {
        let c = run(left, right, &[&Digits]);
        assert_eq!(c.outcome, Outcome::Important);
        assert!(!c.rows.contains(&Some(Importance::Unimportant)));
    }
}

#[test]
fn with_no_rules_every_difference_is_important() {
    let (left, right) = (side(&["head", "total 120 items", "tail"]), side(&["head", "total 345 items", "tail"]));
    let c = run(&left, &right, &[]);
    assert_eq!(c.outcome, Outcome::Important);
    assert_eq!(c.statistics.unimportant, 0);
}

#[test]
fn a_line_holding_an_unimportant_and_an_important_change_is_important() {
    let c = run(&side(&["a1"]), &side(&["b2"]), &[&Digits]);
    assert_eq!(c.rows, [Some(Importance::Important)]);
}

#[test]
fn a_different_line_ending_stays_important_whatever_the_text_rules_mark() {
    let left = side(&["head", "value 1", "tail"]);
    let mut right = side(&["head", "value 2", "tail"]);
    if let Some(line) = right.get_mut(1) {
        line.ending = LineEnding::CrLf;
    }
    assert_eq!(run(&left, &right, &[&Digits]).outcome, Outcome::Important);
}

#[test]
fn the_statistics_partition_the_rows_and_count_orphans_per_side() {
    let left = side(&["a", "b 1", "c", "d", "gone", "e", "f x1", "g"]);
    let right = side(&["a", "b 2", "c", "d", "e", "new", "also new", "f y2", "g"]);
    let comparison = compare(&left, &right);
    let c = classify(&comparison, &left, &right, &[&Digits]);
    let s = c.statistics;
    let kinds = |kind| comparison.rows.iter().filter(|r| r.kind == kind).count();
    assert_eq!(s.important + s.unimportant + s.same, comparison.rows.len());
    assert_eq!(s.same, kinds(RowKind::Same));
    assert_eq!((s.left_orphans, s.right_orphans), (kinds(RowKind::LeftOnly), kinds(RowKind::RightOnly)));
    assert_eq!(s.sections, comparison.sections.len());
    assert_eq!(s.important, c.rows.iter().filter(|i| **i == Some(Importance::Important)).count());
    assert!(s.unimportant > 0 && s.important > s.left_orphans + s.right_orphans);
}

/// A rule returning ranges that are reversed, out of bounds, overlapping or inside a character.
struct Hostile;

impl Rule for Hostile {
    fn unimportant(&self, text: &str) -> Vec<Range<usize>> {
        vec![Range { start: 5, end: 2 }, 0..usize::MAX, usize::MAX..usize::MAX, 1..2, 0..1, text.len()..text.len() + 9]
    }
}

#[test]
fn a_rule_returning_ranges_outside_or_inside_characters_never_panics() {
    let (left, right) = (side(&["é1", "", "ü"]), side(&["é2", "x", "ö"]));
    let c = run(&left, &right, &[&Hostile, &Digits]);
    assert_eq!(c.rows.len(), compare(&left, &right).rows.len());
}
