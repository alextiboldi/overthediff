//! Content options on the Comparison tab (E08-S5-T1): whether content tests run, by CRC or byte by
//! byte, whether pairs the quick tests call same are skipped, and whether a content match
//! overrides the quick tests. They are session settings, so the session keeps them, and changing
//! the Comparison tab compares the listed folders again.
//!
//! The options reach the folder engine as [`ContentOptions`] ([`FolderCompare::content_options`]).
//! Running the tests and folding their results into the rows is E08-S5-T2's: until then the
//! options change no row's status.

use otd_folder_engine::{ContentOptions, ContentTest};
use otd_ui::{Setting, Value};

use crate::{FolderCompare, ID};

// English keys: the dialog looks them up, and the session keeps the tab and choices in English.
pub const COMPARISON: &str = "Comparison";
pub const COMPARE_CONTENTS: &str = "Compare contents";
pub const CONTENT_COMPARISON: &str = "Content comparison";
pub const CRC_COMPARISON: &str = "CRC comparison";
pub const BINARY_COMPARISON: &str = "Binary comparison";
pub const SKIP_IF_QUICK_SAME: &str = "Skip if quick tests indicate files are the same";
pub const OVERRIDE_QUICK_RESULTS: &str = "Override quick test results";

// Stable ids: the session-defaults store keeps the values under them.
const COMPARE_ID: &str = "compare_contents";
const METHOD_ID: &str = "content_comparison";
const SKIP_ID: &str = "skip_if_quick_same";
const OVERRIDE_ID: &str = "override_quick_results";

/// The content options, for `apps/otd/src/session_settings.rs`. Their defaults are
/// [`ContentOptions::default`]'s: Override on (F-8.1), no content test and no skipping
/// (*unspecified*), and CRC as the method once Compare contents is on (*unspecified*: the first
/// the catalogue lists).
pub fn session_settings() -> Vec<Setting> {
    let default = ContentOptions::default();
    let setting = |id, label, default, choices| Setting { session_type: ID, tab: COMPARISON, id, label, default, choices };
    // ponytail: the method stays editable while Compare contents is off; the dialog has no
    // dependent options yet.
    vec![
        setting(COMPARE_ID, COMPARE_CONTENTS, Value::Bool(default.test.is_some()), &[]),
        setting(METHOD_ID, CONTENT_COMPARISON, Value::Choice(CRC_COMPARISON.to_owned()), &[CRC_COMPARISON, BINARY_COMPARISON]),
        setting(SKIP_ID, SKIP_IF_QUICK_SAME, Value::Bool(default.skip_if_quick_same), &[]),
        setting(OVERRIDE_ID, OVERRIDE_QUICK_RESULTS, Value::Bool(default.override_quick_results), &[]),
    ]
}

impl FolderCompare {
    /// The session's content options. An option the session has no value for takes its default.
    pub fn content_options(&self) -> ContentOptions {
        let default = ContentOptions::default();
        let rule = |id| self.session.rule(COMPARISON, id);
        let on = |id, default| match rule(id) {
            Some(Value::Bool(on)) => *on,
            _ => default,
        };
        let method = match rule(METHOD_ID) {
            Some(Value::Choice(choice)) if choice == BINARY_COMPARISON => ContentTest::Binary,
            _ => ContentTest::Crc,
        };
        ContentOptions {
            test: on(COMPARE_ID, default.test.is_some()).then_some(method),
            skip_if_quick_same: on(SKIP_ID, default.skip_if_quick_same),
            override_quick_results: on(OVERRIDE_ID, default.override_quick_results),
        }
    }

    /// Compares again, in place, when the Comparison tab's values differ from the ones the rows
    /// were compared with: Session Settings' OK, or a script, changed them.
    pub(crate) fn recompare_if_changed(&mut self) {
        let now = self.session.rules.get(COMPARISON);
        if self.compared.as_ref() != now {
            self.compared = now.cloned();
            self.refresh();
        }
    }
}
