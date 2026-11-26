//! Auto-saved sessions: the cap keeps the newest, 0 adds nothing, closing a comparison again moves it
//! to the front, and the list and each session's settings survive a restart.

use std::path::Path;

use otd_core::session::{Session, Setting, Value};
use otd_settings::{Location, auto_saved};

fn session(specs: &[&str]) -> Session {
    Session { specs: specs.iter().map(|s| (*s).to_owned()).collect(), ..Session::new("text-compare") }
}

fn open(dir: &Path) -> otd_settings::Store {
    otd_settings::open(Location::Dir(dir.to_path_buf())).0
}

fn lefts(store: &mut otd_settings::Store) -> Vec<String> {
    auto_saved::load(store, &[]).into_iter().map(|s| s.specs[0].clone()).collect()
}

#[test]
fn the_cap_keeps_the_newest_a_repeat_moves_to_the_front_and_0_or_no_inputs_add_nothing() {
    let (mut store, _) = otd_settings::open(Location::Memory);
    for left in ["a", "b", "c"] {
        auto_saved::add(&mut store, &session(&[left, "r"]), 2);
    }
    assert_eq!(lefts(&mut store), ["c", "b"]);
    auto_saved::add(&mut store, &session(&["b", "r"]), 2);
    assert_eq!(lefts(&mut store), ["b", "c"], "listed once, as the newest");
    auto_saved::add(&mut store, &session(&["d", "r"]), 0);
    auto_saved::add(&mut store, &session(&["", ""]), 2);
    assert_eq!(lefts(&mut store), ["b", "c"]);
    auto_saved::trim(&mut store, 1);
    assert_eq!(lefts(&mut store), ["b"], "a lowered cap drops the oldest");
}

#[test]
fn auto_saved_sessions_reload_after_a_restart_with_their_settings() {
    let dir = tempfile::tempdir().unwrap();
    let settings = [Setting { session_type: "text-compare", tab: "Rules", id: "width", label: "width", default: Value::Number(4.0), choices: &[] }];
    let mut saved = session(&["/left/a.txt", "/right/a.txt"]);
    saved.description = "weekly".into();
    saved.rules.entry("Rules".into()).or_default().insert("width".into(), Value::Number(7.5));
    let mut store = open(dir.path());
    auto_saved::add(&mut store, &saved, 10);
    store.flush().unwrap();
    drop(store);
    assert_eq!(auto_saved::load(&mut open(dir.path()), &settings), [saved]);
}
