//! The View menu's display filters (E07-S3-T2): *Show All Lines*, *Show Different Lines*, *Show
//! Matching Lines* and *Show Different Lines in Context*, a radio group whose chosen one is
//! checked, and *Hide All Lines*, hidden by default (no menu entry or toolbar button) but runnable
//! by id. The first three also have toolbar buttons. A filter hides whole aligned rows in both
//! panes, so they stay aligned (`filtered.rs` draws them). Switching keeps each pane's caret on
//! its line while that line stays shown and scrolls it into view; a caret on a line the filter
//! hides moves to the nearest shown line of its pane. Caret moves skip hidden lines the way they
//! go, and Select All selects from the first shown line to the last. Captions replace the feature catalogue's working names (docs/feature-names.md); the ids keep
//! them.

use otd_core::command::{Command, Mark, State};
use otd_text_compare::{Filter, Pos};
use otd_ui::Shell;

use crate::edit::loaded_mut;
use crate::{TEXT_COMPARE, TextCompareView};

pub const SHOW_ALL: &str = "Show All Lines";
pub const SHOW_DIFFERENCES: &str = "Show Different Lines";
pub const SHOW_SAME: &str = "Show Matching Lines";
pub const SHOW_CONTEXT: &str = "Show Different Lines in Context";
pub const SHOW_NONE: &str = "Hide All Lines";

/// The display filters, a radio group. Q24 names no shortcut for them.
pub(crate) fn commands() -> Vec<Command<Shell>> {
    vec![
        Command::new("text-compare.show_all", SHOW_ALL, |shell| set(shell, Filter::All))
            .icon("list")
            .menu(&["View"])
            .toolbar()
            .state(|shell| mark(shell, Filter::All)),
        Command::new("text-compare.show_differences", SHOW_DIFFERENCES, |shell| set(shell, Filter::Differences))
            .icon("diff")
            .menu(&["View"])
            .toolbar()
            .state(|shell| mark(shell, Filter::Differences)),
        Command::new("text-compare.show_same", SHOW_SAME, |shell| set(shell, Filter::Same))
            .icon("equal")
            .menu(&["View"])
            .toolbar()
            .state(|shell| mark(shell, Filter::Same)),
        Command::new("text-compare.show_context", SHOW_CONTEXT, |shell| set(shell, Filter::Context))
            .icon("list-collapse")
            .menu(&["View"])
            .state(|shell| mark(shell, Filter::Context)),
        Command::new("text-compare.show_none", SHOW_NONE, |shell| set(shell, Filter::None)).icon("eye-off").state(|shell| mark(shell, Filter::None)),
    ]
    .into_iter()
    .map(|command| command.views(&[TEXT_COMPARE]))
    .collect()
}

fn set(shell: &mut Shell, filter: Filter) {
    if let Some(view) = shell.view_mut::<TextCompareView>() {
        view.set_filter(filter);
    }
}

fn mark(shell: &Shell, filter: Filter) -> State {
    let on = shell.view::<TextCompareView>().is_some_and(|view| view.filter == filter);
    State { mark: Some(Mark::Radio(on)), ..State::default() }
}

impl TextCompareView {
    /// The display filter in use.
    pub fn filter(&self) -> Filter {
        self.filter
    }

    /// Shows the rows `filter` picks, keeping each caret on its line while that line is shown.
    pub fn set_filter(&mut self, filter: Filter) {
        if self.filter == filter {
            return;
        }
        self.filter = filter;
        self.aligned.refilter(filter);
        for side in 0..2 {
            let Some(pane) = loaded_mut(&mut self.panes, side) else { continue };
            let line = pane.selection.head.line;
            if let Some(nearest) = self.aligned.nearest_shown(side, line).filter(|&nearest| nearest != line) {
                pane.move_caret(Pos { line: nearest, column: 0 }, false);
            }
        }
        let side = self.cursor.unwrap_or(self.chosen);
        if let Some(pane) = self.pane(side) {
            let line = pane.selection.head.line;
            self.aligned.reveal(side, line);
        }
    }

    /// *Select All* in pane `side`: from the first shown line to the end of the last. Hidden lines
    /// between shown ones are selected too, as a selection is one stretch of text. With nothing
    /// shown, nothing is selected.
    pub(crate) fn select_shown(&mut self, side: usize) {
        // A command run since the last draw may have compared the panes again.
        self.aligned.refilter(self.filter);
        let span = self.aligned.shown_span(side);
        let all = self.filter == Filter::All;
        let Some(pane) = loaded_mut(&mut self.panes, side) else { return };
        match span {
            _ if all => pane.select_all(),
            Some((first, last)) => {
                pane.move_caret(Pos { line: first, column: 0 }, false);
                // The last line's own break too when the place after it ends the text.
                let end = pane.end();
                pane.move_caret(if end.line == last + 1 { end } else { pane.line_end(last) }, true);
            }
            None => pane.move_caret(pane.selection.head, false),
        }
    }

    /// After pane `side`'s caret moved by key from `from`: a caret on a hidden line goes on the way
    /// it moved to the next shown line, keeping a selection's anchor. A `vertical` move keeps
    /// `from`'s column; otherwise the caret lands at the shown line's end going up, or its start
    /// going down. Returns the caret's line.
    pub(crate) fn skip_hidden(&mut self, side: usize, from: Pos, vertical: bool) -> usize {
        let Some(pane) = loaded_mut(&mut self.panes, side) else { return from.line };
        let head = pane.selection.head;
        if self.filter == Filter::All {
            return head.line;
        }
        self.aligned.refilter(self.filter);
        match self.aligned.next_shown(side, head.line, head.line >= from.line) {
            Some(line) if line != head.line => {
                let to = if vertical {
                    Pos { line, column: from.column }
                } else if line < head.line {
                    pane.line_end(line)
                } else {
                    Pos { line, column: 0 }
                };
                let extend = !pane.selection.is_empty();
                pane.move_caret(to, extend);
                line
            }
            _ => head.line,
        }
    }
}
