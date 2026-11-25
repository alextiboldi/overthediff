//! Replacement templates: `$1`…`$9` insert the match's groups.

use otd_regex::Regex;

fn replace(pattern: &str, text: &str, template: &str) -> String {
    let regex = Regex::new(pattern).unwrap_or_else(|e| panic!("{pattern:?} compiles: {e}"));
    regex.replace(text, template).unwrap_or_else(|e| panic!("{pattern:?} on {text:?}: {e}"))
}

#[test]
fn the_catalogue_rename_example_keeps_the_captured_suffix() {
    assert_eq!(replace(r".*(\d\.txt)", "abc1.txt", "xyz$1"), "xyz1.txt");
}

#[test]
fn a_group_the_pattern_lacks_or_that_took_no_part_inserts_nothing() {
    assert_eq!(replace("(a)b", "ab", "[$1$2$9]"), "[a]");
    assert_eq!(replace("(a)|(b)", "a", "<$2>"), "<>");
}

#[test]
fn only_a_dollar_before_one_to_nine_is_a_reference_and_it_takes_one_digit() {
    assert_eq!(replace("(a)", "a", "$0 $$ $12 $x $"), "$0 $$ a2 $x $");
}

#[test]
fn every_match_on_every_line_is_replaced_and_the_rest_is_kept() {
    assert_eq!(replace(r"(\d)", "a1b2\r\nc3d", "<$1>"), "a<1>b<2>\r\nc<3>d");
}
