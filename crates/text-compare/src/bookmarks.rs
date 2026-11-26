//! A pane's bookmarks 0–9 (E07-S6-T2): each number marks one line, and moves with that line
//! through every edit, Undo and Redo included, since they all splice lines.

use otd_text_engine::Line;

/// How many bookmarks a pane has, numbered from 0.
pub const BOOKMARKS: usize = 10;

/// Each numbered bookmark's line, if it is set. Several numbers may mark one line.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bookmarks([Option<usize>; BOOKMARKS]);

impl Bookmarks {
    /// Bookmark `number`'s line, if it is set.
    pub fn get(&self, number: usize) -> Option<usize> {
        self.0.get(number).copied().flatten()
    }

    /// Sets bookmark `number` on `line`, or clears it when it is already there. Returns whether it
    /// is now set.
    pub fn toggle(&mut self, number: usize, line: usize) -> bool {
        let Some(slot) = self.0.get_mut(number) else { return false };
        *slot = if *slot == Some(line) { None } else { Some(line) };
        slot.is_some()
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().all(Option::is_none)
    }

    /// The numbers set on `line`, lowest first.
    pub fn on(&self, line: usize) -> impl Iterator<Item = usize> + '_ {
        (0..BOOKMARKS).filter(move |&number| self.get(number) == Some(line))
    }

    /// Moves each bookmark with its line as `removed`, at line `at`, is replaced with `inserted`,
    /// leaving `len` lines. The lines the two share at their start and end are the same lines, so
    /// a bookmark on one keeps it: a line split at its start keeps its bookmark on its text, below
    /// the new break. A bookmark on a changed line stays on what took its place, and on a deleted
    /// line moves to the line after it, or the last line; with no lines left it is cleared.
    pub(crate) fn follow(&mut self, at: usize, removed: &[Line], inserted: &[Line], len: usize) {
        let head = removed.iter().zip(inserted).take_while(|(a, b)| a == b).count();
        let tail = removed[head..].iter().rev().zip(inserted[head..].iter().rev()).take_while(|(a, b)| a == b).count();
        let changed = removed.len() - head - tail;
        let replacing = inserted.len() - head - tail;
        for slot in &mut self.0 {
            let Some(line) = *slot else { continue };
            *slot = if line < at + head {
                Some(line)
            } else if line >= at + head + changed {
                // `line + inserted - removed` never underflows: `inserted` holds the `tail` lines.
                Some(line + inserted.len() - removed.len())
            } else {
                let moved = at + head + (line - at - head).min(replacing.saturating_sub(1));
                len.checked_sub(1).map(|last| moved.min(last))
            };
        }
    }
}
