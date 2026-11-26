//! The Appearance page (E05-S1-T2): its theme choices, driven by keyboard and read back through the
//! accessibility tree, with the OS setting simulated, and the choice kept across a restart.

use std::collections::BTreeMap;

use egui::accesskit::Role;
use egui::{Key, Modifiers, Theme, ThemePreference};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_core::i18n::fmt;
use otd_settings::Location;
use otd_ui::colour::{self, CHROME, ColourRoles, Palette};
use otd_ui::theme::{self, DARK, DARK_THEME, FOLLOW_OS, FOLLOW_OS_SETTING, LIGHT, LIGHT_THEME, THEME};
use otd_ui::{OK, Shell, Value};

/// The app's start: the chrome roles installed, egui following the OS, which says `system`, and the
/// Options dialog opened by its command on the Appearance page, its only page.
fn appearance(saved: &str, system: Theme) -> Harness<'static, Shell> {
    let saved = BTreeMap::from([(THEME.to_owned(), Value::Choice(saved.to_owned()))]);
    let shell = Shell::new(Vec::new()).with_options(theme::options(), saved).unwrap();
    let mut harness = Harness::builder().build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), shell);
    harness.ctx.set_theme(ThemePreference::System);
    harness.input_mut().system_theme = Some(system);
    colour::install(&harness.ctx, ColourRoles::new(CHROME).expect("distinct ids"));
    harness.run();
    assert!(harness.state_mut().run("otd.options"));
    harness.run();
    harness
}

fn follow_os(setting: &str) -> String {
    fmt(FOLLOW_OS_SETTING, &[("setting", setting)])
}

fn focused(harness: &Harness<'_, Shell>) -> Option<(Role, String)> {
    let node = harness.query_by(|node| node.is_focused())?;
    let node = node.accesskit_node();
    Some((node.role(), node.label().unwrap_or_default()))
}

fn press(harness: &mut Harness<'_, Shell>, key: Key) {
    harness.key_press_modifiers(Modifiers::NONE, key);
    harness.run();
}

/// Tabs to `name` and activates it with `key`.
fn activate(harness: &mut Harness<'_, Shell>, role: Role, name: &str, key: Key) {
    for _ in 0..30 {
        if focused(harness).is_some_and(|(r, label)| r == role && label == name) {
            return press(harness, key);
        }
        press(harness, Key::Tab);
    }
    panic!("Tab never reached {name:?}");
}

#[test]
fn follow_operating_system_shows_the_os_setting_when_another_theme_is_chosen() {
    let mut harness = appearance(LIGHT_THEME, Theme::Dark);
    assert!(harness.query_by_role_and_label(Role::RadioButton, &follow_os(DARK)).is_some(), "the page opens showing the dark OS");

    harness.input_mut().system_theme = Some(Theme::Light);
    harness.run();
    assert!(harness.query_by_role_and_label(Role::RadioButton, &follow_os(LIGHT)).is_some(), "the label follows the OS setting");
}

#[test]
fn picking_dark_theme_by_keyboard_and_ok_darkens_the_window_under_a_light_os() {
    let mut harness = appearance(FOLLOW_OS, Theme::Light);
    assert_eq!(colour::palette(&harness.ctx), Palette::Light);
    activate(&mut harness, Role::RadioButton, DARK_THEME, Key::Space);
    activate(&mut harness, Role::Button, OK, Key::Enter);
    assert_eq!(colour::palette(&harness.ctx), Palette::Dark);
}

#[test]
fn every_theme_choice_persists_across_a_restart() {
    for choice in [FOLLOW_OS, LIGHT_THEME, DARK_THEME] {
        let dir = tempfile::tempdir().unwrap();
        let open = || otd_settings::open(Location::Dir(dir.path().to_path_buf())).0;
        let mut store = open();
        otd_settings::options::write(&mut store, &BTreeMap::from([(THEME.to_owned(), Value::Choice(choice.to_owned()))]));
        store.flush().unwrap();
        let saved = otd_settings::options::read(&mut open(), &theme::options());
        assert_eq!(saved.get(THEME), Some(&Value::Choice(choice.to_owned())), "{choice} reads back as itself, not the default");
    }
}
