//! Program options (E01-S1-T6): what a feature contributes to a named page of the Options dialog.
//! UI-free, so the settings store reads and writes the values without a window.
//!
//! A page appears once an option is contributed to it, and pages nest by path: `["Text",
//! "Editing"]` is the Editing page under Text. Several features may contribute to one page.

use std::collections::BTreeSet;
use std::fmt;

pub use crate::session::Value;

/// An option a feature contributes to a page of the Options dialog.
#[derive(Clone, Debug)]
pub struct OptionSpec {
    /// The page's path in the page tree, in English, at least one name long: the dialog looks
    /// each name up. Options naming the same path share the page.
    pub page: &'static [&'static str],
    /// A stable id, unique among all options ([`check_unique`]): the settings store keeps the
    /// value under it, and features read it by it.
    pub id: &'static str,
    /// The option's caption, in English: the dialog looks it up.
    pub label: &'static str,
    /// The value it has until the user changes it; its variant is the value type.
    pub default: Value,
    /// The choices of a [`Value::Choice`] option, in order, in English. Empty for other types.
    pub choices: &'static [&'static str],
}

/// Two options were contributed with one id.
#[derive(Debug, PartialEq, Eq)]
pub struct DuplicateOption {
    pub id: &'static str,
}

impl fmt::Display for DuplicateOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the option {:?} is contributed twice", self.id)
    }
}

impl std::error::Error for DuplicateOption {}

/// Fails on the first option whose id an earlier one has: the two would share one stored value.
pub fn check_unique(options: &[OptionSpec]) -> Result<(), DuplicateOption> {
    let mut seen = BTreeSet::new();
    match options.iter().find(|o| !seen.insert(o.id)) {
        Some(o) => Err(DuplicateOption { id: o.id }),
        None => Ok(()),
    }
}
