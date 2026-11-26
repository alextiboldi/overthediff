//! *View > Show Whitespace* (E07-S9-T1): while it is on, both panes mark each space, each tab
//! and each line's break, with one glyph for a Windows (CR LF), one for a Mac (CR) and one for a
//! Unix (LF) break. The glyphs are painted over the laid-out line, so the text, its columns, the
//! caret and the line's accessible name stay as they are.

use egui::text::CCursor;
use egui::{Align2, Color32, FontId, Galley, Painter, Pos2, pos2};
use otd_core::command::{Args, Command, CommandError, Mark, Outcome, State};
use otd_core::i18n::t;
use otd_text_compare::LineEnding;
use otd_ui::Shell;

use crate::open::NOT_TEXT_COMPARE;
use crate::{TEXT_COMPARE, TextCompareView};

pub const SHOW_WHITESPACE: &str = "Show Whitespace";

/// The View menu's on/off item. Q24 names no shortcut for it.
pub(crate) fn commands() -> Vec<Command<Shell>> {
    vec![
        Command::typed("text-compare.show_whitespace", SHOW_WHITESPACE, &[], toggle).views(&[TEXT_COMPARE]).icon("pilcrow").menu(&["View"]).state(
            |shell| State { mark: Some(Mark::Check(shell.view::<TextCompareView>().is_some_and(|view| view.whitespace))), ..State::default() },
        ),
    ]
}

fn toggle(shell: &mut Shell, _: &Args) -> Outcome {
    let view = shell.view_mut::<TextCompareView>().ok_or_else(|| CommandError::Failed(t(NOT_TEXT_COMPARE).to_owned()))?;
    view.whitespace = !view.whitespace;
    Ok(None)
}

impl TextCompareView {
    /// Whether *Show Whitespace* is on.
    pub fn show_whitespace(&self) -> bool {
        self.whitespace
    }
}

/// The glyph a space or tab shows as.
fn glyph(c: char) -> Option<&'static str> {
    match c {
        ' ' => Some("·"),
        '\t' => Some("→"),
        _ => None,
    }
}

/// The glyph a line break shows as, one per style.
pub(crate) fn end_glyph(ending: LineEnding) -> Option<&'static str> {
    match ending {
        LineEnding::None => None,
        LineEnding::CrLf => Some("↵"),
        LineEnding::Cr => Some("←"),
        LineEnding::Lf => Some("↓"),
    }
}

/// Paints the whitespace glyphs of a line laid out as `galley` at `origin`, ended by `ending`: each
/// character's in `colour` of its byte offset, and the line break's in `break_colour`.
pub(crate) fn paint(
    painter: &Painter,
    (galley, origin, ending): (&Galley, Pos2, LineEnding),
    colour: impl Fn(usize) -> Color32,
    break_colour: Color32,
    font: &FontId,
) {
    let x = |index: usize| galley.pos_from_cursor(CCursor::new(index));
    for (index, (at, c)) in galley.text().char_indices().enumerate() {
        let Some(mark) = glyph(c) else { continue };
        let (start, end) = (x(index), x(index + 1));
        let colour = colour(at);
        // A space's dot in the middle of its cell; a tab's arrow at its start, since a tab is wide.
        let (at, align) = if c == ' ' && start.min.y == end.min.y {
            (pos2((start.min.x + end.min.x) / 2.0, start.center().y), Align2::CENTER_CENTER)
        } else {
            (pos2(start.min.x, start.center().y), Align2::LEFT_CENTER)
        };
        painter.text(origin + at.to_vec2(), align, mark, font.clone(), colour);
    }
    if let Some(mark) = end_glyph(ending) {
        let end = x(galley.text().chars().count());
        painter.text(origin + pos2(end.min.x, end.center().y).to_vec2(), Align2::LEFT_CENTER, mark, font.clone(), break_colour);
    }
}
