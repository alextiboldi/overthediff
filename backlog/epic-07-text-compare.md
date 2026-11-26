# E07 — Text Compare

**Group**: feature
**Label**: `area:text-compare`
**Size**: L
**Depends on**: E01, E02, E03, E04 (foundations); E08 (child sessions, for Next Difference options); E10 (grammars, for editor tweaks); E15, E16, E17 (other file views, for changed-on-disk detection)
**Source**: catalogue §12, F-6.1, F-6.3, F-7.1, F-7.2, F-25.2, F-26.11, Appendix B. Spec-derived from the feature catalogue and not yet validated. Items marked *inferred* are analyst or PO inferences.

Text Compare is the core of the product: two text files side by side, aligned line by line, with every difference highlighted and full editing in place. This epic covers the view itself; walking, filtering, searching and editing differences; the importance rules that decide what matters; and the alignment, display and clipboard options that power users expect.

## Behavioral contract
- **Data**:
  - *Text session*: left and right specs (any path the data sources accept), Description, and rules: importance, alignment, format and encoding per side, unimportance list, and replacements.
  - *View state*: display filter, layout, word wrap, visible whitespace, details panels, thumbnail.
  - *Comparison result* (from the text engine): aligned row pairs (a line or a gap on each side), difference sections, character-level ranges, importance per difference, an overall result (same, unimportant differences only, or important differences) and statistics.
  - *Pane document*: decoded text, encoding, line-ending style, file format, read-only flag with its reason, an unsaved-edits flag, undo history, edited-line marks and bookmarks 0–9.
  - *Options*: the Text Editing page (Auto indent, Backspace unindents, Initialize "Text to find" to be current word, Show filtered line counts, Number of context lines), the Next Difference page, and Tweaks (Editor Display group, thumbnail mode, incremental-search reset delay, check for files changed on disk).
- **Routes/endpoints** (desktop app, so these are commands, options and switches):
  - Session, File, Edit, Search, View and Actions commands, plus the toolbar and shortcuts of the Text Compare view.
  - Session Settings tabs: Specs, Importance, Alignment, Format, and Replacements.
  - Options pages: Text Editing, Next Difference and Tweaks.
  - CLI `<file> <file>` opens Text Compare when both files resolve to a Text format. `clipboard://` is accepted as a spec.
- **Integrations**:
  - E03-S1 text comparison engine: alignment, sections and importance classification.
  - E04 data sources: read, write and read-only capability.
  - E02-S1: the regex engine and regex input aid (find, unimportance list, replacements), and the file-format registry (format per side, file info).
  - E01-S1 shell: commands, the Session Settings dialog, options pages, path edits, unsaved-changes prompts and the status bar.
  - E10 grammars for syntax colours.
- **Rules/states**:
  - Default colours (Appendix B): important differences are red text on a light-red line background; unimportant differences are blue text on a light-blue background; red wins on a line.
  - Orphan lines are always important.
  - The comparison updates live as the user types. Recompare Files keeps edits but clears undo history. Reload Files prompts to save first.
  - Editing is disabled when the comparison errored or was cancelled, the location is read-only, the format's conversion can't save, editing is disabled for the session or its parent folder session, or a CLI read-only switch is set. Each condition arrives with the story that introduces it.
  - Navigation, display filters, statistics and the status bar all use the view's current classification of differences, so rule changes and the Ignore Unimportant Differences toggle affect them consistently.

---

## Stories

### E07-S1 — Text Compare view
**Priority**: Must · **Layer**: 1 · **Size**: L · **Platforms**: All
**Needs**: E01-S1 (application shell), E03-S1 (text comparison engine), E04-S1 (file-system access layer), E02-S1 (file-format registry)
**Traces**: F-12.1, F-12.3, F-12.10, F-6.1, F-25.2, F-7.1, App-B · **Open questions**: Q25
**Not in this story**: navigation → E07-S2; display filters → E07-S3; importance rules and blue/unimportant → E07-S4; find/replace → E07-S5; go to/bookmarks → E07-S6; copy to other side and Select Section → E07-S7; indent/line/word commands/convert → E07-S8; whitespace/wrap/layout/font size → E07-S9; Format tab → E07-S10; thumbnail → E07-S11; details panels/ruler → E07-S12; alignment options → E07-S13; manual alignment → E07-S14; syntax highlighting → E10-S2; tab stops and Tab key → E10-S4; saving conversions → E10-S6; backups → E09-S10; changed-on-disk → E07-S22; /savetarget and /ro → E11-S1; Disable editing → E06-S8; report → E18-S1; clipboard → E07-S18; colour/font customisation → E22-S3; Merge Files → E13-S1; Compare Parent Folders → E06-S10; Open With → E22-S4.

As a developer, I can open two text files side by side, see every difference highlighted and fix them in place so that I can review and reconcile changes without switching to another editor.

**Acceptance criteria**
- [ ] A Text Compare session opens from:
  - the Home launcher
  - *Session > New Session ▸ Text Compare*
  - *File > Open File* (into the chosen pane)
  - the hidden-by-default *Open File on File System*
  - the command line, with two files that resolve to a Text format
- [ ] Each pane has a path edit with typed paths, Browse for File and history.
- [ ] The two editor panes scroll together. Lines are aligned and gaps are padded, so matching lines sit on the same row.
- [ ] Important differences show in red text, their lines get a light-red background, and the changed characters inside each line are highlighted.
- [ ] Each pane shows line numbers, and gap rows have none.
- [ ] Each pane has a file info header with path, size, modified time, detected file format, encoding and line-ending style.
- [ ] Encoding is detected from a byte-order mark, valid UTF-8 or UTF-16, falling back to the system code page. Line endings (Windows, Mac, Unix) are detected per file, and mixed endings show as mixed (*inferred*).
- [ ] Each pane is a full text editor. Typing, *Undo*/*Redo*, *Cut*/*Copy*/*Paste*/*Delete* and *Select All* (the visible lines in the pane) all work, and the comparison updates live as the user types.
- [ ] *Save File* writes the pane back in its original encoding and line-ending style. *Save File As* writes to a new path, and the hidden-by-default *Save File on File System As* saves to a local path.
- [ ] A pane with unsaved edits is marked, and Close Tab and Exit prompt to save.
- [ ] A pane is read-only, with the reason shown, when its location is read-only or the comparison errored or was cancelled.
- [ ] *Reload Files* prompts to save unsaved edits and then reloads both files. *Recompare Files* reruns the comparison, keeps edits and clears undo history.
- [ ] *Swap Sides* exchanges the left and right files and recompares.
- [ ] *Text Compare Info* shows comparison statistics. The status bar shows the overall result and the load time, as decided in T1.

#### Decisions
Recorded by E07-S1-T1 and confirmed by the operator on 2026-11-25, as proposed. The catalogue's figures show an overall result, an insert/overwrite indicator and the load time in the status bar, and it says Text Compare Info shows comparison statistics. Which fields and figures Text Compare shows is *unspecified* there (Q25) and is OverTheDiff's own choice. The fields fill E01-S1's status-bar field model (E01-S1 Decisions, Q25).

**Status bar (Q25)**, in the field model's order:
1. **Overall result**: "Same", "Unimportant differences only" or "Important differences", the text engine's `Outcome`.
2. **Cursor line and column**, the view's own field.
3. **Insert/overwrite**: "Insert" or "Overwrite".
4. **Load time**: how long loading both files and comparing them took.

**Text Compare Info** lists:
- difference sections;
- changed lines;
- inserted lines;
- deleted lines;
- total lines on each side.

Both use the view's current classification of differences, as the behavioral contract's last rule says.

The details below are not part of the confirmed proposal. E07-S1-T1 added them so that E07-S1-T8 can build the decision without guessing:
- **Info figures come from the text engine's `Statistics`.** A row is a line or a gap on each side.
  - Difference sections are `sections`.
  - Deleted lines are left lines opposite a gap (`left_orphans`), and inserted lines are right lines opposite a gap (`right_orphans`), reading the left side as the earlier one.
  - Changed lines are rows with a line on both sides that differ: `important + unimportant - left_orphans - right_orphans`. Orphans are always important, so they are inside `important`.
  - A side's total is `same` + changed + that side's orphans. It counts the lines in the pane as it stands, edits included.
- **Cursor.** The field shows the line number and column of the cursor in the pane that last had it. Lines are the file's own line numbers, so gap rows don't count, and columns count characters. Both start at 1. The field is empty until a pane has a cursor.
- **No partial result.** The overall result shows only once a comparison has finished. While a load or comparison runs, and after one errored or was cancelled, the result field is empty and Info says there is no comparison result instead of showing counts (AGENTS.md rule 15). So a partial or failed comparison never reads as "Same".
- **When fields change.** A live recompare while the user types updates the result and Info. A change of result is announced through the shell's live region, since focus is in the pane and not on the status bar. Load time is set when loading both files and comparing them finishes, after opening, Reload Files, Recompare Files or Swap Sides. A live recompare doesn't change it.
- **Insert/overwrite state.** Both panes of a tab share it, and it starts as Insert.

#### Tasks
- **E07-S1-T1** — Decide the status-bar and Info contents (Q25) · size: S · depends: — · flags: `decision` · status: done
  - AC: decision recorded in this story. Proposed default: the status bar shows the overall result ("Same", "Unimportant differences only" or "Important differences"), cursor line and column, insert/overwrite state and load time. Info lists difference sections, changed, inserted and deleted lines, and total lines per side.
  - Implemented: a `#### Decisions` section in this story records Q25 for Text Compare, as confirmed by the operator on 2026-11-25: the status bar shows the overall result, cursor line and column, insert/overwrite and load time in E01-S1's field order, and Text Compare Info lists difference sections, changed, inserted and deleted lines and total lines per side. Added for E07-S1-T8 and marked as outside the confirmed proposal: each Info figure's mapping onto the text engine's `Statistics`, the cursor field's meaning, no result or counts while a comparison runs or after it errored or was cancelled, when fields update and are announced, and the shared insert/overwrite state. `backlog/00-index.md` and `backlog/PRD.md` mark the E07-S1 part of Q25 decided. No code. `cargo xtask ci` passed on macOS, and the pull request's CI passed on Linux and macOS. https://github.com/alextiboldi/overthediff/pull/66
- **E07-S1-T2** — Open a Text Compare session from Home, New Session, File > Open File, Open File on File System, the path edits and the command line · size: M · depends: — · flags: `naming-check`
  - AC: launching the product with two text files opens one Text Compare tab. A typed or browsed path loads into its pane. A missing file shows an error in that pane and leaves it read-only.
- **E07-S1-T3** — Load files through the data-source layer with encoding and line-ending detection, and show the file info header · size: M · depends: — · flags: `licence-check`, `craft-libs`
  - AC: UTF-8 with and without a BOM, UTF-16 LE and BE, and a legacy code-page file each display correctly with the detected encoding shown. CRLF, CR and LF files show their line-ending style.
- **E07-S1-T4** — Render the aligned comparison: synchronised scrolling, gap padding, red text, light-red line background, character highlights and line numbers · size: L · depends: E07-S1-T3 · flags: `craft-libs`, `accessibility`
  - AC: for a pair with one inserted, one deleted and one changed line, matching lines share rows and gaps appear opposite the inserted and deleted lines. Only the changed characters are highlighted, and scrolling either pane scrolls both.
- **E07-S1-T5** — Edit with live recompare: typing, Undo/Redo, Cut/Copy/Paste/Delete and Select All · size: M · depends: E07-S1-T4 · flags: `accessibility`
  - AC: typing the other side's text into a changed line removes its highlight without a manual recompare. Undo brings the difference back.
- **E07-S1-T6** — Save File, Save File As, Save File on File System As, and unsaved-changes prompts · size: M · depends: E07-S1-T5
  - AC: saving keeps the file's encoding and line endings, so only the edited bytes change. Close Tab on an edited pane offers save, discard or cancel.
- **E07-S1-T7** — Read-only panes with a visible reason · size: S · depends: E07-S1-T3
  - AC: a file on a read-only data source opens read-only and says why. After a cancelled comparison both panes are read-only.
- **E07-S1-T8** — Session commands: Reload Files, Recompare Files, Swap Sides, Text Compare Info and the status bar · size: M · depends: E07-S1-T1, E07-S1-T4
  - AC: Reload Files with unsaved edits prompts first. Recompare Files keeps edits and empties undo history. Swap Sides moves each file to the other pane. Info and the status bar show what T1 decided.

### E07-S2 — Navigate between differences
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E07-S1 (Text Compare view)
**Traces**: F-12.4
**Not in this story**: per-line navigation for independent-line formats → E10-S4; Next/Previous Replacement → E07-S16; Next/Previous Difference Files → E08-S10; Next Difference options → E07-S19.

As a code reviewer, I can jump from one difference to the next so that I can review a change set without scrolling through unchanged text.

**Acceptance criteria**
- [ ] *Next Difference Section* and *Previous Difference Section* jump to the next or previous run of differing lines and scroll both panes to it.
- [ ] *Next Difference* and *Previous Difference* step through individual differences inside lines. When a difference lies beyond the visible width of a long line, the panes scroll horizontally to it.
- [ ] *Next Edit* and *Previous Edit* jump between lines the user has edited since the files were loaded.
- [ ] The navigation arrows are bright yellow while more differences lie in that direction. They turn dull yellow, or disabled, when there are none.
- [ ] Navigation visits differences as the view currently classifies them, so a difference the view treats as same is skipped.
- [ ] Every navigation command has a default keyboard shortcut.

#### Tasks
- **E07-S2-T1** — Find the next or previous section and the next or previous in-line difference from the cursor · size: M · depends: —
  - AC: unit cases cover a cursor before, inside and after a section. A line with three separate changes is visited change by change, in order.
- **E07-S2-T2** — Navigation commands and shortcuts that bring the target into view in both panes · size: S · depends: E07-S2-T1
  - AC: Next Difference Section from the top of the file lands on the first section, with both panes showing it.
- **E07-S2-T3** — Track edited lines, and add Next/Previous Edit · size: S · depends: —
  - AC: after edits on lines 10 and 40, Next Edit from line 1 visits line 10 and then line 40.
- **E07-S2-T4** — Arrow states that follow the cursor and the comparison · size: S · depends: E07-S2-T1
  - AC: at the last section the Next arrow turns dull or disabled. An edit that removes every difference disables both arrows.

### E07-S3 — Display filters and context
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E07-S1 (Text Compare view)
**Traces**: F-12.2, F-26.11
**Not in this story**: Ignore Unimportant Differences → E07-S4; Ignored → E07-S17; remembering the filter → E06-S4.

As a code reviewer, I can hide matching lines, or show differences with a few lines of context, so that large files show only what changed.

**Acceptance criteria**
- [ ] *Show All* shows every line. *Show Differences* shows only lines with differences. *Show Same* shows only matching lines. *Show None* (hidden by default) hides every line.
- [ ] *Show Context* shows the differences plus *N* surrounding lines. *N* is the "Number of context lines" option on the Text Editing page.
- [ ] Filters apply to both panes together, so rows stay aligned. The active filter shows as checked.
- [ ] With "Show filtered line counts" on, each run of hidden lines is summarised by the number of lines it hides (*inferred* presentation).
- [ ] A line counts as different or same according to the view's current classification.

#### Tasks
- **E07-S3-T1** — Work out which rows each filter shows (All, Differences, Same, None, Context with *N*) · size: M · depends: —
  - AC: a unit table for a file with two sections and *N* = 2 gives the expected visible rows under every filter, including overlapping context ranges.
- **E07-S3-T2** — Display-filter commands on the View menu and toolbar (Show None hidden by default) · size: S · depends: E07-S3-T1 · flags: `naming-check`
  - AC: switching filters keeps the cursor on the same line whenever that line stays visible.
- **E07-S3-T3** — "Number of context lines" and "Show filtered line counts" options on the Text Editing page · size: S · depends: E07-S3-T1
  - AC: changing *N* updates an open view. With counts on, a hidden run of 37 lines shows a summary saying 37 lines are hidden.

### E07-S4 — Importance rules for whitespace, case and line endings
**Priority**: Must · **Layer**: 2 · **Size**: L · **Platforms**: All
**Needs**: E07-S1 (Text Compare view)
**Traces**: F-12.6, F-12.2, F-6.3, App-B
**Not in this story**: grammar elements → E10-S5; Edit Grammar button → E10-S3; unimportance list → E07-S15; replacements → E07-S16.

As a developer, I can mark whitespace, letter-case and line-ending differences as unimportant so that reformatting doesn't hide the changes that matter.

**Acceptance criteria**
- [ ] The Importance tab of Session Settings lists the built-in elements *Leading whitespace*, *Embedded whitespace*, *Trailing whitespace* and *Everything else*. A ticked element is important and an unticked one is unimportant.
- [ ] *Everything else* has a **Character case** checkbox, which decides whether case differences in non-whitespace text are important.
- [ ] **Compare line endings (PC/Mac/Unix)** is off by default, so line-ending differences are ignored. When it is on, line endings are compared line by line.
- [ ] An orphan line is always important, even when it is blank or holds only unimportant text.
- [ ] Unimportant differences show in blue text with a light-blue line background. A line holding both kinds gets the light-red background, because red wins.
- [ ] *View > Ignore Unimportant Differences* treats unimportant differences as same. They are not highlighted, and they count as same wherever the view classifies lines.
- [ ] The status bar and Text Compare Info tell "unimportant differences only" apart from important differences and from same.
- [ ] Rule changes apply through the Session Settings scope dropdown and recompare the open view at once.

#### Tasks
- **E07-S4-T1** — Classify leading, embedded and trailing whitespace, and the case of "everything else" · size: M · depends: —
  - AC: unit cases. An indentation-only change is unimportant when Leading whitespace is unticked. `a  b` vs `a b` follows Embedded whitespace. A case-only change is unimportant only when Character case is unticked.
- **E07-S4-T2** — Compare line endings option · size: S · depends: —
  - AC: a CRLF file and its LF copy compare as same by default. With the option on, every line is flagged as different.
- **E07-S4-T3** — Keep orphan lines important · size: S · depends: E07-S4-T1
  - AC: an inserted blank line stays red with every whitespace element unticked.
- **E07-S4-T4** — Importance tab controls · size: S · depends: E07-S4-T1, E07-S4-T2
  - AC: ticking or unticking an element recompares the open view. The values are kept with the session settings.
- **E07-S4-T5** — Show unimportant differences in blue, with red-over-blue line backgrounds · size: M · depends: E07-S4-T1 · flags: `accessibility`
  - AC: a line with a case-only change and a word change shows blue for the case change, red for the word and a light-red background.
- **E07-S4-T6** — Ignore Unimportant Differences toggle and the overall result · size: M · depends: E07-S4-T5
  - AC: with the toggle on, a pair that differs only in indentation shows no highlights. The status bar reports "unimportant differences only" whether or not they are shown.

### E07-S5 — Find and replace
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E07-S1 (Text Compare view), E02-S1 (regular expressions and regex input aid)
**Traces**: F-12.5, F-26.11

As a developer, I can search both files for text or a regular expression and replace the matches so that I can find and fix every occurrence while comparing.

**Acceptance criteria**
- [ ] *Find* opens a find panel whose text box keeps a history of searches.
- [ ] Panel options:
  - **Regular expression**: PCRE subset per Appendix A.2.
  - **Match character case**.
  - **Whole words only**: `apple` doesn't match `apples`, `applesauce` or `snapple`.
  - **Wrap search**.
  - **Sides**: left, right or both.
  - **Close automatically**: hides the panel once the search starts.
- [ ] With Regular expression on, *Insert Regular Expression Element* inserts syntax at the cursor. An invalid pattern shows an error and searches nothing.
- [ ] **Next** finds the next match, and **Select All** selects every line that contains a match.
- [ ] *Find Next* and *Find Previous* repeat the last search forwards or backwards without the panel.
- [ ] *Replace* replaces the current match or all matches on the searched sides. Replacements can be undone and update the comparison live. Read-only panes are skipped with a message (*inferred*).
- [ ] With the option *Initialize "Text to find" to be current word* on, opening Find with no selection fills in the word at the cursor.
- [ ] Typing in the find box searches incrementally. After a pause longer than the tweak "incremental-search reset delay" (ms), the next keystroke starts a new search (*inferred* meaning).

#### Tasks
- **E07-S5-T1** — Match over both panes: plain text, case, whole words, regex, sides, wrap and direction · size: M · depends: — · flags: `licence-check`, `craft-libs`
  - AC: unit cases include the catalogue's whole-word example and a regex with a backreference.
- **E07-S5-T2** — Find panel with history, element picker, options, Next, Select All and Close automatically · size: M · depends: E07-S5-T1
  - AC: Select All for `TODO` selects exactly the lines that contain it on the chosen sides.
- **E07-S5-T3** — Find Next/Previous and the "Initialize Text to find to be current word" option · size: S · depends: E07-S5-T1
  - AC: with the cursor inside `foo` and no selection, Find opens filled with `foo`.
- **E07-S5-T4** — Replace and Replace All, with undo and live recompare · size: M · depends: E07-S5-T1
  - AC: replacing every `a` with `b` on the left changes only the left pane and marks it unsaved. One Undo reverts all the replacements (*inferred*).
- **E07-S5-T5** — Incremental search and its reset-delay tweak · size: S · depends: E07-S5-T2
  - AC: with the delay at 500 ms, typing `ab`, pausing for 1 s, then typing `c` searches for `c`.

### E07-S6 — Go to line and bookmarks
**Priority**: Must · **Layer**: 2 · **Size**: S · **Platforms**: All
**Needs**: E07-S1 (Text Compare view)
**Traces**: F-12.5

As a developer, I can jump to a line and column and set numbered bookmarks so that I can move around long files quickly.

**Acceptance criteria**
- [ ] *Go To* moves the cursor in the active pane to a given line and column and scrolls both panes there. Values out of range go to the nearest valid position (*inferred*).
- [ ] *Toggle Bookmark* sets or clears a bookmark numbered **0–9** on the current line. *Go To Bookmark* jumps to it, and *Clear Bookmarks* removes them all.
- [ ] Bookmarked lines are marked in the gutter, and a bookmark moves with its line when lines are inserted or deleted above it (*inferred*).

#### Tasks
- **E07-S6-T1** — Go To line and column · size: S · depends: —
  - AC: Go To 120:5 puts the cursor on line 120, column 5, of the active pane and both panes show it.
- **E07-S6-T2** — Bookmarks 0–9: toggle, go to, clear, gutter marks, and keeping their position through edits · size: S · depends: —
  - AC: after inserting 3 lines above bookmark 2 on line 50, Go To Bookmark 2 lands on line 53.

### E07-S7 — Copy differences to the other side
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E07-S1 (Text Compare view)
**Traces**: F-12.3
**Not in this story**: go to next difference after copying → E07-S19; gutter copy buttons → E07-S20.

As a developer merging by hand, I can copy a difference section or selected lines to the other side so that I can take changes across without retyping them.

**Acceptance criteria**
- [ ] *Copy to Other Side* copies the selected lines, or the current difference section when nothing is selected, to the other pane. Its caption shows the direction (Copy to Right or Copy to Left) based on the active pane.
- [ ] *Copy to Right* and *Copy to Left* are hidden by default and run from their default shortcuts.
- [ ] *Copy Line to Right*, *Copy Line to Left* and *Copy Line to Other Side* copy only the current line.
- [ ] After a section is copied, the target section is identical to the source. Lines that exist only on the target side are removed, and lines missing there are inserted.
- [ ] A copy is one undoable edit. It marks the target pane unsaved and updates the comparison live.
- [ ] A copy into a read-only pane is refused with the reason, and nothing changes.
- [ ] *Select Section* selects every line of the current difference section.

#### Tasks
- **E07-S7-T1** — Work out which source and target lines a copy covers · size: M · depends: —
  - AC: unit cases cover a changed section, left-only lines (a gap on the target), right-only lines (removed from the target) and a partial selection inside a section.
- **E07-S7-T2** — Copy to Other Side with a dynamic caption, plus the hidden Copy to Right/Left · size: S · depends: E07-S7-T1
  - AC: the caption reads "Copy to Right" while focus is in the left pane, and "Copy to Left" from the right pane.
- **E07-S7-T3** — Copy Line to Right/Left/Other Side · size: S · depends: E07-S7-T1
  - AC: only the current line is copied, even inside a multi-line section.
- **E07-S7-T4** — Apply a copy as one undoable edit with live recompare, and refuse read-only targets · size: S · depends: E07-S7-T1
  - AC: after a section is copied its rows show as same, and Undo restores the difference. A read-only target shows its reason and nothing changes.
- **E07-S7-T5** — Select Section · size: S · depends: E07-S7-T1
  - AC: with the cursor in a 5-line section, Select Section selects those 5 lines in the active pane.

### E07-S8 — Editing commands and file conversions
**Priority**: Should · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E07-S1 (Text Compare view)
**Traces**: F-12.3, F-26.11
**Not in this story**: Tab key inserts spaces and tab stop → E10-S4.

As a developer, I can indent lines, delete words and lines, and convert whitespace and line endings in a pane so that I can clean up a file while comparing it.

**Acceptance criteria**
- [ ] *Increase Indent* and *Decrease Indent* shift the selected lines by one indent unit (*inferred*: a tab, or the current tab width in spaces).
- [ ] These commands are hidden by default and run from their default shortcuts: *Delete Line*, *Delete to Start of Line*, *Delete to End of Line*, *Delete Word*, *Delete to Start of Word*, *Delete to End of Word*, *Insert Line Before* and *Insert Line After*.
- [ ] Text Editing options:
  - **Auto indent**: a new line starts with the previous line's indentation.
  - **Backspace unindents**: Backspace inside leading whitespace removes one indent unit.
- [ ] The *Convert File* submenu offers:
  - *Trim Trailing Whitespace*
  - *Leading Spaces to Tabs*
  - *Tabs to Spaces*, using the built-in tab width of 4 columns (*inferred*)
  - *Line Endings*, which changes every line to the Windows, Mac or Unix style
- [ ] Every command can be undone, marks the pane unsaved and updates the comparison live.

#### Tasks
- **E07-S8-T1** — Increase and Decrease Indent on the selected lines · size: S · depends: —
  - AC: three selected lines indent and unindent together, and a line with no indentation is unchanged by Decrease Indent.
- **E07-S8-T2** — Line and word commands, hidden by default · size: M · depends: —
  - AC: in `foobar baz` with the cursor after `foo`, Delete to End of Word leaves `foo baz`. Insert Line After adds an empty line below the cursor line.
- **E07-S8-T3** — Auto indent and Backspace unindents on the Text Editing page · size: S · depends: —
  - AC: with Auto indent on, Enter after an indented line starts the new line at the same indentation. With Backspace unindents on, one Backspace there removes one indent unit.
- **E07-S8-T4** — Convert File: Trim Trailing Whitespace, Leading Spaces to Tabs and Tabs to Spaces · size: M · depends: —
  - AC: on a file with mixed indentation, each conversion changes only the leading or trailing whitespace it names, using the current tab width.
- **E07-S8-T5** — Convert File > Line Endings · size: S · depends: —
  - AC: converting a CRLF file to Unix changes every line ending. Saving writes LF only, and the file info header shows Unix.

### E07-S9 — Whitespace, word wrap, layout and font size
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E07-S1 (Text Compare view)
**Traces**: F-12.1
**Not in this story**: fonts and colours → E22-S3; editor tweaks → E07-S20; syntax highlighting → E10-S2.

As a developer, I can show whitespace, wrap long lines, stack the panes and resize the text so that I can read any file comfortably on any screen.

**Acceptance criteria**
- [ ] *View > Visible Whitespace* shows spaces, tabs and Windows, Mac and Unix line endings, each with its own glyph.
- [ ] The toolbar *Wrap* button (Word Wrap) wraps lines to the pane width. Continuation rows stay aligned across the panes, and the line number appears only on a line's first row (*inferred*).
- [ ] The *Side-by-Side* and *Over-Under* layouts switch between left/right and top/bottom panes, keeping the scroll position and the cursor.
- [ ] Display font size increase, decrease and reset (hidden by default) change the editor text size in both panes.

#### Tasks
- **E07-S9-T1** — Visible Whitespace glyphs for space, tab, CRLF, CR and LF · size: S · depends: —
  - AC: a file containing all three line-ending styles shows three different end-of-line glyphs.
- **E07-S9-T2** — Word Wrap with aligned continuation rows · size: M · depends: —
  - AC: a 300-character changed line wraps in both panes, and the line after it starts on the same row in both panes.
- **E07-S9-T3** — Side-by-Side and Over-Under layouts · size: M · depends: — · flags: `naming-check`
  - AC: switching layout keeps the current line in view, and both panes still scroll together.
- **E07-S9-T4** — Font size increase, decrease and reset · size: S · depends: —
  - AC: two increases followed by a reset return the text to the configured size.

### E07-S10 — Per-side file format and encoding overrides
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E07-S1 (Text Compare view), E02-S1 (file-format registry)
**Traces**: F-12.8
**Not in this story**: Compare Files Using → E06-S9.

As a developer comparing a file whose name or bytes mislead detection, I can choose the file format and encoding for each side so that it is decoded and compared correctly.

**Acceptance criteria**
- [ ] The Format tab of Session Settings has a file-format picker per side. The default, **Detected**, means matched by mask through the file-format registry.
- [ ] The picker lists every registered format, including formats with an empty mask that never match automatically.
- [ ] Each side has an encoding override. The default, **None**, uses the format's encoding, or auto-detection when the format doesn't fix one.
- [ ] Applying an override reloads that side and recompares, prompting first when it has unsaved edits. The file info header shows the format and encoding in use.
- [ ] After an encoding override, saving writes the file in the overridden encoding (*inferred*).

#### Tasks
- **E07-S10-T1** — Format picker per side, listing every registered format with Detected as the default · size: S · depends: —
  - AC: an empty-mask format appears in the list, and once chosen the file info header names it.
- **E07-S10-T2** — Encoding override per side with None as the default · size: S · depends: —
  - AC: a Windows-1252 file mis-detected as UTF-8 shows the correct characters after Windows-1252 is chosen.
- **E07-S10-T3** — Reload with overrides (prompting when there are unsaved edits) and save in the chosen encoding · size: M · depends: E07-S10-T1, E07-S10-T2
  - AC: after the override is set to UTF-16 LE and the file is saved, the file on disk is UTF-16 LE.

### E07-S11 — Thumbnail overview
**Priority**: Should · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E07-S1 (Text Compare view)
**Traces**: F-12.1, F-26.11

As a code reviewer, I can see a thumbnail map of the whole comparison and click it to jump so that I can spot and reach changes in a long file at a glance.

**Acceptance criteria**
- [ ] *View > Thumbnail* shows an optional thumbnail at the left edge of the view.
- [ ] The thumbnail draws one coloured pixel row per comparison line, coloured by that line's status.
- [ ] A white rectangle marks the current viewport, and a triangle marks the current line.
- [ ] Clicking the thumbnail jumps both panes to that position.
- [ ] Tweak **Compress to fit** scales the whole file into the thumbnail's height. **Allow scrolling** keeps one pixel row per line and scrolls the thumbnail.

#### Tasks
- **E07-S11-T1** — Draw the thumbnail with status colours · size: M · depends: — · flags: `accessibility`
  - AC: a 10,000-line file with three sections shows three coloured bands at proportional positions.
- **E07-S11-T2** — Viewport rectangle and current-line triangle · size: S · depends: E07-S11-T1
  - AC: scrolling or moving the cursor moves the rectangle and the triangle to match.
- **E07-S11-T3** — Click to jump · size: S · depends: E07-S11-T1
  - AC: clicking a band scrolls both panes to that section.
- **E07-S11-T4** — Compress to fit / Allow scrolling tweak · size: S · depends: E07-S11-T1
  - AC: with Allow scrolling on, a file taller than the thumbnail scrolls the thumbnail along with the panes.

### E07-S12 — Details panels and ruler
**Priority**: Should · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E07-S1 (Text Compare view)
**Traces**: F-12.1

As a developer, I can see the current line of both files in full-width detail panels so that I can inspect long-line differences character by character.

**Acceptance criteria**
- [ ] The **Text Details** panel below the panes shows the current line of both files at full window width. It is editable, and its edits change the pane and update the comparison.
- [ ] The **Hex Details** panel shows the current line of both files as read-only hex.
- [ ] The **Alignment Details** panel shows the character-level alignment of the current line and is editable.
- [ ] The **Ruler** shows column positions for the detail lines.
- [ ] Each panel can be shown or hidden on its own and follows the cursor as it moves.

#### Tasks
- **E07-S12-T1** — Text Details panel (editable) · size: M · depends: — · flags: `naming-check`
  - AC: typing in Text Details changes the same line in the pane, and the highlight updates.
- **E07-S12-T2** — Hex Details panel (read-only) · size: S · depends: — · flags: `naming-check`
  - AC: a line that differs only by a non-breaking space shows the differing byte values.
- **E07-S12-T3** — Alignment Details panel (editable) · size: M · depends: — · flags: `naming-check`
  - AC: inserted characters show against gaps in the aligned layout, and an edit there updates the comparison.
- **E07-S12-T4** — Ruler · size: S · depends: E07-S12-T1
  - AC: ruler marks line up with character columns in Text Details.

### E07-S13 — Alignment algorithms and options
**Priority**: Should · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E07-S1 (Text Compare view)
**Traces**: F-12.7, X2

As a developer comparing heavily reorganised files, I can choose how lines are aligned so that the comparison matches the way the file actually changed.

**Acceptance criteria**
- [ ] The Alignment tab of Session Settings offers four methods:
  - **Unaligned**: line *N* against line *N*.
  - **Standard alignment** (default): with progressive display.
  - **Myers O(ND)**: compares the files all at once, so nothing shows until it finishes. It has no similarity matching and groups mismatches in blocks.
  - **Patience Diff**.
- [ ] **Never align differences** shows important-difference lines as add and delete blocks instead of changed lines.
- [ ] **Skew tolerance** sets the maximum number of lines searched forwards and backwards for a match.
- [ ] **Use closeness matching** aligns the remaining mismatched lines by similarity.
- [ ] Changing any of these recompares the open view.

#### Tasks
- **E07-S13-T1** — Unaligned mode · size: S · depends: —
  - AC: with one line inserted at the top, every following line shows as changed.
- **E07-S13-T2** — Myers O(ND) and Patience Diff · size: M · depends: — · flags: `naming-check`, `craft-libs`
  - AC: on reference file pairs with repeated blocks, both algorithms give their specified results. Myers groups mismatches into blocks, and Patience anchors on unique lines.
- **E07-S13-T3** — Never align differences, Skew tolerance and Use closeness matching · size: M · depends: — · flags: `craft-libs`
  - AC: with Never align differences on, a changed line shows as one deleted and one inserted line. A block moved further than the skew tolerance is no longer aligned.
- **E07-S13-T4** — Alignment tab controls with immediate recompare · size: S · depends: E07-S13-T1, E07-S13-T2, E07-S13-T3
  - AC: switching the method updates the open view without reopening it.

### E07-S14 — Manual alignment
**Priority**: Should · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E07-S1 (Text Compare view)
**Traces**: F-12.7

As a developer, I can force specific lines to line up, or to stand alone, so that I can fix an alignment the algorithm got wrong.

**Acceptance criteria**
- [ ] Right-clicking a line, choosing *Align With* and then clicking a line on the other side puts the two lines on the same row. The rest of the file re-aligns around them.
- [ ] One or more lines can be selected first, and the selection aligns starting at the clicked line.
- [ ] *Isolate* realigns so that the selected lines stand on their own, shown against gaps.
- [ ] Pressing Esc while an Align With is pending cancels it (*inferred*).

#### Tasks
- **E07-S14-T1** — Alignment anchors in the comparison: forced pairs and isolated lines · size: M · depends: — · flags: `craft-libs`
  - AC: with an anchor forcing left line 10 against right line 50, the result pairs them and aligns the lines before and after independently.
- **E07-S14-T2** — Align With interaction (pick, click the target, cancel) · size: S · depends: E07-S14-T1
  - AC: aligning a 3-line selection with a target line puts all 3 lines on consecutive rows opposite the target.
- **E07-S14-T3** — Isolate · size: S · depends: E07-S14-T1
  - AC: isolating a changed line shows it against a gap, with its former partner also against a gap.

### E07-S15 — Unimportance list
**Priority**: Should · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E07-S4 (importance rules), E02-S1 (regular expressions)
**Traces**: F-12.6 · **Open questions**: Q7

As a developer, I can list text patterns whose differences don't matter, such as build stamps or generated dates, so that they show as unimportant.

**Acceptance criteria**
- [ ] The Importance tab has an **Unimportance list**, and text matching its items is unimportant.
- [ ] Items are added, edited and removed with the item editor decided in T1.
- [ ] Text that matches an item shows and counts as unimportant like any other unimportant difference. Orphan lines stay important even when their text matches.
- [ ] An invalid regular-expression item is rejected in the editor with a message.

#### Tasks
- **E07-S15-T1** — Decide the unimportance item editor (Q7) · size: S · depends: — · flags: `decision`
  - AC: decision recorded in this story. Proposed default: each item is text or a regular expression, with Match character case and Whole words only flags, and applies to both sides.
- **E07-S15-T2** — Classify text that matches list items as unimportant · size: M · depends: E07-S15-T1
  - AC: with the regex item `Build \d+`, a pair that differs only in the build number shows blue.
- **E07-S15-T3** — Unimportance list editor on the Importance tab · size: S · depends: E07-S15-T1
  - AC: items can be added, edited and removed, and the view recompares after OK.

### E07-S16 — Text replacements
**Priority**: Should · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E07-S4 (importance rules), E07-S2 (difference navigation), E02-S1 (regular expressions and the input aid)
**Traces**: F-12.9, F-12.4, §3.1

As a developer reviewing a rename refactor, I can declare that some text on one side corresponds to other text on the other side so that systematic substitutions show as unimportant.

**Acceptance criteria**
- [ ] The **Replacements** tab holds a list of items. Each item has:
  - **Text to find**
  - **Replace with**
  - *Match character case*
  - *Whole words only*
  - *Regular expression*: replacement-style with backreferences, the element picker and RE Examples
  - **Side** to search
- [ ] When text on one side matches *find* and the other side holds the *replace with* value, the difference is unimportant. Examples are `apple`↔`orange` and a renamed identifier.
- [ ] *Edit > Replacement* creates a replacement item from the current difference, filled with both sides' text.
- [ ] *Next Replacement* and *Previous Replacement* jump between the text matched by replacement rules.

#### Tasks
- **E07-S16-T1** — Match replacements: find on the chosen side and replace-with on the other, with case, whole-word, regex and backreference support · size: M · depends: —
  - AC: the regex `old_(\w+)` with replace `new_$1` makes `old_count` vs `new_count` unimportant.
- **E07-S16-T2** — Replacements tab and item editor · size: S · depends: E07-S16-T1
  - AC: the element picker and RE Examples are available while Regular expression is on. The view recompares after OK.
- **E07-S16-T3** — Edit > Replacement from the current difference · size: S · depends: E07-S16-T1
  - AC: on a difference `foo`→`bar`, the command creates an item that finds `foo` and replaces it with `bar`, and the difference turns blue.
- **E07-S16-T4** — Next/Previous Replacement · size: S · depends: E07-S16-T1
  - AC: each replacement-matched span is visited in document order.

### E07-S17 — Ignored lines
**Priority**: Could · **Layer**: 2 · **Size**: S · **Platforms**: All
**Needs**: E07-S1 (Text Compare view)
**Traces**: F-12.2

As a code reviewer, I can mark specific lines or sections as ignored so that known, accepted differences stop showing.

**Acceptance criteria**
- [ ] *Ignored* suppresses differences for the selected lines, or for the current section when nothing is selected. They show and count as same.
- [ ] Choosing *Ignored* again on ignored lines removes the suppression, and Undo also restores it (*inferred*).

#### Tasks
- **E07-S17-T1** — Ignored ranges in the view's classification · size: S · depends: —
  - AC: ignoring a section removes it from the statistics and from the status-bar result.
- **E07-S17-T2** — Ignored command as a toggle · size: S · depends: E07-S17-T1
  - AC: choosing Ignored twice on the same lines brings their differences back.

### E07-S18 — Clipboard comparisons
**Priority**: Should · **Layer**: 2 · **Size**: S · **Platforms**: All
**Needs**: E07-S1 (Text Compare view)
**Traces**: F-12.10, F-7.2

As a developer, I can compare the clipboard with a file or with selected text so that I can check snippets without saving them first.

**Acceptance criteria**
- [ ] *File > Open Clipboard* loads the clipboard text into the chosen pane, which then compares like a file.
- [ ] `clipboard://` is accepted as a spec wherever a file path is, including the path edit and the command line.
- [ ] *Edit > Compare Selection to Clipboard* opens a new Text Compare with the selected text on one side and the clipboard on the other.
- [ ] A clipboard pane has no file behind it, so Save File asks for a path, as Save File As does (*inferred*).

#### Tasks
- **E07-S18-T1** — Clipboard text as a read-only `clipboard://` source · size: S · depends: — · flags: `licence-check`, `naming-check`
  - AC: launching with `clipboard://` and a file compares the current clipboard text with the file.
- **E07-S18-T2** — File > Open Clipboard into the chosen pane · size: S · depends: E07-S18-T1
  - AC: the pane shows the clipboard text and the comparison updates.
- **E07-S18-T3** — Edit > Compare Selection to Clipboard · size: S · depends: E07-S18-T1
  - AC: a new tab opens, comparing exactly the selected text with the clipboard.

### E07-S19 — Next Difference options
**Priority**: Should · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E07-S2 (difference navigation), E07-S7 (copy to other side), E08-S10 (child sessions)
**Traces**: F-12.4, F-26.11

As a code reviewer working through many files, I can tune how difference navigation behaves so that walking through changes takes as few keystrokes as possible.

**Acceptance criteria**
- [ ] The **Next Difference** options page has *When loading new files, go to the first difference*. With it on, a newly loaded comparison puts the cursor on the first difference.
- [ ] With *Go to the next difference after copying to the other side* on, the cursor moves to the next difference after each copy.
- [ ] With *Limit "Next Difference Files" to the current folder* on, Next and Previous Difference Files in a child session stay within the current folder of the parent comparison.
- [ ] With *Wrap around to the first difference* on, Next at the last difference goes to the first one, and Previous at the first goes to the last.
- [ ] With *Show message panel (and beep) when not found* on, a message panel appears and the app beeps when there are no more differences that way.

#### Tasks
- **E07-S19-T1** — Next Difference options page with the five options · size: S · depends: —
  - AC: each option persists, and each one changes behaviour only while it is ticked.
- **E07-S19-T2** — Go to the first difference on load · size: S · depends: E07-S19-T1
  - AC: opening a pair whose first difference is on line 200 puts the cursor there.
- **E07-S19-T3** — Go to the next difference after copying · size: S · depends: E07-S19-T1
  - AC: after copying section 1 across, the cursor is on section 2.
- **E07-S19-T4** — Wrap around, and the message panel with beep · size: S · depends: E07-S19-T1
  - AC: with wrap on, Next on the last section goes to the first. With wrap off and the panel on, it shows the message and beeps.
- **E07-S19-T5** — Limit Next Difference Files to the current folder · size: M · depends: E07-S19-T1
  - AC: in a child session for `src/a.txt`, Next Difference Files skips a different pair in `docs/` while the option is on.

### E07-S20 — Editor display tweaks
**Priority**: Could · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E07-S7 (copy to other side), E10-S2 (syntax highlighting), E07-S9 (layouts)
**Traces**: F-12.1, F-26.11
**Not in this story**: single vertical scrollbar, sticky splitter, hints → E22-S5.

As a power user, I can fine-tune how the editor panes look so that the display suits my monitor and my reading habits.

**Acceptance criteria**
- [ ] The Tweaks *Editor Display* group has *Show syntax highlighting on difference lines* (off by default). With it on, difference lines keep their syntax colours.
- [ ] *Crosshatch lines beyond end of file* fills the area past each file's last line with a crosshatch.
- [ ] *Right-side gutter for the left editor* puts copy buttons in the middle between the panes, with scrollbars outside. Each button copies its adjacent section across. The option is unavailable in the over-under layout.
- [ ] *Use orphan colour* gives orphan lines their own colour.
- [ ] *Extra line spacing* adds or removes pixels between lines (±px).
- [ ] *Show column line at N* draws a guide at column *N*, and 0 turns it off. N is counted in space-character widths.
- [ ] *Dim inactive pane by X* dims the pane without focus by the chosen amount.

#### Tasks
- **E07-S20-T1** — Syntax highlighting on difference lines · size: S · depends: —
  - AC: with the tweak on, a keyword on a red line keeps its keyword colour, and the difference highlight still shows.
- **E07-S20-T2** — Crosshatch past end of file, and the orphan colour · size: S · depends: —
  - AC: the shorter file shows a crosshatch below its last line. Orphan lines use the orphan colour while the tweak is on.
- **E07-S20-T3** — Right-side gutter with middle copy buttons · size: M · depends: —
  - AC: clicking a middle button copies that section across. The tweak is greyed out in over-under layout.
- **E07-S20-T4** — Extra line spacing and the column guide line · size: S · depends: —
  - AC: +2 px spacing increases the row height in both panes, and column 80 shows a guide that a value of 0 removes.
- **E07-S20-T5** — Dim the inactive pane · size: S · depends: —
  - AC: moving focus to the right pane dims the left pane by the configured amount.

### E07-S21 — Webpages panel
**Priority**: Could · **Layer**: 2 · **Size**: M · **Platforms**: Win, Mac
**Needs**: E07-S1 (Text Compare view)
**Traces**: F-12.1, App-C

As a web developer, I can see both files rendered as web pages beneath the comparison so that I can check how a markup change looks.

**Acceptance criteria**
- [ ] *View > Webpages* shows an embedded browser panel that renders the current left and right files as web pages.
- [ ] The rendering updates after Recompare or Reload (*inferred*).
- [ ] The panel is offered on Windows and macOS only.

#### Tasks
- **E07-S21-T1** — Embedded browser panel that renders each side · size: M · depends: — · flags: `licence-check`, `naming-check`
  - AC: two HTML files that differ in one heading render side by side with the different headings visible.
- **E07-S21-T2** — Refresh on recompare, and platform availability · size: S · depends: E07-S21-T1
  - AC: after an edit and Recompare, the rendering shows the edit. The command doesn't appear on Linux.

### E07-S22 — Detect files changed on disk
**Priority**: Should · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E07-S1 (Text Compare view), E15-S1 (Table Compare view), E16-S1 (Hex Compare view), E17-S1 (Picture Compare view)
**Traces**: F-26.11

As a developer who also edits files in other tools, I can be told when an open file changed on disk and reload it so that I never compare stale content.

**Acceptance criteria**
- [ ] In the Text, Table, Hex and Picture views, when the view regains focus after a loaded file changed on disk, the prompt "File changed on disk, reload?" appears.
- [ ] Answering yes reloads that file and recompares. Answering no keeps the loaded version and doesn't ask again until the file changes again (*inferred*).
- [ ] The File Operations tweak *check for files changed on disk* turns detection on or off.
- [ ] With the tweak *automatically reload unless changes would be discarded* on, views without unsaved edits reload silently, and views with unsaved edits still prompt.

#### Tasks
- **E07-S22-T1** — Detect an external change to a loaded file when its view regains focus · size: S · depends: —
  - AC: touching a loaded file in another program and switching back is detected. Switching back with no change raises nothing.
- **E07-S22-T2** — Prompt and reload in the Text, Table, Hex and Picture views · size: M · depends: E07-S22-T1
  - AC: in each of the four views, answering yes shows the new content and answering no keeps the old.
- **E07-S22-T3** — "Check for files changed on disk" and automatic-reload tweaks · size: S · depends: E07-S22-T2
  - AC: with automatic reload on, an unedited view reloads without a prompt, and an edited view still prompts.
