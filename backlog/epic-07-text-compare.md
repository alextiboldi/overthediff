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
- [ ] A Text Compare session opens from: (Built and tested; *File > Open File*'s native OS picker awaits E07-S1-T2's check by hand.)
  - the Home launcher
  - *Session > New Session ▸ Text Compare*
  - *File > Open File* (into the chosen pane)
  - the hidden-by-default *Open File on File System*
  - the command line, with two files that resolve to a Text format
- [ ] Each pane has a path edit with typed paths, Browse for File and history. (Built and tested with an overridden picker; the real OS picker awaits E01-S1-T9's check by hand.)
- [ ] The two editor panes scroll together. Lines are aligned and gaps are padded, so matching lines sit on the same row. (Built and tested; scrolling with the wheel and the scroll bar in the running app awaits E07-S1-T4's check by hand.)
- [ ] Important differences show in red text, their lines get a light-red background, and the changed characters inside each line are highlighted. (Built and tested; the colours in the light and dark themes await E07-S1-T4's check by hand.)
- [x] Each pane shows line numbers, and gap rows have none.
- [x] Each pane has a file info header with path, size, modified time, detected file format, encoding and line-ending style.
- [x] Encoding is detected from a byte-order mark, valid UTF-8 or UTF-16, falling back to the system code page. Line endings (Windows, Mac, Unix) are detected per file, and mixed endings show as mixed (*inferred*).
- [ ] Each pane is a full text editor. Typing, *Undo*/*Redo*, *Cut*/*Copy*/*Paste*/*Delete* and *Select All* (the visible lines in the pane) all work, and the comparison updates live as the user types. (Built and tested; the clipboard with other apps, the native macOS Edit menu and input methods await E07-S1-T5's check by hand.)
- [ ] *Save File* writes the pane back in its original encoding and line-ending style. *Save File As* writes to a new path, and the hidden-by-default *Save File on File System As* saves to a local path. (Built and tested; Save File As's native OS save picker and Cmd+S from the macOS menu bar await E07-S1-T6's check by hand.)
- [x] A pane with unsaved edits is marked, and Close Tab and Exit prompt to save.
- [x] A pane is read-only, with the reason shown, when its location is read-only or the comparison errored or was cancelled.
- [x] *Reload Files* prompts to save unsaved edits and then reloads both files. *Recompare Files* reruns the comparison, keeps edits and clears undo history.
- [x] *Swap Sides* exchanges the left and right files and recompares.
- [ ] *Text Compare Info* shows comparison statistics. The status bar shows the overall result and the load time, as decided in T1. (Built and tested; the Session menu, status bar and Info dialog in the running app await E07-S1-T8's check by hand.)

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
- **E07-S1-T2** — Open a Text Compare session from Home, New Session, File > Open File, Open File on File System, the path edits and the command line · size: M · depends: — · flags: `naming-check` · status: needs-verification
  - AC: launching the product with two text files opens one Text Compare tab. A typed or browsed path loads into its pane. A missing file shows an error in that pane and leaves it read-only.
  - Implemented: `apps/otd/src/session_types.rs` registers `otd_ui_text_compare::SESSION_TYPE`, so Home and *Session > New Session ▸ Text Compare* open it. In `crates/ui-text-compare`, each pane has a path edit (`text-compare.left`, `text-compare.right`) whose path loads when used (typed and left, browsed or picked from Recent paths; `PathEdit::show_used` and `browse_now` in `crates/ui/src/path_edit.rs`), retrying the same path reloads that side, and a pane that can't load shows its error in place, announced, with no document. `src/open.rs` holds *File > Open File* (`text-compare.open_file`, in the new `File` menu after Session) and the hidden *Open File on File System*, both taking an optional `side` (the chosen pane by default) and `path`, and the bare-paths launch handler `OpenFiles` (`otd <left> <right>`, paths made absolute against the launching folder), refused before the hand-off unless there are two paths through the new `Handler::check` in `crates/launch`. Names checked in `docs/feature-names.md`. Checks: `cargo xtask ci` on macOS after rebasing onto main, pull-request CI on Linux and macOS, and the full Linux, macOS and Windows matrix on the branch. https://github.com/alextiboldi/overthediff/pull/83
  - Verify by hand: in the running app, File > Open File opens the native OS file picker, and the file picked there loads into the chosen pane (the side whose path edit last had focus or that was last clicked).
- **E07-S1-T3** — Load files through the data-source layer with encoding and line-ending detection, and show the file info header · size: M · depends: — · flags: `licence-check`, `craft-libs` · status: done
  - AC: UTF-8 with and without a BOM, UTF-16 LE and BE, and a legacy code-page file each display correctly with the detected encoding shown. CRLF, CR and LF files show their line-ending style.
  - Implemented: `crates/text-compare` (`otd-text-compare`, UI-free) loads a pane through any `DataSource` (`load`, or `open` for a typed spec via `LocalSource`), resolves its format through the E02-S1 registry and detects its encoding (BOM, UTF-8, UTF-16 LE/BE without a BOM, then the system code page: craft-codepage single-byte tables, encoding_rs for 932/936/949/950, the Windows ANSI page read from the registry) and line endings (Windows, Mac, Unix, Mixed, None). `crates/ui-text-compare` (`otd-ui-text-compare`) is `TextCompareView` with a captioned left and right pane, each with `file_info_header` (path, size, modified, format, encoding, line endings); `SESSION_TYPE` is left for T2 to register. Dependencies craft-codepage, encoding_rs, windows-registry and chrono `clock` are in NOTICE and ATTRIBUTION.md; the reusable detector is #71 for craft-libs. Checks: `cargo xtask ci` on macOS, `cargo check --target x86_64-pc-windows-msvc -p otd-text-compare`, pull-request CI and the full Linux, macOS and Windows matrix on the branch. https://github.com/alextiboldi/overthediff/pull/73
- **E07-S1-T4** — Render the aligned comparison: synchronised scrolling, gap padding, red text, light-red line background, character highlights and line numbers · size: L · depends: E07-S1-T3 · flags: `craft-libs`, `accessibility` · status: needs-verification
  - AC: for a pair with one inserted, one deleted and one changed line, matching lines share rows and gaps appear opposite the inserted and deleted lines. Only the changed characters are highlighted, and scrolling either pane scrolls both.
  - Implemented: `crates/ui-text-compare/src/aligned.rs` adds `Aligned`, which draws the text engine's `compare` rows in both panes at one fixed row height, so matching lines share a row and gap rows (no number, text or node) sit opposite inserted and deleted lines. It classifies with no rules yet, so every difference is important. Important rows draw in `IMPORTANT_TEXT` on `IMPORTANT_BACKGROUND` to the pane's right edge, and their changed characters (`char_differences`, cached per row) draw on `IMPORTANT_CHARACTERS`, underlined, through `line_job`. Each file keeps its own line numbers in a gutter that stays in view when the pane scrolls sideways, with `~ - +` markers and an accessible description (Changed, Deleted, Inserted). Both panes share one vertical offset. In `src/lib.rs`, the headers are laid out in one pass and the bodies in a second, so both bodies start at the same height. The roles and their `CONTRAST` pairs are registered in `apps/otd/src/colour_roles.rs`. Moving the widget to craft-libs is tracked in #90. Keyboard scrolling arrives with the editor in E07-S1-T5. Tests: the UI tests `aligned::matching_lines_share_rows_and_gaps_without_numbers_sit_opposite_inserted_and_deleted_lines`, `aligned::scrolling_either_pane_scrolls_both` and `aligned::a_pane_scrolled_sideways_keeps_its_gutter_in_view`, plus the unit test `aligned::tests::a_changed_line_marks_only_its_changed_characters_and_keeps_all_its_text`. The app's contrast test covers both palettes. Checks: `cargo xtask ci` on macOS and pull-request CI on Linux and macOS. The Windows matrix did not run, because the change is rendering only. https://github.com/alextiboldi/overthediff/pull/91
  - Verify by hand: open two text files in Text Compare in the light and dark themes. Check that important lines show red text on a light-red background, that changed characters are on a stronger red and underlined, that line numbers appear and gap rows have none, and that the `~ - +` markers sit in the margin. Scroll with the wheel and drag the scroll bar in each pane: the other pane must follow vertically, with rows staying aligned. Scroll one pane sideways past a short important line: its light-red background must still reach the right edge.
- **E07-S1-T5** — Edit with live recompare: typing, Undo/Redo, Cut/Copy/Paste/Delete and Select All · size: M · depends: E07-S1-T4 · flags: `accessibility` · status: needs-verification
  - AC: typing the other side's text into a changed line removes its highlight without a manual recompare. Undo brings the difference back.
  - Implemented: `crates/text-compare/src/edit.rs` (UI-free) gives `Pane` a character-counted `Selection` and an undo `History`: every edit is one splice of whole lines, Undo and Redo are its inverse, typing on one line is one step until the caret moves, new line breaks take the split line's ending or the pane's style, and `History::clear` serves Recompare Files. `crates/ui-text-compare/src/edit.rs` makes each pane body an editor over E07-S1-T4's aligned rows: it takes real focus from Tab or a click, moves and selects with the arrows, Home, End, Page Up/Down and the platform chords, handles typing (Insert or Overwrite from the status bar), Enter, Backspace, Delete, the clipboard's Cut, Copy and Paste events, Undo, Redo and Select All, commits input-method compositions, and recompares both panes after each edit, scrolling both to the caret. The Edit menu gains `text-compare.undo`, `redo`, `cut`, `copy`, `paste`, `delete` and `select_all`, disabled when there is nothing to act on or the pane is read-only, plus the hidden typed `text-compare.insert_text` for scripts and automation. `aligned.rs` draws the caret and selection in the new `SELECTION` and `SELECTION_TEXT` roles, keeping changed characters underlined. The editor is a `MultilineTextInput` named by its side, its description says how the caret's line differs, and its lines are published as `TextRun` children with the text selection. Recompare Files now empties the undo history, Swap Sides keeps it, and the status bar shows the caret's line and column. AGENTS.md documents the editor; `docs/feature-names.md` records the captions. Moving the editor to craft-libs is tracked in #99, CJK fonts in #103. Tests: the AC test `edit::typing_the_other_sides_text_into_a_changed_line_clears_its_highlight_and_undo_brings_it_back` through the real `Shell`, 14 further UI tests in `tests/it/edit.rs` (clipboard, Edit menu commands, read-only refusal, scrolling, presses, plain-key typing, Insert Text without the window, no keyboard trap, Redo shortcuts, Delete beside a path edit, input methods, the T8 hand-offs, published selection) and unit tests in both crates. Checks: `cargo xtask ci` on macOS after the rebase onto main, pull-request CI on Linux and macOS, and the full Linux, macOS and Windows matrix on the branch. https://github.com/alextiboldi/overthediff/pull/98
  - Verify by hand: in the running app, copy text from another app and paste it into a Text Compare pane, then copy from a pane and paste it into another app; on macOS, use Edit > Undo, Redo, Cut, Copy, Paste and Select All from the native menu, and check that Cmd+V pastes once and Cmd+Z undoes one step through the menu's key equivalents; on Windows and Linux, use the same items from the in-window menu bar; check that the caret, the selection and the pane's focus ring are visible in the light and dark themes, including on red (important) lines; drag with the mouse in a pane and check that the drag selects text; compose text with an input method (a macOS dead key such as Option+E then E, and a Japanese or Chinese input method) and check that the composition shows at the caret and commits into the pane; with VoiceOver (or NVDA on Windows), check that moving the caret within a line reads the character and that Shift+arrow selection is announced.
- **E07-S1-T6** — Save File, Save File As, Save File on File System As, and unsaved-changes prompts · size: M · depends: E07-S1-T5 · status: needs-verification
  - AC: saving keeps the file's encoding and line endings, so only the edited bytes change. Close Tab on an edited pane offers save, discard or cancel.
  - Implemented: `crates/text-compare` adds `encode` in `decode.rs`, the inverse of `decode`, which writes the recorded byte-order mark and each line's own ending and refuses a character its encoding can't hold, naming its line and character. `save` writes a pane through any `DataSource` and `save_as` to a typed spec, then refresh its size and modified time and clear `Pane::unsaved`, which every edit, Undo and Redo set. A file that doesn't read back byte for byte loads as `ReadOnly::Encoding`, and a file marked read-only (no write bits on Unix, the read-only attribute on Windows) as `ReadOnly::File`; neither saves in place. `crates/ui-text-compare/src/save.rs` registers *File > Save File* (`text-compare.save_file`, Cmd/Ctrl+S), *Save File As* (`text-compare.save_file_as`, Cmd/Ctrl+Shift+S) and the hidden *Save File on File System As* (`text-compare.save_file_on_file_system_as`), each with an optional `side` and the As commands with `path`, opening the OS save picker without one through `ask_save` in `crates/ui/src/path_edit.rs`. A pane with unsaved edits shows "Unsaved changes" beside its caption, a failed save says why in the pane, and `View::unsaved` and `View::save` make Close Tab, Exit and Reload Files ask first. Loading another file into an edited pane without asking is #104. Tests: `crates/text-compare/tests/it/save.rs` (only the edited bytes change in six encodings and ending styles, unencodable characters refused, lossy and read-only files) and `crates/ui-text-compare/tests/it/save.rs` through the real `Shell` (Cmd/Ctrl+S from the editor, Close Tab's save, discard or cancel by keyboard, Save File As through the picker and with a path, a failed save, Reload Files' prompt on a real edit), plus the hostile sweep now encoding what it decodes. Checks: `cargo xtask ci` on macOS, pull-request CI on Linux and macOS, and the full Linux, macOS and Windows matrix on the branch. https://github.com/alextiboldi/overthediff/pull/105
  - Verify by hand: in the running app, edit a Text Compare pane and use *Save File As* without a path: the OS's native save picker opens and the pane is saved to the chosen file; then edit again and choose *Save File* (Cmd+S) from the macOS menu bar, not the in-window shortcut, and check that the focused pane is saved.
- **E07-S1-T7** — Read-only panes with a visible reason · size: S · depends: E07-S1-T3 · status: done
  - AC: a file on a read-only data source opens read-only and says why. After a cancelled comparison both panes are read-only.
  - Implemented: `Pane::read_only` in `crates/text-compare` holds an `otd_text_compare::ReadOnly` reason, and `load` sets `Location` when the data source's `Capabilities::read_only` is set. `crates/ui-text-compare/src/read_only.rs` adds `TextCompareView::comparison_ended(End)`, which makes both panes read-only after a failed or cancelled comparison until one completes or other files load, and `TextCompareView::read_only(side)`, which editing and saving check; each read-only pane shows its translated reason below its file info header as a polite live region. The comparison itself starts in E07-S1-T4, which calls `comparison_ended`. Tests: `load::a_file_on_a_read_only_source_loads_read_only_for_its_location`, and UI tests for a file on the read-only macOS system volume and for a cancelled comparison. Checks: `cargo xtask ci` on macOS, pull-request CI on Linux and macOS, and the full Linux, macOS and Windows matrix on the branch. https://github.com/alextiboldi/overthediff/pull/89
- **E07-S1-T8** — Session commands: Reload Files, Recompare Files, Swap Sides, Text Compare Info and the status bar · size: M · depends: E07-S1-T1, E07-S1-T4 · status: needs-verification
  - AC: Reload Files with unsaved edits prompts first. Recompare Files keeps edits and empties undo history. Swap Sides moves each file to the other pane. Info and the status bar show what T1 decided.
  - Implemented: `crates/ui-text-compare/src/session.rs` registers `text-compare.reload_files`, `text-compare.recompare_files`, `text-compare.swap_sides` and `text-compare.info` in the Session menu with no default shortcut. Reload Files goes through the new `Shell::save_first` in `crates/ui/src/unsaved.rs`, which asks the unsaved-changes question first and runs the reload after Save or Don't Save, not after Cancel. Recompare Files compares the panes as they stand without reading the files and lifts the read-only state a failed or cancelled comparison left. Swap Sides exchanges both sides' specs, documents and path edits in memory, then recompares. Text Compare Info opens a modal dialog with each side's total lines and the difference sections, changed, inserted and deleted lines, mapped from the text engine's `Statistics` as this story's Decisions describe, and returns the same text to scripts. `TextCompareView::status` shows the overall result, one shared Insert/Overwrite state and the load time; `Aligned` in `aligned.rs` keeps the classification's outcome and statistics so drawing, status and Info agree. The cursor field and emptying the undo history arrive with the editor in E07-S1-T5; the Reload prompt on a real Text Compare edit is proven only at the shell with a test view until then. AGENTS.md records where the commands live, and `docs/feature-names.md` the naming check. Tests: `unsaved::save_first_on_an_unsaved_view_asks_then_runs_after_save_or_dont_save_but_not_after_cancel`, `session::tests::recompare_compares_the_panes_as_edited_without_reading_the_files_and_reload_reads_them`, `session::swap_sides_moves_each_file_to_the_other_pane_and_compares_them_again`, `session::info_counts_each_panes_lines_and_its_changed_inserted_and_deleted_lines_for_the_window_and_scripts`, `session::after_a_cancelled_comparison_neither_the_status_bar_nor_info_shows_a_result_until_one_completes` and `session::the_info_dialog_takes_focus_lists_the_figures_and_gives_focus_back_on_esc_or_close`. Checks: `cargo xtask ci` on macOS, pull-request CI on Linux and macOS, and the full Linux, macOS and Windows matrix on the branch. https://github.com/alextiboldi/overthediff/pull/97
  - Verify by hand: in the running app, open two different text files in Text Compare and check that the Session menu lists Reload Files, Recompare Files, Swap Sides and Text Compare Info; that the status bar shows the result, Insert (toggles to Overwrite when clicked) and the load time; and that Text Compare Info opens a dialog with the figures that closes on Esc and gives focus back.
- **E07-S1-T9** — Ask before loading another file over a pane's unsaved edits · size: S · depends: E07-S1-T6 · status: done
  - AC: loading a file into a pane with unsaved edits, through its path edit (typed, browsed or a recent path) or *File > Open File*, first asks Save / Don't Save / Cancel, as Close Tab does. Save and Don't Save then load the file; Cancel keeps the edits, the old file and its path in the path edit. A pane without unsaved edits loads without asking. (Added for #104, found in E07-S1-T6.)
  - Implemented: `TextCompareView::load` in `crates/ui-text-compare/src/load.rs` holds the new file when the target pane has unsaved edits, and the pane's path edit (typed, browsed or a recent path) and *File > Open File* (`open.rs`, `lib.rs`) then ask Save / Don't Save / Cancel through `Shell::save_first`, loading the held file after Save or Don't Save (`load_pending`) and keeping the edits, the old file and its path after Cancel. A view asks from its own drawing with the new `otd_ui::save_first_later` in `crates/ui/src/unsaved.rs`, which the shell runs once the view has drawn. A clean pane loads without asking. AGENTS.md records it under Unsaved changes and Saving. Tests: `load::a_path_typed_over_an_edited_pane_asks_and_cancel_keeps_the_edits_and_the_old_file` and `load::open_file_over_an_edited_pane_asks_whether_given_a_path_or_browsing_but_not_over_a_clean_one` in `crates/ui-text-compare/tests/it/load.rs`, through the real `Shell`. Checks: `cargo xtask ci` on macOS after the rebase onto main, and pull-request CI on Linux and macOS. The Windows matrix did not run: the change has no platform-specific code. Closes #104. https://github.com/alextiboldi/overthediff/pull/128

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
- **E07-S2-T1** — Find the next or previous section and the next or previous in-line difference from the cursor · size: M · depends: — · status: done
  - AC: unit cases cover a cursor before, inside and after a section. A line with three separate changes is visited change by change, in order.
  - Implemented: `crates/text-compare/src/navigate.rs` (`otd-text-compare`, UI-free) adds `Differences`, which walks a comparison's rows with the view's own "row differs" test: `next_section` and `previous_section` find the maximal run of differing rows after or before a row, and `next`, `previous` and `changes` find the in-line changes (each run of differing characters, paired across both sides, so an insertion or deletion is one stop from either pane; an orphan line or an ending-only row is one change) strictly past a `Cursor` (row, side, column). Tests in `crates/text-compare/tests/it/navigate.rs`: `sections_are_found_from_a_cursor_before_inside_and_after_them`, `a_row_the_view_treats_as_same_is_skipped`, `a_line_with_three_changes_is_visited_change_by_change_in_order_from_either_pane`, `an_ending_only_change_and_an_orphan_line_are_one_stop_each` and `a_caret_at_a_change_start_is_before_it_and_at_its_end_after_it`. Checks: `cargo xtask ci` on macOS after rebasing onto main, and pull-request CI on Linux and macOS. https://github.com/alextiboldi/overthediff/pull/110
- **E07-S2-T2** — Navigation commands and shortcuts that bring the target into view in both panes · size: S · depends: E07-S2-T1
  - AC: Next Difference Section from the top of the file lands on the first section, with both panes showing it.
- **E07-S2-T3** — Track edited lines, and add Next/Previous Edit · size: S · depends: — · status: done
  - AC: after edits on lines 10 and 40, Next Edit from line 1 visits line 10 and then line 40.
  - Implemented: `otd-text-compare`'s `Pane` keeps an `edited` mark per line (`crates/text-compare/src/lib.rs`): every edit marks the lines it leaves, and Undo puts back the marks it replaced, so an undone edit's line is no longer edited; `Pane::next_edit` and `previous_edit` (`src/edit.rs`) find the start of the next or previous run of edited lines, so a block of edited lines is one stop. `crates/ui-text-compare/src/edited.rs` registers *Search > Next Edit* (`text-compare.next_edit`, Alt+Shift+Down) and *Previous Edit* (`text-compare.previous_edit`, Alt+Shift+Up), which move the chosen pane's caret to the line, scroll both panes to it and focus the pane, and are disabled when there is no edit that way; while disabled, their chord leaves the caret alone. Names checked in `docs/feature-names.md`. Tests: `edit::tests::next_edit_visits_each_edited_line_in_order_and_undo_takes_its_mark_back`, `edited::next_edit_from_line_1_visits_line_10_then_line_40_with_both_panes_showing_it`, `edited::past_the_last_edit_next_edit_is_disabled_and_its_chord_leaves_the_caret_alone` and `edited::previous_edit_from_below_visits_the_later_edit_then_the_earlier_one`. Checks: `cargo xtask ci` on macOS after rebasing onto main, pull-request CI on Linux and macOS, and the full Linux, macOS and Windows matrix on the branch. https://github.com/alextiboldi/overthediff/pull/118
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
- **E07-S3-T1** — Work out which rows each filter shows (All, Differences, Same, None, Context with *N*) · size: M · depends: — · status: done
  - AC: a unit table for a file with two sections and *N* = 2 gives the expected visible rows under every filter, including overlapping context ranges.
  - Implemented: `crates/text-compare/src/filter.rs` (UI-free, exported from `otd-text-compare`) adds `Filter` (All, Differences, Same, None, Context) and `visible(differs, filter, context)`, which takes each aligned row's different-or-same flag from the view's current classification and returns the shown rows as ascending ranges that neither overlap nor touch, so each gap is one run of hidden rows and both panes show the same rows. Context with *N* shows each different row and *N* rows either side, clamped to the file and merged where ranges overlap or touch. Tests: the unit table `filter::tests::each_filter_shows_the_differences_same_rows_or_context_it_names` (two sections, *N* = 2, every filter, overlapping context) and `nothing_shows_when_there_are_no_rows_or_no_differences_to_give_context_to`. Checks: `cargo xtask ci` on macOS and pull-request CI on Linux and macOS; the Windows matrix did not run, because the change is platform-independent row arithmetic. https://github.com/alextiboldi/overthediff/pull/106
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
- **E07-S4-T1** — Classify leading, embedded and trailing whitespace, and the case of "everything else" · size: M · depends: — · status: done
  - AC: unit cases. An indentation-only change is unimportant when Leading whitespace is unticked. `a  b` vs `a b` follows Embedded whitespace. A case-only change is unimportant only when Character case is unticked.
  - Added scope (issue #29, orchestrator default pending the operator): each character range of a changed row also carries its importance by the active rules. A range whose characters all lie in spans the rules mark unimportant, or that only changes letter case where case doesn't matter, is unimportant, and, apart from a different line ending, a row is unimportant exactly when none of its ranges is important. T5 shows it.
  - *Inferred*: a line of only whitespace has no first or last other character, so its whitespace counts as both leading and trailing and is unimportant when either element is unticked. Whitespace is Unicode White_Space. With Embedded whitespace unticked, `ab` vs `a b` is unimportant too, since whitespace that is unimportant is left out. Case is folded one character at a time to its lower case, so pairs such as `Σ` and `ς` still differ.
  - Implemented: `BuiltInRules` (Leading, Embedded and Trailing whitespace, Everything else, Character case; all important by default) in `crates/text-engine/src/builtin.rs` (`otd-text-engine`), a `Rule` that marks unimportant spans and spans whose case is folded; `Rule::case_insensitive` and lower-case folding in `classify` (`importance.rs`); and, for issue #29, `char_differences_with` in `chars.rs`, which gives each `CharRange` its importance by the rules. Tests in `crates/text-engine/tests/it/builtin.rs` and `importance.rs` cover the AC. `cargo xtask ci` passed on macOS; pull-request CI on Linux and macOS. https://github.com/alextiboldi/overthediff/pull/117
- **E07-S4-T2** — Compare line endings option · size: S · depends: — · status: done
  - AC: a CRLF file and its LF copy compare as same by default. With the option on, every line is flagged as different.
  - Implemented: `Options { line_endings }` (*Compare line endings*, off by default) in `crates/text-engine` (`otd-text-engine`): with it off a pair whose text matches is a same row whatever its endings, with it on a different ending makes a changed row; `Comparison` carries its options so `classify` follows them; `compare_with` takes options and `compare_progressive` gains an `options` argument. Tests in `crates/text-engine/tests/it/importance.rs` and `alignment.rs` cover both AC. `cargo xtask ci` passed on macOS; pull-request CI on Linux and macOS. The Importance tab checkbox is E07-S4-T4. https://github.com/alextiboldi/overthediff/pull/114
- **E07-S4-T3** — Keep orphan lines important · size: S · depends: E07-S4-T1 · status: done
  - AC: an inserted blank line stays red with every whitespace element unticked.
  - Implemented: no engine change was needed: `classify` in `crates/text-engine/src/importance.rs` (`otd-text-engine`) already makes every orphan row (a line against a gap) important before any rule is read. The test `an_inserted_or_deleted_blank_line_stays_important_with_every_whitespace_element_unticked` in `crates/text-engine/tests/it/builtin.rs` covers the AC: with every built-in element unticked, an inserted or deleted empty or whitespace-only line is one important row and the outcome is Important; it fails when orphans are classified by their text. The view paints important rows red (`crates/ui-text-compare`); the rules reach it with E07-S4-T4. `cargo xtask ci` passed on macOS; Linux and Windows run in main's CI.
- **E07-S4-T4** — Importance tab controls · size: S · depends: E07-S4-T1, E07-S4-T2 · status: done
  - AC: ticking or unticking an element recompares the open view. The values are kept with the session settings.
  - Implemented: an Importance tab in the Session Settings dialog for the Text Compare type (`crates/ui-text-compare/src/importance.rs`, registered in `apps/otd/src/session_settings.rs`): Leading, Embedded and Trailing whitespace, Everything else, Character case and Compare line endings checkboxes, stored in the session's rules (`otd_core::session`) so they are saved with the session. The view (`aligned.rs`) compares and classifies with those rules and recompares the open panes, keeping undo and scroll, when the tab's values change. `crates/ui-text-compare/tests/it/importance.rs` drives the shell by keyboard and checks the status bar result changes at once after OK. `cargo xtask ci` passed on macOS; Linux and Windows run in main's CI.
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
- **E07-S5-T1** — Match over both panes: plain text, case, whole words, regex, sides, wrap and direction · size: M · depends: — · flags: `licence-check`, `craft-libs` · status: done
  - AC: unit cases include the catalogue's whole-word example and a regex with a backreference.
  - Implemented: `crates/text-compare/src/find.rs` (UI-free, re-exported from `otd-text-compare`) is the matcher: `Finder::new(text, FindOptions)` compiles plain text (escaped, read literally) or an A.2 regex, ignoring case unless Match character case is on, with whole words, wrap and sides (Left, Right, Both); `find_all` returns every match in search order with its side, line and character columns, and `find` steps Next or Previous from the selection, past the selected and empty matches, across panes and wrapping when on. The search runs the left pane, then the right (*inferred*: the catalogue does not say how a search crosses panes). `crates/regex` gains `Regex::whole_words`, which puts the word boundary in the pattern so alternatives and overlapping starts still find whole words, and backreferences keep their numbers. No new third-party dependency (`otd-text-compare` depends on `otd-regex` by path), so NOTICE and ATTRIBUTION.md are unchanged; the craft-libs move is #109. Tests in `crates/text-compare/tests/it/find.rs` (the catalogue's whole-word example, a backreference, case, literals, sides, direction, wrap, empty matches) and a hostile sweep in `tests/it/hostile.rs`. Checks: `cargo xtask ci` on macOS after rebasing onto main, and pull-request CI on Linux and macOS. https://github.com/alextiboldi/overthediff/pull/112
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
- [x] *Go To* moves the cursor in the active pane to a given line and column and scrolls both panes there. Values out of range go to the nearest valid position (*inferred*).
- [x] *Toggle Bookmark* sets or clears a bookmark numbered **0–9** on the current line. *Go To Bookmark* jumps to it, and *Clear Bookmarks* removes them all.
- [x] Bookmarked lines are marked in the gutter, and a bookmark moves with its line when lines are inserted or deleted above it (*inferred*).

#### Tasks
- **E07-S6-T1** — Go To line and column · size: S · depends: — · status: done
  - AC: Go To 120:5 puts the cursor on line 120, column 5, of the active pane and both panes show it.
  - Implemented: `crates/ui-text-compare/src/go_to.rs` registers *Search > Go To* (`text-compare.go_to`, Lucide `corner-down-right`, no default shortcut), a modal dialog with Line and Column fields that moves the chosen pane's caret, scrolls it into view through `Aligned::reveal` and gives the pane focus, and the hidden *Go To Line* (`text-compare.go_to_line`, `line` required, `column` optional) for scripts and automation. Lines and columns count from 1; a value out of range goes to the nearest valid position (*inferred*, confirmed by the operator), and text that isn't a whole number keeps the dialog open with a polite live message. Both panes share one vertical offset, so the other pane shows the same row. `crates/ui/src/commands.rs` puts the Search menu after Edit, and `docs/feature-names.md` records the names. Tests: `crates/ui-text-compare/tests/it/go_to.rs` (Go To 120:5 lands on line 120, column 5, with both panes drawing that line on the same row; out-of-range lines and columns; the dialog by keyboard with its accessibility tree, refused text and Esc giving focus back) and a unit test of the dialog's number parsing. Checks: `cargo xtask ci` on macOS and pull-request CI on Linux and macOS; no Windows matrix run, as the change has no platform-dependent code. https://github.com/alextiboldi/overthediff/pull/108
- **E07-S6-T2** — Bookmarks 0–9: toggle, go to, clear, gutter marks, and keeping their position through edits · size: S · depends: — · status: needs-verification
  - AC: after inserting 3 lines above bookmark 2 on line 50, Go To Bookmark 2 lands on line 53.
  - Implemented: `crates/text-compare/src/bookmarks.rs` (`Bookmarks`, each number's line, on `Pane`) follows every `Pane::splice`, so typing, paste, delete, Undo and Redo move a bookmark with its line; a bookmark on a deleted line moves to the next line and is cleared when no lines are left. `crates/ui-text-compare/src/bookmarks.rs` registers *Search > Toggle Bookmark ▸ 0–9*, *Go To Bookmark ▸ 0–9* (disabled while unset; caret to the line's start, pane focused, both panes scrolled there) and *Clear Bookmarks* (both panes), with no default shortcut; toggles and clears are announced in a polite live region. The gutter (`aligned.rs`) shows the line's lowest bookmark number in a box, in existing colour roles, with a "Bookmark N" node per number. `docs/feature-names.md` records the names. Tests: `crates/ui-text-compare/tests/it/bookmarks.rs` (3 lines inserted above bookmark 2 on line 50, then Go To Bookmark 2 lands on line 53 with the mark in view; Clear across both panes) and `edit::tests::a_bookmark_keeps_its_line_through_edits_around_and_on_it_and_their_undo` in `crates/text-compare`. Checks: `cargo xtask ci` on macOS after rebasing onto E07-S6-T1, and pull-request CI on Linux and macOS; no Windows matrix run, as the change has no platform-dependent code. https://github.com/alextiboldi/overthediff/pull/113
  - Verify by hand: in the running app with a screen reader (VoiceOver on macOS), toggle bookmark 2 from the Search menu and confirm "Bookmark 2 set on line N" is spoken, toggle it again and confirm "Bookmark 2 cleared", and run Clear Bookmarks and confirm "Bookmarks cleared"; the tests check only the tree and its live setting.

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
- **E07-S9-T1** — Visible Whitespace glyphs for space, tab, CRLF, CR and LF · size: S · depends: — · status: done
  - AC: a file containing all three line-ending styles shows three different end-of-line glyphs.
  - Implemented: `crates/ui-text-compare/src/whitespace.rs` registers the View menu's check item *Show Whitespace* (`text-compare.show_whitespace`, Lucide icon `pilcrow`, no default shortcut, Q24) through the command registry. While it is on, both panes paint `·` over each space, `→` over each tab, and a different end-of-line glyph for Windows (CR LF, `↵`), Mac (CR, `←`) and Unix (LF, `↓`) breaks, over the laid-out line, so the text, columns, caret and accessible names don't change; `aligned.rs` and `edit.rs` pass the switch to the painter. The naming check in `docs/feature-names.md` replaces the working name Visible Whitespace with Show Whitespace. `crates/ui-text-compare/tests/it/whitespace.rs` checks that a file with all three line-ending styles shows three different end-of-line glyphs, plus the space and tab glyphs, only while the item is on. Checks: `cargo xtask ci` on macOS after rebasing onto main, and pull-request CI on Linux and macOS. https://github.com/alextiboldi/overthediff/pull/111
- **E07-S9-T2** — Word Wrap with aligned continuation rows · size: M · depends: —
  - AC: a 300-character changed line wraps in both panes, and the line after it starts on the same row in both panes.
- **E07-S9-T3** — Side-by-Side and Over-Under layouts · size: M · depends: — · flags: `naming-check` · status: done
  - AC: switching layout keeps the current line in view, and both panes still scroll together.
  - Implemented: `crates/ui-text-compare/src/layout.rs` registers the View menu's radio pair *Side-by-Side* and *Top and Bottom* (`text-compare.side_by_side`, `text-compare.over_under`, Lucide icons `columns-2` and `rows-2`, no default shortcut) through the command registry, and places the two panes beside each other or stacked. Each pane keeps its widget ids in both layouts, so switching keeps its scroll position, caret and focus, and `aligned.rs` scrolls the caret's line back into view once the panes have their new size; in Top and Bottom the Tab order follows the screen. Menu entries can now show a check or radio mark (`Mark` in `crates/core/src/command.rs`, drawn by `crates/ui/src/commands.rs`). The naming check in `docs/feature-names.md` keeps Side-by-Side and replaces the working name Over-Under with Top and Bottom. `crates/ui-text-compare/tests/it/layout.rs` checks that switching keeps the caret line in view and the editor focused, that the stacked panes still scroll together, and the Tab order. Checks: `cargo xtask ci` on macOS, and pull-request CI on Linux and macOS. https://github.com/alextiboldi/overthediff/pull/115
- **E07-S9-T4** — Font size increase, decrease and reset · size: S · depends: — · status: done
  - AC: two increases followed by a reset return the text to the configured size.
  - Implemented: `crates/ui-text-compare/src/font_size.rs` registers the hidden-by-default commands *Increase Font Size*, *Decrease Font Size* and *Reset Font Size* (`text-compare.increase_font_size`, `text-compare.decrease_font_size`, `text-compare.reset_font_size`) through the command registry. They draw both panes' editor text a whole point larger or smaller than the configured size (the theme's monospace size until E22-S3's Editor Font), within limits, and reset returns to that size. The size is the tab's view state. The commands have no menu entry or default shortcut, because egui's window zoom already uses Cmd/Ctrl + plus, minus and 0. `crates/ui-text-compare/tests/it/font_size.rs` checks that two increases followed by a reset return the text to the configured size. The names are checked in `docs/feature-names.md`. Checks: `cargo xtask ci` on macOS after rebasing onto main, and pull-request CI on Linux and macOS. https://github.com/alextiboldi/overthediff/pull/116

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
- **E07-S10-T1** — Format picker per side, listing every registered format with Detected as the default · size: S · depends: — · status: done
  - AC: an empty-mask format appears in the list, and once chosen the file info header names it.
  - Implemented: `crates/ui-text-compare/src/file_format.rs` adds a format picker per side (`left-format`, `right-format`) to the Comparison Settings dialog's new Format tab through E01-S1's session-settings extension point (`otd_ui_text_compare::session_settings`, one line in `apps/otd/src/session_settings.rs`). Each lists Detected (the default, the format resolved by mask) and then every format of the view's registry, the catch-all included, so an empty-mask format can be picked; `TextCompareView::format(side)` is the format in use and the file info header names it. Names checked in `docs/feature-names.md`. Test: `tests/it/file_format.rs` drives the dialog by keyboard, picks an empty-mask format and reads the left header from the accessibility tree. Checks: `cargo xtask ci` on macOS after rebasing onto main, and pull-request CI on Linux and macOS. https://github.com/alextiboldi/overthediff/pull/120
- **E07-S10-T2** — Encoding override per side with None as the default · size: S · depends: — · status: done
  - AC: a Windows-1252 file mis-detected as UTF-8 shows the correct characters after Windows-1252 is chosen.
  - Implemented: `crates/ui-text-compare/src/encoding.rs` adds an encoding override per side (`left-encoding`, `right-encoding`) to the Comparison Settings dialog's Format tab through E01-S1's session-settings extension point (one line in `apps/otd/src/session_settings.rs`), beside the format pickers. Each lists None (the default, detection) and every encoding the decoder has: UTF-8, UTF-16 LE and BE, and the Windows code pages. A side whose override changes is read again in it through `otd_text_compare::open_as` and `decode_as` (`crates/text-compare/src/decode.rs`), and the file info header names it; a side with unsaved edits keeps them and its pane announces, in a polite live region, that the override waits. Swap Sides takes each override with its file. Tests: `tests/it/encoding.rs` drives the dialog by keyboard, picks Windows-1252 for a file detected as UTF-8 and reads the corrected text from the accessibility tree, and checks the waiting notice over unsaved edits; unit tests in `encoding.rs` and `decode.rs`, and the hostile-input sweep covers `decode_as`. Checks: `cargo xtask ci` on macOS after rebasing onto main, and pull-request CI on Linux and macOS. https://github.com/alextiboldi/overthediff/pull/121
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
