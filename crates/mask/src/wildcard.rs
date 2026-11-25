//! One wildcard matched against one file or folder name (feature catalogue A.1, story E02-S1).
//!
//! - `?` matches one character and `*` matches zero or more.
//! - `[az]` is a set, `[a-z]` a range, `[!az]` a negated set and `[[]` a literal `[`. A `-` at the
//!   start or end of a set is literal.
//! - `*.*` also matches names without an extension.
//! - A mask ending in `.` matches only names without an extension: the final `.` is dropped, and
//!   neither `?` nor `*` matches a period. So `*.` matches names that contain no period.
//!
//! Splitting a mask list on `;`, the `-` exclude prefix and `\` folder parts belong to the mask list,
//! not here. Case is compared exactly; the story leaves case folding unspecified.

use std::fmt;

/// A compiled wildcard. Build it once with [`Wildcard::new`] and match many names with it.
#[derive(Clone, Debug)]
pub struct Wildcard {
    tokens: Vec<Token>,
    /// Set by a final `.`: `?` and `*` then don't match a period.
    no_extension: bool,
}

#[derive(Clone, Debug)]
enum Token {
    Char(char),
    /// `?`
    One,
    /// `*`
    Many,
    /// `[...]`: inclusive ranges, a single character being a range of one.
    Set {
        negated: bool,
        ranges: Vec<(char, char)>,
    },
}

/// Why a mask doesn't compile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WildcardError {
    /// A `[` with no `]` after it.
    UnclosedSet,
    /// `[]` or `[!]`: a set with nothing in it.
    EmptySet,
}

impl fmt::Display for WildcardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UnclosedSet => "a `[` in the mask has no closing `]`",
            Self::EmptySet => "a `[]` set in the mask is empty",
        })
    }
}

impl std::error::Error for WildcardError {}

impl Wildcard {
    /// Compiles `mask`, one name's worth of wildcard with no `;`, `-` or `\` syntax.
    pub fn new(mask: &str) -> Result<Self, WildcardError> {
        // `*.*` matches every name, with or without an extension, so it is `*`.
        if mask == "*.*" {
            return Ok(Self { tokens: vec![Token::Many], no_extension: false });
        }
        let (body, no_extension) = match mask.strip_suffix('.') {
            Some(body) => (body, true),
            None => (mask, false),
        };
        let mut tokens = Vec::new();
        let mut chars = body.chars();
        while let Some(c) = chars.next() {
            tokens.push(match c {
                '?' => Token::One,
                '*' => Token::Many,
                '[' => parse_set(&mut chars)?,
                c => Token::Char(c),
            });
        }
        Ok(Self { tokens, no_extension })
    }

    /// Whether `name`, a single path component, matches the whole mask.
    pub fn matches(&self, name: &str) -> bool {
        let name: Vec<char> = name.chars().collect();
        // ponytail: a (tokens x characters) table, O(mask length x name length). Not single-star
        // backtracking, which is wrong once `*` can't cross a period that a set can match.
        // `row[j]`: the tokens so far match the first `j` characters of the name.
        let mut row = vec![false; name.len() + 1];
        if let Some(empty) = row.first_mut() {
            *empty = true;
        }
        for token in &self.tokens {
            row = self.step(token, &name, &row);
            if !row.contains(&true) {
                return false;
            }
        }
        row.last().copied().unwrap_or(false)
    }

    /// The next row of the table: `row` advanced over `token`.
    fn step(&self, token: &Token, name: &[char], row: &[bool]) -> Vec<bool> {
        let mut next = Vec::with_capacity(row.len());
        next.push(matches!(token, Token::Many) && row.first().copied().unwrap_or(false));
        for (&c, (&before, &here)) in name.iter().zip(row.iter().zip(row.iter().skip(1))) {
            let matched = match token {
                // `*` matches nothing here, or one more character after its match so far.
                Token::Many => here || (next.last().copied().unwrap_or(false) && self.wild(c)),
                Token::One => before && self.wild(c),
                Token::Char(want) => before && *want == c,
                Token::Set { negated, ranges } => before && ranges.iter().any(|&(lo, hi)| (lo..=hi).contains(&c)) != *negated,
            };
            next.push(matched);
        }
        next
    }

    /// Whether `?` or `*` may match `c`.
    fn wild(&self, c: char) -> bool {
        !(self.no_extension && c == '.')
    }
}

/// Parses a set after its `[`, up to and including its `]`.
fn parse_set(chars: &mut std::str::Chars<'_>) -> Result<Token, WildcardError> {
    let mut items = Vec::new();
    loop {
        match chars.next() {
            None => return Err(WildcardError::UnclosedSet),
            Some(']') => break,
            Some(c) => items.push(c),
        }
    }
    let negated = items.first() == Some(&'!');
    let mut rest = if negated { items.get(1..).unwrap_or_default() } else { &items[..] };
    if rest.is_empty() {
        return Err(WildcardError::EmptySet);
    }
    let mut ranges = Vec::new();
    while let Some((&lo, tail)) = rest.split_first() {
        if let ['-', hi, after @ ..] = tail {
            ranges.push((lo, *hi));
            rest = after;
        } else {
            ranges.push((lo, lo));
            rest = tail;
        }
    }
    Ok(Token::Set { negated, ranges })
}
