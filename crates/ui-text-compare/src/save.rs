//! Saving the panes (E07-S1-T6): *File > Save File* writes the chosen pane back to its file in its
//! own encoding, byte-order mark and line endings, so only edited lines' bytes change; *Save File
//! As* writes it to another path, which the pane then shows; the hidden *Save File on File System
//! As* does the same from the local file system. Without a `path` argument, the As commands open
//! the OS save picker, so without the window a script passes `path`.
//!
//! A pane with unsaved edits says so next to its caption, and [`TextCompareView`]'s
//! [`View::unsaved`](otd_ui::View::unsaved) and [`View::save`](otd_ui::View::save) let the shell
//! ask Save / Don't Save / Cancel on Close Tab, Exit and Reload Files. A save that fails says why
//! in its pane, announced, and the pane stays unsaved.

use egui::accesskit::Live;
use egui::{Id, Ui};
use otd_core::command::{Args, Command, CommandError, Key, Kind, Mods, Outcome, Param, State};
use otd_core::i18n::{fmt, t};
use otd_text_compare::{Encoding, ReadOnly, SaveError};
use otd_ui::{Shell, path_edit};

use crate::open::NOT_TEXT_COMPARE;
use crate::read_only::{self, ENCODING_LOSSY};
use crate::{LEFT, RIGHT, SIDES, TEXT_COMPARE, TextCompareView, encoding_name};

// The commands' captions and what the panes say: English keys, drawn through `t`.
pub const SAVE_FILE: &str = "Save File";
pub const SAVE_FILE_AS: &str = "Save File As";
/// The same, from the local file system, with no menu entry until the user adds one (hidden by
/// default).
pub const SAVE_FILE_ON_FILE_SYSTEM_AS: &str = "Save File on File System As";
/// The mark of a pane with unsaved edits.
pub const UNSAVED_CHANGES: &str = "Unsaved changes";
/// A failed save, as its pane shows it and the command fails with.
pub const SAVE_FAILED: &str = "{path} wasn't saved: {reason}";
const CANT_ENCODE: &str = "{character} on line {line} can't be written in {encoding}.";
const LOSSY: &str = "some of its bytes can't be read in its encoding, and saving would change them.";
const MARKED_READ_ONLY: &str = "the file is marked read-only.";
const NO_FILE: &str = "The pane has no file loaded.";

const FILE: &[&str] = &["File"];
const SIDE: Param = Param::optional("side", Kind::Choice(&[LEFT, RIGHT]));
/// `side` is Left or Right, the chosen pane when left out.
const SIDE_PARAM: &[Param] = &[SIDE];
/// And `path` is where to save; without it the OS save picker asks.
const AS_PARAMS: &[Param] = &[SIDE, Param::optional("path", Kind::Path)];

/// The File menu's save commands, which [`crate::commands`] lists after Open File, with the
/// platforms' shortcuts (Q24).
pub(crate) fn commands() -> Vec<Command<Shell>> {
    let shift = Mods { shift: true, ..Mods::CMD };
    [
        Command::typed("text-compare.save_file", SAVE_FILE, SIDE_PARAM, save_file)
            .state(|shell: &Shell| when(shell, TextCompareView::can_save))
            .icon("save")
            .menu(FILE)
            .shortcut(Mods::CMD, Key::S),
        Command::typed("text-compare.save_file_as", SAVE_FILE_AS, AS_PARAMS, save_file_as)
            .state(|shell: &Shell| when(shell, TextCompareView::can_save_as))
            .menu(FILE)
            .shortcut(shift, Key::S),
        Command::typed("text-compare.save_file_on_file_system_as", SAVE_FILE_ON_FILE_SYSTEM_AS, AS_PARAMS, save_file_as)
            .state(|shell: &Shell| when(shell, TextCompareView::can_save_as)),
    ]
    .into_iter()
    .map(|command| command.views(&[TEXT_COMPARE]))
    .collect()
}

/// Enabled while either pane `can` be saved: the state can't see the `side` a script passes, so
/// the handler checks that side.
fn when(shell: &Shell, can: fn(&TextCompareView, usize) -> bool) -> State {
    State::enabled(shell.view::<TextCompareView>().is_some_and(|view| (0..SIDES.len()).any(|side| can(view, side))))
}

fn view_and_side<'a>(shell: &'a mut Shell, args: &Args) -> Result<(&'a mut TextCompareView, Option<usize>), CommandError> {
    let view = shell.view_mut::<TextCompareView>().ok_or_else(|| CommandError::Failed(t(NOT_TEXT_COMPARE).to_owned()))?;
    let side = args.text("side").and_then(|side| SIDES.iter().position(|s| *s == side));
    Ok((view, side))
}

fn save_file(shell: &mut Shell, args: &Args) -> Outcome {
    let (view, side) = view_and_side(shell, args)?;
    view.save_file(side).map(|()| None).map_err(CommandError::Failed)
}

fn save_file_as(shell: &mut Shell, args: &Args) -> Outcome {
    let (view, side) = view_and_side(shell, args)?;
    view.save_file_as(side, args.text("path")).map(|()| None).map_err(CommandError::Failed)
}

/// What a pane says about `error` while saving it to `spec`, looked up.
fn message(error: SaveError, spec: &str, encoding: Encoding) -> String {
    let reason = match error {
        SaveError::Unencodable(at) => {
            let character = format!("{} (U+{:04X})", at.character, u32::from(at.character));
            fmt(t(CANT_ENCODE), &[("character", &character), ("line", &(at.line + 1).to_string()), ("encoding", &encoding_name(encoding))])
        }
        SaveError::Lossy => t(LOSSY).to_owned(),
        SaveError::MarkedReadOnly => t(MARKED_READ_ONLY).to_owned(),
        SaveError::Source(error) => error.message,
    };
    fmt(t(SAVE_FAILED), &[("path", spec), ("reason", &reason)])
}

impl TextCompareView {
    /// Whether *Save File* can write side `side`'s pane back to its file: it is loaded and nothing
    /// about its location, file or bytes stops it.
    pub fn can_save(&self, side: usize) -> bool {
        self.pane(side).is_some_and(|pane| pane.read_only.is_none())
    }

    /// Whether *Save File As* can write side `side`'s pane: it is loaded and reads without loss.
    pub fn can_save_as(&self, side: usize) -> bool {
        self.pane(side).is_some_and(|pane| pane.read_only != Some(ReadOnly::Encoding))
    }

    /// Whether side `side`'s pane has edits it hasn't saved.
    pub fn unsaved(&self, side: usize) -> bool {
        self.pane(side).is_some_and(|pane| pane.unsaved)
    }

    /// *Save File*: writes the pane of `side` (the chosen pane when `None`) back to its file.
    pub fn save_file(&mut self, side: Option<usize>) -> Result<(), String> {
        let side = self.target(side);
        let spec = self.pane(side).map(|pane| pane.path.clone()).ok_or_else(|| t(NO_FILE).to_owned())?;
        if let Some(why) = self.pane(side).and_then(|pane| pane.read_only) {
            return self.refuse(side, read_only::reason(why));
        }
        self.save_to(side, &spec)
    }

    /// *Save File As*: writes the pane of `side` (the chosen pane when `None`) to `path`, or with no
    /// path asks for one with the OS save picker on the next draw.
    pub fn save_file_as(&mut self, side: Option<usize>, path: Option<&str>) -> Result<(), String> {
        let side = self.target(side);
        self.chosen = side;
        if self.pane(side).is_none() {
            return Err(t(NO_FILE).to_owned());
        }
        if !self.can_save_as(side) {
            return self.refuse(side, ENCODING_LOSSY);
        }
        match path {
            Some(path) => self.save_to(side, path),
            None => {
                self.save_as = Some((side, false));
                Ok(())
            }
        }
    }

    /// The pane a save writes: `side`, or with none the chosen pane, or the loaded one when the
    /// chosen pane has no file. The commands are enabled while either pane can save, so an
    /// enabled command always has a pane to write or to say why it can't.
    fn target(&self, side: Option<usize>) -> usize {
        let side = side.unwrap_or_else(|| {
            let chosen = self.chosen.min(SIDES.len() - 1);
            if self.pane(chosen).is_some() { chosen } else { (0..SIDES.len()).find(|&s| self.pane(s).is_some()).unwrap_or(chosen) }
        });
        side.min(SIDES.len() - 1)
    }

    /// Refuses to save side `side`'s pane for `reason` (an English key), saying so in the pane.
    fn refuse(&mut self, side: usize, reason: &str) -> Result<(), String> {
        let message = t(reason).to_owned();
        self.save_errors[side] = Some(message.clone());
        Err(message)
    }

    /// Saves side `side`'s pane to `spec`, which the side then shows without reading it again, so
    /// its edits and undo history stay. A failure is also shown in the pane.
    fn save_to(&mut self, side: usize, spec: &str) -> Result<(), String> {
        let Some(Some(Ok(pane))) = self.panes.get_mut(side) else { return Err(t(NO_FILE).to_owned()) };
        let saved = otd_text_compare::save_as(pane, spec).map_err(|error| message(error, spec, pane.encoding));
        self.save_errors[side] = saved.clone().err();
        if saved.is_ok()
            && let (Some(current), Some(loaded)) = (self.session.specs.get_mut(side), self.loaded.get_mut(side))
        {
            // The path edit follows the spec on its next draw.
            spec.clone_into(current);
            *loaded = Some(spec.to_owned());
        }
        saved
    }

    /// Opens the OS save picker *Save File As* asked for, and saves where it answers.
    pub(crate) fn save_as_ui(&mut self, ui: &Ui) {
        let Some((side, asked)) = self.save_as else { return };
        let id = Id::unique("text-compare.save_as").with(side);
        if !asked {
            let path = self.pane(side).map(|pane| pane.path.clone()).unwrap_or_default();
            path_edit::ask_save(ui.ctx(), id, t(SAVE_FILE_AS), &path);
            self.save_as = Some((side, true));
        }
        let Some(answer) = path_edit::picker_answer(ui.ctx(), id) else { return };
        self.save_as = None;
        match answer {
            Ok(Some(path)) => {
                // A failure is shown in the pane.
                let _ = self.save_to(side, &path);
            }
            Ok(None) => {}
            Err(error) => self.save_errors[side] = Some(error),
        }
        ui.ctx().request_repaint();
    }
}

/// The unsaved-edits mark beside a pane's caption. Announced: it appears as the user types, but it
/// isn't part of the editor.
pub(crate) fn mark(ui: &mut Ui, unsaved: bool) {
    if unsaved {
        let label = ui.label(t(UNSAVED_CHANGES));
        ui.ctx().accesskit_node_builder(label.id, |node| node.set_live(Live::Polite));
    }
}

/// Why the pane's last save failed, if it did. Announced, since a shortcut or the menu saved it.
pub(crate) fn error(ui: &mut Ui, error: Option<&str>) {
    if let Some(error) = error {
        let label = ui.label(error);
        ui.ctx().accesskit_node_builder(label.id, |node| node.set_live(Live::Assertive));
    }
}
