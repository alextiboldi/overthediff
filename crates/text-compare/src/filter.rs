//! Display filters (E07-S3-T1): which aligned rows the view shows. A filter picks whole rows, so
//! both panes show the same rows and stay aligned.

use std::ops::Range;

/// The rows a display filter shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Filter {
    /// Every row.
    #[default]
    All,
    /// Only the rows that differ.
    Differences,
    /// Only the rows that are the same.
    Same,
    /// No row.
    None,
    /// The rows that differ, and the rows within the context count of one.
    Context,
}

/// The rows `filter` shows, as ascending ranges of row indices that neither overlap nor touch, so
/// each gap between two ranges (and before the first, and after the last) is one run of hidden
/// rows. `differs` says, row by row, whether the row counts as different in the view's current
/// classification. `context` is how many rows on each side of a different row
/// [`Filter::Context`] also shows.
pub fn visible(differs: impl IntoIterator<Item = bool>, filter: Filter, context: usize) -> Vec<Range<usize>> {
    let mut out: Vec<Range<usize>> = Vec::new();
    let mut rows = 0;
    for (row, differs) in differs.into_iter().enumerate() {
        rows = row + 1;
        let shown = match filter {
            Filter::All => Some(row..rows),
            Filter::Differences => differs.then_some(row..rows),
            Filter::Same => (!differs).then_some(row..rows),
            Filter::None => None,
            Filter::Context => differs.then(|| row.saturating_sub(context)..rows.saturating_add(context)),
        };
        let Some(range) = shown else { continue };
        match out.last_mut() {
            Some(last) if last.end >= range.start => last.end = last.end.max(range.end),
            _ => out.push(range),
        }
    }
    // Only the last range can run past the end: any earlier one that did would reach the next.
    if let Some(last) = out.last_mut() {
        last.end = last.end.min(rows);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{Filter, visible};

    /// Shown rows as `(start, end)` pairs, which read more plainly in a table than ranges.
    type Shown = Vec<(usize, usize)>;

    /// The rows `filter` shows with `context` rows, of a comparison given one character per row:
    /// `#` differs, `.` is the same.
    fn shown_with(rows: &str, filter: Filter, context: usize) -> Shown {
        visible(rows.chars().map(|c| c == '#'), filter, context).into_iter().map(|r| (r.start, r.end)).collect()
    }

    /// The same, with N = 2.
    fn shown(rows: &str, filter: Filter) -> Shown {
        shown_with(rows, filter, 2)
    }

    #[test]
    fn each_filter_shows_the_differences_same_rows_or_context_it_names() {
        // Two sections, with N = 2: far apart, overlapping context, context that just touches, and
        // context one row short of touching. Sections near the ends clamp to the file.
        let cases: [(&str, [Shown; 2], Shown); 4] = [
            ("..##......#..", [vec![(2, 4), (10, 11)], vec![(0, 2), (4, 10), (11, 13)]], vec![(0, 6), (8, 13)]),
            ("..##...#....", [vec![(2, 4), (7, 8)], vec![(0, 2), (4, 7), (8, 12)]], vec![(0, 10)]),
            ("...#....#...", [vec![(3, 4), (8, 9)], vec![(0, 3), (4, 8), (9, 12)]], vec![(1, 11)]),
            ("...#.....#..", [vec![(3, 4), (9, 10)], vec![(0, 3), (4, 9), (10, 12)]], vec![(1, 6), (7, 12)]),
        ];
        for (rows, [differences, same], context) in cases {
            assert_eq!(shown(rows, Filter::All), vec![(0, rows.len())], "{rows} all");
            assert_eq!(shown(rows, Filter::Differences), differences, "{rows} differences");
            assert_eq!(shown(rows, Filter::Same), same, "{rows} same");
            assert_eq!(shown(rows, Filter::None), vec![], "{rows} none");
            assert_eq!(shown(rows, Filter::Context), context, "{rows} context");
            assert_eq!(shown_with(rows, Filter::Context, 0), differences, "{rows} context 0");
        }
    }

    #[test]
    fn nothing_shows_when_there_are_no_rows_or_no_differences_to_give_context_to() {
        for filter in [Filter::All, Filter::Differences, Filter::Same, Filter::None, Filter::Context] {
            assert_eq!(shown("", filter), vec![], "{filter:?}");
        }
        assert_eq!(shown("....", Filter::Context), vec![]);
        assert_eq!(shown_with("#", Filter::Context, usize::MAX), vec![(0, 1)]);
    }
}
