//! OverTheDiff's text comparison engine. It aligns two line sequences into rows (a line or a gap on
//! each side) and groups the differing rows into sections. It has no UI and runs without a window.
//!
//! Standard alignment walks both inputs forwards, so rows come out in order and a later caller can
//! hand out the top of a comparison before the rest is done. At each mismatch it looks for the
//! nearest point where both sides agree again for [`RUN`] lines (or both end together), so a common
//! line such as a blank or a closing brace doesn't pull the alignment off course. Between two such
//! points it pairs single matching lines, nearest first, and pairs whatever is left over as changed
//! lines from the top, with the surplus on one side against gaps. Of two equally near points, the
//! one where the sides agree for longer wins, so a single edit aligns the same way whichever file
//! is on which side.
//!
//! Lines align on their text. A pair whose text matches but whose ending differs is a changed row.

use std::collections::HashMap;
use std::ops::Range;

/// How a line ends in its file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LineEnding {
    /// The last line of a file that doesn't end with a line break.
    None,
    Lf,
    CrLf,
    Cr,
}

/// One decoded line: its text without the line break, and the break that ended it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Line {
    pub text: String,
    pub ending: LineEnding,
}

/// What an aligned row holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowKind {
    /// Equal lines on both sides.
    Same,
    /// A line on each side, and they differ.
    Changed,
    /// A left line against a gap.
    LeftOnly,
    /// A right line against a gap.
    RightOnly,
}

/// One aligned row: the index of the line on each side, or `None` for a gap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Row {
    pub left: Option<usize>,
    pub right: Option<usize>,
    pub kind: RowKind,
}

/// The aligned rows of a comparison and its sections.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Comparison {
    /// Every line of both inputs appears in exactly one row, in input order on each side.
    pub rows: Vec<Row>,
    /// Each section is a maximal run of contiguous rows that aren't [`RowKind::Same`], as a range
    /// of row indices, in row order.
    pub sections: Vec<Range<usize>>,
}

/// How many consecutive equal lines confirm that the two sides are back in step after a mismatch.
pub const RUN: usize = 3;

/// The id that pads a run past the end of its input, so equal endings confirm a short final run.
const END: u32 = u32::MAX;

/// Aligns `left` against `right` with Standard alignment.
pub fn compare(left: &[Line], right: &[Line]) -> Comparison {
    let (l, r) = intern(left, right);
    let index = Index::new(&r);
    let mut out = Rows { left, right, rows: Vec::with_capacity(left.len().max(right.len())) };
    let (mut i, mut j) = (0, 0);
    loop {
        while l.get(i).is_some_and(|id| r.get(j) == Some(id)) {
            out.pair(i, j);
            i += 1;
            j += 1;
        }
        if i == l.len() && j == r.len() {
            break;
        }
        let (ai, aj) = index.next_anchor(&l, &r, i, j).unwrap_or((l.len(), r.len()));
        fill_hunk(&mut out, (&l, &r), &index, i..ai, j..aj);
        (i, j) = (ai, aj);
    }
    let sections = sections(&out.rows);
    Comparison { rows: out.rows, sections }
}

/// Gives each distinct line text a small id, shared by both sides.
fn intern<'a>(left: &'a [Line], right: &'a [Line]) -> (Vec<u32>, Vec<u32>) {
    let mut ids: HashMap<&'a str, u32> = HashMap::new();
    let mut side = |lines: &'a [Line]| -> Vec<u32> {
        lines
            .iter()
            .map(|line| {
                // ponytail: past u32::MAX - 1 distinct texts, the rest share the last id, so they
                // align as if equal (their rows still say changed); no real input comes near that.
                let next = u32::try_from(ids.len()).unwrap_or(END - 1).min(END - 1);
                *ids.entry(line.text.as_str()).or_insert(next)
            })
            .collect()
    };
    (side(left), side(right))
}

/// Where each line, and each run of [`RUN`] lines, occurs on the right, in ascending order.
struct Index {
    lines: HashMap<u32, Vec<usize>>,
    runs: HashMap<[u32; RUN], Vec<usize>>,
}

impl Index {
    fn new(right: &[u32]) -> Self {
        let mut lines: HashMap<u32, Vec<usize>> = HashMap::new();
        let mut runs: HashMap<[u32; RUN], Vec<usize>> = HashMap::new();
        for (q, &id) in right.iter().enumerate() {
            lines.entry(id).or_default().push(q);
            runs.entry(run_at(right, q)).or_default().push(q);
        }
        Self { lines, runs }
    }

    /// The nearest point at or after `(i, j)` where a run of [`RUN`] lines matches.
    fn next_anchor(&self, left: &[u32], right: &[u32], i: usize, j: usize) -> Option<(usize, usize)> {
        nearest(left, right, i, j, |p| self.runs.get(&run_at(left, p)).and_then(|at| first_from(at, j)))
    }

    /// The nearest single matching line inside `left` × `right`, with the sides agreeing only up to
    /// the ends of those ranges.
    fn next_match(&self, ids: (&[u32], &[u32]), left: Range<usize>, right: Range<usize>) -> Option<(usize, usize)> {
        let (l, r) = (ids.0.get(..left.end).unwrap_or_default(), ids.1.get(..right.end).unwrap_or_default());
        nearest(l, r, left.start, right.start, |p| {
            let id = l.get(p)?;
            self.lines.get(id).and_then(|at| first_from(at, right.start)).filter(|&q| q < right.end)
        })
    }
}

/// The nearest point `(p, q)` at or after `(i, j)`, by lines skipped on both sides together, where
/// `find(p)` gives the first right line at or after `j` that matches from left line `p`. Of equally
/// near points, the one where the sides then agree for longer wins, rather than whichever sits on
/// the left; the earlier left line wins a full tie.
///
/// The search stops once skipping more left lines can't match the best point, so its scan follows
/// the number of lines the point skips.
fn nearest(left: &[u32], right: &[u32], i: usize, j: usize, find: impl Fn(usize) -> Option<usize>) -> Option<(usize, usize)> {
    let mut best: Option<(usize, usize, usize)> = None;
    for p in i..left.len() {
        if best.is_some_and(|(cost, _, _)| p - i > cost) {
            break;
        }
        let Some(q) = find(p) else { continue };
        let cost = (p - i) + (q - j);
        // ponytail: a tie measures both agreements in full. Long ties need periodic text, and the
        // winner's agreement is consumed next, so the extra work is ties × that agreement.
        let wins = best.is_none_or(|(c, bp, bq)| cost < c || (cost == c && agreement(left, right, p, q) > agreement(left, right, bp, bq)));
        if wins {
            best = Some((cost, p, q));
        }
    }
    best.map(|(_, p, q)| (p, q))
}

/// How many lines agree from left `p` and right `q` on, plus one when both sides then end together.
fn agreement(left: &[u32], right: &[u32], p: usize, q: usize) -> usize {
    let (l, r) = (left.get(p..).unwrap_or_default(), right.get(q..).unwrap_or_default());
    let same = l.iter().zip(r).take_while(|(a, b)| a == b).count();
    same + usize::from(same == l.len() && same == r.len())
}

/// The ids of the [`RUN`] lines starting at `p`, padded with [`END`] past the end.
fn run_at(ids: &[u32], p: usize) -> [u32; RUN] {
    std::array::from_fn(|k| ids.get(p + k).copied().unwrap_or(END))
}

/// The first position in the ascending `positions` that is at least `from`.
fn first_from(positions: &[usize], from: usize) -> Option<usize> {
    positions.get(positions.partition_point(|&q| q < from)).copied()
}

/// Aligns the lines between two anchors: single matching lines pair up, nearest first, and the
/// lines between them pair as changed rows.
fn fill_hunk(out: &mut Rows<'_>, ids: (&[u32], &[u32]), index: &Index, mut left: Range<usize>, mut right: Range<usize>) {
    while let Some((p, q)) = index.next_match(ids, left.clone(), right.clone()) {
        out.unmatched(left.start..p, right.start..q);
        out.pair(p, q);
        left.start = p + 1;
        right.start = q + 1;
    }
    out.unmatched(left, right);
}

/// The rows built so far, with the inputs they index.
struct Rows<'a> {
    left: &'a [Line],
    right: &'a [Line],
    rows: Vec<Row>,
}

impl Rows<'_> {
    /// A row holding left line `i` and right line `j`.
    fn pair(&mut self, i: usize, j: usize) {
        let kind = if self.left.get(i) == self.right.get(j) { RowKind::Same } else { RowKind::Changed };
        self.rows.push(Row { left: Some(i), right: Some(j), kind });
    }

    /// Lines with no partner: paired from the top as changed rows, the surplus against gaps.
    fn unmatched(&mut self, left: Range<usize>, right: Range<usize>) {
        let paired = left.len().min(right.len());
        for k in 0..paired {
            self.pair(left.start + k, right.start + k);
        }
        for i in left.start + paired..left.end {
            self.rows.push(Row { left: Some(i), right: None, kind: RowKind::LeftOnly });
        }
        for j in right.start + paired..right.end {
            self.rows.push(Row { left: None, right: Some(j), kind: RowKind::RightOnly });
        }
    }
}

/// The maximal runs of rows that aren't [`RowKind::Same`].
fn sections(rows: &[Row]) -> Vec<Range<usize>> {
    let mut out = Vec::new();
    let mut start = None;
    for (k, row) in rows.iter().enumerate() {
        match (row.kind == RowKind::Same, start) {
            (false, None) => start = Some(k),
            (true, Some(s)) => {
                out.push(s..k);
                start = None;
            }
            _ => {}
        }
    }
    if let Some(s) = start {
        out.push(s..rows.len());
    }
    out
}
