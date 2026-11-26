//! The session model (E01-S1-T7): what a tab compares and the settings it compares with. UI-free, so
//! saving sessions (E06) and the automation interface (E28) work with it without a window.
//!
//! A feature adds an option to a session type's Session Settings dialog by contributing a
//! [`Setting`] to a named tab. Several features may contribute to the same tab; the dialog shows
//! the tab once, holding all of their options.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// A setting's value. The variant is the value type.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    Number(f64),
    /// One of the setting's [`Setting::choices`].
    Choice(String),
    Text(String),
    Path(String),
    /// sRGB with alpha, not premultiplied.
    Colour([u8; 4]),
}

/// An option a feature contributes to a named Session Settings tab of one session type.
#[derive(Clone, Debug)]
pub struct Setting {
    /// The id of the session type whose dialog shows it.
    pub session_type: &'static str,
    /// The tab's name, in English: the dialog looks it up, and the session keeps the setting's
    /// value under it. Settings naming the same tab share it.
    pub tab: &'static str,
    /// A stable id, unique within its tab of its session type ([`check_unique`]).
    pub id: &'static str,
    /// The option's caption, in English: the dialog looks it up.
    pub label: &'static str,
    /// The value an empty session starts with; its variant is the value type.
    pub default: Value,
    /// The choices of a [`Value::Choice`] setting, in order, in English: the dialog shows each
    /// looked up, and the session keeps the English. Empty for other value types.
    pub choices: &'static [&'static str],
}

/// Two settings were contributed with one id to the same tab of one session type.
#[derive(Debug, PartialEq, Eq)]
pub struct DuplicateSetting {
    pub session_type: &'static str,
    pub tab: &'static str,
    pub id: &'static str,
}

impl fmt::Display for DuplicateSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the setting {:?} is contributed twice to the tab {:?} of the session type {:?}", self.id, self.tab, self.session_type)
    }
}

impl std::error::Error for DuplicateSetting {}

/// Fails on the first setting that shares its session type, tab and id with an earlier one: the
/// two would share one value. The same id on another tab or session type is a different setting.
pub fn check_unique(settings: &[Setting]) -> Result<(), DuplicateSetting> {
    let mut seen = BTreeSet::new();
    for s in settings {
        if !seen.insert((s.session_type, s.tab, s.id)) {
            return Err(DuplicateSetting { session_type: s.session_type, tab: s.tab, id: s.id });
        }
    }
    Ok(())
}

/// One comparison's inputs and settings.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Session {
    /// The id of its session type.
    pub type_id: String,
    /// One input per side, such as a file or folder path.
    pub specs: Vec<String>,
    /// The settings' values, by tab and then by setting id.
    pub rules: BTreeMap<String, BTreeMap<String, Value>>,
    pub description: String,
}

impl Session {
    /// An empty session of the type `type_id`: no inputs, rules or description.
    pub fn new(type_id: &str) -> Self {
        Self { type_id: type_id.to_owned(), ..Self::default() }
    }

    /// The value of the setting `id` on the tab `tab`.
    pub fn rule(&self, tab: &str, id: &str) -> Option<&Value> {
        self.rules.get(tab)?.get(id)
    }

    /// Gives each setting contributed for this session's type that has no value yet its default.
    pub fn fill_defaults(&mut self, settings: &[Setting]) {
        for setting in settings.iter().filter(|s| s.session_type == self.type_id) {
            self.rules.entry(setting.tab.to_owned()).or_default().entry(setting.id.to_owned()).or_insert_with(|| setting.default.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setting(session_type: &'static str, tab: &'static str, id: &'static str) -> Setting {
        Setting { session_type, tab, id, label: id, default: Value::Bool(false), choices: &[] }
    }

    #[test]
    fn a_second_setting_with_the_same_id_on_the_same_tab_is_rejected_by_name() {
        let settings = [setting("a", "T", "x"), setting("b", "T", "x"), setting("a", "U", "x"), setting("a", "T", "x")];
        assert_eq!(check_unique(&settings[..3]), Ok(()), "another session type or tab is another setting");
        assert_eq!(check_unique(&settings), Err(DuplicateSetting { session_type: "a", tab: "T", id: "x" }));
    }
}
