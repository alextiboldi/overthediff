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
//! [`FolderCompare::shown`]; commands go in the app's command list for the view type [`ID`].
//!
//! The row area is a tree grid: Tab reaches it once, on the current name cell, and the arrow keys
//! move focus between rows (Up, Down) and sides (Left, Right). Each row and cell is a node, and a
//! gap is a cell named for what is missing, so a screen reader hears the gap that the eye sees.
//! A craft-libs candidate (alextiboldi/overthediff#84): it stays here until craft-libs has a tree grid.

use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use egui::accesskit::{Live, Role};
use egui::{EventFilter, Key, Modifiers, Rect, Response, Sense, StrokeKind, TextStyle, TextWrapMode, Ui, Vec2, WidgetInfo, WidgetText};
use otd_core::i18n::{fmt, t, trn};
use otd_folder_engine::{Criteria, DEFAULT_TOLERANCE, Row, ScanEvent, TreeScan, case_sensitive, compare_trees};
use otd_launch::{Completion, Handler, Pattern, Request};
use otd_source::{Capabilities, DataSource, Item, ItemKind, LocalSource};
use otd_ui::{Browse, PathEdit, Session, SessionType, Shell, View};

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

    fn rescan(&mut self) {
        self.scanned = Some(self.session.specs.clone());
        self.rows.clear();
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
                    // ponytail: background scanning always on; the Handling tab makes it a setting (E08-S11).
                    let (scan, events) = TreeScan::new(Arc::new(source), "", true);
                    match scan.start() {
                        Ok(()) => self.scans[side] = Some(Scan { scan, events, capabilities, idle: false }),
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
                listed = true;
            }
        }
        if !listed {
            return;
        }
        for (side, scan) in self.scans.iter().enumerate() {
            if let Some(error) = scan.as_ref().and_then(|scan| scan.scan.folder(Path::new(""))?.error) {
                self.errors[side] = Some(error.to_string());
            }
        }
        // ponytail: compares the whole tree again after each batch of listings; keep the rows and
        // update the listed folders' if large trees make that slow.
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

    fn rows_ui(&mut self, ui: &mut Ui) {
        let height = ui.text_style_height(&TextStyle::Body) + ui.spacing().item_spacing.y;
        let pane = (ui.available_width() - 2.0 * ui.spacing().item_spacing.x * (1 + COLUMNS.len()) as f32) / 2.0;
        let name_width = pane * (1.0 - COLUMNS.iter().map(|c| c.width).sum::<f32>());
        let grid = ui.unique_id();
        ui.ctx().accesskit_node_builder(grid, |node| {
            node.set_role(Role::TreeGrid);
            node.set_label(t(FOLDER_COMPARE));
        });
        ui.horizontal(|ui| {
            for _ in 0..2 {
                header(ui, NAME, name_width, height);
                for column in COLUMNS {
                    header(ui, column.caption, pane * column.width, height);
                }
            }
        });
        let mut moved = None;
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
                            let response = name_cell(ui, item.as_ref(), side, current, Vec2::new(name_width, height));
                            if current && std::mem::take(&mut self.refocus) {
                                response.request_focus();
                                response.scroll_to_me(None);
                            }
                            if response.has_focus() {
                                moved = moved.or_else(|| arrow(ui, &response, index, side, self.rows.len()));
                            }
                            for column in COLUMNS {
                                let width = pane * column.width;
                                match item.as_ref().and_then(column.cell) {
                                    Some(text) => drop(cell(ui, &text, Vec2::new(width, height), Sense::hover())),
                                    None => ui.add_space(width),
                                }
                            }
                        }
                    });
                });
            }
        });
        if let Some(current) = moved {
            self.current = current;
            self.refocus = true;
            ui.ctx().request_repaint();
        }
    }
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
    }
}

fn header(ui: &mut Ui, caption: &str, width: f32, height: f32) {
    let response = cell(ui, t(caption), Vec2::new(width, height), Sense::hover());
    ui.ctx().accesskit_node_builder(response.id, |node| node.set_role(Role::ColumnHeader));
}

/// A cell showing `text`, cut short to fit, named `text` for screen readers.
fn cell(ui: &mut Ui, text: &str, size: Vec2, sense: Sense) -> Response {
    let (rect, response) = ui.allocate_exact_size(size, sense);
    response.widget_info(|| WidgetInfo::labeled(Role::Cell, true, text));
    paint_text(ui, rect, text);
    response
}

fn paint_text(ui: &Ui, rect: Rect, text: &str) {
    let galley = WidgetText::from(text).into_galley(ui, Some(TextWrapMode::Truncate), rect.width(), TextStyle::Body);
    let at = egui::pos2(rect.left(), rect.center().y - galley.size().y / 2.0);
    ui.painter().galley(at, galley, ui.visuals().text_color());
}

/// A row's name on one side, or the gap where the other side's item has no partner. Only the
/// current cell takes focus, so Tab stops once in the row area.
fn name_cell(ui: &mut Ui, item: Option<&Item>, side: usize, current: bool, size: Vec2) -> Response {
    let (rect, response) = ui.allocate_exact_size(size, if current { Sense::click() } else { Sense::hover() });
    let name = match item {
        Some(item) => fmt(t([ON_LEFT, ON_RIGHT][side]), &[("name", &item.name)]),
        None => t([NOT_ON_LEFT, NOT_ON_RIGHT][side]).to_owned(),
    };
    response.widget_info(|| WidgetInfo::labeled(Role::Cell, true, &name));
    if let Some(item) = item {
        paint_text(ui, rect, &item.name);
    }
    if response.has_focus() {
        ui.painter().rect_stroke(rect, 0.0, ui.visuals().selection.stroke, StrokeKind::Inside);
    }
    response
}

/// The cell an arrow key pressed in the focused name cell moves to. The arrows stay with the row
/// area rather than moving egui's focus to whatever widget lies that way.
fn arrow(ui: &mut Ui, response: &Response, row: usize, side: usize, rows: usize) -> Option<(usize, usize)> {
    let filter = EventFilter { horizontal_arrows: true, vertical_arrows: true, ..EventFilter::default() };
    ui.memory_mut(|memory| memory.set_focus_lock_filter(response.id, filter));
    let pressed = |key| ui.input_mut(|input| input.consume_key(Modifiers::NONE, key));
    if pressed(Key::ArrowUp) {
        Some((row.saturating_sub(1), side))
    } else if pressed(Key::ArrowDown) {
        Some(((row + 1).min(rows.saturating_sub(1)), side))
    } else if pressed(Key::ArrowLeft) {
        Some((row, 0))
    } else if pressed(Key::ArrowRight) {
        Some((row, 1))
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
