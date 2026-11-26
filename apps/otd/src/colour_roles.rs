//! The colour roles views draw with, and which of them is drawn on which. A feature declares its
//! `otd_ui::colour::ColourRole` constants and its `Contrast` pairs, and adds them here, one line
//! each.

use otd_ui::colour::{CHROME, ColourRole, ColourRoles, DuplicateColourRole};

pub fn colour_roles() -> Result<ColourRoles, DuplicateColourRole> {
    let roles: Vec<ColourRole> = [CHROME.as_slice()].concat();
    ColourRoles::new(roles)
}

/// Every text and control role with the role it is drawn on. Only the contrast check below reads it.
#[cfg(test)]
fn contrast() -> Vec<otd_ui::colour::Contrast> {
    [otd_ui::colour::CHROME_CONTRAST.as_slice()].concat()
}

#[cfg(test)]
mod tests {
    #[test]
    fn no_two_registered_roles_share_an_id() {
        assert_eq!(super::colour_roles().err(), None);
    }

    #[test]
    fn every_registered_pair_is_readable_in_both_palettes() {
        let roles = super::colour_roles().expect("distinct ids");
        let low: Vec<String> = roles.low_contrast(&super::contrast()).iter().map(ToString::to_string).collect();
        assert!(low.is_empty(), "{low:#?}");
    }

    #[test]
    fn every_registered_role_is_in_a_contrast_pair() {
        let pairs = super::contrast();
        let roles = super::colour_roles().expect("distinct ids");
        let unpaired: Vec<&str> =
            roles.iter().map(|role| role.id).filter(|&id| !pairs.iter().any(|pair| pair.fore == id || pair.back == id)).collect();
        assert!(unpaired.is_empty(), "no contrast pair names {unpaired:?}");
    }
}
