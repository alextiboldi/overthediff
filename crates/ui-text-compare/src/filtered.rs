//! The Text Editing page's display-filter options (E07-S3-T3), and the rows a display filter
//! leaves in the panes. *Number of context lines* is *Show Context*'s *N*. With *Show filtered line
//! counts* on, each run of rows the filter hides takes one row in both panes that says how many of
//! that pane's lines it hides; off, the run takes no room. An open view reads both options every
//! frame, so Apply, and the Options dialog's Preview, change it at once.

use std::ops::Range;

use egui::Context;
use otd_core::session::Value;
use otd_text_compare::{Filter, visible};
use otd_ui::{OptionSpec, option};

// English keys, drawn only through `t`.
/// The Options dialog page (the feature catalogue's Text Editing page, F-26.11).
pub const TEXT_EDITING: &str = "Text Editing";
pub const CONTEXT_LINES: &str = "Number of context lines";
pub const FILTERED_LINE_COUNTS: &str = "Show filtered line counts";
/// A hidden run's summary, by the number of the pane's lines it hides (`trn`).
pub const LINE_HIDDEN: &str = "{n} line hidden";
pub const LINES_HIDDEN: &str = "{n} lines hidden";

/// The options' ids, under which the settings store keeps their values.
pub const CONTEXT_LINES_ID: &str = "text_editing.context_lines";
pub const FILTERED_LINE_COUNTS_ID: &str = "text_editing.show_filtered_line_counts";

// The feature catalogue leaves both defaults open (unspecified); these are our choice.
pub(crate) const DEFAULT_CONTEXT: usize = 3;
pub(crate) const DEFAULT_COUNTS: bool = true;

/// The two options, for the app's option list.
pub fn options() -> Vec<OptionSpec> {
    vec![
        OptionSpec {
            page: &[TEXT_EDITING],
            id: CONTEXT_LINES_ID,
            label: CONTEXT_LINES,
            default: Value::Number(DEFAULT_CONTEXT as f64),
            choices: &[],
        },
        OptionSpec {
            page: &[TEXT_EDITING],
            id: FILTERED_LINE_COUNTS_ID,
            label: FILTERED_LINE_COUNTS,
            default: Value::Bool(DEFAULT_COUNTS),
            choices: &[],
        },
    ]
}

/// The applied number of context lines, or the default while the option isn't contributed.
// ponytail: the number edit takes any finite number, so a negative one reads as 0 and a fraction
// rounds down here; give `OptionSpec` a range if the dialog should refuse them.
pub(crate) fn context(ctx: &Context) -> usize {
    match option(ctx, CONTEXT_LINES_ID) {
        // `as` saturates: a huge number shows every row, and NaN reads as 0.
        Some(Value::Number(n)) => n.max(0.0) as usize,
        _ => DEFAULT_CONTEXT,
    }
}

/// Whether hidden runs are summarised, or the default while the option isn't contributed.
pub(crate) fn counts(ctx: &Context) -> bool {
    match option(ctx, FILTERED_LINE_COUNTS_ID) {
        Some(Value::Bool(on)) => on,
        _ => DEFAULT_COUNTS,
    }
}

/// One row of what both panes draw.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Slot {
    /// An aligned row, by index.
    Row(usize),
    /// A run of rows the filter hides, summarised.
    Hidden(Range<usize>),
}

/// What `filter` leaves of `rows` aligned rows, each `differs` in the view's classification, in
/// order. `None` for *Show All*: each row is its own slot. Hidden runs get a slot only with `counts`.
pub(crate) fn slots(differs: impl IntoIterator<Item = bool>, rows: usize, filter: Filter, context: usize, counts: bool) -> Option<Vec<Slot>> {
    if filter == Filter::All {
        return None;
    }
    let mut out = Vec::new();
    let mut at = 0;
    // The empty range at the end closes a hidden run that reaches the last row.
    for shown in visible(differs, filter, context).into_iter().chain(std::iter::once(rows..rows)) {
        if counts && at < shown.start {
            out.push(Slot::Hidden(at..shown.start));
        }
        out.extend(shown.clone().map(Slot::Row));
        at = shown.end;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A different row at each end, 41 same rows between: Context with *N* = 2 hides the 37 in the
    /// middle, as one run.
    fn rows() -> Vec<bool> {
        std::iter::once(true).chain(std::iter::repeat_n(false, 41)).chain([true]).collect()
    }

    #[test]
    fn each_hidden_run_takes_one_summary_slot_with_counts_and_none_without() {
        let differs = rows();
        let with = slots(differs.clone(), differs.len(), Filter::Context, 2, true).unwrap();
        let without = slots(differs.clone(), differs.len(), Filter::Context, 2, false).unwrap();
        let hidden: Vec<_> = with.iter().filter_map(|slot| if let Slot::Hidden(rows) = slot { Some(rows.len()) } else { None }).collect();
        assert_eq!(hidden, [37], "one run of 37 rows is summarised");
        assert!(without.iter().all(|slot| matches!(slot, Slot::Row(_))), "without counts no run takes a slot");
        assert_eq!(with.len(), without.len() + 1, "the summary is the only slot counts add");
    }

    #[test]
    fn a_run_reaching_either_end_is_summarised_too() {
        // Show Same hides the different rows at both ends.
        let differs = rows();
        let shown = slots(differs.clone(), differs.len(), Filter::Same, 0, true).unwrap();
        assert_eq!(shown.first(), Some(&Slot::Hidden(0..1)));
        assert_eq!(shown.last(), Some(&Slot::Hidden(42..43)));
    }
}
