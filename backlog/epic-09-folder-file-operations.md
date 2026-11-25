# E09 — Folder file operations

**Group**: feature
**Label**: `area:file-ops`
**Size**: L
**Depends on**: E01, E03, E04 (foundations); E08 (Folder Compare view)
**Source**: catalogue §9, F-8.18, F-26.7, F-26.9, F-26.10. Spec-derived from the feature catalogue and not yet validated. Items marked *inferred* are analyst or PO inferences.

This epic covers reconciling two folder trees from the Folder Compare view: copying, moving, deleting, renaming, retiming and synchronising items, plus the confirmations, backups and log that make bulk changes safe. Without it, users can see what differs but can't fix it in place.

## Behavioral contract
- **Data**:
  - *Operation request*: kind, target side or sides, selected rows, resolved item set and per-kind options (direction, recycle bin, path option, rename mask).
  - *Operation result*: outcome per item (done, skipped or failed, with a message), counts and duration.
  - *Operation settings*: one "Confirm …" option per operation kind (Q3), plus "Synchronize confirmations" (Prompt / Yes to All / No to All) on the File Operations options page.
  - *Backup settings*: enabled, naming, location and retention (Q2).
  - *Log entries*: category, time and text.
- **Routes/endpoints** (desktop app, so these are commands, options and switches): Actions menu, context-menu and toolbar commands in Folder Compare; the File Operations, Backups and Log options; the CLI switch `/nobackups`.
- **Integrations**:
  - E03-S2 selection resolution decides which items an operation touches.
  - E04 data sources perform the copy, move, delete, rename, set-time, set-attributes and recycle-bin calls, and report read-only capability.
  - E01-S1 background-task service provides progress, cancel and the end-of-task error summary.
- **Rules/states**:
  - Selecting a folder means its visible (filtered) contents. If an open folder and some of its children are both selected, only the children are processed.
  - A confirmation appears by default, and each kind has its own option to turn it off.
  - A failure on one item doesn't stop the rest (*inferred*). Cancel stops after the current item.
  - After an operation, the affected rows are re-compared and re-coloured without a full refresh.
  - A read-only target, such as a read-only archive or data source, is refused before anything changes.

---

## Stories

### E09-S1 — Copy to the other side
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view), E04-S1 (file-system access layer)
**Traces**: F-9.1, F-26.10 · **Open questions**: Q3
**Not in this story**: Copy/Move to Folder → E09-S5; backups → E09-S10; log panel → E09-S11; including filtered-out items → E08-S15; Copy File and Open Next Difference → E08-S18.

As a developer reconciling two source trees, I can copy the selected items to the other side so that the target matches the source where I choose.

**Acceptance criteria**
- [ ] *Copy to Other Side* adapts to the selection. Its caption and icon read "Copy to Right" for a left-only selection, "Copy to Left" for a right-only selection, and "Copy to Side…" when both sides are selected.
- [ ] *Copy to Right* (Ctrl+R) and *Copy to Left* (Ctrl+L) are hidden by default and run from those shortcuts. *Copy to Side* asks for the direction first.
- [ ] Copying overwrites existing items on the target and creates any missing parent folders.
- [ ] A selected folder copies only its visible contents. With an open folder plus some of its children selected, only those children are copied.
- [ ] A confirmation shows the item count and direction. The "Confirm file copy" option on the File Operations page can skip it for left-only or right-only selections. Copy to Side still asks for the direction.
- [ ] Progress shows while copying, and Cancel stops after the current item. Failed items are listed in the end-of-task summary while the remaining items still copy.
- [ ] Copied rows re-compare immediately. A copied orphan becomes a paired row, and identical results turn black.
- [ ] A copy onto a read-only data source is refused with a message, and nothing changes.

#### Tasks
- **E09-S1-T1** — Turn a selection into a copy plan (items, direction, target paths, folders to create) · size: S · depends: —
  - AC: unit cases cover a single folder, an open folder with selected children, a mixed file-and-folder selection and an orphan folder. Each yields exactly the visible items with the correct target paths.
- **E09-S1-T2** — Copy to Other Side, Copy to Right/Left and Copy to Side commands with a dynamic caption and icon · size: M · depends: E09-S1-T1
  - AC: caption and icon follow the three selection states. Ctrl+R and Ctrl+L work while hidden from menus. Copy to Side prompts for the direction.
- **E09-S1-T3** — Copy confirmation dialog and the "Confirm file copy" option · size: S · depends: E09-S1-T2
  - AC: the dialog shows the count and direction. With the option off, a left-only or right-only copy runs without a dialog.
- **E09-S1-T4** — Run the copy with progress, cancel and error summary · size: M · depends: E09-S1-T1
  - AC: cancelling halfway leaves completed items copied and reports the rest as not done. A locked target file appears in the summary and the other items still complete. A read-only target is refused before any write.
- **E09-S1-T5** — Re-compare affected rows after the copy · size: S · depends: E09-S1-T4
  - AC: copied rows show their new status with no full refresh. Untouched rows keep their state.

### E09-S2 — Move to the other side
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view), E04-S1 (file-system access layer)
**Traces**: F-9.2, F-26.10 · **Open questions**: Q3
**Not in this story**: Move to Folder → E09-S5; exchanging both sides → E09-S12; backups → E09-S10.

As a user tidying two copies of a project, I can move items to the other side so that I can relocate files without copying and then deleting them by hand.

**Acceptance criteria**
- [ ] *Move to Other Side* has a dynamic caption that follows the selection: "Move to Right" for a left-only selection, "Move to Left" for a right-only selection, "Move to Side…" when both sides are selected. *Move to Right*, *Move to Left* and *Move to Side* are hidden by default, and Move to Side asks for the direction.
- [ ] Each item is written to the target first and removed from the source only after that write succeeds. When the write fails, the source stays.
- [ ] A selected folder moves only its visible contents. With an open folder plus some of its children selected, only those children move. A source folder is removed only when nothing hidden remains in it (*inferred*).
- [ ] A confirmation appears unless "Confirm file move" is off.
- [ ] Progress shows while moving, and Cancel stops after the current item. Failures are listed in the end-of-task summary while the remaining items still move. Rows refresh afterwards: a moved orphan shows on the target side only.

#### Tasks
- **E09-S2-T1** — Move plan with remove-after-successful-write semantics · size: S · depends: —
  - AC: when the write fails for an item, that item's source still exists and the item is listed in the summary.
- **E09-S2-T2** — Move commands with a dynamic caption, and Move to Side direction prompt · size: S · depends: E09-S2-T1
  - AC: caption follows the selection side. The hidden-by-default commands are callable by shortcut.
- **E09-S2-T3** — Move confirmation and the "Confirm file move" option · size: S · depends: E09-S2-T2
  - AC: with the option off, a one-sided move runs without a dialog.
- **E09-S2-T4** — Run the move with progress, cancel, error summary and row refresh · size: M · depends: E09-S2-T1
  - AC: after a cancel, finished items exist only on the target and unfinished items only on the source.

### E09-S3 — Delete
**Priority**: Must · **Layer**: 2 · **Size**: S · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view), E04-S1 (file-system access layer)
**Traces**: F-9.5, F-26.10 · **Open questions**: Q3
**Not in this story**: log panel → E09-S11.

As a user cleaning up a comparison, I can delete items from one or both sides so that I can remove stale files without leaving the tool.

**Acceptance criteria**
- [ ] *Delete* removes the selected items from whichever side or sides they are selected on.
- [ ] The confirmation has a **Use recycle bin if possible** checkbox, which applies to local drives only. Items on sources that report no recycle-bin support, such as network paths, are deleted permanently, and the dialog says so (*inferred*).
- [ ] A selected folder deletes its visible contents. The folder itself goes only when nothing hidden remains in it (*inferred*).
- [ ] The confirmation can be turned off with its own option (Q3).
- [ ] After deletion, rows update: a pair becomes an orphan on the other side, and a deleted orphan disappears.

#### Tasks
- **E09-S3-T1** — Delete plan for one or both sides, with the folder rule · size: S · depends: —
  - AC: unit cases for a left-only selection, a both-sides selection and a folder containing hidden items.
- **E09-S3-T2** — Delete command and confirmation with the recycle-bin checkbox · size: S · depends: E09-S3-T1 · flags: `licence-check`
  - AC: the checkbox is disabled, with an explanation, when every item is on a source without a recycle bin.
- **E09-S3-T3** — Run the delete with progress and errors, then refresh rows · size: S · depends: E09-S3-T1
  - AC: a file in use is reported and the others are deleted. Rows reflect the new state without a full refresh.

### E09-S4 — Synchronize the selection
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view), E04-S1 (file-system access layer)
**Traces**: F-8.18, F-26.10 · **Open questions**: Q3
**Not in this story**: saved, whole-tree sync sessions → E12-S1; script `sync` → E20-S4.

As a user keeping a backup folder current, I can synchronise just the selected part of a comparison in one command so that I don't have to copy and delete item by item.

**Acceptance criteria**
- [ ] The *Actions > Synchronize* submenu offers *Update Right*, *Update Left*, *Update Both*, *Mirror to Right* and *Mirror to Left*. Each acts on the current selection only.
- [ ] *Update Right* copies left-newer and left-orphan items to the right and deletes nothing. *Update Left* does the same in the other direction. *Update Both* copies newer and orphan items in both directions.
- [ ] *Mirror to Right* replaces every different right item with the left version, copies left orphans and deletes right orphans. *Mirror to Left* does the same in the other direction.
- [ ] A confirmation shows the planned copies and deletes as counts before anything changes.
- [ ] During the run, an item whose target is newer than the source or is read-only raises a per-item confirmation (*inferred*). The "Synchronize confirmations" option (Prompt, the default; Yes to All; No to All) decides how those confirmations are answered.
- [ ] Progress shows while synchronising, and Cancel stops after the current item. Failures are listed in the end-of-task summary, and rows refresh afterwards.

#### Tasks
- **E09-S4-T1** — Sync plan per method from row statuses · size: M · depends: —
  - AC: a unit table of status (same, left newer, right newer, different, left orphan, right orphan) × the five methods gives the expected copy or delete action for every cell.
- **E09-S4-T2** — Synchronize submenu and plan confirmation · size: S · depends: E09-S4-T1
  - AC: the confirmation counts match the plan. Choosing No changes nothing.
- **E09-S4-T3** — "Synchronize confirmations" option · size: S · depends: E09-S4-T2
  - AC: Yes to All runs with no further prompts. No to All skips every item that would have prompted.
- **E09-S4-T4** — Run the plan with progress, cancel and error summary, then refresh rows · size: M · depends: E09-S4-T1
  - AC: after Mirror to Right on a fully selected tree, the whole tree compares as same.

### E09-S5 — Copy or move to a folder or archive
**Priority**: Should · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view), E04-S2 (zip archives)
**Traces**: F-9.4, F-7.3
**Not in this story**: other archive formats as targets → E04-S3; script `copyto`/`moveto` → E20-S2.

As a release engineer, I can copy or move selected items to any folder or archive so that I can collect a set of changed files into one deliverable.

**Acceptance criteria**
- [ ] *Copy to Folder* and *Move to Folder* are enabled only when the selection is on one side.
- [ ] The target can be any folder (typed, browsed or picked from history) or an archive file path such as `x.zip`. A missing archive is created.
- [ ] *Keep relative folder structure* recreates the minimum common relative path of the selected items. Intermediate folders are created even when they weren't selected.
- [ ] *Keep base folder structure* recreates the full path from the base folder.
- [ ] *Don't keep folder structure* puts every file flat into the target, and creates only folders that were explicitly selected.
- [ ] Move removes the sources after a successful write. Progress, cancel and the error summary apply.

#### Tasks
- **E09-S5-T1** — Target-path computation for the three path options · size: M · depends: —
  - AC: unit cases with files in sibling and nested folders give the specified layouts for each option.
- **E09-S5-T2** — Copy/Move to Folder dialog (target, browse, history, path option) and commands · size: S · depends: E09-S5-T1
  - AC: the commands are disabled for two-sided selections. The last-used path option is preselected.
- **E09-S5-T3** — Archive targets: create a missing zip and write into an existing one · size: M · depends: E09-S5-T1 · flags: `licence-check`
  - AC: copying three files to a new `x.zip` with Keep relative folder structure produces an archive with that layout, and the archive opens as a folder.
- **E09-S5-T4** — Run copy or move with progress, cancel and error summary · size: S · depends: E09-S5-T1
  - AC: a move leaves no sources for completed items. Failures are listed in the summary.

### E09-S6 — Rename one or many items
**Priority**: Should · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view), E02-S1 (masks and regular expressions)
**Traces**: F-9.6
**Not in this story**: script `rename` → E20-S2.

As a user aligning two trees whose names drifted apart, I can rename one item or a batch of items with a mask or a regular expression so that matching files line up again.

**Acceptance criteria**
- [ ] With one item selected, Rename edits its full name in place.
- [ ] With several items selected, *DOS compatible* mode shows a base mask computed from the names' common parts, with `?` and `*` for the varying parts. Editing the fixed parts renames every item: `abc1.txt` and `abc2.txt` give `abc?.txt`, and `abc?.bak` renames them to `abc1.bak` and `abc2.bak`.
- [ ] *Regular expressions* mode takes an Old mask, a regex matched against each name, and a New mask template using `$1` etc. For example, `.*(\d\.txt)` → `xyz$1` renames `abc1.txt` to `xyz1.txt`, and `txt` → `bak` renames `notes.txt` to `notes.bak` (*inferred*: only the matched part of the name is replaced).
- [ ] A preview lists old and new names. Name clashes are reported before anything is renamed.
- [ ] Renamed rows re-align: an item can pair with an item of the new name on the other side, or become an orphan.

#### Tasks
- **E09-S6-T1** — Compute a base mask from several names · size: M · depends: — · flags: `craft-libs`
  - AC: unit cases include the catalogue's `abc?.txt` example and names that share only an extension.
- **E09-S6-T2** — Apply an edited DOS mask to each name · size: S · depends: E09-S6-T1 · flags: `craft-libs`
  - AC: `abc?.txt` → `xyz?.txt` maps `abc1.txt` to `xyz1.txt`.
- **E09-S6-T3** — Regular-expression rename with replacement templates · size: S · depends: —
  - AC: the two example patterns in the acceptance criteria produce the names given there. An invalid pattern shows an error and renames nothing.
- **E09-S6-T4** — Rename dialog with mode switch, preview and clash check; run and re-align · size: M · depends: E09-S6-T2, E09-S6-T3
  - AC: a clash blocks the run and names the clashing items. After the run, rows re-align without a full refresh.

### E09-S7 — Change attributes and permissions
**Priority**: Should · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view)
**Traces**: F-9.7
**Not in this story**: comparing attributes → E08-S17; script `attrib` → E20-S2.

As a user preparing files for deployment, I can change attributes or permissions on the selected items so that both sides end up with the access rights I expect.

**Acceptance criteria**
- [ ] On Windows, the dialog offers Read-only, Hidden and Archive. System is not offered.
- [ ] On macOS and Linux, the dialog offers read, write and execute for owner, group and other.
- [ ] Checkboxes are tri-state. Unmarked clears the attribute on every item, marked sets it on every item, and gray means mixed. Leaving a box gray leaves each item unchanged (*inferred*).
- [ ] A selected folder changes only its own attributes, not its contents.
- [ ] Per-item failures, such as permission denied, are listed in the summary. Changed rows re-compare.

#### Tasks
- **E09-S7-T1** — Compute each checkbox's initial tri-state from the selection · size: S · depends: —
  - AC: a selection where only some items are read-only shows Read-only as gray.
- **E09-S7-T2** — Attributes dialog for Windows and for Unix permissions · size: M · depends: E09-S7-T1
  - AC: each platform shows only its own controls, and System never appears on Windows.
- **E09-S7-T3** — Apply changes with per-item errors and refresh · size: S · depends: E09-S7-T2 · flags: `licence-check`
  - AC: after the dialog's settings are applied, the OS shows the new attributes on each item. Gray boxes leave items as they were.

### E09-S8 — Touch timestamps
**Priority**: Should · **Layer**: 2 · **Size**: S · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view)
**Traces**: F-9.8
**Not in this story**: script `touch` → E20-S2; FTP local touch → E21-S2.

As a user whose copies differ only by timestamp, I can copy or set modification times so that timestamp-only differences go away.

**Acceptance criteria**
- [ ] *Copy timestamps from other side* gives each selected item its partner's modified time.
- [ ] *Set timestamps to* applies a date and time chosen in a calendar. The calendar starts at the current system time or the last touch value, whichever the user picks.
- [ ] For folders, only the folder's own timestamp changes. If the OS refuses, the failure is reported.
- [ ] Touched rows re-compare. Pairs that differed only by time become same.

#### Tasks
- **E09-S8-T1** — Touch dialog (copy vs set, calendar, initial value; remembers the last touch value) · size: S · depends: —
  - AC: reopening the dialog offers the last value used.
- **E09-S8-T2** — Apply timestamps with per-item errors and refresh · size: S · depends: E09-S8-T1
  - AC: after copying timestamps on a pair that differed only by time, the row turns black.

### E09-S9 — New folders and missing base folders
**Priority**: Should · **Layer**: 3 · **Size**: S · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view), E04-S2 (zip archives)
**Traces**: F-9.9, F-7.1, F-7.3
**Not in this story**: other archive formats → E04-S3.

As a user building out a target tree, I can create folders, including archives and missing base folders, from the comparison so that I can set up the structure where I'm working.

**Acceptance criteria**
- [ ] With a file or a closed folder selected, *New Folder* creates a sibling. With an open folder selected, it creates the new folder inside it.
- [ ] With both sides selected, the folder is created on both sides.
- [ ] A name with an archive extension, such as `bundle.zip`, creates an empty archive that opens as a folder.
- [ ] Typing a base-folder path that doesn't exist and pressing Enter offers **Create**. Create makes the folder and loads it as the base.

#### Tasks
- **E09-S9-T1** — Placement rules for sibling vs inside, and one vs both sides · size: S · depends: —
  - AC: unit cases cover a file, a closed folder, an open folder and a two-sided selection.
- **E09-S9-T2** — New Folder dialog and creation, including archive extensions · size: S · depends: E09-S9-T1
  - AC: `bundle.zip` becomes a valid empty archive with a zipper icon.
- **E09-S9-T3** — Create offer for a missing base folder in the path edit · size: S · depends: —
  - AC: pressing Enter on a missing path shows Create. Declining leaves the session unchanged.

### E09-S10 — Backups before overwriting
**Priority**: Should · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E09-S1 (copy), E09-S2 (move), E07-S1 (Text Compare save)
**Traces**: F-26.9, F-25.3 · **Open questions**: Q2
**Not in this story**: backups for table and hex saves (they follow the same setting once those views can save; see E15-S5 and E16-S2).

As a cautious user, I can have the product keep a backup of every file it overwrites so that a wrong copy or save can be undone.

**Acceptance criteria**
- [ ] A Backups options page turns backups on and holds the naming, location and retention settings decided in T1.
- [ ] With backups on, a copy or move that overwrites a file first writes a backup of the old file.
- [ ] With backups on, saving an edited file in Text Compare first backs up the version on disk.
- [ ] If the backup can't be written, that file isn't overwritten, and the failure is reported (proposed default, confirmed in T1).
- [ ] The `/nobackups` switch turns backups off for that run only.

#### Tasks
- **E09-S10-T1** — Decide backup naming, location and retention (Q2) · size: S · depends: — · flags: `decision`
  - AC: decision recorded in this story. Proposed default: a timestamped copy in a per-user backups folder, mirroring the original path, keeping the last 5 versions per file, with "next to the original as `<name>.bak`" as the alternative.
- **E09-S10-T2** — Backups options page · size: S · depends: E09-S10-T1
  - AC: settings persist, and the page explains where backups go.
- **E09-S10-T3** — Back up before copy or move overwrites a file · size: M · depends: E09-S10-T1
  - AC: overwriting a file leaves a backup with the old content. With no overwrite, no backup is made.
- **E09-S10-T4** — Back up before saving an edited text file · size: S · depends: E09-S10-T1
  - AC: after saving, the backup holds the pre-save content.
- **E09-S10-T5** — `/nobackups` switch · size: S · depends: E09-S10-T3
  - AC: a run with `/nobackups` overwrites without making backups. The setting itself is unchanged.

### E09-S11 — Operation log panel
**Priority**: Should · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E09-S1 (copy), E09-S2 (move), E09-S3 (delete)
**Traces**: F-26.7, F-26.11
**Not in this story**: the FTP trace category → E21-S2; the log in Folder Sync → E12-S2.

As a user running bulk operations, I can read a log of what was done and what failed so that I can check the result and fix problems.

**Acceptance criteria**
- [ ] *View > Log* shows or hides a log panel in Folder Compare.
- [ ] Categories are *Operation summaries*, *Operation details* (every file affected), *Warnings* (warnings and errors) and *Debug messages*. Each can be turned on or off.
- [ ] Copy, move and delete write a summary (counts, duration), details (each affected file) and warnings (each failure).
- [ ] *Display up to N messages* drops the oldest messages beyond N.
- [ ] *Automatically save messages to file* appends messages to a chosen file, which is created when needed.
- [ ] The tweak *Beep after long file operations* beeps when an operation took longer than 3 seconds.

#### Tasks
- **E09-S11-T1** — Log store with categories and a message cap · size: S · depends: —
  - AC: with N = 3, a fourth message drops the first.
- **E09-S11-T2** — Log panel with category toggles · size: M · depends: E09-S11-T1 · flags: `naming-check`, `accessibility`
  - AC: turning Operation details off hides those lines without losing them.
- **E09-S11-T3** — Copy, move and delete emit summaries, details and warnings · size: S · depends: E09-S11-T1
  - AC: a copy with one failure logs one summary, one detail per file and one warning.
- **E09-S11-T4** — Save messages to a file automatically · size: S · depends: E09-S11-T1
  - AC: the file appears on the first message and grows across sessions.
- **E09-S11-T5** — "Beep after long file operations" tweak · size: S · depends: —
  - AC: a 4-second operation beeps and a 1-second one doesn't.

### E09-S12 — Exchange sides
**Priority**: Could · **Layer**: 2 · **Size**: S · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view), E04-S1 (file-system access layer)
**Traces**: F-9.3

As a user who loaded two folders the wrong way round, I can swap the selected items between the sides so that each version ends up where it belongs.

**Acceptance criteria**
- [ ] *Exchange* (hidden by default) moves the selected items on both sides to the opposite side, swapping paired items.
- [ ] Orphans move across to the other side.
- [ ] A confirmation appears first. Failures are reported, and no item is lost when one move of a pair fails (*inferred*).
- [ ] Rows refresh to show the swapped state.

#### Tasks
- **E09-S12-T1** — Exchange plan that swaps paired items safely (temporary names avoid clashes) · size: S · depends: —
  - AC: when the second move of a swapped pair fails, both files still exist and the failure is reported.
- **E09-S12-T2** — Exchange command, confirmation, run and refresh · size: S · depends: E09-S12-T1
  - AC: after Exchange on a pair, the left and right contents are swapped.
