//! How one side of a row looks (E08-S1-T4): its status colours, the colour spot left of its name,
//! its glyphs and the status text that says the same thing without colour.
//!
//! The default colours (Appendix B): gray is unknown or older, black is same, purple is orphan and
//! red is newer or different. A folder pair's look comes from its contents ([`Row::cues`]), never
//! from its own size and time, and it can show several cues at once: the name takes the first
//! colour and the spot shows every one, side by side.
//!
//! *Unspecified*, decided here:
//! - The name's colour, when a folder holds several cues, is the first of red, purple, gray, then
//!   black.
//! - A folder holding an item newer on one side is newer on that side and older on the other. A
//!   folder holding orphans, differences, unread or uncompared items shows that on both sides,
//!   since the pair as a whole differs.
//! - An item that can't be read, or a folder that can't be opened, is red and carries the red X;
//!   its partner, which nothing could be compared with, is gray. A folder holding such an item is
//!   red.
//! - A folder pair whose own listings aren't done is gray, as is anything not yet compared.

use otd_core::i18n::{fmt, t};
use otd_folder_engine::{Cues, Row, ScanState, Status, TreeSide};
use otd_source::{ErrorKind, Item, ItemKind, SourceError};
use otd_ui::colour::{BACKGROUND, ColourRole, Contrast, Rgba, SELECTION, WINDOW};

const fn role(id: &'static str, light: Rgba, dark: Rgba) -> ColourRole {
    ColourRole { id, light, dark }
}

/// Black: same.
pub const SAME: ColourRole = role("folder.status.same", Rgba::rgb(0x1F, 0x1F, 0x1F), Rgba::rgb(0xE4, 0xE4, 0xE4));
/// Gray: unknown, or the older side.
pub const UNKNOWN: ColourRole = role("folder.status.unknown", Rgba::rgb(0x6B, 0x6B, 0x6B), Rgba::rgb(0xA0, 0xA0, 0xA0));
/// Purple: an orphan.
pub const ORPHAN: ColourRole = role("folder.status.orphan", Rgba::rgb(0x7B, 0x1F, 0xA2), Rgba::rgb(0xCE, 0x93, 0xD8));
/// Red: the newer side, or different.
pub const DIFFERENT: ColourRole = role("folder.status.different", Rgba::rgb(0xB7, 0x1C, 0x1C), Rgba::rgb(0xFF, 0x6E, 0x6E));
/// The red X on a folder that couldn't be opened or an item that couldn't be read.
pub const ERROR_GLYPH: ColourRole = role("folder.glyph.error", Rgba::rgb(0xB7, 0x1C, 0x1C), Rgba::rgb(0xFF, 0x6E, 0x6E));

/// Teal: hidden by a filter on this side, shown because its partner isn't. Drawn once filters
/// (E08-S2) are in use; the Legend shows it already.
pub const FILTERED: ColourRole = role("folder.status.filtered", Rgba::rgb(0x00, 0x69, 0x5C), Rgba::rgb(0x4D, 0xB6, 0xAC));
/// Yellow: a folder not scanned yet. Drawn once scanning can be turned off (E08-S11); the Legend
/// shows it already.
pub const UNSCANNED: ColourRole = role("folder.status.unscanned", Rgba::rgb(0x80, 0x66, 0x00), Rgba::rgb(0xFF, 0xD5, 0x4F));

/// The Lucide glyphs after a name: a link's arrow and the red X of an item that can't be read.
pub const LINK_ICON: &str = "arrow-up-right";
pub const ERROR_ICON: &str = "x";

/// Folder Compare's colour roles, for the app's list.
pub const COLOUR_ROLES: [ColourRole; 7] = [SAME, UNKNOWN, ORPHAN, DIFFERENT, ERROR_GLYPH, FILTERED, UNSCANNED];

/// Names are drawn on the view's background (text, so 4.5:1, which also covers the spot and glyph
/// there); spots and glyphs also on a selected row's background, and in the Legend on a dialog's.
pub const CONTRAST: [Contrast; 21] = [
    Contrast::text(&SAME, &BACKGROUND),
    Contrast::text(&UNKNOWN, &BACKGROUND),
    Contrast::text(&ORPHAN, &BACKGROUND),
    Contrast::text(&DIFFERENT, &BACKGROUND),
    Contrast::control(&ERROR_GLYPH, &BACKGROUND),
    Contrast::control(&SAME, &SELECTION),
    Contrast::control(&UNKNOWN, &SELECTION),
    Contrast::control(&ORPHAN, &SELECTION),
    Contrast::control(&DIFFERENT, &SELECTION),
    Contrast::control(&ERROR_GLYPH, &SELECTION),
    Contrast::text(&FILTERED, &BACKGROUND),
    Contrast::control(&FILTERED, &SELECTION),
    Contrast::control(&UNSCANNED, &BACKGROUND),
    Contrast::control(&UNSCANNED, &SELECTION),
    Contrast::control(&SAME, &WINDOW),
    Contrast::control(&UNKNOWN, &WINDOW),
    Contrast::control(&ORPHAN, &WINDOW),
    Contrast::control(&DIFFERENT, &WINDOW),
    Contrast::control(&ERROR_GLYPH, &WINDOW),
    Contrast::control(&FILTERED, &WINDOW),
    Contrast::control(&UNSCANNED, &WINDOW),
];

// English keys, drawn only through `t`.
pub const IS_SAME: &str = "Same";
pub const IS_DIFFERENT: &str = "Different";
pub const IS_NEWER: &str = "Newer";
pub const IS_OLDER: &str = "Older";
pub const IS_ORPHAN: &str = "Orphan";
pub const IS_UNKNOWN: &str = "Not compared yet";
pub const CANT_READ: &str = "Can't be read: {error}";
pub const HOLDS_DIFFERENT: &str = "Contains differences";
pub const HOLDS_NEWER: &str = "Contains newer items";
pub const HOLDS_OLDER: &str = "Contains older items";
pub const HOLDS_ORPHANS: &str = "Contains orphans";
pub const HOLDS_UNKNOWN: &str = "Contains items not compared yet";
pub const HOLDS_ERRORS: &str = "Contains items that can't be read";
pub const LINK: &str = "Link";
/// Between the parts of a status that says several things, so a language can use its own list
/// punctuation.
pub const SEPARATOR: &str = ", ";

/// One side's status, as far as its colour and text go, in the order the name takes its colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Cue {
    Error,
    Different,
    Newer,
    Orphan,
    Older,
    Unknown,
    Same,
}

impl Cue {
    pub fn colour(self) -> &'static ColourRole {
        match self {
            Cue::Error | Cue::Different | Cue::Newer => &DIFFERENT,
            Cue::Orphan => &ORPHAN,
            Cue::Older | Cue::Unknown => &UNKNOWN,
            Cue::Same => &SAME,
        }
    }

    /// Its English key for an item of its own, and for a folder holding it.
    fn text(self) -> (&'static str, &'static str) {
        match self {
            Cue::Error => (CANT_READ, HOLDS_ERRORS),
            Cue::Different => (IS_DIFFERENT, HOLDS_DIFFERENT),
            Cue::Newer => (IS_NEWER, HOLDS_NEWER),
            Cue::Orphan => (IS_ORPHAN, HOLDS_ORPHANS),
            Cue::Older => (IS_OLDER, HOLDS_OLDER),
            Cue::Unknown => (IS_UNKNOWN, HOLDS_UNKNOWN),
            Cue::Same => (IS_SAME, IS_SAME),
        }
    }
}

/// How one side's item looks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Look {
    /// One for an item; for a folder pair, each cue its contents hold, first one first.
    pub cues: Vec<Cue>,
    /// The cues come from a folder's contents rather than the item itself.
    pub contents: bool,
    /// A symbolic link or junction: the arrow glyph.
    pub link: bool,
    /// Why the item couldn't be read, or the folder opened: the red X.
    pub error: Option<String>,
}

impl Look {
    /// The name's colour.
    pub fn colour(&self) -> &'static ColourRole {
        self.cues.first().map_or(&UNKNOWN, |cue| cue.colour())
    }

    /// The spot's colours, left to right: one per colour among the cues.
    pub fn spot(&self) -> Vec<&'static ColourRole> {
        let mut colours: Vec<&'static ColourRole> = Vec::new();
        for colour in self.cues.iter().map(|cue| cue.colour()) {
            if !colours.iter().any(|c| c.id == colour.id) {
                colours.push(colour);
            }
        }
        colours
    }

    /// The status in words, already looked up, for screen readers and the tooltip.
    pub fn description(&self) -> String {
        let mut parts: Vec<String> = self
            .cues
            .iter()
            .map(|&cue| {
                let (own, held) = cue.text();
                match (cue, &self.error) {
                    _ if self.contents => t(held).to_owned(),
                    (Cue::Error, Some(error)) => fmt(t(own), &[("error", error)]),
                    _ => t(own).to_owned(),
                }
            })
            .collect();
        if self.link {
            parts.push(t(LINK).to_owned());
        }
        parts.join(t(SEPARATOR))
    }
}

/// How `side`'s item of `row` looks, or `None` for a gap.
pub fn look(row: &Row, side: TreeSide) -> Option<Look> {
    let (item, other) = match side {
        TreeSide::Left => (row.left.as_ref()?, row.right.as_ref()),
        TreeSide::Right => (row.right.as_ref()?, row.left.as_ref()),
    };
    let link = item.kind == ItemKind::Link || item.link_target.is_some();
    let mut error = item.error.as_ref().map(ToString::to_string);
    let folder = |item: Option<&Item>| item.is_some_and(|item| item.kind == ItemKind::Folder);
    // Until its own listings are done, a folder pair's contents are unknown, so it is too.
    let contents = folder(Some(item)) && folder(other) && row.status == Status::Unknown && row.scan == ScanState::Done;
    let cues = if contents {
        held(row.cues(), side)
    } else {
        vec![own(&row.status, side, error.is_some(), other.is_some_and(|other| other.error.is_some()))]
    };
    // A folder whose listing failed: the row keeps the error rather than the item.
    if let (None, [Cue::Error], Status::Error(listing)) = (&error, cues.as_slice(), &row.status) {
        error = Some(listing.to_string());
    }
    Some(Look { cues, contents, link, error })
}

/// An item's own cue from its row's status. `unread` and `other_unread` say which side's item has
/// an error of its own; when neither has, the row's error is a listing's, so both sides show it.
fn own(status: &Status, side: TreeSide, unread: bool, other_unread: bool) -> Cue {
    let newer = |left: bool| if left == (side == TreeSide::Left) { Cue::Newer } else { Cue::Older };
    match status {
        Status::Same => Cue::Same,
        Status::Different => Cue::Different,
        Status::LeftNewer => newer(true),
        Status::RightNewer => newer(false),
        Status::LeftOrphan | Status::RightOrphan => Cue::Orphan,
        Status::Unknown => Cue::Unknown,
        Status::Error(_) if unread || !other_unread => Cue::Error,
        Status::Error(_) => Cue::Unknown,
    }
}

/// A folder pair's cues on `side`, from its contents. Contents that are all the same, or none,
/// make it same.
fn held(cues: Cues, side: TreeSide) -> Vec<Cue> {
    let statuses = [
        // `Cues::contains` matches any error with any other.
        Status::Error(SourceError::new(ErrorKind::Io, "")),
        Status::Different,
        Status::LeftNewer,
        Status::RightNewer,
        Status::LeftOrphan,
        Status::RightOrphan,
        Status::Unknown,
    ];
    let mut found: Vec<Cue> = statuses.iter().filter(|status| cues.contains(status)).map(|status| own(status, side, true, false)).collect();
    found.sort();
    found.dedup();
    if found.is_empty() {
        found.push(Cue::Same);
    }
    found
}
