//! Loading another file into a pane with unsaved edits asks Save / Don't Save / Cancel first
//! (E07-S1-T9), through the path edit (typed, browsed or a recent path) and *File > Open File*.
//! Save saves the view's unsaved panes, as Close Tab does; Cancel keeps the edits and the old file.

use std::any::Any;

use otd_ui::View;

use crate::TextCompareView;

impl TextCompareView {
    /// Loads `spec` into side `side` on the next draw. When that pane has unsaved edits it holds
    /// `spec` instead and returns `true`: the caller then asks, with [`load_pending`] to run after
    /// Save or Don't Save.
    pub(crate) fn load(&mut self, side: usize, spec: String) -> bool {
        let unsaved = self.unsaved(side);
        if unsaved {
            self.pending = Some((side, spec));
        } else {
            self.set_spec(side, spec);
        }
        unsaved
    }

    pub(crate) fn set_spec(&mut self, side: usize, spec: String) {
        if let Some(current) = self.session.specs.get_mut(side) {
            *current = spec;
            self.retry(side);
        }
    }
}

/// After Save or Don't Save: loads the held file.
pub(crate) fn load_pending(view: &mut dyn View) {
    if let Some(view) = (view as &mut dyn Any).downcast_mut::<TextCompareView>()
        && let Some((side, spec)) = view.pending.take()
    {
        view.set_spec(side, spec);
    }
}
