# E15 — Table Compare

**Group**: feature
**Label**: `area:table-compare`
**Size**: L
**Depends on**: E01, E02, E03, E04 (foundations); E10 (File Formats manager, for E15-S7)
**Source**: catalogue §14, F-21.3, F-21.6. Spec-derived from the feature catalogue and not yet validated. Items marked *inferred* are analyst or PO inferences.

Table Compare compares tabular data cell by cell in two grids that scroll together: CSV and other delimited or fixed-width text, Excel workbooks and HTML tables. Line-based text comparison misreports reordered rows and shifted columns. Aligning by rows, columns and keys shows the real data changes.

## Behavioral contract
- **Data**:
  - *Table*: one or more sheets. Each sheet is rows of cells holding text values.
  - *Comparison*:
    - comparison sheets, each a left sheet paired with a right sheet;
    - comparison columns, each a left column paired with a right column (comparison column 1 might be left C against right E);
    - aligned row pairs, or a row facing a gap;
    - a per-cell result: same, important difference or unimportant difference.
  - *Table format* (file-format kind Table):
    - masks;
    - Type (Detect, Delimited or Fixed): delimiter characters, text qualifier, the consecutive-delimiter and surrounding-whitespace flags, column widths and First line contains;
    - Regional settings;
    - encoding.
  - *Session settings*:
    - Specs: files, Description and Disable editing.
    - Format: table format and encoding per side.
    - Sheets.
    - Columns: column alignment, plus per-column handling with a session default.
    - Rows: row alignment per sheet or by default.
- **Routes/endpoints** (desktop app: commands, settings and CLI):
  - Home and Session > New Session ▸ Table Compare.
  - The CLI with two files that resolve to a Table format.
  - Session Settings tabs Specs, Format, Sheets, Columns and Rows.
  - Edit, Search and View menu commands.
  - Sheet tabs along the bottom of the view.
- **Integrations**:
  - E02-S1: the file-format registry resolves files to the Table kind.
  - E03-S1: the text comparison engine aligns rows, using the same algorithms as for text lines.
  - E04-S1: reads and writes the files.
  - E01-S1: the shell, the session framework, the Session Settings dialog shell and the unsaved-changes contract.
- **Rules/states**:
  - The grid shows comparison columns, not file columns.
  - With key columns set, rows align only when every key matches. Otherwise the Rows settings decide the alignment.
  - A cell difference is important unless column handling makes it unimportant. A row's gutter spot is red when the row has an important difference and blue when it has only unimportant ones; red wins.
  - Editing is unavailable when the location is read-only, loading failed, the format can't save or Disable editing is on.

---

## Stories

### E15-S1 — Table Compare view
**Priority**: Should · **Layer**: 1 · **Size**: L · **Platforms**: All
**Needs**: E01-S1 (application shell), E02-S1 (file-format registry), E03-S1 (text comparison engine), E04-S1 (file-system access layer)
**Traces**: F-14.1, F-14.6, F-14.7, F-21.6
**Not in this story**: key columns → E15-S2; Rows tab → E15-S8; Columns tab, column handling and unimportant differences → E15-S3; multiple sheets and Excel → E15-S4; editing and Disable editing → E15-S5; search, Go To and view options → E15-S6; Type/Regional tabs and the session Format tab → E15-S7; child-session commands → E06-S11; report → E18-S4; Compare Files Using → E06-S9; Compare Parent Folders → E06-S10; changed-on-disk detection → E07-S22.

As a data analyst, I can compare two CSV exports cell by cell in two synchronised grids so that I see exactly which values changed instead of whole changed lines.

**Acceptance criteria**
- [ ] Built-in Table formats are registered for comma-separated and tab-separated files (masks `*.csv` and `*.tsv;*.tab`, *inferred*). Two such files open in Table Compare whether passed on the CLI or chosen through Home or Session > New Session ▸ Table Compare.
- [ ] Each file is detected automatically as delimited or fixed width, and its first line as column names or cell data.
- [ ] Two grids, left and right, scroll together. Their columns are comparison columns, and in this story they pair up by position (*inferred* default).
- [ ] Rows are aligned with the Standard alignment algorithm. An inserted or deleted row faces a gap on the other side.
- [ ] Each row shows a gutter colour spot, differing cells are highlighted, and important differences are red.
- [ ] Display filters Show All, Show Differences, Show Same and Show None (hidden by default) show or hide rows by status.
- [ ] Next/Previous Difference Row and Next/Previous Difference Cell move between differences.
- [ ] Side-by-Side and Over-Under layouts are available.
- [ ] Session Settings has a Specs tab with the left and right files and a Description.
- [ ] File > Open File loads a file into the chosen pane, and File > Open Clipboard loads clipboard text as a table into the chosen pane.
- [ ] Swap Sides, Reload, Recompare and Table Compare Info all work; Info shows statistics for rows, cells and differences. The status bar shows the overall result.
- [ ] A file that can't be parsed as a table shows an error in its own pane, and the other pane still loads (*inferred*).

#### Tasks
- **E15-S1-T1** — Register built-in delimited Table formats and route two table files to the view · size: S · depends: — · flags: `naming-check`
  - AC: `a.csv b.csv` on the CLI opens Table Compare. Home and New Session list Table Compare.
- **E15-S1-T2** — Parse delimited and fixed-width text, with auto-detection of type and header row · size: M · depends: — · flags: `licence-check`, `craft-libs`
  - AC: fixtures cover comma, tab and fixed-width files, each with and without a header row, and all are detected correctly. A quoted field containing a delimiter stays one cell (text qualifier `"` by default, *inferred*).
- **E15-S1-T3** — Build the comparison: positional column pairing, row alignment and cell results · size: M · depends: E15-S1-T2
  - AC: inserting a row on one side gives one gap row and no other differences. A changed cell is the only highlighted cell in its row.
- **E15-S1-T4** — Two synchronised grids with gutter spots, cell highlights and both layouts · size: L · depends: E15-S1-T3 · flags: `accessibility`
  - AC: scrolling either grid scrolls the other. Switching to Over-Under keeps the rows aligned.
- **E15-S1-T5** — Display filters and difference-row and difference-cell navigation · size: M · depends: E15-S1-T4
  - AC: Show Differences hides rows that are the same. Next Difference Cell visits every differing cell in order and stops at the last one.
- **E15-S1-T6** — Specs tab, Open File, Open Clipboard, Swap Sides, Reload, Recompare, Info and status bar · size: M · depends: E15-S1-T4
  - AC: Swap Sides exchanges the grids and recompares. The Info counts match the grid. Tab-separated clipboard text opens as a table.

### E15-S2 — Key columns for row alignment
**Priority**: Should · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E15-S1 (Table Compare view)
**Traces**: F-14.3
**Not in this story**: Rows tab alignment options → E15-S8; Column Handling dialog → E15-S3; the Sort editing command → E15-S5.

As a data analyst comparing exports whose row order differs, I can align rows by key columns so that the same record lines up on both sides.

**Acceptance criteria**
- [ ] Any combination of comparison columns can be marked as key columns. Marking is done from the column header's context menu, and key columns carry a marker (*inferred* placement; the Column Handling dialog may also offer Key).
- [ ] With key columns set, rows align only when every key matches. With several keys, precedence follows comparison-column order. A row whose key appears on one side only faces a gap.
- [ ] Changing the keys recompares immediately (*inferred*).

#### Tasks
- **E15-S2-T1** — Mark and unmark key columns, with markers on the column headers · size: S · depends: —
  - AC: marking two columns as keys shows a marker on both, and the keys are kept with the session settings.
- **E15-S2-T2** — Key-based row alignment with column-order precedence · size: M · depends: E15-S2-T1
  - AC: two files holding the same records in different orders align record by record. A record present on one side only faces a gap.

### E15-S3 — Column alignment and per-column handling
**Priority**: Should · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E15-S1 (Table Compare view), E15-S2 (key columns and row alignment)
**Traces**: F-14.4, F-14.1 · **Open questions**: Q5
**Not in this story**: key-based row alignment itself → E15-S2 (this story adds the Key field to the dialog); per-format regional number and date settings → E15-S7.

As a data analyst, I can pair columns by name and choose how each column is compared so that renamed or reordered columns, case changes and rounding noise don't hide the real changes.

**Acceptance criteria**
- [ ] The column types settled in T1 (Q5) are the types offered by the Column Handling dialog.
- [ ] The Columns tab has a sheet dropdown, with a single entry for single-sheet files. Each line shows the left column letter with sample data, the right column letter with sample data, and a handling summary.
- [ ] Column alignment offers Unaligned (by position), Align by left name, Align by right name, and Custom. Custom has Move Up and Move Down, and Tidy, which removes blank lines.
- [ ] Double-clicking a line opens the Column Handling dialog with these fields:
  - Use Default;
  - Type;
  - Unimportant (the whole column);
  - Text important, except for Character case and/or Whitespace;
  - Date tolerance;
  - Numeric tolerance.
  Edit Default sets the session's default handling.
- [ ] Numbers within the Numeric tolerance, and date-times within the Date tolerance, are unimportant differences. Values are parsed using the system's regional settings (*inferred*).
- [ ] Unimportant differences show in blue, both as a cell highlight and as a gutter spot. Red wins when the row also has an important difference.
- [ ] View > Ignore Unimportant Differences treats unimportant differences as same: they get no highlight, and Show Differences hides them.

- [ ] The Column Handling dialog includes **Key**. Ticking it makes the column a key, exactly as key marking in the grid does, and the Columns tab summary shows it.
#### Tasks
- **E15-S3-T1** — Decide the column types (Q5) · size: S · depends: — · flags: `decision`
  - AC: decision recorded. Proposed default: General (auto-detect), Text, Number and Date/time, with Currency parsed as Number. General detects numbers and dates cell by cell.
- **E15-S3-T2** — Columns tab with sheet dropdown, sample data and handling summary · size: M · depends: —
  - AC: each comparison column appears once, with both letters and the first non-empty sample values.
- **E15-S3-T3** — Column alignment modes, including Custom with Move Up/Down and Tidy · size: M · depends: E15-S3-T2
  - AC: Align by left name pairs `Price` with `Price` when the right file has it in another position. Tidy removes blank lines from a custom list.
- **E15-S3-T4** — Column Handling dialog and Edit Default · size: M · depends: E15-S3-T1, E15-S3-T2
  - AC: a column with Use Default follows the edited default. Overriding one field keeps the other fields from the default.
- **E15-S3-T5** — Unimportant classification: whole column, case, whitespace, and numeric and date tolerances · size: M · depends: E15-S3-T4
  - AC: with Numeric tolerance 0.01, `1.004` vs `1.0` is unimportant and `1.02` vs `1.0` is important. With Character case unimportant, `ABC` vs `abc` is unimportant.
- **E15-S3-T6** — Blue rendering and Ignore Unimportant Differences · size: S · depends: E15-S3-T5 · flags: `accessibility`
  - AC: a row with only unimportant differences shows a blue spot. With Ignore Unimportant Differences on, Show Differences hides that row.
- **E15-S3-T7** — Key field in the Column Handling dialog · size: S · depends: E15-S3-T4
  - AC: ticking Key on a column re-aligns the rows by that key. Unticking it restores the previous alignment. Key marking in the grid and in the dialog always agree.

### E15-S4 — Multiple sheets: Excel workbooks and HTML tables
**Priority**: Should · **Layer**: 2 · **Size**: L · **Platforms**: All
**Needs**: E15-S1 (Table Compare view)
**Traces**: F-14.2, F-21.3
**Not in this story**: Word and PDF converters → E10-S7; external conversion programs → E10-S6.

As a data analyst comparing two versions of a workbook, I can compare every sheet of two Excel files, or every table of two HTML pages, and step between sheets so that I review the whole workbook in one session.

**Acceptance criteria**
- [ ] Built-in Excel → table conversion opens Excel workbooks in Table Compare (masks `*.xlsx;*.xls`, *inferred*). Cell values are compared; formatting and embedded images are not.
- [ ] The built-in conversion is load-only: edits can't be saved back to a workbook (*inferred*).
- [ ] An "HTML tables" format turns each table in an HTML file into a sheet. It ships built in rather than as a separate add-on (PO decision, *inferred*).
- [ ] A file with several sheets shows tabs along the bottom, one per comparison sheet. Each tab has a status spot: red when the sheet has important differences, blue when it has only unimportant ones, and none when it is the same (*inferred*).
- [ ] The Sheets tab pairs a left sheet, given by number and name, with a right sheet on each line. Sheet alignment offers:
  - Unaligned (file order);
  - Align by left name;
  - Align by right name;
  - Custom, with Move Up, Move Down and Tidy (which removes blank lines).
- [ ] Opening the same file on both sides and using custom sheet alignment compares two sheets of one file.
- [ ] Next/Previous Difference Sheet moves to the next or previous sheet with differences.

#### Tasks
- **E15-S4-T1** — Excel workbook reader producing one table per sheet · size: L · depends: — · flags: `licence-check`, `craft-libs`
  - AC: a workbook with three sheets yields three tables with their sheet numbers and names. Formula cells compare by their stored values (*inferred*).
- **E15-S4-T2** — HTML tables format producing one table per HTML table · size: M · depends: — · flags: `licence-check`, `craft-libs`
  - AC: an HTML file with two tables yields two sheets, in document order.
- **E15-S4-T3** — Sheet tabs with status spots · size: M · depends: E15-S4-T1 · flags: `accessibility`
  - AC: selecting a tab shows that comparison sheet, and each spot matches its sheet's status.
- **E15-S4-T4** — Sheets tab pairing: Unaligned, Align by left/right name, and Custom with Move Up/Down and Tidy · size: M · depends: E15-S4-T3
  - AC: Align by left name pairs `Q1` with `Q1` when the right workbook orders its sheets differently. Custom pairs the `Jan` and `Feb` sheets of the same file.
- **E15-S4-T5** — Next/Previous Difference Sheet · size: S · depends: E15-S4-T3
  - AC: starting from the first sheet, Next Difference Sheet skips sheets that are the same.

### E15-S5 — Edit and save tables
**Priority**: Should · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E15-S1 (Table Compare view)
**Traces**: F-14.5, F-6.6
**Not in this story**: profile-based save variants (not yet placed for table views); backups → E09-S10.

As a data analyst reconciling two data files, I can edit cells, rows and columns and copy them across so that I can make one file match the other and save it.

**Acceptance criteria**
- [ ] Modify toggles edit mode on the current cell, and typing changes its value. The comparison updates after the edit (*inferred*: live recompare, as in Text Compare).
- [ ] Copy to Right, Copy to Left and Copy to Other Side copy the selected rows. Copy Cell to Right, Copy Cell to Left and Copy Cell to Other Side copy the selected cells.
- [ ] Cut, Copy, Paste and Delete work on the selection, and Select All selects every cell. Delete Row and Delete Column remove structure, and Insert Row Before/After and Insert Column Before/After add it. Sort sorts the selected data.
- [ ] Align With works by picking a row and then clicking a row on the other side, which aligns the two rows.
- [ ] Undo and Redo cover every edit above.
- [ ] Save and Save As write the table back in its format, keeping its delimiters, text qualifier and encoding. Unsaved changes prompt on close.
- [ ] Disable editing on the Specs tab makes both sides read-only. Editing is also unavailable when the location is read-only or loading failed.

#### Tasks
- **E15-S5-T1** — Cell editing (Modify), Cut/Copy/Paste/Delete and Select All, with live recompare · size: M · depends: —
  - AC: editing a differing cell to match the other side turns both the cell and its row to same.
- **E15-S5-T2** — Structural edits: Delete Row/Column, Insert Row/Column Before/After, and Sort · size: M · depends: E15-S5-T1
  - AC: inserting a column shifts the comparison columns as expected. Sort on a selected range sorts only that range.
- **E15-S5-T3** — Copy rows and cells to the other side · size: S · depends: E15-S5-T1
  - AC: Copy Cell to Other Side on a differing cell makes it same. Copy to Right on a row that faces a gap inserts that row on the right.
- **E15-S5-T4** — Align With for rows · size: S · depends: —
  - AC: two forced rows stay aligned until a recompare resets the alignment (*inferred*).
- **E15-S5-T5** — Undo/Redo across all table edits · size: S · depends: E15-S5-T1, E15-S5-T2, E15-S5-T3
  - AC: undoing a column insert restores the previous grid and the previous comparison.
- **E15-S5-T6** — Save and Save As in the table's own format, plus Disable editing and the read-only conditions · size: M · depends: E15-S5-T1
  - AC: saving a CSV that holds a quoted field with a comma writes that field quoted again. With Disable editing on, every edit command is disabled.

### E15-S6 — Table search, navigation and view options
**Priority**: Should · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E15-S1 (Table Compare view), E02-S1 (masks and regular expressions), E15-S5 (table editing and saving)
**Traces**: F-14.6, F-14.1
**Not in this story**: Next/Previous Difference Sheet → E15-S4; Next/Previous Difference Files and Copy File and Open Next Difference → E06-S11.

As a QA engineer reviewing a large data set, I can search, jump to a row and column, and tune what the grid shows so that I find and read the values I care about quickly.

**Acceptance criteria**
- [ ] Find searches cell text on the chosen sides, as plain text or a regular expression, with case and whole-word options (*inferred*: the same options as Text Compare's find). Find Next and Find Previous step through the matches.
- [ ] Replace replaces the matched cell text. A replaced cell counts as an unsaved change (*inferred*).
- [ ] Next/Previous Edit moves between the cells changed in this view since loading.
- [ ] Go To moves to a given row and column.
- [ ] Hide Same Columns hides comparison columns that have no differences, and Resize Columns to Fit sizes columns to their contents.
- [ ] Row Numbers shows file row numbers, and Visible Whitespace shows the spaces and tabs inside cells.
- [ ] Display font size can be increased, decreased or reset (hidden by default).
- [ ] The Thumbnail shows one colour row per comparison row and marks the viewport; clicking it jumps there (*inferred*: as in Text Compare).
- [ ] Text Details shows the current row of both files at full width. File Info shows each file's path, size, modified time and format.

#### Tasks
- **E15-S6-T1** — Find, Find Next and Find Previous over cells, with regular-expression support · size: M · depends: —
  - AC: a regex match in a cell selects that cell. Searching past the last match reports that there are no more matches.
- **E15-S6-T2** — Replace and Next/Previous Edit · size: S · depends: E15-S6-T1
  - AC: Replace changes one cell's text and marks the view as having unsaved changes, and Next Edit then lands on that cell.
- **E15-S6-T3** — Go To row and column · size: S · depends: —
  - AC: Go To row 500, column 3 scrolls both grids to that cell.
- **E15-S6-T4** — Column display: Hide Same Columns, Resize Columns to Fit, Row Numbers, Visible Whitespace and font size · size: M · depends: —
  - AC: Hide Same Columns hides exactly the columns without differences, and shows them again when toggled off.
- **E15-S6-T5** — Thumbnail, Text Details and File Info · size: M · depends: — · flags: `naming-check`
  - AC: clicking near the end of the thumbnail jumps near the last row. Text Details shows the full text of a long cell.

### E15-S7 — Table format definitions
**Priority**: Should · **Layer**: 4 · **Size**: M · **Platforms**: All
**Needs**: E15-S1 (Table Compare view), E10-S1 (File Formats manager), E15-S3 (column types that parse numbers and dates)
**Traces**: F-21.6, F-14.7

As a data analyst working with semicolon-separated exports and European number formats, I can define how a table format is parsed and pick a format and encoding per side so that unusual files load as proper tables.

**Acceptance criteria**
- [ ] Table formats in the File Formats manager have a Type tab with three choices:
  - Detect: works out automatically whether the file is delimited or fixed width.
  - Delimited: delimiter characters (tab, comma, space and others), a Text qualifier (for example `"`), Treat consecutive delimiters as one, and Treat surrounding whitespace as part of delimiter (on by default).
  - Fixed: column widths as a semicolon-separated list.
- [ ] First line contains takes cell data or column names. It is auto-detected by default and can be overridden.
- [ ] The Regional tab offers Use system, or an explicit Decimal separator, Thousands separator, Date order and Date separator. Numbers and dates in cells are parsed with these settings.
- [ ] Under the General column type, values that parse as the same number or date compare as equal (*inferred*).
- [ ] A user can add a custom Table format, for example `*.txt` exports with `;` delimiters. Matching files then open in Table Compare with that format.
- [ ] The Session Settings Format tab sets a table format per side ("Detected" by default) and an encoding per side.

#### Tasks
- **E15-S7-T1** — Type tab: Detect, Delimited and Fixed settings · size: M · depends: —
  - AC: a `;`-delimited file with `"`-qualified fields parses correctly with a custom format. Fixed widths `5;10;3` split the columns at those widths.
- **E15-S7-T2** — First line contains: auto, cell data or column names · size: S · depends: E15-S7-T1
  - AC: forcing "cell data" makes the header row a data row in the comparison.
- **E15-S7-T3** — Regional tab and number and date parsing · size: M · depends: —
  - AC: with decimal separator `,`, the cells `1,5` and `1,50` compare as the same number (*inferred* numeric comparison).
- **E15-S7-T4** — Session Format tab: table format and encoding per side · size: S · depends: —
  - AC: choosing another table format for the right side reloads it with that format's settings. An encoding override fixes a mis-detected legacy file.

### E15-S8 — Row alignment options (Rows tab)
**Priority**: Should · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E15-S1 (Table Compare view), E15-S2 (key columns), E07-S13 (text alignment algorithms, reused for rows)
**Traces**: F-14.3

As a data analyst comparing tables without key columns, I can choose how rows are aligned so that inserted and reordered rows line up the way I expect.

**Acceptance criteria**
- [ ] With no key columns, the Rows tab decides row alignment. With key columns set, the Rows settings don't apply.
- [ ] Settings apply to the current comparison sheet, or follow the session default when *Use default* is ticked. *Edit Default* sets that session default. A single-sheet file has one entry.
- [ ] The Rows tab offers the algorithms Unaligned, Standard, Myers O(ND) and Patience, plus *Never align differences*, *Skew tolerance* (rows), *Use closeness matching* and *Sort rows before alignment*. The algorithms are the same ones Text Compare uses, applied to rows.
- [ ] Unaligned compares row N with row N. *Sort rows before alignment* sorts both sides by all columns, left to right, before aligning (*inferred* sort key).
- [ ] Changing a Rows setting recompares immediately (*inferred*).

#### Tasks
- **E15-S8-T1** — Rows tab with per-sheet settings, Use default and Edit Default · size: S · depends: —
  - AC: a sheet with Use default ticked follows an edited default. Unticked, it keeps its own settings.
- **E15-S8-T2** — Row alignment through the text alignment algorithms: Unaligned, Standard, Myers O(ND), Patience, Never align differences, Skew tolerance and closeness matching · size: M · depends: E15-S8-T1 · flags: `naming-check`
  - AC: on a fixture with a large inserted block, each algorithm aligns the rows as it aligns lines in Text Compare. With Never align differences, changed rows show as delete-plus-add blocks.
- **E15-S8-T3** — Sort rows before alignment · size: S · depends: E15-S8-T1
  - AC: two files with the same rows in different orders compare as same with sorting on, and as different with it off. With a key column set, the Rows settings have no effect.
