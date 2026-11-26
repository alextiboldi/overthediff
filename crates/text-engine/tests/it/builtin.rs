//! The built-in importance elements: leading, embedded and trailing whitespace, everything else
//! and its character case, for rows and for character ranges.

use otd_text_engine::{BuiltInRules, CharRange, Importance, Line, LineEnding, Outcome, Rule, Side, char_differences_with, classify, compare};

fn line(text: &str) -> Vec<Line> {
    vec![Line { text: text.to_owned(), ending: LineEnding::Lf }]
}

fn outcome(left: &str, right: &str, rules: BuiltInRules) -> Outcome {
    let (left, right) = (line(left), line(right));
    classify(&compare(&left, &right), &left, &right, &[&rules]).outcome
}

const ALL: BuiltInRules =
    BuiltInRules { leading_whitespace: true, embedded_whitespace: true, trailing_whitespace: true, everything_else: true, character_case: true };

/// The rules with each whitespace element ticked or not, in the order leading, embedded, trailing.
fn whitespace([leading_whitespace, embedded_whitespace, trailing_whitespace]: [bool; 3]) -> BuiltInRules {
    BuiltInRules { leading_whitespace, embedded_whitespace, trailing_whitespace, ..ALL }
}

#[test]
fn a_whitespace_change_follows_only_the_element_of_its_place_in_the_line() {
    // A change in leading, embedded and trailing whitespace, in that order.
    let pairs = [("    let x = 1;", "\tlet x = 1;"), ("a  b", "a b"), ("end;   ", "end;")];
    for (place, (left, right)) in pairs.into_iter().enumerate() {
        for element in 0..3 {
            let unticked = outcome(left, right, whitespace(std::array::from_fn(|e| e != element)));
            let want = if element == place { Outcome::UnimportantOnly } else { Outcome::Important };
            assert_eq!(unticked, want, "{left:?} vs {right:?} with element {element} unticked");
        }
        let only_its_own_ticked = outcome(left, right, whitespace(std::array::from_fn(|e| e == place)));
        assert_eq!(only_its_own_ticked, Outcome::Important, "{left:?} vs {right:?}");
    }
}

#[test]
fn a_line_of_only_whitespace_is_unimportant_against_an_empty_one_when_leading_or_trailing_is_unticked() {
    let cases = [(true, true, Outcome::Important), (false, true, Outcome::UnimportantOnly), (true, false, Outcome::UnimportantOnly)];
    for (leading_whitespace, trailing_whitespace, want) in cases {
        let rules = BuiltInRules { leading_whitespace, trailing_whitespace, embedded_whitespace: false, ..ALL };
        assert_eq!(outcome("  \t", "", rules), want, "leading {leading_whitespace}, trailing {trailing_whitespace}");
    }
}

#[test]
fn an_inserted_or_deleted_blank_line_stays_important_with_every_whitespace_element_unticked() {
    let rules = BuiltInRules { everything_else: false, character_case: false, ..whitespace([false; 3]) };
    let without = vec![Line { text: "a".to_owned(), ending: LineEnding::Lf }; 2];
    for blank in ["", "  \t"] {
        let mut with = without.clone();
        with.insert(1, Line { text: blank.to_owned(), ending: LineEnding::Lf });
        for (left, right) in [(&without, &with), (&with, &without)] {
            let c = classify(&compare(left, right), left, right, &[&rules]);
            assert_eq!(c.outcome, Outcome::Important, "{blank:?}");
            assert_eq!((c.statistics.important, c.statistics.unimportant), (1, 0), "{blank:?}");
        }
    }
}

#[test]
fn a_case_only_change_is_unimportant_only_when_character_case_is_unticked() {
    let ignore_case = BuiltInRules { character_case: false, ..ALL };
    assert_eq!(outcome("Hello World", "hello world", ALL), Outcome::Important);
    assert_eq!(outcome("Hello World", "hello world", ignore_case), Outcome::UnimportantOnly);
    assert_eq!(outcome("Hello World", "hello word", ignore_case), Outcome::Important);
}

#[test]
fn with_everything_else_unticked_only_ticked_whitespace_can_matter() {
    let rules = BuiltInRules { everything_else: false, ..ALL };
    assert_eq!(outcome("let x = 1;", "var y = 2;", rules), Outcome::UnimportantOnly);
    assert_eq!(outcome("let x = 1;", "let x  = 1;", rules), Outcome::Important);
}

fn ranges(text: &str, side: Side, all: &[CharRange]) -> Vec<(String, Importance)> {
    all.iter().filter(|r| r.side == side).map(|r| (text[r.start..r.end].to_owned(), r.importance)).collect()
}

#[test]
fn a_case_change_and_a_word_change_in_one_line_get_their_own_importance() {
    let (left, right) = ("Foo bar", "foo baz");
    let rules = BuiltInRules { character_case: false, ..ALL };
    let result = char_differences_with(left, right, &[&rules]);
    let (u, i) = (Importance::Unimportant, Importance::Important);
    assert_eq!(ranges(left, Side::Left, &result.ranges), [("F".to_owned(), u), ("r".to_owned(), i)]);
    assert_eq!(ranges(right, Side::Right, &result.ranges), [("f".to_owned(), u), ("z".to_owned(), i)]);
}

/// Every string of up to three characters over a space, `a`, `A` and `b`.
fn short_strings() -> Vec<String> {
    let mut all = vec![String::new()];
    let mut last = vec![String::new()];
    for _ in 0..3 {
        last = last.iter().flat_map(|s| [' ', 'a', 'A', 'b'].map(|c| format!("{s}{c}"))).collect();
        all.extend(last.iter().cloned());
    }
    all
}

/// `text` with the ranges on `side` taken out.
fn unmarked(text: &str, side: Side, ranges: &[CharRange]) -> String {
    let (mut kept, mut at) = (String::new(), 0);
    for r in ranges.iter().filter(|r| r.side == side) {
        kept.push_str(&text[at..r.start]);
        at = r.end;
    }
    kept.push_str(&text[at..]);
    kept
}

#[test]
fn character_ranges_agree_with_the_row_and_with_the_spans_the_rules_mark() {
    let rule_sets = [
        BuiltInRules { leading_whitespace: false, ..ALL },
        BuiltInRules { embedded_whitespace: false, ..ALL },
        BuiltInRules { trailing_whitespace: false, character_case: false, ..ALL },
        BuiltInRules { leading_whitespace: false, embedded_whitespace: false, trailing_whitespace: false, ..ALL },
        BuiltInRules { everything_else: false, ..ALL },
    ];
    let all = short_strings();
    let lines: Vec<Vec<Line>> = all.iter().map(|text| line(text)).collect();
    for rules in rule_sets {
        // Each pair once: swapping its sides only swaps the ranges, checked below, so the other
        // order holds too.
        for (a, (left, left_line)) in all.iter().zip(&lines).enumerate() {
            for (right, right_line) in all.iter().zip(&lines).skip(a + 1) {
                let result = char_differences_with(left, right, &[&rules]);
                assert!(result.exact, "{rules:?} {left:?} {right:?}: {:?}", result.ranges);
                // Rows and ranges agree on whether anything here matters.
                let row = classify(&compare(left_line, right_line), left_line, right_line, &[&rules]);
                let row_unimportant = row.outcome == Outcome::UnimportantOnly;
                assert_eq!(
                    row_unimportant,
                    result.ranges.iter().all(|r| r.importance == Importance::Unimportant),
                    "{rules:?} {left:?} {right:?}: {:?}",
                    result.ranges
                );
                // Dropping every marked character leaves the lines the same.
                assert_eq!(
                    unmarked(left, Side::Left, &result.ranges),
                    unmarked(right, Side::Right, &result.ranges),
                    "{rules:?} {left:?} {right:?}: {:?}",
                    result.ranges
                );
                // A range lying wholly in spans the rules mark is unimportant.
                for r in &result.ranges {
                    let text = if r.side == Side::Left { left } else { right };
                    let spans = rules.unimportant(text);
                    let covered = (r.start..r.end).all(|b| spans.iter().any(|s| s.contains(&b)));
                    assert!(!covered || r.importance == Importance::Unimportant, "{rules:?} {left:?} {right:?}: {:?}", result.ranges);
                }
                // Swapping the sides only swaps the ranges.
                let mut swapped: Vec<CharRange> = char_differences_with(right, left, &[&rules])
                    .ranges
                    .into_iter()
                    .map(|r| CharRange { side: if r.side == Side::Left { Side::Right } else { Side::Left }, ..r })
                    .collect();
                swapped.sort_by_key(|r| (r.side, r.start));
                assert_eq!(result.ranges, swapped, "{rules:?} {left:?} {right:?}: {:?}", result.ranges);
            }
        }
    }
}
