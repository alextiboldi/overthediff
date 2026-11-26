//! Colour roles (E01-S1): the colours views draw with, each with a light and a dark value. A feature
//! declares its roles as constants and adds them to the app's list (`apps/otd/src/colour_roles.rs`);
//! views draw with `otd_ui::colour::colour`, never with fixed colours. The active [`Palette`] picks
//! each role's light or dark value; E05-S1's theme sets it.
//!
//! Text is readable only against what it is drawn on, so a feature also declares which of its
//! roles is drawn on which as [`Contrast`] pairs, and [`ColourRoles::low_contrast`] reports every
//! pair below its minimum in either palette (E05-S1).

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

/// One role drawn on another, and the contrast ratio the pair needs in both palettes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Contrast {
    /// The id of the role drawn on top: text, a boundary or a focus ring.
    pub fore: &'static str,
    /// The id of the role it is drawn on.
    pub back: &'static str,
    /// The lowest acceptable ratio, from 1 to 21.
    pub min: f64,
}

impl Contrast {
    /// Text on a background, which needs 4.5:1 (WCAG 1.4.3).
    pub const fn text(fore: &ColourRole, back: &ColourRole) -> Self {
        Self { fore: fore.id, back: back.id, min: 4.5 }
    }

    /// A control's boundary or a focus indicator next to a background, which needs 3:1 (WCAG
    /// 1.4.11).
    pub const fn control(fore: &ColourRole, back: &ColourRole) -> Self {
        Self { fore: fore.id, back: back.id, min: 3.0 }
    }
}

/// A [`Contrast`] pair that fails in one palette.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LowContrast {
    pub pair: Contrast,
    pub palette: Palette,
    pub problem: ContrastProblem,
}

/// Why a [`Contrast`] pair fails.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ContrastProblem {
    /// The ratio measured, below the pair's minimum.
    Ratio(f64),
    /// No role with this id is registered, so the pair can't be measured.
    Unregistered(&'static str),
    /// The background isn't opaque, so its contrast depends on what shows through it.
    TranslucentBack,
}

impl fmt::Display for LowContrast {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Contrast { fore, back, min } = self.pair;
        write!(f, "{fore:?} on {back:?} in the {:?} palette: ", self.palette)?;
        match self.problem {
            ContrastProblem::Ratio(ratio) => write!(f, "contrast {ratio:.2}:1, below {min}:1"),
            ContrastProblem::Unregistered(id) => write!(f, "{id:?} isn't registered"),
            ContrastProblem::TranslucentBack => write!(f, "the background isn't opaque"),
        }
    }
}

/// The WCAG 2 contrast ratio of `fore` drawn over the opaque `back`, from 1 to 21. A translucent
/// `fore` is blended over `back` first.
pub fn contrast_ratio(fore: Rgba, back: Rgba) -> f64 {
    let alpha = f64::from(fore.a) / 255.0;
    let blend = |f: u8, b: u8| alpha * f64::from(f) + (1.0 - alpha) * f64::from(b);
    let luminance = |r: f64, g: f64, b: f64| {
        let linear = |c: f64| {
            let c = c / 255.0;
            if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
        };
        0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b)
    };
    let fore = luminance(blend(fore.r, back.r), blend(fore.g, back.g), blend(fore.b, back.b));
    let back = luminance(f64::from(back.r), f64::from(back.g), f64::from(back.b));
    (fore.max(back) + 0.05) / (fore.min(back) + 0.05)
}

impl ColourRoles {
    /// Every pair in `pairs` that falls below its minimum, or can't be measured, in either palette.
    pub fn low_contrast(&self, pairs: &[Contrast]) -> Vec<LowContrast> {
        let mut found = Vec::new();
        for &pair in pairs {
            for palette in [Palette::Light, Palette::Dark] {
                let problem = match (self.get(pair.fore), self.get(pair.back)) {
                    (None, _) => Some(ContrastProblem::Unregistered(pair.fore)),
                    (_, None) => Some(ContrastProblem::Unregistered(pair.back)),
                    (Some(fore), Some(back)) => {
                        let (fore, back) = (fore.value(palette), back.value(palette));
                        let ratio = contrast_ratio(fore, back);
                        if back.a != u8::MAX {
                            Some(ContrastProblem::TranslucentBack)
                        } else if ratio < pair.min {
                            Some(ContrastProblem::Ratio(ratio))
                        } else {
                            None
                        }
                    }
                };
                found.extend(problem.map(|problem| LowContrast { pair, palette, problem }));
            }
        }
        found
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

    #[test]
    fn the_contrast_ratio_follows_wcag_at_the_text_threshold() {
        let white = Rgba::rgb(0xFF, 0xFF, 0xFF);
        // The lightest greys that still reach, and no longer reach, 4.5:1 on white.
        assert!(contrast_ratio(Rgba::rgb(0x76, 0x76, 0x76), white) >= 4.5);
        assert!(contrast_ratio(Rgba::rgb(0x77, 0x77, 0x77), white) < 4.5);
    }

    #[test]
    fn a_translucent_fore_is_measured_blended_over_its_back() {
        let (black, white) = (Rgba::rgb(0, 0, 0), Rgba::rgb(0xFF, 0xFF, 0xFF));
        let faint = Rgba { a: 0x40, ..black };
        assert!(contrast_ratio(faint, white) < contrast_ratio(black, white));
    }

    const BACK: ColourRole = ColourRole { id: "test.back", light: Rgba::rgb(0xFF, 0xFF, 0xFF), dark: Rgba::rgb(0x20, 0x20, 0x20) };
    const READABLE: ColourRole = ColourRole { id: "test.readable", light: Rgba::rgb(0x20, 0x20, 0x20), dark: Rgba::rgb(0xE0, 0xE0, 0xE0) };
    /// Readable in the light palette, but barely different from its background in the dark one.
    const POOR_IN_DARK: ColourRole = ColourRole { id: "test.poor", light: Rgba::rgb(0x20, 0x20, 0x20), dark: Rgba::rgb(0x40, 0x40, 0x40) };

    #[test]
    fn only_a_pair_below_its_minimum_is_reported_and_only_in_that_palette() {
        let roles = ColourRoles::new([BACK, READABLE, POOR_IN_DARK]).expect("distinct ids");
        let found = roles.low_contrast(&[Contrast::text(&READABLE, &BACK), Contrast::text(&POOR_IN_DARK, &BACK)]);
        let reported: Vec<_> = found.iter().map(|low| (low.pair.fore, low.palette)).collect();
        assert_eq!(reported, [(POOR_IN_DARK.id, Palette::Dark)], "{found:?}");
    }

    #[test]
    fn a_pair_naming_an_unregistered_role_is_reported_rather_than_skipped() {
        let roles = ColourRoles::new([READABLE]).expect("distinct ids");
        let found = roles.low_contrast(&[Contrast::text(&READABLE, &BACK)]);
        assert!(found.iter().all(|low| low.problem == ContrastProblem::Unregistered(BACK.id)), "{found:?}");
        assert!(!found.is_empty());
    }
}
