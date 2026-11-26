//! The theme choice (E05-S1): the Appearance page of the Options dialog offers Follow operating
//! system, Light theme and Dark theme, and the shell makes the chosen palette active every frame.
//! Follow operating system shows the OS setting in parentheses and takes its palette from it.

use egui::{Context, Theme, Ui};
use otd_core::i18n::{fmt, t};
use otd_core::session::Value;

use crate::colour::{self, Palette};
use crate::options::{OptionSpec, option};

// English keys, drawn only through `t`.
pub const APPEARANCE: &str = "Appearance";
pub const THEME_LABEL: &str = "Theme";
pub const FOLLOW_OS: &str = "Follow operating system";
pub const LIGHT_THEME: &str = "Light theme";
pub const DARK_THEME: &str = "Dark theme";
/// Follow operating system's caption when the OS reports its setting; `{setting}` is [`LIGHT`] or [`DARK`].
pub const FOLLOW_OS_SETTING: &str = "Follow operating system ({setting})";
pub const LIGHT: &str = "Light";
pub const DARK: &str = "Dark";

/// The theme option's id: the settings store keeps the chosen choice, in English, under it.
pub const THEME: &str = "appearance.theme";

/// The theme option, for the app's option list.
pub fn options() -> Vec<OptionSpec> {
    vec![OptionSpec {
        page: &[APPEARANCE],
        id: THEME,
        label: THEME_LABEL,
        default: Value::Choice(FOLLOW_OS.to_owned()),
        choices: &[FOLLOW_OS, LIGHT_THEME, DARK_THEME],
    }]
}

/// The palette `choice` stands for while the OS reports `system`. With no OS setting, following it
/// is light, egui's own fallback.
pub fn palette_for(choice: &str, system: Option<Theme>) -> Palette {
    match (choice, system) {
        (DARK_THEME, _) | (FOLLOW_OS, Some(Theme::Dark)) => Palette::Dark,
        _ => Palette::Light,
    }
}

/// Makes the applied theme choice's palette active. Does nothing while no theme option is
/// contributed, so a harness can set the palette itself.
pub(crate) fn apply(ctx: &Context) {
    let Some(Value::Choice(choice)) = option(ctx, THEME) else { return };
    let palette = palette_for(&choice, ctx.system_theme());
    // Only on a change: setting it requests a repaint.
    if palette != colour::palette(ctx) {
        colour::set_palette(ctx, palette);
    }
}

/// `choice`'s caption, Follow operating system with the OS setting when the OS reports one.
fn caption(ctx: &Context, choice: &str) -> String {
    let setting = match ctx.system_theme() {
        Some(Theme::Light) => LIGHT,
        Some(Theme::Dark) => DARK,
        None => return t(choice).to_owned(),
    };
    if choice == FOLLOW_OS { fmt(t(FOLLOW_OS_SETTING), &[("setting", t(setting))]) } else { t(choice).to_owned() }
}

/// The theme option on its page: its label and a radio button per choice, all visible, so Follow
/// operating system's OS setting shows whatever is chosen.
pub(crate) fn edit(ui: &mut Ui, spec: &OptionSpec, choice: &mut String) {
    ui.label(t(spec.label));
    for &option in spec.choices {
        if ui.radio(choice == option, caption(ui.ctx(), option)).clicked() {
            *choice = option.to_owned();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_and_dark_theme_ignore_the_os_and_follow_takes_it() {
        for system in [Some(Theme::Light), Some(Theme::Dark), None] {
            assert_eq!(palette_for(LIGHT_THEME, system), Palette::Light, "{system:?}");
            assert_eq!(palette_for(DARK_THEME, system), Palette::Dark, "{system:?}");
        }
        assert_eq!(palette_for(FOLLOW_OS, Some(Theme::Dark)), Palette::Dark);
        assert_eq!(palette_for(FOLLOW_OS, Some(Theme::Light)), Palette::Light);
    }
}
