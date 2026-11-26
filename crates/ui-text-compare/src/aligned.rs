//! The aligned comparison (E07-S1-T4): both panes draw the text engine's rows, a line or a gap on
//! each side, at one fixed row height, so matching lines sit on the same row. Gap rows have no
//! line number. Important differences draw in red text on a light-red line background, with the
//! changed characters inside a changed line on a stronger background and underlined, so colour
//! isn't the only signal. A gutter marker and each line's accessible description say how the
//! line differs. The two panes share one vertical scroll offset.

use std::collections::HashMap;
use std::ops::Range;

use egui::scroll_area::ScrollBarVisibility;
use egui::text::{LayoutJob, TextFormat};
use egui::{Align2, Color32, FontId, Rect, Role, ScrollArea, Sense, Stroke, TextStyle, Ui, WidgetInfo, pos2, vec2};
use otd_core::colour::{ColourRole, Contrast, Rgba};
use otd_core::i18n::t;
use otd_text_compare::Line;
use otd_text_engine::{CharDifferences, CharRange, Importance, Outcome, Row, RowKind, Side, Statistics, char_differences, classify, compare};
use otd_ui::colour::{BACKGROUND, TEXT, colour};

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
pub const CONTRAST: [Contrast; 5] = [
    Contrast::text(&IMPORTANT_TEXT, &IMPORTANT_BACKGROUND),
    Contrast::text(&IMPORTANT_TEXT, &IMPORTANT_CHARACTERS),
    Contrast::text(&LINE_NUMBER, &BACKGROUND),
    Contrast::text(&LINE_NUMBER, &IMPORTANT_BACKGROUND),
    // An important line's gutter marker.
    Contrast::text(&IMPORTANT_TEXT, &BACKGROUND),
];

// A line's accessible description: how it differs.
const CHANGED: &str = "Changed";
const APPROXIMATE: &str = "Changed, character differences approximate";
const DELETED: &str = "Deleted";
const INSERTED: &str = "Inserted";
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
    /// The vertical scroll offset both panes share.
    scroll: f32,
    /// Each pane's offset when it was last drawn. A pane that is already at the shared offset is
    /// left to scroll on its own, so its scroll animations run; the other is moved to match.
    drawn: [Option<f32>; 2],
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
        Self { rows, importance, result, chars: HashMap::new(), digits: longest.to_string().len(), scroll: 0.0, drawn: [None; 2] }
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
        }
    }
}

/// One line's text laid out: in `colours.text`, or for an important difference in
/// `colours.important_text`, with the `changed` byte ranges on the characters background and
/// underlined. Ranges outside the text or off a character boundary are left unmarked.
pub fn line_job(text: &str, important: bool, changed: &[Range<usize>], colours: &Colours, font: FontId) -> LayoutJob {
    let colour = if important { colours.important_text } else { colours.text };
    let plain = TextFormat { font_id: font, color: colour, ..TextFormat::default() };
    let marked = TextFormat { background: colours.important_characters, underline: Stroke::new(1.0, colour), ..plain.clone() };
    let mut job = LayoutJob::default();
    let mut at = 0;
    for range in changed {
        let (Some(before), Some(inside)) = (text.get(at..range.start), text.get(range.clone())) else { continue };
        job.append(before, 0.0, plain.clone());
        job.append(inside, 0.0, marked.clone());
        at = range.end;
    }
    job.append(text.get(at..).unwrap_or_default(), 0.0, plain);
    job
}

/// How a pane's rows are measured and coloured, the same for every row of a frame.
struct Metrics {
    font: FontId,
    height: f32,
    /// A digit's width; the gutter is the line number's digits, a space, the marker and a space.
    digit: f32,
    digits: usize,
    colours: Colours,
}

impl Metrics {
    fn gutter(&self) -> f32 {
        self.digit * (self.digits + 3) as f32
    }
}

/// Side `side`'s pane body (0 is left): every row, in a scroll area whose vertical offset the other
/// pane shares. `lines` are both panes' lines; a missing pane's are empty.
pub fn show(ui: &mut Ui, side: usize, lines: [&[Line]; 2], aligned: &mut Aligned) {
    let font = TextStyle::Monospace.resolve(ui.style());
    let digit = ui.fonts_mut(|fonts| fonts.glyph_width(&font, '0'));
    let metrics = Metrics { height: ui.text_style_height(&TextStyle::Monospace), digit, digits: aligned.digits, colours: Colours::of(ui), font };
    ui.spacing_mut().item_spacing.y = 0.0;
    let mut area = ScrollArea::both().id_salt(("text-compare-pane", side));
    if aligned.drawn.get(side).copied().flatten() != Some(aligned.scroll) {
        area = area.vertical_scroll_offset(aligned.scroll);
    }
    let output = area
        .auto_shrink(false)
        // Both panes always show both bars, so their viewports are the same height: with a sideways
        // bar in one pane only, the shared offset would stop at the shorter pane's end and hide
        // the other's last row under its bar.
        .scroll_bar_visibility(ScrollBarVisibility::AlwaysVisible)
        .show_rows(ui, metrics.height, aligned.rows.len(), |ui, rows| {
            for row in rows {
                draw_row(ui, side, row, lines, aligned, &metrics);
            }
        });
    let offset = output.state.clamped_offset().y;
    if let Some(drawn) = aligned.drawn.get_mut(side) {
        *drawn = Some(offset);
    }
    if offset != aligned.scroll {
        aligned.scroll = offset;
        // The pane drawn before this one catches up on the next frame.
        ui.ctx().request_repaint();
    }
}

fn draw_row(ui: &mut Ui, side: usize, row: usize, lines: [&[Line]; 2], aligned: &mut Aligned, metrics: &Metrics) {
    let Some(&found) = aligned.rows.get(row) else { return };
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
    let Metrics { font, height, digit, digits, colours } = metrics;
    let galley = line.map(|line| ui.fonts_mut(|fonts| fonts.layout_job(line_job(&line.text, important, &changed, colours, font.clone()))));
    let gutter = metrics.gutter();
    let width = ui.available_width().max(gutter + galley.as_ref().map_or(0.0, |g| g.size().x));
    let (_, rect) = ui.allocate_space(vec2(width, *height));
    // A gap is padding: no number, no text, no node. Its opposite line says how it differs.
    let (Some(index), Some(galley)) = (index, galley) else { return };
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
    text_painter.galley(pos2(text_rect.left(), rect.center().y - galley.size().y / 2.0), galley, text_colour);
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
        };
        let (left, right) = ("let tötal = count + 1;", "let tötal = amount + 2;");
        let differences = char_differences(left, right);
        let [(left_marked, left_rest), (right_marked, right_rest)] = [(Side::Left, left), (Side::Right, right)].map(|(side, text)| {
            let ranges: Vec<_> = differences.ranges.iter().filter(|r| r.side == side).map(|r| r.start..r.end).collect();
            let job = line_job(text, true, &ranges, &colours, FontId::monospace(12.0));
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
