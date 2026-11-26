//! The display font size (E07-S9-T4): the hidden-by-default *Increase Font Size*, *Decrease Font
//! Size* and *Reset Font Size* change the editor text in both panes by whole points from the
//! configured size (the theme's monospace size until E22-S3's Editor Font), and reset returns to
//! it. They have no menu entry or default shortcut (E01-S1's keymap conventions: the story names
//! none, and egui's own window zoom already takes the platform's Cmd/Ctrl + plus, minus and 0), so
//! they run through the command registry until the user adds them (E22-S1).
// ponytail: the size is the tab's view state, not saved; it persists once E22-S3 makes the Editor
// Font an option.

use egui::{TextStyle, Ui};
use otd_core::command::{Command, CommandError, State};
use otd_core::i18n::t;
use otd_ui::Shell;

use crate::open::NOT_TEXT_COMPARE;
use crate::{TEXT_COMPARE, TextCompareView};

pub const INCREASE_FONT_SIZE: &str = "Increase Font Size";
pub const DECREASE_FONT_SIZE: &str = "Decrease Font Size";
pub const RESET_FONT_SIZE: &str = "Reset Font Size";

/// How many points the size may move from the configured size, each way.
const SMALLER: i8 = -6;
const LARGER: i8 = 36;
/// The smallest size drawn, whatever the configured size.
const MIN_POINTS: f32 = 4.0;

pub(crate) fn commands() -> Vec<Command<Shell>> {
    [
        Command::typed("text-compare.increase_font_size", INCREASE_FONT_SIZE, &[], |shell, _| step(shell, 1))
            .state(|shell: &Shell| when(shell, |step| step < LARGER)),
        Command::typed("text-compare.decrease_font_size", DECREASE_FONT_SIZE, &[], |shell, _| step(shell, -1))
            .state(|shell: &Shell| when(shell, |step| step > SMALLER)),
        Command::typed("text-compare.reset_font_size", RESET_FONT_SIZE, &[], |shell, _| step(shell, 0))
            .state(|shell: &Shell| when(shell, |step| step != 0)),
    ]
    .into_iter()
    .map(|command| command.views(&[TEXT_COMPARE]))
    .collect()
}

fn when(shell: &Shell, can: fn(i8) -> bool) -> State {
    State::enabled(shell.view::<TextCompareView>().is_some_and(|view| can(view.font_step)))
}

/// Moves the size by `by` points, or back to the configured size when `by` is 0.
fn step(shell: &mut Shell, by: i8) -> otd_core::command::Outcome {
    let view = shell.view_mut::<TextCompareView>().ok_or_else(|| CommandError::Failed(t(NOT_TEXT_COMPARE).to_owned()))?;
    view.font_step = if by == 0 { 0 } else { (view.font_step + by).clamp(SMALLER, LARGER) };
    Ok(None)
}

/// Draws `ui`'s editor text `step` points larger than configured.
pub(crate) fn apply(ui: &mut Ui, step: i8) {
    if step == 0 {
        return;
    }
    if let Some(font) = ui.style_mut().text_styles.get_mut(&TextStyle::Monospace) {
        font.size = (font.size + f32::from(step)).max(MIN_POINTS);
    }
}
