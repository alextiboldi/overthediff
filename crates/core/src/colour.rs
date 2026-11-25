//! Colour roles (E01-S1): the colours views draw with, each with a light and a dark value. A feature
//! declares its roles as constants and adds them to the app's list (`apps/otd/src/colour_roles.rs`);
//! views draw with `otd_ui::colour::colour`, never with fixed colours. The active [`Palette`] picks
//! each role's light or dark value, and is light until a theme choice exists (E05-S1).

use std::collections::BTreeMap;
use std::fmt;

/// An sRGB colour with straight (unmultiplied) alpha.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    /// An opaque colour.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: u8::MAX }
    }
}

/// Which of each role's values applies.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Palette {
    #[default]
    Light,
    Dark,
}

/// A named colour a feature draws with, such as a status colour or a line background.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColourRole {
    /// A stable id, unique among roles, such as `text.line.added`.
    pub id: &'static str,
    pub light: Rgba,
    pub dark: Rgba,
}

impl ColourRole {
    pub fn value(&self, palette: Palette) -> Rgba {
        match palette {
            Palette::Light => self.light,
            Palette::Dark => self.dark,
        }
    }
}

/// The registered colour roles, by id.
#[derive(Clone, Debug, Default)]
pub struct ColourRoles {
    roles: BTreeMap<&'static str, ColourRole>,
}

impl ColourRoles {
    /// Registers `roles`. Two roles with one id are an error, whatever their values.
    pub fn new(roles: impl IntoIterator<Item = ColourRole>) -> Result<Self, DuplicateColourRole> {
        let mut table = BTreeMap::new();
        for role in roles {
            if table.insert(role.id, role).is_some() {
                return Err(DuplicateColourRole(role.id));
            }
        }
        Ok(Self { roles: table })
    }

    /// The registered role with this id.
    pub fn get(&self, id: &str) -> Option<&ColourRole> {
        self.roles.get(id)
    }

    /// Every registered role, in id order.
    pub fn iter(&self) -> impl Iterator<Item = &ColourRole> {
        self.roles.values()
    }
}

/// Two colour roles were registered with this id.
#[derive(Debug, PartialEq, Eq)]
pub struct DuplicateColourRole(pub &'static str);

impl fmt::Display for DuplicateColourRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the colour role {:?} is registered twice", self.0)
    }
}

impl std::error::Error for DuplicateColourRole {}

#[cfg(test)]
mod tests {
    use super::*;

    const fn role(id: &'static str, grey: u8) -> ColourRole {
        ColourRole { id, light: Rgba::rgb(grey, grey, grey), dark: Rgba::rgb(0, 0, 0) }
    }

    #[test]
    fn a_second_role_with_the_same_id_is_rejected_by_name() {
        let roles = [role("a", 1), role("b", 2), role("a", 3)];
        assert_eq!(ColourRoles::new(roles).err(), Some(DuplicateColourRole("a")));
    }
}
