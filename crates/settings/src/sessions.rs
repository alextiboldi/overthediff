//! Named sessions (E06-S1-T1): each saved comparison is a TOML file of its own in the settings
//! folder's `sessions` folder, so a damaged one is skipped and the others still load, and saving
//! one rewrites only its file, crash-safely. A skipped file is moved aside, so that saving a session
//! under its name can't replace what is in it.
//!
//! A file holds the session's name, type, specs, description and settings (by tab, then by setting
//! id). Its file name is the name, encoded so that two names never share a file, even on a file
//! system that ignores case, and so that no name can reach outside the folder.
//!
//! Settings are read against the ones registered for the session's type, as the Options dialog's
//! values are: a missing value, or one that doesn't fit its setting, reads as the setting's
//! default. A saved value whose setting isn't registered any more is dropped.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

use otd_core::i18n::{fmt, t};
use otd_core::session::{Session, Setting};
use serde::{Deserialize, Serialize};

use crate::Store;
use crate::options::{from_toml, to_toml};

/// The folder, in the settings folder, that holds one file per named session.
pub const FOLDER: &str = "sessions";
const EXTENSION: &str = ".toml";
/// The longest file name: inside every supported file system's limit of 255 even with the
/// `.broken-<timestamp>-<n>` that moving a damaged file aside adds (up to 23 characters).
const MAX_FILE_NAME: usize = 230;

/// A session file's contents.
#[derive(Serialize, Deserialize)]
struct File {
    name: String,
    #[serde(rename = "type")]
    type_id: String,
    #[serde(default)]
    specs: Vec<String>,
    #[serde(default)]
    description: String,
    #[serde(default)]
    settings: Tables,
}

/// Saves `session` as the named session `name`, replacing one saved under that name. The write is
/// crash-safe: an interrupted save leaves the previous file whole. Fails, with a message to show
/// the user, when the name is empty or too long, or the settings aren't saved to disk.
pub fn save(store: &Store, name: &str, session: &Session) -> Result<(), String> {
    let path = path(store, name)?;
    let file = File {
        name: name.to_owned(),
        type_id: session.type_id.clone(),
        specs: session.specs.clone(),
        description: session.description.clone(),
        settings: tables(session),
    };
    let text = toml::to_string(&file).map_err(|e| failed(name, &e.to_string()))?;
    let dir = path.parent().unwrap_or(&path);
    std::fs::create_dir_all(dir).and_then(|()| craft_atomic::atomic_write(&path, text.as_bytes())).map_err(|e| failed(name, &e.to_string()))?;
    store.sessions_changes.fetch_add(1, Ordering::Relaxed);
    Ok(())
}

/// A count that goes up with each change this store makes to the named sessions, so a list of
/// them (Home's sessions tree) knows when to [`load`] them again.
pub fn changes(store: &Store) -> u64 {
    store.sessions_changes.load(Ordering::Relaxed)
}

/// Whether a session is saved under `name`, damaged or not: saving under it replaces that file.
pub fn exists(store: &Store, name: &str) -> bool {
    path(store, name).is_ok_and(|path| path.exists())
}

/// Every named session, by name (also its [`Session::name`]), with each registered setting of its type read back (`settings`
/// is every registered setting), and a warning to show the user for each file that was skipped.
pub fn load(store: &Store, settings: &[Setting]) -> (BTreeMap<String, Session>, Vec<String>) {
    let (mut sessions, mut warnings) = (BTreeMap::new(), Vec::new());
    let Some(dir) = store.dir().map(|d| d.join(FOLDER)) else { return (sessions, warnings) };
    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return (sessions, warnings),
        Err(e) => {
            warnings.push(fmt(
                t("The saved sessions in {path} couldn't be read ({error})."),
                &[("path", &dir.display().to_string()), ("error", &e.to_string())],
            ));
            return (sessions, warnings);
        }
    };
    let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    paths.sort();
    // Only our own files, whose names hold no dot but the extension's: not the hidden staging file
    // an interrupted save leaves behind, nor a damaged file already moved aside.
    let ours = |p: &PathBuf| p.file_name().and_then(|n| n.to_str()?.strip_suffix(EXTENSION)).is_some_and(|stem| !stem.contains('.'));
    for path in paths.iter().filter(|p| ours(p)) {
        let why = match read(path, settings) {
            Ok((name, session)) => {
                sessions.insert(name, session);
                continue;
            }
            Err(why) => why,
        };
        let shown = path.display().to_string();
        // A read-only store never saves, so nothing can replace the file.
        warnings.push(if store.is_read_only() {
            fmt(t("The saved session file {path} was skipped: {why}."), &[("path", &shown), ("why", &why)])
        } else {
            match craft_settings::recovery::move_aside(path) {
                Ok(moved) => fmt(
                    t("The saved session file {path} was skipped ({why}), so it was moved to {moved}."),
                    &[("path", &shown), ("why", &why), ("moved", &moved.display().to_string())],
                ),
                Err(e) => fmt(
                    t("The saved session file {path} was skipped ({why}), and it couldn't be moved aside ({error}): saving a session under the name it holds replaces it."),
                    &[("path", &shown), ("why", &why), ("error", &e.to_string())],
                ),
            }
        });
    }
    (sessions, warnings)
}

fn read(path: &Path, settings: &[Setting]) -> Result<(String, Session), String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let file: File = toml::from_str(&text).map_err(|e| e.message().to_owned())?;
    // The name decides the file, so a file under another name (copied, or renamed by hand) would
    // be overwritten by, or shadow, the one the name saves to.
    if path.file_name().and_then(|n| n.to_str()) != Some(&file_name(&file.name)) {
        return Err(fmt(t("its file name doesn't match the session name \"{name}\""), &[("name", &file.name)]));
    }
    let mut session = session(file.type_id, file.specs, file.description, &file.settings, settings);
    session.name = Some(file.name.clone());
    Ok((file.name, session))
}

/// A session with each registered setting of its type read from `saved` (by tab, then by id).
pub(crate) fn session(type_id: String, specs: Vec<String>, description: String, saved: &Tables, settings: &[Setting]) -> Session {
    let mut session = Session { type_id, specs, description, ..Session::default() };
    for setting in settings.iter().filter(|s| s.session_type == session.type_id) {
        let saved = saved.get(setting.tab).and_then(|values| values.get(setting.id));
        let value = saved.and_then(|v| from_toml(&setting.default, setting.choices, v)).unwrap_or_else(|| setting.default.clone());
        session.rules.entry(setting.tab.to_owned()).or_default().insert(setting.id.to_owned(), value);
    }
    session
}

/// A session's settings as saved: `to_toml` values by tab, then by setting id.
pub(crate) fn tables(session: &Session) -> Tables {
    session.rules.iter().map(|(tab, values)| (tab.clone(), values.iter().map(|(id, v)| (id.clone(), to_toml(v))).collect())).collect()
}

pub(crate) type Tables = BTreeMap<String, toml::Table>;

/// The file the session `name` is saved in.
fn path(store: &Store, name: &str) -> Result<PathBuf, String> {
    if name.is_empty() {
        return Err(t("A session needs a name.").to_owned());
    }
    let file = file_name(name);
    if file.len() > MAX_FILE_NAME {
        return Err(fmt(t("The session name \"{name}\" is too long."), &[("name", name)]));
    }
    match store.dir() {
        Some(dir) if !store.is_read_only() => Ok(dir.join(FOLDER).join(file)),
        _ => Err(failed(name, t("settings aren't being saved"))),
    }
}

/// `s-` and the name with every byte but a lowercase ASCII letter, digit or `-` written as `_` and
/// two lowercase hex digits; or, when that is too long, `b-` and the name's bytes in lowercase
/// base32, which takes 1.6 characters a byte rather than up to 3. So it differs for every name,
/// even ignoring case; it holds only lowercase ASCII letters, digits, `-` and `_`, so no separator
/// or dot; and the prefix keeps it from being a name Windows reserves, such as `con`.
fn file_name(name: &str) -> String {
    let mut file = String::from("s-");
    for b in name.bytes() {
        if b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' {
            file.push(char::from(b));
        } else {
            file.push_str(&format!("_{b:02x}"));
        }
    }
    file.push_str(EXTENSION);
    if file.len() <= MAX_FILE_NAME {
        return file;
    }
    // Unpadded base32 is one-to-one: each byte count gives its own length, and the spare bits are 0.
    const DIGITS: &[u8; 32] = b"abcdefghijklmnopqrstuvwxyz234567";
    let (mut file, mut bits, mut held) = (String::from("b-"), 0u32, 0u32);
    for b in name.bytes() {
        bits = ((bits << 8) | u32::from(b)) & 0xfff;
        held += 8;
        while held >= 5 {
            held -= 5;
            file.push(char::from(DIGITS[((bits >> held) & 31) as usize]));
        }
    }
    if held > 0 {
        file.push(char::from(DIGITS[((bits << (5 - held)) & 31) as usize]));
    }
    file + EXTENSION
}

fn failed(name: &str, why: &str) -> String {
    fmt(t("The session \"{name}\" couldn't be saved: {why}."), &[("name", name), ("why", why)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_that_differ_only_in_case_or_by_escaping_get_different_files_with_no_separator_or_dot() {
        let long = |unit: &str, n| unit.repeat(n);
        let names = [
            "Weekly".to_owned(),
            "weekly".into(),
            "WEEKLY".into(),
            "_57eekly".into(),
            "a/b".into(),
            "a\\b".into(),
            "..".into(),
            "con".into(),
            "release 1.2".into(),
            // Too long to escape, so written in base32, and so are names differing only at the end.
            long("Ж", 60),
            long("Ж", 60) + "ж",
            long("ж", 60),
            long("字", 40),
            long("字", 40) + "a",
            long("Weekly Release ", 9),
        ];
        let files: std::collections::BTreeSet<String> = names.iter().map(|n| file_name(n).to_ascii_lowercase()).collect();
        assert_eq!(files.len(), names.len(), "one file per name, ignoring case: {files:?}");
        for file in &files {
            let stem = file.strip_suffix(EXTENSION).unwrap_or(file);
            assert!(!stem.contains(['/', '\\', '.', ':']), "{file} holds a separator or dot");
            assert!(file.len() <= MAX_FILE_NAME, "{file} is too long");
        }
    }
}
