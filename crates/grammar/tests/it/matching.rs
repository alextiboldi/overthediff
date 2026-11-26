//! What each item category marks, how items share text, and how case is decided.

use otd_grammar::{Category, Item, Trigger};

use crate::{element, grammar, item, marked};

fn delimited(start: &str, end: &str, stop_at_end_of_line: bool, escape: Option<char>) -> Category {
    Category::Delimited { start: start.to_owned(), end: end.to_owned(), stop_at_end_of_line, escape }
}

fn list(tokens: &[&str]) -> Category {
    Category::List { tokens: tokens.iter().map(|t| (*t).to_owned()).collect() }
}

#[test]
fn a_basic_item_marks_every_occurrence_of_its_text() {
    let g = grammar(vec![element("Marker", true)], vec![item("Marker", Category::Basic { text: "TODO".into() })]);
    let found = marked(&g, "TODO one; TODO two");
    assert_eq!(found, [(0, "TODO", "Marker"), (0, "TODO", "Marker")]);
}

#[test]
fn an_escaped_end_delimiter_stays_inside_the_string() {
    let g = grammar(vec![element("String", true)], vec![item("String", delimited("'", "'", true, Some('\\')))]);
    assert_eq!(marked(&g, r"say 'I can\'t' now"), [(0, r"'I can\'t'", "String")]);
    assert_eq!(marked(&g, r"'a\\' b"), [(0, r"'a\\'", "String")], "an escaped escape character escapes nothing");
}

#[test]
fn a_delimited_item_runs_across_lines_unless_it_stops_at_the_end_of_the_line() {
    let block = grammar(vec![element("Comment", true)], vec![item("Comment", delimited("/*", "*/", false, None))]);
    let found = marked(&block, "a /* one\n\ntwo */ b");
    assert_eq!(found, [(0, "/* one", "Comment"), (2, "two */", "Comment")]);

    let string = grammar(vec![element("String", true)], vec![item("String", delimited("\"", "\"", true, None))]);
    assert_eq!(marked(&string, "x = \"open\ny = 1"), [(0, "\"open", "String")]);
}

#[test]
fn a_list_keyword_matches_any_case_unless_both_it_and_its_element_match_case() {
    let keywords = |case_sensitive, match_case| {
        let items = vec![Item { match_case, ..item("Keyword", list(&["select"])) }];
        grammar(vec![element("Keyword", case_sensitive)], items)
    };
    assert_eq!(marked(&keywords(true, true), "SELECT select").len(), 1, "both flags set: case must match");
    assert_eq!(marked(&keywords(false, true), "SELECT select").len(), 2, "a case-insensitive element ignores case");
    assert_eq!(marked(&keywords(true, false), "SELECT select").len(), 2, "an item that doesn't match case ignores it");
}

#[test]
fn list_tokens_match_whole_words_and_the_longer_token_wins() {
    let g = grammar(vec![element("Keyword", true)], vec![item("Keyword", list(&["for", "in", "int", "<", "<="]))]);
    let found: Vec<&str> = marked(&g, "format int x; for(in_x <= 2)").into_iter().map(|(_, text, _)| text).collect();
    assert_eq!(found, ["int", "for", "<="]);
}

#[test]
fn a_columns_item_marks_its_character_columns() {
    let g = grammar(
        vec![element("Id", true), element("Rest", true)],
        vec![item("Id", Category::Columns { start: 2, end: Some(4) }), item("Rest", Category::Columns { start: 6, end: None })],
    );
    // `é` is two bytes and one column.
    assert_eq!(marked(&g, "é234 6789\nab"), [(0, "234", "Id"), (0, "6789", "Rest"), (1, "b", "Id")]);
}

#[test]
fn a_lines_header_covers_its_heading_lines_from_each_trigger() {
    let header = Category::Lines { trigger: Trigger::Text("\x0C".into()), heading_lines: 2 };
    let g = grammar(vec![element("Header", true)], vec![item("Header", header)]);
    let found: Vec<usize> = marked(&g, "body\n\x0CPage 1\nDate\nbody\n\x0CPage 2\nDate\nbody").into_iter().map(|(n, _, _)| n).collect();
    assert_eq!(found, [1, 2, 4, 5]);
}

#[test]
fn the_higher_item_wins_where_matches_start_together() {
    let items = |first: &str, second: &str| vec![item(first, Category::Basic { text: "null".into() }), item(second, list(&["null", "true"]))];
    let elements = || vec![element("Constant", true), element("Keyword", true)];
    let constant_first = grammar(elements(), items("Constant", "Keyword"));
    let keyword_first = grammar(elements(), items("Keyword", "Constant"));
    assert_eq!(marked(&constant_first, "null")[0].2, "Constant");
    assert_eq!(marked(&keyword_first, "null")[0].2, "Keyword");
}

#[test]
fn a_match_holds_its_text_against_matches_that_would_start_inside_it() {
    let g = grammar(
        vec![element("Comment", true), element("String", true)],
        vec![item("Comment", delimited("//", "", true, None)), item("String", delimited("\"", "\"", true, Some('\\')))],
    );
    assert_eq!(marked(&g, "s = \"a // b\";"), [(0, "\"a // b\"", "String")]);
    assert_eq!(marked(&g, "// say \"hi\""), [(0, "// say \"hi\"", "Comment")]);
}
