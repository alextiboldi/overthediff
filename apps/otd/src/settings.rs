//! The app's settings store: opened at startup, saved during use and on exit, and the notice the
//! user sees when the settings were reset or can't be saved. It also reads the named sessions
//! again for Home whenever they change.

use std::collections::BTreeMap;

use eframe::egui;
use otd_core::i18n::{fmt, t};
use otd_core::session::Session;
use otd_settings::{Location, SaveStatus, SettingsError, Store};

pub struct Settings {
    pub store: Store,
    /// Why settings were reset or won't be saved: from startup, a damaged section, or a save that
    /// found the file damaged.
    notices: Vec<String>,
    /// The last save's failure, until a save succeeds.
    save_error: Option<String>,
    /// The named sessions' change count ([`otd_settings::sessions::changes`]) when they were last read.
    sessions_read: Option<u64>,
}

impl Settings {
    /// Opens the store; `Location::Auto` is the real one, in the per-user, per-major-version folder.
    pub fn open(location: Location) -> Self {
        let (store, notice) = otd_settings::open(location);
        Self { store, notices: notice.into_iter().collect(), save_error: None, sessions_read: None }
    }

    /// Every named session, by name, read again when they changed since the last call (or on the
    /// first): Home's Saved branch. A file that was skipped adds its warning to the notices, once.
    pub fn saved_sessions(&mut self) -> Option<BTreeMap<String, Session>> {
        let changes = otd_settings::sessions::changes(&self.store);
        if self.sessions_read == Some(changes) {
            return None;
        }
        self.sessions_read = Some(changes);
        let (sessions, warnings) = otd_settings::sessions::load(&self.store, &crate::session_settings::session_settings());
        // A file that can't be moved aside warns again on every read, so it is shown once.
        for warning in warnings {
            if !self.notices.contains(&warning) {
                self.notices.push(warning);
            }
        }
        Some(sessions)
    }

    /// Saves pending changes when they are due, and shows the notice above the window's contents.
    pub fn frame(&mut self, ui: &mut egui::Ui) {
        let tick = self.store.tick(std::time::Instant::now());
        match &tick.status {
            // A read-only store has stopped saving for good, so it promises no retry.
            SaveStatus::Failed(SettingsError::ReadOnly(why)) => {
                self.save_error = Some(fmt(t("Settings won't be saved for the rest of this session: {reason}."), &[("reason", &why.to_string())]))
            }
            SaveStatus::Failed(e) => self.save_error = Some(fmt(t("{error}. OverTheDiff will keep trying."), &[("error", &e.to_string())])),
            SaveStatus::Saved => self.save_error = None,
            SaveStatus::Idle | SaveStatus::Pending => {}
        }
        let moved = self.store.take_moved_aside();
        self.notices.extend(moved.iter().map(|path| otd_settings::damaged_while_running_notice(path)));
        let damaged = self.store.take_notices();
        self.notices.extend(damaged);
        if let Some(deadline) = tick.next_deadline {
            ui.ctx().request_repaint_after(deadline.saturating_duration_since(std::time::Instant::now()));
        }

        // ponytail: a plain label above the shell; the status bar shows the active view's fields
        // (Q25), not the shell's messages. A shared notice surface can take these when one exists.
        if self.notices.is_empty() && self.save_error.is_none() {
            return;
        }
        egui::Panel::top("settings-notice").show(ui, |ui| {
            for text in self.notices.iter().chain(&self.save_error) {
                let label = ui.label(text);
                // Announced, since it appears away from focus.
                ui.ctx().accesskit_node_builder(label.id, |node| node.set_live(egui::accesskit::Live::Polite));
            }
        });
    }

    /// Saves on exit; a failure is reported on stderr, since the window is gone.
    pub fn exit(&mut self) {
        if let Err(e) = self.store.flush() {
            eprintln!("otd: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use egui::accesskit::Live;
    use egui_kittest::Harness;
    use egui_kittest::kittest::{NodeT as _, Queryable as _};

    use super::*;

    #[test]
    fn settings_reset_at_startup_show_an_announced_notice_naming_the_set_aside_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("config.toml"), "[broken").unwrap();
        let settings = Settings::open(Location::Dir(dir.path().to_path_buf()));
        let set_aside = std::fs::read_dir(dir.path()).unwrap().map(|e| e.unwrap().path()).find(|p| p.file_name().unwrap() != "config.toml").unwrap();

        let mut harness = Harness::builder().build_ui_state(|ui, settings: &mut Settings| settings.frame(ui), settings);
        harness.run();
        let set_aside = set_aside.display().to_string();
        let announced: Vec<String> =
            harness.query_all_by(|node| node.live() == Live::Polite).filter_map(|node| node.accesskit_node().value()).collect();
        assert!(announced.iter().any(|text| text.contains(&set_aside)), "an announced notice names where the file went: {announced:?}");
    }

    /// Home's tree lists a session as soon as it is saved, and the folder isn't read every frame.
    #[test]
    fn the_named_sessions_are_read_again_after_a_save_and_only_then() {
        let dir = tempfile::tempdir().unwrap();
        let mut settings = Settings::open(Location::Dir(dir.path().to_path_buf()));
        assert_eq!(settings.saved_sessions().map(|s| s.len()), Some(0), "read on the first call");
        assert!(settings.saved_sessions().is_none(), "not again while nothing changed");
        otd_settings::sessions::save(&settings.store, "Weekly", &Session::new("text")).unwrap();
        let saved = settings.saved_sessions().unwrap_or_default();
        assert!(saved.contains_key("Weekly"), "read again after the save: {saved:?}");
    }
}
