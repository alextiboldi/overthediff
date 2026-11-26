//! Session defaults (E06-S2-T1): one settings set per session type, which new sessions of that
//! type start with. They live in the `session-defaults` section as `[session-defaults.<type>.<tab>]`
//! tables, one key per setting id, with the values as the Options dialog's are (`options`).
//!
//! Only a session's settings are kept: its specs and description are not defaults.

use std::collections::BTreeMap;

use otd_core::session::{Session, Setting, Value};

use crate::Store;
use crate::options::{from_toml, to_toml};

/// The section the defaults live in.
pub const SECTION: &str = "session-defaults";

/// The settings' values, by tab and then by setting id, as [`Session::rules`] holds them.
pub type Rules = BTreeMap<String, BTreeMap<String, Value>>;

/// The defaults of the session type `type_id`: every setting contributed for it, with its stored
/// value, or the setting's own default when none is stored or the stored one doesn't fit.
pub fn read(store: &mut Store, settings: &[Setting], type_id: &str) -> Rules {
    let saved: toml::Table = store.section(SECTION);
    let saved = saved.get(type_id).and_then(toml::Value::as_table);
    let mut rules = Rules::new();
    for setting in settings.iter().filter(|s| s.session_type == type_id) {
        let stored = saved.and_then(|t| t.get(setting.tab)?.as_table()?.get(setting.id));
        let value = stored.and_then(|v| from_toml(&setting.default, setting.choices, v)).unwrap_or_else(|| setting.default.clone());
        rules.entry(setting.tab.to_owned()).or_default().insert(setting.id.to_owned(), value);
    }
    rules
}

/// Makes `session`'s settings the defaults for its type, saved shortly after (or on `flush`).
/// Other types' defaults are left as they are. Pass a session whose settings are complete (as
/// `fill_defaults` leaves them): saving merges key by key, so a setting missing from it keeps its
/// old stored value in the file.
pub fn write(store: &mut Store, session: &Session) {
    let mut saved: toml::Table = store.section(SECTION);
    let tabs: toml::Table = session
        .rules
        .iter()
        .map(|(tab, values)| (tab.clone(), toml::Value::Table(values.iter().map(|(id, v)| (id.clone(), to_toml(v))).collect())))
        .collect();
    saved.insert(session.type_id.clone(), toml::Value::Table(tabs));
    // A table of plain values always serializes.
    store.update(|s| s.set(SECTION, &saved).unwrap_or(()));
}
