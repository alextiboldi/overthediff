//! *Search > Find Next* and *Find Previous* (E07-S5-T3): repeat the last search forwards or
//! backwards from the chosen pane's selection, without the find panel, and select the match. A
//! match in the other pane makes that pane the chosen one. When nothing matches, the status bar
//! says so and it is announced, and the commands fail with the same words for automation.
//!
//! The last search is what [`TextCompareView::set_search`] was last given: the find panel's Next
//! (E07-S5-T2) sets it. Both commands are disabled until there is one.
//!
//! The Text Editing option *Initialize "Text to find" to be current word* is here too:
//! [`TextCompareView::find_text_on_open`] is the text Find opens with, the word at the caret when
//! the option is on and nothing is selected. It is off by default (*inferred*: the catalogue names
//! the option but not its default).

use egui::accesskit::Live;
use egui::{Context, Rect, Role, Sense, Ui, WidgetInfo};
use otd_core::command::{Args, Command, CommandError, Key, Mods, Outcome, Platform, State};
use otd_core::i18n::{fmt, t};
use otd_core::session::Value;
use otd_text_compare::{Direction, FindOptions, Finder, Found, Line, RegexError, Selection, Side};
use otd_ui::{Field, OptionSpec, Shell};

use crate::edit::loaded_mut;
use crate::find::FIND;
use crate::open::NOT_TEXT_COMPARE;
use crate::{TEXT_COMPARE, TextCompareView};

pub const FIND_NEXT: &str = "Find Next";
pub const FIND_PREVIOUS: &str = "Find Previous";
/// The Options page the option is on.
pub const TEXT_EDITING: &str = "Text Editing";
/// The option's id and caption.
pub const CURRENT_WORD: &str = "text-compare.find_current_word";
pub const CURRENT_WORD_LABEL: &str = "Initialize \"Text to find\" to be current word";
/// What the status bar shows, and what is announced, when the search finds nothing.
pub const NOT_FOUND: &str = "Not found: {text}";
const NO_SEARCH: &str = "There is no search to repeat yet.";
pub(crate) const TOO_COMPLEX: &str = "The regular expression is too complex to search this text.";

/// The last search, which Find Next and Find Previous repeat.
pub(crate) struct Search {
    pub(crate) text: String,
    pub(crate) finder: Finder,
    /// The match it found last, so a match that selects nothing isn't found again at the caret.
    pub(crate) found: Option<Found>,
}

/// Find Next and Find Previous, in the Search menu, on each platform's chords (Q24): Cmd+G and
/// Cmd+Shift+G on macOS, F3 and Shift+F3 on Windows and Linux.
pub(crate) fn commands() -> Vec<Command<Shell>> {
    const MAC: &[Platform] = &[Platform::Mac];
    const PC: &[Platform] = &[Platform::Windows, Platform::Linux];
    let shift = |mods: Mods| Mods { shift: true, ..mods };
    let searched = |shell: &Shell| State::enabled(shell.view::<TextCompareView>().is_some_and(TextCompareView::can_find_again));
    [
        Command::typed("text-compare.find_next", FIND_NEXT, &[], |shell: &mut Shell, _: &Args| find_again(shell, Direction::Forward))
            .shortcut_on(MAC, Mods::CMD, Key::G)
            .shortcut_on(PC, Mods::NONE, Key::F3),
        Command::typed("text-compare.find_previous", FIND_PREVIOUS, &[], |shell: &mut Shell, _: &Args| find_again(shell, Direction::Backward))
            .shortcut_on(MAC, shift(Mods::CMD), Key::G)
            .shortcut_on(PC, Mods::SHIFT, Key::F3),
    ]
    .into_iter()
    .map(|command| command.state(searched).menu(&["Search"]).views(&[TEXT_COMPARE]))
    .collect()
}

/// The option, for `apps/otd/src/options.rs`.
pub fn options() -> Vec<OptionSpec> {
    vec![OptionSpec { page: &[TEXT_EDITING], id: CURRENT_WORD, label: CURRENT_WORD_LABEL, default: Value::Bool(false), choices: &[] }]
}

fn find_again(shell: &mut Shell, direction: Direction) -> Outcome {
    let view = shell.view_mut::<TextCompareView>().ok_or_else(|| CommandError::Failed(t(NOT_TEXT_COMPARE).to_owned()))?;
    view.find_again(direction).map(|_| None).map_err(CommandError::Failed)
}

impl TextCompareView {
    /// Makes `text`, with `options`, the last search, which Find Next and Find Previous repeat. An
    /// invalid regular expression is an error and leaves the last search as it was.
    pub fn set_search(&mut self, text: &str, options: FindOptions) -> Result<(), RegexError> {
        let finder = Finder::new(text, options)?;
        self.search = Some(Search { text: text.to_owned(), finder, found: None });
        self.find_said = None;
        Ok(())
    }

    /// Whether there is a last search and a file to repeat it in.
    pub fn can_find_again(&self) -> bool {
        self.search.is_some() && (0..2).any(|side| self.pane(side).is_some())
    }

    /// Find Next (`Forward`) or Find Previous: the next match of the last search from the chosen
    /// pane's selection, selected in its pane, which becomes the chosen one, takes focus and scrolls
    /// to it. When there is none, or no last search, `Err` holds what the status bar now shows.
    pub fn find_again(&mut self, direction: Direction) -> Result<Found, String> {
        let Some(mut search) = self.search.take() else { return Err(t(NO_SEARCH).to_owned()) };
        let side = self.chosen;
        let selection = self.pane(side).map(|pane| pane.selection).unwrap_or_default();
        let panes = [0, 1].map(|side| self.pane(side).map_or(&[] as &[Line], |pane| pane.lines.as_slice()));
        let searched = search.finder.find(panes, [Side::Left, Side::Right][side.min(1)], selection, direction, search.found.as_ref());
        let result = match searched {
            Ok(Some(found)) => {
                let side = usize::from(found.side == Side::Right);
                if let Some(pane) = loaded_mut(&mut self.panes, side) {
                    // Through the caret moves, so typing next starts a new undo step.
                    let Selection { anchor, head } = found.selection();
                    pane.move_caret(anchor, false);
                    pane.move_caret(head, true);
                }
                self.aligned.reveal(side, found.line);
                (self.chosen, self.focus, self.cursor) = (side, Some(side), Some(side));
                search.found = Some(found.clone());
                Ok(found)
            }
            Ok(None) => Err(fmt(t(NOT_FOUND), &[("text", &search.text)])),
            // Matching fails only when the regular expression backtracks too much.
            Err(_) => Err(t(TOO_COMPLEX).to_owned()),
        };
        self.find_said = result.as_ref().err().cloned();
        self.search = Some(search);
        result
    }

    /// The text Find opens with: with the option *Initialize "Text to find" to be current word* on
    /// and nothing selected in the chosen pane, the word at its caret. `None` leaves the find box
    /// as it was.
    pub fn find_text_on_open(&self, ctx: &Context) -> Option<String> {
        if otd_ui::option(ctx, CURRENT_WORD) != Some(Value::Bool(true)) {
            return None;
        }
        let pane = self.pane(self.chosen)?;
        let caret = pane.selection.head;
        if !pane.selection.is_empty() {
            return None;
        }
        word_at(&pane.lines.get(caret.line)?.text, caret.column).map(str::to_owned)
    }

    /// The status bar's field for a search that found nothing.
    pub(crate) fn find_field(&self) -> Option<Field> {
        self.find_said.as_ref().map(|said| Field { id: "find", label: t(FIND).to_owned(), value: said.clone() })
    }

    /// A search that found nothing, as a polite live region, so it is heard while focus stays put;
    /// the status bar shows it to everyone else.
    pub(crate) fn find_said_ui(&self, ui: &mut Ui) {
        if let Some(said) = &self.find_said {
            let response = ui.interact(Rect::from_min_size(ui.cursor().min, egui::Vec2::ZERO), ui.scope_id().with("find-said"), Sense::hover());
            response.widget_info(|| WidgetInfo::labeled(Role::Label, true, said));
            ui.ctx().accesskit_node_builder(response.id, |node| node.set_live(Live::Polite));
        }
    }
}

/// The word around character column `column` of `text`: the run of letters, digits and
/// underscores (the regex `\w`) the column touches. `None` between two non-word characters.
fn word_at(text: &str, column: usize) -> Option<&str> {
    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let column = column.min(chars.len());
    let start = chars[..column].iter().rposition(|&(_, c)| !is_word(c)).map_or(0, |i| i + 1);
    let end = chars[column..].iter().position(|&(_, c)| !is_word(c)).map_or(chars.len(), |i| column + i);
    let byte = |i: usize| chars.get(i).map_or(text.len(), |&(at, _)| at);
    (start < end).then(|| &text[byte(start)..byte(end)])
}

#[cfg(test)]
mod tests {
    use super::word_at;

    #[test]
    fn the_word_at_a_column_is_the_run_of_word_characters_it_touches() {
        assert_eq!(word_at("let foo = 1;", 5), Some("foo"), "inside");
        assert_eq!(word_at("let foo = 1;", 4), Some("foo"), "at its start");
        assert_eq!(word_at("let foo = 1;", 7), Some("foo"), "at its end");
        assert_eq!(word_at("a  b", 2), None, "between two spaces");
        assert_eq!(word_at("größe_2 x", 3), Some("größe_2"), "letters beyond ASCII and underscores");
        assert_eq!(word_at("foo", 99), Some("foo"), "a column past the end is the end");
        assert_eq!(word_at("", 0), None);
    }
}
