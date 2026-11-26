//! Importance classification: rules, orphan lines, the overall result and statistics.

use std::ops::Range;

use otd_text_engine::{Classification, Importance, Line, LineEnding, Options, Outcome, RowKind, Rule, Side, char_differences_with, classify, compare, compare_with};

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

/// `texts` as the lines of a file that ends every line, the last included, with `ending`.
fn ended(texts: &[&str], ending: LineEnding) -> Vec<Line> {
    texts.iter().map(|t| Line { text: (*t).to_owned(), ending }).collect()
}

/// Two sides whose middle line differs only in its digits and its line ending.
fn digits_and_ending() -> (Vec<Line>, Vec<Line>) {
    let left = side(&["head", "value 1", "tail"]);
    let mut right = side(&["head", "value 2", "tail"]);
    if let Some(line) = right.get_mut(1) {
        line.ending = LineEnding::CrLf;
    }
    (left, right)
}

#[test]
fn with_line_endings_compared_a_different_ending_stays_important_whatever_the_text_rules_mark() {
    let (left, right) = digits_and_ending();
    let comparison = compare_with(&left, &right, Options { line_endings: true });
    assert_eq!(classify(&comparison, &left, &right, &[&Digits]).outcome, Outcome::Important);
}

#[test]
fn with_line_endings_ignored_a_different_ending_leaves_a_rule_marked_change_unimportant() {
    let (left, right) = digits_and_ending();
    assert_eq!(run(&left, &right, &[&Digits]).outcome, Outcome::UnimportantOnly);
}

#[test]
fn a_crlf_file_and_its_lf_copy_compare_as_same_by_default() {
    let texts = ["first", "", "  third", "last"];
    let (left, right) = (ended(&texts, LineEnding::CrLf), ended(&texts, LineEnding::Lf));
    let comparison = compare(&left, &right);
    assert!(comparison.sections.is_empty());
    assert_eq!(classify(&comparison, &left, &right, &[]).outcome, Outcome::Same);
}

#[test]
fn with_line_endings_compared_every_line_of_a_crlf_file_differs_from_its_lf_copy() {
    let texts = ["first", "", "  third", "last"];
    let (left, right) = (ended(&texts, LineEnding::CrLf), ended(&texts, LineEnding::Lf));
    let comparison = compare_with(&left, &right, Options { line_endings: true });
    assert!(comparison.rows.iter().all(|row| row.kind == RowKind::Changed), "{:?}", comparison.rows);
    let c = classify(&comparison, &left, &right, &[]);
    assert_eq!(c.statistics.important, texts.len());
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

    fn case_insensitive(&self, text: &str) -> Vec<Range<usize>> {
        self.unimportant(text)
    }
}

#[test]
fn a_rule_returning_ranges_outside_or_inside_characters_never_panics() {
    let (left, right) = (side(&["é1", "", "ü"]), side(&["é2", "x", "ö"]));
    let c = run(&left, &right, &[&Hostile, &Digits]);
    assert_eq!(c.rows.len(), compare(&left, &right).rows.len());
    for (l, r) in left.iter().zip(&right) {
        for range in char_differences_with(&l.text, &r.text, &[&Hostile, &Digits]).ranges {
            let text = if range.side == Side::Left { &l.text } else { &r.text };
            assert!(text.get(range.start..range.end).is_some(), "{range:?}");
        }
    }
}
