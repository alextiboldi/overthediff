//! *View > Word Wrap* and its toolbar button (E07-S9-T2): wraps both panes' lines to the pane width,
//! with continuation rows aligned across the panes (`aligned.rs` lays the rows out). It is a toggle,
//! off when a view opens; the menu shows it checked and the button pressed while it is on. A script
//! passes `on` to set it rather than toggle it.

use otd_core::command::{Args, Command, CommandError, Kind, Mark, Outcome, Param, State};
use otd_core::i18n::t;
use otd_core::session::Value;
use otd_ui::Shell;

use crate::open::NOT_TEXT_COMPARE;
use crate::{TEXT_COMPARE, TextCompareView};

/// The command's caption: its View menu item and toolbar button.
pub const WORD_WRAP: &str = "Word Wrap";
const PARAMS: &[Param] = &[Param::optional("on", Kind::Bool)];

pub(crate) fn commands() -> Vec<Command<Shell>> {
    vec![
        Command::typed("text-compare.word_wrap", WORD_WRAP, PARAMS, word_wrap)
            .state(|shell: &Shell| State { mark: Some(Mark::Check(shell.view::<TextCompareView>().is_some_and(|view| view.wrap))), ..State::default() })
            .icon("wrap-text")
            .menu(&["View"])
            .toolbar()
            .views(&[TEXT_COMPARE]),
    ]
}

/// Toggles Word Wrap, or sets it to `on`, and returns whether it is now on.
fn word_wrap(shell: &mut Shell, args: &Args) -> Outcome {
    let view = shell.view_mut::<TextCompareView>().ok_or_else(|| CommandError::Failed(t(NOT_TEXT_COMPARE).to_owned()))?;
    view.wrap = args.bool("on").unwrap_or(!view.wrap);
    Ok(Some(Value::Bool(view.wrap)))
}
