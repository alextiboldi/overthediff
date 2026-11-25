//! The colour roles views draw with. A feature declares its `otd_ui::colour::ColourRole` constants
//! and adds them here, one line each.

use otd_ui::colour::{ColourRole, ColourRoles, DuplicateColourRole};

pub fn colour_roles() -> Result<ColourRoles, DuplicateColourRole> {
    let roles: Vec<ColourRole> = vec![
        // ponytail: no view draws with a role yet; the first views (E07-S1, E08-S1) add theirs.
    ];
    ColourRoles::new(roles)
}

#[cfg(test)]
mod tests {
    #[test]
    fn no_two_registered_roles_share_an_id() {
        assert_eq!(super::colour_roles().err(), None);
    }
}
