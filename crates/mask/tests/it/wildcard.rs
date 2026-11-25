//! Wildcards against single names: the A.1 conformance table from story E02-S1's acceptance
//! criteria, then the rules that follow from them.

use otd_mask::{Case, Wildcard, WildcardError};

/// Mask, name, whether the name matches. Each row comes from a rule the story's AC state.
const A1: &[(&str, &str, bool)] = &[
    // `?` matches one character, a period included in an ordinary mask.
    ("a?c", "abc", true),
    ("a?c", "a.c", true),
    ("a?c", "ac", false),
    ("a?c", "abbc", false),
    // `*` matches zero or more characters.
    ("a*c", "ac", true),
    ("a*c", "abbbc", true),
    ("a*c", "a.b.c", true),
    ("a*c", "ab", false),
    // `[az]` is a set.
    ("[az]x", "ax", true),
    ("[az]x", "zx", true),
    ("[az]x", "bx", false),
    // `[a-z]` is a range.
    ("[a-c]x", "bx", true),
    ("[a-c]x", "dx", false),
    // `[!az]` is a negated set.
    ("[!az]x", "bx", true),
    ("[!az]x", "ax", false),
    // `[[]` is a literal `[`.
    ("[[]1]", "[1]", true),
    ("[[]1]", "11]", false),
    // `*.*` matches names with one extension, several periods, or none.
    ("*.*", "a.txt", true),
    ("*.*", "a.b.txt", true),
    ("*.*", "readme", true),
    // `*.` matches only names without an extension.
    ("*.", "readme", true),
    ("*.", "a.txt", false),
    ("*.", "a.b.txt", false),
    // In a mask ending in `.`, `?` doesn't match a period.
    ("a?c.", "abc", true),
    ("a?c.", "a.c", false),
    // An ordinary mask against names with several periods or no extension.
    ("*.txt", "a.txt", true),
    ("*.txt", "a.b.txt", true),
    ("*.txt", "a.txt.bak", false),
    ("*.txt", "txt", false),
    ("*.txt", "readme", false),
];

#[test]
fn the_a1_conformance_table_passes() {
    let failures: Vec<String> = A1
        .iter()
        .filter_map(|&(mask, name, want)| {
            let got = Wildcard::new(mask, Case::Sensitive).map(|w| w.matches(name));
            (got != Ok(want)).then(|| format!("{mask:?} vs {name:?}: want {want}, got {got:?}"))
        })
        .collect();
    assert!(failures.is_empty(), "A.1 rows that fail:\n{}", failures.join("\n"));
}

/// The AC restrict only `?` and `*` in a mask ending in `.`, so a set may still match a period. A
/// matcher that backtracks only the last `*` misses this match: the first `*` has to give way.
#[test]
fn a_set_can_match_a_period_that_the_stars_around_it_cannot_in_a_mask_ending_in_a_period() {
    let mask = Wildcard::new("*[.a]*x.", Case::Sensitive).unwrap();
    assert!(mask.matches("aa.bx"));
}

#[test]
fn a_set_without_its_closing_bracket_is_an_error() {
    assert_eq!(Wildcard::new("a[bc", Case::Sensitive).err(), Some(WildcardError::UnclosedSet));
}

/// Whether a mask ignores case is the caller's choice, since it follows the data source. Ignoring
/// case reaches characters, ranges and negated sets alike.
#[test]
fn ignoring_case_matches_names_that_differ_only_in_case_and_comparing_it_does_not() {
    for (mask, name, insensitive) in [("*.txt", "README.TXT", true), ("[a-c]x", "BX", true), ("[!a]x", "Ax", false)] {
        for (case, want) in [(Case::Insensitive, insensitive), (Case::Sensitive, !insensitive)] {
            assert_eq!(Wildcard::new(mask, case).unwrap().matches(name), want, "{mask:?} vs {name:?} with {case:?}");
        }
    }
}
