//! The settings reader's hostile-input sweep: whatever is in `config.toml`, opening the store and
//! reading its sections never panics or hangs.

use craft_testkit::hostile::{HammerConfig, hammer};
use otd_settings::Location;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct Recent {
    paths: Vec<String>,
    limit: u32,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct Window {
    maximised: bool,
    width: f64,
    title: Option<String>,
}

#[test]
fn opening_and_reading_a_damaged_settings_file_never_panics_or_hangs() {
    let seed = "schema_version = 1\n\n[recent-paths]\npaths = [\"a.txt\", \"dir/b.txt\"]\nlimit = 8\n\n[window]\nmaximised = true\nwidth = 1024.5\ntitle = \"x\"\n";
    hammer(&HammerConfig::new("settings file", 300), seed.as_bytes(), |data| {
        let Ok(dir) = tempfile::tempdir() else { return };
        if std::fs::write(dir.path().join("config.toml"), data).is_err() {
            return;
        }
        let (mut store, _notice) = otd_settings::open(Location::Dir(dir.path().to_path_buf()));
        let _: Recent = store.section("recent-paths");
        let _: Window = store.section("window");
    });
}
