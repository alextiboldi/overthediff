//! Folder Compare Info, the Legend and the status bar (E08-S1-T7).
//!
//! *Session > Folder Compare Info* counts the compared items by status, and *View > Legend* explains
//! every colour and glyph of the folder palette. Both are modal dialogs that take focus and give it
//! back on Esc or Close, and both commands also give their content as text, for scripts. The status
//! bar shows the scans' progress while they run, then the overall result.
//!
//! *Unspecified*, decided here:
//! - Info counts every item row once, as [`otd_folder_engine::Cues`] does: a folder pair whose
//!   listings are done counts only through its contents, never its own size and time; one not
//!   listed yet counts as not compared yet. Rows hidden on every side are left out.
//! - The overall result is Different when anything differs, is newer, is an orphan or can't be read;
//!   otherwise Not fully compared when anything is not compared yet (such as a subfolder pair not
//!   listed); otherwise Same. Two empty folders are the same. There is no result while a scan runs
//!   or while either base folder is missing or can't be listed.
//! - The Legend shows a folder not scanned yet as a yellow spot and one whose scan is pending as a
//!   hollow spot, since the colour spot is where each row shows its status.

use egui::{Id, Rect, Sense, StrokeKind, Ui, UiKind, Vec2};
use otd_core::command::{Args, Command, CommandError, Outcome};
use otd_core::i18n::{fmt, t};
use otd_core::session::Value;
use otd_folder_engine::{Row, ScanState, Status as RowStatus};
use otd_source::ItemKind;
use otd_ui::colour::{ColourRole, colour};
use otd_ui::{CLOSE, Field, Shell, Status, paint_lucide};

use crate::content::{CONTENT_COMPARISON, CONTENT_NOT_RUN};
use crate::look::{DIFFERENT, ERROR_GLYPH, ERROR_ICON, FILTERED, IS_DIFFERENT, IS_SAME, LINK_ICON, ORPHAN, SAME, UNKNOWN, UNSCANNED};
use crate::{FolderCompare, GLYPH, SPOT, paint_spot};

// English keys, drawn only through `t`.
/// The command's caption and its dialog's title.
pub const FOLDER_COMPARE_INFO: &str = "Folder Compare Info";
/// The command's caption and its dialog's title.
pub const LEGEND: &str = "Legend";
/// The status bar's result field: its label and the value when not everything could be compared
/// (the others are [`IS_SAME`] and [`IS_DIFFERENT`]).
pub const RESULT: &str = "Result";
pub const NOT_FULLY_COMPARED: &str = "Not fully compared";
/// The status bar's progress field while a scan runs.
pub const SCAN: &str = "Scan";
pub const FOLDERS_LISTED: &str = "{done} of {total} folders listed";
/// What Info says, and what the command fails with, while there is no comparison.
pub const NO_RESULT: &str = "There is no comparison result.";
/// Info's last line while a scan runs, so partial counts never read as final.
pub const STILL_SCANNING: &str = "Still scanning: these counts can change.";
// Info's counts.
pub const COUNT_SAME: &str = "Same";
pub const COUNT_DIFFERENT: &str = "Different";
pub const COUNT_LEFT_NEWER: &str = "Newer on the left";
pub const COUNT_RIGHT_NEWER: &str = "Newer on the right";
pub const COUNT_LEFT_ORPHANS: &str = "Only on the left";
pub const COUNT_RIGHT_ORPHANS: &str = "Only on the right";
pub const COUNT_UNKNOWN: &str = "Not compared yet";
pub const COUNT_ERRORS: &str = "Can't be read";
const FIELD: &str = "{label}: {value}";
const NOT_FOLDER_COMPARE: &str = "The active tab isn't a Folder Compare.";

/// Info's and the Legend's commands, which [`crate::commands`] lists. No default shortcuts: Q24
/// names none for them.
pub(crate) fn commands() -> Vec<Command<Shell>> {
    vec![
        Command::typed("folder-compare.info", FOLDER_COMPARE_INFO, &[], info).icon("info").menu(&["Session"]),
        Command::typed("folder-compare.legend", LEGEND, &[], legend).icon("palette").menu(&["View"]),
    ]
}

fn view(shell: &mut Shell) -> Result<&mut FolderCompare, CommandError> {
    shell.view_mut::<FolderCompare>().ok_or_else(|| CommandError::Failed(t(NOT_FOLDER_COMPARE).to_owned()))
}

/// Opens the Info dialog, and gives its lines as text, for scripts.
fn info(shell: &mut Shell, _: &Args) -> Outcome {
    let view = view(shell)?;
    view.open_dialog(Kind::Info);
    let lines = view.info_lines().ok_or_else(|| CommandError::Failed(t(NO_RESULT).to_owned()))?;
    Ok(Some(Value::Text(lines.join("\n"))))
}

/// Opens the Legend, and gives its entries as text, one per line, for scripts.
fn legend(shell: &mut Shell, _: &Args) -> Outcome {
    view(shell)?.open_dialog(Kind::Legend);
    Ok(Some(Value::Text(LEGEND_ENTRIES.iter().map(|entry| t(entry.text)).collect::<Vec<_>>().join("\n"))))
}

/// Folder Compare Info's counts: how many compared items have each status.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Info {
    pub same: usize,
    pub different: usize,
    pub left_newer: usize,
    pub right_newer: usize,
    pub left_orphans: usize,
    pub right_orphans: usize,
    pub unknown: usize,
    pub errors: usize,
}

impl Info {
    /// The counts of these rows and everything below them.
    pub fn of(rows: &[Row]) -> Self {
        let mut info = Self::default();
        info.count(rows);
        info
    }

    fn count(&mut self, rows: &[Row]) {
        for row in rows.iter().filter(|row| !row.hidden()) {
            self.count(&row.children);
            let folder = |item: &Option<otd_source::Item>| item.as_ref().is_some_and(|item| item.kind == ItemKind::Folder);
            // A compared folder pair is its contents, counted above.
            if folder(&row.left) && folder(&row.right) && row.status == RowStatus::Unknown && row.scan == ScanState::Done {
                continue;
            }
            *match row.status {
                RowStatus::Same => &mut self.same,
                RowStatus::Different => &mut self.different,
                RowStatus::LeftNewer => &mut self.left_newer,
                RowStatus::RightNewer => &mut self.right_newer,
                RowStatus::LeftOrphan => &mut self.left_orphans,
                RowStatus::RightOrphan => &mut self.right_orphans,
                RowStatus::Unknown => &mut self.unknown,
                RowStatus::Error(_) => &mut self.errors,
            } += 1;
        }
    }

    /// One "label: count" line per status, looked up.
    pub fn lines(&self) -> Vec<String> {
        [
            (COUNT_SAME, self.same),
            (COUNT_DIFFERENT, self.different),
            (COUNT_LEFT_NEWER, self.left_newer),
            (COUNT_RIGHT_NEWER, self.right_newer),
            (COUNT_LEFT_ORPHANS, self.left_orphans),
            (COUNT_RIGHT_ORPHANS, self.right_orphans),
            (COUNT_UNKNOWN, self.unknown),
            (COUNT_ERRORS, self.errors),
        ]
        .into_iter()
        .map(|(label, count)| fmt(t(FIELD), &[("label", t(label)), ("value", &count.to_string())]))
        .collect()
    }

    /// The overall result's English key.
    fn outcome(&self) -> &'static str {
        if self.different + self.left_newer + self.right_newer + self.left_orphans + self.right_orphans + self.errors > 0 {
            IS_DIFFERENT
        } else if self.unknown > 0 {
            NOT_FULLY_COMPARED
        } else {
            IS_SAME
        }
    }
}

/// How a Legend entry shows its colour or glyph.
#[derive(Debug, Clone, Copy)]
pub enum Swatch {
    /// The colour spot left of a name, in one part per colour.
    Spot(&'static [&'static ColourRole]),
    /// The colour spot's outline only.
    Hollow(&'static ColourRole),
    /// A Lucide glyph after a name.
    Glyph(&'static str, &'static ColourRole),
}

/// One line of the Legend: what is drawn and what it means (an English key).
#[derive(Debug, Clone, Copy)]
pub struct Entry {
    pub swatch: Swatch,
    pub text: &'static str,
}

/// Every colour and glyph the view can show, including the ones only later features use: teal
/// (E08-S2's filters) and the yellow and hollow folders (E08-S11's scanning).
pub const LEGEND_ENTRIES: [Entry; 10] = [
    Entry { swatch: Swatch::Spot(&[&SAME]), text: "Same" },
    Entry { swatch: Swatch::Spot(&[&DIFFERENT]), text: "Newer, or different" },
    Entry { swatch: Swatch::Spot(&[&UNKNOWN]), text: "Older, or not compared yet" },
    Entry { swatch: Swatch::Spot(&[&ORPHAN]), text: "Orphan: only on this side" },
    Entry { swatch: Swatch::Spot(&[&FILTERED]), text: "Hidden by a filter, shown because its partner isn't" },
    Entry { swatch: Swatch::Spot(&[&DIFFERENT, &ORPHAN]), text: "Folder holding several kinds of items: one part per colour" },
    Entry { swatch: Swatch::Spot(&[&UNSCANNED]), text: "Folder not scanned yet" },
    Entry { swatch: Swatch::Hollow(&UNKNOWN), text: "Folder waiting to be scanned" },
    Entry { swatch: Swatch::Glyph(LINK_ICON, &SAME), text: "Symbolic link or junction" },
    Entry { swatch: Swatch::Glyph(ERROR_ICON, &ERROR_GLYPH), text: "Folder that can't be opened, or item that can't be read" },
];

/// Which dialog is open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Info,
    Legend,
}

/// The open dialog.
pub(crate) struct Dialog {
    kind: Kind,
    /// It has been drawn once, and has taken focus.
    drawn: bool,
    /// What had focus when it opened, which gets it back when it closes.
    focus_back: Option<Id>,
}

impl FolderCompare {
    /// Both scans exist and both base folders could be listed, so the rows are a comparison.
    fn compared(&self) -> bool {
        matches!((&self.scans, &self.errors), ([Some(_), Some(_)], [None, None]))
    }

    /// Folder Compare Info's counts, while there is a comparison. They can still change while
    /// [`FolderCompare::scanning`].
    pub fn info(&self) -> Option<Info> {
        self.compared().then(|| Info::of(self.flat.rows(&self.rows)))
    }

    /// Info's lines, with a last one while a scan runs.
    fn info_lines(&self) -> Option<Vec<String>> {
        let mut lines = self.info()?.lines();
        if self.scanning() {
            lines.push(t(STILL_SCANNING).to_owned());
        }
        if self.content_not_run() {
            lines.push(fmt(t(FIELD), &[("label", t(CONTENT_COMPARISON)), ("value", t(CONTENT_NOT_RUN))]));
        }
        Some(lines)
    }

    fn open_dialog(&mut self, kind: Kind) {
        match &mut self.dialog {
            Some(dialog) => dialog.kind = kind,
            None => self.dialog = Some(Dialog { kind, drawn: false, focus_back: None }),
        }
    }

    /// The status bar: the scans' progress while they run, then the overall result.
    pub(crate) fn status_fields(&self) -> Status {
        let mut status = Status::default();
        if self.scanning() {
            let (done, pending) = self.scans.iter().flatten().fold((0, 0), |(done, pending), scan| (done + scan.done, pending + scan.pending));
            let value = fmt(t(FOLDERS_LISTED), &[("done", &done.to_string()), ("total", &(done + pending).to_string())]);
            status.fields.push(Field { id: "scan", label: t(SCAN).to_owned(), value });
        } else if let Some(info) = self.info() {
            status.result = Some(Field { id: "result", label: t(RESULT).to_owned(), value: t(info.outcome()).to_owned() });
        }
        if self.content_not_run() {
            status.fields.push(Field { id: "content", label: t(CONTENT_COMPARISON).to_owned(), value: t(CONTENT_NOT_RUN).to_owned() });
        }
        status
    }

    /// Draws the open dialog. It takes focus when it opens; Esc or Close closes it and gives focus
    /// back.
    pub(crate) fn dialog_ui(&mut self, ui: &Ui) {
        // After the rows are drawn, so egui doesn't drop it as a widget nobody drew yet.
        if let Some(id) = self.focus_back.take() {
            ui.memory_mut(|memory| memory.request_focus(id));
        }
        let lines = self.info_lines();
        let Some(dialog) = &mut self.dialog else { return };
        if !dialog.drawn {
            dialog.focus_back = ui.memory(|memory| memory.focused());
        }
        let title = t(match dialog.kind {
            Kind::Info => FOLDER_COMPARE_INFO,
            Kind::Legend => LEGEND,
        });
        let modal = egui::Modal::new(Id::unique("folder-compare-dialog")).show(ui.ctx(), |ui| {
            // The dialog's node is the modal's own `Ui`, an ancestor of this one: name it by the title.
            if let Some(frame) = ui.stack().iter().find(|frame| frame.kind() == Some(UiKind::Modal)) {
                ui.ctx().accesskit_node_builder(frame.unique_id, |node| {
                    node.set_label(title);
                    node.set_modal();
                });
            }
            ui.heading(title);
            match (dialog.kind, &lines) {
                (Kind::Info, Some(lines)) => {
                    for line in lines {
                        ui.label(line);
                    }
                }
                (Kind::Info, None) => {
                    ui.label(t(NO_RESULT));
                }
                (Kind::Legend, _) => {
                    for entry in &LEGEND_ENTRIES {
                        ui.horizontal(|ui| {
                            // The swatch only shows; the words beside it say what it means.
                            let (rect, _) = ui.allocate_exact_size(Vec2::splat(GLYPH), Sense::hover());
                            paint_swatch(ui, rect, entry.swatch);
                            ui.label(t(entry.text));
                        });
                    }
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
            self.focus_back = self.dialog.take().and_then(|dialog| dialog.focus_back);
            ui.ctx().request_repaint();
        }
    }
}

fn paint_swatch(ui: &Ui, rect: Rect, swatch: Swatch) {
    match swatch {
        Swatch::Spot(colours) => paint_spot(ui, rect, colours),
        Swatch::Hollow(role) => {
            let spot = Rect::from_center_size(egui::pos2(rect.left() + SPOT / 2.0, rect.center().y), Vec2::splat(SPOT));
            ui.painter().rect_stroke(spot, 0.0, (1.5, colour(ui.ctx(), role)), StrokeKind::Inside);
        }
        Swatch::Glyph(icon, role) => paint_lucide(ui, icon, rect.center(), GLYPH, colour(ui.ctx(), role)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::look::COLOUR_ROLES;

    fn shown(entry: &Entry) -> Vec<&'static str> {
        match entry.swatch {
            Swatch::Spot(colours) => colours.iter().map(|role| role.id).collect(),
            Swatch::Hollow(role) => vec![role.id],
            Swatch::Glyph(icon, role) => vec![icon, role.id],
        }
    }

    #[test]
    fn the_legend_shows_every_colour_role_and_glyph_of_the_view() {
        let shown: Vec<&str> = LEGEND_ENTRIES.iter().flat_map(shown).collect();
        for id in COLOUR_ROLES.iter().map(|role| role.id).chain([LINK_ICON, ERROR_ICON]) {
            assert!(shown.contains(&id), "the Legend shows {id}");
        }
    }
}
