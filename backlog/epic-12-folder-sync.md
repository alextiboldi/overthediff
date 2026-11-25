# E12 — Folder Sync

**Group**: feature
**Label**: `area:folder-sync`
**Size**: L
**Depends on**: E01, E03, E04 (foundations); E08 (Folder Compare)
**Source**: catalogue §11, F-8.14, F-25.3. Spec-derived from the feature catalogue and not yet validated. Items marked *inferred* are analyst or PO inferences.

Folder Sync is a dedicated session that plans the copies and deletes needed to bring two folders in line with a chosen method, then runs them. The user sets it up once, previews every action before anything changes, overrides single items when needed, and saves the session for reuse.

## Behavioral contract
- **Data**:
  - *Sync session*: left and right folders (any data source), sync method (Update Left, Update Right, Update Both, Mirror to Left, Mirror to Right, or a custom method), rules (Comparison, Handling, Name Filters, Other Filters, Sync and Misc tabs) and per-item overrides.
  - *Sync plan*: one action per item (copy left → right, copy right → left, delete left, delete right, or leave alone) and folder aggregate icons.
  - *Run result*: the outcome of each action.
- **Routes/endpoints** (desktop app, so these are views, commands and switches):
  - The Folder Sync view: setup area above, preview below.
  - Session menu: Run Sync, Folder Sync Info, Compare Base Folders and Compare Parent Folders.
  - Actions menu overrides.
  - Folder Compare's *Sync Base Folders* command.
  - CLI `/sync` (Windows, macOS).
- **Integrations**:
  - E03-S2 folder engine supplies the comparison statuses.
  - E04 data sources copy and delete, including the recycle bin and archives.
  - The E08 comparison and filter tabs are reused.
  - E01-S1 background tasks show progress and cancel.
- **Rules/states**:
  - A sync moves through setup → Accept → preview (plan) → Sync Now → run → refreshed preview.
  - Nothing changes on disk before Sync Now or Run Sync.
  - The plan follows from each item's status and the method. A per-item override beats the method, and an excluded action isn't run.

---

## Stories

### E12-S1 — Folder Sync session: set up, preview and run
**Priority**: Should · **Layer**: 3 · **Size**: L · **Platforms**: All
**Needs**: E08-S3 (name filters), E08-S4 (quick-test criteria), E08-S6 (columns), E08-S7 (selection commands, Copy Filename), E08-S8 (base-folder navigation), E08-S9 (find filenames), E08-S10 (opening pairs in file views), E08-S11 (loading and scanning), E08-S12 (other filters), E04-S2 (zip archives)
**Traces**: F-11.1, F-11.2, F-11.3, F-8.14, F-25.3
**Not in this story**: per-item overrides, Next/Previous Conflict, Suppress Filters, Log, Open With, Ignore Unimportant Differences → E12-S2; custom operations → E12-S3; Windows, FTP, automatic refresh and Misc handling → E12-S4; script `sync` → E20-S4; context-menu Sync → E26-S2.

As an IT administrator, I can set up a folder synchronisation once, preview exactly what it will copy and delete, and run it so that two folders stay in line without manual copying.

**Acceptance criteria**
- [ ] Folder Sync is a session type, offered on Home and under New Session. Its upper part holds the **Left folder** and **Right folder** (typed or browsed, and either can be any data source), a **sync method** dropdown, a **Rules** button and **Filters**.
- [ ] Sync methods:
  - *Update Left*: copies newer and orphan files right → left.
  - *Update Right*: copies newer and orphan files left → right.
  - *Update Both*: copies newer and orphan files both ways.
  - *Mirror to Left*: replaces different items on the left, deletes left orphans and copies right orphans to the left.
  - *Mirror to Right*: the same in the other direction.
- [ ] **Accept** applies a changed setup. The lower part then previews every planned copy and delete.
- [ ] Each preview row shows an action icon: right → left copy, left → right copy, delete left, delete right, or **Leave alone**.
- [ ] Folder rows show aggregate icons: contains files to copy to the left, to copy to the right, to delete on the left, to delete on the right, or a mix of copy and delete.
- [ ] **Sync Now** and *Session > Run Sync* run every planned action, with progress, cancel and an error summary, after a confirmation showing the counts (*inferred*). The preview refreshes afterwards.
- [ ] *Session > Folder Sync Info* shows sync statistics.
- [ ] *Compare Base Folders* and *Compare Parent Folders* open a Folder Compare.
- [ ] Folder Compare gains **Sync Base Folders**, which opens a Folder Sync view on its base folders.
- [ ] Double-clicking a pair opens it in a file view to inspect the differences.
- [ ] The preview supports the Folder Compare tree commands *Expand* / *Collapse*, *Refresh*, *Full Refresh* and *View > Legend*. The Legend also explains the sync action icons.
- [ ] The preview also offers the Folder Compare commands *Set as Base Folder* (and its variants), *Copy Filename*, *Find Filename* and *View > Columns*. They behave as in Folder Compare and keep the plan in step: a new base folder re-plans the sync.
- [ ] Rules tabs:
  - The Comparison tab matches Folder Compare's quick tests, including *Align filenames with different extensions* and *Unicode normalization forms*.
  - The Name Filters and Other Filters tabs match Folder Compare's.
  - The Handling tab has *Sync empty folders* (also creates or deletes folders that are empty or entirely filtered out), *Expand subfolders when loading session*, *Sync within archives* (treats archives as folders), *Delete to recycle bin if possible* (local drives only) and *Follow symbolic links*.
- [ ] Folders, method and rules are part of the session's settings, so they are kept wherever the session is saved.
- [ ] `/sync` (Windows and macOS) opens Folder Sync on the two folders given on the command line.

#### Tasks
- **E12-S1-T1** — Folder Sync session type with the setup area (folders, method dropdown, Rules, Filters, Accept) · size: M · depends: — · flags: `naming-check`
  - AC: Home and New Session offer Folder Sync. The setup accepts any two folder specs, and Accept is needed only after a change (*inferred*).
- **E12-S1-T2** — Sync plan per method · size: M · depends: —
  - AC: a unit table of status × the five methods gives the specified action in every cell. Statuses: same, left newer, right newer, different with no newer side, left orphan, right orphan, empty folder. With *Sync empty folders* on, empty or fully filtered folders get create or delete actions.
- **E12-S1-T3** — Preview with per-row action icons and folder aggregate icons · size: M · depends: E12-S1-T2 · flags: `accessibility`
  - AC: each fixture row shows its planned action icon. A folder holding both copies and deletes shows the mixed icon, and rows with no action show Leave alone.
- **E12-S1-T4** — Sync Now / Run Sync with confirmation, progress, cancel, error summary and a refreshed preview · size: M · depends: E12-S1-T2
  - AC: after Mirror to Right on a fixture, the refreshed preview shows no remaining actions. With *Delete to recycle bin if possible* on, deletes on a local side go to the recycle bin.
- **E12-S1-T5** — Rules tabs reused from Folder Compare, plus the sync Handling options · size: M · depends: E12-S1-T2
  - AC: excluding `*.tmp` removes tmp files from the plan. With *Sync within archives* on, a zip's contents are planned item by item. With *Follow symbolic links* on, links are synced as their targets.
- **E12-S1-T6** — Folder Sync Info, Compare Base Folders, Compare Parent Folders, Sync Base Folders in Folder Compare, double-click to inspect a pair, and the tree commands (Expand/Collapse, Refresh, Full Refresh, Legend) · size: S · depends: E12-S1-T3
  - AC: Sync Base Folders in a Folder Compare opens Folder Sync with the same bases. Double-clicking a planned copy opens that pair in its file view. Full Refresh re-plans after a change on disk, and the Legend lists the action icons.
- **E12-S1-T8** — Set as Base, Copy Filename, Find Filename and Columns in the sync preview · size: S · depends: E12-S1-T3
  - AC: Set as Base Folder on a preview subfolder re-plans the sync for that subfolder. Copy Filename copies the selected rows' paths. Find Filename selects the next matching row. Columns shows and hides the Size and Modified columns.
- **E12-S1-T7** — `/sync` switch on Windows and macOS · size: S · depends: E12-S1-T1
  - AC: `otd /sync A B` opens Folder Sync with A and B. On Linux the switch is reported as unsupported (*inferred*).

### E12-S2 — Per-item overrides and sync review commands
**Priority**: Should · **Layer**: 4 · **Size**: M · **Platforms**: All
**Needs**: E12-S1 (Folder Sync session), E08-S13 (rules-based content comparison), E08-S15 (suppress filters), E08-S16 (Open With), E09-S11 (operation log panel)
**Traces**: F-11.2

As an IT administrator, I can override the planned action for single items and review the plan with the usual folder tools so that exceptions don't force me to change the whole sync setup.

**Acceptance criteria**
- [ ] Actions menu overrides on the selected preview rows: **Leave Alone**, **Copy Right to Left**, **Copy Left to Right**, **Delete Left** and **Delete Right**. The row's icon changes, and the run uses the override.
- [ ] **Exclude** skips the selected operations: they leave the plan, its totals and the run.
- [ ] **Next Conflict** and **Previous Conflict** jump between rows whose action the method can't decide on its own, such as a pair that differs with neither side newer (*inferred* definition).
- [ ] *View > Suppress Filters* temporarily shows the items the sync **won't** affect.
- [ ] *View > Log* shows the operation log for sync runs, with summaries, details and warnings.
- [ ] *Open With* is available on preview rows.
- [ ] *View > Ignore Unimportant Differences* plans pairs with only unimportant (rules-based) differences as same, so they get no action.

#### Tasks
- **E12-S2-T1** — Per-item override actions and Exclude · size: M · depends: —
  - AC: overriding a planned copy to Leave Alone changes its icon, and the run leaves the item untouched. Exclude removes the action from the run and from the totals.
- **E12-S2-T2** — Next/Previous Conflict · size: S · depends: —
  - AC: with Update Both on a fixture containing a pair that differs with equal timestamps, Next Conflict lands on that pair.
- **E12-S2-T3** — Suppress Filters in Folder Sync shows unaffected items · size: S · depends: —
  - AC: turning it on shows the leave-alone and filtered items, and turning it off hides them again.
- **E12-S2-T4** — Log, Open With and Ignore Unimportant Differences in Folder Sync · size: S · depends: —
  - AC: a run writes a summary and one detail line per action to the log. With Ignore Unimportant on, a whitespace-only pair gets no action under Mirror to Right.

### E12-S3 — Custom sync operations
**Priority**: Could · **Layer**: 4 · **Size**: M · **Platforms**: All
**Needs**: E12-S1 (Folder Sync session)
**Traces**: F-11.1 · **Open questions**: Q1

As an automation power user, I can build a custom sync method from per-status actions so that a sync can follow rules the five presets don't cover.

**Acceptance criteria**
- [ ] The Session Settings **Sync** tab has *Operations* and *Presets*. The user can pick a predefined method or build a custom one.
- [ ] A custom method maps each comparison status to an action, following the matrix decided in T1.
- [ ] The preview and the run follow a custom method exactly as they follow a predefined one.
- [ ] A custom method can be saved as a named preset and picked again later (*inferred*).

#### Tasks
- **E12-S3-T1** — Define the custom operation matrix (Q1) · size: M · depends: — · flags: `decision`
  - AC: decision recorded in this story. Proposed default:
    - Rows are statuses: left newer, right newer, different with no newer side, left orphan, right orphan, same.
    - Columns are actions: copy left → right, copy right → left, delete left, delete right, leave alone.
    - Each status gets exactly one action, and the five presets are expressed in this matrix.
- **E12-S3-T2** — Sync tab with the Presets list and the Operations matrix editor · size: M · depends: E12-S3-T1
  - AC: choosing a preset fills the matrix, and changing any cell switches the method to Custom.
- **E12-S3-T3** — Plan and run from the matrix · size: S · depends: E12-S3-T1
  - AC: a custom method that copies left orphans only plans copies for left orphans and nothing else.
- **E12-S3-T4** — Save custom methods as presets · size: S · depends: E12-S3-T2
  - AC: a saved custom preset appears in the method dropdown of new Folder Sync sessions.

### E12-S4 — Advanced sync handling
**Priority**: Could · **Layer**: 5 · **Size**: S · **Platforms**: All
**Needs**: E12-S1 (Folder Sync session), E08-S17 (Windows file details), E08-S20 (automatic refresh), E08-S22 (alignment overrides), E21-S2 (advanced FTP settings)
**Traces**: F-11.1, F-7.4

As an IT administrator syncing to Windows servers or FTP sites, I can carry Windows metadata across, fix timestamps after FTP uploads and keep the plan current so that synced copies are faithful.

**Acceptance criteria**
- [ ] The Folder Sync Handling tab gains *Maintain DOS 8.3 aliases*, *Copy NTFS file permissions* and *Copy creation dates* (Windows), applied to synced copies.
- [ ] The Handling tab gains *Touch local files when copying to an FTP site*: after an upload, the local file's timestamp is set to match the server copy, for servers that can't set timestamps.
- [ ] The Handling tab gains *Automatic refresh*, which re-plans the sync by itself.
- [ ] The **Misc** tab gains alignment override rules that work as in Folder Compare.

#### Tasks
- **E12-S4-T1** — Windows metadata options in synced copies · size: S · depends: — · flags: `licence-check`
  - AC: with each option on, synced copies keep the source's short name, permissions and creation date.
- **E12-S4-T2** — Touch local files when copying to an FTP site · size: S · depends: —
  - AC: after a sync uploads to an FTP server that ignores timestamps, the local file's modified time equals the server copy's.
- **E12-S4-T3** — Automatic refresh and Misc tab alignment overrides in Folder Sync · size: S · depends: — · flags: `licence-check`
  - AC: a file added on disk appears in the preview without user action. An override rule `*.gif` ↔ `*.png` pairs those files in the plan.
