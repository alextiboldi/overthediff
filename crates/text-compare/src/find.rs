//! Finding text in both panes (E07-S5-T1, catalogue F-12.5): the matcher behind Find, Find Next,
//! Find Previous, Select All and Replace. The find panel and its commands are E07-S5-T2 to T5.
//!
//! The text to find is plain text or an A.2 regular expression (`otd-regex`), matched line by
//! line, ignoring case unless Match character case is on. With Whole words only, a match has no
//! word character (`\w`) just before or after it.
//!
//! The searched sides form one sequence: the left pane top to bottom, then the right pane top to
//! bottom (*inferred*: the catalogue doesn't say how a search crosses from one pane to the other).
//! A forward search starts at the end of the selection and a backward one ends before its start,
//! so the match just found and selected isn't found again.

use std::ops::Range;

use otd_regex::{Regex, RegexError};
use otd_text_engine::{Line, Side};

use crate::{Pos, Selection};

/// Which panes a search looks in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Sides {
    Left,
    Right,
    #[default]
    Both,
}

impl Sides {
    fn includes(self, side: Side) -> bool {
        matches!((self, side), (Self::Both, _) | (Self::Left, Side::Left) | (Self::Right, Side::Right))
    }
}

/// The find panel's options.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FindOptions {
    /// The text is an A.2 regular expression, not plain text.
    pub regex: bool,
    pub match_case: bool,
    pub whole_words: bool,
    /// A search that runs off the end (or the start) goes on from the other end.
    pub wrap: bool,
    pub sides: Sides,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Backward,
}

/// A match: on `side`, in `line`, the characters `columns` (counted in characters, as [`Pos`]).
/// A regular expression can match nothing, such as `^\s*$` on a blank line; then `columns` is
/// empty.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    pub side: Side,
    pub line: usize,
    pub columns: Range<usize>,
}

impl Found {
    /// The match as a selection, with the caret at its end.
    pub fn selection(&self) -> Selection {
        Selection { anchor: Pos { line: self.line, column: self.columns.start }, head: Pos { line: self.line, column: self.columns.end } }
    }

    fn start(&self) -> (Side, Pos) {
        (self.side, Pos { line: self.line, column: self.columns.start })
    }
}

/// The text to find, compiled with its options.
#[derive(Clone, Debug)]
pub struct Finder {
    regex: Regex,
    options: FindOptions,
}

impl Finder {
    /// Compiles `text` with `options`. An invalid regular expression is an error, and nothing is
    /// searched; plain text always compiles.
    pub fn new(text: &str, options: FindOptions) -> Result<Self, RegexError> {
        let mut pattern = if options.match_case { String::new() } else { "(?i)".to_owned() };
        if options.regex {
            pattern.push_str(text);
        } else {
            // Every character that isn't a letter or digit is escaped, so A.2 reads it literally.
            for c in text.chars() {
                if !c.is_alphanumeric() {
                    pattern.push('\\');
                }
                pattern.push(c);
            }
        }
        let regex = if options.whole_words { Regex::whole_words(&pattern) } else { Regex::new(&pattern) }?;
        Ok(Self { regex, options })
    }

    /// Every match on the searched sides of `panes` (left, right), in search order.
    // ponytail: every search scans both panes in full; stop at the first match when large files
    // make Find Next slow.
    pub fn find_all(&self, panes: [&[Line]; 2]) -> Result<Vec<Found>, RegexError> {
        let mut all = Vec::new();
        for (side, lines) in [Side::Left, Side::Right].into_iter().zip(panes) {
            if !self.options.sides.includes(side) {
                continue;
            }
            for (line, Line { text, .. }) in lines.iter().enumerate() {
                for found in self.regex.matches(text) {
                    let bytes = found?.range();
                    all.push(Found { side, line, columns: column(text, bytes.start)..column(text, bytes.end) });
                }
            }
        }
        Ok(all)
    }

    /// The next match in `direction` from `selection` on `side`: after its end going forward,
    /// before its start going backward. From a side that isn't searched, a search starts at the
    /// first (or last) match. With Wrap search off, `None` once the matches run out.
    pub fn find(&self, panes: [&[Line]; 2], side: Side, selection: Selection, direction: Direction) -> Result<Option<Found>, RegexError> {
        let all = self.find_all(panes)?;
        let (start, end) = selection.range();
        let searched = self.options.sides.includes(side);
        let next = match direction {
            // A match that selects nothing at the selection's end is the one just found.
            Direction::Forward => {
                all.iter().find(|found| !searched || found.start() > (side, end) || (found.start() == (side, end) && !found.columns.is_empty()))
            }
            Direction::Backward => all.iter().rev().find(|found| !searched || found.start() < (side, start)),
        };
        let wrapped = || match direction {
            Direction::Forward => all.first(),
            Direction::Backward => all.last(),
        };
        Ok(next.or_else(|| self.options.wrap.then(wrapped).flatten()).cloned())
    }
}

/// The character column of byte offset `at` in `text`. `at` is on a character boundary, as
/// regex matches are.
fn column(text: &str, at: usize) -> usize {
    text.get(..at).map_or(0, |before| before.chars().count())
}
