//! *Search > Next Edit* and *Previous Edit* (E07-S2-T3): the chosen pane's caret goes to the next or
//! previous run of lines edited since its file was loaded ([`Pane::next_edit`]), the panes scroll to
//! it and the pane takes focus. Each is disabled when there is no edit that way.

use otd_core::command::{Command, Key, Mods, State};
use otd_text_compare::Pos;
use otd_ui::Shell;

use crate::{TEXT_COMPARE, TextCompareView};

pub const NEXT_EDIT: &str = "Next Edit";
pub const PREVIOUS_EDIT: &str = "Previous Edit";

/// The two commands. Q24 names no chord for them, and the story asks for one: Alt+Shift+Down and
/// Up are in neither craft-keymap's `text_editing_chords` nor its reserved lists, so they reach the
/// shell while a pane has focus. (Cocoa text fields extend the selection by paragraph with them;
/// the panes have no paragraph moves.)
pub(crate) fn commands() -> Vec<Command<Shell>> {
    let alt_shift = Mods { shift: true, ..Mods::ALT };
    [
        Command::new("text-compare.next_edit", NEXT_EDIT, |shell: &mut Shell| go(shell, true))
            .state(|shell: &Shell| State::enabled(target(shell, true).is_some()))
            .shortcut(alt_shift, Key::Down),
        Command::new("text-compare.previous_edit", PREVIOUS_EDIT, |shell: &mut Shell| go(shell, false))
            .state(|shell: &Shell| State::enabled(target(shell, false).is_some()))
            .shortcut(alt_shift, Key::Up),
    ]
    .into_iter()
    .map(|command| command.menu(&["Search"]).views(&[TEXT_COMPARE]))
    .collect()
}

/// The line Next (`forward`) or Previous Edit goes to in the chosen pane, from its caret's line.
fn target(shell: &Shell, forward: bool) -> Option<usize> {
    let view = shell.view::<TextCompareView>()?;
    let pane = view.pane(view.chosen)?;
    let from = pane.selection.head.line;
    if forward { pane.next_edit(from) } else { pane.previous_edit(from) }
}

fn go(shell: &mut Shell, forward: bool) {
    let Some(line) = target(shell, forward) else { return };
    let Some(view) = shell.view_mut::<TextCompareView>() else { return };
    let side = view.chosen;
    if let Some(pane) = view.panes.get_mut(side).and_then(Option::as_mut).and_then(|pane| pane.as_mut().ok()) {
        pane.move_caret(Pos { line, column: 0 }, false);
        view.aligned.reveal(side, line);
        view.focus = Some(side);
    }
}
