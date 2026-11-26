//! The Session menu's Reload Files, Recompare Files, Swap Sides and Text Compare Info, and the
//! status bar's fields (E07-S1-T8; what they show is the Q25 decision in E07-S1).
//!
//! Reload Files reads both files again, first asking to save unsaved edits ([`Shell::save_first`]).
//! Recompare Files compares the panes as they stand, edits included, without reading the files.
//! Swap Sides exchanges the two panes, documents and all, and recompares. The status bar's result
//! and Info come from the comparison's classification, the one the panes draw. Neither shows a
//! result while there is no completed comparison: a pane is missing, or the last comparison
//! failed or was cancelled.

use std::any::Any;
use std::time::Instant;

use egui::{Id, Ui, UiKind};
use otd_core::command::{Args, Command, CommandError, Outcome as CommandOutcome};
use otd_core::i18n::{fmt, t};
use otd_core::session::Value;
use otd_text_engine::{Outcome, Statistics};
use otd_ui::{CLOSE, Field, Shell, Status};

use crate::aligned::Aligned;
use crate::open::NOT_TEXT_COMPARE;
use crate::{FIELD, TEXT_COMPARE, TextCompareView};

// The commands' captions and the text they show: English keys, drawn through `t`.
pub const RELOAD_FILES: &str = "Reload Files";
pub const RECOMPARE_FILES: &str = "Recompare Files";
pub const SWAP_SIDES: &str = "Swap Sides";
/// The command's caption and its dialog's title.
pub const TEXT_COMPARE_INFO: &str = "Text Compare Info";
/// The status bar's result field: its label and its values, the text engine's `Outcome`.
pub const RESULT: &str = "Result";
pub const SAME: &str = "Same";
pub const UNIMPORTANT_ONLY: &str = "Unimportant differences only";
pub const IMPORTANT: &str = "Important differences";
/// What Info says, and what the command fails with, while there is no completed comparison.
pub const NO_RESULT: &str = "There is no comparison result.";
// Info's figures.
pub const DIFFERENCE_SECTIONS: &str = "Difference sections";
pub const CHANGED_LINES: &str = "Changed lines";
pub const INSERTED_LINES: &str = "Inserted lines";
pub const DELETED_LINES: &str = "Deleted lines";
pub const LEFT_LINES: &str = "Lines on the left";
pub const RIGHT_LINES: &str = "Lines on the right";

/// The status bar's cursor field: its label, and its value.
pub const CURSOR: &str = "Cursor";
const LINE_COLUMN: &str = "Line {line}, column {column}";

const SESSION: &[&str] = &["Session"];

/// The Session menu's commands, which [`crate::commands`] lists. None has a default shortcut: Q24
/// names none for them.
pub(crate) fn commands() -> Vec<Command<Shell>> {
    [
        // The prompt is the window's, so Reload needs it.
        Command::typed("text-compare.reload_files", RELOAD_FILES, &[], reload_files).icon("refresh-cw").needs_window(),
        Command::typed("text-compare.recompare_files", RECOMPARE_FILES, &[], |shell, _| {
            view(shell)?.recompare_files();
            Ok(None)
        })
        .icon("git-compare"),
        Command::typed("text-compare.swap_sides", SWAP_SIDES, &[], |shell, _| {
            view(shell)?.swap_sides();
            Ok(None)
        })
        .icon("arrow-left-right"),
        Command::typed("text-compare.info", TEXT_COMPARE_INFO, &[], info).icon("info"),
    ]
    .into_iter()
    .map(|command| command.views(&[TEXT_COMPARE]).menu(SESSION))
    .collect()
}

fn view(shell: &mut Shell) -> Result<&mut TextCompareView, CommandError> {
    shell.view_mut::<TextCompareView>().ok_or_else(|| CommandError::Failed(t(NOT_TEXT_COMPARE).to_owned()))
}

fn reload_files(shell: &mut Shell, _: &Args) -> CommandOutcome {
    view(shell)?;
    shell.save_first(|view| {
        if let Some(view) = (view as &mut dyn Any).downcast_mut::<TextCompareView>() {
            view.reload_files();
        }
    });
    Ok(None)
}

/// Opens the Info dialog, and gives its figures as text, one per line, for scripts.
fn info(shell: &mut Shell, _: &Args) -> CommandOutcome {
    let view = view(shell)?;
    view.info.get_or_insert(InfoDialog { drawn: false, focus_back: None });
    let info = view.info().ok_or_else(|| CommandError::Failed(t(NO_RESULT).to_owned()))?;
    Ok(Some(Value::Text(info.lines().join("\n"))))
}

/// Text Compare Info's figures, counted in lines (E07-S1 Decisions, Q25).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Info {
    pub sections: usize,
    /// Rows with a line on both sides that differ.
    pub changed: usize,
    /// Right lines opposite a gap, reading the left side as the earlier one.
    pub inserted: usize,
    /// Left lines opposite a gap.
    pub deleted: usize,
    /// The lines in each pane as it stands.
    pub left_lines: usize,
    pub right_lines: usize,
}

impl Info {
    fn of(statistics: Statistics) -> Self {
        let Statistics { important, unimportant, same, left_orphans, right_orphans, sections } = statistics;
        // Orphans are always important, so they are inside `important`.
        let changed = (important + unimportant).saturating_sub(left_orphans + right_orphans);
        Self {
            sections,
            changed,
            inserted: right_orphans,
            deleted: left_orphans,
            left_lines: same + changed + left_orphans,
            right_lines: same + changed + right_orphans,
        }
    }

    /// One "label: value" line per figure, looked up.
    pub fn lines(&self) -> Vec<String> {
        [
            (DIFFERENCE_SECTIONS, self.sections),
            (CHANGED_LINES, self.changed),
            (INSERTED_LINES, self.inserted),
            (DELETED_LINES, self.deleted),
            (LEFT_LINES, self.left_lines),
            (RIGHT_LINES, self.right_lines),
        ]
        .into_iter()
        .map(|(label, count)| fmt(t(FIELD), &[("label", t(label)), ("value", &count.to_string())]))
        .collect()
    }
}

fn outcome_name(outcome: Outcome) -> &'static str {
    match outcome {
        Outcome::Same => SAME,
        Outcome::UnimportantOnly => UNIMPORTANT_ONLY,
        Outcome::Important => IMPORTANT,
    }
}

/// The open Info dialog.
pub(crate) struct InfoDialog {
    /// It has been drawn once, and has taken focus.
    drawn: bool,
    /// What had focus when it opened, which gets it back when it closes.
    focus_back: Option<Id>,
}

impl TextCompareView {
    /// The completed comparison's overall result and statistics: `None` with a pane missing, or
    /// after a failed or cancelled comparison, so neither reads as "Same".
    fn result(&self) -> Option<(Outcome, Statistics)> {
        if self.comparison.is_some() { None } else { self.aligned.result() }
    }

    /// Text Compare Info's figures, while there is a completed comparison.
    pub fn info(&self) -> Option<Info> {
        self.result().map(|(_, statistics)| Info::of(statistics))
    }

    /// Reload Files: reads both files again and compares them. Edits are lost, so the command asks
    /// to save them first.
    pub fn reload_files(&mut self) {
        for side in 0..self.loaded.len() {
            self.retry(side);
        }
        self.reload_if_changed();
    }

    /// Recompare Files: compares the panes as they stand, edits included, without reading the
    /// files again, and empties both panes' undo history.
    pub fn recompare_files(&mut self) {
        for pane in self.panes.iter_mut().flatten().flatten() {
            pane.history.clear();
        }
        self.compare_as_they_stand();
    }

    /// Compares the panes as they stand, with each pane's undo history kept.
    pub(crate) fn compare_as_they_stand(&mut self) {
        // This comparison completes, which lifts a failed or cancelled one's read-only panes.
        self.comparison = None;
        let old = std::mem::replace(&mut self.aligned, Aligned::new(None, None, Default::default()));
        self.compare(Instant::now());
        // The panes stay where the user scrolled them; only reading new files starts at the top.
        self.aligned.keep_scroll(&old);
    }

    /// Swap Sides: the left pane's file, with its edits, goes right and the right one left; then
    /// they are compared.
    pub fn swap_sides(&mut self) {
        // Loads what hasn't loaded yet, so every per-side list has both sides.
        self.reload_if_changed();
        if self.session.specs.len() < 2 {
            return;
        }
        self.session.specs.swap(0, 1);
        self.loaded.swap(0, 1);
        self.panes.swap(0, 1);
        self.edits.swap(0, 1);
        self.synced.swap(0, 1);
        self.save_errors.swap(0, 1);
        self.swap_overrides();
        // Each pane's undo history goes with it.
        self.compare_as_they_stand();
    }

    /// The status bar's fields: the overall result, the cursor's line and column in the pane that
    /// last had it (from 1, the file's own lines, in characters), insert/overwrite and the load
    /// time.
    pub(crate) fn status_fields(&self) -> Status {
        let result =
            self.result().map(|(outcome, _)| Field { id: "result", label: t(RESULT).to_owned(), value: t(outcome_name(outcome)).to_owned() });
        let cursor = self.cursor.and_then(|side| self.pane(side)).map(|pane| {
            let head = pane.selection.head;
            let value = fmt(t(LINE_COLUMN), &[("line", &(head.line + 1).to_string()), ("column", &(head.column + 1).to_string())]);
            Field { id: "cursor", label: t(CURSOR).to_owned(), value }
        });
        Status { result, fields: cursor.into_iter().chain(self.find_field()).collect(), insert: Some(self.insert), load_time: self.load_time }
    }

    /// Draws the Info dialog while it is open. It takes focus when it opens; Esc or Close closes it
    /// and gives focus back.
    pub(crate) fn info_ui(&mut self, ui: &Ui) {
        // After the panes are drawn, so egui doesn't drop it as a widget nobody drew yet.
        if let Some(id) = self.refocus.take() {
            ui.memory_mut(|memory| memory.request_focus(id));
        }
        let lines = self.info().map(|info| info.lines());
        let Some(dialog) = &mut self.info else { return };
        if !dialog.drawn {
            dialog.focus_back = ui.memory(|memory| memory.focused());
        }
        let title = t(TEXT_COMPARE_INFO);
        let modal = egui::Modal::new(Id::unique("text-compare-info")).show(ui.ctx(), |ui| {
            // The dialog's node is the modal's own `Ui`, an ancestor of this one: name it by the title.
            if let Some(frame) = ui.stack().iter().find(|frame| frame.kind() == Some(UiKind::Modal)) {
                ui.ctx().accesskit_node_builder(frame.unique_id, |node| {
                    node.set_label(title);
                    node.set_modal();
                });
            }
            ui.heading(title);
            match &lines {
                Some(lines) => {
                    for line in lines {
                        ui.label(line);
                    }
                }
                None => {
                    ui.label(t(NO_RESULT));
                }
            }
            let close = ui.button(t(CLOSE));
            if !dialog.drawn {
                close.request_focus();
            }
            close.clicked()
        });
        dialog.drawn = true;
        if modal.inner || modal.should_close() {
            // On the next draw: focus can't leave the dialog in the frame that closes it.
            self.refocus = self.info.take().and_then(|dialog| dialog.focus_back);
            ui.ctx().request_repaint();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn outcome(view: &TextCompareView) -> Option<Outcome> {
        view.result().map(|(outcome, _)| outcome)
    }

    #[test]
    fn recompare_compares_the_panes_as_edited_without_reading_the_files_and_reload_reads_them() {
        let folder = tempfile::tempdir().unwrap();
        let [left, right] = [("left.txt", "same\nleft\n"), ("right.txt", "same\nright\n")].map(|(name, text)| {
            let path = folder.path().join(name);
            fs::write(&path, text).unwrap();
            path
        });
        let mut view = TextCompareView::new();
        view.session.specs = vec![left.display().to_string(), right.display().to_string()];
        view.reload_if_changed();
        assert_eq!(outcome(&view), Some(Outcome::Important));

        // An edit in memory: the right pane takes the left's text. The right file changes on disk too.
        let edited = view.pane(0).unwrap().lines.clone();
        if let Some(Some(Ok(pane))) = view.panes.get_mut(1) {
            pane.lines.clone_from(&edited);
        }
        fs::write(&right, "changed on disk\n").unwrap();
        view.recompare_files();
        assert_eq!(outcome(&view), Some(Outcome::Same), "Recompare compares the edit");
        assert_eq!(view.pane(1).unwrap().lines, edited, "and keeps it: the file isn't read");

        view.reload_files();
        assert_eq!(view.pane(1).unwrap().lines[0].text, "changed on disk", "Reload reads the file again");
        assert_eq!(outcome(&view), Some(Outcome::Important));
    }
}
