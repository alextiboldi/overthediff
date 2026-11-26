//! *Search > Find* (E07-S5-T2, catalogue F-12.5): a find panel above the panes. Its *Find what*
//! box keeps the view's earlier searches in a history list, and with Regular expression on, the
//! regex input aid (`otd-ui-regex`) beside it inserts A.2 syntax at the box's cursor. Its options
//! are Regular expression, Match character case, Whole words only, Wrap search, the sides to
//! search (left, right or both) and Close automatically, which hides the panel once a search
//! starts. **Next** selects the next match (`otd_text_compare::Finder`); **Select All** selects
//! every line on the searched sides that holds a match. An invalid pattern says why and searches
//! nothing.
//!
//! The panel isn't modal: the panes stay usable while it is open. Esc or Close hides it and gives
//! focus back. The hidden *Find Text* takes the text and options as arguments and runs Next or
//! Select All, so a script can search without the window.
//!
//! Next and Select All make their search the last one, which Find Next and Find Previous repeat
//! (`find_next.rs`, E07-S5-T3), and Find opens with the word at the caret when the option
//! *Initialize "Text to find" to be current word* is on.
//!
//! Lines Select All selects are a selection of their own, beside each pane's caret selection,
//! drawn in the selection's colours and marked selected for assistive technology. They go as soon
//! as the pane's caret moves or either pane's text changes (*inferred*). They are only a
//! highlight: typing, Copy, Cut and Delete still act at the caret, and Select All says so in the
//! panel and in Find Text's result. Making them act on the lines is issue #149.

use egui::accesskit::{Live, Role};
use egui::{Id, Key, TextEdit, Ui, UiBuilder};
use otd_core::command::{Args, Command, CommandError, Key as Chord, Kind, Mods, Outcome, Param, State};
use otd_core::i18n::{fmt, t};
use otd_core::session::Value;
use otd_text_compare::{Direction, FindOptions, Finder, Found, RegexError, Selection, Side, Sides};
use otd_ui::Shell;

use crate::edit::{SELECT_ALL, loaded_mut};
use crate::find_next::{NOT_FOUND, Search, TOO_COMPLEX};
use crate::open::NOT_TEXT_COMPARE;
use crate::{LEFT, RIGHT, TEXT_COMPARE, TextCompareView};

/// The menu command's caption, and the panel's name.
pub const FIND: &str = "Find";
/// The hidden command that takes the text and options as arguments.
pub const FIND_TEXT: &str = "Find Text";
pub const FIND_WHAT: &str = "Find what";
pub const SEARCH_HISTORY: &str = "Search history";
pub const NO_SEARCHES: &str = "No earlier searches";
pub const REGULAR_EXPRESSION: &str = "Regular expression";
pub const MATCH_CASE: &str = "Match character case";
pub const WHOLE_WORDS: &str = "Whole words only";
pub const WRAP_SEARCH: &str = "Wrap search";
pub const SEARCH_IN: &str = "Search in";
pub const BOTH: &str = "Both";
pub const CLOSE_AUTOMATICALLY: &str = "Close automatically";
pub const NEXT: &str = "Next";
pub const CLOSE: &str = "Close";
pub const LINES_SELECTED: &str = "Lines selected: {count}. They are only highlighted: typing, Copy, Cut and Delete still act at the caret.";
pub const INVALID_PATTERN: &str = "The regular expression can't be used: {problem} (at character {at}).";
pub const NOTHING_TO_FIND: &str = "Type the text to find.";

/// How many earlier searches the history keeps, newest first.
// ponytail: per view and not saved; keep it in the settings store, as the path edits keep their
// recent paths, if users want it across tabs and restarts.
const HISTORY: usize = 20;

const SIDE_CHOICES: &[&str] = &[LEFT, RIGHT, BOTH];
const PARAMS: &[Param] = &[
    Param::required("text", Kind::Text),
    Param::optional("regex", Kind::Bool),
    Param::optional("match_case", Kind::Bool),
    Param::optional("whole_words", Kind::Bool),
    Param::optional("wrap", Kind::Bool),
    Param::optional("sides", Kind::Choice(SIDE_CHOICES)),
    Param::optional("select_all", Kind::Bool),
];

/// *Find* in the Search menu (Cmd/Ctrl+F, Q24) and the hidden *Find Text*.
pub(crate) fn commands() -> Vec<Command<Shell>> {
    let loaded = |shell: &Shell| State::enabled(shell.view::<TextCompareView>().is_some_and(|view| view.pane(0).is_some() || view.pane(1).is_some()));
    vec![
        Command::new("text-compare.find", FIND, |shell: &mut Shell| {
            if let Some(view) = shell.view_mut::<TextCompareView>() {
                view.open_find();
            }
        })
        .state(loaded)
        .icon("search")
        .menu(&["Search"])
        .shortcut(Mods::CMD, Chord::F)
        .needs_window(),
        Command::typed("text-compare.find_text", FIND_TEXT, PARAMS, find_text).state(loaded),
    ]
    .into_iter()
    .map(|command| command.views(&[TEXT_COMPARE]))
    .collect()
}

/// Sets the panel's text and options from `args`, then runs Next, or Select All with
/// `select_all`. Next returns whether it found a match, Select All what the panel says: how many
/// lines it selected, and that only the highlight acts on them.
fn find_text(shell: &mut Shell, args: &Args) -> Outcome {
    let view = shell.view_mut::<TextCompareView>().ok_or_else(|| CommandError::Failed(t(NOT_TEXT_COMPARE).to_owned()))?;
    let find = &mut view.find;
    find.text = args.text("text").unwrap_or_default().to_owned();
    let flag = |name: &str| args.bool(name).unwrap_or(false);
    find.options = FindOptions {
        regex: flag("regex"),
        match_case: flag("match_case"),
        whole_words: flag("whole_words"),
        wrap: flag("wrap"),
        sides: match args.text("sides") {
            Some(LEFT) => Sides::Left,
            Some(RIGHT) => Sides::Right,
            _ => Sides::Both,
        },
    };
    let failed = |error: String| CommandError::Failed(error);
    if flag("select_all") {
        view.select_all_matches().map(|_| view.find.said.clone().map(Value::Text)).map_err(failed)
    } else {
        view.find_next().map(|found| Some(Value::Bool(found.is_some()))).map_err(failed)
    }
}

/// The find panel's state, kept while it is hidden so it opens as it was left.
#[derive(Default)]
pub(crate) struct Find {
    /// The panel shows.
    open: bool,
    text: String,
    options: FindOptions,
    close_automatically: bool,
    /// Earlier searches, newest first.
    history: Vec<String>,
    /// What the last search said: nothing found, an invalid pattern, the lines selected. Announced.
    said: Option<String>,
    /// The box takes the word at the caret on the next draw, if the option asks for it.
    fill: bool,
    /// The *Find what* box takes focus on the next draw.
    focus: bool,
    /// The panel has been drawn since it opened.
    drawn: bool,
    /// What had focus when it opened, which gets it back when it closes.
    focus_back: Option<Id>,
    /// A control of the panel had focus at the end of the last draw, so Esc is the panel's.
    had_focus: bool,
    /// The panel closed: on the next draw, focus goes back to this, or with `None` to the chosen
    /// pane. Not at once, so the key that closed it doesn't reach the pane too.
    give_back: Option<Option<Id>>,
    /// Per side, the lines Select All selected, ascending, and the caret selection they go with.
    lines: [Vec<usize>; 2],
    lines_at: [Option<Selection>; 2],
}

impl Find {
    /// The lines of side `side` that Select All selected, ascending.
    pub(crate) fn lines(&self, side: usize) -> &[usize] {
        self.lines.get(side).map_or(&[], Vec::as_slice)
    }

    /// Forgets the lines Select All selected.
    pub(crate) fn clear_lines(&mut self) {
        self.lines = Default::default();
        self.lines_at = [None; 2];
    }

    fn remember(&mut self) {
        let text = self.text.clone();
        self.history.retain(|earlier| *earlier != text);
        self.history.insert(0, text);
        self.history.truncate(HISTORY);
    }
}

/// `error` in words, with where in `text` it is, counted in characters from 1.
fn invalid(text: &str, error: &RegexError) -> String {
    match error {
        RegexError::Syntax { at, problem } => {
            let at = text.get(..*at).map_or(0, |before| before.chars().count()) + 1;
            fmt(t(INVALID_PATTERN), &[("problem", t(problem)), ("at", &at.to_string())])
        }
        RegexError::TooComplex => t(TOO_COMPLEX).to_owned(),
    }
}

fn side_of(index: usize) -> Side {
    if index == 0 { Side::Left } else { Side::Right }
}

impl TextCompareView {
    /// *Find*: shows the find panel with focus in its *Find what* box.
    pub fn open_find(&mut self) {
        let find = &mut self.find;
        if !find.open {
            find.drawn = false;
        }
        find.open = true;
        find.focus = true;
        find.fill = true;
    }

    /// Whether the find panel shows.
    pub fn find_open(&self) -> bool {
        self.find.open
    }

    /// The lines of side `side` (0 is left) that the find panel's Select All selected, ascending.
    pub fn selected_lines(&self, side: usize) -> &[usize] {
        self.find.lines(side)
    }

    /// Compiles the panel's text with its options, after keeping it in the history. Empty text is
    /// refused: it would match every line.
    fn finder(&mut self) -> Result<Finder, String> {
        let find = &mut self.find;
        if find.text.is_empty() {
            return Err(t(NOTHING_TO_FIND).to_owned());
        }
        find.remember();
        Finder::new(&find.text, find.options).map_err(|error| invalid(&find.text, &error))
    }

    /// Both panes' lines; a missing pane's are empty.
    fn lines(&self) -> [&[otd_text_compare::Line]; 2] {
        [0, 1].map(|side| self.pane(side).map_or(&[][..], |pane| pane.lines.as_slice()))
    }

    /// **Next**: selects the next match after the chosen pane's selection, on the searched sides,
    /// and scrolls to it. Returns the match, `None` when there is none (said in the panel), or why
    /// the text can't be searched for (said there too).
    pub fn find_next(&mut self) -> Result<Option<Found>, String> {
        let searched = self.finder().and_then(|finder| {
            let selection = self.pane(self.chosen).map(|pane| pane.selection).unwrap_or_default();
            // The match found last, so a match that selects nothing is stepped past once.
            let last = self.search.as_ref().and_then(|search| search.found.as_ref());
            let found = finder.find(self.lines(), side_of(self.chosen), selection, Direction::Forward, last);
            Ok((finder, found.map_err(|error| invalid(&self.find.text, &error))?))
        });
        let found = searched.map(|(finder, found)| {
            // The last search, which Find Next and Find Previous repeat.
            self.search = Some(Search { text: self.find.text.clone(), finder, found: found.clone() });
            self.find_said = None;
            found
        });
        self.find.said = match &found {
            Ok(Some(found)) => {
                let side = usize::from(found.side == Side::Right);
                if let Some(pane) = loaded_mut(&mut self.panes, side) {
                    // Through the caret moves, so typing next starts a new undo step.
                    let Selection { anchor, head } = found.selection();
                    pane.move_caret(anchor, false);
                    pane.move_caret(head, true);
                }
                self.chosen = side;
                self.cursor = Some(side);
                self.aligned.reveal(side, found.line);
                None
            }
            Ok(None) => Some(fmt(t(NOT_FOUND), &[("text", &self.find.text)])),
            Err(error) => Some(error.clone()),
        };
        found
    }

    /// **Select All**: selects every line on the searched sides that holds a match, puts each
    /// pane's caret at the start of its first such line and scrolls to the first. Returns how many
    /// lines it selected, or why the text can't be searched for. Either is said in the panel.
    pub fn select_all_matches(&mut self) -> Result<usize, String> {
        self.find.clear_lines();
        let all = self.finder().and_then(|finder| Ok((finder.find_all(self.lines()).map_err(|error| invalid(&self.find.text, &error))?, finder)));
        let all = match all {
            Ok((all, finder)) => {
                self.search = Some(Search { text: self.find.text.clone(), finder, found: None });
                self.find_said = None;
                all
            }
            Err(error) => {
                self.find.said = Some(error.clone());
                return Err(error);
            }
        };
        let mut lines: [Vec<usize>; 2] = Default::default();
        for found in all {
            let side = &mut lines[usize::from(found.side == Side::Right)];
            // In search order, so a line's matches are next to each other.
            if side.last() != Some(&found.line) {
                side.push(found.line);
            }
        }
        let count: usize = lines.iter().map(Vec::len).sum();
        for (side, lines) in lines.iter().enumerate() {
            let Some(&first) = lines.first() else { continue };
            if let Some(pane) = loaded_mut(&mut self.panes, side) {
                pane.move_caret(otd_text_compare::Pos { line: first, column: 0 }, false);
                self.find.lines_at[side] = Some(pane.selection);
            }
        }
        if let Some(side) = lines.iter().position(|lines| !lines.is_empty()) {
            self.chosen = side;
            self.cursor = Some(side);
            self.aligned.reveal(side, lines[side][0]);
        }
        self.find.lines = lines;
        self.find.said =
            Some(if count == 0 { fmt(t(NOT_FOUND), &[("text", &self.find.text)]) } else { fmt(t(LINES_SELECTED), &[("count", &count.to_string())]) });
        Ok(count)
    }

    /// Hides the panel and gives focus back to what had it, or to the chosen pane.
    fn close_find(&mut self) {
        self.find.open = false;
        self.find.had_focus = false;
        self.find.give_back = Some(self.find.focus_back.take());
    }

    /// Draws the find panel above the panes while it is open. Called on every draw before the
    /// panes, also while it is hidden: a pane whose caret moved loses the lines Select All chose.
    pub(crate) fn find_ui(&mut self, ui: &mut Ui) {
        for side in 0..2 {
            let at = self.pane(side).map(|pane| pane.selection);
            if self.find.lines_at[side].is_some() && self.find.lines_at[side] != at {
                self.find.lines[side].clear();
                self.find.lines_at[side] = None;
                self.find.said = None;
                self.find_said = None;
            }
        }
        match self.find.give_back.take() {
            Some(Some(id)) => self.refocus = Some(id),
            Some(None) => self.focus = Some(self.chosen),
            None => {}
        }
        if !self.find.open {
            return;
        }
        if !self.find.drawn {
            self.find.drawn = true;
            self.find.focus_back = ui.memory(|memory| memory.focused());
        }
        if std::mem::take(&mut self.find.fill)
            && let Some(word) = self.find_text_on_open(ui.ctx())
        {
            self.find.text = word;
        }
        // Esc while a control of the panel has focus closes it (egui drops that focus first),
        // unless it closes the history list or a regex menu.
        if self.find.had_focus && !egui::Popup::is_any_open(ui.ctx()) && ui.input(|i| i.key_pressed(Key::Escape)) {
            self.find.said = None;
            self.close_find();
            ui.ctx().request_repaint();
            return;
        }
        let builder = UiBuilder::new().id_salt("find-panel").accessibility_role(Role::Search).accessibility_label(t(FIND));
        let panel = ui.scope_builder(builder, |ui| self.panel(ui));
        let (next, select_all, close) = panel.inner;
        ui.separator();
        let focused = ui.memory(|memory| memory.focused()).and_then(|id| ui.ctx().read_response(id));
        self.find.had_focus = focused.is_some_and(|focused| panel.response.rect.contains_rect(focused.rect));
        if !(next || select_all || close) {
            return;
        }
        ui.ctx().request_repaint();
        if close {
            self.find.said = None;
            self.close_find();
            return;
        }
        // Started: the text was there and compiled, whether or not it was found.
        let started = if next { self.find_next().is_ok() } else { self.select_all_matches().is_ok() };
        if started && self.find.close_automatically {
            // What it said goes to the status bar, shown and announced while focus is in the pane.
            self.find_said = self.find.said.take();
            self.find.focus_back = None;
            self.close_find();
        }
    }

    /// The panel's controls. Returns whether Next, Select All or Close was chosen.
    fn panel(&mut self, ui: &mut Ui) -> (bool, bool, bool) {
        let field = ui.make_persistent_id("find-what");
        let find = &mut self.find;
        let mut next = false;
        ui.horizontal(|ui| {
            let label = ui.label(t(FIND_WHAT));
            let response = ui.add(TextEdit::singleline(&mut find.text).id(field).desired_width(240.0)).labelled_by(label.id);
            // Enter gives up the box's focus; it searches and keeps focus there.
            if response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                next = true;
                find.focus = true;
            }
            if std::mem::take(&mut find.focus) {
                response.request_focus();
            }
            let history = egui::ComboBox::from_id_salt(field.with("history")).selected_text("").width(0.0).show_ui(ui, |ui| {
                if find.history.is_empty() {
                    ui.add_enabled(false, egui::Label::new(t(NO_SEARCHES)));
                }
                let mut chosen = None;
                for earlier in &find.history {
                    if ui.selectable_label(false, earlier).clicked() {
                        chosen = Some(earlier.clone());
                        // egui's list closes itself on a pointer click only; this closes it on Enter and Space too.
                        ui.close();
                    }
                }
                chosen
            });
            // egui drops focus on Esc; closing the list with it puts focus back on the list button.
            if history.inner.is_some() && ui.input(|i| i.key_pressed(Key::Escape)) {
                history.response.request_focus();
            }
            ui.ctx().accesskit_node_builder(history.response.id, |node| node.set_label(t(SEARCH_HISTORY)));
            if let Some(text) = history.inner.flatten() {
                find.text = text;
                response.request_focus();
            }
            if find.options.regex {
                otd_ui_regex::regex_aid(ui, field, &mut find.text);
            }
        });
        ui.horizontal_wrapped(|ui| {
            let options = &mut find.options;
            for (caption, value) in [
                (REGULAR_EXPRESSION, &mut options.regex),
                (MATCH_CASE, &mut options.match_case),
                (WHOLE_WORDS, &mut options.whole_words),
                (WRAP_SEARCH, &mut options.wrap),
                (CLOSE_AUTOMATICALLY, &mut find.close_automatically),
            ] {
                ui.checkbox(value, t(caption));
            }
        });
        let (select_all, close) = ui
            .horizontal(|ui| {
                ui.label(t(SEARCH_IN));
                // A group named by its label, so its Left and Right read apart from the panes'.
                let group = UiBuilder::new().accessibility_role(Role::RadioGroup).accessibility_label(t(SEARCH_IN));
                ui.scope_builder(group, |ui| {
                    ui.horizontal(|ui| {
                        for (caption, sides) in [(LEFT, Sides::Left), (RIGHT, Sides::Right), (BOTH, Sides::Both)] {
                            ui.radio_value(&mut find.options.sides, sides, t(caption));
                        }
                    })
                });
                ui.separator();
                let next_button = ui.button(t(NEXT));
                let select_all = ui.button(t(SELECT_ALL));
                let close = ui.button(t(CLOSE));
                next |= next_button.clicked();
                (select_all.clicked(), close.clicked())
            })
            .inner;
        if let Some(said) = &find.said {
            let label = ui.label(said);
            // Announced: it comes from the search, while focus stays on the control that ran it.
            ui.ctx().accesskit_node_builder(label.id, |node| node.set_live(Live::Polite));
        }
        (next, select_all, close)
    }
}
