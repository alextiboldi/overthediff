//! Recomparing after edits. A [`Pair`] holds two texts and their comparison, and keeps the
//! comparison current as either side is edited, so a view can recompare as the user types.
//!
//! Each edit reruns the full comparison of the edited texts, so its rows are exactly what
//! [`compare`] gives for them.

use std::fmt;
use std::ops::Range;

use crate::{Comparison, Line, compare};

/// Two texts and their up-to-date comparison.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pair {
    left: Vec<Line>,
    right: Vec<Line>,
    comparison: Comparison,
}

/// An edit's line range doesn't lie inside its side's lines. The edit was not made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutOfRange {
    /// The range the edit asked for.
    pub lines: Range<usize>,
    /// How many lines the side has.
    pub len: usize,
}

impl fmt::Display for OutOfRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "lines {}..{} are not inside a side of {} lines", self.lines.start, self.lines.end, self.len)
    }
}

impl std::error::Error for OutOfRange {}

impl Pair {
    /// Compares `left` against `right`.
    pub fn new(left: Vec<Line>, right: Vec<Line>) -> Self {
        let comparison = compare(&left, &right);
        Self { left, right, comparison }
    }

    pub fn left(&self) -> &[Line] {
        &self.left
    }

    pub fn right(&self) -> &[Line] {
        &self.right
    }

    /// The comparison of the current texts. Its rows index [`Pair::left`] and [`Pair::right`].
    pub fn comparison(&self) -> &Comparison {
        &self.comparison
    }

    /// Replaces the left `lines` with `with` (an empty range inserts, an empty `with` deletes) and
    /// recompares.
    pub fn edit_left(&mut self, lines: Range<usize>, with: Vec<Line>) -> Result<&Comparison, OutOfRange> {
        splice(&mut self.left, lines, with)?;
        Ok(self.recompare())
    }

    /// Replaces the right `lines` with `with`, like [`Pair::edit_left`], and recompares.
    pub fn edit_right(&mut self, lines: Range<usize>, with: Vec<Line>) -> Result<&Comparison, OutOfRange> {
        splice(&mut self.right, lines, with)?;
        Ok(self.recompare())
    }

    fn recompare(&mut self) -> &Comparison {
        // ponytail: a full recompare, about 1 to 4 ms for a 10,000-line pair in a release build,
        // inside E03-S1-T6's proposed 100 ms. If that budget is missed, reuse the old rows past the
        // edit: Standard alignment from a point (i, j) reads only the lines after it, so once the
        // new walk reaches an old point beyond the edit, the old rows from there on hold, shifted.
        self.comparison = compare(&self.left, &self.right);
        &self.comparison
    }
}

/// Replaces `lines` of `side` with `with`, or leaves `side` alone when the range isn't inside it.
fn splice(side: &mut Vec<Line>, lines: Range<usize>, with: Vec<Line>) -> Result<(), OutOfRange> {
    if lines.start > lines.end || lines.end > side.len() {
        return Err(OutOfRange { lines, len: side.len() });
    }
    side.splice(lines, with);
    Ok(())
}
