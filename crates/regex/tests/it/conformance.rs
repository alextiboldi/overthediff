//! The A.2 conformance cases and the sample library, as tables of pattern, text and the part of
//! the text the first match covers (`None`: no match).

use otd_regex::{Regex, RegexError};

fn first<'t>(pattern: &str, text: &'t str) -> Option<&'t str> {
    let regex = Regex::new(pattern).unwrap_or_else(|e| panic!("{pattern:?} compiles: {e}"));
    let found = regex.matches(text).next()?.unwrap_or_else(|e| panic!("{pattern:?} on {text:?}: {e}"));
    Some(&text[found.range()])
}

fn check(cases: &[(&str, &str, Option<&str>)]) {
    for &(pattern, text, expected) in cases {
        assert_eq!(first(pattern, text), expected, "{pattern:?} on {text:?}");
    }
}

#[test]
fn escapes_stand_for_the_characters_a2_names() {
    check(&[
        (r"\x41", "zA", Some("A")),
        (r"\x{263A}", "a☺", Some("☺")),
        (r"\x{41}1", "A1", Some("A1")),
        (r"\t", "a\tb", Some("\t")),
        (r"\f", "a\x0Cb", Some("\x0C")),
        (r"\a", "a\x07b", Some("\x07")),
        (r"\e", "a\x1Bb", Some("\x1B")),
        (r"a\.b", "axb a.b", Some("a.b")),
        (r"\$\^\(\)\[\]\{\}\*\+\?\|\\", r"$^()[]{}*+?|\", Some(r"$^()[]{}*+?|\")),
        // A `{` that starts no quantifier is a literal.
        ("a{x", "a{x", Some("a{x")),
        // Characters that mean something in other syntaxes are plain here.
        ("#&&~~", "#&&~~", Some("#&&~~")),
    ]);
}

#[test]
fn classes_match_exactly_their_characters() {
    check(&[
        ("a.c", "abc", Some("abc")),
        (r"\w+", "-ab_9-", Some("ab_9")),
        (r"\W", "ab-", Some("-")),
        (r"\d+", "ab12c", Some("12")),
        (r"\D", "12c", Some("c")),
        (r"\s+", "a \t b", Some(" \t ")),
        (r"\s", "a\x0Bb\x0C", None),
        (r"\S", " \tx", Some("x")),
        ("[a-c]+", "xabcd", Some("abc")),
        ("[^a-c]", "abcd", Some("d")),
        ("[-a]+", "x-a", Some("-a")),
        ("[a-]+", "x-a", Some("-a")),
        (r"[\d_]+", "a1_2b", Some("1_2")),
        (r"[^\s]+", " ab ", Some("ab")),
        (r"[\]\\]+", "a]\\", Some("]\\")),
        (r"[\x41-\x43]+", "ABCD", Some("ABC")),
        (r"[\d-z]+", "a1-z", Some("1-z")),
    ]);
}

#[test]
fn alternation_takes_the_leftmost_alternative_that_matches() {
    check(&[("get|getter", "getter", Some("get")), ("getter|get", "getter", Some("getter"))]);
}

#[test]
fn greedy_quantifiers_take_the_most_and_lazy_ones_the_least() {
    check(&[
        ("a*", "aaa", Some("aaa")),
        ("a*?", "aaa", Some("")),
        ("a+", "aaa", Some("aaa")),
        ("a+?", "aaa", Some("a")),
        ("ab?", "abb", Some("ab")),
        ("ab??", "abb", Some("a")),
        ("a{2}", "aaa", Some("aa")),
        ("a{2}?", "aaa", Some("aa")),
        ("a{2,}", "aaaa", Some("aaaa")),
        ("a{2,}?", "aaaa", Some("aa")),
        ("a{1,3}", "aaaa", Some("aaa")),
        ("a{1,3}?", "aaaa", Some("a")),
        ("<.*>", "<a><b>", Some("<a><b>")),
        ("<.*?>", "<a><b>", Some("<a>")),
    ]);
}

#[test]
fn backreferences_repeat_what_their_group_matched() {
    check(&[
        (r"(a|b)\1", "abba", Some("bb")),
        (r"(\w)(\w)\2\1", "xabbay", Some("abba")),
        // `\1` then a literal `0`, not group 10.
        (r"(a)\10", "aa0", Some("aa0")),
    ]);
}

#[test]
fn case_switches_apply_only_up_to_the_end_of_their_group() {
    check(&[
        ("(?i)abc", "xABC", Some("ABC")),
        ("a(?i)b", "AB aB", Some("aB")),
        ("(a(?i)b)c", "aBC aBc", Some("aBc")),
        ("(?i)a((?-i)b)c", "ABC AbC", Some("AbC")),
        ("(?i)[a-c]+", "xBCA", Some("BCA")),
        ("(?i)x|y", "Y", Some("Y")),
        ("a(?i)b|c", "C", Some("C")),
        ("(a(?i)b|c)d", "aBD cd", Some("cd")),
        ("(?i)a(?-i)b|c", "AB C c", Some("c")),
    ]);
}

#[test]
fn no_match_crosses_a_line_break() {
    for text in ["a\nb", "a\r\nb", "a\rb"] {
        for pattern in [r"a\nb", r"a\r\nb", "a.b", r"a\sb", r"a\Wb", "a[^x]b", r"a\Sb"] {
            assert_eq!(first(pattern, text), None, "{pattern:?} on {text:?}");
        }
    }
}

#[test]
fn anchors_hold_at_each_line_and_match_ranges_point_into_the_whole_text() {
    let text = "x abc\nabc\r\nabc y\n";
    for (pattern, starts) in [("^abc", vec![6, 11]), ("abc$", vec![2, 6]), (r"\Aabc", vec![6, 11]), (r"abc\Z", vec![2, 6]), ("^abc$", vec![6])] {
        let regex = Regex::new(pattern).unwrap_or_else(|e| panic!("{pattern:?}: {e}"));
        let found: Vec<_> = regex.matches(text).map(|m| m.map(|m| m.range())).collect::<Result<_, _>>().unwrap_or_default();
        assert!(found.iter().all(|r| &text[r.clone()] == "abc"), "{pattern:?} ranges cover abc");
        assert_eq!(found.iter().map(|r| r.start).collect::<Vec<_>>(), starts, "{pattern:?}");
    }
}

#[test]
fn groups_report_their_part_of_the_match() {
    let regex = Regex::new(r"(\w+)=(x)?(\d+)").unwrap_or_else(|e| panic!("{e}"));
    let text = "line\nkey=42";
    let found = regex.matches(text).next().and_then(Result::ok).unwrap_or_else(|| panic!("a match"));
    assert_eq!(found.group(1).map(|r| &text[r]), Some("key"));
    assert_eq!(found.group(2), None, "a group that took no part");
    assert_eq!(found.group(3).map(|r| &text[r]), Some("42"));
    assert_eq!(found.group(4), None, "a group the pattern doesn't have");
}

#[test]
fn the_sample_library_matches_exactly_the_specified_strings() {
    let samples: [(&str, &[&str], &[&str]); 8] = [
        ("^abc", &["abc", "abcdef"], &["xabc", " abc"]),
        ("abc$", &["abc", "xyzabc"], &["abcx", "abc "]),
        ("^abc$", &["abc"], &["abcabc", " abc", "abc "]),
        (r"^\s*abc", &["abc", "  abc", "\t abcd"], &["xabc", " x abc"]),
        (r"^\s*end;?\s*$", &["end", "end;", "  end; \t", "\tend"], &["end;;", "ending", "x end", "end x"]),
        ("abc|def", &["abc", "xdefx", "abcdef"], &["abde", "ab"]),
        ("a(b|c)d", &["abd", "acd", "xacdx"], &["aed", "abcd", "ad"]),
        (r"a(b|c)d\1", &["abdb", "acdc"], &["abdc", "acdb", "abd"]),
    ];
    for (pattern, matching, other) in samples {
        let regex = Regex::new(pattern).unwrap_or_else(|e| panic!("{pattern:?}: {e}"));
        for text in matching {
            assert_eq!(regex.is_match(text), Ok(true), "{pattern:?} matches {text:?}");
        }
        for text in other {
            assert_eq!(regex.is_match(text), Ok(false), "{pattern:?} doesn't match {text:?}");
        }
    }
}

#[test]
fn constructs_a2_does_not_define_are_syntax_errors_at_their_offset() {
    for (pattern, at) in [
        ("ab(", 3),
        ("a)", 1),
        ("[ab", 0),
        ("[]", 0),
        ("*a", 0),
        ("a|+", 2),
        ("a**", 2),
        ("^*", 1),
        ("(?i)*", 4),
        (r"a\q", 1),
        (r"\b", 0),
        (r"\x4", 0),
        (r"\x{}", 0),
        (r"\x{110000}", 0),
        ("[z-a]", 1),
        (r"[a-\d]", 3),
        (r"[\1]", 1),
        (r"(a)\2", 3),
        ("(?:a)", 0),
        ("a{3,2}", 1),
        ("a\\", 1),
    ] {
        match Regex::new(pattern) {
            Err(RegexError::Syntax { at: found, .. }) => assert_eq!(found, at, "{pattern:?}"),
            other => panic!("{pattern:?} gave {other:?}"),
        }
    }
}

#[test]
fn runaway_backtracking_is_reported_not_endured() {
    let regex = Regex::new(r"(a*)*\1b").unwrap_or_else(|e| panic!("{e}"));
    let text = "a".repeat(64);
    assert_eq!(regex.is_match(&text), Err(RegexError::TooComplex));

    let lines = vec![text; 3].join("\n");
    let found: Vec<_> = regex.matches(&lines).collect();
    assert_eq!(found, [Err(RegexError::TooComplex)], "the first error ends the search");
}
