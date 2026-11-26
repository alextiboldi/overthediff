use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use otd_settings::{Location, Sections, Store};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, PartialEq, Debug)]
#[serde(default)]
struct Recent {
    paths: Vec<String>,
}

fn open(dir: &Path) -> (Store, Option<String>) {
    otd_settings::open(Location::Dir(dir.to_path_buf()))
}

#[test]
fn a_section_saved_before_a_restart_reads_back_after_it() {
    let dir = tempfile::tempdir().unwrap();
    let (mut store, _) = open(dir.path());
    store.update(|s| s.set("recent-paths", &Recent { paths: vec!["left.txt".into(), "right.txt".into()] }).unwrap());
    store.flush().unwrap();
    // A shorter list replaces the longer one: AGENTS.md tells features to keep what shrinks in arrays.
    let recent = Recent { paths: vec!["right.txt".into()] };
    store.update(|s| s.set("recent-paths", &recent).unwrap());
    store.flush().unwrap();
    drop(store);

    let (store, notice) = open(dir.path());
    assert_eq!(notice, None);
    assert_eq!(store.get().get::<Recent>("recent-paths").unwrap(), recent);
}

#[test]
fn a_corrupt_file_is_set_aside_and_the_store_starts_from_defaults_with_a_notice() {
    let dir = tempfile::tempdir().unwrap();
    let junk = "[recent-paths\npaths = [\"left.txt\"";
    std::fs::write(dir.path().join("config.toml"), junk).unwrap();
    assert_set_aside_with_defaults(dir.path(), junk.as_bytes());
}

#[test]
fn a_settings_file_that_is_not_text_is_set_aside_too() {
    let dir = tempfile::tempdir().unwrap();
    let junk = b"\xff\xfe\x00schema_version = 1";
    std::fs::write(dir.path().join("config.toml"), junk).unwrap();
    assert_set_aside_with_defaults(dir.path(), junk);
}

fn assert_set_aside_with_defaults(dir: &Path, junk: &[u8]) {
    let (store, notice) = open(dir);
    assert_eq!(store.get(), &Sections::default(), "defaults are in use");
    assert!(!store.is_read_only(), "the store saves over the defaults");
    let moved: Vec<_> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).filter(|p| p.file_name().unwrap() != "config.toml").collect();
    assert_eq!(moved.len(), 1, "the damaged file is set aside, not deleted: {moved:?}");
    assert_eq!(std::fs::read(&moved[0]).unwrap(), junk, "the set-aside copy keeps what was there");
    // What the notice says, and that the window announces it, is the app's test.
    assert!(notice.is_some(), "a notice is shown");
}

#[derive(Serialize, Deserialize, Default, PartialEq, Debug)]
#[serde(default)]
struct Window {
    maximised: bool,
    width: f64,
}

#[test]
fn a_section_that_does_not_fit_reads_as_defaults_with_one_notice_and_the_file_kept() {
    let dir = tempfile::tempdir().unwrap();
    let text = "[recent-paths]\npaths = [\"left.txt\"]\n\n[window]\nmaximised = true\nwidth = \"wide\"\n";
    std::fs::write(dir.path().join("config.toml"), text).unwrap();
    let (mut store, notice) = open(dir.path());
    assert_eq!(notice, None, "the file itself is fine");

    for _ in 0..3 {
        assert_eq!(store.section::<Window>("window"), Window::default(), "defaults are in use");
    }
    assert_eq!(store.section::<Recent>("recent-paths").paths, ["left.txt"], "the other sections are untouched");
    let notices = store.take_notices();
    assert_eq!(notices.len(), 1, "one notice, however often the section is read: {notices:?}");
    assert!(notices[0].contains("window"), "the notice names the section: {notices:?}");

    // The feature saves its defaults over the section, twice so `config.toml.bak` turns over too.
    for width in [800.0, 900.0] {
        store.update(|s| s.set("window", &Window { maximised: false, width }).unwrap());
        store.flush().unwrap();
    }
    let kept =
        std::fs::read_dir(dir.path()).unwrap().map(|e| e.unwrap().path()).filter(|p| std::fs::read_to_string(p).is_ok_and(|t| t == text)).count();
    assert_eq!(kept, 1, "what the user wrote is still on disk");
}

/// The section the writer child fills: a counter and many copies of it, so a torn write shows as
/// copies that disagree, and the file is big enough that a kill usually lands inside a write.
#[derive(Serialize, Deserialize, Default, PartialEq, Debug)]
#[serde(default)]
struct Load {
    count: u64,
    copies: Vec<u64>,
}

const WRITER_DIR: &str = "OTD_SETTINGS_TEST_WRITER_DIR";
const READY: &str = "writer-ready";

/// Not a test of its own: the body of the child process `killing_the_process_during_writes_…`
/// spawns. Without the variable it does nothing.
#[test]
fn writer_child() {
    let Some(dir) = std::env::var_os(WRITER_DIR) else { return };
    let dir = Path::new(&dir);
    let (mut store, _) = open(dir);
    for count in 1..=u64::MAX {
        store.update(|s| s.set("load", &Load { count, copies: vec![count; 20_000] }).unwrap());
        store.flush().unwrap();
        if count == 1 {
            std::fs::write(dir.join(READY), "").unwrap();
        }
    }
}

#[test]
fn killing_the_process_during_writes_leaves_the_previous_settings_readable() {
    let dir = tempfile::tempdir().unwrap();
    for round in 0..5u64 {
        let _ = std::fs::remove_file(dir.path().join(READY));
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "store::writer_child", "--nocapture", "--test-threads=1"])
            .env(WRITER_DIR, dir.path())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(60);
        while !dir.path().join(READY).exists() {
            assert!(Instant::now() < deadline, "the writer never got going");
            assert!(child.try_wait().unwrap().is_none(), "the writer exited early");
            std::thread::sleep(Duration::from_millis(5));
        }
        // Not asserted: varying the wait only spreads the kill over different points of a write.
        std::thread::sleep(Duration::from_millis(10 + 13 * round));
        child.kill().unwrap();
        child.wait().unwrap();

        let (store, notice) = open(dir.path());
        assert_eq!(notice, None, "round {round}: the file was neither damaged nor locked");
        let load: Load = store.get().get("load").unwrap();
        assert!(load.count >= 1, "round {round}: a completed save is there");
        assert!(load.copies.iter().all(|&c| c == load.count) && load.copies.len() == 20_000, "round {round}: one whole save, not a mix");
    }
}
