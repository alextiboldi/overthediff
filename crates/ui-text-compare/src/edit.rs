//! Editing in the panes (E07-S1-T5). Each pane body is an editor: it takes focus (Tab, or a click
//! that also places the caret), types (inserting or overwriting, as the status bar's switch says),
//! takes input methods' compositions, moves the caret with the arrows, Home, End and the page keys
//! (Shift selects), and answers the clipboard and the text-editing chords. After every edit both
//! panes are compared again, so the highlights follow the typing. Its accessibility node publishes
//! the drawn lines as text runs with the caret and selection.
//!
//! The Edit menu's commands act on the chosen pane: [`commands`] holds Undo, Redo, Cut, Copy,
//! Paste, Delete and Select All, and the hidden Insert Text, which types its `text` argument
//! there, so a script can edit without the window. While an editor has focus its own keys run the
//! same edits, and the shell leaves it the text-editing chords, as it does a text field (Q24). A
//! read-only pane still moves, selects and copies, and refuses the rest.

use std::sync::Arc;

use egui::accesskit::{self, Role, TextDirection, TextPosition, TextSelection};
use egui::os::OperatingSystem;
use egui::output::IMEOutput;
use egui::text_edit::TextEditState;
use egui::{
    Event, EventFilter, Galley, IMEPurpose, Id, ImeEvent, Key as EguiKey, Modifiers, Pos2, Rect, Sense, StrokeKind, TextStyle, Ui, UiBuilder,
    ViewportCommand, WidgetInfo, pos2, vec2,
};
use otd_core::command::{Args, Command, CommandError, Key, Kind, Mods, Outcome, Param, Platform, State};
use otd_core::i18n::t;
use otd_text_compare::{Pane, Pos, Selection};
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
        // Disabled while another text field has focus: with no shortcut, the menu can't hand the
        // click to that field as it does the others', and it would edit a pane the user isn't in.
        Command::new("text-compare.delete", DELETE, |shell: &mut Shell| on_view(shell, TextCompareView::delete))
            .state(|shell: &Shell| when(shell, |view| view.can_edit() && !view.field_focus))
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

    /// Side `side`'s pane body: the editor over the aligned rows. Returns whether it has focus.
    pub(crate) fn body(&mut self, ui: &mut Ui, side: usize) -> bool {
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
        if response.gained_focus() {
            self.ime = self.ime.wrapping_add(1).max(1);
        }
        if (response.lost_focus() || pressed) && !self.preedit.is_empty() {
            self.preedit.clear();
            ctx.memory_mut(|m| m.interrupt_ime());
        }
        if focused {
            self.chosen = side;
            self.cursor = Some(side);
            // The arrows move the caret, not focus; Tab still leaves, so there is no trap.
            ctx.memory_mut(|m| {
                m.set_focus_lock_filter(response.id, EventFilter { horizontal_arrows: true, vertical_arrows: true, ..Default::default() })
            });
            if std::mem::take(&mut self.paste) {
                ctx.send_viewport_cmd(ViewportCommand::RequestPaste);
            }
            let input = Input { editable, page: self.aligned.page(side, ui), mac: ctx.os() == OperatingSystem::Mac, insert: self.insert };
            let events = ui.input(|i| i.events.clone());
            let typed = loaded_mut(&mut self.panes, side).map(|pane| (keys(&ctx, &events, pane, input, &mut self.preedit), pane.selection.head.line));
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
        let drawn = aligned::show(ui, side, lines, &mut self.aligned, &Caret { selection, shown: focused, pointer });
        if let (Some(at), Some(pane)) = (drawn.hit, loaded_mut(&mut self.panes, side)) {
            pane.move_caret(at, extend);
            ctx.request_repaint();
        }
        if focused {
            ui.painter().rect_stroke(rect, 0.0, ui.visuals().selection.stroke, StrokeKind::Inside);
        }
        if focused && editable {
            let height = ui.text_style_height(&TextStyle::Monospace);
            let caret = drawn.caret.unwrap_or_else(|| Rect::from_min_size(rect.min, vec2(2.0, height)));
            if !self.preedit.is_empty() {
                preedit(ui, &self.preedit, caret, rect);
            }
            // Asked for on every pass while the editor can take text: without it the window turns
            // its input methods off.
            ctx.output_mut(|o| o.ime = Some(IMEOutput::new(response.id, self.ime, IMEPurpose::Normal, rect, caret)));
        }

        // Named by its side's caption, with how the caret's line differs, which changes as the
        // user types, as its description. Its text, caret and selection are the drawn lines' runs.
        let pane = loaded(&self.panes, side);
        let head = pane.map(|pane| pane.selection.head).unwrap_or_default();
        let difference = self.aligned.difference(side, head.line);
        response.widget_info(|| WidgetInfo::labeled(Role::MultilineTextInput, true, t(SIDES[side])));
        let built = ctx.accesskit_node_builder(response.id, |node| {
            if !editable {
                node.set_read_only();
            }
            if let Some(difference) = difference {
                node.set_description(t(difference));
            }
        });
        if built.is_some() {
            text_runs(ui, response.id, &drawn.lines, pane.map(|pane| pane.selection).unwrap_or_default());
        }
        focused
    }
}

/// The input method's composition `text`, drawn over the text at `caret`, underlined, inside the
/// pane's `rect`.
// ponytail: drawn over the characters after the caret rather than pushing them aside; layout the
// line with it if users find that confusing.
fn preedit(ui: &Ui, text: &str, caret: Rect, rect: Rect) {
    let painter = ui.painter().with_clip_rect(rect);
    let galley = painter.layout_no_wrap(text.to_owned(), TextStyle::Monospace.resolve(ui.style()), ui.visuals().text_color());
    let at = pos2(caret.left(), caret.center().y - galley.size().y / 2.0);
    let area = Rect::from_min_size(at, galley.size());
    painter.rect_filled(area, 0.0, ui.visuals().extreme_bg_color);
    painter.galley(at, galley, ui.visuals().text_color());
    painter.hline(area.x_range(), area.bottom(), ui.visuals().widgets.active.fg_stroke);
}

/// AccessKit's character indices in a text run are `u8`, as egui's text widgets keep them.
const RUN: usize = 255;

/// The drawn `lines` as text runs of the editor `editor`, with `selection` as its text selection,
/// so screen readers follow the caret and selection inside a line and magnifiers find the caret.
/// Only the drawn lines have runs; an end of the selection outside them is taken to the nearest.
fn text_runs(ui: &mut Ui, editor: Id, lines: &[(usize, Arc<Galley>, Pos2, bool)], selection: Selection) {
    // Each drawn line's characters, with their left edges and widths as drawn.
    let glyphs: Vec<Vec<(char, f32, f32)>> = lines
        .iter()
        .map(|(_, galley, _, _)| {
            galley.rows.iter().flat_map(|row| row.glyphs.iter().map(|g| (g.chr, row.pos.x + g.pos.x, g.advance_width))).collect()
        })
        .collect();
    let chunks = |chars: &[(char, f32, f32)]| chars.len().max(1).div_ceil(RUN);
    let run_id = |line: usize, chunk: usize| editor.with(("run", line, chunk));
    let position = |pos: Pos| -> Option<TextPosition> {
        let found = lines.iter().position(|line| line.0 == pos.line);
        let (at, column) = match found {
            Some(at) => (at, pos.column.min(glyphs[at].len())),
            None if pos.line < lines.first()?.0 => (0, 0),
            // Past the drawn lines, or after a final line break: the end of the last drawn line.
            None => (lines.len() - 1, glyphs.last()?.len() + usize::from(lines.last()?.3)),
        };
        let chunk = (column / RUN).min(chunks(&glyphs[at]) - 1);
        Some(TextPosition { node: run_id(lines[at].0, chunk).accesskit_id(), character_index: column - chunk * RUN })
    };
    let (Some(anchor), Some(focus)) = (position(selection.anchor), position(selection.head)) else { return };
    ui.ctx().accesskit_node_builder(editor, |node| node.set_text_selection(TextSelection { anchor, focus }));
    ui.scope_builder(UiBuilder::new().accessibility_parent(editor), |ui| {
        for ((index, galley, origin, ends_with_break), glyphs) in lines.iter().zip(&glyphs) {
            let end = glyphs.last().map_or(0.0, |&(_, x, width)| x + width);
            let chunks = chunks(glyphs);
            for chunk in 0..chunks {
                let part = glyphs.get(chunk * RUN..glyphs.len().min((chunk + 1) * RUN)).unwrap_or_default();
                let left = part.first().map_or(end, |&(_, x, _)| x);
                let right = part.last().map_or(end, |&(_, x, width)| x + width);
                let mut value: String = part.iter().map(|&(c, _, _)| c).collect();
                let mut lengths: Vec<u8> = part.iter().map(|&(c, _, _)| c.len_utf8() as u8).collect();
                let mut positions: Vec<f32> = part.iter().map(|&(_, x, _)| x - left).collect();
                let mut widths: Vec<f32> = part.iter().map(|&(_, _, width)| width).collect();
                if *ends_with_break && chunk + 1 == chunks {
                    value.push('\n');
                    lengths.push(1);
                    positions.push(right - left);
                    widths.push(0.0);
                }
                let mut word_starts = Vec::new();
                let mut in_word = false;
                for (i, &(c, _, _)) in part.iter().enumerate() {
                    let word = c.is_alphanumeric() || c == '_';
                    if word && !in_word {
                        word_starts.push(i as u8);
                    }
                    in_word = word;
                }
                let bounds = Rect::from_min_max(*origin + vec2(left, 0.0), *origin + vec2(right, galley.size().y));
                let id = run_id(*index, chunk);
                ui.interact(bounds, id, Sense::hover());
                ui.ctx().accesskit_node_builder(id, |node| {
                    node.set_role(Role::TextRun);
                    // ponytail: left to right, as the lines are laid out; split runs by direction
                    // (egui's `accesskit_text`) when the panes lay out right-to-left text.
                    node.set_text_direction(TextDirection::LeftToRight);
                    node.set_bounds(accesskit::Rect {
                        x0: bounds.min.x.into(),
                        y0: bounds.min.y.into(),
                        x1: bounds.max.x.into(),
                        y1: bounds.max.y.into(),
                    });
                    node.set_value(value);
                    node.set_character_lengths(lengths);
                    node.set_character_positions(positions);
                    node.set_character_widths(widths);
                    node.set_word_starts(word_starts);
                    if chunk > 0 {
                        node.set_previous_on_line(run_id(*index, chunk - 1).accesskit_id());
                    }
                    if chunk + 1 < chunks {
                        node.set_next_on_line(run_id(*index, chunk + 1).accesskit_id());
                    }
                });
            }
        }
    });
}

/// What the focused editor's keys may do this frame.
#[derive(Clone, Copy)]
struct Input {
    editable: bool,
    /// Rows in a page, for Page Up and Page Down.
    page: usize,
    mac: bool,
    /// Insert, or overwrite the characters after the caret.
    insert: bool,
}

/// Runs this frame's `events` in the focused `pane`: typing, the input method's composition
/// (`preedit`) and commits, the clipboard, the text-editing chords, Enter, Backspace and Delete,
/// and caret moves. Returns whether the text changed and whether the caret moved.
// ponytail: no word-wise moves yet.
fn keys(ctx: &egui::Context, events: &[Event], pane: &mut Pane, input: Input, preedit: &mut String) -> (bool, bool) {
    let mut edited = false;
    let mut moved = false;
    let typed = |pane: &mut Pane, text: &str| {
        if input.insert { pane.type_text(text) } else { pane.overwrite(text) }
    };
    for event in events {
        match event {
            Event::Text(text) if input.editable => {
                typed(pane, text);
                edited = true;
            }
            Event::Ime(ime) if input.editable => match ime {
                // Enter while composing is the input method's own key, not a line break.
                ImeEvent::Preedit { text, .. } if text != "\n" && text != "\r" => text.clone_into(preedit),
                ImeEvent::Commit(text) => {
                    preedit.clear();
                    if !text.is_empty() && text != "\n" && text != "\r" {
                        typed(pane, text);
                        edited = true;
                    }
                }
                // ponytail: no surrounding text is published, so an input method has none to
                // delete; publish it (`IMESurroundingText`) for input methods that reconvert.
                _ => {}
            },
            Event::Paste(text) if input.editable => {
                pane.insert(text);
                edited = true;
            }
            Event::Copy if !pane.selection.is_empty() => ctx.copy_text(pane.selected_text()),
            Event::Cut if input.editable && !pane.selection.is_empty() => {
                ctx.copy_text(pane.selected_text());
                pane.delete(true);
                edited = true;
            }
            Event::Key { key, pressed: true, modifiers, .. } => match key_press(pane, *key, *modifiers, input) {
                Some(true) => edited = true,
                Some(false) => moved = true,
                None => {}
            },
            _ => {}
        }
    }
    if moved && !preedit.is_empty() {
        preedit.clear();
        ctx.memory_mut(|m| m.interrupt_ime());
    }
    (edited, moved || edited)
}

/// One key press in the focused pane: `Some(true)` when it edited the text, `Some(false)` when it
/// only moved the caret or selection.
fn key_press(pane: &mut Pane, key: EguiKey, modifiers: Modifiers, input: Input) -> Option<bool> {
    let Input { editable, page, mac, .. } = input;
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
