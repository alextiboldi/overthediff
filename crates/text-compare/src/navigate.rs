//! Finding the next or previous difference from the cursor (E07-S2-T1), without a UI.
//!
//! A *section* is a maximal run of rows the view counts as different. A *change* is one stop of
//! *Next Difference*: one run of differing characters inside a changed row, paired across the two
//! sides, so an insertion or a deletion is one change whichever pane has the cursor. An orphan line,
//! or a changed row that differs only in its line ending, is a single change.
//!
//! Both finders go strictly past the cursor: from inside a section or a change, *next* gives the
//! following one and *previous* the preceding one, so at the last one there is no next (*inferred*).
//! A section that starts on the cursor's row counts as the one the cursor is in, so a caller that
//! has visited nothing yet (the cursor at the top of a file whose first row differs) lands on that
//! section itself rather than asking for the next.
//!
//! A caret is inside a change only strictly between its ends, or on it when it is empty. At a
//! change's start it is before the change and at its end after it, so *next* from the start of a
//! line finds a change there and *previous* from just after one finds it. A caller that lands on a
//! change passes its end to *next* and its start to *previous*. An empty change at the cursor (an
//! orphan line's gap, say) counts as the one the cursor is in; a caller that has visited nothing
//! yet finds it in [`changes`](Differences::changes).
//!
//! Which rows differ is the caller's to say, so navigation follows the view's current
//! classification: a row the view treats as same is skipped.

use std::ops::Range;

use otd_text_engine::{CharRange, Line, Row, Side, char_differences};

/// Where the cursor is: its row in the comparison, its pane, and its column in characters (as
/// [`Pos::column`](crate::Pos)). The row may be one past the last, for the caret after a final line
/// break.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cursor {
    pub row: usize,
    pub side: Side,
    pub column: usize,
}

/// One change inside row `row`: the columns, in characters, it covers on each side. A side that
/// lost or gained nothing has an empty range where the other side's text goes, and a gap has `0..0`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    pub row: usize,
    pub left: Range<usize>,
    pub right: Range<usize>,
}

impl Change {
    /// The columns the change covers on `side`.
    pub fn on(&self, side: Side) -> &Range<usize> {
        if side == Side::Left { &self.left } else { &self.right }
    }
}

/// What navigation walks: a comparison's rows, the lines they index, and which rows the view
/// counts as different.
pub struct Differences<'a, F: Fn(usize) -> bool> {
    pub rows: &'a [Row],
    pub left: &'a [Line],
    pub right: &'a [Line],
    /// Whether the view counts row `k` as different.
    pub differs: F,
}

impl<F: Fn(usize) -> bool> Differences<'_, F> {
    fn differs(&self, row: usize) -> bool {
        row < self.rows.len() && (self.differs)(row)
    }

    /// The section after the one the cursor's row `row` is in, as a range of rows.
    // ponytail: each call scans rows from the cursor, linear in the rows passed over; keep the
    // sections if a view ever navigates millions of rows per frame.
    pub fn next_section(&self, row: usize) -> Option<Range<usize>> {
        let mut k = row;
        while self.differs(k) {
            k += 1;
        }
        let start = (k..self.rows.len()).find(|&r| self.differs(r))?;
        Some(start..self.section_end(start))
    }

    /// The section before the one the cursor's row `row` is in, as a range of rows.
    pub fn previous_section(&self, row: usize) -> Option<Range<usize>> {
        let mut k = row.min(self.rows.len());
        while k > 0 && self.differs(k) && self.differs(k - 1) {
            k -= 1;
        }
        let last = (0..k).rev().find(|&r| self.differs(r))?;
        let start = (0..last).rev().find(|&r| !self.differs(r)).map_or(0, |r| r + 1);
        Some(start..last + 1)
    }

    fn section_end(&self, start: usize) -> usize {
        (start..self.rows.len()).find(|&r| !self.differs(r)).unwrap_or(self.rows.len())
    }

    /// The first change after the cursor: in a later row, or in its row starting at or right of it
    /// and not empty at it.
    pub fn next(&self, at: Cursor) -> Option<Change> {
        (at.row..self.rows.len()).filter(|&row| self.differs(row)).find_map(|row| {
            self.changes(row).into_iter().find(|c| {
                let r = c.on(at.side);
                row > at.row || (r.start >= at.column && r.end > at.column)
            })
        })
    }

    /// The last change before the cursor: in an earlier row, or in its row ending at or left of it
    /// and not empty at it.
    pub fn previous(&self, at: Cursor) -> Option<Change> {
        (0..self.rows.len().min(at.row.saturating_add(1))).rev().filter(|&row| self.differs(row)).find_map(|row| {
            self.changes(row).into_iter().rev().find(|c| {
                let r = c.on(at.side);
                row < at.row || (r.end <= at.column && r.start < at.column)
            })
        })
    }

    /// Row `row`'s changes, in order, whether or not the view counts the row as different.
    // ponytail: the row's characters are diffed again on each call; at most two rows per step
    // (the cursor's and the target's), so the view's cache isn't threaded through.
    pub fn changes(&self, row: usize) -> Vec<Change> {
        let Some(found) = self.rows.get(row) else { return Vec::new() };
        let left = found.left.and_then(|i| self.left.get(i)).map(|line| line.text.as_str());
        let right = found.right.and_then(|j| self.right.get(j)).map(|line| line.text.as_str());
        let whole = |text: Option<&str>| 0..text.map_or(0, |t| t.chars().count());
        match (left, right) {
            (Some(l), Some(r)) => {
                let changes = pair(row, l, r, &char_differences(l, r).ranges);
                if changes.is_empty() {
                    // Only the line endings differ: the change is where they are.
                    let (l, r) = (whole(left).end, whole(right).end);
                    return vec![Change { row, left: l..l, right: r..r }];
                }
                changes
            }
            (None, None) => Vec::new(),
            _ => vec![Change { row, left: whole(left), right: whole(right) }],
        }
    }
}

/// Pairs a changed row's left and right ranges into changes. The two lines are walked together:
/// the characters outside the ranges are the ones a shortest edit keeps, equal in number on both
/// sides, so a left and a right range at the same point are one change.
fn pair(row: usize, left: &str, right: &str, ranges: &[CharRange]) -> Vec<Change> {
    let mut ls = ranges.iter().filter(|r| r.side == Side::Left).peekable();
    let mut rs = ranges.iter().filter(|r| r.side == Side::Right).peekable();
    let (mut l, mut r) = (Walk { text: left, byte: 0, column: 0 }, Walk { text: right, byte: 0, column: 0 });
    let mut out = Vec::new();
    loop {
        let a = ls.next_if(|x| x.start == l.byte);
        let b = rs.next_if(|x| x.start == r.byte);
        if a.is_some() || b.is_some() {
            out.push(Change { row, left: l.take(a), right: r.take(b) });
        } else if !(l.step() && r.step()) {
            return out;
        }
    }
}

/// A place in a line, as a byte offset and a column in characters.
struct Walk<'a> {
    text: &'a str,
    byte: usize,
    column: usize,
}

impl Walk<'_> {
    /// Passes over `range`, if any, and gives the columns it covered.
    fn take(&mut self, range: Option<&CharRange>) -> Range<usize> {
        let start = self.column;
        if let Some(range) = range {
            self.column += self.text.get(range.start..range.end).map_or(0, |s| s.chars().count());
            self.byte = range.end;
        }
        start..self.column
    }

    /// Steps over one character; `false` at the end.
    fn step(&mut self) -> bool {
        let Some(c) = self.text.get(self.byte..).and_then(|s| s.chars().next()) else { return false };
        self.byte += c.len_utf8();
        self.column += 1;
        true
    }
}
