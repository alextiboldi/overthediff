//! File text is untrusted, so marking damaged text never panics or hangs, and every span stays a
//! well-formed piece of its line.

use craft_testkit::hostile::{HammerConfig, hammer};
use otd_grammar::{Category, Item, Trigger};

use crate::{element, grammar, item};

#[test]
fn marking_damaged_text_never_panics_and_keeps_spans_in_order_inside_their_line() {
    let regex = |category| Item { regex: true, match_case: false, ..item("Keyword", category) };
    let g = grammar(
        vec![element("Comment", true), element("String", true), element("Keyword", false), element("Header", true), element("Id", true)],
        vec![
            item("Header", Category::Lines { trigger: Trigger::Text("\x0C".into()), heading_lines: 2 }),
            item("Comment", Category::Delimited { start: "/*".into(), end: "*/".into(), stop_at_end_of_line: false, escape: None }),
            item("String", Category::Delimited { start: "'".into(), end: "'".into(), stop_at_end_of_line: true, escape: Some('é') }),
            regex(Category::List { tokens: vec!["fo+".into(), "x*".into(), "^\\w".into()] }),
            regex(Category::Basic { text: "\\d+|$".into() }),
            item("Id", Category::Columns { start: 3, end: Some(5) }),
        ],
    );
    let seed = "\x0Chead☺\nfoo /* a\r\n'é'' b */ 'xé\x00 12 ☺☺☺ fooo\n\n\x0C";
    hammer(&HammerConfig::new("grammar text", 2_000), seed.as_bytes(), move |data| {
        let text = String::from_utf8_lossy(data);
        let lines: Vec<&str> = text.split(['\n', '\r']).collect();
        let Ok(spans) = g.highlight(lines.iter().copied()) else { return };
        assert_eq!(spans.len(), lines.len(), "one span list per line");
        for (line, line_spans) in lines.iter().zip(&spans) {
            let mut end = 0;
            for span in line_spans {
                assert!(
                    end <= span.range.start && span.range.start < span.range.end && span.range.end <= line.len(),
                    "spans are ordered, non-empty and inside the line"
                );
                assert!(line.get(span.range.clone()).is_some(), "spans fall on character boundaries");
                end = span.range.end;
            }
        }
    });
}

/// Minified and generated files put a whole program on one line, so a line's cost must grow with
/// its length, not its square: a regression here makes this test take minutes rather than
/// milliseconds.
#[test]
fn a_megabyte_on_one_line_marks_every_keyword() {
    let mut tokens: Vec<String> = (0..40).map(|n| format!("absent{n}")).collect();
    tokens.push("var".into());
    let g = grammar(
        vec![element("Keyword", true), element("String", true)],
        vec![
            item("String", Category::Delimited { start: "\"".into(), end: "\"".into(), stop_at_end_of_line: true, escape: Some('\\') }),
            item("Keyword", Category::List { tokens }),
        ],
    );
    let piece = "var a=\"b\";";
    let line = piece.repeat(1_000_000 / piece.len());
    let keywords = crate::marked(&g, &line).iter().filter(|(_, text, element)| (*text, *element) == ("var", "Keyword")).count();
    assert_eq!(keywords, line.matches("var").count(), "every keyword on the line is marked");
}
