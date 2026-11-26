//! The session types Home offers, one button each, in this order. A feature adds its
//! `otd_ui::SessionType` here, one line each.

use otd_ui::SessionType;

pub fn session_types() -> Vec<SessionType> {
    vec![otd_ui_text_compare::SESSION_TYPE]
}
