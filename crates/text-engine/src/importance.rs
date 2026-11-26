//! Importance: which differences matter. Rules mark parts of a line's text unimportant; a changed
//! row is unimportant when its two lines are equal once those parts are left out. An orphan line
//! (a line against a gap) is always important, whatever its text, and with no rules every
//! difference is important. A different line ending is important when the comparison compared line
//! endings ([`Options::line_endings`](crate::Options)), and ignored otherwise.

use std::ops::Range;

use crate::{Comparison, Line, RowKind};

/// A rule that marks the parts of a line's text whose differences don't matter.
pub trait Rule {
    /// Byte ranges of `text` that are unimportant. They may overlap and come in any order. The
    /// parts of a range that fall outside `text` are ignored, and a reversed range marks nothing.
    fn unimportant(&self, text: &str) -> Vec<Range<usize>>;
}

/// How much a difference matters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Importance {
    Important,
    Unimportant,
}

/// The overall result of a comparison.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Every row is the same.
    Same,
    /// Some rows differ, and every difference is unimportant.
    UnimportantOnly,
    /// At least one difference is important.
    Important,
}

/// Counts over the rows of a comparison. Differences are counted in rows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Statistics {
    /// Rows that differ in a way that matters, orphan rows included.
    pub important: usize,
    /// Changed rows whose differences are all unimportant.
    pub unimportant: usize,
    /// Rows that are the same.
    pub same: usize,
    /// Left lines against a gap.
    pub left_orphans: usize,
    /// Right lines against a gap.
    pub right_orphans: usize,
    /// Difference sections.
    pub sections: usize,
}

/// A comparison's rows classified by importance, with its overall result and statistics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Classification {
    /// One entry per row of the comparison, in row order: `None` for a same row.
    pub rows: Vec<Option<Importance>>,
    pub outcome: Outcome,
    pub statistics: Statistics,
}

/// Classifies every row of `comparison`, which aligned `left` against `right`, by the `rules`. A
/// part of a line that any rule marks is unimportant.
pub fn classify(comparison: &Comparison, left: &[Line], right: &[Line], rules: &[&dyn Rule]) -> Classification {
    let mut statistics = Statistics { sections: comparison.sections.len(), ..Statistics::default() };
    let rows: Vec<Option<Importance>> = comparison
        .rows
        .iter()
        .map(|row| {
            let importance = match row.kind {
                RowKind::Same => None,
                RowKind::LeftOnly | RowKind::RightOnly => Some(Importance::Important),
                RowKind::Changed => {
                    let pair = row.left.and_then(|i| left.get(i)).zip(row.right.and_then(|j| right.get(j)));
                    // A line ending isn't text a rule can mark, so a different ending matters
                    // whenever line endings are compared.
                    let endings = comparison.options.line_endings;
                    let minor = pair
                        .is_some_and(|(l, r)| (!endings || l.ending == r.ending) && important_part(&l.text, rules) == important_part(&r.text, rules));
                    Some(if minor { Importance::Unimportant } else { Importance::Important })
                }
            };
            match (row.kind, importance) {
                (_, None) => statistics.same += 1,
                (_, Some(Importance::Unimportant)) => statistics.unimportant += 1,
                (kind, Some(Importance::Important)) => {
                    statistics.important += 1;
                    statistics.left_orphans += usize::from(kind == RowKind::LeftOnly);
                    statistics.right_orphans += usize::from(kind == RowKind::RightOnly);
                }
            }
            importance
        })
        .collect();
    let outcome = match statistics {
        Statistics { important: 0, unimportant: 0, .. } => Outcome::Same,
        Statistics { important: 0, .. } => Outcome::UnimportantOnly,
        _ => Outcome::Important,
    };
    Classification { rows, outcome, statistics }
}

/// The bytes of `text` that no rule marks unimportant.
fn important_part(text: &str, rules: &[&dyn Rule]) -> Vec<u8> {
    let mut marked = vec![false; text.len()];
    for range in rules.iter().flat_map(|rule| rule.unimportant(text)) {
        // ponytail: each range is marked byte by byte, so the cost is the ranges' total length,
        // clamped to the line; sort and merge them first if rules ever return many overlapping ones.
        for m in marked.get_mut(range.start.min(text.len())..range.end.min(text.len())).unwrap_or_default() {
            *m = true;
        }
    }
    text.bytes().zip(marked).filter(|&(_, m)| !m).map(|(b, _)| b).collect()
}
