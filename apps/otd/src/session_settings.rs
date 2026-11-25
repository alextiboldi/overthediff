//! The options features contribute to Session Settings tabs. A feature adds its `otd_ui::Setting`s
//! here, one line each (`settings.extend(feature::session_settings())`). Settings naming the same
//! session type and tab share that tab.

use otd_ui::Setting;

pub fn session_settings() -> Vec<Setting> {
    // ponytail: no feature contributes session settings yet; the first session types (E07-S1,
    // E08-S1) add theirs.
    Vec::new()
}
