//! Reads an A.2 pattern and writes the same pattern in fancy-regex syntax, rejecting anything
//! A.2 doesn't define.
//!
//! Every literal character is written as `\x{…}`, so no character can take on a meaning the
//! fancy-regex syntax gives it and A.2 doesn't (`#`, `&&`, `~~`, a digit after a backreference).

use std::fmt::Write as _;

use crate::RegexError;

pub(crate) fn translate(pattern: &str) -> Result<String, RegexError> {
    Reader { pattern, chars: pattern.char_indices().collect(), pos: 0, out: String::new() }.pattern()
}

/// What one escape sequence stands for.
enum Escape {
    Char(char),
    /// A predefined class, as written outside and inside a bracket class.
    Class {
        outside: &'static str,
        inside: &'static str,
    },
    /// `\A` and `\Z`: matching is line by line, so they are `^` and `$`.
    Anchor(&'static str),
    Backref(u32),
}

/// A group the reader is inside.
struct Frame {
    /// Whether case was ignored where the group opened.
    entry_case: bool,
    /// The case switches opened in the group's current alternative and not yet closed.
    switches: usize,
}

struct Reader<'a> {
    pattern: &'a str,
    chars: Vec<(usize, char)>,
    pos: usize,
    out: String,
}

fn syntax(at: usize, problem: &'static str) -> RegexError {
    RegexError::Syntax { at, problem }
}

impl Reader<'_> {
    fn peek_at(&self, ahead: usize) -> Option<char> {
        self.chars.get(self.pos + ahead).map(|&(_, c)| c)
    }

    fn peek(&self) -> Option<char> {
        self.peek_at(0)
    }

    /// The byte offset of the next character, or the pattern's length at its end.
    fn at(&self) -> usize {
        self.chars.get(self.pos).map_or(self.pattern.len(), |&(i, _)| i)
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += 1;
        Some(c)
    }

    fn eat(&mut self, c: char) -> bool {
        let found = self.peek() == Some(c);
        if found {
            self.pos += 1;
        }
        found
    }

    fn literal(&mut self, c: char) {
        let _ = write!(self.out, "\\x{{{:X}}}", u32::from(c));
    }

    fn pattern(mut self) -> Result<String, RegexError> {
        let mut groups = 0;
        // The groups open around the current position, innermost last; the top level is the first.
        let mut frames = vec![Frame { entry_case: false, switches: 0 }];
        let mut ignore_case = false;
        let mut backrefs = Vec::new();
        // Whether the last item can take a quantifier: a quantifier, anchor, `|`, `(` or flag
        // switch can't.
        let mut can_repeat = false;
        loop {
            let at = self.at();
            let Some(c) = self.bump() else { break };
            can_repeat = match c {
                '\\' => match self.escape(at)? {
                    Escape::Char(c) => {
                        self.literal(c);
                        true
                    }
                    Escape::Class { outside, .. } => {
                        self.out.push_str(outside);
                        true
                    }
                    Escape::Anchor(a) => {
                        self.out.push_str(a);
                        false
                    }
                    Escape::Backref(n) => {
                        backrefs.push((n, at));
                        let _ = write!(self.out, "\\{n}");
                        true
                    }
                },
                '.' => {
                    self.out.push('.');
                    true
                }
                '^' | '$' => {
                    self.out.push(c);
                    false
                }
                // The next alternative starts with the case setting the last one ended with.
                '|' => {
                    if let Some(frame) = frames.last_mut() {
                        self.close(frame.switches);
                        self.out.push('|');
                        frame.switches = usize::from(ignore_case != frame.entry_case);
                        if frame.switches > 0 {
                            self.switch(ignore_case);
                        }
                    }
                    false
                }
                '(' if self.eat('?') => {
                    let off = self.eat('-');
                    if !(self.eat('i') && self.eat(')')) {
                        return Err(syntax(at, "only (?i) and (?-i) may follow (?"));
                    }
                    ignore_case = !off;
                    self.switch(ignore_case);
                    if let Some(frame) = frames.last_mut() {
                        frame.switches += 1;
                    }
                    false
                }
                '(' => {
                    groups += 1;
                    frames.push(Frame { entry_case: ignore_case, switches: 0 });
                    self.out.push('(');
                    false
                }
                ')' => {
                    let frame = frames.pop().filter(|_| !frames.is_empty());
                    let frame = frame.ok_or_else(|| syntax(at, "a ) has no matching ("))?;
                    self.close(frame.switches);
                    self.out.push(')');
                    ignore_case = frame.entry_case;
                    true
                }
                '[' => {
                    self.class(at)?;
                    true
                }
                '*' | '+' | '?' => {
                    if !can_repeat {
                        return Err(syntax(at, "a quantifier has nothing to repeat"));
                    }
                    self.out.push(c);
                    self.lazy();
                    false
                }
                '{' => match self.bounds(at)? {
                    Some(bounds) => {
                        if !can_repeat {
                            return Err(syntax(at, "a quantifier has nothing to repeat"));
                        }
                        self.out.push_str(&bounds);
                        self.lazy();
                        false
                    }
                    // ponytail: a `{` that starts no `{n}`, `{n,}` or `{n,m}` is a literal (*inferred*).
                    None => {
                        self.literal('{');
                        true
                    }
                },
                c => {
                    self.literal(c);
                    true
                }
            };
        }
        match frames.as_slice() {
            [top] => self.close(top.switches),
            _ => return Err(syntax(self.pattern.len(), "a ( has no matching )")),
        }
        if let Some(&(_, at)) = backrefs.iter().find(|&&(n, _)| n > groups) {
            return Err(syntax(at, "a backreference names a group the pattern doesn't have"));
        }
        Ok(self.out)
    }

    /// Opens a group that sets the case. fancy-regex lets a bare `(?i)` reach past the end of its
    /// group, so each switch becomes a `(?i:` group closed where A.2's switch ends: at the next
    /// `|` or `)` of its own group, or at the end of the pattern.
    fn switch(&mut self, ignore_case: bool) {
        self.out.push_str(if ignore_case { "(?i:" } else { "(?-i:" });
    }

    fn close(&mut self, switches: usize) {
        self.out.extend(std::iter::repeat_n(')', switches));
    }

    /// A `?` after a quantifier makes it lazy.
    fn lazy(&mut self) {
        if self.eat('?') {
            self.out.push('?');
        }
    }

    /// Reads the escape after the `\` at `at`.
    fn escape(&mut self, at: usize) -> Result<Escape, RegexError> {
        let c = self.bump().ok_or_else(|| syntax(at, "the pattern ends with \\"))?;
        let class = |outside, inside| Escape::Class { outside, inside };
        Ok(match c {
            't' => Escape::Char('\t'),
            'n' => Escape::Char('\n'),
            'r' => Escape::Char('\r'),
            'f' => Escape::Char('\x0C'),
            'a' => Escape::Char('\x07'),
            'e' => Escape::Char('\x1B'),
            'x' => Escape::Char(self.hex(at)?),
            // ponytail: \w is Unicode letters, digits and `_`; \d is ASCII digits (*inferred*).
            'w' => class(r"\w", r"\w"),
            'W' => class(r"\W", r"\W"),
            'd' => class("[0-9]", "0-9"),
            'D' => class("[^0-9]", "[^0-9]"),
            's' => class("[ \t]", " \t"),
            'S' => class("[^ \t]", "[^ \t]"),
            'A' => Escape::Anchor("^"),
            'Z' => Escape::Anchor("$"),
            '1'..='9' => Escape::Backref(u32::from(c) - u32::from('0')),
            c if c.is_alphanumeric() => return Err(syntax(at, "unknown escape")),
            c => Escape::Char(c),
        })
    }

    /// Reads the digits of `\xnn` or `\x{nnnn}` for the escape at `at`.
    fn hex(&mut self, at: usize) -> Result<char, RegexError> {
        let bad = || syntax(at, "\\x needs two hex digits, or hex digits in { }");
        let braced = self.eat('{');
        let mut digits = String::new();
        while let Some(c) = self.peek().filter(char::is_ascii_hexdigit) {
            if !braced && digits.len() == 2 {
                break;
            }
            digits.push(c);
            self.pos += 1;
        }
        let complete = if braced { self.eat('}') && !digits.is_empty() } else { digits.len() == 2 };
        if !complete {
            return Err(bad());
        }
        u32::from_str_radix(&digits, 16).ok().and_then(char::from_u32).ok_or_else(|| syntax(at, "\\x names no character"))
    }

    /// Reads `n}`, `n,}` or `n,m}` after a `{` at `at`. `None` means it isn't a quantifier, and
    /// the reader is back after the `{`.
    fn bounds(&mut self, at: usize) -> Result<Option<String>, RegexError> {
        let start = self.pos;
        let Some(min) = self.number(at)? else { return Ok(None) };
        let bounds = if self.eat('}') {
            Some(format!("{{{min}}}"))
        } else if self.eat(',') {
            if self.eat('}') {
                Some(format!("{{{min},}}"))
            } else {
                match self.number(at)? {
                    Some(max) if self.eat('}') => {
                        if max < min {
                            return Err(syntax(at, "a quantifier's maximum is below its minimum"));
                        }
                        Some(format!("{{{min},{max}}}"))
                    }
                    _ => None,
                }
            }
        } else {
            None
        };
        if bounds.is_none() {
            self.pos = start;
        }
        Ok(bounds)
    }

    fn number(&mut self, at: usize) -> Result<Option<u32>, RegexError> {
        let mut digits = String::new();
        while let Some(c) = self.peek().filter(char::is_ascii_digit) {
            digits.push(c);
            self.pos += 1;
        }
        if digits.is_empty() {
            return Ok(None);
        }
        digits.parse().map(Some).map_err(|_| syntax(at, "a quantifier's count is too large"))
    }

    /// Reads a bracket class after the `[` at `at`.
    fn class(&mut self, at: usize) -> Result<(), RegexError> {
        self.out.push('[');
        if self.eat('^') {
            self.out.push('^');
        }
        let mut empty = true;
        loop {
            let item_at = self.at();
            let c = self.bump().ok_or_else(|| syntax(at, "a [ has no matching ]"))?;
            let lo = match c {
                ']' if empty => return Err(syntax(at, "a [ ] class is empty")),
                ']' => break,
                '\\' => match self.escape(item_at)? {
                    Escape::Char(c) => c,
                    Escape::Class { inside, .. } => {
                        self.out.push_str(inside);
                        empty = false;
                        continue;
                    }
                    Escape::Anchor(_) | Escape::Backref(_) => {
                        return Err(syntax(item_at, "only characters and classes may go inside [ ]"));
                    }
                },
                c => c,
            };
            empty = false;
            self.literal(lo);
            // A `-` between two characters makes a range; at the start or end it is literal.
            if self.peek() != Some('-') || matches!(self.peek_at(1), None | Some(']')) {
                continue;
            }
            self.pos += 1;
            let hi_at = self.at();
            let hi = match self.bump() {
                Some('\\') => match self.escape(hi_at)? {
                    Escape::Char(c) => c,
                    _ => return Err(syntax(hi_at, "a range must end with a character")),
                },
                Some(c) => c,
                None => return Err(syntax(at, "a [ has no matching ]")),
            };
            if hi < lo {
                return Err(syntax(item_at, "a range ends before it starts"));
            }
            self.out.push('-');
            self.literal(hi);
        }
        self.out.push(']');
        Ok(())
    }
}
