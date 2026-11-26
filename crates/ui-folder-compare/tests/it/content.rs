//! The content options on the Comparison tab (E08-S5-T1), set as a user sets them: in Comparison
//! Settings, then OK.

use std::fs;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use otd_folder_engine::{ContentTest, Status};
use otd_ui::{OK, Shell, View as _};
use otd_ui_folder_compare::{COMPARE_CONTENTS, COMPARISON, CONTENT_NOT_RUN, FolderCompare, commands, session_settings, session_type};

use crate::view::{fixture, settle};

fn same_txt(harness: &Harness<'_, Shell>) -> Status {
    let view = harness.state().view::<FolderCompare>().expect("a Folder Compare tab");
    view.shown().iter().find(|shown| shown.row.name == "same.txt").expect("same.txt has a row").row.status.clone()
}

fn says_not_run(harness: &Harness<'_, Shell>) -> bool {
    let view = harness.state().view::<FolderCompare>().expect("a Folder Compare tab");
    view.status().fields.iter().any(|field| field.value == CONTENT_NOT_RUN)
}

fn click(harness: &mut Harness<'_, Shell>, role: Role, label: &str) {
    harness.get_by(|node| node.role() == role && node.label().as_deref() == Some(label)).click();
    harness.run();
}

#[test]
fn turning_on_compare_contents_keeps_it_in_the_session_and_compares_again() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    let shell = Shell::new(vec![session_type()]).with_commands(commands()).with_settings(session_settings()).unwrap();
    let mut harness = Harness::builder().build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), shell);
    harness.state_mut().open(Box::new(FolderCompare::of_folders(&["left".into(), "right".into()], root.path())));
    settle(&mut harness);
    assert_eq!(same_txt(&harness), Status::Same);
    let view = |harness: &Harness<'_, Shell>| harness.state().view::<FolderCompare>().unwrap().content_options();
    assert_eq!(view(&harness).test, None, "off until turned on");
    assert!(!says_not_run(&harness), "no note while Compare contents is off");

    // Longer, so its size differs: only a comparison that lists it again sees the change.
    fs::write(root.path().join("right").join("same.txt"), "no longer the same").unwrap();
    harness.state_mut().session_settings().unwrap();
    harness.run();
    click(&mut harness, Role::Tab, COMPARISON);
    click(&mut harness, Role::CheckBox, COMPARE_CONTENTS);
    click(&mut harness, Role::Button, OK);
    settle(&mut harness);

    assert_eq!(view(&harness).test, Some(ContentTest::Crc), "the session keeps the option, by CRC until Binary is chosen");
    assert_ne!(same_txt(&harness), Status::Same, "the change compared the folders again");
    // Until E08-S5-T2 runs content tests, the status bar says they don't (rule 15).
    assert!(says_not_run(&harness), "the status bar says content tests don't run yet");
}
