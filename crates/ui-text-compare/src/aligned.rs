//! The aligned comparison (E07-S1-T4): both panes draw the text engine's rows, a line or a gap on
//! each side, at one fixed row height, so matching lines sit on the same row. Gap rows have no
//! line number. Important differences draw in red text on a light-red line background, with the
//! changed characters inside a changed line on a stronger background and underlined, so colour
//! isn't the only signal. A gutter marker and each line's accessible description say how the
//! line differs. The two panes share one vertical scroll offset.
//!
//! Each pane also draws its caret and selection, and maps a pointer press to a place in its text,
//! for the editor (E07-S1-T5, `edit.rs`).
//!
//! With Word Wrap on (E07-S9-T2, `wrap.rs`), each line wraps to its pane's width. A row is then as
//! tall as the taller of its two lines, so the continuation rows of a line sit opposite those of its
//! counterpart, or opposite padding, and the next line starts on the same row in both panes. The
//! line number and marker stay on a line's first row.

use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;

use egui::scroll_area::{ScrollBarVisibility, ScrollSource};
use egui::text::CCursor;
use egui::text::{LayoutJob, TextFormat};
use egui::{Align2, Color32, FontId, Galley, Pos2, Rect, Role, ScrollArea, Sense, Stroke, TextStyle, Ui, UiBuilder, WidgetInfo, pos2, vec2};
use otd_core::colour::{ColourRole, Contrast, Rgba};
use otd_core::i18n::t;
use otd_text_compare::{Bookmarks, Line, LineEnding, Pos, Selection};
use otd_text_engine::{CharDifferences, CharRange, Importance, Outcome, Row, RowKind, Side, Statistics, char_differences, classify, compare};
use otd_ui::colour::{BACKGROUND, SELECTION, SELECTION_TEXT, TEXT, colour};

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
/// Line numbers in the gutter.
pub const LINE_NUMBER: ColourRole =
    ColourRole { id: "text-compare.line-number", light: Rgba::rgb(0x61, 0x61, 0x61), dark: Rgba::rgb(0x9A, 0x9A, 0x9A) };

/// Text Compare's colour roles, which the app registers.
pub const ROLES: [ColourRole; 4] = [IMPORTANT_TEXT, IMPORTANT_BACKGROUND, IMPORTANT_CHARACTERS, LINE_NUMBER];

/// What each role is drawn on.
pub const CONTRAST: [Contrast; 6] = [
    Contrast::text(&IMPORTANT_TEXT, &IMPORTANT_BACKGROUND),
    Contrast::text(&IMPORTANT_TEXT, &IMPORTANT_CHARACTERS),
    Contrast::text(&LINE_NUMBER, &BACKGROUND),
    Contrast::text(&LINE_NUMBER, &IMPORTANT_BACKGROUND),
    // An important line's gutter marker.
    Contrast::text(&IMPORTANT_TEXT, &BACKGROUND),
    // The focused pane's focus ring, where it crosses an important line.
    Contrast::control(&SELECTION_TEXT, &IMPORTANT_BACKGROUND),
];

// A line's accessible description: how it differs.
pub(crate) const CHANGED: &str = "Changed";
const APPROXIMATE: &str = "Changed, character differences approximate";
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
    /// The vertical scroll offset both panes share.
    scroll: f32,
    /// Each pane's offset when it was last drawn. A pane that is already at the shared offset is
    /// left to scroll on its own, so its scroll animations run; the other is moved to match.
    drawn: [Option<f32>; 2],
    /// Each pane's viewport height when it was last drawn, for keeping the caret in view.
    viewport: [f32; 2],
    /// The pane and line whose caret is to be scrolled into view on the next draw.
    reveal: Option<(usize, usize)>,
    /// Each pane's viewport width when it was last drawn, which Word Wrap wraps its lines to.
    widths: [Option<f32>; 2],
    /// Each row's top, then the blank row's after them, then the end: `rows.len() + 2` offsets
    /// from the first row's top, as laid out for `laid_out`.
    tops: Vec<f32>,
    /// What `tops` was laid out for: each side's wrap width (`None` when not wrapping) and the
    /// line height.
    laid_out: Option<([Option<f32>; 2], f32)>,
}

impl Aligned {
    /// Aligns the panes' lines. With one side missing, the other draws its lines plainly: there is
    /// nothing to compare it with, so nothing reads as different.
    // ponytail: compared on the UI thread when the panes load, like the load itself; a background
    // comparison with progress (`compare_progressive`) comes when large files need one.
    pub fn new(left: Option<&[Line]>, right: Option<&[Line]>) -> Self {
        let (rows, importance, result) = match (left, right) {
            (Some(left), Some(right)) => {
                let comparison = compare(left, right);
                // ponytail: no rules yet, so every difference is important; E07-S4 passes them.
                let classification = classify(&comparison, left, right, &[]);
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
            scroll: 0.0,
            drawn: [None; 2],
            viewport: [0.0; 2],
            reveal: None,
            widths: [None; 2],
            tops: Vec::new(),
            laid_out: None,
        }
    }

    /// Compares the panes' lines again after an edit, keeping where both panes are scrolled.
    // ponytail: a full compare and classify per edit, like `otd_text_engine::Pair`, inside E03-S1's
    // budget for a recompare after a one-line edit; reuse the rows past the edit if that is missed.
    pub fn recompare(&mut self, left: Option<&[Line]>, right: Option<&[Line]>) {
        let Self { scroll, drawn, viewport, reveal, widths, .. } = *self;
        *self = Self { scroll, drawn, viewport, reveal, widths, ..Self::new(left, right) };
    }

    /// The row that shows line `line` of side `side`. The place after a final line break is the
    /// row after the last line's, which may be a gap or the blank row after every row.
    fn row_of(&self, side: usize, line: usize) -> Option<usize> {
        if line == *self.lines.get(side)? {
            return match line.checked_sub(1) {
                None => Some(0),
                Some(last) => Some(self.row_of(side, last)? + 1),
            };
        }
        self.rows.iter().position(|row| if side == 0 { row.left } else { row.right } == Some(line))
    }

    /// Row `row`'s top, from the first row's.
    fn top(&self, row: usize) -> f32 {
        self.tops.get(row).or(self.tops.last()).copied().unwrap_or_default()
    }

    /// The row at height `y` from the first row's top.
    fn row_at(&self, y: f32) -> usize {
        self.tops.partition_point(|&top| top <= y).saturating_sub(1)
    }

    /// Each side's wrap width: its viewport's, less the gutter and room for the caret, or the other
    /// side's before it has been drawn. `None` when not wrapping.
    fn wrap_widths(&self, wrap: bool, metrics: &Metrics) -> [Option<f32>; 2] {
        let text = |width: f32| (width - metrics.gutter() - metrics.digit).max(metrics.digit);
        [0, 1].map(|side| if wrap { self.widths[side].or(self.widths[1 - side]).map(text) } else { None })
    }

    /// Lays the rows out for `wrap` and the line height, unless they already are: unwrapped, every
    /// row is one line high; wrapped, each is as tall as the taller of its two lines. When the
    /// layout changes, the row at the top of the panes stays there.
    // ponytail: wrapped, every line is laid out again after each edit, toggle and resize, which a
    // large file feels while typing or resizing; lay out only the rows past an edit if it shows.
    fn lay_out(&mut self, ui: &Ui, lines: [&[Line]; 2], wrap: [Option<f32>; 2], metrics: &Metrics) {
        let key = (wrap, metrics.height);
        if self.laid_out == Some(key) {
            return;
        }
        let top_row = self.laid_out.is_some().then(|| self.row_at(self.scroll));
        let height = metrics.height;
        let mut tops = Vec::with_capacity(self.rows.len() + 2);
        let mut y = 0.0;
        ui.fonts_mut(|fonts| {
            let mut measure = |side: usize, index: Option<usize>| {
                let (Some(width), Some(line)) = (wrap[side], index.and_then(|i| lines[side].get(i))) else { return height };
                let mut job = LayoutJob::single_section(line.text.clone(), TextFormat::simple(metrics.font.clone(), Color32::PLACEHOLDER));
                job.wrap.max_width = width;
                fonts.layout_job(job).size().y
            };
            for row in &self.rows {
                tops.push(y);
                y += measure(0, row.left).max(measure(1, row.right)).max(height);
            }
        });
        tops.extend([y, y + height]);
        self.tops = tops;
        self.laid_out = Some(key);
        if let Some(row) = top_row {
            self.scroll = self.top(row);
            self.drawn = [None; 2];
        }
    }

    /// How line `line` of side `side` differs, if it does, as an English key.
    pub(crate) fn difference(&self, side: usize, line: usize) -> Option<&'static str> {
        match self.rows.get(self.row_of(side, line)?)?.kind {
            RowKind::Same => None,
            RowKind::Changed => Some(CHANGED),
            RowKind::LeftOnly => Some(DELETED),
            RowKind::RightOnly => Some(INSERTED),
        }
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
        Some(self.chars.entry(row).or_insert_with(|| char_differences(&left.text, &right.text)))
    }
}

/// The colours a line draws with, resolved from the roles.
#[derive(Clone, Copy)]
pub struct Colours {
    pub text: Color32,
    pub important_text: Color32,
    pub important_background: Color32,
    pub important_characters: Color32,
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
            line_number: colour(ctx, &LINE_NUMBER),
            selection: colour(ctx, &SELECTION),
            selection_text: colour(ctx, &SELECTION_TEXT),
        }
    }
}

/// One line's text laid out: in `colours.text`, or for an important difference in
/// `colours.important_text`, with the `changed` byte ranges on the characters background and
/// underlined, and the `selected` byte range in the selection's colours (still underlined where
/// it changed). Range ends outside the text or off a character boundary are ignored.
pub fn line_job(text: &str, important: bool, changed: &[Range<usize>], selected: Range<usize>, colours: &Colours, font: FontId) -> LayoutJob {
    let colour = if important { colours.important_text } else { colours.text };
    let plain = TextFormat { font_id: font, color: colour, ..TextFormat::default() };
    let underline = Stroke::new(1.0, colour);
    let mut cuts: Vec<usize> = changed.iter().flat_map(|r| [r.start, r.end]).chain([0, text.len(), selected.start, selected.end]).collect();
    cuts.retain(|&at| text.is_char_boundary(at));
    cuts.sort_unstable();
    cuts.dedup();
    let mut job = LayoutJob::default();
    for pair in cuts.windows(2) {
        let (start, end) = (pair[0], pair[1]);
        let Some(part) = text.get(start..end) else { continue };
        let inside = |r: &Range<usize>| r.start <= start && end <= r.end;
        let marked = changed.iter().any(inside);
        let format = if inside(&selected) {
            let underline = if marked { Stroke::new(1.0, colours.selection_text) } else { Stroke::NONE };
            TextFormat { color: colours.selection_text, background: colours.selection, underline, ..plain.clone() }
        } else if marked {
            TextFormat { background: colours.important_characters, underline, ..plain.clone() }
        } else {
            plain.clone()
        };
        job.append(part, 0.0, format);
    }
    job
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
    /// The width this pane's lines wrap to, if they wrap.
    wrap: Option<f32>,
}

impl Metrics {
    fn gutter(&self) -> f32 {
        self.digit * (self.digits + 5) as f32
    }
}

/// Side `side`'s pane body (0 is left): every row, in a scroll area whose vertical offset the other
/// pane shares, its lines wrapped to the pane's width when `wrap` is set. `lines` are both panes'
/// lines; a missing pane's are empty.
pub fn show(ui: &mut Ui, side: usize, lines: [&[Line]; 2], aligned: &mut Aligned, caret: &Caret, wrap: bool) -> Drawn {
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
    let mut metrics = Metrics { height, digit, digits: aligned.digits, colours: Colours::of(ui), font, side, reveal, wrap: None };
    ui.spacing_mut().item_spacing.y = 0.0;
    aligned.lay_out(ui, lines, aligned.wrap_widths(wrap, &metrics), &metrics);
    if let Some(row) = reveal {
        // The caret's row into view: up to the top, or down to the bottom, keeping the top of a row
        // taller than the pane in view. Drawing the caret then brings it in.
        let (top, bottom) = (aligned.top(row), aligned.top(row + 1));
        let scroll = aligned.scroll.max(bottom - viewport).min(top);
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
    let laid_out = aligned.laid_out;
    let output = area
        .auto_shrink(false)
        // A drag selects text instead.
        .scroll_source(ScrollSource { drag: ScrollSource::NONE.drag, ..ScrollSource::ALL })
        // Both panes always show both bars, so their viewports are the same height: with a sideways
        // bar in one pane only, the shared offset would stop at the shorter pane's end and hide
        // the other's last row under its bar.
        .scroll_bar_visibility(ScrollBarVisibility::AlwaysVisible)
        .show_viewport(ui, |ui, viewport| {
            if let Some(width) = aligned.widths.get_mut(side) {
                *width = Some(ui.available_width());
            }
            let wrap = aligned.wrap_widths(wrap, &metrics);
            aligned.lay_out(ui, lines, wrap, &metrics);
            metrics.wrap = wrap[side];
            // One blank row after the rest, where the caret can stand after a final line break.
            let rows = aligned.rows.len() + 1;
            ui.set_height(aligned.top(rows));
            let first = aligned.row_at(viewport.min.y).min(rows);
            let last = aligned.tops.partition_point(|&top| top < viewport.max.y).clamp(first, rows);
            let top = ui.max_rect().top();
            let shown = Rect::from_x_y_ranges(ui.max_rect().x_range(), top + aligned.top(first)..=top + aligned.top(last));
            ui.scope_builder(UiBuilder::new().max_rect(shown), |ui| {
                ui.skip_ahead_auto_ids(first);
                for row in first..last {
                    draw_row(ui, row, lines, aligned, &metrics, caret, &mut drawn);
                }
            });
        });
    let offset = output.state.clamped_offset().y;
    if let Some(viewport) = aligned.viewport.get_mut(side) {
        *viewport = output.inner_rect.height();
    }
    if aligned.laid_out != laid_out {
        // Laid out again while drawing, as the pane's width changed: the shared offset already
        // keeps the top row, and both panes take it on the next frame.
        ui.ctx().request_repaint();
        return drawn;
    }
    if let Some(drawn) = aligned.drawn.get_mut(side) {
        *drawn = Some(offset);
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

/// Draws row `row` of the pane, with the caret and selection where they fall on its line, and
/// adds what it drew to `drawn`.
fn draw_row(ui: &mut Ui, row: usize, lines: [&[Line]; 2], aligned: &mut Aligned, metrics: &Metrics, caret: &Caret, drawn: &mut Drawn) {
    let side = metrics.side;
    let row_height = aligned.top(row + 1) - aligned.top(row);
    let Some(&found) = aligned.rows.get(row) else {
        let (_, rect) = ui.allocate_space(vec2(ui.available_width(), metrics.height));
        return after_last_line(ui, row, rect, aligned, metrics, caret, drawn);
    };
    let index = if side == 0 { found.left } else { found.right };
    let line = index.and_then(|i| lines[side].get(i));
    let important = aligned.importance(row) == Some(Importance::Important);
    let (changed, exact) = match aligned.chars(row, lines) {
        Some(differences) => {
            let this = if side == 0 { Side::Left } else { Side::Right };
            let ranges: Vec<Range<usize>> =
                differences.ranges.iter().filter(|r| r.side == this).map(|&CharRange { start, end, .. }| start..end).collect();
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
    let galley = line.map(|line| {
        let mut job = line_job(&line.text, important, &changed, selected.clone(), colours, font.clone());
        job.wrap.max_width = metrics.wrap.unwrap_or(f32::INFINITY);
        ui.fonts_mut(|fonts| fonts.layout_job(job))
    });
    let gutter = metrics.gutter();
    // A digit past the text, room for a selected line break.
    let width = ui.available_width().max(gutter + galley.as_ref().map_or(0.0, |g| g.size().x) + digit);
    let (_, rect) = ui.allocate_space(vec2(width, row_height));
    // A gap is padding: no number, no text, no node. Its opposite line says how it differs.
    let (Some(index), Some(galley), Some(line)) = (index, galley, line) else {
        return after_last_line(ui, row, rect, aligned, metrics, caret, drawn);
    };
    let painter = ui.painter();
    if important {
        // To the pane's right edge, also when it is scrolled sideways past a short line.
        let background = rect.with_max_x(rect.right().max(ui.clip_rect().right()));
        painter.rect_filled(background, 0.0, colours.important_background);
    }
    // The gutter stays at the viewport's left edge when the pane is scrolled sideways, and the
    // text scrolls under it.
    let left = rect.left().max(ui.clip_rect().left());
    let number = (index + 1).to_string();
    let bookmark = Rect::from_min_size(pos2(left, rect.top()), vec2(digit * 2.0, *height));
    crate::bookmarks::mark(ui, &caret.bookmarks, index, bookmark, font, if important { colours.important_text } else { colours.text });
    let number_rect = Rect::from_min_size(pos2(bookmark.right(), rect.top()), vec2(digit * *digits as f32, *height));
    painter.text(number_rect.right_center(), Align2::RIGHT_CENTER, &number, font.clone(), colours.line_number);
    ui.interact(number_rect, ui.scope_id().with(("number", row)), Sense::hover()).widget_info(|| WidgetInfo::labeled(Role::Label, true, &number));
    let (marker, description) = match found.kind {
        RowKind::Same => (None, None),
        RowKind::Changed => (Some("~"), Some(if exact { CHANGED } else { APPROXIMATE })),
        RowKind::LeftOnly => (Some("-"), Some(DELETED)),
        RowKind::RightOnly => (Some("+"), Some(INSERTED)),
    };
    let text_colour = if important { colours.important_text } else { colours.text };
    if let Some(marker) = marker {
        painter.text(pos2(number_rect.right() + digit * 1.5, number_rect.center().y), Align2::CENTER_CENTER, marker, font.clone(), text_colour);
    }
    // To the row's end, so a blank line's node has a size too.
    let text_rect = Rect::from_min_max(pos2(rect.left() + gutter, rect.top()), rect.max);
    let name = if galley.text().trim().is_empty() { t(BLANK).to_owned() } else { galley.text().to_owned() };
    let text_painter = painter.with_clip_rect(painter.clip_rect().intersect(Rect::everything_right_of(left + gutter)));
    // At the row's top: a wrapped line opposite a longer one leaves padding below it.
    let origin = pos2(text_rect.left(), rect.top());
    if let Some(p) = caret.pointer.filter(|p| (rect.top()..rect.bottom()).contains(&p.y)) {
        drawn.hit = Some(Pos { line: index, column: galley.cursor_from_pos(p - origin).index.into() });
    }
    drawn.lines.push((index, galley.clone(), origin, line.ending != LineEnding::None));
    if let Some(last) = galley.rows.last().filter(|_| break_selected) {
        // A selected line break shows as a space's width of selection after the text's last row.
        let last = last.rect().translate(origin.to_vec2());
        text_painter.rect_filled(Rect::from_min_max(last.right_top(), pos2(last.right() + digit, last.bottom())), 0.0, colours.selection);
    }
    let caret_rect = (caret.shown && caret.selection.head.line == index).then(|| {
        let at = galley.pos_from_cursor(CCursor::new(caret.selection.head.column)).translate(origin.to_vec2());
        Rect::from_min_max(at.min, pos2(at.min.x + 2.0, at.max.y))
    });
    text_painter.galley(origin, galley.clone(), text_colour);
    if caret.whitespace {
        // Over the text, so the selection's background doesn't hide them.
        let chars = |bytes: usize| line.text.get(..bytes).map_or(0, |before| before.chars().count());
        let colours = [text_colour, colours.selection_text];
        whitespace::paint(&text_painter, (&galley, origin, line.ending), (chars(selected.start)..chars(selected.end), break_selected), colours, font);
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
    use otd_text_engine::{Side, char_differences};

    use super::{Colours, line_job};

    #[test]
    fn a_changed_line_marks_only_its_changed_characters_and_keeps_all_its_text() {
        let colours = Colours {
            text: Color32::BLACK,
            important_text: Color32::RED,
            important_background: Color32::LIGHT_RED,
            important_characters: Color32::YELLOW,
            line_number: Color32::GRAY,
            selection: Color32::BLUE,
            selection_text: Color32::WHITE,
        };
        let (left, right) = ("let tötal = count + 1;", "let tötal = amount + 2;");
        let differences = char_differences(left, right);
        let [(left_marked, left_rest), (right_marked, right_rest)] = [(Side::Left, left), (Side::Right, right)].map(|(side, text)| {
            let ranges: Vec<_> = differences.ranges.iter().filter(|r| r.side == side).map(|r| r.start..r.end).collect();
            let job = line_job(text, true, &ranges, 0..0, &colours, FontId::monospace(12.0));
            assert_eq!(job.text, text, "the whole line is laid out, in order");
            let part = |marked: bool| -> String {
                job.sections
                    .iter()
                    .filter(|s| (s.format.background == colours.important_characters) == marked)
                    .filter_map(|s| job.text.get(s.byte_range.start.0..s.byte_range.end.0))
                    .collect()
            };
            (part(true), part(false))
        });
        assert!(!left_marked.is_empty() && !right_marked.is_empty(), "both sides mark what changed");
        assert!(left_marked.len() < left.len() / 2, "the unchanged characters aren't marked: {left_marked:?}");
        assert_eq!(left_rest, right_rest, "the unmarked characters are the ones both lines share");
    }
}
