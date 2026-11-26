//! Editing in the panes (E07-S1-T5). Each pane body is an editor: it takes focus (Tab, or a click
//! that also places the caret), types, moves the caret with the arrows, Home, End and the page
//! keys (Shift selects), and answers the clipboard and the text-editing chords. After every edit
//! both panes are compared again, so the highlights follow the typing.
//!
//! The Edit menu's commands act on the chosen pane: [`commands`] holds Undo, Redo, Cut, Copy,
//! Paste, Delete and Select All, and the hidden Insert Text, which types its `text` argument
//! there, so a script can edit without the window. While an editor has focus its own keys run the same edits, and
//! the shell leaves it the text-editing chords, as it does a text field (Q24). A read-only pane
//! still moves, selects and copies, and refuses the rest.

use egui::accesskit::Role;
use egui::os::OperatingSystem;
use egui::text_edit::TextEditState;
use egui::{Event, EventFilter, Key as EguiKey, Modifiers, Sense, StrokeKind, Ui, ViewportCommand, WidgetInfo};
use otd_core::command::{Args, Command, CommandError, Key, Kind, Mods, Outcome, Param, Platform, State};
use otd_core::i18n::t;
use otd_text_compare::{Pane, Pos};
use otd_ui::Shell;

use crate::aligned::{self, Caret};
use crate::{SIDES, TEXT_COMPARE, TextCompareView};

// The Edit menu's commands: English keys, drawn through `t`.
pub const UNDO: &str = "Undo";
pub const REDO: &str = "Redo";
pub const CUT: &str = "Cut";
pub const COPY: &str = "Copy";
pub const PASTE: &str = "Paste";
pub const DELETE: &str = "Delete";
pub const SELECT_ALL: &str = "Select All";
pub const INSERT_TEXT: &str = "Insert Text";
const EDIT: &[&str] = &["Edit"];
const NOT_EDITABLE: &str = "The chosen pane has no file loaded, or is read-only.";
const TEXT: &[Param] = &[Param::required("text", Kind::Text)];

/// The Edit menu's commands, with the platforms' shortcuts (Q24). Delete has none: the Delete key
/// is the focused editor's own. Only the clipboard's need the window.
pub(crate) fn commands() -> Vec<Command<Shell>> {
    const PC: &[Platform] = &[Platform::Windows, Platform::Linux];
    let shift = Mods { shift: true, ..Mods::CMD };
    vec![
        Command::new("text-compare.undo", UNDO, |shell: &mut Shell| on_view(shell, TextCompareView::undo))
            .state(|shell: &Shell| when(shell, TextCompareView::can_undo))
            .icon("undo-2")
            .shortcut(Mods::CMD, Key::Z),
        Command::new("text-compare.redo", REDO, |shell: &mut Shell| on_view(shell, TextCompareView::redo))
            .state(|shell: &Shell| when(shell, TextCompareView::can_redo))
            .icon("redo-2")
            .shortcut(shift, Key::Z)
            .shortcut_on(PC, Mods::CMD, Key::Y),
        Command::new("text-compare.cut", CUT, |shell: &mut Shell| on_view(shell, TextCompareView::cut))
            .state(|shell: &Shell| when(shell, TextCompareView::can_cut))
            .icon("scissors")
            .shortcut(Mods::CMD, Key::X)
            .needs_window(),
        Command::new("text-compare.copy", COPY, |shell: &mut Shell| on_view(shell, TextCompareView::copy))
            .state(|shell: &Shell| when(shell, TextCompareView::can_copy))
            .icon("copy")
            .shortcut(Mods::CMD, Key::C)
            .needs_window(),
        Command::new("text-compare.paste", PASTE, |shell: &mut Shell| on_view(shell, TextCompareView::paste))
            .state(|shell: &Shell| when(shell, TextCompareView::can_edit))
            .icon("clipboard-paste")
            .shortcut(Mods::CMD, Key::V)
            .needs_window(),
        Command::new("text-compare.delete", DELETE, |shell: &mut Shell| on_view(shell, TextCompareView::delete))
            .state(|shell: &Shell| when(shell, TextCompareView::can_edit))
            .icon("delete"),
        Command::new("text-compare.select_all", SELECT_ALL, |shell: &mut Shell| on_view(shell, TextCompareView::select_all))
            .state(|shell: &Shell| when(shell, |view| view.pane(view.chosen).is_some()))
            .icon("text-select")
            .shortcut(Mods::CMD, Key::A),
    ]
    .into_iter()
    .map(|command| command.menu(EDIT))
    .chain([Command::typed("text-compare.insert_text", INSERT_TEXT, TEXT, insert_text).state(|shell: &Shell| when(shell, TextCompareView::can_edit))])
    .map(|command| command.views(&[TEXT_COMPARE]))
    .collect()
}

fn insert_text(shell: &mut Shell, args: &Args) -> Outcome {
    let view = shell.view_mut::<TextCompareView>().filter(|view| view.can_edit()).ok_or_else(|| CommandError::Failed(t(NOT_EDITABLE).to_owned()))?;
    let text = args.text("text").unwrap_or_default().to_owned();
    view.edit(|pane| pane.insert(&text));
    Ok(None)
}

fn on_view(shell: &mut Shell, edit: fn(&mut TextCompareView)) {
    if let Some(view) = shell.view_mut::<TextCompareView>() {
        edit(view);
    }
}

fn when(shell: &Shell, can: fn(&TextCompareView) -> bool) -> State {
    State::enabled(shell.view::<TextCompareView>().is_some_and(can))
}

/// Side `side`'s loaded pane.
fn loaded(panes: &[Option<Result<Pane, String>>], side: usize) -> Option<&Pane> {
    panes.get(side)?.as_ref()?.as_ref().ok()
}

fn loaded_mut(panes: &mut [Option<Result<Pane, String>>], side: usize) -> Option<&mut Pane> {
    panes.get_mut(side)?.as_mut()?.as_mut().ok()
}

impl TextCompareView {
    /// Whether the chosen pane is loaded and can be edited.
    pub fn can_edit(&self) -> bool {
        self.pane(self.chosen).is_some() && self.read_only(self.chosen).is_none()
    }

    pub fn can_undo(&self) -> bool {
        self.can_edit() && self.pane(self.chosen).is_some_and(Pane::can_undo)
    }

    pub fn can_redo(&self) -> bool {
        self.can_edit() && self.pane(self.chosen).is_some_and(Pane::can_redo)
    }

    /// Whether the chosen pane has text selected.
    pub fn can_copy(&self) -> bool {
        self.pane(self.chosen).is_some_and(|pane| !pane.selection.is_empty())
    }

    pub fn can_cut(&self) -> bool {
        self.can_copy() && self.can_edit()
    }

    /// *Undo* in the chosen pane.
    pub fn undo(&mut self) {
        self.edit(|pane| {
            pane.undo();
        });
    }

    /// *Redo* in the chosen pane.
    pub fn redo(&mut self) {
        self.edit(|pane| {
            pane.redo();
        });
    }

    /// *Copy*: the chosen pane's selected text goes to the clipboard on the next draw.
    pub fn copy(&mut self) {
        if let Some(pane) = self.pane(self.chosen).filter(|pane| !pane.selection.is_empty()) {
            self.clipboard = Some(pane.selected_text());
        }
        self.focus = Some(self.chosen);
    }

    /// *Cut*: copies the selected text and deletes it.
    pub fn cut(&mut self) {
        if self.can_cut() {
            self.copy();
            self.edit(|pane| pane.delete(true));
        }
    }

    /// *Paste*: asks for the clipboard's text, which the chosen pane's editor takes when it comes.
    pub fn paste(&mut self) {
        self.paste = self.can_edit();
        self.focus = Some(self.chosen);
    }

    /// *Delete*: the selected text, or the character after the caret.
    pub fn delete(&mut self) {
        self.edit(|pane| pane.delete(true));
    }

    /// *Select All*: every line of the chosen pane.
    pub fn select_all(&mut self) {
        if let Some(pane) = loaded_mut(&mut self.panes, self.chosen) {
            pane.select_all();
        }
        self.focus = Some(self.chosen);
    }

    /// Runs `edit` on the chosen pane unless it is read-only, then compares again.
    fn edit(&mut self, edit: impl FnOnce(&mut Pane)) {
        let side = self.chosen;
        self.focus = Some(side);
        if !self.can_edit() {
            return;
        }
        if let Some(pane) = loaded_mut(&mut self.panes, side) {
            edit(pane);
            let line = pane.selection.head.line;
            self.recompare();
            self.aligned.reveal(side, line);
        }
    }

    /// Compares both panes' lines again, as they now stand.
    pub(crate) fn recompare(&mut self) {
        let [left, right] = [0, 1].map(|side| loaded(&self.panes, side).map(|pane| pane.lines.as_slice()));
        self.aligned.recompare(left, right);
    }

    /// Side `side`'s pane body: the editor over the aligned rows.
    pub(crate) fn body(&mut self, ui: &mut Ui, side: usize) {
        let editable = self.read_only(side).is_none();
        let rect = ui.available_rect_before_wrap();
        let response = ui.interact(rect, ui.scope_id().with("editor"), Sense::click_and_drag());
        let ctx = ui.ctx().clone();
        // Counted as a text field, so the shell's shortcuts leave it plain keys and the
        // text-editing chords (Q24), and its keys run its own edits.
        if TextEditState::load(&ctx, response.id).is_none() {
            TextEditState::default().store(&ctx, response.id);
        }
        let pressed = response.is_pointer_button_down_on() && ui.input(|i| i.pointer.primary_pressed());
        if self.focus.take_if(|s| *s == side).is_some() || pressed {
            response.request_focus();
        }
        let focused = response.has_focus();
        if focused {
            self.chosen = side;
            // The arrows move the caret, not focus; Tab still leaves, so there is no trap.
            ctx.memory_mut(|m| {
                m.set_focus_lock_filter(response.id, EventFilter { horizontal_arrows: true, vertical_arrows: true, ..Default::default() })
            });
            if std::mem::take(&mut self.paste) {
                ctx.send_viewport_cmd(ViewportCommand::RequestPaste);
            }
            let page = self.aligned.page(side, ui);
            let mac = ctx.os() == OperatingSystem::Mac;
            let events = ui.input(|i| i.events.clone());
            let typed = loaded_mut(&mut self.panes, side).map(|pane| (keys(&ctx, &events, pane, editable, page, mac), pane.selection.head.line));
            if let Some(((edited, moved), line)) = typed {
                if edited {
                    self.recompare();
                }
                if moved {
                    self.aligned.reveal(side, line);
                }
            }
        }
        if let Some(text) = self.clipboard.take() {
            ctx.copy_text(text);
        }

        let pointer = response.is_pointer_button_down_on().then(|| ui.input(|i| i.pointer.interact_pos())).flatten();
        let extend = ui.input(|i| !i.pointer.primary_pressed() || i.modifiers.shift);
        let panes = [0, 1].map(|side| loaded(&self.panes, side));
        let lines = panes.map(|pane| pane.map_or(&[][..], |pane| pane.lines.as_slice()));
        let selection = panes[side].map(|pane| pane.selection).unwrap_or_default();
        let hit = aligned::show(ui, side, lines, &mut self.aligned, &Caret { selection, shown: focused, pointer });
        if let (Some(at), Some(pane)) = (hit, loaded_mut(&mut self.panes, side)) {
            pane.move_caret(at, extend);
            ctx.request_repaint();
        }
        if focused {
            ui.painter().rect_stroke(rect, 0.0, ui.visuals().selection.stroke, StrokeKind::Inside);
        }

        // Named by its side's caption, with the caret's line as its value and how that line
        // differs, which changes as the user types, as its description.
        let pane = loaded(&self.panes, side);
        let head = pane.map(|pane| pane.selection.head).unwrap_or_default();
        let value = pane.and_then(|pane| pane.lines.get(head.line)).map(|line| line.text.clone()).unwrap_or_default();
        let difference = self.aligned.difference(side, head.line);
        response.widget_info(|| WidgetInfo::labeled(Role::MultilineTextInput, true, t(SIDES[side])));
        ctx.accesskit_node_builder(response.id, |node| {
            node.set_value(value);
            if !editable {
                node.set_read_only();
            }
            if let Some(difference) = difference {
                node.set_description(t(difference));
            }
        });
    }
}

/// Runs this frame's `events` in the focused `pane`: typing and the clipboard, the text-editing
/// chords, Enter, Backspace and Delete, and caret moves. Returns whether the text changed and
/// whether the caret moved.
// ponytail: no IME composition, overwrite mode or word-wise moves yet; composed text arrives once
// committed, as text.
fn keys(ctx: &egui::Context, events: &[Event], pane: &mut Pane, editable: bool, page: usize, mac: bool) -> (bool, bool) {
    let mut edited = false;
    let mut moved = false;
    for event in events {
        match event {
            Event::Text(text) if editable => {
                pane.type_text(text);
                edited = true;
            }
            Event::Paste(text) if editable => {
                pane.insert(text);
                edited = true;
            }
            Event::Copy if !pane.selection.is_empty() => ctx.copy_text(pane.selected_text()),
            Event::Cut if editable && !pane.selection.is_empty() => {
                ctx.copy_text(pane.selected_text());
                pane.delete(true);
                edited = true;
            }
            Event::Key { key, pressed: true, modifiers, .. } => match key_press(pane, *key, *modifiers, editable, page, mac) {
                Some(true) => edited = true,
                Some(false) => moved = true,
                None => {}
            },
            _ => {}
        }
    }
    (edited, moved || edited)
}

/// One key press in the focused pane: `Some(true)` when it edited the text, `Some(false)` when it
/// only moved the caret or selection.
fn key_press(pane: &mut Pane, key: EguiKey, modifiers: Modifiers, editable: bool, page: usize, mac: bool) -> Option<bool> {
    let head = pane.selection.head;
    let (start, end) = pane.selection.range();
    let collapse = !modifiers.shift && !pane.selection.is_empty();
    let line_start = Pos { line: head.line, column: 0 };
    let to = match key {
        EguiKey::Z if modifiers.command => return editable.then(|| if modifiers.shift { pane.redo() } else { pane.undo() }),
        EguiKey::Y if modifiers.command && !mac => return editable.then(|| pane.redo()),
        EguiKey::A if modifiers.command => {
            pane.select_all();
            return Some(false);
        }
        EguiKey::Enter if editable => {
            pane.insert("\n");
            return Some(true);
        }
        EguiKey::Backspace | EguiKey::Delete if editable => {
            pane.delete(key == EguiKey::Delete);
            return Some(true);
        }
        EguiKey::ArrowLeft if mac && modifiers.mac_cmd => line_start,
        EguiKey::ArrowRight if mac && modifiers.mac_cmd => pane.line_end(head.line),
        EguiKey::ArrowUp if mac && modifiers.mac_cmd => Pos::default(),
        EguiKey::ArrowDown if mac && modifiers.mac_cmd => pane.end(),
        EguiKey::ArrowLeft if collapse => start,
        EguiKey::ArrowRight if collapse => end,
        EguiKey::ArrowLeft => pane.before(head),
        EguiKey::ArrowRight => pane.after(head),
        EguiKey::ArrowUp => Pos { line: head.line.saturating_sub(1), ..head },
        EguiKey::ArrowDown => Pos { line: head.line + 1, ..head },
        EguiKey::PageUp => Pos { line: head.line.saturating_sub(page), ..head },
        EguiKey::PageDown => Pos { line: head.line.saturating_add(page), ..head },
        EguiKey::Home if modifiers.command => Pos::default(),
        EguiKey::End if modifiers.command => pane.end(),
        EguiKey::Home => line_start,
        EguiKey::End => pane.line_end(head.line),
        _ => return None,
    };
    pane.move_caret(to, modifiers.shift);
    Some(false)
}
