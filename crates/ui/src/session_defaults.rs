//! New sessions start from their type's defaults (E06-S2-T3). The shell keeps a copy of each session
//! type's defaults from the settings store (`otd_settings::session_defaults`), and every new view's
//! session, from Home, the Session menu or the command line, takes them for the settings it has no
//! value for yet. Specs and description are never defaults, so they come only from the view.

use otd_settings::{Store, session_defaults};

use crate::{Session, Shell};

impl Shell {
    /// Loads each session type's defaults from `store`: the settings added with
    /// [`Shell::with_settings`], so call it after that, and again after the stored defaults change.
    pub fn load_session_defaults(&mut self, store: &mut Store) {
        self.session_defaults = self.session_types.iter().map(|t| (t.id.to_owned(), session_defaults::read(store, &self.settings, t.id))).collect();
    }

    /// Gives each setting `session` has no value for its type's loaded default.
    pub(crate) fn apply_session_defaults(&self, session: &mut Session) {
        let Some(defaults) = self.session_defaults.get(&session.type_id) else { return };
        for (tab, values) in defaults {
            let rules = session.rules.entry(tab.clone()).or_default();
            for (id, value) in values {
                rules.entry(id.clone()).or_insert_with(|| value.clone());
            }
        }
    }
}
