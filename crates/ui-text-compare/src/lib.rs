//! The Text Compare view (story E07-S1): a left and a right pane, each with its file info header
//! (path, size, modified time, file format, encoding and line-ending style) above its text. The
//! panes load their files through `otd-text-compare` from the session's inputs, one per side,
//! whenever those change.
//!
//! [`SESSION_TYPE`] is the session type that opens it, from Home and *New Session*; the app
//! registers it in `apps/otd/src/session_types.rs`. Each pane has a path edit, [`commands`] are
//! *File > Open File* and the hidden *Open File on File System*, and [`OpenFiles`] opens two files
//! given on the command line (E07-S1-T2).
//!
//! `edited.rs` holds *Search > Next Edit* and *Previous Edit* (E07-S2-T3).
//!
//! Below the headers, [`aligned`] draws the two files aligned row by row, with their differences
//! highlighted, scrolling together (E07-S1-T4). Each pane body is also an editor, compared again as
//! the user types, with the Edit menu's commands (E07-S1-T5, `edit.rs`).
//!
//! [`session`] holds the Session menu's Reload Files, Recompare Files, Swap Sides and Text Compare
//! Info, and the status bar's fields (E07-S1-T8).
//!
//! `save.rs` holds *File > Save File*, *Save File As* and the hidden *Save File on File System As*,
//! each pane's unsaved-edits mark, and the view's answers to the shell's unsaved-changes prompts
//! (E07-S1-T6).
//!
//! `font_size.rs` holds the hidden *Increase*, *Decrease* and *Reset Font Size* (E07-S9-T4).
//!
//! `go_to.rs` holds *Search > Go To* and the hidden *Go To Line* (E07-S6-T1).
//!
//! `bookmarks.rs` holds the Search menu's bookmarks 0–9 and their gutter marks (E07-S6-T2).
//!
//! `layout.rs` holds the View menu's Side-by-Side and Top and Bottom layouts (E07-S9-T3).
//!
//! `load.rs` asks before the path edit or *Open File* loads another file over a pane's unsaved
//! edits (E07-S1-T9).

mod read_only;
pub mod session;

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Local};
pub mod aligned;
mod bookmarks;
mod edit;
mod edited;
mod font_size;
mod go_to;
mod layout;
mod load;
mod open;
mod save;

use aligned::Aligned;
use egui::accesskit::{Live, Role};
use egui::{Id, Ui, UiBuilder};
use otd_core::i18n::{file_size, fmt, t};
use otd_format::{Kind, Registry};
use otd_launch::Completion;
use otd_text_compare::{Encoding, LineEndings, Pane, ReadOnly};
use otd_ui::{Browse, PathEdit, Session, SessionType, View};

pub use bookmarks::{BOOKMARK, CLEAR_BOOKMARKS, GO_TO_BOOKMARK, TOGGLE_BOOKMARK};
pub use edit::{COPY, CUT, DELETE, INSERT_TEXT, PASTE, REDO, SELECT_ALL, UNDO};
pub use edited::{NEXT_EDIT, PREVIOUS_EDIT};
pub use font_size::{DECREASE_FONT_SIZE, INCREASE_FONT_SIZE, RESET_FONT_SIZE};
pub use go_to::{COLUMN, GO_TO, GO_TO_LINE, LINE, NOT_A_NUMBER};
pub use layout::{Layout, SIDE_BY_SIDE, TOP_AND_BOTTOM};
pub use open::{OPEN_FILE, OPEN_FILE_ON_FILE_SYSTEM, OpenFiles, commands};

pub use read_only::{COMPARISON_CANCELLED, COMPARISON_FAILED, ENCODING_LOSSY, FILE_READ_ONLY, LOCATION};
pub use save::{SAVE_FAILED, SAVE_FILE, SAVE_FILE_AS, SAVE_FILE_ON_FILE_SYSTEM_AS, UNSAVED_CHANGES};
pub use session::{Info, NO_RESULT, RECOMPARE_FILES, RELOAD_FILES, SWAP_SIDES, TEXT_COMPARE_INFO};

/// The session type's and the view type's id.
pub const TEXT_COMPARE: &str = "text-compare";
/// The view's name: Home's button, *New Session ▸ Text Compare* and the tab's title
/// (docs/feature-names.md).
pub const NAME: &str = "Text Compare";
pub const LEFT: &str = "Left";
pub const RIGHT: &str = "Right";
const SIDES: [&str; 2] = [LEFT, RIGHT];
/// Each pane's path edit field, whose recent paths it keeps.
pub const FIELDS: [&str; 2] = ["text-compare.left", "text-compare.right"];

// The file info header's labels and values: English keys, drawn through `t`.
pub const PATH: &str = "Path";
pub const SIZE: &str = "Size";
pub const MODIFIED: &str = "Modified";
pub const FORMAT: &str = "Format";
pub const ENCODING: &str = "Encoding";
pub const LINE_ENDINGS: &str = "Line endings";
pub const UNKNOWN: &str = "Unknown";
/// One header field; its accessible name starts with its label.
const FIELD: &str = "{label}: {value}";

pub const SESSION_TYPE: SessionType = SessionType { id: TEXT_COMPARE, name: NAME, kinds: &[Kind::Text], sides: &[LEFT, RIGHT], new: new_view };

fn new_view() -> Box<dyn View> {
    Box::new(TextCompareView::new())
}

/// Two text files side by side.
pub struct TextCompareView {
    session: Session,
    // ponytail: Text alone, so every file gets the catch-all format; the app-wide registry with
    // user formats comes with the File Formats manager (E10-S1).
    formats: Registry,
    /// Per side, the spec its pane was loaded from, so a change reloads that pane; `None` reloads
    /// it even when the spec is the same, so a file that failed can be tried again.
    loaded: Vec<Option<String>>,
    /// Per side: nothing chosen, the loaded file, or why it couldn't load. A pane that couldn't
    /// load has no document, so it stays read-only until a file loads.
    panes: Vec<Option<Result<Pane, String>>>,
    /// Per side, the path edit's text: typing changes it, and the side's spec takes it when the edit
    /// loses focus, so a half-typed path isn't loaded.
    edits: [String; 2],
    /// Per side, the spec `edits` last took or gave, so a spec changed elsewhere (the Session
    /// Settings dialog) shows in the edit.
    synced: [String; 2],
    /// The chosen pane, which *Open File* loads into: the side last focused or clicked.
    chosen: usize,
    /// The side whose file picker opens on the next draw.
    browse: Option<usize>,
    /// Reports this view's close to a launcher waiting for it (the command line).
    done: Option<Completion>,
    /// Why the last comparison left both panes read-only, if it did.
    comparison: Option<ReadOnly>,
    /// The panes' rows, aligned, and the comparison behind them.
    aligned: Aligned,
    /// Insert (`true`) or overwrite, which both panes share.
    insert: bool,
    /// How long the last load and comparison of both files took.
    load_time: Option<Duration>,
    /// The open Text Compare Info dialog.
    info: Option<session::InfoDialog>,
    /// The widget that takes focus on the next draw, back from the closed Info dialog.
    refocus: Option<Id>,
    /// Text *Copy* or *Cut* put on the clipboard on the next draw, which has the context.
    clipboard: Option<String>,
    /// Whether *Paste* asked for the clipboard's text, asked for once the chosen pane has focus.
    paste: bool,
    /// The side whose editor takes focus on the next draw, after an Edit menu command.
    focus: Option<usize>,
    /// The side whose editor last had focus, whose caret the status bar shows.
    cursor: Option<usize>,
    /// Whether a text field other than the editors (a path edit) had focus at the last draw. Edit >
    /// Delete has no shortcut for the menu to hand that field, so it is disabled then.
    field_focus: bool,
    /// The input method's composition, drawn at the caret until it commits. It is never part of
    /// the pane, so it doesn't enter undo, the comparison or a save.
    preedit: String,
    /// The editors' input-method context, new each time one gains focus (`IMEOutput`).
    ime: u64,
    /// The side whose *Save File As* picker opens on the next draw, and whether it has been asked.
    save_as: Option<(usize, bool)>,
    /// Per side, why its last save failed, shown in the pane until it saves or loads again.
    save_errors: [Option<String>; 2],
    /// Points the editor text is drawn above (or below) its configured size, in both panes.
    font_step: i8,
    /// The open Go To dialog (E07-S6-T1).
    go_to: Option<go_to::GoToDialog>,
    /// What the last bookmark toggle or clear did, announced (E07-S6-T2).
    bookmark_said: Option<String>,
    /// Side by side or over and under (`layout.rs`).
    layout: Layout,
    /// Each pane header's height on the last frame, which places a stacked top body.
    headers: [f32; 2],
    /// The side and spec to load once the unsaved-changes question is answered (E07-S1-T9).
    pending: Option<(usize, String)>,
}

impl Drop for TextCompareView {
    fn drop(&mut self) {
        if let Some(done) = self.done.take() {
            done.finish(0);
        }
    }
}

impl Default for TextCompareView {
    fn default() -> Self {
        Self::new()
    }
}

impl TextCompareView {
    pub fn new() -> Self {
        let mut formats = Registry::default();
        formats.register_kind(Kind::Text);
        let mut session = Session::new(TEXT_COMPARE);
        session.specs = vec![String::new(); SIDES.len()];
        Self {
            session,
            formats,
            loaded: Vec::new(),
            panes: Vec::new(),
            edits: Default::default(),
            synced: Default::default(),
            chosen: 0,
            browse: None,
            done: None,
            comparison: None,
            aligned: Aligned::new(None, None),
            insert: true,
            load_time: None,
            info: None,
            refocus: None,
            clipboard: None,
            paste: false,
            focus: None,
            cursor: None,
            field_focus: false,
            preedit: String::new(),
            ime: 0,
            save_as: None,
            save_errors: Default::default(),
            font_step: 0,
            go_to: None,
            bookmark_said: None,
            layout: Layout::default(),
            headers: [0.0; 2],
            pending: None,
        }
    }

    /// Why the file of side `side` (0 is left) couldn't load, if it couldn't.
    pub fn error(&self, side: usize) -> Option<&str> {
        self.panes.get(side)?.as_ref()?.as_ref().err().map(String::as_str)
    }

    /// The chosen pane (0 is left): the side last focused or clicked.
    pub fn chosen(&self) -> usize {
        self.chosen
    }

    /// *Open File*: loads `path` into the pane of `side` (the chosen pane when `None`), or with no
    /// path opens that pane's file picker. Returns `true` when the pane has unsaved edits, so the
    /// load waits for [`otd_ui::Shell::save_first`].
    fn open_file(&mut self, side: Option<usize>, path: Option<&str>) -> bool {
        let side = side.unwrap_or(self.chosen).min(SIDES.len() - 1);
        self.chosen = side;
        match path {
            Some(path) => self.load(side, path.to_owned()),
            None => {
                self.browse = Some(side);
                false
            }
        }
    }

    /// Side `side`'s path edit, named by its caption `caption`.
    fn path_edit(&mut self, ui: &mut Ui, side: usize, caption: Id) {
        let field = FIELDS[side];
        let Some(spec) = self.session.specs.get_mut(side) else { return };
        if self.synced[side] != *spec {
            self.edits[side].clone_from(spec);
            self.synced[side].clone_from(spec);
        }
        let browse = self.browse.take_if(|s| *s == side).is_some();
        let (response, used) = PathEdit::new(field, &mut self.edits[side], Browse::File).browse_now(browse).show_used(ui);
        let response = response.labelled_by(caption);
        if response.has_focus() {
            self.chosen = side;
        }
        if used {
            // The edit shows the loaded file until the load happens, so Cancel leaves it there.
            let path = std::mem::replace(&mut self.edits[side], spec.clone());
            if self.load(side, path) {
                // A typed path left the edit; it keeps focus so Cancel gives it back there.
                if response.lost_focus() {
                    response.request_focus();
                }
                otd_ui::save_first_later(ui.ctx(), load::load_pending);
            }
            ui.ctx().request_repaint();
        }
    }

    /// Loads side `side` again on the next draw, even if its spec is unchanged.
    fn retry(&mut self, side: usize) {
        if let Some(loaded) = self.loaded.get_mut(side) {
            *loaded = None;
        }
    }

    /// The pane of side `side` (0 is left), once loaded.
    pub fn pane(&self, side: usize) -> Option<&Pane> {
        self.panes.get(side)?.as_ref()?.as_ref().ok()
    }

    /// Loads each side whose spec changed or that [`Self::retry`] marked, leaving the other alone.
    fn reload_if_changed(&mut self) {
        let started = Instant::now();
        let specs = &self.session.specs;
        self.loaded.resize(specs.len(), None);
        self.panes.resize_with(specs.len(), || None);
        let mut changed = false;
        for (((spec, loaded), pane), error) in specs.iter().zip(&mut self.loaded).zip(&mut self.panes).zip(&mut self.save_errors) {
            if loaded.as_ref() != Some(spec) {
                *error = None;
                *loaded = Some(spec.clone());
                self.comparison = None;
                *pane = (!spec.trim().is_empty()).then(|| otd_text_compare::open(spec, &self.formats).map_err(|e| e.message));
                changed = true;
            }
        }
        if changed {
            self.compare(started);
        }
    }

    /// Compares the panes as they stand. The load time, counted from `started`, is set when both
    /// panes are loaded, and cleared when there was nothing to compare.
    fn compare(&mut self, started: Instant) {
        self.aligned = Aligned::new(self.pane(0).map(|pane| pane.lines.as_slice()), self.pane(1).map(|pane| pane.lines.as_slice()));
        self.load_time = self.aligned.result().map(|_| started.elapsed());
    }
}

impl View for TextCompareView {
    fn title(&self) -> String {
        t(NAME).to_owned()
    }

    fn view_type(&self) -> &'static str {
        TEXT_COMPARE
    }

    fn session(&mut self) -> Option<&mut Session> {
        Some(&mut self.session)
    }

    fn status(&self) -> otd_ui::Status {
        self.status_fields()
    }

    fn toggle_insert(&mut self) {
        self.insert = !self.insert;
    }

    fn unsaved(&self) -> bool {
        (0..SIDES.len()).any(|side| TextCompareView::unsaved(self, side))
    }

    /// Saves each pane with unsaved edits, stopping at the first that fails.
    fn save(&mut self) -> Result<(), String> {
        for side in 0..SIDES.len() {
            if TextCompareView::unsaved(self, side) {
                self.save_file(Some(side))?;
            }
        }
        Ok(())
    }

    fn ui(&mut self, ui: &mut Ui) {
        self.reload_if_changed();
        // Each pane's header and body keep their ids in either layout (`layout.rs`), so switching
        // keeps their scroll offset, caret and focus. Side by side, both headers come before both
        // bodies; stacked, each header comes just before its body. Either way the Tab and
        // accessibility order follow the screen, top to bottom and left to right.
        let slots = layout::slots(self.layout, ui.available_rect_before_wrap(), ui.spacing().item_spacing.x);
        let stacked = self.layout == Layout::OverUnder;
        // Stacked, the top body is placed before the bottom header is measured, so its height
        // uses that header's height from the last frame.
        let mut headers = self.headers;
        let mut editing = false;
        for side in 0..SIDES.len() {
            headers[side] = self.header_ui(ui, side, slots[side]);
            if stacked {
                editing |= self.body_ui(ui, side, layout::bodies(self.layout, slots, headers, ui)[side]);
            }
        }
        if !stacked {
            for (side, rect) in layout::bodies(self.layout, slots, headers, ui).into_iter().enumerate() {
                editing |= self.body_ui(ui, side, rect);
            }
        }
        if headers != self.headers {
            self.headers = headers;
            ui.ctx().request_repaint();
        }
        self.field_focus = ui.ctx().text_edit_focused() && !editing;
        self.info_ui(ui);
        self.save_as_ui(ui);
        self.go_to_ui(ui);
        self.bookmark_ui(ui);
    }
}

impl TextCompareView {
    /// The header of `side` in `slot`: its caption, path edit and file facts. Returns its height.
    fn header_ui(&mut self, ui: &mut Ui, side: usize, slot: egui::Rect) -> f32 {
        let builder = UiBuilder::new().id_salt(("header", side)).max_rect(slot);
        ui.scope_builder(builder, |ui| {
            // The side's caption, so the two headers' identical facts read apart; it also names
            // the path edit.
            let unsaved = TextCompareView::unsaved(self, side);
            let caption = ui
                .horizontal(|ui| {
                    let caption = ui.strong(t(SIDES[side]));
                    save::mark(ui, unsaved);
                    caption
                })
                .inner;
            self.path_edit(ui, side, caption.id);
            pane(ui, self.panes.get(side).and_then(Option::as_ref), self.read_only(side), self.save_errors[side].as_deref());
            if clicked(ui) {
                self.chosen = side;
            }
        })
        .response
        .rect
        .height()
    }

    /// The body of `side` in `rect`, if it has a pane, named by its side's caption so a line and
    /// its counterpart read apart. Returns whether its editor has focus.
    fn body_ui(&mut self, ui: &mut Ui, side: usize, rect: egui::Rect) -> bool {
        if self.pane(side).is_none() {
            return false;
        }
        let body = UiBuilder::new().id_salt(side).max_rect(rect).accessibility_role(Role::Group).accessibility_label(t(SIDES[side]));
        ui.scope_builder(body, |ui| {
            font_size::apply(ui, self.font_step);
            let editing = self.body(ui, side);
            if clicked(ui) {
                self.chosen = side;
            }
            editing
        })
        .inner
    }
}

/// Whether the primary button was pressed inside what `ui` has drawn so far.
fn clicked(ui: &Ui) -> bool {
    ui.input(|i| i.pointer.primary_pressed()) && ui.rect_contains_pointer(ui.min_rect())
}

fn pane(ui: &mut Ui, pane: Option<&Result<Pane, String>>, read_only: Option<ReadOnly>, save_error: Option<&str>) {
    match pane {
        None => {}
        Some(Err(message)) => {
            let label = ui.label(message);
            // Announced: it comes from the load, away from focus.
            ui.ctx().accesskit_node_builder(label.id, |node| node.set_live(Live::Polite));
        }
        Some(Ok(pane)) => {
            file_info_header(ui, pane);
            read_only::show(ui, read_only);
            save::error(ui, save_error);
            ui.separator();
        }
    }
}

/// A pane's file info header: one labelled line per fact.
pub fn file_info_header(ui: &mut Ui, pane: &Pane) {
    let modified = pane.modified.and_then(local_time).unwrap_or_else(|| t(UNKNOWN).to_owned());
    let fields = [
        (PATH, pane.path.clone()),
        (SIZE, file_size(pane.size)),
        (MODIFIED, modified),
        (FORMAT, t(&pane.format).to_owned()),
        (ENCODING, encoding_name(pane.encoding)),
        (LINE_ENDINGS, t(endings_name(pane.endings)).to_owned()),
    ];
    for (label, value) in fields {
        ui.label(fmt(t(FIELD), &[("label", t(label)), ("value", &value)]));
    }
}

/// `time` in the local time zone, to the second; `None` outside the dates chrono holds. (chrono's
/// `From<SystemTime>` panics there, and a file system or source can report any time.)
fn local_time(time: SystemTime) -> Option<String> {
    let seconds = match time.duration_since(UNIX_EPOCH) {
        Ok(after) => i64::try_from(after.as_secs()).ok()?,
        Err(before) => i64::try_from(before.duration().as_secs()).ok()?.checked_neg()?,
    };
    Some(DateTime::from_timestamp(seconds, 0)?.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S").to_string())
}

fn encoding_name(encoding: Encoding) -> String {
    let (name, bom) = match encoding {
        Encoding::Utf8 { bom } => ("UTF-8", bom),
        Encoding::Utf16Le { bom } => ("UTF-16 LE", bom),
        Encoding::Utf16Be { bom } => ("UTF-16 BE", bom),
        Encoding::CodePage(page) => return fmt(t("Windows-{page}"), &[("page", &page.to_string())]),
    };
    if bom { fmt(t("{encoding} with BOM"), &[("encoding", name)]) } else { name.to_owned() }
}

fn endings_name(endings: LineEndings) -> &'static str {
    match endings {
        LineEndings::None => "None",
        LineEndings::Windows => "Windows (CRLF)",
        LineEndings::Mac => "Mac (CR)",
        LineEndings::Unix => "Unix (LF)",
        LineEndings::Mixed => "Mixed",
    }
}
