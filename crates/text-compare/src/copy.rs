//! Which lines a copy to the other side covers (E07-S7-T1): the lines it takes from the source
//! pane and the target lines they replace, worked out on the comparison's aligned rows.
//!
//! A copy covers a run of rows. Its source lines are the source side's lines in those rows and its
//! target lines the target side's, so replacing the target lines with the source lines makes those
//! rows the same: lines only the target has are removed, and lines it lacks are put in at the gap.
//!
//! - [`copy_selection`]: the selected lines, from the first selected line's row to the last's, so
//!   the rows of target-only lines between them are covered too. With nothing selected, the
//!   difference section the caret's line is in ([`copy_section`]).
//! - [`copy_lines`]: given lines, as *Copy Line* copies the caret's line.
//!
//! A section with no source lines at all (only gaps on the source side) holds no source line for
//! the caret to stand on, so a caret on the line just above the gap takes it, or else one on the
//! line just below (*inferred*). The caret line after the last line break, the only one an empty
//! pane has, takes the section after the last line, which has no source lines (*inferred*). Lines are copied with their own endings; making a copied last line
//! (one without a break) fit mid-file is the applying edit's concern (E07-S7-T4).

use std::ops::Range;

use otd_text_engine::{Row, RowKind, Side};

use crate::Selection;

/// What a copy covers: the `source` lines replace the `target` lines. Either range may be empty.
/// An empty `source` removes the `target` lines, and an empty `target` is the place on the target
/// side where the `source` lines go in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CopyLines {
    pub source: Range<usize>,
    pub target: Range<usize>,
}

/// *Copy to Other Side* from side `from` with `selection` in its pane: the lines the selection
/// touches, or with nothing selected the caret's section. A selection that ends at the start of a
/// line doesn't take that line. `None` when there is nothing to copy.
pub fn copy_selection(rows: &[Row], from: Side, selection: Selection) -> Option<CopyLines> {
    if selection.is_empty() {
        return copy_section(rows, from, selection.head.line);
    }
    let (start, end) = selection.range();
    let last = if end.column == 0 && end.line > start.line { end.line - 1 } else { end.line };
    copy_lines(rows, from, start.line..last + 1)
}

/// The difference section that line `line` of side `from` is in, or for a line that is the same
/// on both sides, a section next to it that has no lines on side `from`: the one below, else the
/// one above (*inferred*). Line `line` may be the caret line after the last line break, which takes
/// the section after the last line. `None` when there is no such section, or no such line.
pub fn copy_section(rows: &[Row], from: Side, line: usize) -> Option<CopyLines> {
    let Some(row) = row_of(rows, from, line) else {
        let after = rows.iter().rposition(|row| self::line(row, from).is_some()).map_or(0, |i| i + 1);
        let count = rows.iter().filter(|row| self::line(row, from).is_some()).count();
        return (line == count && after < rows.len()).then(|| section(rows, from, after));
    };
    let changed = |row: &usize| rows.get(*row).is_some_and(|r| r.kind != RowKind::Same);
    if changed(&row) {
        return Some(section(rows, from, row));
    }
    [Some(row + 1), row.checked_sub(1)].into_iter().flatten().filter(changed).map(|row| section(rows, from, row)).find(|copy| copy.source.is_empty())
}

/// The section that changed row `row` is in.
fn section(rows: &[Row], from: Side, row: usize) -> CopyLines {
    let same = |r: &Row| r.kind == RowKind::Same;
    let start = rows.iter().take(row).rposition(same).map_or(0, |i| i + 1);
    let end = rows.iter().skip(row).position(same).map_or(rows.len(), |i| row + i);
    covering(rows, from, start..end)
}

/// The rows from line `lines.start`'s to the last line's, on side `from`. `None` when `lines` is
/// empty or past that side's lines.
pub fn copy_lines(rows: &[Row], from: Side, lines: Range<usize>) -> Option<CopyLines> {
    let first = row_of(rows, from, lines.start)?;
    let last = row_of(rows, from, lines.end.checked_sub(1)?)?;
    (first <= last).then(|| covering(rows, from, first..last + 1))
}

fn line(row: &Row, side: Side) -> Option<usize> {
    match side {
        Side::Left => row.left,
        Side::Right => row.right,
    }
}

fn row_of(rows: &[Row], side: Side, at: usize) -> Option<usize> {
    rows.iter().position(|row| line(row, side) == Some(at))
}

/// Each side's lines in rows `covered`. Every line is in exactly one row, in order, so a side's
/// lines there start at the count of its lines in the rows before.
fn covering(rows: &[Row], from: Side, covered: Range<usize>) -> CopyLines {
    let count = |side: Side, end: usize| rows.iter().take(end).filter(|row| line(row, side).is_some()).count();
    let lines = |side: Side| count(side, covered.start)..count(side, covered.end);
    let to = match from {
        Side::Left => Side::Right,
        Side::Right => Side::Left,
    };
    CopyLines { source: lines(from), target: lines(to) }
}

#[cfg(test)]
mod tests {
    use otd_text_engine::{Line, LineEnding, RowKind, Side, compare};

    use super::{CopyLines, copy_section, copy_selection};
    use crate::{Encoding, LineEndings, Pane, Pos, Selection, WESTERN, decode};

    fn lines(texts: &[&str]) -> Vec<Line> {
        texts.iter().map(|text| Line { text: (*text).to_owned(), ending: LineEnding::Lf }).collect()
    }

    /// Applies `copy` from side `from` and returns the sides after it.
    fn apply(left: &[Line], right: &[Line], from: Side, copy: &CopyLines) -> (Vec<Line>, Vec<Line>) {
        let (source, mut target) = match from {
            Side::Left => (left, right.to_vec()),
            Side::Right => (right, left.to_vec()),
        };
        target.splice(copy.target.clone(), source[copy.source.clone()].iter().cloned());
        match from {
            Side::Left => (left.to_vec(), target),
            Side::Right => (target, right.to_vec()),
        }
    }

    // Three equal lines either side of each change, so the engine aligns back in step around it.
    const TOP: [&str; 3] = ["a", "b", "c"];
    const BOTTOM: [&str; 3] = ["x", "y", "z"];

    fn around(middle: &[&str]) -> Vec<Line> {
        lines(&[&TOP[..], middle, &BOTTOM[..]].concat())
    }

    #[test]
    fn copying_a_changed_section_from_either_side_makes_the_other_side_match() {
        let (left, right) = (around(&["l1", "l2"]), around(&["r1", "r2"]));
        for (from, source) in [(Side::Left, &left), (Side::Right, &right)] {
            let rows = compare(&left, &right).rows;
            let copy = copy_section(&rows, from, 4).expect("line 4 is in the changed section");
            let (new_left, new_right) = apply(&left, &right, from, &copy);
            assert_eq!(new_left, new_right, "{from:?}: the copy made both sides the same");
            assert_eq!(&new_left, source, "{from:?}: and the source didn't change");
        }
    }

    #[test]
    fn copying_lines_only_the_source_has_puts_them_in_at_the_gap() {
        let (left, right) = (around(&["new1", "new2"]), around(&[]));
        let rows = compare(&left, &right).rows;
        let copy = copy_section(&rows, Side::Left, 3).expect("line 3 is left-only");
        assert!(copy.target.is_empty(), "nothing on the target is replaced");
        let (left, right) = apply(&left, &right, Side::Left, &copy);
        assert_eq!(left, right);
    }

    #[test]
    fn copying_a_section_removes_the_lines_only_the_target_has() {
        let (left, right) = (around(&["one"]), around(&["uno", "dos", "tres"]));
        let rows = compare(&left, &right).rows;
        assert!(rows.iter().any(|row| row.kind == RowKind::RightOnly), "the section has right-only lines");
        let copy = copy_section(&rows, Side::Left, 3).expect("line 3 is changed");
        assert!(copy.target.len() > copy.source.len(), "the target loses lines");
        let (left, right) = apply(&left, &right, Side::Left, &copy);
        assert_eq!(left, right);
    }

    #[test]
    fn a_partial_selection_copies_only_the_rows_of_its_lines() {
        let (left, right) = (around(&["l1", "l2", "l3", "l4"]), around(&["r1", "r2", "r3", "r4"]));
        // From inside l2 to the start of l4: l2 and l3, not l4, which the selection only touches.
        let selection = Selection { anchor: Pos { line: 4, column: 1 }, head: Pos { line: 6, column: 0 } };
        let copy = copy_selection(&compare(&left, &right).rows, Side::Left, selection).expect("lines are selected");
        let (new_left, new_right) = apply(&left, &right, Side::Left, &copy);
        let rows = compare(&new_left, &new_right).rows;
        let changed = |line| rows.iter().find(|row| row.left == Some(line)).is_some_and(|row| row.kind != RowKind::Same);
        assert!(!changed(4) && !changed(5), "the selected lines' rows are the same");
        assert!(changed(3) && changed(6), "the rest of the section still differs");
    }

    #[test]
    fn a_caret_beside_lines_only_the_target_has_copies_their_removal() {
        let (left, right) = (around(&[]), around(&["extra1", "extra2"]));
        let rows = compare(&left, &right).rows;
        // "c" sits just above the gap and "x" just below it; "a" is nowhere near.
        for line in [2, 3] {
            let copy = copy_section(&rows, Side::Left, line).expect("the caret is beside the gap");
            assert!(copy.source.is_empty(), "line {line}: nothing comes from the source");
            let (new_left, new_right) = apply(&left, &right, Side::Left, &copy);
            assert_eq!(new_left, new_right, "line {line}: the extra lines are removed");
        }
        assert_eq!(copy_section(&rows, Side::Left, 0), None);
    }

    /// A pane holding `text` as decoded from a file.
    fn pane(text: &str) -> Pane {
        Pane {
            path: String::new(),
            size: 0,
            modified: None,
            format: String::new(),
            encoding: Encoding::Utf8 { bom: false },
            endings: LineEndings::Unix,
            lines: decode(text.as_bytes(), WESTERN).lines,
            read_only: None,
            unsaved: false,
            selection: Selection::default(),
            history: Default::default(),
        }
    }

    #[test]
    fn select_all_copies_the_whole_file() {
        // The last two files end with a break, which decodes to no line after it.
        for (left, right) in [("a\nb", "a\nc"), ("a\nb\n", "a\nc\n"), ("a\nb\n\n", "a\nb\n")] {
            let (mut source, target) = (pane(left), pane(right));
            source.select_all();
            let copy = copy_selection(&compare(&source.lines, &target.lines).rows, Side::Left, source.selection)
                .unwrap_or_else(|| panic!("{left:?}: everything is selected"));
            let (new_left, new_right) = apply(&source.lines, &target.lines, Side::Left, &copy);
            assert_eq!(new_left, new_right, "{left:?}");
        }
    }

    #[test]
    fn the_caret_after_the_last_line_removes_lines_only_the_target_has_after_it() {
        for (left, right) in [("a\nb\nc\n", "a\nb\nc\nextra\n"), ("", "a\nb\n")] {
            let (source, target) = (pane(left), pane(right));
            let rows = compare(&source.lines, &target.lines).rows;
            let copy = copy_section(&rows, Side::Left, source.end().line).unwrap_or_else(|| panic!("{left:?}: the caret is below the last line"));
            let (new_left, new_right) = apply(&source.lines, &target.lines, Side::Left, &copy);
            assert_eq!(new_left, new_right, "{left:?}");
        }
        // With the sides the same, there is nothing after the last line to copy.
        let same = pane("a\n");
        assert_eq!(copy_section(&compare(&same.lines, &same.lines).rows, Side::Left, same.end().line), None);
    }
}
