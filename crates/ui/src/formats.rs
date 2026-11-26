//! The File Formats manager (E10-S1-T1): Tools > File Formats, and File Formats in the macOS
//! application menu.
//!
//! The shell holds the app's file-format registry (`otd-format`), [`Shell::formats`]. The manager
//! is a dialog listing its single ordered list, top first: a row per format with a checkbox named
//! by the format that enables or disables it, its kind and its mask. It changes the registry
//! itself, so resolution follows at once. Move Up and Move Down move the selected row, the one
//! whose checkbox last had focus or whose row was last clicked; each move is announced. Close and
//! Esc close it and give focus back to the window. [`commands`] has the menu command and the same
//! changes by format name, for scripts and the automation interface.
//!
//! Not here yet: the shadowed and disabled cues (E10-S1-T2); New, Duplicate, Edit and Delete
//! (E10-S1-T3); and saving the list (E10-S1-T4). The catch-all Text format isn't a row: it has no
//! place in the order.

use egui::accesskit::{Live, Role};
use egui::{Align, Context, Id, Key, Layout, Modifiers, Sense, Shape, TextStyle, TextWrapMode, Ui, Vec2, WidgetText};
use otd_core::command::{Args, Command, CommandError, Kind as ParamKind, Outcome, Param};
use otd_core::i18n::{fmt, t, trn};
use otd_format::{CATCH_ALL, Kind, Registry};

use crate::{CLOSE, Shell};

// English keys, drawn only through `t`.
/// The dialog's title and the command's caption.
pub const FILE_FORMATS: &str = "File Formats";
pub const MOVE_UP: &str = "Move Up";
pub const MOVE_DOWN: &str = "Move Down";
/// The column headers.
pub const NAME: &str = "Name";
pub const KIND: &str = "Kind";
pub const MASK: &str = "Mask";
/// What an empty list shows; `{format}` is the catch-all's name.
pub const NO_FORMATS: &str = "No file formats are registered yet, so every file opens as {format}.";
/// The announcements of a move; `{format}` is the moved format's name and `{n}` its row, from 1 at
/// the top, so that each move of the same format reads differently and is announced again.
pub const MOVED_UP: &str = "Moved {format} up to row {n}.";
pub const MOVED_DOWN: &str = "Moved {format} down to row {n}.";
/// The hidden commands' captions and their failure; `{format}` is the name given.
pub const MOVE_FORMAT_UP: &str = "Move Format Up";
pub const MOVE_FORMAT_DOWN: &str = "Move Format Down";
pub const ENABLE_FORMAT: &str = "Enable Format";
pub const NO_SUCH_FORMAT: &str = "No file format is called {format}.";

/// Each kind's caption, in English.
pub fn kind_caption(kind: Kind) -> &'static str {
    match kind {
        Kind::Text => "Text",
        Kind::Table => "Table",
        Kind::Hex => "Hex",
        Kind::Picture => "Picture",
        Kind::External => "External",
    }
}

/// The registry and the open manager.
#[derive(Default)]
pub(crate) struct Formats {
    registry: Registry,
    pub(crate) dialog: Option<Dialog>,
}

/// The open File Formats manager.
pub(crate) struct Dialog {
    /// The selected format, by name: the one Move Up and Move Down move.
    selected: Option<String>,
    /// The format whose checkbox takes focus on the next draw.
    focus: Option<String>,
    /// Close takes focus on the next draw: the list is empty.
    focus_close: bool,
    /// The last move, announced.
    moved: Option<String>,
}

impl Dialog {
    /// Moves the selected format, if there is one, and says so. One that reaches the end it moved
    /// towards takes focus, since the button that moved it is then disabled.
    fn move_selected(&mut self, registry: &mut Registry, up: bool) {
        let Some(name) = self.selected.clone() else { return };
        let Some(at) = registry.move_format(&name, up) else { return };
        let template = if up { MOVED_UP } else { MOVED_DOWN };
        self.moved = Some(fmt(&trn(at as u64 + 1, template, template), &[("format", &name)]));
        if (up && at == 0) || (!up && at + 1 == registry.formats().len()) {
            self.focus = Some(name);
        }
    }
}

/// File Formats, and the hidden commands that move, enable and disable a format by name. Only File
/// Formats needs the window.
pub(crate) fn commands() -> Vec<Command<Shell>> {
    const FORMAT: &[Param] = &[Param::required("format", ParamKind::Text)];
    const ENABLE: &[Param] = &[Param::required("format", ParamKind::Text), Param::required("enabled", ParamKind::Bool)];
    vec![
        Command::new("otd.file_formats", FILE_FORMATS, Shell::open_file_formats).application_menu(&["Tools"]).needs_window(),
        Command::typed("otd.move_format_up", MOVE_FORMAT_UP, FORMAT, |shell: &mut Shell, args| move_command(shell, args, true)),
        Command::typed("otd.move_format_down", MOVE_FORMAT_DOWN, FORMAT, |shell: &mut Shell, args| move_command(shell, args, false)),
        Command::typed("otd.enable_format", ENABLE_FORMAT, ENABLE, |shell: &mut Shell, args| {
            let name = args.text("format").unwrap_or_default();
            let enabled = args.bool("enabled").unwrap_or(true);
            if shell.formats.registry.set_enabled(name, enabled) { Ok(None) } else { Err(no_such(name)) }
        }),
    ]
}

fn move_command(shell: &mut Shell, args: &Args, up: bool) -> Outcome {
    let name = args.text("format").unwrap_or_default();
    shell.formats.registry.move_format(name, up).map(|_| None).ok_or_else(|| no_such(name))
}

fn no_such(name: &str) -> CommandError {
    CommandError::Failed(fmt(t(NO_SUCH_FORMAT), &[("format", name)]))
}

impl Shell {
    /// The app's file formats: what resolves each file name.
    pub fn formats(&self) -> &Registry {
        &self.formats.registry
    }

    /// The app's file formats, to register kinds and formats in.
    pub fn formats_mut(&mut self) -> &mut Registry {
        &mut self.formats.registry
    }

    /// Tools > File Formats: opens the manager with the top format selected and focused.
    pub fn open_file_formats(&mut self) {
        let first = self.formats.registry.formats().first().map(|f| f.name.clone());
        let focus_close = first.is_none();
        self.formats.dialog = Some(Dialog { selected: first.clone(), focus: first, focus_close, moved: None });
    }

    /// Draws the File Formats manager, if open, over the window.
    pub(crate) fn formats_ui(&mut self, ctx: &Context) {
        let Formats { registry, dialog: Some(dialog) } = &mut self.formats else { return };
        let id = Id::unique("otd-file-formats");
        let mut close = false;
        let modal = egui::Modal::new(id).show(ctx, |ui| {
            // The modal's area is its Dialog node; it takes the area's id.
            ui.ctx().accesskit_node_builder(id, |node| node.set_label(t(FILE_FORMATS)));
            ui.heading(t(FILE_FORMATS));
            if registry.formats().is_empty() {
                ui.label(fmt(t(NO_FORMATS), &[("format", t(CATCH_ALL))]));
            } else {
                list_ui(ui, registry, dialog);
            }
            ui.separator();
            ui.horizontal(|ui| {
                let at = dialog.selected.as_deref().and_then(|name| registry.formats().iter().position(|f| f.name == name));
                let last = registry.formats().len().saturating_sub(1);
                for (caption, up, can) in [(MOVE_UP, true, at.is_some_and(|at| at > 0)), (MOVE_DOWN, false, at.is_some_and(|at| at < last))] {
                    if ui.add_enabled(can, egui::Button::new(t(caption))).clicked() {
                        dialog.move_selected(registry, up);
                    }
                }
                let close_button = ui.button(t(CLOSE));
                if std::mem::take(&mut dialog.focus_close) {
                    close_button.request_focus();
                }
                close = close_button.clicked();
            });
            if let Some(moved) = &dialog.moved {
                let label = ui.label(moved);
                // Announced: focus stays on the button that moved it, away from the row.
                ui.ctx().accesskit_node_builder(label.id, |node| node.set_live(Live::Polite));
            }
        });
        // Esc closes it, unless it closes a popup first.
        let escape = modal.is_top_modal && !modal.any_popup_open && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape));
        if close || escape {
            self.formats.dialog = None;
            // Focus goes back to the window: its selected tab.
            self.focus_active = true;
        }
    }
}

fn text_width(ui: &Ui, text: &str) -> f32 {
    WidgetText::from(text).into_galley(ui, Some(TextWrapMode::Extend), f32::INFINITY, TextStyle::Body).size().x
}

/// The list: a header row, then a row per format, top first, each column as wide as its widest
/// entry.
fn list_ui(ui: &mut Ui, registry: &mut Registry, dialog: &mut Dialog) {
    let formats = registry.formats();
    let checkbox = ui.spacing().icon_width + ui.spacing().icon_spacing;
    let widest = |header: &str, cells: &mut dyn Iterator<Item = &str>| cells.map(|c| text_width(ui, c)).fold(text_width(ui, t(header)), f32::max);
    let widths = [
        widest(NAME, &mut formats.iter().map(|f| f.name.as_str())) + checkbox,
        widest(KIND, &mut formats.iter().map(|f| t(kind_caption(f.kind)))),
        widest(MASK, &mut formats.iter().map(|f| f.masks())),
    ];
    let height = ui.spacing().interact_size.y;
    let cell = |ui: &mut Ui, column: usize, add: &mut dyn FnMut(&mut Ui) -> egui::Response| {
        ui.allocate_ui_with_layout(Vec2::new(widths[column], height), Layout::left_to_right(Align::Center), |ui| {
            ui.set_min_width(widths[column]);
            add(ui)
        })
        .inner
    };
    let mut toggled = None;
    ui.scope(|ui| {
        ui.ctx().accesskit_node_builder(ui.unique_id(), |node| {
            node.set_role(Role::Table);
            node.set_label(t(FILE_FORMATS));
        });
        ui.horizontal(|ui| {
            ui.ctx().accesskit_node_builder(ui.unique_id(), |node| node.set_role(Role::Row));
            for (column, header) in [NAME, KIND, MASK].into_iter().enumerate() {
                let response = cell(ui, column, &mut |ui| ui.label(t(header)));
                ui.ctx().accesskit_node_builder(response.id, |node| node.set_role(Role::ColumnHeader));
            }
        });
        let max_height = ui.ctx().content_rect().height() * 0.6;
        egui::ScrollArea::vertical().max_height(max_height).show(ui, |ui| {
            for format in formats {
                let name = format.name.as_str();
                let selected = dialog.selected.as_deref() == Some(name);
                // By name, so a row keeps its ids, and its checkbox its focus, as it moves.
                ui.push_id(name, |ui| {
                    let background = ui.painter().add(Shape::Noop);
                    let row = ui.horizontal(|ui| {
                        ui.ctx().accesskit_node_builder(ui.unique_id(), |node| {
                            node.set_role(Role::Row);
                            node.set_label(name);
                            node.set_selected(selected);
                        });
                        let mut enabled = format.enabled;
                        let check = cell(ui, 0, &mut |ui| ui.checkbox(&mut enabled, name));
                        if enabled != format.enabled {
                            toggled = Some((name.to_owned(), enabled));
                        }
                        if dialog.focus.as_deref() == Some(name) {
                            check.request_focus();
                            check.scroll_to_me(None);
                            dialog.focus = None;
                        }
                        let kind = cell(ui, 1, &mut |ui| ui.add(egui::Label::new(t(kind_caption(format.kind))).sense(Sense::CLICK)));
                        let mask = cell(ui, 2, &mut |ui| ui.add(egui::Label::new(format.masks()).sense(Sense::CLICK)));
                        check.has_focus() || check.clicked() || kind.clicked() || mask.clicked()
                    });
                    if row.inner {
                        dialog.selected = Some(name.to_owned());
                    }
                    if selected {
                        let fill = Shape::rect_filled(row.response.rect, 0.0, ui.visuals().selection.bg_fill);
                        ui.painter().set(background, fill);
                    }
                });
            }
        });
    });
    if let Some((name, enabled)) = toggled {
        registry.set_enabled(&name, enabled);
    }
}
