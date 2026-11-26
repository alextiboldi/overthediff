//! Auto-saved sessions (E06-S1-T4): the recent unnamed sessions Home's Auto-saved branch lists,
//! newest first, kept in the `auto-saved-sessions` section as one array, so dropping an entry
//! drops it from the file too.
//!
//! The tweak *Max auto-saved sessions* caps the list: adding past it drops the oldest, and 0 turns
//! auto-save off. An entry holds a session's type, specs, description and settings, read back as
//! named sessions are (`sessions`). Closing the same comparison again (same type and specs) moves
//! it to the front rather than listing it twice, since the branch lists sessions by their specs.

use otd_core::options::{OptionSpec, Value};
use otd_core::session::{Session, Setting};
use serde::{Deserialize, Serialize};

use crate::Store;
use crate::sessions::{Tables, session, tables};

/// The section the list lives in.
pub const SECTION: &str = "auto-saved-sessions";
/// The cap's option id.
pub const MAX: &str = "tweaks.max-auto-saved-sessions";
// English keys, drawn only through `t`.
pub const TWEAKS: &str = "Tweaks";
pub const MAX_LABEL: &str = "Max auto-saved sessions";
/// The cap until the user changes it (*inferred*: the feature catalogue gives none).
const DEFAULT_MAX: f64 = 10.0;

/// The cap's option, for the app's option list.
pub fn options() -> Vec<OptionSpec> {
    vec![OptionSpec { page: &[TWEAKS], id: MAX, label: MAX_LABEL, default: Value::Number(DEFAULT_MAX), choices: &[] }]
}

/// The cap the option's value stands for: a whole number of sessions, at least 0.
pub fn cap(value: Option<&Value>) -> usize {
    match value {
        // `as` saturates, so a huge number is a huge cap.
        Some(Value::Number(n)) => n.max(0.0) as usize,
        _ => DEFAULT_MAX as usize,
    }
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct List {
    sessions: Vec<Entry>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct Entry {
    #[serde(rename = "type")]
    type_id: String,
    specs: Vec<String>,
    description: String,
    settings: Tables,
}

/// Adds the closed session `closed` as the newest, keeping at most `cap`, saved shortly after (or on
/// `flush`). Does nothing when `cap` is 0, or when the session has no inputs to reopen.
pub fn add(store: &mut Store, closed: &Session, cap: usize) {
    if cap == 0 || closed.specs.iter().all(String::is_empty) {
        return;
    }
    let mut list: List = store.section(SECTION);
    list.sessions.retain(|e| e.type_id != closed.type_id || e.specs != closed.specs);
    let entry =
        Entry { type_id: closed.type_id.clone(), specs: closed.specs.clone(), description: closed.description.clone(), settings: tables(closed) };
    list.sessions.insert(0, entry);
    list.sessions.truncate(cap);
    store.update(|s| s.set(SECTION, &list).unwrap_or(()));
}

/// Drops the oldest sessions past `cap`, as when the cap is lowered.
pub fn trim(store: &mut Store, cap: usize) {
    let mut list: List = store.section(SECTION);
    if list.sessions.len() > cap {
        list.sessions.truncate(cap);
        store.update(|s| s.set(SECTION, &list).unwrap_or(()));
    }
}

/// The auto-saved sessions, newest first, each with every registered setting of its type read back
/// (`settings` is every registered setting).
pub fn load(store: &mut Store, settings: &[Setting]) -> Vec<Session> {
    let list: List = store.section(SECTION);
    list.sessions.into_iter().map(|e| session(e.type_id, e.specs, e.description, &e.settings, settings)).collect()
}
