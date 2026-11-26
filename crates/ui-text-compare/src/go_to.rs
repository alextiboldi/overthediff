//! *Search > Go To* (E07-S6-T1): moves the caret in the chosen pane to a line and column, and
//! scrolls there. Both panes share one vertical offset, so the other pane shows the same row.
//!
//! The menu command opens a dialog with a Line and a Column field. The hidden *Go To Line* takes
//! them as arguments, so a script can move the caret without the window. Lines and columns count
//! from 1, as the status bar shows them; a value out of range goes to the nearest valid position
//! (*inferred*).

use egui::accesskit::Live;
use egui::{Id, Key, TextEdit, Ui, UiKind};
use otd_core::command::{Args, Command, CommandError, Kind, Outcome, Param, State};
use otd_core::i18n::t;
use otd_text_compare::Pos;
use otd_ui::Shell;

use crate::open::NOT_TEXT_COMPARE;
use crate::{TEXT_COMPARE, TextCompareView};

/// The menu command's caption and its dialog's title.
pub const GO_TO: &str = "Go To";
/// The hidden command that takes the line and column as arguments.
pub const GO_TO_LINE: &str = "Go To Line";
pub const LINE: &str = "Line";
pub const COLUMN: &str = "Column";
pub const CANCEL: &str = "Cancel";
/// What the dialog says when a field holds something other than a whole number.
pub const NOT_A_NUMBER: &str = "Type the line, and optionally the column, as whole numbers.";
pub(crate) const NOT_LOADED: &str = "The chosen pane has no file loaded.";

/// Any number: one out of range goes to the nearest valid position rather than failing.
const ANY: Kind = Kind::Number { min: f64::MIN, max: f64::MAX };
const PARAMS: &[Param] = &[Param::required("line", ANY), Param::optional("column", ANY)];

/// Go To, in the Search menu, and the hidden Go To Line. Neither has a default shortcut: no
/// platform standard or catalogue shortcut names one (Q24).
pub(crate) fn commands() -> Vec<Command<Shell>> {
    let loaded = |shell: &Shell| State::enabled(shell.view::<TextCompareView>().is_some_and(|view| view.pane(view.chosen).is_some()));
    vec![
        Command::new("text-compare.go_to", GO_TO, |shell: &mut Shell| {
            if let Some(view) = shell.view_mut::<TextCompareView>() {
                view.go_to = Some(GoToDialog::default());
            }
        })
        .state(loaded)
        .icon("corner-down-right")
        .menu(&["Search"])
        .needs_window(),
        Command::typed("text-compare.go_to_line", GO_TO_LINE, PARAMS, go_to_line).state(loaded),
    ]
    .into_iter()
    .map(|command| command.views(&[TEXT_COMPARE]))
    .collect()
}

fn go_to_line(shell: &mut Shell, args: &Args) -> Outcome {
    let view = shell.view_mut::<TextCompareView>().ok_or_else(|| CommandError::Failed(t(NOT_TEXT_COMPARE).to_owned()))?;
    // `as` saturates, so a negative or huge number lands at the nearest end.
    let [line, column] = ["line", "column"].map(|name| args.number(name).map_or(1, |n| n as usize));
    view.go_to_line(line, column).ok_or_else(|| CommandError::Failed(t(NOT_LOADED).to_owned()))?;
    Ok(None)
}

/// The open Go To dialog.
#[derive(Default)]
pub(crate) struct GoToDialog {
    line: String,
    column: String,
    /// It has been drawn once.
    drawn: bool,
    /// The field to give focus to on the next draw: the Line field when it opens, and the field
    /// that failed when Enter or Go To is refused.
    focus: Option<&'static str>,
    /// What had focus when it opened, which gets it back when it is cancelled.
    focus_back: Option<Id>,
    /// A field didn't hold a number when the user pressed Go To.
    invalid: bool,
}

/// `text` as a whole number: a negative one is 0, and one too big for `usize` is `usize::MAX`.
fn number(text: &str) -> Option<usize> {
    let text = text.trim();
    let (negative, digits) = text.strip_prefix('-').map_or((false, text), |rest| (true, rest));
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some(if negative { 0 } else { digits.parse().unwrap_or(usize::MAX) })
}

impl TextCompareView {
    /// Go To: puts the caret of the chosen pane on line `line`, column `column` (both from 1), or
    /// the nearest valid position, and scrolls both panes there. Returns where the caret went, or
    /// `None` when the chosen pane has no file loaded.
    pub fn go_to_line(&mut self, line: usize, column: usize) -> Option<Pos> {
        let side = self.chosen;
        let pane = crate::edit::loaded_mut(&mut self.panes, side)?;
        pane.move_caret(Pos { line: line.saturating_sub(1), column: column.saturating_sub(1) }, false);
        let head = pane.selection.head;
        self.aligned.reveal(side, head.line);
        self.focus = Some(side);
        self.cursor = Some(side);
        Some(head)
    }

    /// Draws the Go To dialog while it is open. It takes focus in the Line field; Enter or its Go
    /// To button goes there and gives focus to the pane, and Esc or Cancel gives focus back.
    pub(crate) fn go_to_ui(&mut self, ui: &Ui) {
        let Some(dialog) = &mut self.go_to else { return };
        if !dialog.drawn {
            dialog.focus_back = ui.memory(|memory| memory.focused());
            dialog.focus = Some(LINE);
        }
        let title = t(GO_TO);
        let modal = egui::Modal::new(Id::unique("text-compare-go-to")).show(ui.ctx(), |ui| {
            if let Some(frame) = ui.stack().iter().find(|frame| frame.kind() == Some(UiKind::Modal)) {
                ui.ctx().accesskit_node_builder(frame.unique_id, |node| {
                    node.set_label(title);
                    node.set_modal();
                });
            }
            ui.heading(title);
            let mut enter = false;
            for (caption, text) in [(LINE, &mut dialog.line), (COLUMN, &mut dialog.column)] {
                ui.horizontal(|ui| {
                    let label = ui.label(t(caption));
                    let field = ui.add(TextEdit::singleline(text).desired_width(80.0)).labelled_by(label.id);
                    if dialog.focus == Some(caption) {
                        field.request_focus();
                    }
                    enter |= field.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
                });
            }
            dialog.focus = None;
            if dialog.invalid {
                let message = ui.label(t(NOT_A_NUMBER));
                // Announced: focus goes back to the field that failed.
                ui.ctx().accesskit_node_builder(message.id, |node| node.set_live(Live::Polite));
            }
            let (go, cancel) = ui.horizontal(|ui| (ui.button(t(GO_TO)).clicked(), ui.button(t(CANCEL)).clicked())).inner;
            (go || enter, cancel)
        });
        dialog.drawn = true;
        let (go, cancel) = modal.inner;
        // Asked once: it consumes Esc.
        let cancel = !go && (cancel || modal.should_close());
        if go || cancel {
            ui.ctx().request_repaint();
        }
        if go {
            let column = if dialog.column.trim().is_empty() { Some(1) } else { number(&dialog.column) };
            match (number(&dialog.line), column) {
                (Some(line), Some(column)) => {
                    self.go_to = None;
                    // The pane takes focus on the next draw, once the dialog is gone.
                    self.go_to_line(line, column);
                }
                (line, _) => {
                    dialog.invalid = true;
                    dialog.focus = Some(if line.is_none() { LINE } else { COLUMN });
                }
            }
        } else if cancel {
            self.refocus = self.go_to.take().and_then(|dialog| dialog.focus_back);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_number_out_of_range_is_kept_at_its_nearest_end_and_text_is_refused() {
        assert_eq!(number(" 120 "), Some(120));
        assert_eq!(number("-3"), Some(0));
        assert_eq!(number("99999999999999999999999"), Some(usize::MAX));
        for text in ["", "-", "12a", "1.5", "120:5"] {
            assert_eq!(number(text), None, "{text:?}");
        }
    }
}
