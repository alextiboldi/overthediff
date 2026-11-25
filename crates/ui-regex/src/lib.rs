//! The regex input aid: two menu buttons that sit beside any field taking a regular expression.
//! *Regex elements* inserts one construct of the A.2 syntax (`otd-regex`), *Regex examples* one
//! whole sample pattern. Either goes in at the field's cursor, replacing a selection, and focus
//! goes back to the field with the cursor after what was inserted.
//!
//! ponytail: captions are English literals until E01-S1-T13's translation lookup exists; they
//! are the English keys it will look up.

use egui::text::{CCursor, CCursorRange};
use egui::{Id, TextBuffer as _, TextEdit, Ui};

/// The visible label, and accessible name, of the element menu's button.
pub const ELEMENTS_LABEL: &str = "Regex elements";
/// The visible label, and accessible name, of the example menu's button.
pub const EXAMPLES_LABEL: &str = "Regex examples";

/// One menu entry: an English caption and the text it inserts.
pub type Entry = (&'static str, &'static str);

/// The element menu, in groups. Each entry is valid inside a pattern, so a template like `{2,4}`
/// carries sample numbers rather than placeholders.
pub const ELEMENTS: &[&[Entry]] = &[
    &[
        ("Any character", "."),
        ("Word character", r"\w"),
        ("Not a word character", r"\W"),
        ("Digit", r"\d"),
        ("Not a digit", r"\D"),
        ("Space or tab", r"\s"),
        ("Not a space or tab", r"\S"),
        ("Tab", r"\t"),
        ("Character by hex code", r"\x20"),
        ("Literal period", r"\."),
    ],
    &[("One of a set", "[abc]"), ("One in a range", "[a-z]"), ("None of a set", "[^abc]")],
    &[("Start of line", "^"), ("End of line", "$")],
    &[
        ("Zero or more", "*"),
        ("One or more", "+"),
        ("Optional", "?"),
        ("Exactly n times", "{2}"),
        ("At least n times", "{2,}"),
        ("Between n and m times", "{2,4}"),
        ("Zero or more, fewest first", "*?"),
        ("One or more, fewest first", "+?"),
        ("Optional, skipped first", "??"),
    ],
    &[("Group", "()"), ("Or", "|"), ("Same text as group 1", r"\1"), ("Ignore case from here", "(?i)"), ("Match case from here", "(?-i)")],
];

/// The example menu: whole patterns, each with what it finds.
pub const EXAMPLES: &[Entry] = &[
    ("Lines that start with abc", "^abc"),
    ("Lines that end with abc", "abc$"),
    ("Lines that are just abc", "^abc$"),
    ("Lines that start with abc after any indent", r"^\s*abc"),
    ("Lines that hold only end, with or without a semicolon", r"^\s*end;?\s*$"),
    ("abc or def", "abc|def"),
    ("abd or acd", "a(b|c)d"),
    ("abdb or acdc", r"a(b|c)d\1"),
];

/// Draws both menu buttons, left to right, for the `TextEdit` whose id is `field` and whose text
/// is `text`. Returns whether it changed `text`.
pub fn regex_aid(ui: &mut Ui, field: Id, text: &mut String) -> bool {
    let mut chosen = None;
    ui.horizontal(|ui| {
        ui.menu_button(ELEMENTS_LABEL, |ui| {
            for (n, group) in ELEMENTS.iter().enumerate() {
                if n > 0 {
                    ui.separator();
                }
                entries(ui, group, &mut chosen);
            }
        });
        ui.menu_button(EXAMPLES_LABEL, |ui| entries(ui, EXAMPLES, &mut chosen));
    });
    let Some(snippet) = chosen else { return false };
    insert(ui, field, text, snippet);
    true
}

fn entries(ui: &mut Ui, entries: &[Entry], chosen: &mut Option<&'static str>) {
    for &(caption, snippet) in entries {
        if ui.button(format!("{caption}   {snippet}")).clicked() {
            *chosen = Some(snippet);
        }
    }
}

/// Puts `snippet` in place of the field's selection, or at its cursor, and focuses the field with
/// the cursor after it. A field that never had a cursor gets the snippet at its end.
fn insert(ui: &Ui, field: Id, text: &mut String, snippet: &str) {
    let mut state = TextEdit::load_state(ui.ctx(), field).unwrap_or_default();
    let range = state.cursor.char_range().unwrap_or_else(|| CCursorRange::one(CCursor::end_of_str(text)));
    // ponytail: egui's own edit primitives, so char indices (not bytes) and a cursor past a text
    // the host shortened both stay safe.
    let mut at = text.delete_selected(&range);
    text.insert_text_at(&mut at, snippet, usize::MAX);
    state.cursor.set_char_range(Some(CCursorRange::one(at)));
    TextEdit::store_state(ui.ctx(), field, state);
    ui.memory_mut(|m| m.request_focus(field));
}
