//! Named sessions in the settings folder: they reload after a restart as they were saved, and a
//! damaged session file is skipped with a warning, and moved aside, while the others load.

use std::collections::BTreeMap;
use std::path::Path;

use otd_core::session::{Session, Setting, Value};
use otd_settings::{Location, sessions};

const TEXT: &str = "text-compare";
const FOLDER: &str = "folder-compare";

/// A harness tab on each session type, with every value type. `Rules`/`mode` is on both types with
/// a different value type, so each session must be read against its own type's settings.
fn settings() -> Vec<Setting> {
    let setting = |session_type, tab, id, default| Setting { session_type, tab, id, label: id, default, choices: &[] };
    vec![
        setting(TEXT, "Rules", "mode", Value::Bool(false)),
        setting(TEXT, "Rules", "width", Value::Number(4.0)),
        Setting { choices: &["Lines", "Words"], ..setting(TEXT, "Display", "unit", Value::Choice("Lines".into())) },
        setting(TEXT, "Display", "font", Value::Text("Mono".into())),
        Setting { choices: &["Quick", "Exact"], ..setting(FOLDER, "Rules", "mode", Value::Choice("Quick".into())) },
        setting(FOLDER, "Rules", "filter", Value::Path(String::new())),
        setting(FOLDER, "Look", "accent", Value::Colour([0, 0, 0, 255])),
    ]
}

/// A session of `type_id` with every setting changed from its default.
fn session(type_id: &str, specs: &[&str], description: &str, values: &[(&str, &str, Value)]) -> Session {
    let mut rules: BTreeMap<String, BTreeMap<String, Value>> = BTreeMap::new();
    for (tab, id, value) in values {
        rules.entry((*tab).to_owned()).or_default().insert((*id).to_owned(), value.clone());
    }
    Session {
        name: None,
        type_id: type_id.to_owned(),
        specs: specs.iter().map(|s| (*s).to_owned()).collect(),
        rules,
        description: description.to_owned(),
    }
}

fn open(dir: &Path) -> otd_settings::Store {
    otd_settings::open(Location::Dir(dir.to_path_buf())).0
}

#[test]
fn saved_text_and_folder_sessions_reload_after_a_restart_as_they_were_saved() {
    let dir = tempfile::tempdir().unwrap();
    let text = session(
        TEXT,
        &["/left/a.txt", "/right/a.txt"],
        "Ünïcode \"quoted\"\nsecond line",
        &[
            ("Rules", "mode", Value::Bool(true)),
            ("Rules", "width", Value::Number(7.5)),
            ("Display", "unit", Value::Choice("Words".into())),
            ("Display", "font", Value::Text("Serif".into())),
        ],
    );
    let folder = session(
        FOLDER,
        &["C:\\left", "/right"],
        "",
        &[
            ("Rules", "mode", Value::Choice("Exact".into())),
            ("Rules", "filter", Value::Path("*.rs".into())),
            ("Look", "accent", Value::Colour([10, 20, 30, 128])),
        ],
    );
    // Names that differ only in case, or hold separators, keep sessions of their own.
    let named = |name: &str, session: &Session| (name.to_owned(), Session { name: Some(name.to_owned()), ..session.clone() });
    let saved = BTreeMap::from([named("Weekly", &text), named("weekly", &folder), named("../a/b: c", &text)]);
    {
        let store = open(dir.path());
        for (name, session) in &saved {
            // Saved by the name given, whatever the session holds.
            sessions::save(&store, name, &Session { name: None, ..session.clone() }).unwrap();
        }
    }

    let (loaded, warnings) = sessions::load(&open(dir.path()), &settings());
    assert_eq!(warnings, Vec::<String>::new());
    // Each comes back named.
    assert_eq!(loaded, saved);
    // Nothing was written outside the settings folder's sessions folder.
    let outside: Vec<_> =
        std::fs::read_dir(dir.path()).unwrap().map(|e| e.unwrap().file_name()).filter(|n| n != sessions::FOLDER && n != "config.toml").collect();
    assert_eq!(outside, Vec::<std::ffi::OsString>::new());
}

#[test]
fn a_damaged_session_file_is_skipped_with_a_warning_and_the_others_load() {
    let dir = tempfile::tempdir().unwrap();
    let store = open(dir.path());
    let good = session(FOLDER, &["/a", "/b"], "kept", &[]);
    sessions::save(&store, "good", &good).unwrap();
    sessions::save(&store, "bad", &session(TEXT, &[], "", &[])).unwrap();
    let folder = dir.path().join(sessions::FOLDER);
    let bad =
        std::fs::read_dir(&folder).unwrap().map(|e| e.unwrap().path()).find(|p| std::fs::read_to_string(p).unwrap().contains("\"bad\"")).unwrap();
    std::fs::write(&bad, "name = \"bad\"\ntype = [").unwrap();
    // What an interrupted save leaves behind is no session, and no warning either.
    std::fs::write(folder.join(".s-good.toml.1-0.craft_tmp"), "half a fi").unwrap();

    let (loaded, warnings) = sessions::load(&open(dir.path()), &settings());
    assert_eq!(loaded.keys().collect::<Vec<_>>(), ["good"]);
    assert_eq!(loaded["good"].description, "kept");
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].contains(&bad.display().to_string()), "the warning names the skipped file: {}", warnings[0]);

    // The damaged file was moved aside, so saving under its name keeps what the user wrote in it.
    sessions::save(&store, "bad", &session(TEXT, &[], "new", &[])).unwrap();
    let damaged_kept = std::fs::read_dir(&folder).unwrap().any(|e| std::fs::read_to_string(e.unwrap().path()).unwrap() == "name = \"bad\"\ntype = [");
    assert!(damaged_kept, "the damaged file's contents are still in the sessions folder");
    let moved = warnings[0].split(" moved to ").nth(1).map(|rest| rest.trim_end_matches('.').to_owned());
    assert!(moved.is_some_and(|p| Path::new(&p).exists()), "the warning names where the file went: {}", warnings[0]);
    // And the moved file is not read again as a session.
    let (loaded, warnings) = sessions::load(&open(dir.path()), &settings());
    assert_eq!(warnings, Vec::<String>::new());
    assert_eq!(loaded.keys().collect::<Vec<_>>(), ["bad", "good"]);
}
