//! Regular expressions in the feature catalogue's A.2 syntax, matched line by line.
//!
//! The syntax:
//! - Escapes: `\xnn`, `\x{nnnn}`, `\t`, `\n`, `\r`, `\f`, `\a`, `\e`, and `\` before any other
//!   character that isn't a letter or digit stands for that character.
//! - Classes: `.`, `\w`, `\W`, `\d`, `\D`, `\s` (space and tab), `\S`, and bracket classes with
//!   ranges, `[^…]` negation, a literal `-` at the start or end, and predefined classes inside.
//! - Alternation `|`, which tries the leftmost alternative first.
//! - Anchors `^` and `$`, and `\A` and `\Z`, which mean the same.
//! - Greedy quantifiers `*`, `+`, `?`, `{n}`, `{n,}`, `{n,m}`, each made lazy by a following `?`.
//! - Groups `( )`, backreferences `\1`–`\9`, and `(?i)`/`(?-i)`, which turn case-insensitive
//!   matching on and off up to the end of the enclosing group.
//!
//! Anything else is a [`RegexError::Syntax`]. To match without regard to case, put `(?i)` in
//! front of the pattern.
//!
//! A text is matched one line at a time: lines end at `\n`, `\r\n` or `\r`, and no match spans a
//! line break, so a pattern containing `\n` never matches.

use std::fmt;
use std::ops::Range;

mod syntax;

/// A compiled A.2 pattern.
#[derive(Debug, Clone)]
pub struct Regex {
    inner: fancy_regex::Regex,
}

/// One match, with byte ranges into the text that was searched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    range: Range<usize>,
    groups: Vec<Option<Range<usize>>>,
}

/// Why a pattern can't be compiled or a match can't be finished.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegexError {
    /// The pattern isn't A.2 syntax. `at` is the byte offset in the pattern where the problem
    /// starts; `problem` is English text for the user.
    Syntax { at: usize, problem: &'static str },
    /// The pattern nests too deeply, compiles too large, or backtracks too much on the text.
    TooComplex,
}

impl fmt::Display for RegexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax { at, problem } => write!(f, "{problem} (at offset {at})"),
            Self::TooComplex => f.write_str("the regular expression is too complex"),
        }
    }
}

impl std::error::Error for RegexError {}

impl From<fancy_regex::Error> for RegexError {
    // ponytail: the translated pattern is already valid syntax, so fancy-regex can only refuse
    // it for size, nesting or backtracking.
    fn from(_: fancy_regex::Error) -> Self {
        Self::TooComplex
    }
}

impl Match {
    /// The whole match.
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }

    /// Group `n` (1–9), or `None` when the pattern has no such group or it took no part in the
    /// match.
    pub fn group(&self, n: usize) -> Option<Range<usize>> {
        self.groups.get(n.checked_sub(1)?).cloned().flatten()
    }
}

impl Regex {
    pub fn new(pattern: &str) -> Result<Self, RegexError> {
        Ok(Self { inner: fancy_regex::Regex::new(&syntax::translate(pattern)?)? })
    }

    /// Every non-overlapping match in `text`, line by line, in order.
    pub fn matches<'a>(&'a self, text: &'a str) -> impl Iterator<Item = Result<Match, RegexError>> + 'a {
        lines(text).flat_map(move |(start, line)| {
            self.inner.captures_iter(line).map(move |found| {
                let found = found?;
                let shift = |m: fancy_regex::Match<'_>| m.start() + start..m.end() + start;
                Ok(Match { range: found.get(0).map(shift).unwrap_or_default(), groups: found.iter().skip(1).map(|m| m.map(shift)).collect() })
            })
        })
    }

    pub fn is_match(&self, text: &str) -> Result<bool, RegexError> {
        self.matches(text).next().transpose().map(|found| found.is_some())
    }
}

/// The lines of `text` with their byte offsets. A break at the very end starts no further line,
/// but an empty text is one empty line.
fn lines(text: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut next = Some(0);
    std::iter::from_fn(move || {
        let start = next?;
        let rest = &text[start..];
        let Some(end) = rest.find(['\n', '\r']) else {
            next = None;
            return (start == 0 || !rest.is_empty()).then_some((start, rest));
        };
        let after = start + end + if rest[end..].starts_with("\r\n") { 2 } else { 1 };
        next = Some(after);
        Some((start, &rest[..end]))
    })
}

#[cfg(test)]
mod tests {
    use super::lines;

    #[test]
    fn every_line_break_style_ends_a_line_and_a_final_break_starts_none() {
        let found: Vec<_> = lines("a\nb\r\nc\rd\n").map(|(_, line)| line).collect();
        assert_eq!(found, ["a", "b", "c", "d"]);
        assert_eq!(lines("").count(), 1, "an empty text is one line");
        assert_eq!(lines("\n\n").count(), 2, "blank lines in the middle stay");
    }
}
