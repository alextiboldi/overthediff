//! The built-in elements of the Importance tab: leading, embedded and trailing whitespace, and
//! everything else, with the letter case of everything else.
//!
//! Whitespace is what [`char::is_whitespace`] says it is. Leading whitespace comes before a line's
//! first other character, trailing whitespace after its last, and embedded whitespace in between.
//! A line holding only whitespace has no first or last other character, so its whitespace is both
//! leading and trailing (*inferred*): it is unimportant when either element is unticked.

use std::ops::Range;

use crate::Rule;

/// Which built-in elements are important. Each flag is that element's tick: `true` makes its
/// differences important, `false` unimportant. The default ticks everything, so every difference
/// matters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuiltInRules {
    pub leading_whitespace: bool,
    pub embedded_whitespace: bool,
    pub trailing_whitespace: bool,
    /// Every character that isn't whitespace.
    pub everything_else: bool,
    /// Whether case differences in everything else are important.
    pub character_case: bool,
}

impl Default for BuiltInRules {
    fn default() -> Self {
        Self { leading_whitespace: true, embedded_whitespace: true, trailing_whitespace: true, everything_else: true, character_case: true }
    }
}

impl Rule for BuiltInRules {
    fn unimportant(&self, text: &str) -> Vec<Range<usize>> {
        let ends = text.find(|c: char| !c.is_whitespace()).zip(text.rfind(|c: char| !c.is_whitespace()));
        text.char_indices()
            .filter(|&(at, c)| {
                let important = match (c.is_whitespace(), ends) {
                    (false, _) => self.everything_else,
                    (true, None) => self.leading_whitespace && self.trailing_whitespace,
                    (true, Some((first, _))) if at < first => self.leading_whitespace,
                    (true, Some((_, last))) if at > last => self.trailing_whitespace,
                    (true, Some(_)) => self.embedded_whitespace,
                };
                !important
            })
            .map(|(at, c)| at..at + c.len_utf8())
            .collect()
    }

    fn case_insensitive(&self, text: &str) -> Vec<Range<usize>> {
        // Whitespace has no case, so the whole line stands for everything else.
        if self.character_case { Vec::new() } else { vec![Range { start: 0, end: text.len() }] }
    }
}
