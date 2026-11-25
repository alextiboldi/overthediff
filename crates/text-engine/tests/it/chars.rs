//! Character-level differences inside a changed row.

use otd_text_engine::{CharRange, Side, char_differences};

/// Every string of up to four characters over `a`, `b` and a two-byte `é`: small enough to pair
/// each with each, and enough to hit repeats, ties and multi-byte offsets.
fn short_strings() -> Vec<String> {
    let mut all = vec![String::new()];
    let mut last = vec![String::new()];
    for _ in 0..4 {
        last = last.iter().flat_map(|s| ['a', 'b', 'é'].map(|c| format!("{s}{c}"))).collect();
        all.extend(last.iter().cloned());
    }
    all
}

/// `text` with the ranges on `side` taken out, and how many characters they held.
fn unmarked(text: &str, side: Side, ranges: &[CharRange]) -> (String, usize) {
    let (mut kept, mut marked, mut at) = (String::new(), 0, 0);
    for r in ranges.iter().filter(|r| r.side == side) {
        kept.push_str(&text[at..r.start]);
        marked += text[r.start..r.end].chars().count();
        at = r.end;
    }
    kept.push_str(&text[at..]);
    (kept, marked)
}

/// The length of a longest common subsequence of the characters, by the textbook table.
fn lcs(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut row = vec![0; b.len() + 1];
    for x in &a {
        let mut diag = 0;
        for (j, y) in b.iter().enumerate() {
            let up = row[j + 1];
            row[j + 1] = if x == y { diag + 1 } else { up.max(row[j]) };
            diag = up;
        }
    }
    row[b.len()]
}

/// The byte span of the first `word` in `text`.
fn span(text: &str, word: &str) -> std::ops::Range<usize> {
    let at = text.find(word).unwrap();
    at..at + word.len()
}

#[test]
fn ranges_run_left_then_right_in_order_without_touching_on_character_boundaries() {
    let all = short_strings();
    for left in &all {
        for right in &all {
            let ranges = char_differences(left, right).ranges;
            assert!(ranges.windows(2).all(|w| (w[0].side, w[0].end) < (w[1].side, w[1].start)), "{left:?} {right:?}: {ranges:?}");
            for r in &ranges {
                let text = if r.side == Side::Left { left } else { right };
                assert!(r.start < r.end && text.get(r.start..r.end).is_some(), "{left:?} {right:?}: {r:?}");
            }
        }
    }
}

#[test]
fn the_ranges_cover_exactly_the_characters_a_shortest_edit_changes() {
    let all = short_strings();
    for left in &all {
        for right in &all {
            let result = char_differences(left, right);
            assert!(result.exact, "{left:?} {right:?}");
            let ranges = result.ranges;
            let (left_kept, left_marked) = unmarked(left, Side::Left, &ranges);
            let (right_kept, right_marked) = unmarked(right, Side::Right, &ranges);
            assert_eq!(left_kept, right_kept, "{left:?} {right:?}: {ranges:?}");
            let fewest = left.chars().count() + right.chars().count() - 2 * lcs(left, right);
            assert_eq!(left_marked + right_marked, fewest, "{left:?} {right:?}: {ranges:?}");
        }
    }
}

#[test]
fn swapping_the_sides_only_swaps_the_ranges() {
    let all = short_strings();
    for left in &all {
        for right in &all {
            let reversed = char_differences(right, left);
            let mut swapped: Vec<CharRange> =
                reversed.ranges.into_iter().map(|r| CharRange { side: if r.side == Side::Left { Side::Right } else { Side::Left }, ..r }).collect();
            swapped.sort_by_key(|r| (r.side, r.start));
            let result = char_differences(left, right);
            assert_eq!((result.ranges, result.exact), (swapped, reversed.exact), "{left:?} {right:?}");
        }
    }
}

#[test]
fn changed_words_in_a_long_line_mark_only_those_words_characters() {
    let left = "The quick brown fox jumps over the lazy dog, then naps in the warm afternoon sun by the river.";
    let one_word = "The quick green fox jumps over the lazy dog, then naps in the warm afternoon sun by the river.";
    let two_words = "The quick green fox jumps over the lazy dog, then sleeps in the warm afternoon sun by the river.";
    for (right, words) in [(one_word, &[("brown", "green")][..]), (two_words, &[("brown", "green"), ("naps", "sleeps")][..])] {
        let ranges = char_differences(left, right).ranges;
        for &(old, new) in words {
            let (old, new) = (span(left, old), span(right, new));
            assert!(ranges.iter().any(|r| r.side == Side::Left && old.contains(&r.start)), "{right:?}: {ranges:?}");
            assert!(ranges.iter().any(|r| r.side == Side::Right && new.contains(&r.start)), "{right:?}: {ranges:?}");
        }
        for r in &ranges {
            let (text, inside) = match r.side {
                Side::Left => (left, words.iter().map(|w| span(left, w.0)).collect::<Vec<_>>()),
                Side::Right => (right, words.iter().map(|w| span(right, w.1)).collect()),
            };
            assert!(inside.iter().any(|w| w.start <= r.start && r.end <= w.end), "{r:?} in {text:?} is outside the changed words");
        }
    }
}

#[test]
fn changed_whitespace_marks_only_whitespace() {
    let pairs = [("a  b  c", "a b c"), ("\tlet x = 1;", "    let x = 1;"), ("x = 1;  // y", "x =\t1; // y"), ("end;   ", "end;")];
    for (left, right) in pairs {
        let ranges = char_differences(left, right).ranges;
        assert!(!ranges.is_empty(), "{left:?} {right:?}");
        for r in &ranges {
            let text = if r.side == Side::Left { left } else { right };
            assert!(text[r.start..r.end].chars().all(char::is_whitespace), "{left:?} {right:?}: {r:?}");
        }
    }
}

#[test]
fn lines_too_different_to_search_are_reported_as_not_exact_but_still_covered() {
    let (left, right) = ("a".repeat(20_000), "b".repeat(20_000));
    let result = char_differences(&left, &right);
    assert!(!result.exact);
    assert_eq!(unmarked(&left, Side::Left, &result.ranges).0, unmarked(&right, Side::Right, &result.ranges).0);
}

#[test]
fn a_long_insertion_is_exact() {
    let left = "start end";
    let right = format!("start {}end", "x".repeat(20_000));
    assert!(char_differences(left, &right).exact);
}
