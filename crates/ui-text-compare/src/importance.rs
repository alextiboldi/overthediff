//! The Session Settings dialog's Importance tab (E07-S4-T4): a checkbox per built-in element
//! (*Leading whitespace*, *Embedded whitespace*, *Trailing whitespace*, *Everything else*), then
//! *Character case* for everything else, and *Compare line endings*. A ticked element is important.
//! The values live in the session's rules, so saved sessions keep them, and the open view compares
//! again as soon as they change.

use otd_core::session::{Setting, Value};
use otd_text_engine::{BuiltInRules, Options};

use crate::{TEXT_COMPARE, TextCompareView};

/// The tab, in English.
pub const IMPORTANCE_TAB: &str = "Importance";
/// Each element's setting id and label, in the tab's order.
pub const ELEMENTS: [(&str, &str); 5] = [
    ("leading-whitespace", "Leading whitespace"),
    ("embedded-whitespace", "Embedded whitespace"),
    ("trailing-whitespace", "Trailing whitespace"),
    ("everything-else", "Everything else"),
    ("character-case", "Character case"),
];
/// *Compare line endings*: its setting id and label. Off by default.
pub const LINE_ENDINGS: (&str, &str) = ("line-endings", "Compare line endings (PC/Mac/Unix)");

/// The elements, all ticked, then Compare line endings, off.
pub fn session_settings() -> Vec<Setting> {
    let ticked = ELEMENTS.iter().map(|&(id, label)| (id, label, true));
    ticked
        .chain([(LINE_ENDINGS.0, LINE_ENDINGS.1, false)])
        .map(|(id, label, on)| Setting { session_type: TEXT_COMPARE, tab: IMPORTANCE_TAB, id, label, default: Value::Bool(on), choices: &[] })
        .collect()
}

impl TextCompareView {
    /// The rules and options the session's Importance tab sets. A value the session lacks takes
    /// its default.
    pub(crate) fn importance(&self) -> (BuiltInRules, Options) {
        let on = |id, default| match self.session.rule(IMPORTANCE_TAB, id) {
            Some(Value::Bool(on)) => *on,
            _ => default,
        };
        let [leading, embedded, trailing, everything_else, case] = ELEMENTS.map(|(id, _)| on(id, true));
        let rules = BuiltInRules {
            leading_whitespace: leading,
            embedded_whitespace: embedded,
            trailing_whitespace: trailing,
            everything_else,
            character_case: case,
        };
        (rules, Options { line_endings: on(LINE_ENDINGS.0, false) })
    }

    /// Compares the panes again when the Importance tab changed since they were compared.
    pub(crate) fn recompare_if_rules_changed(&mut self) {
        if self.aligned.importance_rules() != self.importance() {
            self.compare_as_they_stand();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use otd_text_engine::Outcome;

    use super::*;

    /// The overall result of comparing `left` with `right` with the element `id` set to `on`.
    fn outcome(left: &str, right: &str, id: &str, on: bool) -> Option<Outcome> {
        let folder = tempfile::tempdir().unwrap();
        let mut view = TextCompareView::new();
        view.session.specs = [("left.txt", left), ("right.txt", right)]
            .map(|(name, text)| {
                let path = folder.path().join(name);
                fs::write(&path, text).unwrap();
                path.display().to_string()
            })
            .to_vec();
        view.reload_if_changed();
        view.session.rules.entry(IMPORTANCE_TAB.to_owned()).or_default().insert(id.to_owned(), Value::Bool(on));
        view.recompare_if_rules_changed();
        view.aligned.result().map(|(outcome, _)| outcome)
    }

    #[test]
    fn each_checkbox_decides_its_own_element() {
        let pairs = [("  a\n", "a\n"), ("a  b\n", "a b\n"), ("a  \n", "a\n"), ("a\n", "b\n"), ("A\n", "a\n")];
        for ((id, _), (left, right)) in ELEMENTS.iter().zip(pairs) {
            assert_eq!(outcome(left, right, id, true), Some(Outcome::Important), "{id} ticked: {left:?} vs {right:?} matters");
            assert_eq!(outcome(left, right, id, false), Some(Outcome::UnimportantOnly), "{id} unticked: it doesn't");
        }
        let id = LINE_ENDINGS.0;
        assert_eq!(outcome("a\r\n", "a\n", id, false), Some(Outcome::Same), "line endings are ignored when off");
        assert_eq!(outcome("a\r\n", "a\n", id, true), Some(Outcome::Important), "and compared when on");
    }
}
