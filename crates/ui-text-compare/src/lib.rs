//! The Text Compare view (story E07-S1): a left and a right pane, each with its file info header
//! (path, size, modified time, file format, encoding and line-ending style) above its text. The
//! panes load their files through `otd-text-compare` from the session's inputs, one per side,
//! whenever those change.
//!
//! [`SESSION_TYPE`] is the session type that opens it; the app registers it in
//! `apps/otd/src/session_types.rs` (E07-S1-T2, after its naming check).
//!
//! ponytail: the text is a plain list of lines until E07-S1-T4 draws the aligned comparison.

use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Local};
use egui::{RichText, ScrollArea, Ui};
use otd_core::i18n::{file_size, fmt, t};
use otd_format::{Kind, Registry};
use otd_text_compare::{Encoding, LineEndings, Pane};
use otd_ui::{Session, SessionType, View};

/// The session type's and the view type's id.
pub const TEXT_COMPARE: &str = "text-compare";
/// The view's working name (a backlog working name until the naming check).
pub const NAME: &str = "Text Compare";
pub const LEFT: &str = "Left";
pub const RIGHT: &str = "Right";

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
    /// The specs the panes were loaded from, so a change reloads them.
    loaded: Vec<String>,
    /// Per side: nothing chosen, the loaded file, or why it couldn't load.
    panes: Vec<Option<Result<Pane, String>>>,
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
        Self { session: Session::new(TEXT_COMPARE), formats, loaded: Vec::new(), panes: Vec::new() }
    }

    /// The pane of side `side` (0 is left), once loaded.
    pub fn pane(&self, side: usize) -> Option<&Pane> {
        self.panes.get(side)?.as_ref()?.as_ref().ok()
    }

    fn reload_if_changed(&mut self) {
        if self.loaded == self.session.specs {
            return;
        }
        self.loaded.clone_from(&self.session.specs);
        let formats = &self.formats;
        self.panes =
            self.loaded.iter().map(|spec| (!spec.trim().is_empty()).then(|| otd_text_compare::open(spec, formats).map_err(|e| e.message))).collect();
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

    fn ui(&mut self, ui: &mut Ui) {
        self.reload_if_changed();
        ui.columns(2, |columns| {
            for (side, column) in columns.iter_mut().enumerate() {
                column.push_id(side, |ui| {
                    // The side's caption, so the two headers' identical facts read apart.
                    ui.strong(t(if side == 0 { LEFT } else { RIGHT }));
                    pane(ui, self.panes.get(side).and_then(Option::as_ref));
                });
            }
        });
    }
}

fn pane(ui: &mut Ui, pane: Option<&Result<Pane, String>>) {
    match pane {
        None => {}
        Some(Err(message)) => {
            ui.label(message);
        }
        Some(Ok(pane)) => {
            file_info_header(ui, pane);
            ui.separator();
            lines(ui, pane);
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

fn lines(ui: &mut Ui, pane: &Pane) {
    ScrollArea::both().auto_shrink(false).show_rows(ui, ui.text_style_height(&egui::TextStyle::Monospace), pane.lines.len(), |ui, rows| {
        for line in pane.lines.get(rows).unwrap_or_default() {
            ui.label(RichText::new(&line.text).monospace());
        }
    });
}
