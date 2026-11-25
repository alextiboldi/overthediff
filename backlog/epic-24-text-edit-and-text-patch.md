# E24 — Text Edit and Text Patch

**Group**: feature
**Label**: `area:text-edit`
**Size**: L
**Depends on**: E02, E03 (foundations); E07 (Text Compare components); E08 (Open With in folder views); E10 (grammars)
**Source**: catalogue §20, F-25.2 and F-25.3. Spec-derived from the feature catalogue and not yet validated. Items marked *inferred* are analyst or PO inferences.

Two views reuse the text components outside a two-way comparison. Text Edit is a single-pane editor with the product's file formats and search tools. Text Patch turns a `.diff` or `.patch` file back into a readable comparison and can apply it.

## Behavioral contract
- **Data**:
  - *Text Edit document*: one file, with its format (grammar), encoding and line endings.
  - *Patch document*: one or more file entries, each with an old name, a new name and hunks in unified, context or normal diff format. Each entry is recreated as a left and a right line sequence: context lines on both sides, removed lines on the left, added lines on the right.
  - *Find in Files query*: text, folder, file mask, case, whole words, regular expression and subfolders.
- **Routes/endpoints** (desktop app, so these are commands and CLI forms):
  - *Tools > Edit Text File* and *Tools > View Patch*.
  - CLI `/edit <file>`, and passing a `<file>.diff` or `<file>.patch`.
  - *Open With > Text Edit* and *Open With > Text Patch*.
  - *File > Apply Patch* in Text Patch, and *Search > Find in Files* in Text Edit.
- **Integrations**:
  - E07 provides the text panes, editing, search and view options.
  - E02-S1 provides masks and regular expressions.
  - E10 grammars drive syntax highlighting.
  - E03-S1 aligns the recreated sides.
  - Text Patch reads standard patch files, whether another diff tool wrote them or the text report's patch layout did.
- **Rules/states**:
  - Text Patch is read-only, and lines outside the patch's hunks aren't shown (*inferred*).
  - A multi-file patch is browsed one file pair at a time.
  - Target resolution, dry run and reject handling for Apply Patch follow the Q23 decision.

---

## Stories

### E24-S1 — Text Edit view
**Priority**: Could · **Layer**: 3 · **Size**: L · **Platforms**: All
**Needs**: E07-S5 (find and replace), E07-S6 (go to and bookmarks), E07-S8 (editing commands and conversions), E07-S9 (whitespace, word wrap, layout and font size), E10-S2 (syntax highlighting)
**Traces**: F-20.1, F-25.3
**Not in this story**: Find in Files → E24-S3; Open With entry → E24-S5; Webpages panel → E07-S21.

As a developer, I can open a single file in the product's own editor so that I can fix it with the same formats, search and editing tools I use when comparing.

**Acceptance criteria**
- [ ] *Tools > Edit Text File* and the CLI switch `/edit <file>` open a single-pane Text Edit view on a file.
- [ ] The view applies the file's text format: syntax highlighting from its grammar, toggled by *View > Syntax Highlighting*. It also honours the Text Editing options *Auto indent* and *Backspace unindents*.
- [ ] File menu: *Open File* and *Open File on File System* load another file; *Save File*, *Save File As* and *Save File on File System As* write it. Closing with unsaved edits prompts first.
- [ ] Edit menu:
  - *Undo/Redo*, *Increase/Decrease Indent*, *Cut/Copy/Paste/Delete* and *Select All*;
  - the hidden-by-default line and word commands: Delete Line, Delete to Start/End of Line, Delete Word, Delete to Start/End of Word, Insert Line Before/After;
  - *Convert File*: Trim Trailing Whitespace, Leading Spaces to Tabs, Tabs to Spaces, Line Endings.
- [ ] Search menu: *Next/Previous Edit*, *Find*, *Replace*, *Find Next/Previous*, *Go To* (line and column) and *Bookmarks 0–9*.
- [ ] View menu: *Visible Whitespace*, *Line Numbers*, *Syntax Highlighting*, *Word Wrap*, display font size, *File Info* and *Toolbar*.

#### Tasks
- **E24-S1-T1** — Single-pane Text Edit view with file info, open and save variants, and unsaved-changes prompts · size: M · depends: — · flags: `naming-check`
  - AC: editing and saving a UTF-8 file with CRLF line endings keeps its encoding and line endings.
- **E24-S1-T2** — Entry points: Tools > Edit Text File and `/edit` · size: S · depends: E24-S1-T1
  - AC: `/edit notes.txt` opens Text Edit on that file in the running instance.
- **E24-S1-T3** — Editing commands, Text Editing options and Convert File in the single pane · size: M · depends: E24-S1-T1
  - AC: Tabs to Spaces converts every tab in the file. With Auto indent on, Enter keeps the current indentation.
- **E24-S1-T4** — Next/Previous Edit, Find, Replace, Go To and Bookmarks in the single pane · size: M · depends: E24-S1-T1
  - AC: after edits on three lines, Next Edit visits each of those lines. Replacing all matches changes every occurrence.
- **E24-S1-T5** — View options and syntax highlighting by file format · size: S · depends: E24-S1-T1
  - AC: in a `.py` file, keywords are highlighted. With Word Wrap on, long lines wrap to the pane width.

### E24-S2 — Text Patch view
**Priority**: Could · **Layer**: 3 · **Size**: L · **Platforms**: All
**Needs**: E07-S1 (Text Compare view), E03-S1 (text comparison engine), E07-S2 (difference navigation), E07-S5 (find), E07-S6 (go to and bookmarks), E07-S7 (Select Section), E07-S9 (whitespace, wrap, layout), E07-S11 (thumbnail), E07-S12 (details panels), E10-S2 (syntax highlighting)
**Traces**: F-20.2, F-25.2
**Not in this story**: Apply Patch → E24-S4.

As a code reviewer, I can open a `.diff` or `.patch` file as a side-by-side comparison so that I can read a change set as easily as a live comparison.

**Acceptance criteria**
- [ ] *Tools > View Patch* opens a Text Patch view, and so does passing a `.diff` or `.patch` file on the command line.
- [ ] Unified, context and normal diff formats are all recognised.
- [ ] Each file in the patch is shown like a Text Compare: removed lines on the left, added lines on the right, context lines on both sides, in the usual difference colours.
- [ ] The view is read-only.
- [ ] *Next/Previous Difference Files* moves between the file pairs of a multi-file patch (*inferred*).
- [ ] *Next/Previous Difference* and *Next/Previous Difference Section* move within the current file pair.
- [ ] Edit menu: *Copy*, *Select All* and *Select Section*. Search menu: *Find*, *Go To* and *Bookmarks*.
- [ ] View menu:
  - *Visible Whitespace*, *Line Numbers* and *Word Wrap*;
  - *Syntax Highlighting*, chosen by the patched file's name;
  - fonts, and the Side-by-Side and Over-Under layouts;
  - *Thumbnail*, *Text Details*, *Hex Details*, *Alignment Details*, *Ruler* and *Toolbar*.
- [ ] A malformed patch shows an error that names the first line that couldn't be read (*inferred*).

#### Tasks
- **E24-S2-T1** — Patch parser for unified, context and normal formats, with one file or many · size: M · depends: — · flags: `licence-check`, `craft-libs`
  - AC: fixtures from two different diff tools, in all three formats, parse into the same file pairs and hunks.
- **E24-S2-T2** — Recreate each file pair as aligned left and right sides · size: M · depends: E24-S2-T1
  - AC: a hunk that replaces two lines shows them as one changed section, with the context lines aligned on both sides.
- **E24-S2-T3** — Read-only Text Patch view, opened from Tools > View Patch or from `.diff` and `.patch` files on the CLI · size: M · depends: E24-S2-T2 · flags: `naming-check`
  - AC: passing `change.patch` on the command line opens the view on the patch's first file pair. Typing in the view is refused.
- **E24-S2-T4** — Navigation across differences, sections and file pairs; Copy, Select All, Select Section, Find, Go To and Bookmarks · size: M · depends: E24-S2-T3
  - AC: in a three-file patch, Next Difference Files visits each file pair once.
- **E24-S2-T5** — View options in the patch view · size: M · depends: E24-S2-T3
  - AC: the Over-Under layout and Word Wrap both work. Syntax highlighting follows the patched file's extension.

### E24-S3 — Find in Files
**Priority**: Could · **Layer**: 4 · **Size**: M · **Platforms**: All
**Needs**: E24-S1 (Text Edit view), E02-S1 (masks and regular expressions)
**Traces**: F-20.1

As a developer, I can search a folder tree for text from the editor so that I can find every file I need to change.

**Acceptance criteria**
- [ ] *Search > Find in Files* in Text Edit asks for Text to find, Folder, File mask, Match character case, Whole words only, Regular expression and Search subfolders.
- [ ] The results list every matching line, with its file and line number (*inferred* layout).
- [ ] Opening a result opens that file in Text Edit at the matching line.
- [ ] File masks use the product's mask syntax. An invalid regular expression shows an error before any searching starts.

#### Tasks
- **E24-S3-T1** — Search a folder tree with the mask, case, whole-word, regex and subfolder options · size: M · depends: —
  - AC: on a fixture tree, every combination of options returns exactly the expected matches.
- **E24-S3-T2** — Find in Files dialog, with a result list that opens Text Edit · size: S · depends: E24-S3-T1 · flags: `naming-check`
  - AC: double-clicking a result opens Text Edit with the cursor on the matching line.

### E24-S4 — Apply patches
**Priority**: Could · **Layer**: 4 · **Size**: M · **Platforms**: All
**Needs**: E24-S2 (Text Patch view)
**Traces**: F-20.2 · **Open questions**: Q23

As a developer, I can apply the patch I'm viewing to the files it describes so that I can take a change set without leaving the tool.

**Acceptance criteria**
- [ ] *File > Apply Patch* applies the open patch, following the T1 decision.
- [ ] Before anything changes, the user sees which files will be modified (a dry run, proposed pending T1).
- [ ] Hunks that don't apply are reported. The files they belong to are handled as T1 decides: left unchanged or partly patched.
- [ ] After a run, a summary lists the files that were patched and the files that were rejected.

#### Tasks
- **E24-S4-T1** — Decide target resolution, dry run and reject handling (Q23) · size: S · depends: — · flags: `decision`
  - AC: decision recorded in this story. Proposed default:
    - targets resolve relative to a chosen base folder, with a count of leading path components to strip, like `patch -p`;
    - a dry-run preview is always shown first;
    - a file with any failed hunk is left untouched and listed in a rejects report.
- **E24-S4-T2** — Apply engine with hunk matching and rejects · size: M · depends: E24-S4-T1 · flags: `craft-libs`
  - AC: a patch with one good and one stale hunk for the same file reports the stale hunk and leaves that file unchanged, as the proposed default requires.
- **E24-S4-T3** — Apply Patch command with preview and result summary · size: S · depends: E24-S4-T2
  - AC: the preview lists the target files. After applying, the summary lists the patched files and the rejected ones.

### E24-S5 — Open With Text Edit and Text Patch
**Priority**: Could · **Layer**: 4 · **Size**: S · **Platforms**: All
**Needs**: E24-S1 (Text Edit view), E24-S2 (Text Patch view), E08-S16 (Open With in folder views), E15-S1 (Table Compare view)
**Traces**: F-8.15, F-20.1, F-20.2

As a developer browsing a comparison, I can open any file in Text Edit, or read it as a patch, from the Open With menu so that I can edit or review it without leaving the product.

**Acceptance criteria**
- [ ] In Folder Compare, *Open With > Text Edit* opens the selected file in Text Edit, and *Open With > Text Patch* opens it in Text Patch.
- [ ] In Text Compare, *Open With* offers Text Edit and Text Patch for the current file.
- [ ] In Table Compare, Open With offers Text Edit but not Text Patch.
- [ ] In Text Edit, *File > Open With* offers *Associated Application* and *Text Patch*.

#### Tasks
- **E24-S5-T1** — Open With > Text Edit and Open With > Text Patch in Folder Compare · size: S · depends: —
  - AC: choosing Text Patch on a `.diff` file opens the Text Patch view. Choosing Text Edit on any text file opens the editor.
- **E24-S5-T2** — Open With entries in Text Compare, Table Compare and Text Edit · size: S · depends: —
  - AC: Table Compare's Open With has no Text Patch entry. In Text Edit, Open With > Associated Application opens the file in the OS default application.

### E24-S6 — Text Edit extras: tab stops, profiles, shell menu and Webpages panel
**Priority**: Could · **Layer**: 4 · **Size**: S · **Platforms**: All
**Needs**: E24-S1 (Text Edit view), E10-S4 (text format tab behaviour), E21-S6 (browse using profiles), E26-S5 (native shell menu), E07-S21 (Webpages panel)
**Traces**: F-20.1

As a user editing a single file, I can count on the same tab handling, remote open and save, shell menu and web preview in Text Edit that the compare views have, so that I don't need another editor for these tasks.

**Acceptance criteria**
- [ ] Text Edit follows the file format's *Tab stop* and *Insert spaces instead of tabs*: the Tab key and the display behave as in Text Compare.
- [ ] The hidden-by-default *Open File Using Profile* and *Save File Using Profile As* open and save through any named profile.
- [ ] On Windows, *Explorer* shows the native shell context menu for the current file.
- [ ] On Windows and macOS, the *Webpages* panel renders the current file as a web page.

#### Tasks
- **E24-S6-T1** — Tab stop and Insert spaces instead of tabs in Text Edit · size: S · depends: —
  - AC: with a format that inserts spaces and has a tab stop of 4, pressing Tab inserts four spaces and existing tabs display four columns wide.
- **E24-S6-T2** — Open File Using Profile and Save File Using Profile As in Text Edit · size: S · depends: —
  - AC: a file opened through an FTP profile can be edited and saved back through the same profile.
- **E24-S6-T3** — Explorer shell menu and Webpages panel in Text Edit · size: S · depends: — · flags: `licence-check`, `naming-check`
  - AC: on Windows, *Explorer* shows the shell menu for the open file. On Windows and macOS, the *Webpages* panel shows the rendered HTML file and updates after a save.
