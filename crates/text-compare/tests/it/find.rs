//! Finding text in both panes (E07-S5-T1).

use otd_text_compare::{Direction, FindOptions, Finder, Found, Line, LineEnding, Pos, RegexError, Selection, Side, Sides};

fn lines(text: &str) -> Vec<Line> {
    text.lines().map(|line| Line { text: line.to_owned(), ending: LineEnding::Lf }).collect()
}

fn finder(text: &str, options: FindOptions) -> Finder {
    Finder::new(text, options).unwrap_or_else(|e| panic!("{text:?} compiles: {e}"))
}

fn all(finder: &Finder, left: &str, right: &str) -> Vec<Found> {
    let (left, right) = (lines(left), lines(right));
    finder.find_all([&left, &right]).unwrap_or_else(|e| panic!("the search runs: {e}"))
}

fn at(line: usize, column: usize) -> Selection {
    Selection::caret(Pos { line, column })
}

#[test]
fn whole_words_only_matches_a_word_with_no_word_character_next_to_it() {
    let whole = FindOptions { whole_words: true, ..FindOptions::default() };
    // The catalogue's example: only the first `apple` is a whole word.
    let found = all(&finder("apple", whole), "apples applesauce snapple apple", "");
    assert_eq!(found.iter().map(|f| (f.columns.start, f.columns.end)).collect::<Vec<_>>(), [(26, 31)]);
    // A rejected candidate doesn't hide a whole word: not the leftmost alternative, nor text a
    // rejected match would have consumed.
    assert_eq!(all(&finder("app|apple", FindOptions { regex: true, ..whole }), "apple", "").len(), 1);
    assert_eq!(all(&finder("a a", whole), "xa a a", "").iter().map(|f| (f.columns.start, f.columns.end)).collect::<Vec<_>>(), [(3, 6)]);
}

#[test]
fn a_regex_backreference_matches_only_a_repeat_of_its_group() {
    let repeat = finder(r"(\w+) \1", FindOptions { regex: true, match_case: true, ..FindOptions::default() });
    let found = all(&repeat, "the the cat\nthe cat", "");
    assert_eq!(found.iter().map(|f| f.line).collect::<Vec<_>>(), [0], "only the repeated word matches");
}

#[test]
fn case_is_ignored_unless_match_character_case_is_on() {
    assert_eq!(all(&finder("Apple", FindOptions::default()), "APPLE apple", "").len(), 2);
    assert_eq!(all(&finder("Apple", FindOptions { match_case: true, ..FindOptions::default() }), "APPLE apple", "").len(), 0);
}

#[test]
fn plain_text_finds_itself_literally() {
    let text = "a.c (x)*+? [^$] {2} | <_-#> \\1 \t ☺ e\u{301}";
    let found = all(&finder(text, FindOptions { match_case: true, ..FindOptions::default() }), &format!("say {text}!"), "");
    assert_eq!(found.iter().map(|f| (f.columns.start, f.columns.end)).collect::<Vec<_>>(), [(4, 4 + text.chars().count())]);
    assert!(all(&finder("a.c", FindOptions::default()), "abc", "").is_empty(), "a period is no wildcard");
}

#[test]
fn an_invalid_regex_is_an_error_at_its_place_in_the_text_to_find() {
    for text in ["a)", "(a", "a\\q", "*a"] {
        let at = |match_case, whole_words| match Finder::new(text, FindOptions { regex: true, match_case, whole_words, ..FindOptions::default() }) {
            Err(RegexError::Syntax { at, .. }) => at,
            other => panic!("{text:?} gave {other:?}"),
        };
        assert!(at(true, false) <= text.len(), "{text:?}: the error is inside the text or at its end");
        for (match_case, whole_words) in [(false, false), (false, true), (true, true)] {
            assert_eq!(at(match_case, whole_words), at(true, false), "{text:?}: the options don't move the error");
        }
    }
}

#[test]
fn columns_count_characters_not_bytes() {
    let found = all(&finder("apple", FindOptions::default()), "é☺apple", "");
    assert_eq!(found[0].columns, 2..7);
}

#[test]
fn sides_pick_the_panes_searched_and_the_left_comes_first() {
    let sides =
        |sides| all(&finder("x", FindOptions { sides, ..FindOptions::default() }), "x", "x\nx").into_iter().map(|f| f.side).collect::<Vec<_>>();
    assert_eq!(sides(Sides::Left), [Side::Left]);
    assert_eq!(sides(Sides::Right), [Side::Right, Side::Right]);
    assert_eq!(sides(Sides::Both), [Side::Left, Side::Right, Side::Right]);
}

#[test]
fn next_and_previous_step_past_the_selected_match_and_cross_panes() {
    let (left, right) = (lines("ab ab"), lines("ab"));
    let panes = [left.as_slice(), right.as_slice()];
    let finder = finder("ab", FindOptions::default());
    let step = |side, selection, direction| finder.find(panes, side, selection, direction, None).unwrap_or_else(|e| panic!("{e}"));

    let first = step(Side::Left, at(0, 0), Direction::Forward).unwrap_or_else(|| panic!("a first match"));
    let second = step(Side::Left, first.selection(), Direction::Forward).unwrap_or_else(|| panic!("a second match"));
    assert!(second.columns.start > first.columns.start, "forward moves on from the selected match");
    let third = step(Side::Left, second.selection(), Direction::Forward).unwrap_or_else(|| panic!("a third match"));
    assert_eq!(third.side, Side::Right, "the left pane's end leads on to the right pane");
    assert_eq!(step(Side::Right, third.selection(), Direction::Backward), Some(second.clone()), "backward steps back across the panes");
    assert_eq!(step(Side::Left, second.selection(), Direction::Backward), Some(first), "backward moves back from the selected match");
}

#[test]
fn wrap_search_goes_on_from_the_other_end_only_when_on() {
    let (left, right) = (lines("ab"), lines(""));
    let panes = [left.as_slice(), right.as_slice()];
    for (direction, from) in [(Direction::Forward, at(0, 1)), (Direction::Backward, at(0, 0))] {
        let search =
            |wrap| finder("ab", FindOptions { wrap, ..FindOptions::default() }).find(panes, Side::Left, from, direction, None).ok().flatten();
        assert_eq!(search(false), None, "{direction:?} without wrap");
        assert!(search(true).is_some(), "{direction:?} with wrap");
    }
}

#[test]
fn a_match_that_selects_nothing_is_stepped_past_both_ways() {
    let (left, right) = (lines("a\n\nb\n  \nc"), lines(""));
    let panes = [left.as_slice(), right.as_slice()];
    let finder = finder(r"^\s*$", FindOptions { regex: true, ..FindOptions::default() });
    let step = |selection, direction, previous| finder.find(panes, Side::Left, selection, direction, previous).ok().flatten();

    let first = step(at(0, 0), Direction::Forward, None).unwrap_or_else(|| panic!("the first blank line"));
    assert_eq!(first.line, 1);
    let second = step(first.selection(), Direction::Forward, Some(&first)).unwrap_or_else(|| panic!("the second blank line"));
    assert_eq!(second.line, 3);
    assert_eq!(step(second.selection(), Direction::Forward, Some(&second)), None);
    assert_eq!(step(Selection::caret(Pos { line: 3, column: 0 }), Direction::Backward, None), Some(first.clone()));
    assert_eq!(step(first.selection(), Direction::Backward, Some(&first)), None, "backward steps past the match just found");
}

#[test]
fn a_match_that_selects_nothing_at_a_fresh_caret_is_found() {
    let (left, right) = (lines("\na"), lines(""));
    let panes = [left.as_slice(), right.as_slice()];
    let finder = finder(r"^\s*$", FindOptions { regex: true, ..FindOptions::default() });
    for direction in [Direction::Forward, Direction::Backward] {
        let found = finder.find(panes, Side::Left, at(0, 0), direction, None).ok().flatten();
        assert_eq!(found.map(|f| f.line), Some(0), "{direction:?}");
    }
}

#[test]
fn a_search_from_a_side_not_searched_starts_at_the_first_or_last_match() {
    let (left, right) = (lines("x x"), lines("x"));
    let panes = [left.as_slice(), right.as_slice()];
    let finder = finder("x", FindOptions { sides: Sides::Left, ..FindOptions::default() });
    let from_right = |direction| finder.find(panes, Side::Right, at(0, 0), direction, None).ok().flatten().map(|f| f.columns.start);
    assert_eq!(from_right(Direction::Forward), Some(0));
    assert_eq!(from_right(Direction::Backward), Some(2));
}
