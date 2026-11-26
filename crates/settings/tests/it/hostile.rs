//! The settings reader's hostile-input sweep: whatever is in `config.toml`, opening the store and
//! reading its sections, the Options dialog's values and the session defaults among them, never
//! panics or hangs.

use craft_testkit::hostile::{HammerConfig, hammer};
use otd_core::options::{OptionSpec, Value};
use otd_core::session::Setting;
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
    let seed = "schema_version = 1\n\n[recent-paths]\npaths = [\"a.txt\", \"dir/b.txt\"]\nlimit = 8\n\n[window]\nmaximised = true\nwidth = 1024.5\ntitle = \"x\"\n\n[options]\nwrap = true\nwidth = 4.5\nunit = \"Words\"\naccent = \"#0a141eff\"\n\n[session-defaults.folder.Rules]\nwrap = true\nunit = \"Words\"\n";
    let options = [
        OptionSpec { page: &["P"], id: "wrap", label: "Wrap", default: Value::Bool(false), choices: &[] },
        OptionSpec { page: &["P"], id: "width", label: "Width", default: Value::Number(4.0), choices: &[] },
        OptionSpec { page: &["P"], id: "unit", label: "Unit", default: Value::Choice("Lines".into()), choices: &["Lines", "Words"] },
        OptionSpec { page: &["P"], id: "accent", label: "Accent", default: Value::Colour([0; 4]), choices: &[] },
    ];
    let settings = [
        Setting { session_type: "folder", tab: "Rules", id: "wrap", label: "Wrap", default: Value::Bool(false), choices: &[] },
        Setting {
            session_type: "folder",
            tab: "Rules",
            id: "unit",
            label: "Unit",
            default: Value::Choice("Lines".into()),
            choices: &["Lines", "Words"],
        },
    ];
    hammer(&HammerConfig::new("settings file", 300), seed.as_bytes(), move |data| {
        let Ok(dir) = tempfile::tempdir() else { return };
        if std::fs::write(dir.path().join("config.toml"), data).is_err() {
            return;
        }
        let (mut store, _notice) = otd_settings::open(Location::Dir(dir.path().to_path_buf()));
        let _: Recent = store.section("recent-paths");
        let _: Window = store.section("window");
        let _ = otd_settings::options::read(&mut store, &options);
        let _ = otd_settings::session_defaults::read(&mut store, &settings, "folder");
    });
}

#[test]
fn loading_a_damaged_session_file_never_panics_or_hangs() {
    use otd_core::session::Setting;
    let seed = "name = \"x\"\ntype = \"t\"\nspecs = [\"/a\", \"/b\"]\ndescription = \"d\"\n\n[settings.Rules]\nwrap = true\nwidth = 4.5\nunit = \"Words\"\naccent = \"#0a141eff\"\n";
    let settings: Vec<Setting> =
        [("wrap", Value::Bool(false)), ("width", Value::Number(4.0)), ("unit", Value::Choice("Lines".into())), ("accent", Value::Colour([0; 4]))]
            .into_iter()
            .map(|(id, default)| Setting { session_type: "t", tab: "Rules", id, label: id, default, choices: &["Lines", "Words"] })
            .collect();
    hammer(&HammerConfig::new("session file", 300), seed.as_bytes(), move |data| {
        let Ok(dir) = tempfile::tempdir() else { return };
        let folder = dir.path().join(otd_settings::sessions::FOLDER);
        if std::fs::create_dir_all(&folder).and_then(|()| std::fs::write(folder.join("s-x.toml"), data)).is_err() {
            return;
        }
        let (store, _notice) = otd_settings::open(Location::Dir(dir.path().to_path_buf()));
        let _ = otd_settings::sessions::load(&store, &settings);
    });
}
