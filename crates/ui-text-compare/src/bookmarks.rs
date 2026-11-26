//! Bookmarks 0–9 (E07-S6-T2), in the Search menu: *Toggle Bookmark ▸ 0–9* sets or clears that
//! bookmark on the chosen pane's caret line, *Go To Bookmark ▸ 0–9* puts the caret on its line and
//! scrolls both panes there, and *Clear Bookmarks* removes every bookmark in both panes. The
//! bookmarks live in each pane's document (`otd_text_compare::Bookmarks`), so they move with
//! their lines as the user edits. The gutter marks each bookmarked line with its number, which is
//! also a node of its own, and each toggle and clear is announced.
//!
//! None has a default shortcut: neither the story nor a platform standard names one (Q24).

use egui::accesskit::Live;
use egui::{Align2, Color32, FontId, Rect, Role, Sense, Stroke, StrokeKind, Ui, WidgetInfo};
use otd_core::command::{Command, State};
use otd_core::i18n::{fmt, t};
use otd_text_compare::{Bookmarks, Pos};
use otd_ui::Shell;

use crate::edit::{loaded_mut, on_view};
use crate::{TEXT_COMPARE, TextCompareView};

// The commands' captions and what they announce: English keys, drawn through `t`.
pub const TOGGLE_BOOKMARK: &str = "Toggle Bookmark";
pub const GO_TO_BOOKMARK: &str = "Go To Bookmark";
pub const CLEAR_BOOKMARKS: &str = "Clear Bookmarks";
/// A gutter mark's accessible name, one per number on the line.
pub const BOOKMARK: &str = "Bookmark {number}";
const SET: &str = "Bookmark {number} set on line {line}";
const CLEARED: &str = "Bookmark {number} cleared";
const ALL_CLEARED: &str = "Bookmarks cleared";

/// One command per bookmark number, for each of `$n`.
macro_rules! numbered {
    ($($n:literal)*) => {
        [$(
            Command::new(concat!("text-compare.toggle_bookmark_", $n), concat!("Toggle Bookmark ", $n), |shell: &mut Shell| {
                on_view(shell, |view| view.toggle_bookmark($n))
            })
            .state(|shell: &Shell| when(shell, TextCompareView::can_toggle_bookmark))
            .icon("bookmark")
            .menu(&["Search", TOGGLE_BOOKMARK]),
        )*
        $(
            Command::new(concat!("text-compare.go_to_bookmark_", $n), concat!("Go To Bookmark ", $n), |shell: &mut Shell| {
                on_view(shell, |view| view.go_to_bookmark($n))
            })
            .state(|shell: &Shell| when(shell, |view| view.bookmark($n).is_some()))
            .menu(&["Search", GO_TO_BOOKMARK]),
        )*]
    };
}

/// The Search menu's bookmark commands, which [`crate::commands`] lists.
pub(crate) fn commands() -> Vec<Command<Shell>> {
    numbered!(0 1 2 3 4 5 6 7 8 9)
        .into_iter()
        .chain([Command::new("text-compare.clear_bookmarks", CLEAR_BOOKMARKS, |shell: &mut Shell| on_view(shell, TextCompareView::clear_bookmarks))
            .state(|shell: &Shell| when(shell, |view| view.panes().any(|pane| !pane.bookmarks.is_empty())))
            .icon("bookmark-x")
            .menu(&["Search"])])
        .map(|command| command.views(&[TEXT_COMPARE]))
        .collect()
}

fn when(shell: &Shell, can: fn(&TextCompareView) -> bool) -> State {
    State::enabled(shell.view::<TextCompareView>().is_some_and(can))
}

impl TextCompareView {
    /// Whether the chosen pane has a line to bookmark.
    pub fn can_toggle_bookmark(&self) -> bool {
        self.pane(self.chosen).is_some_and(|pane| !pane.lines.is_empty())
    }

    /// Bookmark `number`'s line in the chosen pane, if it is set.
    pub fn bookmark(&self, number: usize) -> Option<usize> {
        self.pane(self.chosen)?.bookmarks.get(number)
    }

    /// *Toggle Bookmark*: sets bookmark `number` on the chosen pane's caret line, or clears it
    /// there. After a final line break the caret has no line, so the last line takes it.
    pub fn toggle_bookmark(&mut self, number: usize) {
        self.focus = Some(self.chosen);
        let Some(pane) = loaded_mut(&mut self.panes, self.chosen) else { return };
        let Some(last) = pane.lines.len().checked_sub(1) else { return };
        let line = pane.selection.head.line.min(last);
        let number_text = number.to_string();
        self.bookmark_said = Some(if pane.bookmarks.toggle(number, line) {
            fmt(t(SET), &[("number", &number_text), ("line", &(line + 1).to_string())])
        } else {
            fmt(t(CLEARED), &[("number", &number_text)])
        });
    }

    /// *Go To Bookmark*: the caret to the start of bookmark `number`'s line in the chosen pane, with
    /// both panes scrolled to it.
    pub fn go_to_bookmark(&mut self, number: usize) {
        let side = self.chosen;
        self.focus = Some(side);
        let Some(pane) = loaded_mut(&mut self.panes, side) else { return };
        let Some(line) = pane.bookmarks.get(number) else { return };
        pane.move_caret(Pos { line, column: 0 }, false);
        self.aligned.reveal(side, line);
    }

    /// *Clear Bookmarks*: removes every bookmark in both panes.
    pub fn clear_bookmarks(&mut self) {
        for side in 0..self.panes.len() {
            if let Some(pane) = loaded_mut(&mut self.panes, side) {
                pane.bookmarks.clear();
            }
        }
        self.bookmark_said = Some(t(ALL_CLEARED).to_owned());
        self.focus = Some(self.chosen);
    }

    fn panes(&self) -> impl Iterator<Item = &otd_text_compare::Pane> {
        (0..self.panes.len()).filter_map(|side| self.pane(side))
    }

    /// The last toggle or clear, as a polite live region, so it is heard while focus stays in the
    /// editor; the gutter shows it to everyone else. It has no visible text of its own.
    pub(crate) fn bookmark_ui(&self, ui: &mut Ui) {
        if let Some(said) = &self.bookmark_said {
            let response = ui.interact(Rect::from_min_size(ui.cursor().min, egui::Vec2::ZERO), ui.scope_id().with("bookmark-said"), Sense::hover());
            response.widget_info(|| WidgetInfo::labeled(Role::Label, true, said));
            ui.ctx().accesskit_node_builder(response.id, |node| node.set_live(Live::Polite));
        }
    }
}

/// Paints line `line`'s gutter mark in `cell`, a digit wide: its lowest bookmark number, boxed so
/// it doesn't read as a line number, in `colour`. Each of its numbers is a node named
/// "Bookmark N", so every one is heard even where one digit shows.
pub(crate) fn mark(ui: &Ui, bookmarks: &Bookmarks, line: usize, cell: Rect, font: &FontId, colour: Color32) {
    let mut numbers = bookmarks.on(line).peekable();
    let Some(&first) = numbers.peek() else { return };
    let painter = ui.painter();
    painter.rect_stroke(cell.shrink(1.0), 1.0, Stroke::new(1.0, colour), StrokeKind::Inside);
    painter.text(cell.center(), Align2::CENTER_CENTER, first.to_string(), font.clone(), colour);
    for number in numbers {
        let name = fmt(t(BOOKMARK), &[("number", &number.to_string())]);
        ui.interact(cell, ui.scope_id().with(("bookmark", line, number)), Sense::hover())
            .widget_info(|| WidgetInfo::labeled(Role::Label, true, &name));
    }
}
