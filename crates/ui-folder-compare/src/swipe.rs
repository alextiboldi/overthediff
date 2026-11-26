//! Swipe selection (E08-S7-T1): pressing on a row's cells and dragging selects the range of rows
//! from there to the cell under the pointer. A swipe that stays in one pane selects on that side
//! only; one that crosses the middle into the other pane selects the same rows on both sides.
//!
//! *Unspecified*, decided here:
//! - A swipe is a new selection: the side it doesn't reach ends up with nothing selected, unlike a
//!   click, which leaves the other side's selection alone. So a swipe that crosses the middle and
//!   comes back selects on its own side only again.
//! - Shift and Ctrl (Cmd on macOS) don't change a swipe.
//! - A range takes only the items in it, as a Shift+click range does, so a gap stays unselected.
//! - The pointer over a gap between cells, or outside the rows, keeps the range it last had.
//!
//! The same selections are reachable without dragging (accessibility.md, pointer rule): Shift+click
//! or Shift+Up/Down selects a range on one side, then the same on the other side.

use crate::{FolderCompare, Pick};

impl FolderCompare {
    /// Selects as a swipe from the cell `from` to the cell `to` does, each a row of
    /// [`FolderCompare::shown`] and a side (0 left, 1 right).
    pub(crate) fn swipe(&mut self, from: (usize, usize), to: (usize, usize)) {
        let Some(start) = self.shown().get(from.0).map(|shown| shown.key.clone()) else { return };
        for side in 0..2 {
            if side == from.1 || side == to.1 {
                self.anchor[side] = Some(start.clone());
                self.pick(to.0, side, Pick::Range);
            } else {
                self.selected[side].clear();
            }
        }
    }
}
