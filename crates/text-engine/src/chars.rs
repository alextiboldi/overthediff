//! Character-level differences inside a changed row.
//!
//! The two lines lose their common start and end, and a shortest edit between what remains (an
//! O(ND) search for the fewest characters to delete and insert) leaves the rest unmarked. So the
//! marked characters are exactly what a shortest edit changes: drop them from both lines and the
//! two read the same. A pair is searched in one fixed order, so swapping the sides only swaps the
//! ranges.
//!
//! Characters are Unicode scalar values (`char`), so a changed combining mark is marked on its own,
//! apart from its base letter.

/// Which side of a comparison a range lies on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Side {
    Left,
    Right,
}

/// A run of differing characters in one line of a changed row: byte offsets `start..end` into that
/// line's [`Line::text`](crate::Line), both on character boundaries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CharRange {
    pub side: Side,
    pub start: usize,
    pub end: usize,
}

/// The most characters a pair may differ by for the search to stay exact. It bounds the search's
/// memory, which grows with its square.
const MAX_EDITS: usize = 2048;

/// The most steps (diagonals tried plus equal characters walked) the search may take. It bounds the
/// search's time, which grows with the length of the lines times the characters they differ by.
const MAX_STEPS: usize = 1 << 26;

/// The characters that differ between `left` and `right`, the texts of a changed row's two lines:
/// the left ranges, then the right ones, each in order, with no two touching. Equal texts give none,
/// so a row that differs only in its line ending has no character ranges.
///
/// Lines that differ by more than [`MAX_EDITS`] characters, or that would take more than
/// [`MAX_STEPS`] to search, get one range per side over everything between their common start and
/// end. That still covers every difference, but isn't the fewest characters.
// ponytail: the limits are fixed guesses; E03-S1-T6 sets the budgets for large inputs.
pub fn char_differences(left: &str, right: &str) -> Vec<CharRange> {
    within(left, right, MAX_EDITS, MAX_STEPS)
}

fn within(left: &str, right: &str, max_edits: usize, max_steps: usize) -> Vec<CharRange> {
    if left > right {
        let mut ranges: Vec<CharRange> = within(right, left, max_edits, max_steps)
            .into_iter()
            .map(|r| CharRange { side: if r.side == Side::Left { Side::Right } else { Side::Left }, ..r })
            .collect();
        ranges.sort_by_key(|r| (r.side, r.start));
        return ranges;
    }
    let a: Vec<char> = left.chars().collect();
    let b: Vec<char> = right.chars().collect();
    let head = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let (a_rest, b_rest) = (a.get(head..).unwrap_or_default(), b.get(head..).unwrap_or_default());
    let tail = a_rest.iter().rev().zip(b_rest.iter().rev()).take_while(|(x, y)| x == y).count();
    let a_mid = a_rest.get(..a_rest.len() - tail).unwrap_or_default();
    let b_mid = b_rest.get(..b_rest.len() - tail).unwrap_or_default();

    // Everything in the middle is marked, then the characters a shortest edit keeps are unmarked.
    let mut a_marks: Vec<bool> = (0..a.len()).map(|k| k >= head && k < head + a_mid.len()).collect();
    let mut b_marks: Vec<bool> = (0..b.len()).map(|k| k >= head && k < head + b_mid.len()).collect();
    for (x, y) in shortest_edit_keeps(a_mid, b_mid, max_edits, max_steps).unwrap_or_default() {
        if let Some(m) = a_marks.get_mut(head + x) {
            *m = false;
        }
        if let Some(m) = b_marks.get_mut(head + y) {
            *m = false;
        }
    }
    let mut ranges = runs(Side::Left, left, &a_marks);
    ranges.extend(runs(Side::Right, right, &b_marks));
    ranges
}

/// The maximal runs of marked characters in `text`, as byte ranges.
fn runs(side: Side, text: &str, marks: &[bool]) -> Vec<CharRange> {
    let mut out: Vec<CharRange> = Vec::new();
    for ((at, c), &marked) in text.char_indices().zip(marks) {
        if !marked {
            continue;
        }
        let end = at + c.len_utf8();
        match out.last_mut() {
            Some(last) if last.end == at => last.end = end,
            _ => out.push(CharRange { side, start: at, end }),
        }
    }
    out
}

/// The positions `(in a, in b)` of the equal characters that a shortest edit from `a` to `b`
/// keeps, or `None` past either limit.
///
/// This is the greedy O(ND) shortest-edit search over diagonals `k = x - y` from Eugene W. Myers,
/// "An O(ND) Difference Algorithm and Its Variations" (Algorithmica, 1986), keeping each round's
/// furthest points so the path can be walked back.
fn shortest_edit_keeps(a: &[char], b: &[char], max_edits: usize, max_steps: usize) -> Option<Vec<(usize, usize)>> {
    let (n, m) = (isize::try_from(a.len()).ok()?, isize::try_from(b.len()).ok()?);
    let mut steps = max_steps;
    // `rounds[d]` holds, for each diagonal `k` in `-d, -d + 2, ..., d`, the furthest `x` reached
    // with `d` edits.
    let mut rounds: Vec<Vec<isize>> = Vec::new();
    for d in 0..=isize::try_from(max_edits.min(a.len() + b.len())).ok()? {
        let mut round = Vec::with_capacity(rounds.len() + 1);
        for k in (-d..=d).step_by(2) {
            let mut x = match rounds.last() {
                None => 0,
                Some(prev) if from_above(prev, d, k) => furthest(prev, d - 1, k + 1),
                Some(prev) => furthest(prev, d - 1, k - 1) + 1,
            };
            let mut y = x - k;
            while x < n && y < m && at(a, x) == at(b, y) {
                x += 1;
                y += 1;
                steps = steps.checked_sub(1)?;
            }
            steps = steps.checked_sub(1)?;
            round.push(x);
            if x >= n && y >= m {
                rounds.push(round);
                return Some(walk_back(&rounds, n, m));
            }
        }
        rounds.push(round);
    }
    None
}

/// Walks the shortest edit back from `(n, m)` and collects the equal characters it steps over.
fn walk_back(rounds: &[Vec<isize>], n: isize, m: isize) -> Vec<(usize, usize)> {
    let mut keeps = Vec::new();
    let (mut x, mut y) = (n, m);
    for d in (1..rounds.len()).rev() {
        let Some(prev) = rounds.get(d - 1) else { break };
        let d = isize::try_from(d).unwrap_or(isize::MAX);
        let k = x - y;
        let (from_k, start) = if from_above(prev, d, k) {
            let px = furthest(prev, d - 1, k + 1);
            (k + 1, (px, px - k))
        } else {
            let px = furthest(prev, d - 1, k - 1);
            (k - 1, (px + 1, px + 1 - k))
        };
        while x > start.0 && y > start.1 {
            x -= 1;
            y -= 1;
            keeps.extend(pair(x, y));
        }
        x = furthest(prev, d - 1, from_k);
        y = x - from_k;
    }
    // Round 0 is a run of equal characters from the start.
    while x > 0 && y > 0 {
        x -= 1;
        y -= 1;
        keeps.extend(pair(x, y));
    }
    keeps
}

/// Whether round `d` reaches diagonal `k` by a step down (an insertion from diagonal `k + 1`) rather
/// than a step right (a deletion from diagonal `k - 1`), given the previous round.
fn from_above(prev: &[isize], d: isize, k: isize) -> bool {
    k == -d || (k != d && furthest(prev, d - 1, k - 1) < furthest(prev, d - 1, k + 1))
}

/// The furthest `x` on diagonal `k` in round `d`.
fn furthest(round: &[isize], d: isize, k: isize) -> isize {
    usize::try_from((k + d) / 2).ok().and_then(|i| round.get(i)).copied().unwrap_or(0)
}

fn at(chars: &[char], i: isize) -> Option<char> {
    usize::try_from(i).ok().and_then(|i| chars.get(i)).copied()
}

fn pair(x: isize, y: isize) -> Option<(usize, usize)> {
    Some((usize::try_from(x).ok()?, usize::try_from(y).ok()?))
}

#[cfg(test)]
mod tests {
    use super::{Side, within};

    fn marked(text: &str, side: Side, ranges: &[super::CharRange]) -> String {
        ranges.iter().filter(|r| r.side == side).map(|r| &text[r.start..r.end]).collect()
    }

    #[test]
    fn past_a_limit_the_whole_differing_middle_is_marked() {
        // Exact, only `X` and `Y` are marked; with no room to search, all of `XdefY` is.
        let (left, right) = ("abcXdefYghi", "abcdefghi");
        let ranges = within(left, right, 0, usize::MAX);
        assert_eq!(marked(left, Side::Left, &ranges), "XdefY");
        let ranges = within(left, right, usize::MAX, 1);
        assert_eq!(marked(left, Side::Left, &ranges), "XdefY");
    }
}
