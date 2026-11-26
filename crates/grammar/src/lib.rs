//! A text format's grammar (feature catalogue F-21.4, story E10-S2): an ordered list of items, each
//! marking text as one of the format's named elements, such as Comment, String or Keyword. UI-free:
//! the Text Compare view paints the [`Span`]s, and the importance rules read them.
//!
//! Build a [`Grammar`] from its [`Element`]s and [`Item`]s with [`Grammar::new`], then mark a text's
//! lines with [`Grammar::highlight`]. Patterns are literal text, or A.2 regular expressions
//! (`otd-regex`) when the item's `regex` flag is set; they match within a line.
//!
//! - **Precedence** (*inferred*): the text is read from the start, and the item whose match starts
//!   first takes the text; where several start at the same place, the highest item wins. A match
//!   holds its text until it ends, so a string's `//` starts no comment and a comment's quote
//!   starts no string. Columns and Lines items mark positions rather than tokens, so they give way
//!   where a higher item's match starts inside them; a lower match that starts before a column
//!   range still holds its text into it.
//! - **Case**: an item matches case-sensitively only when its `match_case` flag is set and its
//!   element is case sensitive, so a case-insensitive element makes all its items ignore case
//!   (*inferred*).
//! - **List** tokens match whole words (*inferred*): a token's end that is a letter, digit or `_`
//!   must not touch another one, so `for` doesn't match in `format`, and `int` wins over `in`.
//!   Where two tokens match at the same place, the longer wins.
//! - **Delimited** items with an end delimiter can run across lines, until the end or the end of
//!   the text. An unescaped end delimiter ends them; the escape character makes the next
//!   character plain text. With *Stop at end of line*, or with no end delimiter, they end with
//!   their line at the latest.
//! - **Columns** count characters from 1, both ends included; a tab is one character
//!   (*inferred*).
//! - **Lines** (*inferred*): a line containing the trigger text, or the text's first line, is the
//!   first of the header's heading lines, and the header covers each of those lines whole.
//! - No nesting and no context-specific rules: one grammar marks the whole text.
//!
//! Not here: editing grammars (E10-S3), the built-in grammars (E10-S2-T3) and element colours
//! (E10-S2-T4).

use std::fmt;
use std::ops::Range;

use otd_regex::{Regex, RegexError};

/// A named kind of text, such as Comment, that a grammar's items mark.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Element {
    pub name: String,
    /// This element is case sensitive.
    pub case_sensitive: bool,
}

/// One grammar item: what it matches, and the element it marks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    /// The [`Element::name`] the item marks.
    pub element: String,
    pub category: Category,
    /// Match character case. Columns items ignore it.
    pub match_case: bool,
    /// The item's patterns are A.2 regular expressions rather than literal text. Columns items
    /// ignore it.
    pub regex: bool,
}

/// The five kinds of grammar item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Category {
    /// Every occurrence of `text`.
    Basic { text: String },
    /// From `start` to `end`. An empty `end` means the end of the line.
    Delimited { start: String, end: String, stop_at_end_of_line: bool, escape: Option<char> },
    /// Any of `tokens`, such as keywords, as whole words.
    List { tokens: Vec<String> },
    /// Character columns `start` to `end`, counted from 1 with both included. `None` is the end
    /// of the line.
    Columns { start: usize, end: Option<usize> },
    /// Repeating page headers: `heading_lines` lines, starting with each line the trigger matches.
    Lines { trigger: Trigger, heading_lines: usize },
}

/// What starts a [`Category::Lines`] header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Trigger {
    /// A line that contains this text, typically a form feed.
    Text(String),
    /// The text's first line.
    FirstLine,
}

/// Text that an item marked: a byte range of one line, and the index of its element in
/// [`Grammar::elements`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub range: Range<usize>,
    pub element: usize,
}

/// Why [`Grammar::new`] refused an item. `item` is its index in the list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GrammarError {
    /// The item names an element the grammar doesn't have.
    UnknownElement { item: usize },
    /// A Columns item starts at column 0 or ends before it starts.
    Columns { item: usize },
    /// One of the item's patterns isn't a valid regular expression.
    Regex { item: usize, error: RegexError },
}

impl fmt::Display for GrammarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownElement { item } => write!(f, "grammar item {} names an element the grammar doesn't have", item + 1),
            Self::Columns { item } => write!(f, "grammar item {} needs columns that start at 1 or later and don't end before they start", item + 1),
            Self::Regex { item, error } => write!(f, "grammar item {}: {error}", item + 1),
        }
    }
}

impl std::error::Error for GrammarError {}

/// A compiled grammar.
#[derive(Clone, Debug)]
pub struct Grammar {
    elements: Vec<Element>,
    items: Vec<Compiled>,
}

#[derive(Clone, Debug)]
struct Compiled {
    element: usize,
    matcher: Matcher,
}

#[derive(Clone, Debug)]
enum Matcher {
    Basic(Regex),
    /// `end` is `None` when the item ends at the end of the line.
    Delimited {
        start: Regex,
        end: Option<Regex>,
        stop_at_end_of_line: bool,
        escape: Option<char>,
    },
    List(Vec<Regex>),
    /// Character indexes from 0; `end` is exclusive.
    Columns {
        start: usize,
        end: Option<usize>,
    },
    /// `trigger` is `None` for the first line.
    Lines {
        trigger: Option<Regex>,
        heading_lines: usize,
    },
}

/// Compiles one of an item's patterns, literal or not, with or without case.
fn pattern(text: &str, regex: bool, case_sensitive: bool) -> Result<Regex, RegexError> {
    let body = if regex { text.to_owned() } else { text.chars().map(|c| format!("\\x{{{:X}}}", u32::from(c))).collect() };
    Regex::new(&if case_sensitive { body } else { format!("(?i){body}") })
}

impl Grammar {
    pub fn new(elements: Vec<Element>, items: Vec<Item>) -> Result<Self, GrammarError> {
        let items = items.into_iter().enumerate().map(|(n, item)| compile(&elements, n, item)).collect::<Result<_, _>>()?;
        Ok(Self { elements, items })
    }

    pub fn elements(&self) -> &[Element] {
        &self.elements
    }

    /// The spans of each of `lines` (each without its break), in order and not overlapping. A
    /// regular expression that backtracks too much on the text ends it with
    /// [`RegexError::TooComplex`].
    pub fn highlight<'a>(&self, lines: impl IntoIterator<Item = &'a str>) -> Result<Vec<Vec<Span>>, RegexError> {
        let mut state = State { open: None, heading_left: vec![0; self.items.len()] };
        lines.into_iter().enumerate().map(|(n, line)| self.line(&mut state, n, line)).collect()
    }

    fn line(&self, state: &mut State, n: usize, line: &str) -> Result<Vec<Span>, RegexError> {
        for (item, left) in self.items.iter().zip(&mut state.heading_left) {
            if let Matcher::Lines { trigger, heading_lines } = &item.matcher {
                let starts = match trigger {
                    None => n == 0,
                    Some(trigger) => find(trigger, line, 0)?.is_some(),
                };
                if starts {
                    *left = *heading_lines;
                }
            }
        }
        let mut spans = Vec::new();
        let mut at = 0;
        if let Some(open) = state.open.take() {
            at = self.extend(state, open, line, 0)?;
            if at > 0 {
                spans.push(Span { range: 0..at, element: self.items[open].element });
            }
        }
        // Each item's first match at or after `at`, kept while `at` hasn't passed its start.
        let mut next: Vec<Option<Option<Range<usize>>>> = vec![None; self.items.len()];
        while at < line.len() {
            let mut best: Option<(usize, Range<usize>)> = None;
            for (index, cached) in next.iter_mut().enumerate() {
                let found = match cached {
                    Some(found) if found.as_ref().is_none_or(|r| r.start >= at) => found.clone(),
                    _ => cached.insert(self.candidate(state, index, line, at)?).clone(),
                };
                if let Some(range) = found.filter(|r| best.as_ref().is_none_or(|(_, b)| r.start < b.start)) {
                    best = Some((index, range));
                }
            }
            let Some((index, range)) = best else { break };
            let end = match self.items[index].matcher {
                Matcher::Delimited { .. } => self.extend(state, index, line, range.end)?,
                // A column range or header isn't a token: it gives way where a higher item's match
                // starts inside it, and goes on after that match.
                Matcher::Columns { .. } | Matcher::Lines { .. } => {
                    next[..index].iter().flatten().flatten().map(|r| r.start).filter(|&start| start < range.end).min().unwrap_or(range.end)
                }
                _ => range.end,
            };
            spans.push(Span { range: range.start..end, element: self.items[index].element });
            at = end;
        }
        for left in &mut state.heading_left {
            *left = left.saturating_sub(1);
        }
        Ok(spans)
    }

    /// Where item `index`'s match, a non-empty range, would start at or after `at` in `line`.
    fn candidate(&self, state: &State, index: usize, line: &str, at: usize) -> Result<Option<Range<usize>>, RegexError> {
        Ok(match &self.items[index].matcher {
            Matcher::Basic(regex) | Matcher::Delimited { start: regex, .. } => find(regex, line, at)?,
            // ponytail: every token searches again once `at` passes its last find, so a long line
            // with many tokens costs tokens × matches searches; keep each token's find if that
            // shows up in large files.
            Matcher::List(tokens) => {
                let mut best: Option<Range<usize>> = None;
                for token in tokens {
                    if let Some(r) = find_word(token, line, at)?.filter(|r| best.as_ref().is_none_or(|b| (r.start, b.end) < (b.start, r.end))) {
                        best = Some(r);
                    }
                }
                best
            }
            Matcher::Columns { start, end } => {
                let range = byte_of(line, *start).max(at)..end.map_or(line.len(), |end| byte_of(line, end));
                (!range.is_empty()).then_some(range)
            }
            Matcher::Lines { .. } => (state.heading_left[index] > 0).then_some(at..line.len()),
        })
    }

    /// Where the Delimited item `index`, whose start delimiter ends at `from`, ends in `line`.
    /// When it runs on past the line, [`State::open`] carries it to the next one.
    fn extend(&self, state: &mut State, index: usize, line: &str, from: usize) -> Result<usize, RegexError> {
        let Matcher::Delimited { end, stop_at_end_of_line, escape, .. } = &self.items[index].matcher else { return Ok(line.len()) };
        let Some(end) = end else { return Ok(line.len()) };
        let mut search = from;
        // Escapes are resolved up to `scan`, which never stops on an escaped character.
        let mut scan = from;
        while let Some(found) = find(end, line, search)? {
            while scan < found.start {
                let c = char_at(line, scan);
                scan += c.len_utf8();
                if Some(c) == *escape && scan < line.len() {
                    scan += char_at(line, scan).len_utf8();
                }
            }
            if scan == found.start {
                return Ok(found.end);
            }
            search = found.start + char_at(line, found.start).len_utf8();
        }
        if !stop_at_end_of_line {
            state.open = Some(index);
        }
        Ok(line.len())
    }
}

/// What carries from one line to the next.
struct State {
    /// A Delimited item whose end delimiter hasn't come yet.
    open: Option<usize>,
    /// For each Lines item, how many lines its header still covers, this one included.
    heading_left: Vec<usize>,
}

fn compile(elements: &[Element], n: usize, item: Item) -> Result<Compiled, GrammarError> {
    let element = elements.iter().position(|e| e.name == item.element).ok_or(GrammarError::UnknownElement { item: n })?;
    let case = item.match_case && elements[element].case_sensitive;
    let compile = |text: &str| pattern(text, item.regex, case).map_err(|error| GrammarError::Regex { item: n, error });
    let matcher = match item.category {
        Category::Basic { text } => Matcher::Basic(compile(&text)?),
        Category::Delimited { start, end, stop_at_end_of_line, escape } => {
            Matcher::Delimited { start: compile(&start)?, end: if end.is_empty() { None } else { Some(compile(&end)?) }, stop_at_end_of_line, escape }
        }
        Category::List { tokens } => Matcher::List(tokens.iter().map(|t| compile(t)).collect::<Result<_, _>>()?),
        Category::Columns { start, end } => {
            if start == 0 || end.is_some_and(|end| end < start) {
                return Err(GrammarError::Columns { item: n });
            }
            Matcher::Columns { start: start - 1, end }
        }
        Category::Lines { trigger, heading_lines } => Matcher::Lines {
            trigger: match trigger {
                Trigger::Text(text) => Some(compile(&text)?),
                Trigger::FirstLine => None,
            },
            heading_lines,
        },
    };
    Ok(Compiled { element, matcher })
}

/// The first non-empty match of `regex` in `line` at or after `from`.
fn find(regex: &Regex, line: &str, mut from: usize) -> Result<Option<Range<usize>>, RegexError> {
    while from <= line.len() {
        match regex.find_at(line, from)? {
            Some(found) if found.is_empty() => from = found.start + line[found.start..].chars().next().map_or(1, char::len_utf8),
            found => return Ok(found),
        }
    }
    Ok(None)
}

/// The first match of `token` at or after `from` that is a whole word: a word character at either
/// end doesn't touch another.
fn find_word(token: &Regex, line: &str, mut from: usize) -> Result<Option<Range<usize>>, RegexError> {
    let word = |c: Option<char>| c.is_some_and(|c| c.is_alphanumeric() || c == '_');
    while let Some(found) = find(token, line, from)? {
        let (before, inside, after) = (&line[..found.start], &line[found.clone()], &line[found.end..]);
        let joined_start = word(inside.chars().next()) && word(before.chars().next_back());
        let joined_end = word(inside.chars().next_back()) && word(after.chars().next());
        if !joined_start && !joined_end {
            return Ok(Some(found));
        }
        from = found.start + char_at(line, found.start).len_utf8();
    }
    Ok(None)
}

/// The character at byte `at`, which is a boundary before the line's end.
fn char_at(line: &str, at: usize) -> char {
    line[at..].chars().next().unwrap_or_default()
}

/// The byte offset of character `index` of `line`, or the line's length past its end.
fn byte_of(line: &str, index: usize) -> usize {
    line.char_indices().nth(index).map_or(line.len(), |(byte, _)| byte)
}
