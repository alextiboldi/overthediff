//! Drawing with colour roles (E01-S1). The app installs the registered [`ColourRoles`] once; a view
//! draws with [`colour`], which gives a role's value in the active [`Palette`]. The table and the
//! palette live in the egui context, so any view or painter reaches them without the shell.
//!
//! The window chrome and the selection draw with the [`CHROME`] roles too (E05-S1): [`install`] and
//! [`set_palette`] rebuild egui's visuals from them, for both of egui's styles, so the OS setting
//! alone never swaps them for egui's own.
//!
//! [`follow_system`] makes the palette the OS light or dark setting's instead, at start-up and
//! whenever the setting changes while the app runs (E05-S1-T3).

use std::sync::Arc;

use egui::{Color32, Context, Id, Stroke, Theme, Visuals};
pub use otd_core::colour::{ColourRole, ColourRoles, Contrast, ContrastProblem, DuplicateColourRole, LowContrast, Palette, Rgba, contrast_ratio};

const ROLES: &str = "otd-colour-roles";
const PALETTE: &str = "otd-colour-palette";
const FOLLOW: &str = "otd-colour-follow-system";

/// What a role that was never registered draws with: a colour no palette would choose, so the
/// mistake shows on screen.
pub const UNREGISTERED: Color32 = Color32::from_rgb(0xFF, 0x00, 0xFF);

const fn role(id: &'static str, light: Rgba, dark: Rgba) -> ColourRole {
    ColourRole { id, light, dark }
}

const fn grey(level: u8) -> Rgba {
    Rgba::rgb(level, level, level)
}

/// Panels: the tab strip, the status bar and behind every view.
pub const BACKGROUND: ColourRole = role("chrome.background", grey(0xF3), grey(0x1E));
/// Dialogs, menus and popups.
pub const WINDOW: ColourRole = role("chrome.window", grey(0xFF), grey(0x26));
/// Labels and control captions.
pub const TEXT: ColourRole = role("chrome.text", grey(0x1F), grey(0xE4));
/// A button or other control at rest.
pub const CONTROL: ColourRole = role("chrome.control", grey(0xE3), grey(0x3A));
/// A control under the pointer, pressed or focused.
pub const CONTROL_ACTIVE: ColourRole = role("chrome.control.active", grey(0xD4), grey(0x4A));
/// Text fields.
pub const INPUT: ColourRole = role("chrome.input", grey(0xFF), grey(0x14));
/// Control, window and separator borders.
pub const BORDER: ColourRole = role("chrome.border", grey(0x76), grey(0x8C));
/// Behind selected items and selected text.
pub const SELECTION: ColourRole = role("selection.background", Rgba::rgb(0xB9, 0xD8, 0xFF), Rgba::rgb(0x1F, 0x4E, 0x7A));
/// Selected items and selected text, and the focus ring.
pub const SELECTION_TEXT: ColourRole = role("selection.text", Rgba::rgb(0x00, 0x3A, 0x70), Rgba::rgb(0xE8, 0xF1, 0xFF));

/// The chrome and selection roles, which the app registers with every feature's.
pub const CHROME: [ColourRole; 9] = [BACKGROUND, WINDOW, TEXT, CONTROL, CONTROL_ACTIVE, INPUT, BORDER, SELECTION, SELECTION_TEXT];

/// What the chrome and selection roles are drawn on, for the contrast check.
pub const CHROME_CONTRAST: [Contrast; 11] = [
    Contrast::text(&TEXT, &BACKGROUND),
    Contrast::text(&TEXT, &WINDOW),
    Contrast::text(&TEXT, &CONTROL),
    Contrast::text(&TEXT, &CONTROL_ACTIVE),
    Contrast::text(&TEXT, &INPUT),
    Contrast::text(&SELECTION_TEXT, &SELECTION),
    // Every control's edge, in every state, on the panel or window it sits on.
    Contrast::control(&BORDER, &BACKGROUND),
    Contrast::control(&BORDER, &WINDOW),
    // A control's edge on a selected row, and the focus ring there.
    Contrast::control(&SELECTION_TEXT, &SELECTION),
    // The focus ring.
    Contrast::control(&SELECTION_TEXT, &BACKGROUND),
    Contrast::control(&SELECTION_TEXT, &WINDOW),
];

/// Makes `roles` the registered roles that [`colour`] resolves, and restyles the chrome from them.
pub fn install(ctx: &Context, roles: ColourRoles) {
    ctx.data_mut(|data| data.insert_temp(Id::unique(ROLES), Arc::new(roles)));
    apply(ctx);
}

/// The active palette: light until something sets another.
pub fn palette(ctx: &Context) -> Palette {
    ctx.data(|data| data.get_temp(Id::unique(PALETTE))).unwrap_or_default()
}

/// Makes `palette` active, whatever the OS setting, and stops [`follow_system`]. Every view takes its
/// values on the next frame, which this requests.
pub fn set_palette(ctx: &Context, palette: Palette) {
    ctx.data_mut(|data| data.remove_temp::<bool>(Id::unique(FOLLOW)));
    show(ctx, palette);
}

/// Makes the active palette the OS setting's: dark while the OS is dark, light otherwise, including
/// while the OS reports no setting. The shell takes each change of the setting on the frame that
/// reports it ([`sync_system`]), until [`set_palette`] picks a palette.
pub fn follow_system(ctx: &Context) {
    ctx.data_mut(|data| data.insert_temp(Id::unique(FOLLOW), true));
    sync_system(ctx);
}

/// While following the OS, takes its setting's palette when that differs from the active one. The
/// shell calls it every frame; restyling only on a change keeps an idle window from repainting.
pub(crate) fn sync_system(ctx: &Context) {
    if !ctx.data(|data| data.get_temp::<bool>(Id::unique(FOLLOW))).unwrap_or(false) {
        return;
    }
    let system = match ctx.system_theme() {
        Some(Theme::Dark) => Palette::Dark,
        Some(Theme::Light) | None => Palette::Light,
    };
    if system != palette(ctx) {
        show(ctx, system);
    }
}

fn show(ctx: &Context, palette: Palette) {
    ctx.data_mut(|data| data.insert_temp(Id::unique(PALETTE), palette));
    apply(ctx);
}

/// Styles the chrome from the [`CHROME`] roles in the active palette, whatever the OS setting.
///
/// The palette's visuals go into both of egui's styles rather than pinning egui's theme preference:
/// egui would pass a pinned theme on to the native window, and winit then stops reporting the OS
/// setting's changes (`Context::system_theme`), which following the OS (E05-S1-T3) reads.
fn apply(ctx: &Context) {
    let theme = match palette(ctx) {
        Palette::Light => Theme::Light,
        Palette::Dark => Theme::Dark,
    };
    let visuals = visuals(ctx, theme);
    for style in [Theme::Light, Theme::Dark] {
        ctx.set_visuals_of(style, visuals.clone());
    }
    ctx.request_repaint();
}

/// egui's visuals for `theme`, with every chrome and selection colour taken from its role.
fn visuals(ctx: &Context, theme: Theme) -> Visuals {
    let mut visuals = theme.default_visuals();
    let [background, window, text, control, active, input, border, selection, selection_text] = CHROME.map(|role| colour(ctx, &role));
    visuals.panel_fill = background;
    visuals.window_fill = window;
    visuals.window_stroke.color = border;
    visuals.extreme_bg_color = input;
    visuals.text_cursor.stroke.color = text;
    visuals.selection.bg_fill = selection;
    visuals.selection.stroke.color = selection_text;
    let widgets = &mut visuals.widgets;
    for (state, fill) in [
        (&mut widgets.noninteractive, window),
        (&mut widgets.inactive, control),
        (&mut widgets.hovered, active),
        (&mut widgets.active, active),
        (&mut widgets.open, control),
    ] {
        state.bg_fill = fill;
        state.weak_bg_fill = fill;
        // egui draws no edge on a control at rest; every state gets one, so a text field or a
        // checkbox box shows its boundary against the window or panel it sits on.
        state.bg_stroke = Stroke::new(state.bg_stroke.width.max(1.0), border);
        state.fg_stroke.color = text;
    }
    visuals
}

/// The registered value of `role` in the active palette, or [`UNREGISTERED`] if no role with its id
/// is registered.
pub fn colour(ctx: &Context, role: &ColourRole) -> Color32 {
    let roles: Option<Arc<ColourRoles>> = ctx.data(|data| data.get_temp(Id::unique(ROLES)));
    match roles.as_deref().and_then(|roles| roles.get(role.id)) {
        Some(registered) => {
            let Rgba { r, g, b, a } = registered.value(palette(ctx));
            Color32::from_rgba_unmultiplied(r, g, b, a)
        }
        None => UNREGISTERED,
    }
}
