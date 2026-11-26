//! The encoding override per side on the Session Settings dialog's Format tab (E07-S10-T2). Each
//! side offers **None**, the default, which detects the encoding as the file loads, and every
//! encoding the decoder has: UTF-8, UTF-16 LE and BE, and the Windows code pages. A side decodes
//! in its override instead, and the file info header names it.
//!
//! When a side's override changes, that side is read again in it, unless it has unsaved edits:
//! those stay until *Reload Files*. Prompting to save them first is E07-S10-T3's.

use otd_core::session::{Setting, Value};
use otd_text_compare::Encoding;

use crate::{TEXT_COMPARE, TextCompareView};

/// The Session Settings tab, in English, which the file-format pickers (E07-S10-T1) share.
const TAB: &str = "Format";
/// The override's default: no override, so the encoding is detected.
// ponytail: no format fixes an encoding yet (`otd_format::Format` has no such property), so None
// means detection; a format's own encoding comes with the File Formats manager (E10-S1).
pub const NONE: &str = "None";
/// Each side's override, left then right: its setting id and its label.
pub const ENCODING_OVERRIDES: [(&str, &str); 2] = [("left-encoding", "Left encoding"), ("right-encoding", "Right encoding")];
/// The choices, spelled as the file info header names each encoding.
const CHOICES: &[&str] = &[
    NONE,
    "UTF-8",
    "UTF-16 LE",
    "UTF-16 BE",
    "Windows-874",
    "Windows-1250",
    "Windows-1251",
    "Windows-1252",
    "Windows-1253",
    "Windows-1254",
    "Windows-1255",
    "Windows-1256",
    "Windows-1257",
    "Windows-1258",
    "Windows-932",
    "Windows-936",
    "Windows-949",
    "Windows-950",
];

/// Each side's encoding override, None by default.
pub fn session_settings() -> Vec<Setting> {
    ENCODING_OVERRIDES
        .iter()
        .map(|&(id, label)| Setting { session_type: TEXT_COMPARE, tab: TAB, id, label, default: Value::Choice(NONE.to_owned()), choices: CHOICES })
        .collect()
}

/// The encoding a choice names; `None` for [`NONE`] or anything not offered.
fn parse(choice: &str) -> Option<Encoding> {
    if !CHOICES.contains(&choice) {
        return None;
    }
    match choice {
        "UTF-8" => Some(Encoding::Utf8 { bom: false }),
        "UTF-16 LE" => Some(Encoding::Utf16Le { bom: false }),
        "UTF-16 BE" => Some(Encoding::Utf16Be { bom: false }),
        _ => choice.strip_prefix("Windows-")?.parse().ok().map(Encoding::CodePage),
    }
}

impl TextCompareView {
    /// The encoding side `side` (0 is left) is decoded in, when its override names one.
    pub fn encoding_override(&self, side: usize) -> Option<Encoding> {
        match self.session.rule(TAB, ENCODING_OVERRIDES.get(side)?.0) {
            Some(Value::Choice(choice)) => parse(choice),
            _ => None,
        }
    }

    /// Reads again each side whose override changed since it was read, unless it has unsaved edits.
    pub(crate) fn retry_changed_overrides(&mut self) {
        for side in 0..self.decoded_as.len() {
            if self.decoded_as[side] != self.encoding_override(side) && !self.unsaved(side) {
                self.retry(side);
            }
        }
    }

    /// Swap Sides: each side's override goes with its file, as the override it was read in does.
    pub(crate) fn swap_overrides(&mut self) {
        self.decoded_as.swap(0, 1);
        if let Some(rules) = self.session.rules.get_mut(TAB) {
            let [left, right] = ENCODING_OVERRIDES.map(|(id, _)| rules.remove(id));
            for ((id, _), value) in ENCODING_OVERRIDES.iter().zip([right, left]) {
                if let Some(value) = value {
                    rules.insert((*id).to_owned(), value);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    /// A view of a left file in Windows-1252 that reads as UTF-8 and a plain right one, loaded.
    fn view(folder: &tempfile::TempDir) -> TextCompareView {
        let specs = [("left.txt", &b"Caf\xC3\xA9\n"[..]), ("right.txt", b"other\n")].map(|(name, bytes)| {
            let path = folder.path().join(name);
            fs::write(&path, bytes).unwrap();
            path.display().to_string()
        });
        let mut view = TextCompareView::new();
        view.session.specs = specs.to_vec();
        view.reload_if_changed();
        view
    }

    fn choose(view: &mut TextCompareView, side: usize, choice: &str) {
        view.session.rules.entry(TAB.to_owned()).or_default().insert(ENCODING_OVERRIDES[side].0.to_owned(), Value::Choice(choice.to_owned()));
        view.reload_if_changed();
    }

    #[test]
    fn a_pane_with_unsaved_edits_keeps_them_when_its_override_changes() {
        let folder = tempfile::tempdir().unwrap();
        let mut view = view(&folder);
        if let Some(Some(Ok(pane))) = view.panes.get_mut(0) {
            pane.lines[0].text.push_str(" edited");
            pane.unsaved = true;
        }
        choose(&mut view, 0, "Windows-1252");
        let pane = view.pane(0).unwrap();
        assert_eq!(pane.lines[0].text, "Café edited", "the edit stays");
        assert_ne!(pane.encoding, Encoding::CodePage(1252), "and the header still names the encoding it was read in");
    }

    #[test]
    fn swapping_sides_takes_each_override_with_its_file() {
        let folder = tempfile::tempdir().unwrap();
        let mut view = view(&folder);
        choose(&mut view, 0, "Windows-1252");
        let read = view.pane(0).unwrap().lines.clone();
        view.swap_sides();
        view.reload_if_changed();
        assert_eq!(view.encoding_override(1), Some(Encoding::CodePage(1252)));
        assert_eq!(view.encoding_override(0), None);
        assert_eq!(view.pane(1).unwrap().lines, read, "the file still reads in its override");
    }

    #[test]
    fn every_choice_but_none_names_an_encoding_the_header_spells_the_same_way() {
        for &choice in &CHOICES[1..] {
            let encoding = parse(choice).unwrap_or_else(|| panic!("{choice} parses"));
            assert_eq!(crate::encoding_name(encoding), choice);
        }
        assert_eq!(parse(NONE), None);
    }
}
