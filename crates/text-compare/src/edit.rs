//! Editing a pane (E07-S1-T5): a caret and a selection, edits that replace the selection with
//! text, and undo history. Every edit is one splice of whole lines, so Undo and Redo are the
//! inverse splice and the caret goes back with it.
//!
//! Columns count characters, never bytes, so a column can't fall inside a character. When the last
//! line ends with a line break, the caret can also stand after it, on line `lines.len()`, as in
//! any editor: that place has no line of its own, so it doesn't show as an inserted blank line.

use otd_text_engine::{Line, LineEnding};

use crate::{LineEndings, Pane};

/// A place between two characters: a line index and a column in characters.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pos {
    pub line: usize,
    pub column: usize,
}

/// The selected text, from `anchor` to `head`, where the caret is. Empty when they are equal.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Selection {
    pub anchor: Pos,
    pub head: Pos,
}

impl Selection {
    /// No text selected, the caret at `at`.
    pub fn caret(at: Pos) -> Self {
        Self { anchor: at, head: at }
    }

    pub fn is_empty(&self) -> bool {
        self.anchor == self.head
    }

    /// Its start and end, in document order.
    pub fn range(&self) -> (Pos, Pos) {
        (self.anchor.min(self.head), self.anchor.max(self.head))
    }
}

/// One edit: at line `at`, `removed` was replaced with `inserted`.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Step {
    at: usize,
    removed: Vec<Line>,
    inserted: Vec<Line>,
    before: Selection,
    after: Selection,
}

/// A pane's undo history.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct History {
    undo: Vec<Step>,
    redo: Vec<Step>,
    /// Whether the last step was typed with the caret left where it ended, so typing goes on in it
    /// and one Undo takes back a typed word.
    typing: bool,
}

impl History {
    /// Forgets every step, as Recompare Files does (E07-S1-T8).
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

impl Pane {
    fn line_len(&self, line: usize) -> usize {
        self.lines.get(line).map_or(0, |line| line.text.chars().count())
    }

    /// The caret's last line: after the final line break when the text ends with one.
    fn last_line(&self) -> usize {
        match self.lines.last() {
            Some(last) if last.ending == LineEnding::None => self.lines.len() - 1,
            _ => self.lines.len(),
        }
    }

    /// `pos` moved inside the text.
    pub fn clamp(&self, pos: Pos) -> Pos {
        let line = pos.line.min(self.last_line());
        Pos { line, column: pos.column.min(self.line_len(line)) }
    }

    /// The end of the text.
    pub fn end(&self) -> Pos {
        let line = self.last_line();
        Pos { line, column: self.line_len(line) }
    }

    /// The end of line `line`.
    pub fn line_end(&self, line: usize) -> Pos {
        self.clamp(Pos { line, column: usize::MAX })
    }

    /// One character before `pos`, across a line break; `pos` itself at the start.
    pub fn before(&self, pos: Pos) -> Pos {
        let pos = self.clamp(pos);
        match (pos.column, pos.line) {
            (0, 0) => pos,
            (0, line) => self.line_end(line - 1),
            (column, line) => Pos { line, column: column - 1 },
        }
    }

    /// One character after `pos`, across a line break; `pos` itself at the end.
    pub fn after(&self, pos: Pos) -> Pos {
        let pos = self.clamp(pos);
        if pos.column < self.line_len(pos.line) {
            Pos { column: pos.column + 1, ..pos }
        } else if pos.line < self.last_line() {
            Pos { line: pos.line + 1, column: 0 }
        } else {
            pos
        }
    }

    /// Moves the caret to `to`. With `extend` the selection's anchor stays, so the selection grows
    /// or shrinks to `to`.
    pub fn move_caret(&mut self, to: Pos, extend: bool) {
        let head = self.clamp(to);
        self.selection = if extend { Selection { anchor: self.clamp(self.selection.anchor), head } } else { Selection::caret(head) };
        self.history.typing = false;
    }

    /// Selects every line. With display filters (E07-S3) it will be the lines the pane shows; until
    /// then the pane shows them all.
    pub fn select_all(&mut self) {
        self.selection = Selection { anchor: Pos::default(), head: self.end() };
        self.history.typing = false;
    }

    /// The selected text, each line break as its line's own ending.
    pub fn selected_text(&self) -> String {
        let (start, end) = self.selection.range();
        let mut text = String::new();
        for (index, line) in self.lines.iter().enumerate().take(end.line + 1).skip(start.line) {
            let from = if index == start.line { start.column } else { 0 };
            let to = if index == end.line { end.column } else { usize::MAX };
            text.extend(line.text.chars().skip(from).take(to.saturating_sub(from)));
            if index < end.line {
                text.push_str(ending_text(line.ending));
            }
        }
        text
    }

    /// Typing: replaces the selection with `text`. Typed text joins the step before it while the
    /// caret stays where that step left it, within one line.
    pub fn type_text(&mut self, text: &str) {
        self.replace(text, true);
    }

    /// Typing in overwrite mode: `text` replaces as many characters after the caret, up to the
    /// line's end. A selection is replaced as in insert mode.
    pub fn overwrite(&mut self, text: &str) {
        if self.selection.is_empty() {
            let head = self.clamp(self.selection.head);
            let column = head.column.saturating_add(text.chars().count()).min(self.line_len(head.line));
            self.selection = Selection { anchor: Pos { column, ..head }, head };
        }
        self.type_text(text);
    }

    /// Replaces the selection with `text` (a paste, or Enter's line break) as a step of its own.
    /// Line breaks of any style become the pane's.
    pub fn insert(&mut self, text: &str) {
        self.replace(text, false);
    }

    /// Deletes the selection, or with none the character before the caret (Backspace) or after it
    /// (Delete), joining two lines at a line break.
    pub fn delete(&mut self, forward: bool) {
        if self.selection.is_empty() {
            let head = self.clamp(self.selection.head);
            let other = if forward { self.after(head) } else { self.before(head) };
            self.selection = Selection { anchor: other, head };
        }
        self.replace("", false);
    }

    pub fn can_undo(&self) -> bool {
        !self.history.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.history.redo.is_empty()
    }

    /// Takes back the last step, if there is one.
    pub fn undo(&mut self) -> bool {
        let Some(step) = self.history.undo.pop() else { return false };
        if !self.splice(step.at, step.inserted.len(), step.removed.clone()) {
            return false;
        }
        // The inverse edit: from the step's start to the end of what it inserted, the text after
        // that going back to the line it came from.
        let (start, end) = step.before.range();
        self.bookmarks.follow(start, step.after.head, end.line, self.lines.len());
        self.selection = step.before;
        self.history.redo.push(step);
        self.history.typing = false;
        true
    }

    /// Makes the last undone step again, if there is one.
    pub fn redo(&mut self) -> bool {
        let Some(step) = self.history.redo.pop() else { return false };
        if !self.splice(step.at, step.removed.len(), step.inserted.clone()) {
            return false;
        }
        let (start, end) = step.before.range();
        self.bookmarks.follow(start, end, step.after.head.line, self.lines.len());
        self.selection = step.after;
        self.history.undo.push(step);
        self.history.typing = false;
        true
    }

    /// Replaces `count` lines at `at` with `with`. A step that no longer fits the lines (they were
    /// changed without going through these edits) empties the history instead of panicking.
    fn splice(&mut self, at: usize, count: usize, with: Vec<Line>) -> bool {
        if at.checked_add(count).is_none_or(|end| end > self.lines.len()) {
            self.history.clear();
            return false;
        }
        self.lines.splice(at..at + count, with);
        self.unsaved = true;
        true
    }

    fn replace(&mut self, text: &str, typing: bool) {
        let before = Selection { anchor: self.clamp(self.selection.anchor), head: self.clamp(self.selection.head) };
        let (start, end) = before.range();
        if start == end && text.is_empty() {
            return;
        }
        // The place after the final line break, and an empty pane's only place, have no line yet:
        // the edit makes it.
        let removed: Vec<Line> = self.lines.get(start.line..(end.line + 1).min(self.lines.len())).map(<[Line]>::to_vec).unwrap_or_default();
        let reaches_end = start.line + removed.len() == self.lines.len();
        let prefix: String = self.lines.get(start.line).map(|line| line.text.chars().take(start.column).collect()).unwrap_or_default();
        let suffix: String = self.lines.get(end.line).map(|line| line.text.chars().skip(end.column).collect()).unwrap_or_default();
        let last_ending = self.lines.get(end.line).map_or(LineEnding::None, |line| line.ending);
        // A new break takes the ending of the line it splits; the last line has none, so then the
        // pane's style.
        let ending = if last_ending == LineEnding::None { style(self.endings) } else { last_ending };
        let parts: Vec<&str> = text.split("\r\n").flat_map(|part| part.split(['\r', '\n'])).collect();
        let mut inserted = Vec::with_capacity(parts.len());
        let mut head = start;
        for (i, part) in parts.iter().enumerate() {
            let mut line = if i == 0 { prefix.clone() } else { String::new() };
            line.push_str(part);
            let last = i + 1 == parts.len();
            if last {
                head = Pos { line: start.line + i, column: line.chars().count() };
                line.push_str(&suffix);
            }
            inserted.push(Line { text: line, ending: if last { last_ending } else { ending } });
        }
        // An empty last line without a break is no line at all: the text ends with a break, and the
        // caret stands after it.
        if reaches_end && inserted.last().is_some_and(|line| line.text.is_empty() && line.ending == LineEnding::None) {
            inserted.pop();
        }
        if !self.splice(start.line, removed.len(), inserted.clone()) {
            return;
        }
        self.bookmarks.follow(start, end, head.line, self.lines.len());
        let after = Selection::caret(head);
        let joins = typing && self.history.typing && removed.len() == 1 && inserted.len() == 1;
        match self.history.undo.last_mut() {
            Some(step) if joins && step.at == start.line && step.inserted.len() == 1 => {
                step.inserted = inserted;
                step.after = after;
            }
            _ => self.history.undo.push(Step { at: start.line, removed, inserted, before, after }),
        }
        self.history.redo.clear();
        self.history.typing = typing;
        self.selection = after;
    }
}

/// The break a new line gets in a pane of `endings`.
// ponytail: a pane without a break of its own (one line, or none) gets Unix breaks; the platform's
// default comes with the Text Editing options if anyone asks.
fn style(endings: LineEndings) -> LineEnding {
    match endings {
        LineEndings::Windows => LineEnding::CrLf,
        LineEndings::Mac => LineEnding::Cr,
        LineEndings::Unix | LineEndings::Mixed | LineEndings::None => LineEnding::Lf,
    }
}

pub(crate) fn ending_text(ending: LineEnding) -> &'static str {
    match ending {
        LineEnding::None => "",
        LineEnding::Lf => "\n",
        LineEnding::CrLf => "\r\n",
        LineEnding::Cr => "\r",
    }
}

#[cfg(test)]
mod tests {
    use otd_text_engine::{Line, LineEnding};

    use super::{Pos, Selection};
    use crate::{Encoding, LineEndings, Pane};

    fn pane(lines: &[(&str, LineEnding)]) -> Pane {
        Pane {
            path: String::new(),
            size: 0,
            modified: None,
            format: String::new(),
            encoding: Encoding::Utf8 { bom: false },
            endings: LineEndings::Windows,
            lines: lines.iter().map(|&(text, ending)| Line { text: text.to_owned(), ending }).collect(),
            read_only: None,
            unsaved: false,
            selection: Selection::default(),
            history: Default::default(),
            bookmarks: Default::default(),
        }
    }

    #[test]
    fn a_multi_line_paste_over_a_selection_inside_characters_undoes_and_redoes_to_the_exact_lines() {
        let mut pane = pane(&[("ä€x", LineEnding::CrLf), ("ü", LineEnding::CrLf), ("öyz", LineEnding::None)]);
        let original = pane.lines.clone();
        // From after "ä€" to after "ö": inside multi-byte characters' neighbours, across a break.
        pane.selection = Selection { anchor: Pos { line: 0, column: 2 }, head: Pos { line: 2, column: 1 } };
        assert_eq!(pane.selected_text(), "x\r\nü\r\nö", "copied with the lines' own breaks");

        pane.insert("1\n2\r3");
        let texts: Vec<(&str, LineEnding)> = pane.lines.iter().map(|l| (l.text.as_str(), l.ending)).collect();
        assert_eq!(texts, [("ä€1", LineEnding::CrLf), ("2", LineEnding::CrLf), ("3yz", LineEnding::None)], "pasted breaks take the pane's");
        assert_eq!(pane.selection, Selection::caret(Pos { line: 2, column: 1 }), "the caret ends after the pasted text");
        let pasted = pane.lines.clone();

        assert!(pane.undo());
        assert_eq!(pane.lines, original, "Undo restores the lines and their endings exactly");
        assert!(pane.redo());
        assert_eq!(pane.lines, pasted);
    }

    #[test]
    fn typing_goes_on_in_one_step_until_the_caret_moves() {
        let mut pane = pane(&[("ab", LineEnding::None)]);
        pane.move_caret(Pos { line: 0, column: 1 }, false);
        for c in ["x", "y"] {
            pane.type_text(c);
        }
        pane.move_caret(Pos { line: 0, column: 0 }, false);
        pane.type_text("z");
        assert_eq!(pane.lines[0].text, "zaxyb");
        assert!(pane.undo());
        assert_eq!(pane.lines[0].text, "axyb", "the typing after a move is a step of its own");
        assert!(pane.undo());
        assert_eq!(pane.lines[0].text, "ab", "the typing before it is one step");
        assert!(!pane.can_undo());
    }

    #[test]
    fn overwriting_replaces_the_characters_after_the_caret_up_to_the_lines_end_in_one_step() {
        let mut pane = pane(&[("abc", LineEnding::CrLf), ("next", LineEnding::None)]);
        pane.move_caret(Pos { line: 0, column: 1 }, false);
        for c in ["x", "y", "z"] {
            pane.overwrite(c);
        }
        assert_eq!(pane.lines[0].text, "axyz", "past the line's end it adds");
        assert_eq!(pane.lines[1].text, "next", "and never takes the line break");
        assert!(pane.undo());
        assert_eq!(pane.lines[0].text, "abc", "one Undo takes it all back");
    }

    #[test]
    fn the_caret_stands_after_a_final_line_break_without_a_line_of_its_own() {
        let mut pane = pane(&[("one", LineEnding::CrLf)]);
        pane.move_caret(pane.end(), false);
        assert_eq!(pane.selection.head, Pos { line: 1, column: 0 }, "after the break");
        pane.insert("two\n");
        assert_eq!(
            pane.lines.iter().map(|l| (l.text.as_str(), l.ending)).collect::<Vec<_>>(),
            [("one", LineEnding::CrLf), ("two", LineEnding::CrLf)]
        );
        pane.delete(false);
        pane.delete(false);
        assert_eq!(
            pane.lines,
            [Line { text: "one".to_owned(), ending: LineEnding::CrLf }, Line { text: "tw".to_owned(), ending: LineEnding::None }],
            "Backspace takes the final break too"
        );
        pane.select_all();
        pane.delete(false);
        assert!(pane.lines.is_empty(), "deleting everything leaves no blank line behind");
    }

    #[test]
    fn deleting_at_a_line_break_joins_the_lines_and_keeps_the_later_lines_ending() {
        let mut pane = pane(&[("one", LineEnding::CrLf), ("two", LineEnding::None)]);
        pane.move_caret(Pos { line: 1, column: 0 }, false);
        pane.delete(false);
        assert_eq!(pane.lines, [Line { text: "onetwo".to_owned(), ending: LineEnding::None }]);
        assert_eq!(pane.selection.head, Pos { line: 0, column: 3 });
        pane.move_caret(pane.end(), false);
        pane.delete(true);
        assert!(pane.undo(), "Delete at the end changed nothing, so the join is the last step");
        assert_eq!(pane.lines.len(), 2);
    }

    #[test]
    fn a_bookmark_keeps_its_line_through_edits_around_and_on_it_and_their_undo() {
        let mut pane =
            pane(&[("a", LineEnding::CrLf), ("b", LineEnding::CrLf), ("c", LineEnding::CrLf), ("d", LineEnding::CrLf), ("e", LineEnding::None)]);
        let original = pane.lines.clone();
        let marked = |pane: &Pane| pane.bookmarks.get(2).map(|line| pane.lines[line].text.clone());
        pane.bookmarks.toggle(2, 3);

        // A break at the line's start pushes its text down, and the bookmark with it.
        pane.move_caret(Pos { line: 3, column: 0 }, false);
        pane.insert("\n");
        assert_eq!(marked(&pane).as_deref(), Some("d"));
        // Lines deleted above pull it up.
        pane.selection = Selection { anchor: Pos::default(), head: Pos { line: 2, column: 0 } };
        pane.delete(false);
        assert_eq!(marked(&pane).as_deref(), Some("d"));
        // Typing in the line keeps it there.
        pane.move_caret(pane.line_end(pane.bookmarks.get(2).unwrap()), false);
        pane.type_text("x");
        assert_eq!(marked(&pane).as_deref(), Some("dx"));

        while pane.undo() {}
        assert_eq!(pane.lines, original);
        assert_eq!(pane.bookmarks.get(2), Some(3), "Undo takes it back to its line");

        // Deleting its line moves it to the line after.
        pane.selection = Selection { anchor: Pos { line: 3, column: 0 }, head: Pos { line: 4, column: 0 } };
        pane.delete(false);
        assert_eq!(marked(&pane).as_deref(), Some("e"));
    }

    #[test]
    fn a_bookmark_follows_its_own_line_among_equal_lines() {
        let blank = |count: usize| vec![("", LineEnding::Lf); count];
        // Enter at the start of a blank line, three times, carries its bookmark down; Undo back.
        let mut doc = pane(&blank(60));
        doc.bookmarks.toggle(2, 50);
        doc.move_caret(Pos { line: 50, column: 0 }, false);
        for _ in 0..3 {
            doc.insert("\n");
        }
        assert_eq!(doc.bookmarks.get(2), Some(53));
        while doc.undo() {}
        assert_eq!(doc.bookmarks.get(2), Some(50));

        // Backspace at the start of a blank line under another takes its bookmark up with it.
        let mut doc = pane(&[("", LineEnding::Lf), ("", LineEnding::Lf), ("foo", LineEnding::None)]);
        doc.bookmarks.toggle(1, 1);
        doc.move_caret(Pos { line: 1, column: 0 }, false);
        doc.delete(false);
        assert_eq!(doc.bookmarks.get(1), Some(0), "its blank line is now line 0, above foo");

        // A copy of a line pasted at its start goes above it, not under its bookmark.
        let mut doc = pane(&[("foo", LineEnding::Lf), ("bar", LineEnding::None)]);
        doc.bookmarks.toggle(3, 0);
        doc.insert("foo\n");
        assert_eq!(doc.bookmarks.get(3), Some(1));
    }
}
