//! Drawing with colour roles (E01-S1). The app installs the registered [`ColourRoles`] once; a view
//! draws with [`colour`], which gives a role's value in the active [`Palette`]. The table and the
//! palette live in the egui context, so any view or painter reaches them without the shell.

use std::sync::Arc;

use egui::{Color32, Context, Id};
pub use otd_core::colour::{ColourRole, ColourRoles, DuplicateColourRole, Palette, Rgba};

const ROLES: &str = "otd-colour-roles";
const PALETTE: &str = "otd-colour-palette";

/// What a role that was never registered draws with: a colour no palette would choose, so the
/// mistake shows on screen.
pub const UNREGISTERED: Color32 = Color32::from_rgb(0xFF, 0x00, 0xFF);

/// Makes `roles` the registered roles that [`colour`] resolves.
pub fn install(ctx: &Context, roles: ColourRoles) {
    ctx.data_mut(|data| data.insert_temp(Id::unique(ROLES), Arc::new(roles)));
    ctx.request_repaint();
}

/// The active palette: light until something sets another.
pub fn palette(ctx: &Context) -> Palette {
    ctx.data(|data| data.get_temp(Id::unique(PALETTE))).unwrap_or_default()
}

/// Makes `palette` active. Every view takes its values on the next frame, which this requests.
pub fn set_palette(ctx: &Context, palette: Palette) {
    ctx.data_mut(|data| data.insert_temp(Id::unique(PALETTE), palette));
    ctx.request_repaint();
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
