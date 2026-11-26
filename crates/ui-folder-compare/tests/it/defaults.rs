//! New Folder Compare sessions start from the stored Folder Compare defaults (E06-S2-T3), on a
//! settings tab a test harness contributes, so the test doesn't depend on which real settings exist.

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use otd_core::i18n::t;
use otd_launch::{Config, Launch, Registry, Requested};
use otd_settings::{Location, session_defaults};
use otd_ui::{Session, Setting, Shell, Value, View};
use otd_ui_folder_compare::{FOLDER_COMPARE, FolderCompare, ID, OpenFolders, session_type};

const TAB: &str = "Harness";
const FLAG: &str = "flag";

fn settings() -> Vec<Setting> {
    vec![Setting { session_type: ID, tab: TAB, id: FLAG, label: "Flag", default: Value::Bool(false), choices: &[] }]
}

fn active_session(harness: &mut Harness<'_, Shell>) -> Session {
    harness.state_mut().view_mut::<FolderCompare>().and_then(View::session).expect("a Folder Compare tab is active").clone()
}

#[test]
fn folder_compare_from_home_and_from_the_command_line_start_from_the_updated_defaults_with_their_own_specs() {
    let dir = tempfile::tempdir().unwrap();
    let store_at = || otd_settings::open(Location::Dir(dir.path().join("settings"))).0;
    // Updated defaults, from a session whose specs and description must not carry over.
    let mut updated = Session::new(ID);
    updated.fill_defaults(&settings());
    updated.rules.get_mut(TAB).unwrap().insert(FLAG.into(), Value::Bool(true));
    updated.specs = vec!["stored-left".into(), "stored-right".into()];
    updated.description = "stored description".into();
    let mut store = store_at();
    session_defaults::write(&mut store, &updated);
    store.flush().unwrap();
    drop(store);

    // A restart: the app loads the defaults into its shell.
    let mut store = store_at();
    let mut shell = Shell::new(vec![session_type()]).with_settings(settings()).unwrap();
    shell.load_session_defaults(&mut store);
    let mut harness = Harness::builder().build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), shell);
    harness.run();

    // Home's sessions tree opens a row with Enter.
    harness.get_by_role_and_label(Role::TreeItem, t(FOLDER_COMPARE)).focus();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    let home = active_session(&mut harness);

    for side in ["left", "right"] {
        std::fs::create_dir(dir.path().join(side)).unwrap();
    }
    let mut registry = Registry::new();
    registry.register(OpenFolders);
    let config = Config { app_version: "1.0.0".into(), runtime: None, spawn_instance: Box::new(|| Ok(())) };
    // Absolute, since the launch checks for folders against the process's own working folder.
    let args = ["left", "right"].map(|side| dir.path().join(side).to_string_lossy().into_owned());
    // A new instance of its own, so the test reaches no running app.
    let Launch::Instance(instance) = otd_launch::launch(&registry, &args, Requested::NewInstance, &config) else { panic!("the launch is refused") };
    registry.open(instance.try_next().unwrap(), harness.state_mut()).unwrap();
    harness.run();
    let command_line = active_session(&mut harness);

    let flag = Some(&Value::Bool(true));
    assert_ne!(flag, Some(&settings()[0].default), "the updated default differs from the setting's own");
    for (session, specs) in [(&home, vec![String::new(); 2]), (&command_line, args.to_vec())] {
        assert_eq!(session.rule(TAB, FLAG), flag, "the new session has the updated default");
        assert_eq!(session.specs, specs, "specs come only from the session's own inputs");
        assert_eq!(session.description, "", "the description is not a default");
    }

    // Defaults updated while the app runs, not yet saved to disk, reach the next new session too.
    updated.rules.get_mut(TAB).unwrap().insert(FLAG.into(), Value::Bool(false));
    session_defaults::write(&mut store, &updated);
    harness.state_mut().load_session_defaults(&mut store);
    harness.state_mut().new_session(ID).unwrap();
    harness.run();
    assert_ne!(active_session(&mut harness).rule(TAB, FLAG), flag, "the next new session has the newer default");
}
