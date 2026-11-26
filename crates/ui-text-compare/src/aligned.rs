//! The aligned comparison (E07-S1-T4): both panes draw the text engine's rows, a line or a gap on
//! each side, at one fixed row height, so matching lines sit on the same row. Gap rows have no
//! line number. Important differences draw in red text on a light-red line background, with the
//! changed characters inside a changed line on a stronger background and underlined, so colour
//! isn't the only signal. Unimportant differences, by the importance rules (E07-S4-T5), draw the
//! same way in blue, with a dashed underline instead of a solid one; a line holding both kinds
//! gets the light-red background, because red wins. A gutter marker and each line's accessible
//! description say how the line differs. The two panes share one vertical scroll offset.
//!
//! Under a display filter both panes draw only the rows it leaves, and a summary row for each hidden
//! run while *Show filtered line counts* is on (E07-S3-T3, `filtered.rs`).
//!
//! Each pane also draws its caret and selection, and maps a pointer press to a place in its text,
//! for the editor (E07-S1-T5, `edit.rs`).

use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;

use egui::scroll_area::{ScrollBarVisibility, ScrollSource};
use egui::text::CCursor;
use egui::text::{LayoutJob, TextFormat};
use egui::{Align2, Color32, FontId, Galley, Pos2, Rect, Role, ScrollArea, Sense, Shape, Stroke, TextStyle, Ui, WidgetInfo, pos2, vec2};
use otd_core::colour::{ColourRole, Contrast, Rgba};
use otd_core::i18n::{t, trn};
use otd_text_compare::{Bookmarks, Differences, Filter, Line, LineEnding, Pos, Selection};
use otd_text_engine::{
    BuiltInRules, CharDifferences, CharRange, Importance, Options, Outcome, Row, RowKind, Side, Statistics, char_differences_with, classify,
    compare_with,
};
use otd_ui::colour::{BACKGROUND, SELECTION, SELECTION_TEXT, TEXT, colour};

use crate::filtered::{self, LINE_HIDDEN, LINES_HIDDEN, Slot};
use crate::whitespace;

/// An important difference's text.
pub const IMPORTANT_TEXT: ColourRole =
    ColourRole { id: "text-compare.important.text", light: Rgba::rgb(0xA4, 0x00, 0x00), dark: Rgba::rgb(0xFF, 0x9C, 0x9C) };
/// Behind a line with an important difference.
pub const IMPORTANT_BACKGROUND: ColourRole =
    ColourRole { id: "text-compare.important.background", light: Rgba::rgb(0xFF, 0xE4, 0xE4), dark: Rgba::rgb(0x4D, 0x1F, 0x1F) };
/// Behind the changed characters inside a changed line.
pub const IMPORTANT_CHARACTERS: ColourRole =
    ColourRole { id: "text-compare.important.characters", light: Rgba::rgb(0xFF, 0xB8, 0xB8), dark: Rgba::rgb(0x6E, 0x28, 0x28) };
/// An unimportant difference's text.
pub const UNIMPORTANT_TEXT: ColourRole =
    ColourRole { id: "text-compare.unimportant.text", light: Rgba::rgb(0x00, 0x45, 0xA8), dark: Rgba::rgb(0x9C, 0xC8, 0xFF) };
/// Behind a line whose differences are all unimportant.
pub const UNIMPORTANT_BACKGROUND: ColourRole =
    ColourRole { id: "text-compare.unimportant.background", light: Rgba::rgb(0xE6, 0xF0, 0xFF), dark: Rgba::rgb(0x1C, 0x2C, 0x46) };
/// Behind the unimportant changed characters inside a changed line.
pub const UNIMPORTANT_CHARACTERS: ColourRole =
    ColourRole { id: "text-compare.unimportant.characters", light: Rgba::rgb(0xC6, 0xDC, 0xFF), dark: Rgba::rgb(0x2A, 0x40, 0x66) };
/// Line numbers in the gutter.
pub const LINE_NUMBER: ColourRole =
    ColourRole { id: "text-compare.line-number", light: Rgba::rgb(0x61, 0x61, 0x61), dark: Rgba::rgb(0x9A, 0x9A, 0x9A) };

/// Text Compare's colour roles, which the app registers.
pub const ROLES: [ColourRole; 7] =
    [IMPORTANT_TEXT, IMPORTANT_BACKGROUND, IMPORTANT_CHARACTERS, UNIMPORTANT_TEXT, UNIMPORTANT_BACKGROUND, UNIMPORTANT_CHARACTERS, LINE_NUMBER];

/// What each role is drawn on.
pub const CONTRAST: [Contrast; 13] = [
    Contrast::text(&IMPORTANT_TEXT, &IMPORTANT_BACKGROUND),
    Contrast::text(&IMPORTANT_TEXT, &IMPORTANT_CHARACTERS),
    Contrast::text(&UNIMPORTANT_TEXT, &UNIMPORTANT_BACKGROUND),
    Contrast::text(&UNIMPORTANT_TEXT, &UNIMPORTANT_CHARACTERS),
    Contrast::text(&LINE_NUMBER, &BACKGROUND),
    Contrast::text(&LINE_NUMBER, &IMPORTANT_BACKGROUND),
    Contrast::text(&LINE_NUMBER, &UNIMPORTANT_BACKGROUND),
    // A line's gutter marker.
    Contrast::text(&IMPORTANT_TEXT, &BACKGROUND),
    Contrast::text(&UNIMPORTANT_TEXT, &BACKGROUND),
    // An unimportant range's dashed underline, where it is selected.
    Contrast::control(&UNIMPORTANT_TEXT, &SELECTION),
    // The focused pane's focus ring, where it crosses a changed line.
    Contrast::control(&SELECTION_TEXT, &IMPORTANT_BACKGROUND),
    Contrast::control(&SELECTION_TEXT, &UNIMPORTANT_BACKGROUND),
    Contrast::control(&SELECTION_TEXT, &UNIMPORTANT_CHARACTERS),
];

// A line's accessible description: how it differs.
pub(crate) const CHANGED: &str = "Changed";
const APPROXIMATE: &str = "Changed, character differences approximate";
pub(crate) const UNIMPORTANT: &str = "Changed, unimportant differences only";
const UNIMPORTANT_APPROXIMATE: &str = "Changed, unimportant differences only, character differences approximate";
pub(crate) const DELETED: &str = "Deleted";
pub(crate) const INSERTED: &str = "Inserted";
/// An empty line's accessible name, since it has no text to be named by.
const BLANK: &str = "Blank line";

/// The rows both panes draw, and what the comparison found.
pub struct Aligned {
    rows: Vec<Row>,
    /// Each row's importance (`None` for a same row), when both panes are loaded.
    importance: Vec<Option<Importance>>,
    /// The overall result and the counts behind it, when both panes are loaded.
    result: Option<(Outcome, Statistics)>,
    /// Character differences of the changed rows drawn so far, by row.
    chars: HashMap<usize, CharDifferences>,
    /// Digits in the longest line number on either side, so both gutters are one width.
    digits: usize,
    /// How many lines each side has.
    lines: [usize; 2],
    /// The Importance tab's rules and options it was compared with.
    rules: (BuiltInRules, Options),
    /// The vertical scroll offset both panes share.
    scroll: f32,
    /// Each pane's offset when it was last drawn. A pane that is already at the shared offset is
    /// left to scroll on its own, so its scroll animations run; the other is moved to match.
    drawn: [Option<f32>; 2],
    /// Each pane's viewport height when it was last drawn, for keeping the caret in view.
    viewport: [f32; 2],
    /// The pane and line whose caret is to be scrolled into view on the next draw.
    reveal: Option<(usize, usize)>,
    /// What the display filter leaves, `None` for every row, and the filter, context and counts
    /// setting it was worked out for.
    slots: Option<Vec<Slot>>,
    shown: Option<(Filter, usize, bool)>,
}

impl Aligned {
    /// Aligns the panes' lines. With one side missing, the other draws its lines plainly: there is
    /// nothing to compare it with, so nothing reads as different.
    // ponytail: compared on the UI thread when the panes load, like the load itself; a background
    // comparison with progress (`compare_progressive`) comes when large files need one.
    pub fn new(left: Option<&[Line]>, right: Option<&[Line]>, rules: (BuiltInRules, Options)) -> Self {
        let (rows, importance, result) = match (left, right) {
            (Some(left), Some(right)) => {
                let comparison = compare_with(left, right, rules.1);
                let classification = classify(&comparison, left, right, &[&rules.0]);
                (comparison.rows, classification.rows, Some((classification.outcome, classification.statistics)))
            }
            (one, other) => {
                let lines = one.or(other).map_or(0, <[Line]>::len);
                ((0..lines).map(|i| Row { left: Some(i), right: Some(i), kind: RowKind::Same }).collect(), Vec::new(), None)
            }
        };
        let longest = left.map_or(0, <[Line]>::len).max(right.map_or(0, <[Line]>::len));
        Self {
            rows,
            importance,
            result,
            chars: HashMap::new(),
            digits: longest.to_string().len(),
            lines: [left, right].map(|lines| lines.map_or(0, <[Line]>::len)),
            rules,
            scroll: 0.0,
            drawn: [None; 2],
            viewport: [0.0; 2],
            reveal: None,
            slots: None,
            shown: None,
        }
    }

    /// Compares the panes' lines again after an edit, keeping where both panes are scrolled.
    // ponytail: a full compare and classify per edit, like `otd_text_engine::Pair`, inside E03-S1's
    // budget for a recompare after a one-line edit; reuse the rows past the edit if that is missed.
    pub fn recompare(&mut self, left: Option<&[Line]>, right: Option<&[Line]>) {
        let Self { scroll, drawn, viewport, reveal, rules, .. } = *self;
        *self = Self { scroll, drawn, viewport, reveal, ..Self::new(left, right, rules) };
    }

    /// The Importance tab's rules and options it was compared with.
    pub(crate) fn importance_rules(&self) -> (BuiltInRules, Options) {
        self.rules
    }

    /// The row that shows line `line` of side `side`. The place after a final line break is the
    /// row after the last line's, which may be a gap or the blank row after every row.
    pub(crate) fn row_of(&self, side: usize, line: usize) -> Option<usize> {
        if line == *self.lines.get(side)? {
            return match line.checked_sub(1) {
                None => Some(0),
                Some(last) => Some(self.row_of(side, last)? + 1),
            };
        }
        self.rows.iter().position(|row| if side == 0 { row.left } else { row.right } == Some(line))
    }

    /// Shows the rows `filter` leaves with `context` lines, summarising hidden runs if `counts`.
    pub(crate) fn filter(&mut self, filter: Filter, context: usize, counts: bool) {
        if self.shown != Some((filter, context, counts)) {
            let differs = (0..self.rows.len()).map(|row| self.importance(row).is_some());
            self.slots = filtered::slots(differs, self.rows.len(), filter, context, counts);
            self.shown = Some((filter, context, counts));
        }
    }

    /// How many slots the panes draw, before the blank one after them.
    fn slot_count(&self) -> usize {
        self.slots.as_ref().map_or(self.rows.len(), Vec::len)
    }

    /// The slot at `at`; `None` for the blank one after the rest.
    fn slot(&self, at: usize) -> Option<Slot> {
        match &self.slots {
            None => (at < self.rows.len()).then_some(Slot::Row(at)),
            Some(slots) => slots.get(at).cloned(),
        }
    }

    /// The slot that draws row `row`, if the filter shows it; past the rows, the blank slot.
    fn slot_of(&self, row: usize) -> Option<usize> {
        match &self.slots {
            None => Some(row),
            Some(slots) if row >= self.rows.len() => Some(slots.len()),
            Some(slots) => slots.iter().position(|slot| *slot == Slot::Row(row)),
        }
    }

    /// How line `line` of side `side` differs, if it does, as an English key.
    pub(crate) fn difference(&self, side: usize, line: usize) -> Option<&'static str> {
        let row = self.row_of(side, line)?;
        match self.rows.get(row)?.kind {
            RowKind::Same => None,
            RowKind::Changed if self.importance(row) == Some(Importance::Unimportant) => Some(UNIMPORTANT),
            RowKind::Changed => Some(CHANGED),
            RowKind::LeftOnly => Some(DELETED),
            RowKind::RightOnly => Some(INSERTED),
        }
    }

    /// What navigation walks (`arrows.rs`): the rows, both panes' `lines`, and as different the
    /// rows the view classifies as different.
    pub(crate) fn differences<'a>(&'a self, lines: [&'a [Line]; 2]) -> Differences<'a, impl Fn(usize) -> bool + 'a> {
        Differences { rows: &self.rows, left: lines[0], right: lines[1], differs: move |row| self.importance(row).is_some() }
    }

    /// How many rows pane `side` showed when it was last drawn: a page.
    pub(crate) fn page(&self, side: usize, ui: &Ui) -> usize {
        let height = ui.text_style_height(&TextStyle::Monospace).max(1.0);
        (self.viewport.get(side).copied().unwrap_or_default() / height).floor().max(1.0) as usize
    }

    /// Scrolls pane `side` on its next draw so that line `line`'s caret is in view.
    pub(crate) fn reveal(&mut self, side: usize, line: usize) {
        self.reveal = Some((side, line));
    }

    /// The panes are about to change size (a new layout): their viewports are measured again, and
    /// then `caret`'s line (side, line) is scrolled into view.
    pub(crate) fn relayout(&mut self, caret: Option<(usize, usize)>) {
        self.viewport = [0.0; 2];
        self.reveal = caret.or(self.reveal);
    }

    /// Takes `old`'s shared scroll offset, so a comparison rerun over the same panes keeps their
    /// place. egui clamps it if the rows got fewer.
    pub(crate) fn keep_scroll(&mut self, old: &Aligned) {
        self.scroll = old.scroll;
    }

    /// The comparison's overall result and statistics, or `None` with a pane missing.
    pub fn result(&self) -> Option<(Outcome, Statistics)> {
        self.result
    }

    fn importance(&self, row: usize) -> Option<Importance> {
        self.importance.get(row).copied().flatten()
    }

    /// The changed row `row`'s character differences, computed once.
    fn chars(&mut self, row: usize, lines: [&[Line]; 2]) -> Option<&CharDifferences> {
        let found = *self.rows.get(row)?;
        if found.kind != RowKind::Changed {
            return None;
        }
        let left = lines[0].get(found.left?)?;
        let right = lines[1].get(found.right?)?;
        let rules = self.rules.0;
        Some(self.chars.entry(row).or_insert_with(|| char_differences_with(&left.text, &right.text, &[&rules])))
    }
}

/// The colours a line draws with, resolved from the roles.
#[derive(Clone, Copy)]
pub struct Colours {
    pub text: Color32,
    pub important_text: Color32,
    pub important_background: Color32,
    pub important_characters: Color32,
    pub unimportant_text: Color32,
    pub unimportant_background: Color32,
    pub unimportant_characters: Color32,
    pub line_number: Color32,
    pub selection: Color32,
    pub selection_text: Color32,
}

impl Colours {
    fn of(ui: &Ui) -> Self {
        let ctx = ui.ctx();
        Self {
            text: colour(ctx, &TEXT),
            important_text: colour(ctx, &IMPORTANT_TEXT),
            important_background: colour(ctx, &IMPORTANT_BACKGROUND),
            important_characters: colour(ctx, &IMPORTANT_CHARACTERS),
            unimportant_text: colour(ctx, &UNIMPORTANT_TEXT),
            unimportant_background: colour(ctx, &UNIMPORTANT_BACKGROUND),
            unimportant_characters: colour(ctx, &UNIMPORTANT_CHARACTERS),
            line_number: colour(ctx, &LINE_NUMBER),
            selection: colour(ctx, &SELECTION),
            selection_text: colour(ctx, &SELECTION_TEXT),
        }
    }
}

impl Colours {
    /// A line's text, by how its row differs: `None` for a same row.
    fn line_text(&self, importance: Option<Importance>) -> Color32 {
        match importance {
            None => self.text,
            Some(Importance::Important) => self.important_text,
            Some(Importance::Unimportant) => self.unimportant_text,
        }
    }

    /// A line's background, by how its row differs: none for a same row, and light red for a row
    /// with any important difference, even where it also has unimportant ones, because red wins.
    fn line_background(&self, importance: Option<Importance>) -> Option<Color32> {
        match importance {
            None => None,
            Some(Importance::Important) => Some(self.important_background),
            Some(Importance::Unimportant) => Some(self.unimportant_background),
        }
    }
}

/// One line's text laid out: in the colour of its row's `importance` (`None` for a same row), with
/// each `changed` range on its importance's characters background in its importance's text
/// colour, an important one underlined, and the `selected` byte range in the selection's colours
/// (still underlined where an important range is). An unimportant range's dashed underline is
/// painted over the laid-out text ([`dashed_underlines`]). Range ends outside the text or off a
/// character boundary are ignored.
pub fn line_job(
    text: &str,
    importance: Option<Importance>,
    changed: &[CharRange],
    selected: Range<usize>,
    colours: &Colours,
    font: FontId,
) -> LayoutJob {
    let plain = TextFormat { font_id: font, color: colours.line_text(importance), ..TextFormat::default() };
    let mut cuts: Vec<usize> = changed.iter().flat_map(|r| [r.start, r.end]).chain([0, text.len(), selected.start, selected.end]).collect();
    cuts.retain(|&at| text.is_char_boundary(at));
    cuts.sort_unstable();
    cuts.dedup();
    let mut job = LayoutJob::default();
    for pair in cuts.windows(2) {
        let (start, end) = (pair[0], pair[1]);
        let Some(part) = text.get(start..end) else { continue };
        let marked = changed.iter().find(|r| r.start <= start && end <= r.end).map(|r| r.importance);
        let format = if selected.start <= start && end <= selected.end {
            let underline = if marked == Some(Importance::Important) { Stroke::new(1.0, colours.selection_text) } else { Stroke::NONE };
            TextFormat { color: colours.selection_text, background: colours.selection, underline, ..plain.clone() }
        } else {
            match marked {
                None => plain.clone(),
                Some(Importance::Important) => {
                    let color = colours.important_text;
                    TextFormat { color, background: colours.important_characters, underline: Stroke::new(1.0, color), ..plain.clone() }
                }
                Some(Importance::Unimportant) => {
                    TextFormat { color: colours.unimportant_text, background: colours.unimportant_characters, ..plain.clone() }
                }
            }
        };
        job.append(part, 0.0, format);
    }
    job
}

/// The colour of the whitespace mark on the character at byte `at`, the same as [`line_job`] gives
/// that character: the selection's text colour inside the `selected` byte range, blue inside an
/// unimportant range of `changed`, and otherwise `line`, the line's text colour.
fn mark_colour(changed: &[CharRange], selected: Range<usize>, colours: &Colours, line: Color32, at: usize) -> Color32 {
    if selected.contains(&at) {
        colours.selection_text
    } else if changed.iter().any(|r| r.importance == Importance::Unimportant && (r.start..r.end).contains(&at)) {
        colours.unimportant_text
    } else {
        line
    }
}

/// A dashed line under each unimportant range of `changed` in `text`, laid out as `galley` at
/// `origin`, so an unimportant change is told from an important one's solid underline by more
/// than its colour.
fn dashed_underlines(text: &str, changed: &[CharRange], galley: &Galley, origin: Pos2, colour: Color32) -> Vec<Shape> {
    let mut unimportant = changed.iter().filter(|r| r.importance == Importance::Unimportant).peekable();
    if unimportant.peek().is_none() {
        return Vec::new();
    }
    // One pass over the text, so each range end finds its column by a binary search.
    let starts: Vec<usize> = text.char_indices().map(|(at, _)| at).collect();
    let x = |byte: usize| galley.pos_from_cursor(CCursor::new(starts.partition_point(|&at| at < byte)));
    let stroke = Stroke::new(1.0, colour);
    unimportant
        .flat_map(|r| {
            let (from, to) = (x(r.start), x(r.end));
            Shape::dashed_line(&[origin + from.left_bottom().to_vec2(), origin + pos2(to.left(), from.bottom()).to_vec2()], stroke, 2.0, 2.0)
        })
        .collect()
}

/// What a pane's editor draws and asks of its rows this frame.
pub struct Caret {
    pub selection: Selection,
    /// Whether the caret is drawn: the editor has focus.
    pub shown: bool,
    /// Where the pointer presses or drags in the pane, if it does.
    pub pointer: Option<Pos2>,
    /// The pane's bookmarks, marked in the gutter (E07-S6-T2).
    pub bookmarks: Bookmarks,
    /// Whether spaces, tabs and line breaks show as glyphs (*Show Whitespace*).
    pub whitespace: bool,
}

/// What a pane drew this frame, for its editor.
#[derive(Default)]
pub struct Drawn {
    /// The place in the text under `Caret::pointer`, if it is over a line.
    pub hit: Option<Pos>,
    /// Where the caret was drawn, if it was.
    pub caret: Option<Rect>,
    /// The lines drawn, in order: the line's index, its text laid out, where that was painted, and
    /// whether the line ends with a line break.
    pub lines: Vec<(usize, Arc<Galley>, Pos2, bool)>,
}

/// Byte offset of character `column` in `text`, or its end.
fn byte_at(text: &str, column: usize) -> usize {
    text.char_indices().nth(column).map_or(text.len(), |(at, _)| at)
}

/// How a pane's rows are measured and coloured, the same for every row of a frame.
struct Metrics {
    font: FontId,
    height: f32,
    /// A digit's width; the gutter is the bookmark mark, two digits wide, the line number's digits,
    /// a space, the marker and a space.
    digit: f32,
    digits: usize,
    colours: Colours,
    /// The pane drawn: 0 is left.
    side: usize,
    /// The row whose caret is scrolled into view sideways.
    reveal: Option<usize>,
}

impl Metrics {
    fn gutter(&self) -> f32 {
        self.digit * (self.digits + 5) as f32
    }
}

/// Side `side`'s pane body (0 is left): every row, in a scroll area whose vertical offset the other
/// pane shares. `lines` are both panes' lines; a missing pane's are empty.
pub fn show(ui: &mut Ui, side: usize, lines: [&[Line]; 2], aligned: &mut Aligned, caret: &Caret) -> Drawn {
    let font = TextStyle::Monospace.resolve(ui.style());
    let digit = ui.fonts_mut(|fonts| fonts.glyph_width(&font, '0'));
    let height = ui.text_style_height(&TextStyle::Monospace);
    let viewport = aligned.viewport.get(side).copied().unwrap_or_default();
    // Kept until the pane's viewport is measured, after a new layout, and then applied on the next
    // draw.
    if aligned.reveal.is_some_and(|(s, _)| s == side) && viewport <= 0.0 {
        ui.ctx().request_repaint();
    }
    let reveal = aligned.reveal.take_if(|(s, _)| *s == side && viewport > 0.0).and_then(|(_, line)| aligned.row_of(side, line));
    let metrics = Metrics { height, digit, digits: aligned.digits, colours: Colours::of(ui), font, side, reveal };
    ui.spacing_mut().item_spacing.y = 0.0;
    if let Some(slot) = reveal.and_then(|row| aligned.slot_of(row)) {
        // The caret's row into view: up to the top, or down to the bottom.
        let top = slot as f32 * metrics.height;
        let scroll = aligned.scroll.min(top).max(top + metrics.height - viewport);
        if scroll != aligned.scroll {
            aligned.scroll = scroll.max(0.0);
            aligned.drawn = [None; 2];
        }
    }
    let mut area = ScrollArea::both().id_salt(("text-compare-pane", side));
    if aligned.drawn.get(side).copied().flatten() != Some(aligned.scroll) {
        area = area.vertical_scroll_offset(aligned.scroll);
    }
    let mut drawn = Drawn::default();
    let output = area
        .auto_shrink(false)
        // A drag selects text instead.
        .scroll_source(ScrollSource { drag: ScrollSource::NONE.drag, ..ScrollSource::ALL })
        // Both panes always show both bars, so their viewports are the same height: with a sideways
        // bar in one pane only, the shared offset would stop at the shorter pane's end and hide
        // the other's last row under its bar.
        .scroll_bar_visibility(ScrollBarVisibility::AlwaysVisible)
        // One blank row after the rest, where the caret can stand after a final line break.
        .show_rows(ui, metrics.height, aligned.slot_count() + 1, |ui, slots| {
            for slot in slots {
                match aligned.slot(slot) {
                    Some(Slot::Row(row)) => draw_row(ui, row, lines, aligned, &metrics, caret, &mut drawn),
                    Some(Slot::Hidden(rows)) => hidden_row(ui, rows, aligned, &metrics),
                    None => draw_row(ui, aligned.rows.len(), lines, aligned, &metrics, caret, &mut drawn),
                }
            }
        });
    let offset = output.state.clamped_offset().y;
    if let Some(drawn) = aligned.drawn.get_mut(side) {
        *drawn = Some(offset);
    }
    if let Some(viewport) = aligned.viewport.get_mut(side) {
        *viewport = output.inner_rect.height();
    }
    if offset != aligned.scroll {
        aligned.scroll = offset;
        // The pane drawn before this one catches up on the next frame.
        ui.ctx().request_repaint();
    }
    drawn
}

/// On the row after the pane's last line, `rect`: the caret when it stands after a final line
/// break, and that place when the pointer is on the row.
fn after_last_line(ui: &Ui, row: usize, rect: Rect, aligned: &Aligned, metrics: &Metrics, caret: &Caret, drawn: &mut Drawn) {
    let Some(&line) = aligned.lines.get(metrics.side) else { return };
    if aligned.row_of(metrics.side, line) != Some(row) {
        return;
    }
    if caret.shown && caret.selection.head.line == line {
        let x = rect.left() + metrics.gutter();
        let caret_rect = Rect::from_min_max(pos2(x, rect.top()), pos2(x + 2.0, rect.bottom()));
        ui.painter().rect_filled(caret_rect, 0.0, metrics.colours.text);
        if metrics.reveal == Some(row) {
            ui.scroll_to_rect(caret_rect.expand2(vec2(metrics.gutter() + metrics.digit, 0.0)), None);
        }
        drawn.caret = Some(caret_rect);
    }
    // The end of the text: after the break, or the last line's end when there is none.
    if caret.pointer.is_some_and(|p| (rect.top()..rect.bottom()).contains(&p.y)) {
        drawn.hit = Some(Pos { line, column: usize::MAX });
    }
}

/// A run of hidden rows: how many of the pane's lines it hides, after the gutter, in the line
/// numbers' colour. A run of the other pane's lines only hides none of this pane's, so its row here
/// stays blank, like a gap.
fn hidden_row(ui: &mut Ui, rows: Range<usize>, aligned: &Aligned, metrics: &Metrics) {
    let side = metrics.side;
    let hidden =
        aligned.rows.get(rows.clone()).unwrap_or_default().iter().filter(|row| if side == 0 { row.left } else { row.right }.is_some()).count();
    let (_, rect) = ui.allocate_space(vec2(ui.available_width(), metrics.height));
    if hidden == 0 {
        return;
    }
    let text = trn(hidden as u64, LINE_HIDDEN, LINES_HIDDEN);
    let left = rect.left().max(ui.clip_rect().left());
    ui.painter().text(pos2(left + metrics.gutter(), rect.center().y), Align2::LEFT_CENTER, &text, metrics.font.clone(), metrics.colours.line_number);
    ui.interact(rect, ui.scope_id().with(("hidden", rows.start)), Sense::hover()).widget_info(|| WidgetInfo::labeled(Role::Label, true, &text));
}

/// Draws row `row` of the pane, with the caret and selection where they fall on its line, and
/// adds what it drew to `drawn`.
fn draw_row(ui: &mut Ui, row: usize, lines: [&[Line]; 2], aligned: &mut Aligned, metrics: &Metrics, caret: &Caret, drawn: &mut Drawn) {
    let side = metrics.side;
    let Some(&found) = aligned.rows.get(row) else {
        let (_, rect) = ui.allocate_space(vec2(ui.available_width(), metrics.height));
        return after_last_line(ui, row, rect, aligned, metrics, caret, drawn);
    };
    let index = if side == 0 { found.left } else { found.right };
    let line = index.and_then(|i| lines[side].get(i));
    let importance = aligned.importance(row);
    let (changed, exact) = match aligned.chars(row, lines) {
        Some(differences) => {
            let this = if side == 0 { Side::Left } else { Side::Right };
            let ranges: Vec<CharRange> = differences.ranges.iter().filter(|r| r.side == this).copied().collect();
            (ranges, differences.exact)
        }
        None => (Vec::new(), true),
    };
    let Metrics { font, height, digit, digits, colours, reveal, .. } = metrics;
    // The selected characters of this line, and whether its line break is selected too.
    let (start, end) = caret.selection.range();
    let (selected, break_selected) = match (index, line) {
        (Some(index), Some(line)) if (start.line..=end.line).contains(&index) && !caret.selection.is_empty() => {
            let from = if index == start.line { byte_at(&line.text, start.column) } else { 0 };
            let to = if index == end.line { byte_at(&line.text, end.column) } else { line.text.len() };
            (from..to, index < end.line)
        }
        _ => (0..0, false),
    };
    let galley =
        line.map(|line| ui.fonts_mut(|fonts| fonts.layout_job(line_job(&line.text, importance, &changed, selected.clone(), colours, font.clone()))));
    let gutter = metrics.gutter();
    // A digit past the text, room for a selected line break.
    let width = ui.available_width().max(gutter + galley.as_ref().map_or(0.0, |g| g.size().x) + digit);
    let (_, rect) = ui.allocate_space(vec2(width, *height));
    // A gap is padding: no number, no text, no node. Its opposite line says how it differs.
    let (Some(index), Some(galley), Some(line)) = (index, galley, line) else {
        return after_last_line(ui, row, rect, aligned, metrics, caret, drawn);
    };
    let painter = ui.painter();
    if let Some(background) = colours.line_background(importance) {
        // To the pane's right edge, also when it is scrolled sideways past a short line.
        let behind = rect.with_max_x(rect.right().max(ui.clip_rect().right()));
        painter.rect_filled(behind, 0.0, background);
    }
    let text_colour = colours.line_text(importance);
    // The gutter stays at the viewport's left edge when the pane is scrolled sideways, and the
    // text scrolls under it.
    let left = rect.left().max(ui.clip_rect().left());
    let number = (index + 1).to_string();
    let bookmark = Rect::from_min_size(pos2(left, rect.top()), vec2(digit * 2.0, *height));
    crate::bookmarks::mark(ui, &caret.bookmarks, index, bookmark, font, text_colour);
    let number_rect = Rect::from_min_size(pos2(bookmark.right(), rect.top()), vec2(digit * *digits as f32, *height));
    painter.text(number_rect.right_center(), Align2::RIGHT_CENTER, &number, font.clone(), colours.line_number);
    ui.interact(number_rect, ui.scope_id().with(("number", row)), Sense::hover()).widget_info(|| WidgetInfo::labeled(Role::Label, true, &number));
    let (marker, description) = match found.kind {
        RowKind::Same => (None, None),
        // An unimportant line's marker differs from an important one's, so colour isn't the only
        // signal.
        RowKind::Changed if importance == Some(Importance::Unimportant) => {
            (Some("\u{2248}"), Some(if exact { UNIMPORTANT } else { UNIMPORTANT_APPROXIMATE }))
        }
        RowKind::Changed => (Some("~"), Some(if exact { CHANGED } else { APPROXIMATE })),
        RowKind::LeftOnly => (Some("-"), Some(DELETED)),
        RowKind::RightOnly => (Some("+"), Some(INSERTED)),
    };
    if let Some(marker) = marker {
        painter.text(pos2(number_rect.right() + digit * 1.5, rect.center().y), Align2::CENTER_CENTER, marker, font.clone(), text_colour);
    }
    // To the row's end, so a blank line's node has a size too.
    let text_rect = Rect::from_min_max(pos2(rect.left() + gutter, rect.top()), rect.max);
    let name = if galley.text().trim().is_empty() { t(BLANK).to_owned() } else { galley.text().to_owned() };
    let text_painter = painter.with_clip_rect(painter.clip_rect().intersect(Rect::everything_right_of(left + gutter)));
    let origin = pos2(text_rect.left(), rect.center().y - galley.size().y / 2.0);
    if let Some(p) = caret.pointer.filter(|p| (rect.top()..rect.bottom()).contains(&p.y)) {
        drawn.hit = Some(Pos { line: index, column: galley.cursor_from_pos(p - origin).index.into() });
    }
    drawn.lines.push((index, galley.clone(), origin, line.ending != LineEnding::None));
    if break_selected {
        // A selected line break shows as a space's width of selection after the text.
        let at = origin.x + galley.size().x;
        text_painter.rect_filled(Rect::from_min_max(pos2(at, rect.top()), pos2(at + digit, rect.bottom())), 0.0, colours.selection);
    }
    let caret_rect = (caret.shown && caret.selection.head.line == index).then(|| {
        let x = origin.x + galley.pos_from_cursor(CCursor::new(caret.selection.head.column)).min.x;
        Rect::from_min_max(pos2(x, rect.top()), pos2(x + 2.0, rect.bottom()))
    });
    text_painter.galley(origin, galley.clone(), text_colour);
    text_painter.extend(dashed_underlines(&line.text, &changed, &galley, origin, colours.unimportant_text));
    if caret.whitespace {
        // Over the text, so the selection's background doesn't hide them.
        let colour = |at: usize| mark_colour(&changed, selected.clone(), colours, text_colour, at);
        let break_colour = if break_selected { colours.selection_text } else { text_colour };
        whitespace::paint(&text_painter, (&galley, origin, line.ending), colour, break_colour, font);
    }
    if let Some(caret_rect) = caret_rect {
        text_painter.rect_filled(caret_rect, 0.0, text_colour);
        if *reveal == Some(row) {
            // Sideways into view, past the gutter that covers the text's left edge.
            ui.scroll_to_rect(caret_rect.expand2(vec2(gutter + digit, 0.0)), None);
        }
        drawn.caret = Some(caret_rect);
    }
    let response = ui.interact(text_rect, ui.scope_id().with(("line", row)), Sense::hover());
    response.widget_info(|| WidgetInfo::labeled(Role::Label, true, &name));
    if let Some(description) = description {
        ui.ctx().accesskit_node_builder(response.id, |node| node.set_description(t(description)));
    }
    if !exact {
        // Said in text too, since the marked characters then include unchanged ones.
        response.on_hover_text(t(APPROXIMATE));
    }
}

#[cfg(test)]
mod tests {
    use egui::{Color32, FontId};
    use otd_text_compare::{Line, LineEnding};
    use otd_text_engine::{BuiltInRules, Importance, Options, Side, char_differences};

    use super::{Aligned, Colours, UNIMPORTANT, line_job, mark_colour};

    /// Every colour told apart from the others.
    const COLOURS: Colours = Colours {
        text: Color32::BLACK,
        important_text: Color32::RED,
        important_background: Color32::LIGHT_RED,
        important_characters: Color32::YELLOW,
        unimportant_text: Color32::DARK_BLUE,
        unimportant_background: Color32::LIGHT_BLUE,
        unimportant_characters: Color32::LIGHT_GREEN,
        line_number: Color32::GRAY,
        selection: Color32::BLUE,
        selection_text: Color32::WHITE,
    };

    fn lines(text: &str) -> Vec<Line> {
        text.lines().map(|text| Line { text: text.to_owned(), ending: LineEnding::Lf }).collect()
    }

    /// The first row's line on side `side`, laid out, as (text colour, background) and the text
    /// drawn in them.
    fn drawn(aligned: &mut Aligned, sides: [&[Line]; 2], side: Side) -> Vec<((Color32, Color32), String)> {
        let importance = aligned.importance(0);
        let ranges: Vec<_> = aligned.chars(0, sides).map(|d| d.ranges.iter().filter(|r| r.side == side).copied().collect()).unwrap_or_default();
        let text = &sides[if side == Side::Left { 0 } else { 1 }][0].text;
        let job = line_job(text, importance, &ranges, 0..0, &COLOURS, FontId::monospace(12.0));
        assert_eq!(&job.text, text, "the whole line is laid out, in order");
        job.sections
            .iter()
            .filter_map(|s| Some(((s.format.color, s.format.background), job.text.get(s.byte_range.start.0..s.byte_range.end.0)?.to_owned())))
            .collect()
    }

    #[test]
    fn a_changed_line_marks_only_its_changed_characters_and_keeps_all_its_text() {
        let (left, right) = ("let tötal = count + 1;", "let tötal = amount + 2;");
        let differences = char_differences(left, right);
        let [(left_marked, left_rest), (right_marked, right_rest)] = [(Side::Left, left), (Side::Right, right)].map(|(side, text)| {
            let ranges: Vec<_> = differences.ranges.iter().filter(|r| r.side == side).copied().collect();
            let job = line_job(text, Some(Importance::Important), &ranges, 0..0, &COLOURS, FontId::monospace(12.0));
            assert_eq!(job.text, text, "the whole line is laid out, in order");
            let part = |marked: bool| -> String {
                job.sections
                    .iter()
                    .filter(|s| (s.format.background == COLOURS.important_characters) == marked)
                    .filter_map(|s| job.text.get(s.byte_range.start.0..s.byte_range.end.0))
                    .collect()
            };
            (part(true), part(false))
        });
        assert!(!left_marked.is_empty() && !right_marked.is_empty(), "both sides mark what changed");
        assert!(left_marked.len() < left.len() / 2, "the unchanged characters aren't marked: {left_marked:?}");
        assert_eq!(left_rest, right_rest, "the unmarked characters are the ones both lines share");
    }

    #[test]
    fn a_line_with_a_case_change_and_a_word_change_shows_the_case_in_blue_and_the_word_in_red_on_red() {
        let (left, right) = (lines("Hello big world\n"), lines("hello big earth\n"));
        let sides = [left.as_slice(), right.as_slice()];
        let mut aligned =
            Aligned::new(Some(&left), Some(&right), (BuiltInRules { character_case: false, ..BuiltInRules::default() }, Options::default()));
        assert_eq!(aligned.importance(0), Some(Importance::Important), "the word change makes the line important");
        assert_eq!(COLOURS.line_background(aligned.importance(0)), Some(COLOURS.important_background), "red wins on the line");

        for side in [Side::Left, Side::Right] {
            let drawn = drawn(&mut aligned, sides, side);
            let blue: String =
                drawn.iter().filter(|(c, _)| *c == (COLOURS.unimportant_text, COLOURS.unimportant_characters)).map(|(_, t)| t.as_str()).collect();
            let red: String =
                drawn.iter().filter(|(c, _)| *c == (COLOURS.important_text, COLOURS.important_characters)).map(|(_, t)| t.as_str()).collect();
            assert!(blue.eq_ignore_ascii_case("h"), "the case change is blue: {blue:?}");
            assert!(!red.is_empty() && red.chars().all(|c| "worldearth".contains(c)), "only the changed word is red: {red:?}");
            let plain: String = drawn.iter().filter(|(c, _)| *c == (COLOURS.important_text, Color32::TRANSPARENT)).map(|(_, t)| t.as_str()).collect();
            assert!(plain.contains("ello big "), "the unchanged text keeps the line's red text, unmarked: {plain:?}");
        }
    }

    #[test]
    fn a_line_with_only_unimportant_changes_is_blue_and_says_so_in_text() {
        let (left, right) = (lines("same\nHello\n"), lines("same\nhello\n"));
        let sides = [left.as_slice(), right.as_slice()];
        let mut aligned =
            Aligned::new(Some(&left), Some(&right), (BuiltInRules { character_case: false, ..BuiltInRules::default() }, Options::default()));
        let row = 1;
        assert_eq!(aligned.importance(row), Some(Importance::Unimportant));
        assert_eq!(COLOURS.line_background(aligned.importance(row)), Some(COLOURS.unimportant_background), "a light-blue line");
        assert_eq!(aligned.difference(0, row), Some(UNIMPORTANT), "described as unimportant, not only coloured");
        assert_ne!(aligned.difference(0, row), Aligned::new(Some(&left), Some(&right), Default::default()).difference(0, row));
        let ranges: Vec<_> =
            aligned.chars(row, sides).map(|d| d.ranges.iter().filter(|r| r.side == Side::Left).copied().collect()).unwrap_or_default();
        assert!(!ranges.is_empty() && ranges.iter().all(|r| r.importance == Importance::Unimportant), "its changed characters are all blue");
    }

    #[test]
    fn whitespace_marks_take_the_colour_of_the_range_they_are_in() {
        let (left, right) = (lines("one big world\n"), lines("one big earth  \n"));
        let sides = [left.as_slice(), right.as_slice()];
        let rules = BuiltInRules { trailing_whitespace: false, ..BuiltInRules::default() };
        let mut aligned = Aligned::new(Some(&left), Some(&right), (rules, Options::default()));
        assert_eq!(aligned.importance(0), Some(Importance::Important), "the word change makes the line red");
        let text = &right[0].text;
        let ranges: Vec<_> =
            aligned.chars(0, sides).map(|d| d.ranges.iter().filter(|r| r.side == Side::Right).copied().collect()).unwrap_or_default();
        let line = COLOURS.line_text(aligned.importance(0));
        let colour = |at| mark_colour(&ranges, 0..0, &COLOURS, line, at);
        let trailing = text.len() - 1;
        assert_eq!(colour(trailing), COLOURS.unimportant_text, "an unimportant trailing space's mark is blue on a red line");
        assert_eq!(colour(3), line, "an unchanged space's mark keeps the line's colour");
        assert_eq!(
            mark_colour(&ranges, 0..text.len(), &COLOURS, line, trailing),
            COLOURS.selection_text,
            "a selected mark is in the selection's colour"
        );
    }
}
