//! The aligned comparison (E07-S1-T4): both panes draw the text engine's rows, a line or a gap on
//! each side, at one fixed row height, so matching lines sit on the same row. Gap rows have no
//! line number. Important differences draw in red text on a light-red line background, with the
//! changed characters inside a changed line on a stronger background and underlined, so colour
//! isn't the only signal. A gutter marker and each line's accessible description say how the
//! line differs. The two panes share one vertical scroll offset.
//!
//! Each pane also draws its caret and selection, and maps a pointer press to a place in its text,
//! for the editor (E07-S1-T5, `edit.rs`).

use std::collections::HashMap;
use std::ops::Range;

use egui::scroll_area::{ScrollBarVisibility, ScrollSource};
use egui::text::CCursor;
use egui::text::{LayoutJob, TextFormat};
use egui::{Align2, Color32, FontId, Pos2, Rect, Role, ScrollArea, Sense, Stroke, TextStyle, Ui, WidgetInfo, pos2, vec2};
use otd_core::colour::{ColourRole, Contrast, Rgba};
use otd_core::i18n::t;
use otd_text_compare::{Line, Pos, Selection};
use otd_text_engine::{CharDifferences, CharRange, Importance, Outcome, Row, RowKind, Side, Statistics, char_differences, classify, compare};
use otd_ui::colour::{BACKGROUND, SELECTION, SELECTION_TEXT, TEXT, colour};

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
        }
    }

    /// Compares the panes' lines again after an edit, keeping where both panes are scrolled.
    // ponytail: a full compare and classify per edit, like `otd_text_engine::Pair`, inside E03-S1's
    // budget for a recompare after a one-line edit; reuse the rows past the edit if that is missed.
    pub fn recompare(&mut self, left: Option<&[Line]>, right: Option<&[Line]>) {
        let Self { scroll, drawn, viewport, reveal, .. } = *self;
        *self = Self { scroll, drawn, viewport, reveal, ..Self::new(left, right) };
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
}

/// Byte offset of character `column` in `text`, or its end.
fn byte_at(text: &str, column: usize) -> usize {
    text.char_indices().nth(column).map_or(text.len(), |(at, _)| at)
}

/// How a pane's rows are measured and coloured, the same for every row of a frame.
struct Metrics {
    font: FontId,
    height: f32,
    /// A digit's width; the gutter is the line number's digits, a space, the marker and a space.
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
        self.digit * (self.digits + 3) as f32
    }
}

/// Side `side`'s pane body (0 is left): every row, in a scroll area whose vertical offset the other
/// pane shares. `lines` are both panes' lines; a missing pane's are empty. Returns the place in the
/// text under `caret.pointer`, if it is over a line.
pub fn show(ui: &mut Ui, side: usize, lines: [&[Line]; 2], aligned: &mut Aligned, caret: &Caret) -> Option<Pos> {
    let font = TextStyle::Monospace.resolve(ui.style());
    let digit = ui.fonts_mut(|fonts| fonts.glyph_width(&font, '0'));
    let height = ui.text_style_height(&TextStyle::Monospace);
    let reveal = aligned.reveal.take_if(|(s, _)| *s == side).and_then(|(_, line)| aligned.row_of(side, line));
    let metrics = Metrics { height, digit, digits: aligned.digits, colours: Colours::of(ui), font, side, reveal };
    ui.spacing_mut().item_spacing.y = 0.0;
    let viewport = aligned.viewport.get(side).copied().unwrap_or_default();
    if let Some(row) = reveal.filter(|_| viewport > 0.0) {
        // The caret's row into view: up to the top, or down to the bottom.
        let top = row as f32 * metrics.height;
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
    let mut hit = None;
    let output = area
        .auto_shrink(false)
        // A drag selects text instead.
        .scroll_source(ScrollSource { drag: ScrollSource::NONE.drag, ..ScrollSource::ALL })
        // Both panes always show both bars, so their viewports are the same height: with a sideways
        // bar in one pane only, the shared offset would stop at the shorter pane's end and hide
        // the other's last row under its bar.
        .scroll_bar_visibility(ScrollBarVisibility::AlwaysVisible)
        // One blank row after the rest, where the caret can stand after a final line break.
        .show_rows(ui, metrics.height, aligned.rows.len() + 1, |ui, rows| {
            for row in rows {
                let at = draw_row(ui, row, lines, aligned, &metrics, caret);
                hit = hit.or(at);
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
    hit
}

/// On the row after the pane's last line, `rect`: the caret when it stands after a final line
/// break, and that place when the pointer is on the row.
fn after_last_line(ui: &Ui, row: usize, rect: Rect, aligned: &Aligned, metrics: &Metrics, caret: &Caret) -> Option<Pos> {
    let line = *aligned.lines.get(metrics.side)?;
    if aligned.row_of(metrics.side, line) != Some(row) {
        return None;
    }
    if caret.shown && caret.selection.head.line == line {
        let x = rect.left() + metrics.gutter();
        let caret_rect = Rect::from_min_max(pos2(x, rect.top()), pos2(x + 2.0, rect.bottom()));
        ui.painter().rect_filled(caret_rect, 0.0, metrics.colours.text);
        if metrics.reveal == Some(row) {
            ui.scroll_to_rect(caret_rect.expand2(vec2(metrics.gutter() + metrics.digit, 0.0)), None);
        }
    }
    // The end of the text: after the break, or the last line's end when there is none.
    caret.pointer.filter(|p| (rect.top()..rect.bottom()).contains(&p.y)).map(|_| Pos { line, column: usize::MAX })
}

/// Draws row `row` of the pane, with the caret and selection where they fall on its line, and
/// returns the place under `caret.pointer` when it is on this row's line.
fn draw_row(ui: &mut Ui, row: usize, lines: [&[Line]; 2], aligned: &mut Aligned, metrics: &Metrics, caret: &Caret) -> Option<Pos> {
    let side = metrics.side;
    let Some(&found) = aligned.rows.get(row) else {
        let (_, rect) = ui.allocate_space(vec2(ui.available_width(), metrics.height));
        return after_last_line(ui, row, rect, aligned, metrics, caret);
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
    let galley = line.map(|line| ui.fonts_mut(|fonts| fonts.layout_job(line_job(&line.text, important, &changed, selected, colours, font.clone()))));
    let gutter = metrics.gutter();
    // A digit past the text, room for a selected line break.
    let width = ui.available_width().max(gutter + galley.as_ref().map_or(0.0, |g| g.size().x) + digit);
    let (_, rect) = ui.allocate_space(vec2(width, *height));
    // A gap is padding: no number, no text, no node. Its opposite line says how it differs.
    let (Some(index), Some(galley)) = (index, galley) else { return after_last_line(ui, row, rect, aligned, metrics, caret) };
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
    let number_rect = Rect::from_min_size(pos2(left, rect.top()), vec2(digit * *digits as f32, *height));
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
        painter.text(pos2(number_rect.right() + digit * 1.5, rect.center().y), Align2::CENTER_CENTER, marker, font.clone(), text_colour);
    }
    // To the row's end, so a blank line's node has a size too.
    let text_rect = Rect::from_min_max(pos2(rect.left() + gutter, rect.top()), rect.max);
    let name = if galley.text().trim().is_empty() { t(BLANK).to_owned() } else { galley.text().to_owned() };
    let text_painter = painter.with_clip_rect(painter.clip_rect().intersect(Rect::everything_right_of(left + gutter)));
    let origin = pos2(text_rect.left(), rect.center().y - galley.size().y / 2.0);
    let hit = caret
        .pointer
        .filter(|p| (rect.top()..rect.bottom()).contains(&p.y))
        .map(|p| Pos { line: index, column: galley.cursor_from_pos(p - origin).index.into() });
    if break_selected {
        // A selected line break shows as a space's width of selection after the text.
        let at = origin.x + galley.size().x;
        text_painter.rect_filled(Rect::from_min_max(pos2(at, rect.top()), pos2(at + digit, rect.bottom())), 0.0, colours.selection);
    }
    let caret_rect = (caret.shown && caret.selection.head.line == index).then(|| {
        let x = origin.x + galley.pos_from_cursor(CCursor::new(caret.selection.head.column)).min.x;
        Rect::from_min_max(pos2(x, rect.top()), pos2(x + 2.0, rect.bottom()))
    });
    text_painter.galley(origin, galley, text_colour);
    if let Some(caret_rect) = caret_rect {
        text_painter.rect_filled(caret_rect, 0.0, text_colour);
        if *reveal == Some(row) {
            // Sideways into view, past the gutter that covers the text's left edge.
            ui.scroll_to_rect(caret_rect.expand2(vec2(gutter + digit, 0.0)), None);
        }
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
    hit
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
