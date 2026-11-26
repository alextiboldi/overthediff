# E08 — Folder Compare

**Group**: feature
**Label**: `area:folder-compare`
**Size**: L
**Depends on**: E01, E02, E03, E04 (foundations); E07 (Text Compare, for file pairs and rules-based content)
**Source**: catalogue §8, F-6.1, F-6.4, F-25.2–F-25.4, Appendix A and B. Spec-derived from the feature catalogue and not yet validated. Items marked *inferred* are analyst or PO inferences.

Folder Compare shows two folder trees side by side. It aligns their items, colours each by status, and lets the user filter, select, navigate and drill into file pairs. Either side can be any data source (local, network, archive, remote or snapshot), and the view is the hub from which file comparisons and folder operations start.

## Behavioral contract
- **Data**:
  - *Folder session*: left and right base folders, description, rules (Comparison, Handling, Name Filters, Other Filters and Misc tabs), display filter, folder display mode, and view state (expanded folders, visible columns, sort).
  - *Comparison row*: a left item and a right item. Either can be absent, which makes the row an orphan. Each item carries name, extension, size, modified time, attributes or Unix mode, owner, group, CRC (when known), link target and error.
  - *Row status*: unknown, same, different, left or right newer or older, orphan, error, or filtered-but-shown (teal).
  - *Content result*, shown in the center column: binary same, binary differences, rules-based same, unimportant differences or important differences.
  - *Folder aggregate cues* summarise a folder's contents.
  - *Filters*: name masks (include and exclude, for files and folders), other-filter items (date, size, content, Unix file type, DOS attribute) and the "Exclude protected operating system files" flag.
  - *Alignment override rules*: left pattern, right pattern, regex flag and folder limit.
  - *Base-folder history* (Back/Forward) and *marked items* (Space) are per view.
- **Routes/endpoints** (desktop app, so these are commands, tabs and switches):
  - Folder Compare's Session, Actions, Edit, Search and View menus, toolbar and context menu.
  - Session Settings tabs Specs, Comparison, Handling, Name Filters, Other Filters and Misc.
  - CLI `otd <folder> <folder>` and `/qc` (rules-based).
  - Options: Startup (Quick Compare), Tweaks (Name Filter Presets), File Operations (Include hidden items by default).
- **Integrations**:
  - E03-S2 folder engine: scan, align, quick and content tests, aggregation, flattening, the visibility hook and selection resolution.
  - E04 data sources supply the items.
  - E02-S1 masks and regular expressions.
  - E07 Text Compare: file pairs open in it, and rules-based content tests use its rules.
  - E01-S1 shell: commands, tabs, Session Settings, path edits and background tasks.
- **Rules/states**:
  - Items align by name within each folder. Case sensitivity follows the data source (*inferred*; for FTP it is a profile setting).
  - Colours follow Appendix B: gray unknown or older, black same, purple orphan, red newer or different, teal filtered-but-shown, yellow folder not yet scanned, hollow folder with its scan pending.
  - A folder's status aggregates its contents, and the folder's own timestamp is ignored.
  - File filters remove items, display filters hide rows by status, and folder display modes decide which folders show. When a filter hides only one side of a pair, both sides stay visible and the hidden side is teal.
  - Selecting a folder resolves to its visible contents for every operation.
  - Child file sessions report their result back to the parent row.

---

## Stories

### E08-S1 — Folder Compare view
**Priority**: Must · **Layer**: 1 · **Size**: L · **Platforms**: All
**Needs**: E01-S1 (application shell), E03-S2 (folder comparison engine), E04-S1 (file-system access layer)
**Traces**: F-8.1, F-8.2, F-8.4, F-8.14, F-6.1, F-25.2, F-7.1, F-9.10, App-B
**Not in this story**: display filters → E08-S2; name filters → E08-S3; Comparison tab → E08-S4; content compare and center column → E08-S5; more columns and sorting → E08-S6; selection commands → E08-S7; base-folder navigation → E08-S8; next difference and find → E08-S9; opening pairs → E08-S10; Handling tab and scan cues → E08-S11; file operations → E09; colours/fonts/stripes → E22-S2; Create for a missing base folder → E09-S9.

As a developer, I can open two folders side by side and see at a glance which items are the same, different, newer or missing so that I know where two trees diverge.

**Acceptance criteria**
- [x] A Folder Compare opens from the Home button, from *Session > New Session ▸ Folder Compare*, or from the CLI with two folders (`otd <folder> <folder>`). The view has a left and a right pane, each with a path edit above it (typed path, Browse for Folder, history).
- [x] Items align by name within each folder. Partners share a row. An item with no partner is an orphan, with a gap on the other side.
- [x] *Open Folder* / *Close Folder*, *Open Subfolders* / *Close Subfolders* (recursive), *Expand All* and *Collapse All* work on either side.
- [x] Name, Size and Modified columns show for both sides.
- [x] Quick tests compare size and modified time within the default tolerance decided in T1.
- [ ] Rows use the default colours: gray = unknown or older, black = same, purple = orphan, red = newer or different. (Built and tested; the colours in the running app, in light and dark themes, await E08-S1-T4's check by hand.)
- [ ] A folder's colour aggregates its contents. It is "newer" when it contains newer files, its own timestamp is ignored, and it can show several cues at once, such as red for a newer file inside plus purple for an orphan inside. (Built and tested; an opened parent folder's cues in the running app await E08-S1-T4's check by hand.)
- [ ] A colour spot left of each name shows the status even when the selection hides the row colour. (Built and tested; the spot in the running app awaits E08-S1-T4's check by hand.)
- [ ] Symbolic links and junctions carry a small arrow glyph. A folder that couldn't be opened carries a small red "X". (Built and tested; the glyphs in the running app, and a junction on Windows, await E08-S1-T4's check by hand.)
- [ ] Click selects an item on either side, Ctrl+click adds to the selection, and Shift+click selects a range. (Built and tested; Cmd+click on macOS and Ctrl+click on Windows and Linux in the running app await E08-S1-T5's check by hand.)
- [ ] *Refresh* re-scans open folders, *Full Refresh* redoes the whole comparison, and *Refresh Selection* re-scans the selected items. (Refresh and Full Refresh are built and tested; Refresh Selection still acts on the focused row, not the selection: issue #100.)
- [x] *Swap Sides* exchanges the left and right base folders and recompares.
- [x] *Session > Folder Compare Info* shows comparison statistics.
- [ ] *View > Legend* explains every colour and glyph of the folder palette, including the ones that appear only once other features are in use: teal (filtered but shown), yellow folder (not scanned) and hollow folder (scan pending). (Built and tested; the swatches in the running app, in light and dark themes, await E08-S1-T7's check by hand.)
- [ ] The status bar shows scan progress and the overall result. (Built and tested; the Scan field during a large scan in the running app awaits E08-S1-T7's check by hand.)

#### Decisions
**Default timestamp tolerance (E08-S1-T1).** Recorded by E08-S1-T1 and confirmed by the operator on 2026-11-25, as proposed. The catalogue gives the tolerance in seconds but sets no default, so the value is *unspecified* there and is OverTheDiff's own choice.

- The default is **2 seconds**. FAT volumes and zip archives keep modified times to 2 seconds, so a file copied to or from one can move by up to 2 seconds and still compares same.
- Times exactly 2 seconds apart count as same; only a gap greater than the tolerance makes the later file newer (E03-S2-T4's exactly-at-tolerance cases).
- A new Folder Compare session's quick tests use it until the user changes the tolerance on the Comparison tab (E08-S4). Daylight-saving and time-zone offsets stay with E08-S4.
- The value lives in code as `otd_folder_engine::DEFAULT_TOLERANCE` (`crates/folder-engine/src/status.rs`).

#### Tasks
- **E08-S1-T1** — Decide the default timestamp tolerance · size: S · depends: — · flags: `decision` · status: done
  - AC: decision recorded in this story. Proposed default: 2 seconds, which absorbs the 2-second timestamp resolution of FAT volumes and zip archives. Quick tests use it until the user changes it.
  - Implemented: 2-second default recorded in this story's Decisions and in `00-index.md`, confirmed by the operator on 2026-11-25; the value lives as `otd_folder_engine::DEFAULT_TOLERANCE` in `crates/folder-engine/src/status.rs`. No new test: the exactly-at-tolerance boundary is already covered by E03-S2-T4's tests in `crates/folder-engine/tests/it/status.rs`. `cargo xtask ci` passed on macOS, and the pull request's CI passed on Linux and macOS (no platform-dependent code, so no full-matrix run). https://github.com/alextiboldi/overthediff/pull/65
- **E08-S1-T2** — Two-pane view with aligned rows, path edits and the entry points (Home, New Session, CLI with two folders) · size: M · depends: — · flags: `naming-check`, `craft-libs`, `accessibility` · status: needs-verification
  - AC: `otd A B` with two folders opens the view with A and B in the path edits. Rows of a fixture tree align by name, and orphans show gaps.
  - Implemented: `otd-ui-folder-compare` (`crates/ui-folder-compare`) registers the `folder-compare` session type (Home and New Session list it) and the `FolderCompare` view: two path edits bound to the session's specs, the base folders listed through folder-engine `TreeScan` over `LocalSource` and aligned by `compare_trees` with the 2-second tolerance, gap cells for orphans, a `COLUMNS` table (Size, Modified) and an accessible tree grid with arrow-key focus. `otd A B` with two folders opens it through the new `Pattern::Folders` launch pattern in `crates/launch`. Tests in `crates/ui-folder-compare/tests/it/view.rs` and `crates/launch/tests/it/launch.rs`. `cargo xtask ci` passed on macOS, pull-request CI passed on Linux and macOS, and the full-matrix run passed on Windows too. Rule 9: tree grid widget filed as #84. https://github.com/alextiboldi/overthediff/pull/87
  - Verify by hand: with a screen reader (VoiceOver or NVDA), open a Folder Compare of two folders and arrow through the tree grid; rows, name cells (`<name> on the left/right`) and gap cells (`Not on the left/right`) are read correctly, and the reader follows arrow-key focus.
- **E08-S1-T3** — Tree expansion commands · size: S · depends: E08-S1-T2 · status: needs-verification
  - AC: Open Subfolders expands a folder recursively, Expand All opens every folder (scanning as needed), and Collapse All closes them all.
  - Implemented: `crates/ui-folder-compare/src/expand.rs` keeps the open folders by row key and gives `FolderCompare::shown` the flattened visible rows (`Shown`: row, depth, key and each side's folder path in that side's spelling); six View-menu commands for the `folder-compare` view, held by `otd_ui_folder_compare::commands()` with no default shortcuts (Q24): Open Folder, Close Folder, Open Subfolders, Close Subfolders, Expand All and Collapse All (`folder-compare.open_folder` and so on). They act on the current cell's row from either side, are disabled where they have nothing to do, and ask each side's `TreeScan` to list folders as they open; Open Subfolders and Expand All keep opening below as listings arrive, Close Folder keeps the folders open inside it, and a close drops the selected rows it hides. `crates/ui-folder-compare/src/lib.rs` indents names by depth with Lucide's chevron, gives each row its tree level and a folder its open state, and keeps focus on its row by key as rows open above it. Recorded in AGENTS.md "## This repository" and `docs/feature-names.md`; ATTRIBUTION.md and NOTICE cover the chevrons. Tests in `crates/ui-folder-compare/tests/it/expand.rs`. `cargo xtask ci` passed on macOS, pull-request CI passed on Linux and macOS, and the full-matrix run passed on Windows too. https://github.com/alextiboldi/overthediff/pull/95
  - Verify by hand: with a screen reader (VoiceOver or NVDA), open a Folder Compare, run View > Expand All and arrow through the tree grid; each row's level and each folder row's expanded or collapsed state are announced, and change when Open Folder or Close Folder runs on it.
- **E08-S1-T4** — Status colours, folder aggregation, colour spot and glyphs · size: M · depends: E08-S1-T1, E08-S1-T2 · flags: `accessibility` · status: needs-verification
  - AC: a fixture with same, newer, older, size-different, orphan, linked and unreadable items shows the specified colour, spot and glyph for each. Parent folders show the aggregated cues.
  - Implemented: `look(row, side)` in `crates/ui-folder-compare/src/look.rs` works out each side's cues, link flag and error, with the five colour roles (`folder.status.same`, `.unknown`, `.orphan`, `.different`, `folder.glyph.error`) and their contrast pairs registered in `apps/otd/src/colour_roles.rs`; a folder pair whose listings are done takes every cue its contents hold, and one not listed yet is unknown. `name_cell` in `crates/ui-folder-compare/src/lib.rs` draws each side in its status colour with a colour spot left of the name (split into one part per cue for a folder), Lucide `arrow-up-right` after a link or junction and a red Lucide `x` after an unreadable item, and gives the name cell its status in words as tooltip (on hover and keyboard focus) and accessible description; `otd_ui::paint_lucide` in `crates/ui/src/lib.rs` paints the icons. Recorded in AGENTS.md "## This repository"; ATTRIBUTION.md and NOTICE updated. Tests in `crates/ui-folder-compare/tests/it/look.rs` and `tests/it/view.rs`. `cargo xtask ci` passed on macOS, pull-request CI passed on Linux and macOS, and the full-matrix run passed on Windows too. https://github.com/alextiboldi/overthediff/pull/96
  - Verify by hand: with VoiceOver or NVDA, arrow through a Folder Compare: each name cell reads its status (for example "Newer", "Orphan", "Can't be read: ...", "..., Link") after its name. In the running app, in both light and dark themes: rows show gray, black, purple and red as specified, the colour spot left of each name matches, a folder with several cues shows a split spot, a symlink shows the arrow, and a folder that can't be opened shows the small red X. On Windows: a junction in one of the compared folders shows the arrow glyph in the view. Once T3 (opening folders) is merged: an opened parent folder shows its aggregated cues, for example red plus purple for a newer file and an orphan inside.
- **E08-S1-T5** — Click, Ctrl+click and Shift+click selection on either side · size: S · depends: E08-S1-T2 · flags: `accessibility` · status: needs-verification
  - AC: left and right selections are independent, and Shift+click selects a contiguous range on that side.
  - Implemented: per-side selection in `FolderCompare` (`crates/ui-folder-compare/src/lib.rs`): each side keeps its own set of row-name paths (`FolderCompare::selected(side)`, the shape `otd_folder_engine::resolve_selection` takes) with its own anchor; click selects one item on that side, Ctrl+click (Cmd on macOS) adds or removes one, Shift+click selects the contiguous range from that side's anchor, and gaps are never selected. All picks go through `FolderCompare::pick`: keyboard Up/Down select, Shift+Up/Down extend, Ctrl/Cmd+Up/Down only move, Ctrl+Space toggles, and a screen reader's activate picks as a click does. Selected cells paint with the `SELECTION` colour roles, and name cells carry AccessKit `selected` in a multiselectable grid. Recorded in AGENTS.md "## This repository". Tests in `crates/ui-folder-compare/tests/it/view.rs`. `cargo xtask ci` passed on macOS, and pull-request CI passed on Linux and macOS. https://github.com/alextiboldi/overthediff/pull/94
  - Verify by hand: in the running app, Cmd+click on macOS and Ctrl+click on Windows and Linux add or remove one item on that side only, and Shift+click selects a contiguous range on that side. With a screen reader (VoiceOver or NVDA), open a Folder Compare of two folders and use Up/Down, Shift+Down and Ctrl+Space; the focused name cell is announced as selected or not selected, and the grid as multi-selectable.
- **E08-S1-T6** — Refresh, Full Refresh, Refresh Selection and Swap Sides · size: S · depends: E08-S1-T2 · status: done
  - AC: after a file changes on disk, Refresh shows its new status. Swap Sides puts the right folder on the left and recompares.
  - Implemented: four commands for the `folder-compare` view in `crates/ui-folder-compare/src/refresh.rs`, held by the app's command list through `otd_ui_folder_compare::commands()`: Refresh lists every folder the scans have listed again in place, Full Refresh starts both scans over, Refresh Selection lists again the selected items' folders on each side by that side's own names, and Swap Sides exchanges the two folders and recompares. Folder-engine `TreeScan::refresh` (`crates/folder-engine/src/scan.rs`) queues listed folders again without scanning more of the tree. No default shortcuts (Q24). Tests in `crates/ui-folder-compare/tests/it/refresh.rs` and `crates/folder-engine/tests/it/scan.rs`. `cargo xtask ci` passed on macOS, and pull-request CI passed on Linux and macOS (no platform-dependent code, so no full-matrix run). https://github.com/alextiboldi/overthediff/pull/93
- **E08-S1-T7** — Folder Compare Info, Legend and status bar · size: S · depends: E08-S1-T4 · status: needs-verification
  - AC: the Info counts per status match the rows of a fixture. The Legend lists every colour and glyph the view can show.
  - Implemented: `crates/ui-folder-compare/src/info.rs`, held by the app's command list through `otd_ui_folder_compare::commands()` (no default shortcuts, Q24): `folder-compare.info` (Session menu) opens a modal dialog of counts per status from `Info::of(rows)`, which counts each row once by the same rule as the row colours (a listed folder pair only through its contents), and returns the same lines as text to scripts; `folder-compare.legend` (View menu) opens a modal dialog with a swatch and meaning for every colour role and glyph the view shows (`LEGEND_ENTRIES`), and returns the meanings as text. Both dialogs focus Close, and Esc or Close gives focus back. The status bar shows a Scan field ("{done} of {total} folders listed") while a scan runs, then a Result field (Same, Different or Not fully compared) in the shell's polite live region. Colour roles `folder.status.filtered` and `folder.status.unscanned` registered, with 3:1 pairs for all folder roles against `chrome.window`, in `crates/ui-folder-compare/src/look.rs`. Recorded in AGENTS.md "## This repository" and `docs/feature-names.md`. Tests in `crates/ui-folder-compare/tests/it/info.rs` and the `info` unit tests. `cargo xtask ci` passed on macOS, and pull-request CI passed on Linux and macOS (no platform-dependent code, so no full-matrix run). https://github.com/alextiboldi/overthediff/pull/102
  - Verify by hand: with VoiceOver or NVDA, open a Folder Compare and run Session > Folder Compare Info and View > Legend: each dialog is announced by its title, focus lands on Close, every count and Legend entry is read, and Esc gives focus back to the grid cell; the status bar's Result (Same, Different or Not fully compared) is announced when the scan finishes. In the running app, in light and dark themes: the Legend's swatches are visible (black, red, gray, purple, teal and yellow spots, the split red and purple spot, the hollow gray spot, the arrow glyph and the red X), and the Scan progress field shows during a scan of a large folder.

### E08-S2 — Display filters and folder display modes
**Priority**: Must · **Layer**: 2 · **Size**: L · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view)
**Traces**: F-8.11, F-8.12, F-8.4
**Not in this story**: Ignore Unimportant Differences → E08-S13; Suppress Filters → E08-S15; toolbar layouts → E05-S4; remembering filters → E06-S4.

As a code reviewer, I can show only the rows that matter (differences, orphans, newer files on one side) so that I can focus on what changed in a large tree.

**Acceptance criteria**
- [ ] The View menu offers these display filters, and exactly one is active at a time:
  - *Show All*: everything.
  - *Show Differences*: hides matches.
  - *Show Same*: matches only.
  - *Show No Orphans*: hides orphans.
  - *Show Differences but No Orphans*: hides matches and orphans.
  - *Show Orphans*: orphans only.
  - *Show Left Newer* and *Show Right Newer*.
  - *Show Left Newer and Left Orphans* and *Show Right Newer and Right Orphans*.
  - *Show Left Orphans* and *Show Right Orphans*.
  - *Show None*, hidden by default, hides all files.
- [ ] The toolbar gives one-click access to Show All, Show Differences and Show Same, with the other filters in a dropdown.
- [ ] Folder display modes:
  - Default: a folder shows only if at least one file in it is visible.
  - *Always Show Folders*: every folder shows unless file filters exclude it.
  - *Compare Files and Folder Structure*: folders follow the display filter like files. Under Show Orphans, only orphan folders and folders containing orphans show.
  - *Compare Files Only*: suppresses every empty folder, whether truly empty or empty because of filtering.
  - *Ignore Folder Structure*: a flat list where files align by name regardless of subfolder, and each row shows its relative folder (*inferred*).
- [ ] *Show None* together with *Always Show Folders* shows only the folder structure.
- [ ] Changing a filter or mode updates the view without re-scanning the folders (*inferred*).

#### Tasks
- **E08-S2-T1** — Visibility rules for the 13 display filters · size: M · depends: — · status: done
  - AC: a unit table of row status (same, left newer, right newer, different, left orphan, right orphan) × filter gives the specified visibility in every cell.
  - Implemented: `DisplayFilter`, the 13 display filters, with `shows(status)` answering whether a row of a given status shows, in `crates/folder-engine/src/display.rs` (`otd-folder-engine`). A filter worded as hiding something keeps unknown and error rows in view; one worded as showing something only hides them (*unspecified*, decided in the module). `crates/folder-engine/tests/it/display.rs` checks every cell of the row status × filter table. Checks: `cargo xtask ci` on macOS and pull-request CI on Linux and macOS passed. Pull request: https://github.com/alextiboldi/overthediff/pull/131
- **E08-S2-T2** — Display-filter commands and default toolbar access · size: S · depends: E08-S2-T1 · flags: `naming-check`
  - AC: choosing a filter checks it in the View menu and updates the rows. Show None is hidden from menus by default but callable.
- **E08-S2-T3** — Folder visibility for the default mode, Always Show Folders, Compare Files and Folder Structure, and Compare Files Only · size: M · depends: E08-S2-T1 · flags: `naming-check`
  - AC: a fixture with empty folders, folders emptied by filtering and orphan folders gives the specified visibility in each mode, including Show Orphans with Compare Files and Folder Structure.
- **E08-S2-T4** — Ignore Folder Structure (flat list) · size: M · depends: — · status: done
  - AC: same-named files from different subfolders align on one row and show their relative folders. Turning the mode off brings back the tree.
  - Implemented: *View > Flat List* (`folder-compare.flat_list`, a check item, no default shortcut, caption per `docs/feature-names.md`) in `crates/ui-folder-compare/src/flat.rs`: while on, `FolderCompare::shown` and the grid give one row per file pair from folder-engine `align_flat`, each name cell showing the file's path from its side's base; every folder on both sides is listed without touching the tree's open folders, so turning it off brings back the tree as it was. Unpaired files stay unknown until the other side is fully listed, an unlistable folder gets an error row, Refresh keeps rows of folders waiting to be relisted; Info and the status bar count the flat rows, the selection clears on each toggle, and Expand All, Collapse All and Refresh Selection are disabled while it is on. Recorded in AGENTS.md "## This repository". Tests in `crates/ui-folder-compare/tests/it/flat.rs` and `flat.rs`'s unit tests. `cargo xtask ci` passed on macOS, pull-request CI passed on Linux and macOS, and the full-matrix run passed on Windows too. https://github.com/alextiboldi/overthediff/pull/135
- **E08-S2-T5** — Structure-only view with Show None and Always Show Folders · size: S · depends: E08-S2-T2, E08-S2-T3
  - AC: with both set, only folders show.

### E08-S3 — Name filters
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view), E02-S1 (file masks)
**Traces**: F-8.8, F-8.7, F-9.10, App-A
**Not in this story**: presets and Add To Presets → E08-S14; other filters → E08-S12; Suppress Filters → E08-S15; /filters → E11-S3.

As a developer, I can include or exclude items by name pattern so that build output, caches and other noise drop out of the comparison.

**Acceptance criteria**
- [ ] The toolbar *Filters* edit takes `;`-separated masks, where a `-` prefix excludes and a trailing `\` means a folder. `*.cs;-obj\;-bin\` shows only `.cs` files outside `obj` and `bin` folders.
- [ ] Masks follow the Appendix A.1 syntax, including `*.` (no extension) and the relative forms `p\f`, `.\f` and `...\f`. The base folder is always included.
- [ ] The *Filters* toolbar button opens Session Settings on the **Name Filters** tab. It has four multi-line lists: *Include Files*, *Exclude Files*, *Include Folders* and *Exclude Folders*, one mask per line and without the `-` prefix. **Clear** empties all four.
- [ ] The toolbar edit and the Name Filters tab always show the same filter (*inferred*).
- [ ] *Actions > Exclude* adds the selected items to the Exclude lists and hides them. When several files of one type are selected, it offers to exclude the whole type, such as `*.log`. Removing the entry from the Exclude lists brings the items back.
- [ ] Excluded items take no part in the comparison and don't affect folder colours.
- [ ] For any operation, a selected folder resolves to its visible contents only, so excluded items are never part of the resolved set.

#### Tasks
- **E08-S3-T1** — Feed the name masks into the comparison's visibility hook · size: M · depends: — · flags: `craft-libs`
  - AC: each Appendix A.1 example shows exactly the expected items on a fixture tree: `*.pas;*.dfm;*.dpr`, `Source\`, `-*.dcu;-*.ddp`, `-Backup\`, `Windows\*.txt`, `...\Windows\*.txt` and `.\Windows\*.txt`.
- **E08-S3-T2** — Toolbar Filters edit · size: S · depends: E08-S3-T1
  - AC: typing masks and pressing Enter re-filters. An invalid mask shows an error and keeps the previous filter (*inferred*).
- **E08-S3-T3** — Name Filters tab with four lists and Clear, opened by the Filters button · size: S · depends: E08-S3-T1
  - AC: a mask in Exclude Folders hides that folder, Clear restores the unfiltered view, and the toolbar edit reflects the tab's contents.
- **E08-S3-T4** — Actions > Exclude, with the offer to exclude a whole type · size: S · depends: E08-S3-T3
  - AC: excluding one file adds its name to Exclude Files. Excluding three `.log` files offers `*.log`. Removing the entry shows them again.
- **E08-S3-T5** — Resolved operation sets leave out filtered items · size: S · depends: E08-S3-T1
  - AC: selecting a folder whose `*.tmp` files are excluded resolves to its other files only.

### E08-S4 — Quick-test comparison criteria
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view)
**Traces**: F-8.3, F-8.4
**Not in this story**: Windows attributes → E08-S17; content tests → E08-S5; versions → E25-S3.

As a release engineer, I can choose which metadata decides "different" so that copies made by tools that shift timestamps or change case don't flood the view with false differences.

**Acceptance criteria**
- [ ] The Comparison tab's quick tests, each of which can be turned on or off:
  - **Compare file size**.
  - **Compare timestamps**, with a tolerance in seconds and the sub-options *Ignore daylight saving difference* (ignores an exact 1-hour difference) and *Ignore timezone differences* (ignores differences of a whole number of hours).
  - **Compare filename case**: different when only the capitalisation differs.
  - **Compare Unix metadata**: *Permissions* (user/group/other rwx and sticky), *Owner* ID and *Group* ID.
  - **Align filenames with different extensions**: lines up `a.txt` with `a.csv`.
  - **Align filenames with different Unicode normalization forms**: lines up NFC and NFD equivalents.
- [ ] Timestamps within the tolerance count as equal. Outside it, the later file is newer.
- [ ] A test that is turned off never contributes to "different".
- [ ] Criteria apply to files. Folders take their status from their contents.
- [ ] Closing Session Settings after a change recompares the view.

#### Tasks
- **E08-S4-T1** — Timestamp rules: tolerance, daylight saving and time zone · size: M · depends: —
  - AC: unit cases with a 2 s tolerance:
    - Files 1 s apart are same.
    - Files exactly 3600 s apart are same with Ignore daylight saving on.
    - Files 7200 s apart are same with Ignore timezone on.
    - Files 3599 s apart are different with both on.
    - The tolerance also applies around the 1-hour and whole-hour offsets (*inferred*).
- **E08-S4-T2** — Filename-case and Unix-metadata tests · size: S · depends: — · status: done
  - AC: `Readme.md` vs `README.md` differ only when Compare filename case is on. A permissions-only difference shows red only when Permissions is on.
  - Implemented: `Criteria` in `crates/folder-engine/src/status.rs` gets four switches, `filename_case`, `unix_permissions`, `unix_owner` and `unix_group`, and `quick_test` makes a file pair different when a switch that is on sees a difference: names equal only after Unicode lower-casing, mode bits `0o1777` (rwx and sticky), or the owner or group as the source reports it. The Unix tests compare only when both sides report a Unix mode (*unspecified*, defaulted). Every existing `Criteria` literal (the view's `CRITERIA` in `crates/ui-folder-compare`, the headless quick compare in `apps/otd`, the test constants) sets them off until E08-S4-T4 adds the controls; red comes from `look`'s `Status::Different` mapping (E08-S1-T4). Tests in `crates/folder-engine/tests/it/status.rs`. `cargo xtask ci` passed on macOS, and pull-request CI passed on Linux and macOS (no platform-dependent code, so no full-matrix run). https://github.com/alextiboldi/overthediff/pull/136
- **E08-S4-T3** — Alignment across extensions and Unicode normalization forms · size: M · depends: —
  - AC: `a.txt` and `a.csv` share a row only with the extension option on. An NFC name and its NFD twin share a row only with the Unicode option on.
- **E08-S4-T4** — Quick-test controls on the Comparison tab; applying recompares · size: S · depends: E08-S4-T1, E08-S4-T2, E08-S4-T3 · flags: `naming-check`
  - AC: each control switches its test, and closing the dialog after a change updates the colours.

### E08-S5 — Content comparison by CRC and binary
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view)
**Traces**: F-8.3, F-8.1
**Not in this story**: rules-based → E08-S13; comparison triggered by opening a pair → E08-S10.

As a QA engineer, I can check that files really have the same content, not just the same size and date, so that I can trust "same" before a release.

**Acceptance criteria**
- [ ] The Comparison tab has **Compare contents**, with either *CRC comparison* or *Binary comparison* (byte by byte).
- [ ] With **Skip if quick tests indicate files are the same** on, only pairs that the quick tests say differ are read.
- [ ] **Override quick test results** is on by default: a content match turns the row black even when timestamps differ.
- [ ] A center column between the panes shows each pair's result: *Binary same* or *Binary differences*, each with its own glyph (*inferred* glyphs).
- [ ] With Compare contents on, content tests run when the session loads.
- [ ] *Actions > Compare Contents* runs a content test on demand for the selection, letting the user choose CRC or Binary.
- [ ] Content tests run in the background with progress and can be cancelled. A folder containing a binary-different pair shows as different.

- [ ] Clicking a row's cell in the center column selects both sides of that row.
#### Tasks
- **E08-S5-T1** — Content options on the Comparison tab: Compare contents (CRC or Binary), Skip if quick tests indicate files are the same, Override quick test results · size: S · depends: —
  - AC: the options are kept in the session settings, and changing them recompares.
- **E08-S5-T2** — Run content tests on load and fold the results into row status, applying the skip and override rules · size: M · depends: E08-S5-T1 · flags: `licence-check`
  - AC: a pair with equal content but different timestamps shows black with Override on and red with it off. With Skip on, quick-same pairs get no content result.
- **E08-S5-T3** — Center column with content results · size: S · depends: E08-S5-T2
  - AC: binary-same and binary-different pairs show distinct glyphs, with a tooltip naming the result.
- **E08-S5-T4** — Actions > Compare Contents on the selection, choosing CRC or Binary, with progress and cancel · size: M · depends: E08-S5-T2
  - AC: running it on a selected folder fills the center column for its visible pairs only. Cancel leaves the remaining pairs untested.
- **E08-S5-T5** — Select both sides by clicking the center column · size: S · depends: E08-S5-T3
  - AC: clicking a row's center cell selects its left and right items. Ctrl+click on another center cell adds both of that row's items.

### E08-S6 — Columns: choose, sort and extra metadata
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view)
**Traces**: F-8.2
**Not in this story**: Status and Revision → E26-S4; Version → E25-S3; DOS attribute letters in names → E08-S17.

As an IT administrator, I can choose which file details show and sort by any of them so that I can spot size, ownership or permission differences quickly.

**Acceptance criteria**
- [ ] Clicking a column header sorts by that column, and clicking again reverses the order. Rows stay paired while sorted.
- [ ] Right-clicking the header, or *View > Columns*, shows or hides **Name**, **Ext**, **Size**, **CRC**, **Modified**, **Attributes**, **Owner** and **Group**.
- [ ] **Ext** shows the extension.
- [ ] **Size** shows the file size, or for a folder the total size of its contents. A folder's Size stays blank until its contents are fully loaded.
- [ ] **CRC** shows the CRC32. A value the data source already holds, such as one stored in a zip, is used as-is. Otherwise it is computed on demand.
- [ ] **Attributes** shows Windows attribute codes (A archive, C compressed, D directory, E encrypted, H hidden, I not indexed, L symlink or reparse point, M recall on data access, N normal, O offline, P pinned, R read-only, S system, T temporary, U unpinned, Z sparse) or a Unix mode string like `ls` shows.
- [ ] **Owner** and **Group** show the Unix owner and group.
- [ ] Column choices and sort order stay in place through a refresh (*inferred*).

#### Tasks
- **E08-S6-T1** — Column chooser (header right-click and View > Columns) · size: S · depends: —
  - AC: hiding Size removes it from both panes, and the choice survives a refresh.
- **E08-S6-T2** — Sort by any visible column · size: S · depends: —
  - AC: sorting by Size orders files within each folder ascending, then descending on a second click, and pairs stay on one row.
- **E08-S6-T3** — Ext, CRC and folder Size values · size: M · depends: —
  - AC: a folder's Size equals the sum of its files once loaded and is blank before. CRC shows the source-provided value when one exists and computes it otherwise.
- **E08-S6-T4** — Attributes, Owner and Group values per platform · size: S · depends: — · flags: `licence-check`
  - AC: on Windows, a read-only hidden file shows the R and H codes. On Unix, a file shows a mode string such as `-rw-r--r--` plus owner and group names.

### E08-S7 — Selection commands
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view)
**Traces**: F-8.13, F-8.15, F-9.10
**Not in this story**: Space marking → E08-S10; selecting both sides by clicking the center column → E08-S5.

As a power user, I can select exactly the items I want with the mouse, the keyboard and selection commands so that later actions hit the right files.

**Acceptance criteria**
- [ ] Press-and-drag ("swipe") selects a range. A swipe that crosses the middle selects both sides.
- [ ] *Select All* selects all visible items, and *Select All Files* selects visible files only.
- [ ] After a both-sides selection, Shift+Left arrow limits it to the left side and Shift+Right arrow to the right side.
- [ ] *Select Newer* (left, right or both) selects the items that are newer on that side. *Select Orphans* (left, right or both) selects orphans on that side.
- [ ] *Invert Selection* selects the visible unselected items and deselects the rest.
- [ ] *Copy Filename* copies the full paths of the selected items to the clipboard, one per line (*inferred* separator).

#### Tasks
- **E08-S7-T1** — Swipe selection across one or both sides · size: M · depends: —
  - AC: a swipe from the left pane across the middle selects rows on both sides. A swipe inside one pane selects that side only.
- **E08-S7-T2** — Select All and Select All Files · size: S · depends: —
  - AC: hidden rows are never selected, and Select All Files leaves folders unselected.
- **E08-S7-T3** — Shift+Left/Right limits the selection to one side · size: S · depends: —
  - AC: after Select All, Shift+Right leaves only right-side items selected.
- **E08-S7-T4** — Select Newer, Select Orphans and Invert Selection · size: S · depends: —
  - AC: on a fixture, Select Newer (left) selects exactly the left-newer items, and Invert flips the selection within the visible items.
- **E08-S7-T5** — Copy Filename · size: S · depends: —
  - AC: with two files selected, both full paths are on the clipboard.

### E08-S8 — Base-folder navigation and history
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view)
**Traces**: F-8.14, F-6.1
**Not in this story**: Sync Base Folders → E12-S1; Merge Base Folders → E14-S1.

As a developer, I can re-root the comparison on any pair of subfolders and step back and forth through what I compared so that I can drill into one area and return.

**Acceptance criteria**
- [ ] *Set as Base Folder* makes the selected folder the base on its own side. *Set as Base on Other Side* makes it the base on the opposite side.
- [ ] *Set as Base Folders* makes two selected folders the left and right bases in this view.
- [ ] *Up One Level* (left, right or both) moves the base to its parent.
- [ ] *Browse for Folder* (left or right) picks a new base with a folder dialog.
- [ ] *Compare Parent Folders* opens a new Folder Compare view on the parents of the current bases.
- [ ] *Back* and *Forward* step through the base-folder pairs previously compared in this view.

#### Tasks
- **E08-S8-T1** — Set as Base Folder, Set as Base on Other Side and Set as Base Folders · size: M · depends: —
  - AC: each command loads the expected pair. Set as Base Folders is enabled only with exactly two folders selected (*inferred*).
- **E08-S8-T2** — Up One Level (left, right, both) and Browse for Folder (left, right) · size: S · depends: —
  - AC: Up One Level on both sides shows the two parents. At a root it is disabled (*inferred*).
- **E08-S8-T3** — Compare Parent Folders in a new view · size: S · depends: —
  - AC: a new tab opens on the two parent folders, and the original view is unchanged.
- **E08-S8-T4** — Back/Forward history of base-folder pairs · size: S · depends: E08-S8-T1, E08-S8-T2
  - AC: after three re-roots, three Backs return to the first pair and Forward replays them. A new re-root after Back clears the forward history (*inferred*).

### E08-S9 — Step through differences and find filenames
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view), E02-S1 (regular expressions)
**Traces**: F-8.14

As a code reviewer, I can jump from one differing item to the next and find items by name so that I can walk a large comparison without scrolling.

**Acceptance criteria**
- [ ] *Next Difference* and *Previous Difference* move to the next or previous differing item, skipping same and hidden rows. They open collapsed folders when the next difference is inside one (*inferred*).
- [ ] The **Find Filename** dialog has these options: text, *Match character case*, *Whole words only*, *Regular expression* (with the Insert Regular Expression Element picker and RE Examples), *Wrap search*, *Search up* and *Sides* (left, right or both).
- [ ] *Find Next Filename* and *Find Previous Filename* repeat the last search.
- [ ] When no further match or difference exists, the view says so (*inferred*).

#### Tasks
- **E08-S9-T1** — Next/Previous Difference across the tree · size: M · depends: —
  - AC: from the top of a fixture, repeated Next Difference visits every differing item in display order and never a same item.
- **E08-S9-T2** — Find Filename dialog with every option · size: M · depends: —
  - AC: regex `^test_.*\.py$` with Sides = left finds only left-side matches. With Whole words only, `apple` doesn't match `apples`. Matches inside collapsed folders are found and revealed (*inferred*).
- **E08-S9-T3** — Find Next/Previous Filename with wrap and search-up · size: S · depends: E08-S9-T2
  - AC: with Wrap search on, Find Next after the last match returns to the first. Search up reverses the direction.

### E08-S10 — Open file pairs in file views
**Priority**: Must · **Layer**: 2 · **Size**: L · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view), E07-S1 (Text Compare view)
**Traces**: F-8.15, F-6.4, F-8.3
**Not in this story**: Open With → E08-S16; Quick Compare → E08-S19; Copy File and Open Next Difference → E08-S18; child scopes and placement → E06-S7; read-only children → E06-S8; limit to current folder → E07-S19; child commands in other views → E06-S11.

As a code reviewer, I can open any file pair from the folder view in a file comparison and step to the next differing pair so that I can review a whole change set file by file.

**Acceptance criteria**
- [ ] *Open* (Enter or double-click) on a file row opens a child session in a new tab, in the view that the file's format chooses. Text formats open in Text Compare. On a row the view can expand (a folder, or an item a data source exposes as a container), Open expands it instead.
- [ ] With exactly two items selected, Open compares them with each other, whatever their rows or names.
- [ ] With one item selected, Open compares it with the item across from it. An orphan opens with the other side empty (*inferred*).
- [ ] *Compare To*: select one item, choose Compare To, then click a second item. The two open in a new view.
- [ ] Space marks an item, and the mark is visible. With two items marked, Open compares them even across rows and names.
- [ ] Opening a pair runs its comparison, and the file view's result (same or different) becomes the row's content result, so the row's colour follows it the same way it follows a content test.
- [ ] Text Compare child sessions offer *Next Difference Files* and *Previous Difference Files*. These open the parent's next or previous pair with differences, in the parent's display order. When none is left in that direction, the view says so (*inferred*).

#### Tasks
- **E08-S10-T1** — Open/double-click a row into a child session, with the view chosen by the file's format · size: M · depends: —
  - AC: double-clicking a `.txt` pair opens a Text Compare tab with both files, and the tab is linked to its parent folder view. Double-clicking a test source's container row expands it instead.
- **E08-S10-T2** — Pair-selection rules: two selected items, one item and its partner across, Compare To · size: M · depends: E08-S10-T1
  - AC: with `a.txt` on the left and `b.txt` on the right of another row selected, Open compares `a.txt` with `b.txt`. Compare To compares the first item with the clicked one.
- **E08-S10-T3** — Space marks and opening two marked items · size: S · depends: E08-S10-T2
  - AC: marking two files in different folders and pressing Open compares them. Marks clear after opening (*inferred*).
- **E08-S10-T4** — Report child results back to the parent row · size: M · depends: E08-S10-T1
  - AC: opening a pair with identical content but different timestamps turns the row black, because override-quick-results is on by default. Child results map to same or different. Saving an edit in the child updates the row (*inferred*).
- **E08-S10-T5** — Next/Previous Difference Files in Text Compare child sessions · size: M · depends: E08-S10-T1, E08-S10-T4
  - AC: from the first differing pair, Next Difference Files opens the next differing pair in display order and skips same pairs. At the last pair it reports that none is left.

### E08-S11 — Folder loading and scanning behaviour
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view)
**Traces**: F-8.6, F-8.1
**Not in this story**: Archive handling → E04-S2; FTP touch → E21-S2; Windows options → E08-S17; Automatic refresh → E08-S20.

As an IT administrator comparing large or slow trees, I can control how and when subfolders are scanned and expanded so that I see accurate colours quickly without waiting for everything.

**Acceptance criteria**
- [ ] The Handling tab has these options:
  - *Automatically scan subfolders in background*: reads subfolders after load so folders are coloured before they are opened.
  - *Automatically scan top-level orphan subfolders*: scans orphan folders too, which would otherwise be coloured without scanning.
  - *Expand subfolders when loading session*, with the sub-option *Only expand subfolders with differences*.
  - *Follow symbolic links*: shows symlinks and junctions as their target, with its type, size, modified time and attributes.
- [ ] With background scanning off, a folder that hasn't been opened yet shows as a **yellow folder**.
- [ ] A folder whose background scan hasn't finished shows as a **hollow folder**.
- [ ] The user can browse, expand and select while background scanning runs (*inferred*).

#### Tasks
- **E08-S11-T1** — Background subfolder scanning with the hollow-folder cue · size: M · depends: —
  - AC: on a deep fixture, folders show hollow until scanned and then show their aggregated colour, while other folders can be expanded meanwhile.
- **E08-S11-T2** — Unscanned yellow folders when background scanning is off · size: S · depends: —
  - AC: with the option off, unopened folders are yellow, and opening one scans and colours it.
- **E08-S11-T3** — Scan top-level orphan subfolders · size: S · depends: E08-S11-T1
  - AC: with the option on, an orphan folder is scanned in the background like a paired one. With it off, the folder is coloured as an orphan without being read.
- **E08-S11-T4** — Expand subfolders when loading, optionally only those with differences · size: S · depends: —
  - AC: loading with the sub-option on opens exactly the folders that contain differences.
- **E08-S11-T5** — Follow symbolic links · size: S · depends: —
  - AC: with the option on, a link to a folder shows the target's contents and metadata. With it off, it shows as a link item with the link glyph.

### E08-S12 — Other filters
**Priority**: Should · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view), E02-S1 (regular expressions)
**Traces**: F-8.9
**Not in this story**: DOS attribute filter → E08-S17; version-control status filter → E26-S4.

As a release engineer, I can filter items by date, size, content or file type so that I see only the files relevant to a release window.

**Acceptance criteria**
- [ ] The **Other Filters** tab holds a list of filter items (New, edit, delete). It also has the checkbox *Exclude protected operating system files*, which is on by default. Protected means both system and hidden.
- [ ] Filter items:
  - *Date modified*: Older than or Newer than, using either Day(s) ago (whole days relative to midnight on the day the comparison runs) or an absolute date and time.
  - *Size*: Smaller than or Larger than, with the value in bytes, KB, MB or GB.
  - *Content*: Containing or Not containing, plus a text or regex search definition.
  - *Unix file type*: Is or Is not, plus the type (block device, character device, symbolic link, named pipe, regular file, socket). Matching items are excluded.
- [ ] An item must pass every active filter to stay visible (*inferred*).
- [ ] While other filters are active, a comment bar at the bottom of the view shows an *Other Filters* button that opens the tab.
- [ ] When a filter hides an item on one side but not its partner, both stay visible and the hidden one is teal.
- [ ] Folders aren't filtered directly. They are affected only through their filtered contents.

#### Tasks
- **E08-S12-T1** — Other Filters tab with the item list and the protected-files checkbox · size: S · depends: —
  - AC: items can be added, edited and removed, and the list is kept in the session settings.
- **E08-S12-T2** — Date-modified and Size filters · size: M · depends: E08-S12-T1
  - AC: "Older than 2 days ago" is evaluated against midnight on the day the comparison runs. "Larger than 1 MB" hides smaller files, with 1 KB = 1024 bytes (*inferred*).
- **E08-S12-T3** — Content filter with text or regex · size: M · depends: E08-S12-T1
  - AC: "Containing TODO" keeps only files containing TODO. Regex mode uses the Appendix A.2 syntax. An unreadable file is reported and treated as not matching (*inferred*).
- **E08-S12-T4** — Unix file type filter and Exclude protected operating system files · size: S · depends: E08-S12-T1
  - AC: "Is symbolic link" hides links. On Windows, a file that is both hidden and system is excluded by default.
- **E08-S12-T5** — One-sided hiding shown in teal, and the comment bar · size: S · depends: E08-S12-T2 · flags: `accessibility`
  - AC: in a pair where only the left file is older than the cutoff, both rows stay visible with the hidden side teal. The comment bar's button opens the tab.

### E08-S13 — Rules-based content comparison
**Priority**: Must · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E08-S5 (CRC and binary content comparison), E07-S4 (text importance rules), E06-S2 (session defaults, the source of the text rules), E11-S2 (headless `/qc`)
**Traces**: F-8.3, F-8.11, F-25.3, F-25.4, F-21.1 · **Open questions**: Q17

As a developer, I can have the folder view compare file contents with the same rules as Text Compare so that files differing only in whitespace, line endings or other unimportant ways don't show as changed.

**Acceptance criteria**
- [ ] *Compare contents* offers **Rules-based comparison** alongside CRC and Binary.
- [ ] A rules-based comparison runs a binary check first. Files that differ are then compared through their file formats with the text comparison rules, taken from the Text Compare session defaults (*inferred*). Files without a text format fall back to binary (*inferred*).
- [ ] The center column shows one of four results:
  - *Binary same*.
  - *Rules-based same*: byte differences the rules ignore, such as encoding or line endings.
  - *Unimportant differences*.
  - *Important differences*.
- [ ] *Actions > Compare Contents* offers Rules-based for the selection.
- [ ] *View > Ignore Unimportant Differences* makes pairs with only unimportant differences count as same in colouring and statistics.
- [ ] `/qc` with no type compares two files rules-based without UI. The exit code is 1 for binary same, 2 for rules-based same, 12 for similar and 13 for rules-based differences, per T1. A load error exits with 100.

#### Tasks
- **E08-S13-T1** — Decide when "similar" (exit code 12) applies (Q17) · size: S · depends: — · flags: `decision`
  - AC: decision recorded in this story. Proposed default:
    - 12 when only unimportant differences remain.
    - 13 when any important difference remains.
    - 2 when only byte differences that the rules ignore (encoding, line endings) remain.
- **E08-S13-T2** — Rules-based content test: a binary check, then a text comparison with the file's format and the text rules · size: M · depends: E08-S13-T1
  - AC: in a fixture:
    - Identical files give Binary same.
    - CRLF vs LF only gives Rules-based same.
    - A trailing-whitespace-only change, with trailing whitespace unimportant, gives Unimportant differences.
    - A changed word gives Important differences.
- **E08-S13-T3** — Rules-based option in Compare contents and in Actions > Compare Contents; center-column results · size: S · depends: E08-S13-T2 · flags: `naming-check`
  - AC: the four results show distinct glyphs and tooltips, and an on-demand run on a selection fills them in.
- **E08-S13-T4** — Ignore Unimportant Differences in Folder Compare · size: S · depends: E08-S13-T2
  - AC: with the toggle on, an Unimportant-differences pair is black and counted as same in Folder Compare Info. With it off, the pair counts as different.
- **E08-S13-T5** — Headless rules-based `/qc` with exit codes 1, 2, 12, 13 and 100 · size: S · depends: E08-S13-T2
  - AC: the four fixture pairs from T2 exit with 1, 2, 12 and 13. A missing file exits with 100.

### E08-S14 — Name filter presets
**Priority**: Should · **Layer**: 3 · **Size**: S · **Platforms**: All
**Needs**: E08-S3 (name filters)
**Traces**: F-8.8, F-26.11

As a developer, I can save name filters as presets and pick them from the toolbar so that I don't retype the same masks for every comparison.

**Acceptance criteria**
- [ ] *Tweaks > Name Filter Presets* defines named masks, such as "Web files" = `*.htm;*.html;*.jpg`.
- [ ] The toolbar filter dropdown lists the presets, and picking one applies its masks.
- [ ] **Add To Presets** on the Name Filters tab saves the current filter as a named preset, with an editable mask.
- [ ] Presets are available in every Folder Compare session (*inferred*).

#### Tasks
- **E08-S14-T1** — Preset store and Tweaks editor (add, rename, edit mask, delete) · size: S · depends: —
  - AC: presets survive a restart.
- **E08-S14-T2** — Toolbar dropdown applies a preset · size: S · depends: E08-S14-T1
  - AC: picking "Web files" puts its masks in the filter edit and re-filters.
- **E08-S14-T3** — Add To Presets from the Name Filters tab · size: S · depends: E08-S14-T1
  - AC: the new preset's mask defaults to the current filter and can be edited before saving.

### E08-S15 — Suppress filters
**Priority**: Should · **Layer**: 3 · **Size**: S · **Platforms**: All
**Needs**: E08-S2 (display filters), E08-S3 (name filters), E09-S1 (copy), E09-S2 (move), E09-S3 (delete), E09-S4 (synchronize selection)
**Traces**: F-8.7, F-26.10

As a power user, I can temporarily see and act on filtered-out items so that I can handle an exception without editing my filters.

**Acceptance criteria**
- [ ] *View > Suppress Filters* temporarily turns off the file filters and any display filter in effect. Previously hidden items appear in **teal**.
- [ ] Teal items can be selected and processed by Copy, Move, Delete and Synchronize.
- [ ] Turning Suppress Filters off restores the filtered view.
- [ ] With the File Operations option **Include hidden items by default** on, operations on a selected folder also include its filtered-out contents.
- [ ] The Copy, Move, Delete and Synchronize confirmations each offer a per-operation override of that option.

#### Tasks
- **E08-S15-T1** — Suppress Filters toggle with teal display of filtered items · size: S · depends: — · flags: `accessibility`
  - AC: with `-*.log` excluded, turning it on shows the log files in teal, and turning it off hides them again.
- **E08-S15-T2** — Processing teal items · size: S · depends: E08-S15-T1
  - AC: copying a teal file while filters are suppressed copies it.
- **E08-S15-T3** — "Include hidden items by default" and the per-operation override in the copy, move, delete and synchronize confirmations · size: M · depends: —
  - AC: with the option on, copying a folder also copies its excluded files. Clearing the override in the dialog leaves them out for that run.

### E08-S16 — Open With an associated application or a chosen view
**Priority**: Should · **Layer**: 3 · **Size**: S · **Platforms**: All
**Needs**: E08-S10 (opening pairs in file views)
**Traces**: F-8.15
**Not in this story**: configured external applications → E22-S4; Text Edit and Text Patch entries → E24-S5.

As a designer, I can open an item in its default application or force a particular comparison view so that I can inspect files the way I need.

**Acceptance criteria**
- [ ] *Open With > <session type>* lists every registered file view type and opens the selection in the chosen type as a child session, overriding the type the file format would pick.
- [ ] *Open With > Associated Application* opens each selected item with the operating system's default application.
- [ ] The folder view doesn't refresh by itself after an external application edits files. *Refresh* picks up the changes.

#### Tasks
- **E08-S16-T1** — Open With > <session type> submenu built from the registered view types · size: S · depends: —
  - AC: every registered file view type appears, and choosing one opens the pair in that type as a child session.
- **E08-S16-T2** — Open With > Associated Application · size: S · depends: —
  - AC: each selected item opens in its OS default application. For a pair, both items open (*inferred*).

### E08-S17 — Windows file details
**Priority**: Should · **Layer**: 3 · **Size**: M · **Platforms**: Win
**Needs**: E08-S4 (quick-test criteria), E08-S6 (columns), E08-S12 (other filters), E09-S1 (copy), E07-S1 (text views that save files)
**Traces**: F-8.1, F-8.2, F-8.3, F-8.6, F-8.9, F-26.11

As a Windows administrator, I can compare and preserve Windows file attributes and metadata so that copies keep their attributes, permissions, short names and creation dates.

**Acceptance criteria**
- [ ] The Comparison tab gains **Compare file attributes**, off by default because copying changes the archive bit. The user picks which attributes count (archive, hidden, read-only, system…). In the Attributes column, compared attributes show as letters and the others as dashes.
- [ ] When the Ext column is hidden, names show DOS attribute letters in parentheses: `(r)`, `(h)`, `(a)`, `(s)`.
- [ ] Other Filters gains a **DOS attribute** item: *Is set* or *Is not set*, plus the attribute. Matching files are excluded.
- [ ] The Handling tab gains these options, applied when copying:
  - *Maintain DOS 8.3 aliases*: copies keep the same short names.
  - *Copy NTFS file permissions*: copies the security descriptors. Files edited in a text view don't keep them.
  - *Copy creation dates*.
- [ ] The tweak *Remove read-only flag when copying from CD/DVD* clears that flag on copies taken from optical media.

#### Tasks
- **E08-S17-T1** — Attribute quick test with an attribute picker, and letters or dashes in the Attributes column · size: M · depends: — · flags: `licence-check`
  - AC: with only Hidden compared, a pair differing in Archive alone shows as same, while a pair differing in Hidden shows as different. The column shows H or a dash.
- **E08-S17-T2** — DOS attribute letters after names when Ext is hidden · size: S · depends: —
  - AC: hiding Ext shows `(r)` after a read-only file's name, and showing Ext removes it.
- **E08-S17-T3** — DOS attribute Other Filter · size: S · depends: —
  - AC: "Is set: Hidden" excludes hidden files on both sides.
- **E08-S17-T4** — Copy-time options (8.3 aliases, NTFS permissions, creation dates) and the CD/DVD read-only tweak · size: M · depends: — · flags: `licence-check`
  - AC: with each option on, a copied file keeps the source's short name, permissions and creation date. A file copied from optical media loses its read-only flag when the tweak is on.

### E08-S18 — Copy a file across and open the next difference
**Priority**: Should · **Layer**: 3 · **Size**: S · **Platforms**: All
**Needs**: E08-S10 (child sessions), E09-S1 (copy)
**Traces**: F-6.4, F-12.4

As a code reviewer, I can accept a whole file from one side and move straight on to the next differing pair so that I can triage many files quickly.

**Acceptance criteria**
- [ ] A Text Compare child session offers *Copy File to Right* and *Copy File to Left*. Each copies the whole file over the other side using the parent's copy behaviour, including its confirmation setting, and then opens the next pair with differences.
- [ ] After the copy, the parent row shows the pair as same.
- [ ] Unsaved edits in the child are saved or discarded first, after a prompt (*inferred*).
- [ ] At the last differing pair, the copy still happens, and the view reports that no differences are left (*inferred*).

#### Tasks
- **E08-S18-T1** — Copy File to Right/Left in Text Compare child sessions · size: S · depends: —
  - AC: the file is copied by the parent's copy rules, the confirmation setting is honoured, and the parent row updates to same.
- **E08-S18-T2** — Open the next differing pair after the copy · size: S · depends: E08-S18-T1
  - AC: the child tab then shows the next differing pair in the parent's display order.

### E08-S19 — Quick Compare dialog
**Priority**: Should · **Layer**: 4 · **Size**: M · **Platforms**: All
**Needs**: E08-S10 (opening pairs in file views), E08-S13 (rules-based content comparison)
**Traces**: F-8.15, F-26.1

As a QA engineer, I can check whether two files match without opening a full comparison so that quick yes-or-no checks stay quick.

**Acceptance criteria**
- [ ] *Quick Compare* shows a dialog stating whether the files match, giving the binary or rules-based result. It uses the same two-item and one-item rules as Open.
- [ ] The dialog can open the full view for the same pair (*inferred*).
- [ ] The Startup option **When starting with a file comparison, show the Quick Compare dialog** has the modes *Binary quick compare* and *Rules-based quick compare*.
- [ ] With **Open view automatically if files are different**, a launch with different files goes straight to the full view. The dialog appears only when the files match.

#### Tasks
- **E08-S19-T1** — Quick Compare dialog from the folder view, following the Open selection rules · size: M · depends: — · flags: `naming-check`
  - AC: with two selected items, the dialog reports their result. With one item, it compares that item with the one across.
- **E08-S19-T2** — Startup option for file comparisons, with Binary or Rules-based mode · size: S · depends: E08-S19-T1
  - AC: launching with two files shows the dialog instead of the view, using the chosen mode.
- **E08-S19-T3** — "Open view automatically if files are different" · size: S · depends: E08-S19-T2
  - AC: different files open the full view directly, and identical files show the dialog.

### E08-S20 — Automatic refresh
**Priority**: Should · **Layer**: 2 · **Size**: S · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view)
**Traces**: F-8.6 · **Open questions**: Q18

As an IT administrator watching a folder that changes, I can have the comparison refresh itself so that the view stays current without pressing Refresh.

**Acceptance criteria**
- [ ] The Handling tab option **Automatic refresh** re-scans and refreshes the comparison by itself, using the trigger and interval decided in T1.
- [ ] An automatic refresh keeps folder expansion and the selection for items that still exist (*inferred*).
- [ ] With the option off, changes on disk show only after a manual Refresh.

#### Tasks
- **E08-S20-T1** — Decide the refresh trigger and interval (Q18) · size: S · depends: — · flags: `decision`
  - AC: decision recorded in this story. Proposed default: react to file-system change notifications where the data source supports them, otherwise poll every 5 seconds, with the interval configurable on the Handling tab.
- **E08-S20-T2** — Automatic refresh option and behaviour · size: S · depends: E08-S20-T1 · flags: `licence-check`
  - AC: with the option on, a file added on disk appears within the decided interval and the expansion state is kept. With it off, the file appears only after Refresh.

### E08-S21 — Ignored items
**Priority**: Could · **Layer**: 4 · **Size**: S · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view), E14-S1 (Folder Merge view)
**Traces**: F-8.16, F-10.3, F-9.10

As a team lead, I can mark known, accepted differences as ignored so that they stop showing up as differences.

**Acceptance criteria**
- [ ] *Actions > Ignored* treats the selected items in Folder Compare as same, suppressing their differences.
- [ ] The same command is available in Folder Merge.
- [ ] Running Ignored again on an ignored item restores its real status (*inferred* toggle).
- [ ] Ignored items stay ignored until the view is closed or refreshed in full (*inferred*; persistence isn't specified).

#### Tasks
- **E08-S21-T1** — Ignored toggle in Folder Compare · size: S · depends: —
  - AC: an ignored different pair shows black and is treated as same by every status-based command. Toggling again restores red.
- **E08-S21-T2** — Ignored in Folder Merge · size: S · depends: E08-S21-T1
  - AC: an ignored conflicting row shows as unchanged and needs no merge action (*inferred*).

### E08-S22 — Alignment overrides and manual alignment
**Priority**: Should · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view), E02-S1 (masks and regular expressions)
**Traces**: F-8.5, F-8.10
**Not in this story**: per-session enabled formats → E10-S8.

As a developer comparing trees where files were renamed, I can force differently named items onto the same row so that renamed files are compared with their old versions.

**Acceptance criteria**
- [ ] *Align With*: pick one item, choose Align With, then click an item on the other side, and the two share a row. With two items already selected, they align at once.
- [ ] Align With works only when the items' parent folders are already aligned. Otherwise it is refused with an explanation (*inferred* message).
- [ ] The **Misc** tab holds alignment override rules (**New**). Each rule has *Align left file (or folder)*, *with right file (or folder)*, *Regular expression* (replacement-style; otherwise DOS wildcards) and *Limit to this folder* (a relative path).
- [ ] The catalogue's examples work: `x.txt` ↔ `x.csv`; `*.gif` ↔ `*.png` (same stem); `x*.jpg` ↔ `y*.jpg` (`x1`↔`y1`, `x2`↔`y2`).
- [ ] Override rules are kept in the session settings and apply every time the comparison loads.

#### Tasks
- **E08-S22-T1** — Align With for two items, by pick-then-click or with two already selected · size: M · depends: —
  - AC: aligning `old.txt` (left) with `new.txt` (right) in the same folder puts them on one row and compares them. Aligning across unaligned parent folders is refused.
- **E08-S22-T2** — Wildcard override rules · size: M · depends: —
  - AC: `*.gif` ↔ `*.png` aligns `logo.gif` with `logo.png`. `x*.jpg` ↔ `y*.jpg` aligns `x1.jpg`↔`y1.jpg` and `x2.jpg`↔`y2.jpg`.
- **E08-S22-T3** — Regular-expression rules and Limit to this folder · size: M · depends: E08-S22-T2
  - AC: a regex rule with a `$1` template aligns names as its template says. A rule limited to `src\` has no effect outside that folder.
- **E08-S22-T4** — Misc tab override list (New, edit, delete) · size: S · depends: E08-S22-T2
  - AC: rules are kept with the session settings and applied when the comparison loads.
