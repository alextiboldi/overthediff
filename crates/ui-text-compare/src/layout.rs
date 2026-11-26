//! The View menu's *Side-by-Side* and *Top and Bottom* layouts (E07-S9-T3; the feature catalogue's
//! working name for the second is Over-Under, renamed by the naming check in docs/feature-names.md):
//! the left pane beside the right one, or above it. Each pane keeps its widget ids in both layouts, so switching keeps its
//! scroll position, caret and focus; the caret's line is scrolled back into view once the panes
//! have their new size.

use egui::{Rect, Ui, pos2, vec2};
use otd_core::command::{Command, Mark, State};
use otd_ui::Shell;

use crate::{TEXT_COMPARE, TextCompareView};

pub const SIDE_BY_SIDE: &str = "Side-by-Side";
pub const TOP_AND_BOTTOM: &str = "Top and Bottom";

/// How the two panes are arranged.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Layout {
    /// Left and right.
    #[default]
    SideBySide,
    /// Left on top, right below.
    OverUnder,
}

/// The View menu's two layout commands, a radio pair. Neither has a default shortcut: Q24 names
/// none for them.
pub(crate) fn commands() -> Vec<Command<Shell>> {
    vec![
        Command::new("text-compare.side_by_side", SIDE_BY_SIDE, |shell| set(shell, Layout::SideBySide))
            .icon("columns-2")
            .state(|shell| mark(shell, Layout::SideBySide)),
        Command::new("text-compare.over_under", TOP_AND_BOTTOM, |shell| set(shell, Layout::OverUnder))
            .icon("rows-2")
            .state(|shell| mark(shell, Layout::OverUnder)),
    ]
    .into_iter()
    .map(|command| command.views(&[TEXT_COMPARE]).menu(&["View"]))
    .collect()
}

fn set(shell: &mut Shell, layout: Layout) {
    if let Some(view) = shell.view_mut::<TextCompareView>() {
        view.set_layout(layout);
    }
}

fn mark(shell: &Shell, layout: Layout) -> State {
    let on = shell.view::<TextCompareView>().is_some_and(|view| view.layout == layout);
    State { mark: Some(Mark::Radio(on)), ..State::default() }
}

impl TextCompareView {
    /// The panes' layout.
    pub fn layout(&self) -> Layout {
        self.layout
    }

    /// Arranges the panes by `layout`, keeping the caret's line in view.
    pub fn set_layout(&mut self, layout: Layout) {
        if self.layout == layout {
            return;
        }
        self.layout = layout;
        let caret = self.cursor.and_then(|side| Some((side, self.pane(side)?.selection.head.line)));
        self.aligned.relayout(caret);
    }
}

/// Each pane's place in `rect`: two columns, or two rows, `gap` apart, whole points wide or tall so
/// both panes' rows land on the same pixel offsets.
pub(crate) fn slots(layout: Layout, rect: Rect, gap: f32) -> [Rect; 2] {
    match layout {
        Layout::SideBySide => {
            let width = ((rect.width() - gap) / 2.0).floor().max(0.0);
            [rect.left(), rect.left() + width + gap].map(|x| Rect::from_min_size(pos2(x, rect.top()), vec2(width, rect.height())))
        }
        Layout::OverUnder => {
            let height = ((rect.height() - gap) / 2.0).floor().max(0.0);
            [rect.top(), rect.top() + height + gap].map(|y| Rect::from_min_size(pos2(rect.left(), y), vec2(rect.width(), height)))
        }
    }
}

/// Each pane body's place: below its header, which is `headers` tall, in its slot. Both bodies get
/// one height, below the taller header, so the panes' viewports match and their shared scroll
/// offset reaches the end of both.
pub(crate) fn bodies(layout: Layout, slots: [Rect; 2], headers: [f32; 2], ui: &Ui) -> [Rect; 2] {
    let header = headers[0].max(headers[1]) + ui.spacing().item_spacing.y;
    [0, 1].map(|side| {
        let slot = slots[side];
        let top = match layout {
            // Side by side, both start on one line, so their rows line up.
            Layout::SideBySide => slot.top() + header,
            Layout::OverUnder => slot.top() + headers[side] + ui.spacing().item_spacing.y,
        };
        let height = (slot.height() - header).max(0.0);
        Rect::from_min_size(pos2(slot.left(), top), vec2(slot.width(), height))
    })
}
