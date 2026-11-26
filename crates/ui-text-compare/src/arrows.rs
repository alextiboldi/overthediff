//! The toolbar's navigation arrows (E07-S2-T4): *Previous Difference Section* and *Next Difference
//! Section* move the chosen pane's caret to the start of the previous or next section of
//! differing lines, scroll both panes to it and give the pane focus. Each arrow is yellow while a
//! section lies that way from the caret in the comparison as it stands, and disabled (greyed, and
//! unavailable to screen readers and automation) when none does: the story's "dull yellow, or
//! disabled", taken as disabled. Their state is asked every frame, so it follows the caret as it
//! moves and the comparison as each edit reruns it. A section counts from either pane, including
//! one of lines only the other file has.
use otd_core::colour::{ColourRole, Contrast, Rgba};
use otd_core::command::{Command, State};
use otd_text_compare::Pos;
use otd_ui::Shell;
use otd_ui::colour::{BACKGROUND, CONTROL_ACTIVE};

use crate::{TEXT_COMPARE, TextCompareView};

pub const NEXT_SECTION: &str = "Next Difference Section";
pub const PREVIOUS_SECTION: &str = "Previous Difference Section";

/// An enabled arrow's icon: bright yellow on the dark palette, and a dark gold on the light one,
/// where a bright yellow wouldn't stand out from the toolbar.
pub const ARROW: ColourRole =
    ColourRole { id: "text-compare.navigation-arrow", light: Rgba::rgb(0x94, 0x6B, 0x00), dark: Rgba::rgb(0xFF, 0xD2, 0x33) };

/// The arrows' colour roles, which the app registers.
pub const ROLES: [ColourRole; 1] = [ARROW];

/// The toolbar the arrows sit on, at rest and under the pointer.
pub const CONTRAST: [Contrast; 2] = [Contrast::control(&ARROW, &BACKGROUND), Contrast::control(&ARROW, &CONTROL_ACTIVE)];

pub(crate) fn commands() -> Vec<Command<Shell>> {
    [
        Command::new("text-compare.previous_section", PREVIOUS_SECTION, |shell: &mut Shell| go(shell, false))
            .icon("arrow-up")
            .state(|shell: &Shell| State::enabled(target(shell, false).is_some())),
        Command::new("text-compare.next_section", NEXT_SECTION, |shell: &mut Shell| go(shell, true))
            .icon("arrow-down")
            .state(|shell: &Shell| State::enabled(target(shell, true).is_some())),
    ]
    .into_iter()
    .map(|command| command.icon_colour(&ARROW).toolbar().views(&[TEXT_COMPARE]))
    .collect()
}

/// The pane and line the next (`forward`) or previous section starts on, from the chosen pane's
/// caret. Where the section's first row is a gap in the chosen pane (lines only the other file
/// has), the other pane's line (*inferred*): a caret in a gap would have nowhere to stand.
fn target(shell: &Shell, forward: bool) -> Option<(usize, usize)> {
    let view = shell.view::<TextCompareView>()?;
    let chosen = view.chosen;
    let row = view.aligned.row_of(chosen, view.pane(chosen)?.selection.head.line)?;
    let differences = view.aligned.differences([&[], &[]]);
    let section = if forward { differences.next_section(row) } else { differences.previous_section(row) }?;
    let found = differences.rows.get(section.start)?;
    let line = |side: usize| if side == 0 { found.left } else { found.right };
    let side = if line(chosen).is_some() { chosen } else { 1 - chosen };
    Some((side, line(side)?))
}

fn go(shell: &mut Shell, forward: bool) {
    let Some((side, line)) = target(shell, forward) else { return };
    let Some(view) = shell.view_mut::<TextCompareView>() else { return };
    if let Some(pane) = view.panes.get_mut(side).and_then(Option::as_mut).and_then(|pane| pane.as_mut().ok()) {
        pane.move_caret(Pos { line, column: 0 }, false);
        view.aligned.reveal(side, line);
        view.chosen = side;
        view.focus = Some(side);
    }
}
