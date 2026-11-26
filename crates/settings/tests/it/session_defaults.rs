//! Session defaults in the settings store: one settings set per session type, kept across restarts.

use otd_core::session::{Session, Setting, Value};
use otd_settings::{Location, session_defaults};

const FOLDER: &str = "folder-compare";
const TEXT: &str = "text-compare";

/// The same tab and ids on both types, so defaults leaking from one type to the other would show.
fn settings() -> Vec<Setting> {
    let setting = |session_type, id, default| Setting { session_type, tab: "Rules", id, label: id, default, choices: &[] };
    let mut all = Vec::new();
    for session_type in [FOLDER, TEXT] {
        all.push(setting(session_type, "flag", Value::Bool(false)));
        all.push(Setting { choices: &["Lines", "Words"], ..setting(session_type, "unit", Value::Choice("Lines".into())) });
    }
    all
}

fn open(dir: &std::path::Path) -> otd_settings::Store {
    otd_settings::open(Location::Dir(dir.to_path_buf())).0
}

fn changed(type_id: &str) -> Session {
    let mut session = Session::new(type_id);
    session.fill_defaults(&settings());
    let rules = session.rules.get_mut("Rules").unwrap();
    rules.insert("flag".into(), Value::Bool(true));
    rules.insert("unit".into(), Value::Choice("Words".into()));
    session
}

fn defaults(type_id: &str) -> session_defaults::Rules {
    let mut session = Session::new(type_id);
    session.fill_defaults(&settings());
    session.rules
}

#[test]
fn folder_compare_defaults_survive_a_restart_and_never_apply_to_text_compare() {
    let dir = tempfile::tempdir().unwrap();
    let mut folder = changed(FOLDER);
    folder.specs = vec!["left-spec-dir".into(), "right-spec-dir".into()];
    folder.description = "the-description".into();
    assert_ne!(folder.rules, defaults(FOLDER));

    let mut store = open(dir.path());
    session_defaults::write(&mut store, &folder);
    store.flush().unwrap();
    drop(store);

    let mut store = open(dir.path());
    assert_eq!(session_defaults::read(&mut store, &settings(), FOLDER), folder.rules);
    assert_eq!(session_defaults::read(&mut store, &settings(), TEXT), defaults(TEXT), "Text Compare keeps its own defaults");
    let file = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(!file.contains("spec-dir") && !file.contains("the-description"), "specs and description aren't defaults:\n{file}");

    // Updating Text Compare's defaults leaves Folder Compare's as they were.
    session_defaults::write(&mut store, &changed(TEXT));
    store.flush().unwrap();
    drop(store);
    let mut store = open(dir.path());
    assert_eq!(session_defaults::read(&mut store, &settings(), FOLDER), folder.rules);
    assert_eq!(session_defaults::read(&mut store, &settings(), TEXT), changed(TEXT).rules);
}

#[test]
fn defaults_set_back_to_the_settings_own_replace_the_stored_ones_after_a_restart() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = open(dir.path());
    session_defaults::write(&mut store, &changed(FOLDER));
    store.flush().unwrap();
    let mut plain = Session::new(FOLDER);
    plain.rules = defaults(FOLDER);
    session_defaults::write(&mut store, &plain);
    store.flush().unwrap();
    drop(store);
    assert_eq!(session_defaults::read(&mut open(dir.path()), &settings(), FOLDER), defaults(FOLDER));
}
