//! otd-grammar's integration tests: one binary, a module per area (AGENTS.md rule 30).

mod hostile;
mod matching;

use otd_grammar::{Category, Element, Grammar, Item};

/// An element called `name`.
pub fn element(name: &str, case_sensitive: bool) -> Element {
    Element { name: name.to_owned(), case_sensitive }
}

/// A literal, case-matching item that marks `element`.
pub fn item(element: &str, category: Category) -> Item {
    Item { element: element.to_owned(), category, match_case: true, regex: false }
}

pub fn grammar(elements: Vec<Element>, items: Vec<Item>) -> Grammar {
    Grammar::new(elements, items).unwrap_or_else(|e| panic!("the grammar compiles: {e}"))
}

/// Each marked piece of `text`, as (line index, text, element name).
pub fn marked<'a>(grammar: &'a Grammar, text: &'a str) -> Vec<(usize, &'a str, &'a str)> {
    let lines: Vec<&str> = text.split('\n').collect();
    let spans = grammar.highlight(lines.iter().copied()).unwrap_or_else(|e| panic!("highlighting finishes: {e}"));
    let mut out = Vec::new();
    for (n, line_spans) in spans.iter().enumerate() {
        let line: &'a str = lines[n];
        for span in line_spans {
            out.push((n, &line[span.range.clone()], grammar.elements()[span.element].name.as_str()));
        }
    }
    out
}
