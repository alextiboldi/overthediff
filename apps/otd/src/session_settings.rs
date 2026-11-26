//! The options features contribute to Session Settings tabs. A feature adds its `otd_ui::Setting`s
//! here, one line each (`settings.extend(feature::session_settings())`). Settings naming the same
//! session type and tab share that tab; two with one id there fail the test below and the app's start.

use otd_ui::Setting;

pub fn session_settings() -> Vec<Setting> {
    // ponytail: no feature contributes session settings yet; the first session types (E07-S1,
    // E08-S1) add theirs.
    Vec::new()
}

#[cfg(test)]
mod tests {
    #[test]
    fn no_two_contributed_settings_share_an_id_on_one_tab() {
        let shell = otd_ui::Shell::new(crate::session_types::session_types()).with_settings(super::session_settings());
        assert_eq!(shell.err(), None);
    }
}
