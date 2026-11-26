//! A pane's bookmarks 0–9 (E07-S6-T2): each number marks one line, and moves with that line
//! through every edit, Undo and Redo included, since they all splice lines.

use crate::Pos;

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

    /// Moves each bookmark with its line as the text from `start` to `end` is replaced, the text
    /// after `end` landing on line `rest`, leaving `len` lines. The edit's places decide, never the
    /// lines' text, so equal lines can't mislead it. A line keeps its bookmark while its start is
    /// kept: `start`'s line when `start` is past its first column. Every other line the edit
    /// touches goes with the text after `end`, onto `rest`: a line split at its start keeps its
    /// bookmark on its text, below the new break, and a deleted line's moves to the line after it,
    /// or the last line; with no lines left it is cleared.
    pub(crate) fn follow(&mut self, start: Pos, end: Pos, rest: usize, len: usize) {
        for slot in &mut self.0 {
            let Some(line) = *slot else { continue };
            let moved = if line < start.line || (line == start.line && start.column > 0) {
                line
            } else if line > end.line {
                line - end.line + rest
            } else {
                rest
            };
            *slot = len.checked_sub(1).map(|last| moved.min(last));
        }
    }
}
