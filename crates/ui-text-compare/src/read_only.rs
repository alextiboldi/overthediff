//! Read-only panes, each with the reason shown (E07-S1-T7). A pane is read-only when its data
//! source refuses writes ([`ReadOnly::Location`], set when it loads), the file is marked read-only
//! ([`ReadOnly::File`]) or has bytes its encoding can't read ([`ReadOnly::Encoding`], E07-S1-T6),
//! or when the last comparison
//! failed or was cancelled; the comparison's runner reports how it ended through
//! [`TextCompareView::comparison_ended`]. Editing and saving check [`TextCompareView::read_only`].

use egui::Ui;
use egui::accesskit::Live;
use otd_core::i18n::t;
use otd_core::task::End;
use otd_text_compare::ReadOnly;

use crate::TextCompareView;

// Each reason as the pane shows it: English keys, drawn through `t`.
pub const LOCATION: &str = "Read-only: the file's location can't be written to";
pub const FILE_READ_ONLY: &str = "Read-only: the file is marked read-only";
pub const ENCODING_LOSSY: &str = "Read-only: some of the file's bytes can't be read in its encoding, and saving would change them";
pub const COMPARISON_FAILED: &str = "Read-only: the comparison failed";
pub const COMPARISON_CANCELLED: &str = "Read-only: the comparison was cancelled";

/// What the pane says for `reason`, as an English key.
pub(crate) fn reason(reason: ReadOnly) -> &'static str {
    match reason {
        ReadOnly::Location => LOCATION,
        ReadOnly::File => FILE_READ_ONLY,
        ReadOnly::Encoding => ENCODING_LOSSY,
        ReadOnly::ComparisonFailed => COMPARISON_FAILED,
        ReadOnly::ComparisonCancelled => COMPARISON_CANCELLED,
    }
}

impl TextCompareView {
    /// Records how the comparison ended: a failed or cancelled one makes both panes read-only, and
    /// a completed one lifts that again. Loading other files forgets it, so call it once the files
    /// it compared have loaded.
    pub fn comparison_ended(&mut self, end: End) {
        self.comparison = match end {
            End::Completed => None,
            End::Failed => Some(ReadOnly::ComparisonFailed),
            End::Cancelled => Some(ReadOnly::ComparisonCancelled),
        };
    }

    /// Why the loaded pane of side `side` (0 is left) can't be edited, if it can't. Its location
    /// comes first, since a later comparison can't change it.
    pub fn read_only(&self, side: usize) -> Option<ReadOnly> {
        self.pane(side)?.read_only.or(self.comparison)
    }
}

/// The pane's reason, if it is read-only. Announced, since it can change away from focus (a
/// comparison ending).
pub(crate) fn show(ui: &mut Ui, read_only: Option<ReadOnly>) {
    if let Some(read_only) = read_only {
        let label = ui.label(t(reason(read_only)));
        ui.ctx().accesskit_node_builder(label.id, |node| node.set_live(Live::Polite));
    }
}
