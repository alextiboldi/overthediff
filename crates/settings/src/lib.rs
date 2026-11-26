//! OverTheDiff's settings store, on craft-settings: one `config.toml` in a per-user,
//! per-major-version folder, saved atomically, with a damaged file moved aside.
//!
//! Each feature keeps its settings in a section of its own, named after it, and needs no edit
//! anywhere else:
//!
//! ```
//! use otd_settings::{Location, Store};
//!
//! #[derive(serde::Serialize, serde::Deserialize, Default, PartialEq, Debug)]
//! #[serde(default)]
//! struct Recent {
//!     paths: Vec<String>,
//! }
//!
//! let (mut store, _notice) = otd_settings::open(Location::Memory);
//! store.update(|s| s.set("recent-paths", &Recent { paths: vec!["a.txt".into()] }).unwrap_or(()));
//! let recent: Recent = store.section("recent-paths");
//! assert_eq!(recent.paths, ["a.txt"]);
//! ```
//!
//! Read a section with [`Store::section`], not `store.get().get(..)`: when the section in the file
//! doesn't fit its type, `section` keeps a copy of the file and tells the user before the
//! feature's next save replaces the section with defaults.
//!
//! Saving merges into the file key by key, so a key that disappears from a section stays in the
//! file and comes back on the next start. Keep what can shrink in an array (a list of entries,
//! not a map or `Option` fields), which is replaced whole (alextiboldi/overthediff#11).

pub mod auto_saved;
pub mod options;
pub mod session_defaults;
pub mod sessions;

use std::collections::BTreeSet;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

use craft_settings::{LoadReport, ReadOnlyReason, SettingsStore};
use otd_core::i18n::{fmt, t};
use serde::Serialize;
use serde::de::DeserializeOwned;

pub use craft_settings::{Location, SaveStatus, SettingsError, TickResult};

/// The settings folder's name and the start of its override variable (`OVERTHEDIFF_<major>_CONFIG_DIR`;
/// set but empty keeps settings in memory). Each major version has its own, so two can run side
/// by side (F-27.1). The folder sits in the OS's per-user config folder: `~/.config` (or
/// `$XDG_CONFIG_HOME`) on Linux, `~/Library/Application Support` on macOS, `%APPDATA%` on Windows.
pub const APP_ID: &str = concat!("overthediff-", env!("CARGO_PKG_VERSION_MAJOR"));

/// Every setting, one named section per feature. A section that doesn't fit its type reads as an
/// error and leaves the others alone.
#[derive(Serialize, serde::Deserialize, Default, Clone, PartialEq, Debug)]
#[serde(transparent)]
pub struct Sections(toml::Table);

impl craft_settings::Settings for Sections {
    const APP_ID: &'static str = APP_ID;
    const SCHEMA_VERSION: u32 = 1;
    // Portable installs are E23-S4's; until then settings stay in the per-user folder.
    const PORTABLE_MARKERS: &'static [&'static str] = &[];
}

impl Sections {
    /// The section `name`, or `S`'s default when there is none. An error means the section in the
    /// file doesn't fit `S`; features use [`Store::section`], which handles it.
    pub fn get<S: DeserializeOwned + Default>(&self, name: &str) -> Result<S, toml::de::Error> {
        match self.0.get(name) {
            Some(value) => value.clone().try_into(),
            None => Ok(S::default()),
        }
    }

    /// Replaces the section `name`.
    pub fn set<S: Serialize>(&mut self, name: &str, value: &S) -> Result<(), toml::ser::Error> {
        self.0.insert(name.to_owned(), toml::Value::try_from(value)?);
        Ok(())
    }
}

/// The store: craft-settings' store, plus the notices for damaged sections. Read a section with
/// `section`, change it with `update` (saved shortly after), call `tick` every frame and `flush`
/// on exit. `dir()` is the folder, for collections kept in files of their own.
pub struct Store {
    inner: SettingsStore<Sections>,
    /// Sections already found damaged, so each is copied and reported once.
    damaged: BTreeSet<String>,
    notices: Vec<String>,
}

impl Deref for Store {
    type Target = SettingsStore<Sections>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Store {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Store {
    /// The section `name`, or `S`'s default when there is none. When the section in the file
    /// doesn't fit `S` (say a hand edit put text where a number goes), this returns the default,
    /// and the first time keeps a copy of the file and queues a notice naming it, so the feature's
    /// next `set` can't lose what the user wrote.
    pub fn section<S: DeserializeOwned + Default>(&mut self, name: &str) -> S {
        let error = match self.inner.get().get(name) {
            Ok(value) => return value,
            Err(error) => error,
        };
        if self.damaged.insert(name.to_owned()) {
            let kept = match self.inner.path() {
                // A read-only store never writes, so the file needs no copy.
                Some(path) if !self.inner.is_read_only() => Some(keep_copy(path)),
                _ => None,
            };
            let why = error.message();
            self.notices.push(match kept {
                Some(Ok(copy)) => fmt(
                    t("The \"{name}\" settings didn't fit ({why}), so the default ones are in use and a copy of the settings file was kept at {copy}."),
                    &[("name", name), ("why", why), ("copy", &copy.display().to_string())],
                ),
                Some(Err(e)) => fmt(
                    t("The \"{name}\" settings didn't fit ({why}), so the default ones are in use, and a copy of the settings file couldn't be kept ({error}): the next change to them replaces what is there."),
                    &[("name", name), ("why", why), ("error", &e.to_string())],
                ),
                None => fmt(t("The \"{name}\" settings didn't fit ({why}), so the default ones are in use."), &[("name", name), ("why", why)]),
            });
        }
        S::default()
    }

    /// The notices queued by `section` since the last call, to show the user.
    pub fn take_notices(&mut self) -> Vec<String> {
        std::mem::take(&mut self.notices)
    }
}

/// Copies `config.toml` to `config.damaged-<unix-ts>.toml` beside it, never over another file.
fn keep_copy(path: &Path) -> std::io::Result<PathBuf> {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    for n in 0..1000u32 {
        let target = dir.join(if n == 0 { format!("config.damaged-{ts}.toml") } else { format!("config.damaged-{ts}-{n}.toml") });
        if !target.exists() {
            std::fs::copy(path, &target)?;
            return Ok(target);
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "too many copies"))
}

/// Opens the store at `location` (`Location::Auto` for the real one). Never fails; the second
/// value is a notice to show the user when the settings were reset or won't be saved.
pub fn open(location: Location) -> (Store, Option<String>) {
    let (store, notice) = open_inner(location);
    (Store { inner: store, damaged: BTreeSet::new(), notices: Vec::new() }, notice)
}

fn open_inner(location: Location) -> (SettingsStore<Sections>, Option<String>) {
    let (store, report) = SettingsStore::<Sections>::open(location.clone());
    // craft-settings sets aside a file that isn't TOML, but opens read-only on one that isn't even
    // UTF-8 (alextiboldi/overthediff#17), so that one is set aside here and the store reopened.
    let not_text = matches!(&report.read_only, Some(ReadOnlyReason::Unreadable(e)) if e.kind() == std::io::ErrorKind::InvalidData);
    if let (true, Some(path)) = (not_text, store.path().map(Path::to_path_buf)) {
        drop(store);
        let moved = craft_settings::recovery::move_aside(&path);
        let (store, mut report) = SettingsStore::<Sections>::open(location);
        report.broken_moved_to = moved.ok();
        return (store, notice(&report));
    }
    (store, notice(&report))
}

fn notice(report: &LoadReport) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(moved) = &report.broken_moved_to {
        parts.push(fmt(
            t("The settings file was damaged, so it was moved to {path} and the default settings are in use."),
            &[("path", &moved.display().to_string())],
        ));
    }
    if let Some(why) = &report.read_only {
        parts.push(match why {
            // An older build's settings are untouched, so this isn't a fault to alarm anyone about.
            ReadOnlyReason::NewerSchema { .. } => t("These settings come from a newer version of OverTheDiff, so changes won't be saved.").to_owned(),
            other => fmt(t("Settings won't be saved: {reason}."), &[("reason", &other.to_string())]),
        });
    }
    (!parts.is_empty()).then(|| parts.join(" "))
}

/// The notice for a settings file someone damaged while the app ran, which a save moved to `moved`
/// (`Store::take_moved_aside`) before writing the current settings in its place.
pub fn damaged_while_running_notice(moved: &Path) -> String {
    fmt(
        t("The settings file was damaged while OverTheDiff was running, so it was moved to {path} and the current settings were saved in its place."),
        &[("path", &moved.display().to_string())],
    )
}
