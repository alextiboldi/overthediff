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

/// A text field and a checkbox, neither hovered nor focused.
#[derive(Default)]
struct Controls {
    text: String,
    checked: bool,
}

impl View for Controls {
    fn title(&self) -> String {
        "Controls".to_owned()
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let label = ui.label("Name");
        ui.text_edit_singleline(&mut self.text).labelled_by(label.id);
        ui.checkbox(&mut self.checked, "Check");
    }
}

#[test]
fn a_text_field_and_a_checkbox_at_rest_are_outlined_with_the_border_role() {
    let mut harness = Harness::builder().build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), Shell::new(Vec::new()));
    colour::install(&harness.ctx, ColourRoles::new(CHROME).expect("distinct ids"));
    harness.state_mut().open(Box::new(Controls::default()));
    for palette in [Palette::Light, Palette::Dark] {
        colour::set_palette(&harness.ctx, palette);
        harness.run();
        let border = colour32(colour::BORDER.value(palette));
        let outlined: Vec<Color32> = harness
            .output()
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(rect) if rect.stroke.width > 0.0 && rect.stroke.color == border => Some(rect.fill),
                _ => None,
            })
            .collect();
        for role in [colour::INPUT, colour::CONTROL] {
            assert!(outlined.contains(&colour32(role.value(palette))), "{} in {palette:?}: {outlined:?}", role.id);
        }
    }
}

/// A shell following the OS, with the OS reporting `system`.
fn following(system: Theme) -> Harness<'static, Shell> {
    let mut harness = harness(&[SWATCH], SWATCH);
    harness.ctx.set_theme(ThemePreference::System);
    harness.input_mut().system_theme = Some(system);
    colour::follow_system(&harness.ctx);
    harness.run();
    harness
}

fn palette_of(theme: Theme) -> Palette {
    match theme {
        Theme::Light => Palette::Light,
        Theme::Dark => Palette::Dark,
    }
}

/// The open view and the chrome are in `palette` and not in the other one.
fn assert_drawn_in(harness: &Harness<'_, Shell>, palette: Palette) {
    let other = match palette {
        Palette::Light => Palette::Dark,
        Palette::Dark => Palette::Light,
    };
    let fills = fills(harness);
    for role in CHROME_FILLS.iter().chain([&SWATCH]) {
        assert!(fills.contains(&colour32(role.value(palette))), "{} in {palette:?}: {fills:?}", role.id);
        assert!(!fills.contains(&colour32(role.value(other))), "{} in {palette:?}: {fills:?}", role.id);
    }
}

#[test]
fn following_the_os_recolours_an_open_view_and_the_chrome_each_time_the_os_setting_changes() {
    for start in [Theme::Light, Theme::Dark] {
        let mut harness = following(start);
        assert_drawn_in(&harness, palette_of(start));
        let flipped = match start {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        };
        for system in [flipped, start] {
            harness.input_mut().system_theme = Some(system);
            harness.run();
            assert_drawn_in(&harness, palette_of(system));
        }
    }
}

#[test]
fn choosing_a_palette_stops_following_the_os() {
    let mut harness = following(Theme::Dark);
    colour::set_palette(&harness.ctx, Palette::Light);
    harness.run();
    harness.input_mut().system_theme = Some(Theme::Dark);
    harness.run();
    assert_drawn_in(&harness, Palette::Light);
}
