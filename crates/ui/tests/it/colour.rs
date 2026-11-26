//! A view in a shell tab drawing with a colour role, and the shell's chrome and selection, read back
//! from the frame's painted shapes.

use egui::{Color32, Sense, Theme, ThemePreference, Vec2};
use egui_kittest::Harness;
use otd_ui::colour::{self, CHROME, ColourRole, ColourRoles, Palette, Rgba, UNREGISTERED};
use otd_ui::{Shell, View};

const SWATCH: ColourRole = ColourRole { id: "test.swatch", light: Rgba::rgb(0xE8, 0xD0, 0xB0), dark: Rgba::rgb(0x30, 0x20, 0x10) };

/// Fills a rectangle with one role.
struct Swatch(ColourRole);

impl View for Swatch {
    fn title(&self) -> String {
        "Swatch".to_owned()
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(40.0, 20.0), Sense::hover());
        ui.painter().rect_filled(rect, 0.0, colour::colour(ui.ctx(), &self.0));
    }
}

/// A shell showing a [`Swatch`] of `drawn`, with the chrome roles and `registered` as the registered
/// roles.
fn harness(registered: &[ColourRole], drawn: ColourRole) -> Harness<'static, Shell> {
    let mut harness = Harness::builder().build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), Shell::new(Vec::new()));
    let roles = ColourRoles::new(CHROME.iter().chain(registered).copied()).expect("distinct ids");
    colour::install(&harness.ctx, roles);
    harness.state_mut().open(Box::new(Swatch(drawn)));
    harness.run();
    harness
}

fn colour32(Rgba { r, g, b, a }: Rgba) -> Color32 {
    Color32::from_rgba_unmultiplied(r, g, b, a)
}

/// The fill of every rectangle painted in the last frame.
fn fills(harness: &Harness<'_, Shell>) -> Vec<Color32> {
    harness
        .output()
        .shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Rect(rect) => Some(rect.fill),
            _ => None,
        })
        .collect()
}

#[test]
fn a_view_drawing_with_a_registered_role_shows_its_light_value_by_default() {
    let harness = harness(&[SWATCH], SWATCH);
    let fills = fills(&harness);
    assert!(fills.contains(&colour32(SWATCH.light)), "{fills:?}");
    assert!(!fills.contains(&colour32(SWATCH.dark)), "{fills:?}");
}

#[test]
fn switching_the_palette_recolours_an_open_view() {
    let mut harness = harness(&[SWATCH], SWATCH);
    colour::set_palette(&harness.ctx, Palette::Dark);
    harness.run();
    let fills = fills(&harness);
    assert!(fills.contains(&colour32(SWATCH.dark)), "{fills:?}");
    assert!(!fills.contains(&colour32(SWATCH.light)), "{fills:?}");
}

#[test]
fn a_role_nobody_registered_draws_as_unregistered_rather_than_its_own_values() {
    let harness = harness(&[], SWATCH);
    let fills = fills(&harness);
    assert!(fills.contains(&UNREGISTERED), "{fills:?}");
    assert!(!fills.contains(&colour32(SWATCH.light)), "{fills:?}");
}

/// The chrome background behind the tab strip, and the selection behind the selected (Home) tab.
const CHROME_FILLS: [ColourRole; 2] = [colour::BACKGROUND, colour::SELECTION];

#[test]
fn the_chrome_and_the_selection_take_the_active_palettes_values() {
    let mut harness = harness(&[], SWATCH);
    for palette in [Palette::Light, Palette::Dark] {
        colour::set_palette(&harness.ctx, palette);
        harness.run();
        let fills = fills(&harness);
        for role in CHROME_FILLS {
            assert!(fills.contains(&colour32(role.value(palette))), "{} in {palette:?}: {fills:?}", role.id);
        }
    }
}

#[test]
fn an_os_in_dark_mode_leaves_the_chrome_in_the_light_palette() {
    let mut harness = Harness::builder().build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), Shell::new(Vec::new()));
    // As in the app, egui follows the OS, and the OS says dark.
    harness.ctx.set_theme(ThemePreference::System);
    harness.input_mut().system_theme = Some(Theme::Dark);
    colour::install(&harness.ctx, ColourRoles::new(CHROME).expect("distinct ids"));
    harness.run();
    let fills = fills(&harness);
    for role in CHROME_FILLS {
        assert!(fills.contains(&colour32(role.light)), "{}: {fills:?}", role.id);
        assert!(!fills.contains(&colour32(role.dark)), "{}: {fills:?}", role.id);
    }
}
