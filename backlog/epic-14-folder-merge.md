# E14 — Folder Merge

**Group**: feature
**Label**: `area:folder-merge`
**Size**: L
**Depends on**: E01, E03, E04 (foundations); E08 (Folder Compare); E09 (folder file operations); E13 (Text Merge); E22 (folder colours)
**Source**: catalogue §10, including the 3-way merge concept and Appendix B. Spec-derived from the feature catalogue and not yet validated. Items marked *inferred* are analyst or PO inferences.

Folder Merge brings the 3-way idea to whole folder trees. With a common ancestor, it can tell an addition on one side from a deletion on the other, merge independent changes automatically and flag the rest. It is the tool for combining two diverged copies of a project into an output folder.

## Behavioral contract
- **Data**:
  - *Folder Merge session*: Left, Center (optional common ancestor), Right, Merge to (the output folder) and a Description. Its Comparison, Handling, Name Filters and Other Filters settings work as in Folder Compare, except that it has no alignment by extension or by Unicode normalization form.
  - *Row status*: unknown or older, same, left change, right change, mergeable (same change or difference change), or conflict.
  - *Merge action per row*: No action required, Automatic merge or Manual merge required. The remaining actions depend on the output:
    - merging to the left: Copy to left, Delete left;
    - merging to the right: Copy to right, Delete right;
    - merging to a separate output: Take left, Take center, Take right, Delete.
- **Routes/endpoints** (desktop app, so these are commands and entry points):
  - Opening: Home, Session > New Session, and *Merge Base Folders* in Folder Compare.
  - Actions menu: Merge, Copy to Output and the file operations.
  - View menu: merge display filters, folder display modes, Center Pane and Legend.
  - Session menu: Folder Merge Info, Compare to Output and Merge Parent Folders.
- **Integrations**:
  - E03-S2 folder engine scans every input.
  - E04 data sources provide the inputs and the output.
  - E09 provides the file operations.
  - E13 Text Merge is the child view for a file's three versions.
  - The E22-S2 Folder Views colour page holds the Merge colors.
- **Rules/states**:
  - The ancestor resolves ambiguity. An item present on one side only is an addition there if the ancestor lacks it, or a deletion on the other side if the ancestor has it.
  - Independent changes merge automatically. The same change made on both sides is mergeable. Different changes to the same item are a conflict.
  - Which merge action a row shows depends on where the output is.
  - Without a center, the ambiguity can't be resolved, and every differing item needs a manual merge (*inferred*).

---

## Stories

### E14-S1 — Folder Merge view with 3-way status
**Priority**: Should · **Layer**: 3 · **Size**: L · **Platforms**: All
**Needs**: E08-S2 (display filters and folder display modes), E08-S3 (name filters), E08-S4 (quick-test criteria), E08-S5 (content comparison), E08-S11 (loading and scanning), E08-S12 (other filters), E22-S2 (folder view colours and fonts)
**Traces**: F-10.1, F-10.2, F-10.3, F-8.14, F-26.4
**Not in this story**: Merge, Copy to Output, file operations, Compare to Output, Merge Parent Folders, child Text Merge sessions and Ignore Unimportant Differences → E14-S2; Ignored → E08-S21; per-session enabled formats (Misc tab) → E10-S8; Source Control submenu → E26-S4.

As a team lead combining two diverged branches of a project, I can see every item's 3-way status against the common ancestor so that I know what will merge automatically and what needs a decision.

**Acceptance criteria**
- [ ] A Folder Merge session has Specs for Left, an optional Center (the common ancestor), Right and Merge to (the output folder), plus a Description.
- [ ] A Folder Merge opens from Home and from Session > New Session. *Merge Base Folders* in Folder Compare opens one on that comparison's two base folders.
- [ ] Rows use these default colours, and *View > Legend* explains them: gray for unknown or older, black for same, teal for a left change, magenta for a right change, orange for mergeable and red for a conflict.
- [ ] With a center, an item present on one side only is classed as added on that side or deleted on the other, depending on whether the ancestor has it.
- [ ] Each row shows a merge-action icon: *No action required*, *Automatic merge* or *Manual merge required*. Depending on the output, the icon can also be:
  - with the output set to the left folder: *Copy to left* or *Delete left*;
  - with the output set to the right folder: *Copy to right* or *Delete right*;
  - with a separate output: *Take left*, *Take center*, *Take right* or *Delete*.
- [ ] The merge display filters are:
  - *Show All*;
  - *Show Changes* (left and right adds, deletes and changes, and conflicts);
  - *Show Conflicts*, *Show Left Changes* and *Show Right Changes*;
  - *Show Mergeable* (same-change and difference-change items);
  - *Show Unchanged*;
  - *Show None* (hidden by default).
- [ ] *Ignore Same Changes* treats the same change made on both sides as same.
- [ ] The folder display modes *Always Show Folders*, *Compare Files and Folder Structure*, *Compare Files Only* and *Ignore Folder Structure* work as in Folder Compare.
- [ ] *Center Pane* hides or shows the center input.
- [ ] Session Settings has Comparison, Handling, Name Filters and Other Filters tabs that work as in Folder Compare. The Comparison tab has no *Align filenames with different extensions* or *Align filenames with different Unicode normalization forms*.
- [ ] The Folder Views colour page has *Merge colors*, one per merge status.
- [ ] *Folder Merge Info* shows merge statistics.

#### Tasks
- **E14-S1-T1** — 3-way folder status computed from three scans · size: L · depends: —
  - AC: unit cases cover add left, add right, delete left, delete right, change left, change right, same change and different change, plus the no-center case, in which every differing item needs a manual merge (*inferred*).
- **E14-S1-T2** — Merge action per row for left, right and separate outputs · size: M · depends: E14-S1-T1
  - AC: for each status in a fixed table, the row shows the specified action icon under each of the three output targets.
- **E14-S1-T3** — Folder Merge view with three input panes, colours, legend, Center Pane toggle and Info · size: M · depends: E14-S1-T1 · flags: `naming-check`, `accessibility`
  - AC: a fixture tree shows each status in its colour. With the center hidden, the left and right panes stay aligned.
- **E14-S1-T4** — Merge display filters, Ignore Same Changes and the folder display modes · size: M · depends: E14-S1-T1
  - AC: on the fixture tree, each filter shows exactly the statuses the acceptance criteria list.
- **E14-S1-T5** — Session settings tabs taken from Folder Compare, without extension and Unicode alignment · size: S · depends: —
  - AC: a name filter excluding `*.log` hides log files in every pane. Neither alignment option appears on the Comparison tab.
- **E14-S1-T6** — Merge colors on the Folder Views page, and Merge Base Folders in Folder Compare · size: S · depends: E14-S1-T3
  - AC: changing the conflict colour recolours every conflict row. Merge Base Folders opens a Folder Merge on the two base folders, with no center set.

### E14-S2 — Folder Merge actions
**Priority**: Should · **Layer**: 4 · **Size**: L · **Platforms**: All
**Needs**: E14-S1 (Folder Merge view), E09-S1 (copy), E09-S3 (delete), E09-S6 (rename), E09-S7 (attributes), E09-S8 (touch), E09-S9 (new folder), E13-S1 (Text Merge view), E08-S13 (rules-based content comparison), E08-S10 (open file pairs as child sessions), E08-S3 (Exclude), E08-S7 (Copy Filename), E11-S1 (/fv view-type switch), E11-S5 (merge switches incl. /mergeoutput)
**Traces**: F-10.3, F-6.4
**Not in this story**: Ignored → E08-S21; Source Control submenu → E26-S4.

As a team lead, I can merge selected items automatically, copy them to the output and resolve file-level conflicts in Text Merge so that the output folder ends up holding the combined result.

**Acceptance criteria**
- [ ] *Merge* merges the selected items into the output automatically. A selected folder means its visible contents.
- [ ] A confirmation precedes Merge unless the "Confirm file merge" option turns it off.
- [ ] A file changed on both sides is merged line by line when that merge has no conflicts. Otherwise it is left for a manual merge and listed in the summary (*inferred*).
- [ ] *Copy to Output* copies the selected items to the output folder.
- [ ] These operations behave as in Folder Compare: *Copy to Left/Right/Side/Other Side*, *Delete*, *Rename*, *Attributes*, *Touch*, *Exclude*, *New Folder*, *Copy Filename* and *Refresh Selection*.
- [ ] *Move*, *Exchange* and *Copy/Move to Folder* aren't offered.
- [ ] *Browse for Folder* and *Up One Level* work on Left, Center, Right, Output or All Sides.
- [ ] *Set as Base Folders* makes two selected folders the left and right inputs, or three selected folders the left, right and center inputs.
- [ ] *Compare to Output* (Left, Center or Right) opens a Folder Compare of that input against the output folder.
- [ ] *Merge Parent Folders* opens a new Folder Merge on the parent folders.
- [ ] Opening a file row opens a Text Merge child session on that file's left, center and right versions, with the output file set.
- [ ] In that child session, *Take Left File and Open Next Difference* and *Take Right File and Open Next Difference* take the whole file from that side and open the next file with differences.
- [ ] *View > Ignore Unimportant Differences* makes Folder Merge treat rules-based unimportant differences as same.

- [ ] `/fv="Folder Merge"` with left, right and optional center folders opens a Folder Merge, and `/mergeoutput=<folder>` sets its output folder (*inferred*).
#### Tasks
- **E14-S2-T1** — Merge action: plan and apply automatic merges for the selection · size: L · depends: —
  - AC: on a fixture with left-only, right-only and two-sided changes, Merge produces the expected output files. An item that needs a manual merge is reported and left unmerged.
- **E14-S2-T2** — Copy to Output, and the Folder Compare operations available in Folder Merge · size: M · depends: —
  - AC: every listed operation works on the fixture. Move, Exchange and Copy/Move to Folder don't appear in any menu.
- **E14-S2-T3** — Browse for Folder and Up One Level across four sides; Set as Base Folders for two or three folders · size: S · depends: —
  - AC: Up One Level (All Sides) moves all four sides up a level. Setting three selected folders as base makes them the left, right and center inputs.
- **E14-S2-T4** — Compare to Output and Merge Parent Folders · size: S · depends: —
  - AC: Compare to Output (Center) opens a Folder Compare of the center against the output. Merge Parent Folders opens a Folder Merge one level up.
- **E14-S2-T5** — Text Merge child sessions with take-file-and-open-next · size: M · depends: —
  - AC: opening a conflicting file opens Text Merge with all three inputs and the output path. Take Right File and Open Next Difference writes the right file to the output and opens the next differing file.
- **E14-S2-T6** — Ignore Unimportant Differences in Folder Merge · size: S · depends: —
  - AC: with rules-based content comparison on, a file whose only changes are unimportant shows as same while the toggle is on.
- **E14-S2-T7** — Open Folder Merge from the command line with an output folder · size: S · depends: —
  - AC: `otd /fv="Folder Merge" L R C /mergeoutput=OUT` opens Folder Merge with L, R and C as inputs and OUT as the Merge to folder (*inferred* argument order, as for Text Merge).
