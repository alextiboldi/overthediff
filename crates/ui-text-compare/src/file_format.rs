//! The file-format picker per side on the Session Settings dialog's Format tab (E07-S10-T1). Each
//! side's picker offers **Detected**, the default, which keeps the format the registry matched by
//! mask (or its fallback), and every registered format by name, including those with an empty mask
//! that never match automatically. The file info header shows the format in use.
//!
//! Choosing a format changes the name the header shows; it doesn't read the file again, so unsaved
//! edits stay. Reloading with the override, and its encoding, is E07-S10-T3's.

use otd_core::session::{Setting, Value};
use otd_format::{Kind, Registry};

use crate::{TEXT_COMPARE, TextCompareView};

/// The Session Settings tab, in English. The encoding override (E07-S10-T2) shares it.
pub const FORMAT_TAB: &str = "Format";
/// The picker's default choice: the format resolved by mask through the registry.
pub const DETECTED: &str = "Detected";
/// Each side's picker, left then right: its setting id and its label.
pub const FORMAT_PICKERS: [(&str, &str); 2] = [("left-format", "Left format"), ("right-format", "Right format")];

/// The registry the view resolves its files' formats in.
// ponytail: Text alone, so every file gets the catch-all format; the app-wide registry with user
// formats comes with the File Formats manager (E10-S1).
pub fn formats() -> Registry {
    let mut formats = Registry::default();
    formats.register_kind(Kind::Text);
    formats
}

/// Each side's format picker: Detected, then every format in `formats` top first, the catch-all last.
pub fn session_settings(formats: &Registry) -> Vec<Setting> {
    // ponytail: `Setting::choices` is static, so the names are leaked once, when the app starts. A
    // registry that changes while the app runs (E10-S1) needs owned choices in `Setting`.
    let names = formats.formats().iter().chain([formats.catch_all()]).map(|format| &*format.name.clone().leak());
    let choices: &'static [&'static str] = std::iter::once(DETECTED).chain(names).collect::<Vec<_>>().leak();
    FORMAT_PICKERS
        .iter()
        .map(|&(id, label)| Setting { session_type: TEXT_COMPARE, tab: FORMAT_TAB, id, label, default: Value::Choice(DETECTED.to_owned()), choices })
        .collect()
}

impl TextCompareView {
    /// A view that resolves its files' formats in `formats` instead of [`formats`].
    pub fn with_formats(formats: Registry) -> Self {
        let mut view = Self::new();
        view.formats = formats;
        view
    }

    /// The format side `side` (0 is left) is shown as, once loaded: the one its picker names, or
    /// the detected one when the picker says Detected or names no registered format.
    pub fn format(&self, side: usize) -> Option<&str> {
        let pane = self.pane(side)?;
        let chosen = match self.session.rule(FORMAT_TAB, FORMAT_PICKERS.get(side)?.0) {
            Some(Value::Choice(name)) => self.formats.by_name(name),
            _ => None,
        };
        Some(chosen.map_or(&pane.format, |format| &format.name))
    }
}
