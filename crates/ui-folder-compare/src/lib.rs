//! Folder Compare (E08-S1): two folder trees side by side, their items aligned by name, so that
//! partners share a row and an item with no partner faces a gap.
//!
//! It opens from Home and *Session > New Session* (the [`session_type`] the app registers) and
//! from the command line with two folders ([`OpenFolders`], `otd <folder> <folder>`). Each pane
//! has a path edit; the two paths are the session's inputs (`Session::specs`), so the Inputs tab
//! of Comparison Settings edits the same two paths.
//!
//! The scans and the comparison are the folder engine's (E03-S2): each side is a [`TreeScan`] of a
//! [`LocalSource`], and the rows are [`compare_trees`]' rows. Where later stories extend the view:
//! the columns each pane shows after the name are [`COLUMNS`]; the rows drawn, in order, come from
//! [`FolderCompare::shown`]; commands go in [`commands`], which the app's command list holds, for
//! the view type [`ID`].
//!
//! The row area is a tree grid: Tab reaches it once, on the current name cell, and the arrow keys
//! move focus between rows (Up, Down) and sides (Left, Right). Each row and cell is a node, and a
//! gap is a cell named for what is missing, so a screen reader hears the gap that the eye sees.
//! A craft-libs candidate (alextiboldi/overthediff#84): it stays here until craft-libs has a tree grid.
//!
//! Each side has a selection of its own (E08-S1-T5): a click on a row's left or right cells selects
//! that side's item, Ctrl+click (Cmd+click on macOS) adds or removes one, and Shift+click selects
//! the range from the last item clicked, on that side only. By keyboard, Up and Down move focus and
//! select as a click does, Shift+Up and Shift+Down select the range, Ctrl+Up and Ctrl+Down (Cmd on
//! macOS) move focus only, and Ctrl+Space adds or removes the focused item.
//!
//! Each side of a row is drawn in its status colour, with a colour spot left of the name and a
//! link's arrow or an unreadable item's red X after it; [`look`] decides them. Its status text is
//! the name cell's accessible description and its tooltip, shown on hover and on keyboard focus
//! (Esc hides it), so colour is never the only signal.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use egui::accesskit::{Action, Live, Role};
use egui::{
    Color32, EventFilter, Key, Modifiers, PointerButton, Rect, Response, Sense, StrokeKind, TextStyle, TextWrapMode, Ui, Vec2, WidgetInfo, WidgetText,
};
use otd_core::i18n::{fmt, t, trn};
use otd_folder_engine::{Criteria, DEFAULT_TOLERANCE, Row, ScanEvent, TreeScan, TreeSide, case_sensitive, compare_trees};
use otd_launch::{Completion, Handler, Pattern, Request};
use otd_source::{Capabilities, DataSource, Item, ItemKind, LocalSource};
use otd_ui::colour::{ColourRole, SELECTION, SELECTION_TEXT, colour};
use otd_ui::{Browse, PathEdit, Session, SessionType, Shell, Status, View, paint_lucide};

mod info;
pub mod look;
mod refresh;

pub use info::{
    COUNT_DIFFERENT, COUNT_ERRORS, COUNT_LEFT_NEWER, COUNT_LEFT_ORPHANS, COUNT_RIGHT_NEWER, COUNT_RIGHT_ORPHANS, COUNT_SAME, COUNT_UNKNOWN, Entry,
    FOLDER_COMPARE_INFO, FOLDERS_LISTED, Info, LEGEND, LEGEND_ENTRIES, NO_RESULT, NOT_FULLY_COMPARED, RESULT, SCAN, STILL_SCANNING, Swatch,
};
pub use look::{COLOUR_ROLES, CONTRAST, Look, look};
pub use refresh::{FULL_REFRESH, REFRESH, REFRESH_SELECTION, SWAP_SIDES, commands};

/// The session type's id, which is also the view type its commands name.
pub const ID: &str = "folder-compare";

// English keys, drawn only through `t`.
pub const FOLDER_COMPARE: &str = "Folder Compare";
pub const LEFT: &str = "Left";
pub const RIGHT: &str = "Right";
pub const NAME: &str = "Name";
pub const SIZE: &str = "Size";
pub const MODIFIED: &str = "Modified";
/// A gap's name: the row's item is only on the other side.
pub const NOT_ON_LEFT: &str = "Not on the left";
pub const NOT_ON_RIGHT: &str = "Not on the right";
/// A name cell's accessible name, so the two sides' cells of one row differ.
pub const ON_LEFT: &str = "{name} on the left";
pub const ON_RIGHT: &str = "{name} on the right";
const BYTE: &str = "{n} byte";
const BYTES: &str = "{n} bytes";
/// The usage line of `otd <folder> <folder>`.
// ponytail: English, as `otd_launch::Registry::usage` prints every handler's line.
const USAGE: &str = "<folder> <folder>   Compare two folders side by side";

/// Size and time quick tests with the default tolerance (E08-S1-T1); the Comparison tab (E08-S4)
/// makes them settings.
const CRITERIA: Criteria = Criteria { size: true, modified: true, tolerance: DEFAULT_TOLERANCE };

/// How often the view looks for scan progress while a scan runs; the scan's worker can't wake it.
const SCAN_POLL: Duration = Duration::from_millis(50);

/// The Folder Compare session type, for the app's session-type list.
pub fn session_type() -> SessionType {
    SessionType { id: ID, name: FOLDER_COMPARE, kinds: &[], sides: &[LEFT, RIGHT], new: || Box::new(FolderCompare::new()) }
}

/// A column each pane shows after the name: its caption (an English key) and an item's text in
/// it, `None` for a blank cell. E08-S6 adds columns here.
pub struct Column {
    pub caption: &'static str,
    /// Its share of a pane's width; the name has the rest.
    pub width: f32,
    pub cell: fn(&Item) -> Option<String>,
}

pub const COLUMNS: &[Column] = &[Column { caption: SIZE, width: 0.2, cell: size }, Column { caption: MODIFIED, width: 0.3, cell: modified }];

fn size(item: &Item) -> Option<String> {
    (item.kind == ItemKind::File).then(|| trn(item.size, BYTE, BYTES))
}

/// In the local time zone.
// ponytail: one ISO-like format for every language; the region's own date format comes with the
// column options (E08-S6).
fn modified(item: &Item) -> Option<String> {
    let time: chrono::DateTime<chrono::Local> = item.modified?.into();
    Some(time.format("%Y-%m-%d %H:%M:%S").to_string())
}

/// One side's scan.
struct Scan {
    scan: TreeScan,
    events: Receiver<ScanEvent>,
    capabilities: Capabilities,
    /// The last event said nothing was left to list.
    idle: bool,
    /// The last event's folders listed so far and still to list, for the status bar.
    done: usize,
    pending: usize,
}

/// A Folder Compare tab.
pub struct FolderCompare {
    session: Session,
    /// The inputs the scans are of. When the session's differ (a path typed or browsed, or the
    /// Inputs tab), the next draw scans again.
    scanned: Option<Vec<String>>,
    scans: [Option<Scan>; 2],
    /// Why a side can't be shown.
    errors: [Option<String>; 2],
    rows: Vec<Row>,
    /// The name cell that Tab reaches in the row area: a row of [`FolderCompare::shown`] and a side.
    current: (usize, usize),
    /// Move focus to `current` on the next draw.
    refocus: bool,
    /// Each side's selected rows, as row-name paths from the base: the shape
    /// [`otd_folder_engine::resolve_selection`] takes. Only rows with an item on that side.
    selected: [BTreeSet<PathBuf>; 2],
    /// Each side's last plain or Ctrl click (or key), where a Shift range starts. With none, a range
    /// starts at the focused cell when it is on that side.
    anchor: [Option<PathBuf>; 2],
    /// Esc hid the focused cell's status tooltip; it shows again when focus moves to a cell.
    quiet: bool,
    /// The open Info or Legend dialog.
    dialog: Option<info::Dialog>,
    /// Gets focus on the next draw: what had it before a dialog opened.
    focus_back: Option<egui::Id>,
    /// Reports the close to a launcher waiting for this view.
    done: Option<Completion>,
}

impl Default for FolderCompare {
    fn default() -> Self {
        Self::new()
    }
}

impl FolderCompare {
    /// An empty session: no folders yet.
    pub fn new() -> Self {
        Self {
            session: Session::new(ID),
            scanned: None,
            scans: [None, None],
            errors: [None, None],
            rows: Vec::new(),
            current: (0, 0),
            refocus: false,
            selected: Default::default(),
            anchor: [None, None],
            quiet: false,
            dialog: None,
            focus_back: None,
            done: None,
        }
    }

    /// A session of the folders `args` names, relative ones resolved against `cwd`, the folder the
    /// command line was typed in.
    pub fn of_folders(args: &[String], cwd: &Path) -> Self {
        let mut view = Self::new();
        view.session.specs = args.iter().map(|arg| cwd.join(arg).to_string_lossy().into_owned()).collect();
        view
    }

    /// Whether a scan is still listing folders, so the rows may still change.
    pub fn scanning(&self) -> bool {
        self.scans.iter().flatten().any(|scan| !scan.idle)
    }

    /// The rows drawn, in order.
    // ponytail: the base folders' rows only; expanding a folder to show its rows is E08-S1-T3.
    pub fn shown(&self) -> &[Row] {
        &self.rows
    }

    /// The rows selected on `side` (0 left, 1 right), as row-name paths from the base, in tree order.
    /// A path whose row has since gone stays until the next click; `resolve_selection` ignores it.
    pub fn selected(&self, side: usize) -> Vec<&Path> {
        self.selected.get(side).into_iter().flatten().map(PathBuf::as_path).collect()
    }

    /// Selects on `side` as a click on row `index` of [`FolderCompare::shown`] does. A gap has no item
    /// to select, so a click there only clears (plain) or keeps (Ctrl) the side's selection and moves
    /// the anchor, and a range takes only the items in it (*unspecified*, decided here).
    fn pick(&mut self, index: usize, side: usize, pick: Pick) {
        let has = |row: &Row| if side == 0 { row.left.is_some() } else { row.right.is_some() };
        let Some(row) = self.shown().get(index) else { return };
        let (path, item) = (row_path(row), has(row));
        let selected = &mut self.selected[side];
        match pick {
            Pick::Only => {
                selected.clear();
                if item {
                    selected.insert(path.clone());
                }
            }
            Pick::Toggle => {
                if item && !selected.remove(&path) {
                    selected.insert(path.clone());
                }
            }
            Pick::Range => {
                let anchor = self.anchor[side].as_ref().and_then(|anchor| self.rows.iter().position(|row| row_path(row) == *anchor));
                // With no anchor yet, the range starts at the focused cell on this side, so Shift+Down
                // after Tab takes the focused item too.
                let from = anchor.or((self.current.1 == side).then_some(self.current.0)).unwrap_or(index).min(self.rows.len().saturating_sub(1));
                let range = &self.rows[from.min(index)..=from.max(index)];
                *selected = range.iter().filter(|row| has(row)).map(row_path).collect();
                if anchor.is_none() {
                    self.anchor[side] = Some(row_path(&self.rows[from]));
                }
                return;
            }
        }
        self.anchor[side] = Some(path);
    }

    fn rescan(&mut self) {
        self.scanned = Some(self.session.specs.clone());
        self.rows.clear();
        self.selected = Default::default();
        self.anchor = [None, None];
        self.errors = [None, None];
        self.scans = [None, None];
        for side in 0..2 {
            let spec = self.session.specs.get(side).map_or("", |spec| spec.trim());
            if spec.is_empty() {
                continue;
            }
            match LocalSource::open(spec) {
                Ok(source) => {
                    let capabilities = source.capabilities();
                    // Background scanning off: T2 draws only the base folders' rows, and with it on each
                    // batch of listings would compare the whole tree listed so far on this thread.
                    // ponytail: the Handling tab makes it a setting (E08-S11); expanding opens folders (T3).
                    let (scan, events) = TreeScan::new(Arc::new(source), "", false);
                    match scan.start() {
                        Ok(()) => self.scans[side] = Some(Scan { scan, events, capabilities, idle: false, done: 0, pending: 0 }),
                        Err(err) => self.errors[side] = Some(err.to_string()),
                    }
                }
                Err(err) => self.errors[side] = Some(err.to_string()),
            }
        }
    }

    /// Takes in what the scans listed since the last draw.
    fn update(&mut self) {
        let mut listed = false;
        for scan in self.scans.iter_mut().flatten() {
            while let Ok(event) = scan.events.try_recv() {
                scan.idle = event.pending == 0;
                (scan.done, scan.pending) = (event.done, event.pending);
                listed = true;
            }
        }
        if !listed {
            return;
        }
        // Assigned, not only set, so a base folder that a refresh could list again shows again.
        for (side, scan) in self.scans.iter().enumerate() {
            if let Some(scan) = scan {
                self.errors[side] = scan.scan.folder(Path::new("")).and_then(|folder| folder.error).map(|error| error.to_string());
            }
        }
        // ponytail: compares everything listed so far again after each batch of listings, which is the
        // base folders until expansion (T3); update only the listed folders' rows if that gets slow.
        // A side that can't be listed has unknown contents, so nothing on the other faces a gap,
        // even when the other side was listed first.
        self.rows = match (&self.scans, &self.errors) {
            ([Some(left), Some(right)], [None, None]) => {
                let case_sensitive = case_sensitive(&left.capabilities, &right.capabilities);
                compare_trees(&left.scan, &right.scan, &CRITERIA, case_sensitive, &|_, _, _| true)
            }
            _ => Vec::new(),
        };
        self.current.0 = self.current.0.min(self.rows.len().saturating_sub(1));
    }

    /// The label, path edit and any error of one side. Returns the path edit's response.
    fn pane_header(&mut self, ui: &mut Ui, side: usize) -> Response {
        let (caption, field) = [(LEFT, "folder-compare.left"), (RIGHT, "folder-compare.right")][side];
        if self.session.specs.len() <= side {
            self.session.specs.resize(side + 1, String::new());
        }
        let response = ui
            .horizontal(|ui| {
                let label = ui.label(t(caption));
                PathEdit::new(field, &mut self.session.specs[side], Browse::Folder).show(ui).labelled_by(label.id)
            })
            .inner;
        if let Some(error) = &self.errors[side] {
            let label = ui.label(error);
            // Announced: it comes from the scan, away from focus.
            ui.ctx().accesskit_node_builder(label.id, |node| node.set_live(Live::Polite));
        }
        response
    }

    /// The tree grid, in a ui of its own so that the path edits above it aren't grid content.
    fn rows_ui(&mut self, ui: &mut Ui) {
        ui.scope(|ui| self.grid_ui(ui));
    }

    fn grid_ui(&mut self, ui: &mut Ui) {
        let height = ui.text_style_height(&TextStyle::Body) + ui.spacing().item_spacing.y;
        let pane = (ui.available_width() - 2.0 * ui.spacing().item_spacing.x * (1 + COLUMNS.len()) as f32) / 2.0;
        let name_width = pane * (1.0 - COLUMNS.iter().map(|c| c.width).sum::<f32>());
        let grid = ui.unique_id();
        ui.ctx().accesskit_node_builder(grid, |node| {
            node.set_role(Role::TreeGrid);
            node.set_label(t(FOLDER_COMPARE));
        });
        ui.horizontal(|ui| {
            ui.ctx().accesskit_node_builder(ui.unique_id(), |node| node.set_role(Role::Row));
            for _ in 0..2 {
                header(ui, NAME, name_width, height);
                for column in COLUMNS {
                    header(ui, column.caption, pane * column.width, height);
                }
            }
        });
        ui.ctx().accesskit_node_builder(grid, |node| node.set_multiselectable());
        let mut moved = None;
        let mut picked = None;
        egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            for (index, row) in self.rows.iter().enumerate() {
                ui.push_id(index, |ui| {
                    ui.horizontal(|ui| {
                        ui.ctx().accesskit_node_builder(ui.unique_id(), |node| {
                            node.set_role(Role::Row);
                            node.set_label(row.name.as_str());
                        });
                        for (side, item) in [&row.left, &row.right].into_iter().enumerate() {
                            let current = self.current == (index, side);
                            let selected = item.is_some().then(|| self.selected[side].contains(&row_path(row)));
                            // ponytail: `look` walks a folder's listed subtree (`Row::cues`) every frame; work the
                            // looks out in `update` when the rows change if expanded trees (T3, E08-S11) make it slow.
                            let look = look(row, [TreeSide::Left, TreeSide::Right][side]);
                            let size = Vec2::new(name_width, height);
                            let response = name_cell(ui, item.as_ref(), look.as_ref(), side, current, selected, size);
                            if current && std::mem::take(&mut self.refocus) {
                                response.request_focus();
                                response.scroll_to_me(None);
                            }
                            if response.has_focus() {
                                if response.gained_focus() {
                                    self.quiet = false;
                                }
                                moved = moved.or_else(|| arrow(ui, &response, index, side, self.rows.len()));
                                self.quiet |= ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape));
                                // The status in words on keyboard focus too, while the cell is in view. It covers
                                // the row below, so Esc hides it without moving focus (WCAG 1.4.13).
                                if let Some(look) = &look
                                    && !self.quiet
                                    && !response.hovered()
                                    && ui.is_rect_visible(response.rect)
                                {
                                    response.show_tooltip_text(look.description());
                                }
                            }
                            // The whole side of the row takes its status colour.
                            let status = look.as_ref().map_or(ui.visuals().text_color(), |look| colour(ui.ctx(), look.colour()));
                            // A click on any of the side's cells.
                            let mut clicked = activated(&response);
                            for column in COLUMNS {
                                // A blank cell is still allocated, so it takes the item spacing a filled one does.
                                let text = item.as_ref().and_then(column.cell).unwrap_or_default();
                                let response = cell(ui, &text, Vec2::new(pane * column.width, height), Sense::CLICK, selected == Some(true), status);
                                clicked |= activated(&response);
                            }
                            if clicked {
                                picked = Some(((index, side), Some(Pick::of_click(ui.input(|input| input.modifiers)))));
                            }
                        }
                    });
                });
            }
        });
        if let Some((current, pick)) = picked.or(moved) {
            if let Some(pick) = pick {
                self.pick(current.0, current.1, pick);
            }
            self.current = current;
            self.refocus = true;
            ui.ctx().request_repaint();
        }
    }
}

/// A pointer click, or a screen reader's activate (AccessKit `Click`). Not `clicked()`, which also
/// counts Space and Enter on the focused cell: those stay free for E08-S10.
fn activated(response: &Response) -> bool {
    response.clicked_by(PointerButton::Primary) || response.ctx.input(|input| input.has_accesskit_action_request(response.id, Action::Click))
}

/// What a click or key does to one side's selection.
#[derive(Debug, Clone, Copy)]
enum Pick {
    /// Select this item only.
    Only,
    /// Add or remove this item.
    Toggle,
    /// Select from the anchor to here.
    Range,
}

impl Pick {
    fn of_click(modifiers: Modifiers) -> Self {
        // `command` is Ctrl, or Cmd on macOS, where Ctrl+click is a secondary click.
        if modifiers.shift {
            Self::Range
        } else if modifiers.command {
            Self::Toggle
        } else {
            Self::Only
        }
    }
}

/// A shown row's path of row names from the base.
// ponytail: the base folders' rows only, so the path is the row's name; T3's expansion makes it
// the path through the open folders.
fn row_path(row: &Row) -> PathBuf {
    PathBuf::from(&row.name)
}

impl Drop for FolderCompare {
    fn drop(&mut self) {
        // ponytail: 0 for any close; the exit codes for a compared result are E11's.
        if let Some(done) = self.done.take() {
            done.finish(0);
        }
    }
}

impl View for FolderCompare {
    fn title(&self) -> String {
        t(FOLDER_COMPARE).to_owned()
    }

    fn view_type(&self) -> &'static str {
        ID
    }

    fn session(&mut self) -> Option<&mut Session> {
        Some(&mut self.session)
    }

    fn status(&self) -> Status {
        self.status_fields()
    }

    fn ui(&mut self, ui: &mut Ui) {
        let editing = ui.columns(2, |columns| {
            let left = self.pane_header(&mut columns[0], 0).has_focus();
            let right = self.pane_header(&mut columns[1], 1).has_focus();
            left || right
        });
        // Not while a path is being typed: it counts once the edit loses focus (Enter or Tab).
        if !editing && self.scanned.as_ref() != Some(&self.session.specs) {
            self.rescan();
        }
        self.update();
        if self.scanning() {
            ui.ctx().request_repaint_after(SCAN_POLL);
        }
        ui.separator();
        self.rows_ui(ui);
        self.dialog_ui(ui);
    }
}

fn header(ui: &mut Ui, caption: &str, width: f32, height: f32) {
    let response = cell(ui, t(caption), Vec2::new(width, height), Sense::hover(), false, ui.visuals().text_color());
    ui.ctx().accesskit_node_builder(response.id, |node| node.set_role(Role::ColumnHeader));
}

/// A cell showing `text` in `colour`, cut short to fit, named `text` for screen readers.
fn cell(ui: &mut Ui, text: &str, size: Vec2, sense: Sense, selected: bool, colour: Color32) -> Response {
    let (rect, response) = ui.allocate_exact_size(size, sense);
    response.widget_info(|| WidgetInfo::labeled(Role::Cell, true, text));
    let colour = paint_selection(ui, rect, selected, colour);
    paint_text(ui, rect, text, colour);
    response
}

/// Fills a selected cell, gaps between cells included, and gives the colour its text takes:
/// `unselected` when it isn't selected.
fn paint_selection(ui: &Ui, rect: Rect, selected: bool, unselected: Color32) -> Color32 {
    if !selected {
        return unselected;
    }
    ui.painter().rect_filled(rect.expand2(ui.spacing().item_spacing / 2.0), 0.0, colour(ui.ctx(), &SELECTION));
    colour(ui.ctx(), &SELECTION_TEXT)
}

/// Paints `text` cut short to fit `rect`, and returns how wide it came out.
fn paint_text(ui: &Ui, rect: Rect, text: &str, colour: Color32) -> f32 {
    let galley = WidgetText::from(text).into_galley(ui, Some(TextWrapMode::Truncate), rect.width(), TextStyle::Body);
    let at = egui::pos2(rect.left(), rect.center().y - galley.size().y / 2.0);
    let width = galley.size().x;
    ui.painter().galley(at, galley, colour);
    width
}

/// The colour spot's side, and a glyph's.
const SPOT: f32 = 10.0;
const GLYPH: f32 = 12.0;

/// A row's name on one side, in its status colour after its colour spot, or the gap where the
/// other side's item has no partner. Only the current cell takes focus, so Tab stops once in the
/// row area; every cell takes clicks. `selected` is `None` for a gap, which has nothing to select.
fn name_cell(ui: &mut Ui, item: Option<&Item>, look: Option<&Look>, side: usize, current: bool, selected: Option<bool>, size: Vec2) -> Response {
    let (rect, mut response) = ui.allocate_exact_size(size, if current { Sense::click() } else { Sense::CLICK });
    let name = match item {
        Some(item) => fmt(t([ON_LEFT, ON_RIGHT][side]), &[("name", &item.name)]),
        None => t([NOT_ON_LEFT, NOT_ON_RIGHT][side]).to_owned(),
    };
    response.widget_info(|| WidgetInfo::labeled(Role::Cell, true, &name));
    if let Some(selected) = selected {
        ui.ctx().accesskit_node_builder(response.id, |node| node.set_selected(selected));
    }
    let status = look.map_or(ui.visuals().text_color(), |look| colour(ui.ctx(), look.colour()));
    let text_colour = paint_selection(ui, rect, selected == Some(true), status);
    if let (Some(item), Some(look)) = (item, look) {
        paint_spot(ui, rect, &look.spot());
        let glyphs = [(look.link, look::LINK_ICON, look.colour()), (look.error.is_some(), look::ERROR_ICON, &look::ERROR_GLYPH)];
        let glyphs: Vec<_> = glyphs.into_iter().filter(|(on, ..)| *on).collect();
        let text =
            Rect::from_min_max(rect.min + Vec2::new(SPOT + ui.spacing().icon_spacing, 0.0), rect.max - Vec2::new(GLYPH * glyphs.len() as f32, 0.0));
        let width = paint_text(ui, text, &item.name, text_colour);
        for (at, (_, icon, role)) in glyphs.into_iter().enumerate() {
            let center = egui::pos2(text.left() + width + GLYPH * (at as f32 + 0.5), rect.center().y);
            paint_lucide(ui, icon, center, GLYPH, colour(ui.ctx(), role));
        }
        // The tooltip is also the cell's accessible description.
        response = response.on_hover_text(look.description());
    }
    if response.has_focus() {
        ui.painter().rect_stroke(rect, 0.0, ui.visuals().selection.stroke, StrokeKind::Inside);
    }
    response
}

/// The colour spot at the left of a name cell: one colour, or a folder's several side by side.
fn paint_spot(ui: &Ui, cell: Rect, colours: &[&ColourRole]) {
    let spot = Rect::from_center_size(egui::pos2(cell.left() + SPOT / 2.0, cell.center().y), Vec2::splat(SPOT));
    let part = SPOT / colours.len().max(1) as f32;
    for (at, role) in colours.iter().enumerate() {
        let left = spot.left() + part * at as f32;
        let rect = Rect::from_min_max(egui::pos2(left, spot.top()), egui::pos2(left + part, spot.bottom()));
        ui.painter().rect_filled(rect, 0.0, colour(ui.ctx(), role));
    }
}

/// The cell a key pressed in the focused name cell moves to, and what it does to that side's
/// selection. The arrows stay with the row area rather than moving egui's focus to whatever widget
/// lies that way. Up and Down select as a click does (with Shift, the range from the anchor), and
/// with Ctrl (Cmd on macOS) only move; Left and Right only move; Ctrl+Space adds or removes the
/// focused item, as Ctrl+click does (the Control key on every platform: Cmd+Space is the system's).
fn arrow(ui: &mut Ui, response: &Response, row: usize, side: usize, rows: usize) -> Option<((usize, usize), Option<Pick>)> {
    // Esc too, so it hides the status tooltip instead of taking focus from the cell.
    let filter = EventFilter { horizontal_arrows: true, vertical_arrows: true, escape: true, ..EventFilter::default() };
    ui.memory_mut(|memory| memory.set_focus_lock_filter(response.id, filter));
    let pressed = |modifiers, key| ui.input_mut(|input| input.consume_key(modifiers, key));
    let up = (row.saturating_sub(1), side);
    let down = ((row + 1).min(rows.saturating_sub(1)), side);
    let pick = if ui.input(|input| input.modifiers.shift) { Pick::Range } else { Pick::Only };
    if pressed(Modifiers::COMMAND, Key::ArrowUp) {
        Some((up, None))
    } else if pressed(Modifiers::COMMAND, Key::ArrowDown) {
        Some((down, None))
    } else if pressed(Modifiers::CTRL, Key::Space) {
        Some(((row, side), Some(Pick::Toggle)))
    } else if pressed(Modifiers::NONE, Key::ArrowUp) {
        Some((up, Some(pick)))
    } else if pressed(Modifiers::NONE, Key::ArrowDown) {
        Some((down, Some(pick)))
    } else if pressed(Modifiers::NONE, Key::ArrowLeft) {
        Some(((row, 0), None))
    } else if pressed(Modifiers::NONE, Key::ArrowRight) {
        Some(((row, 1), None))
    } else {
        None
    }
}

/// `otd <folder> <folder>`: one or two existing folders open a Folder Compare.
pub struct OpenFolders;

impl Handler<Shell> for OpenFolders {
    fn id(&self) -> &'static str {
        ID
    }

    fn pattern(&self) -> Pattern {
        Pattern::Folders
    }

    fn usage(&self) -> &'static str {
        USAGE
    }

    fn open(&self, request: Request, done: Completion, shell: &mut Shell) {
        let mut view = FolderCompare::of_folders(&request.args, &request.cwd);
        view.done = Some(done);
        shell.open(Box::new(view));
    }
}
