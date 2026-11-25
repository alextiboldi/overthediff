# E13 — Text Merge

**Group**: feature
**Label**: `area:text-merge`
**Size**: L
**Depends on**: E01, E03, E04 (foundations); E07 (Text Compare); E14 (Folder Merge, for Merge Parent Folders)
**Source**: catalogue §13, the 3-way merge concept in §10 and Appendix B. Spec-derived from the feature catalogue and not yet validated. Items marked *inferred* are analyst or PO inferences.

Text Merge combines two changed versions of a file. Using their common ancestor, it takes every non-conflicting change automatically and leaves only real conflicts for the user to decide. It is the file-level merge and the basis of mergetool integration.

## Behavioral contract
- **Data**:
  - *Merge session*: specs Left, Center (optional common ancestor), Right and Merge to (the output file), plus a Description. Each input has a file format and encoding. Importance and alignment settings work as in Text Compare, and the session also stores a conflict definition.
  - *Merge model*: rows aligned across the inputs. Each row is one of unchanged, left change, right change, same change, different change or conflict. Each side's changes are additions, modifications or deletions.
  - *Output*: lines grouped into sections. Each section records its origin (center, left, right or edited) and a conflict flag.
- **Routes/endpoints** (desktop app, so these are commands and entry points):
  - Opening: the Text Merge view from Home, Session > New Session and the CLI with three or four files, and Text Compare's *Merge Files*.
  - Commands: Take, navigation and display-filter commands.
  - Session menu: Save, Save As, Text Merge Info, Compare to Output, Merge Parent Folders, Reload and Recompare.
  - The headless merge switches are covered in E11-S5.
- **Integrations**:
  - E03-S1 text engine compares each side with the center.
  - E07-S1 provides the editor panes.
  - The file-system layer writes the output.
  - Folder Merge opens Text Merge as a child view.
- **Rules/states**:
  - With a center file, each side is compared with the center. A change on one side only is taken automatically. The same change on both sides is never a conflict. Different changes on the same lines, or within N lines once that setting exists, are a conflict.
  - The output is built during loading: the center plus every non-conflicting change.
  - A direct edit in the output marks that section as edited (yellow). Taking a side for the section again replaces the edit.
  - Saving writes the whole output and ignores whatever the output file held before.
  - Input panes are read-only.

---

## Stories

### E13-S1 — Text Merge view with automatic merge
**Priority**: Should · **Layer**: 2 · **Size**: L · **Platforms**: All
**Needs**: E03-S1 (text comparison engine), E07-S1 (Text Compare view)
**Traces**: F-13.1, F-13.2, F-13.3, F-13.5, F-12.10, F-25.2, App-B
**Not in this story**: conflict definition and conflict flags → E13-S2; difference, taken and edit navigation, search and line-level takes → E13-S3; display filters and view options → E13-S4; Align With, Isolate, Format/Importance/Alignment tabs, Compare to Output, Merge Parent Folders, Reload and Recompare → E13-S5; headless merge switches → E11-S5; folder merges → E14-S1.

As a developer resolving a merge, I can open the base and both changed versions in one view that pre-merges every non-conflicting change so that I only have to decide the real conflicts.

**Acceptance criteria**
- [ ] A Text Merge session has Specs for Left, an optional Center (the common ancestor), Right and Merge to (the output file), plus a Description.
- [ ] It opens from Home, from Session > New Session, and from the CLI with three files (left, right, center) or four (plus the output).
- [ ] *Merge Files* in Text Compare opens a Text Merge on the two files that Text Compare has loaded.
- [ ] Input panes are read-only. The output pane is editable.
- [ ] With a center file, each side is compared with the center, and changes are classed as additions, modifications or deletions per side. Left changes are teal, right changes magenta and conflicts red.
- [ ] Different changes on the same lines are a conflict. The same change made on both sides is not.
- [ ] On load, the output starts equal to the center, with every non-conflicting change already taken. Each output section is coloured by the pane it came from, and a conflict section shows the center text.
- [ ] *Take Left*, *Take Center* and *Take Right* replace the selected lines or the current section of the output. They are available on the toolbar, in each pane's popup menu, and as inline buttons beside each output section.
- [ ] Typing in the output turns that section yellow. Taking a side for the same section replaces the edit.
- [ ] *Next Conflict Section* and *Previous Conflict Section* move between conflicts and keep all panes aligned.
- [ ] *Save* and *Save As* write the output. The first save warns that the output file's existing content is ignored and overwritten.
- [ ] *Text Merge Info* shows merge statistics: counts of left, right, same and different changes and of conflicts (*inferred* fields). *Swap Sides* exchanges the left and right inputs.
- [ ] Without a center file, the view still opens: left and right are compared directly, and every difference is flagged as a conflict for manual choice (*inferred*).

#### Tasks
- **E13-S1-T1** — 3-way merge model that classifies each row against the center · size: L · depends: — · flags: `craft-libs`
  - AC: unit cases cover a left-only change, a right-only change, the same change on both sides, different changes on the same lines, additions and deletions on each side, and the case without a center.
- **E13-S1-T2** — Build the output automatically and record each section's origin · size: M · depends: E13-S1-T1
  - AC: for a fixture with two non-conflicting changes and one conflict, the output equals the center plus both changes, and the conflict section holds the center text with its conflict flag set.
- **E13-S1-T3** — Merge view: read-only inputs, editable output, colours and synchronised scrolling · size: M · depends: E13-S1-T1 · flags: `naming-check`, `accessibility`
  - AC: teal, magenta and red mark left changes, right changes and conflicts. Typing in an input pane is refused, and scrolling one pane scrolls them all.
- **E13-S1-T4** — Take Left/Center/Right from the toolbar, popup menus and inline buttons; edited sections · size: M · depends: E13-S1-T2, E13-S1-T3
  - AC: each take replaces the current section with that side's lines. Typing turns a section yellow, and a later take for that section replaces the edit.
- **E13-S1-T5** — Next and Previous Conflict Section · size: S · depends: E13-S1-T3
  - AC: starting from the top, Next Conflict Section visits every conflict once, in order, with all panes aligned.
- **E13-S1-T6** — Session specs, entry points (Home, New Session, CLI with three or four files, Merge Files), Save/Save As with the overwrite warning, Info, Swap Sides · size: M · depends: E13-S1-T2
  - AC: launching with four files opens the view with the output path set. After the warning, saving replaces an output file that already held other content.

### E13-S2 — Conflict definition and conflict flags
**Priority**: Should · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E13-S1 (Text Merge view)
**Traces**: F-13.1, F-13.2

As a code reviewer merging nearby edits, I can set how close two changes must be to count as a conflict, and flag or clear conflicts by hand, so that the merge matches my judgement.

**Acceptance criteria**
- [ ] The Text Merge Alignment tab has a setting "A merge conflict is", with two choices: *Only different change lines*, or *Left change and right change separated by ≤ N lines* with N editable.
- [ ] With the second choice, a left change and a right change within N lines of each other form one conflict even when they touch different lines. Changes further apart merge automatically.
- [ ] Under either choice, the same change made on both sides is never a conflict.
- [ ] Changing the setting rebuilds the merge with the new set of conflicts. If the output has unsaved edits, the user is asked first (*inferred*).
- [ ] The *Conflict* command sets or clears the conflict flag on the selected lines or the current section.
- [ ] *Clear Conflict Section, Next* clears the current conflict and moves to the next one.

#### Tasks
- **E13-S2-T1** — Proximity rule for conflicts in the merge model · size: M · depends: —
  - AC: with N = 2, changes 2 lines apart form one conflict and changes 3 lines apart merge automatically. Identical changes never conflict.
- **E13-S2-T2** — "A merge conflict is" setting on the Text Merge Alignment tab, with a rebuild on change · size: S · depends: E13-S2-T1
  - AC: switching the setting changes the conflict count in Text Merge Info without reopening the files.
- **E13-S2-T3** — Conflict command and Clear Conflict Section, Next · size: S · depends: —
  - AC: flagging a non-conflicting section turns it red, and conflict navigation stops there. Clear Conflict Section, Next clears the flag and moves to the next conflict.

### E13-S3 — Merge navigation, search and line-level takes
**Priority**: Should · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E13-S1 (Text Merge view), E07-S5 (find and replace), E07-S6 (go to and bookmarks)
**Traces**: F-13.3, F-13.2

As a developer working through a long merge, I can jump between differences, takes and edits, search the files and take single lines so that I can finish the merge quickly and precisely.

**Acceptance criteria**
- [ ] *Next Difference* and *Previous Difference*, and *Next Difference Section* and *Previous Difference Section*, move between differences in the merge.
- [ ] *Next Left Taken* and *Previous Left Taken*, and *Next Right Taken* and *Previous Right Taken*, move between output sections taken from that side.
- [ ] *Next Edit* and *Previous Edit* move between output sections the user edited directly.
- [ ] Every navigation command keeps the input panes and the output pane on the same row.
- [ ] *Find*, *Find Next/Previous*, *Replace* (in the output pane), *Go To* (line and column) and *Bookmarks 0–9* work in the merge view, with the same options as in Text Compare.
- [ ] *Take Left Then Right* and *Take Right Then Left* put both sides' lines into the output, in the order named.
- [ ] *Take Left Line*, *Take Center Line* and *Take Right Line* take only the current line.

#### Tasks
- **E13-S3-T1** — Difference, section, taken and edit navigation with the panes kept in step · size: M · depends: —
  - AC: on a fixture with two left takes and one edit, Next Left Taken visits two sections and Next Edit visits one. All panes stay aligned throughout.
- **E13-S3-T2** — Find, Replace, Go To and Bookmarks in the merge view · size: M · depends: —
  - AC: Replace changes text in the output pane only. Go To line 50 moves every pane to that row. Bookmark 3, set in the output, takes the user back there.
- **E13-S3-T3** — Take Left Then Right, Take Right Then Left and the single-line takes · size: S · depends: —
  - AC: on a conflict, Take Left Then Right puts the left lines followed by the right lines into the output. Take Right Line replaces only the current line.

### E13-S4 — Merge display filters and view options
**Priority**: Should · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E13-S1 (Text Merge view), E07-S3 (display filters and context lines), E07-S9 (whitespace, wrap, layout, font size), E07-S11 (thumbnail), E07-S12 (details panels)
**Traces**: F-13.4, F-26.11
**Not in this story**: Ignore Unimportant Differences in merges → E13-S5.

As a code reviewer checking a merge, I can filter what the merge view shows and arrange its panes so that I can focus on the conflicts or on one side's changes.

**Acceptance criteria**
- [ ] The display filters show these line classes:

  | Filter | Shows |
  |---|---|
  | *Show All* | unchanged, same change, left change, right change, different change and conflict lines |
  | *Show Changes* | all changes and conflicts |
  | *Show Conflicts* | conflicts only |
  | *Show Left Changes* | same, left and different changes, and conflicts |
  | *Show Right Changes* | same, right and different changes, and conflicts |
  | *Show Mergeable* | same, left, right and different changes, without conflicts |
  | *Show Unchanged* | unchanged lines only |
  | *Show None* (hidden by default) | nothing |
  | *Show Context* | changes plus the number of context lines set in the Text Editing options |
- [ ] *Ignore Same Changes* treats the same change made on both sides as unchanged.
- [ ] *Favor Left Changes* and *Favor Right Changes* stop the output from highlighting that side's changes.
- [ ] *Ignored* suppresses differences for the selected lines or the current section, and can be undone.
- [ ] *Center Pane* shows or hides the center input.
- [ ] *Detached Output Pane* moves the output into a window of its own, for example on a second monitor, and moves it back.
- [ ] The tweak *Use different font for merge input panes* gives the input panes their own font.

- [ ] The merge panes offer the Text Compare display options: *Visible Whitespace*, *Word Wrap*, *Line Numbers*, *Thumbnail*, *Text Details* and *Hex Details*, plus font size increase/decrease/reset. *Show Context* uses the *Number of context lines* option.
#### Tasks
- **E13-S4-T1** — Merge display filters · size: M · depends: — · flags: `naming-check`
  - AC: on a fixture with one line of each class, every filter shows exactly the lines the table lists.
- **E13-S4-T2** — Ignore Same Changes, Favor Left/Right and Ignored · size: M · depends: —
  - AC: the same change made on both sides doesn't appear under Show Changes while Ignore Same Changes is on. Lines marked Ignored also drop out of Show Changes.
- **E13-S4-T3** — Center Pane toggle and Detached Output Pane · size: M · depends: —
  - AC: hiding the center widens the left and right panes. The detached output window edits the same output and docks again on command.
- **E13-S4-T4** — Tweak for a different input-pane font · size: S · depends: —
  - AC: setting the tweak changes the input panes' font and leaves the output pane's font unchanged.
- **E13-S4-T5** — Text Compare display options in the merge panes · size: M · depends: —
  - AC: *Word Wrap* wraps the input panes and the output. *Visible Whitespace* shows glyphs in all panes. The thumbnail shows merge colours. *Text Details* shows the current line of every pane.

### E13-S5 — Merge alignment, settings and session commands
**Priority**: Should · **Layer**: 4 · **Size**: M · **Platforms**: All
**Needs**: E13-S1 (Text Merge view), E07-S4 (importance rules), E07-S13 (alignment algorithms), E07-S14 (manual alignment), E14-S1 (Folder Merge view), E07-S7 (Select Section), E07-S8 (editing commands), E07-S10 (Format tab), E07-S18 (Compare Selection to Clipboard), E13-S4 (merge display filters and toggles), E11-S5 (headless automatic merge), E06-S2 (session defaults, the rules a headless merge uses)
**Traces**: F-13.2, F-13.5, F-13.1

As a developer with an awkward merge, I can realign lines, tune the comparison settings and jump to related comparisons so that the merge lines up with how the code actually changed.

**Acceptance criteria**
- [ ] *Align With* works in the merge view for one line or for a multi-line selection: select the lines, then click the line in another pane to align them with.
- [ ] *Isolate* realigns so that the selected lines stand on their own.
- [ ] Session Settings for Text Merge has three tabs and no Replacements tab:
  - a Format tab: file format and encoding for each input, defaulting to "Detected" and "None";
  - an Importance tab with the same settings as Text Compare's;
  - an Alignment tab with the same algorithm options as Text Compare's.
- [ ] Changing these settings merges the inputs again. If the output has unsaved edits, the user is asked first (*inferred*).
- [ ] *Compare to Output* (Left, Center or Right) opens a Text Compare of that input against the current output.
- [ ] *Merge Parent Folders* opens a Folder Merge on the parent folders of the inputs.
- [ ] *Reload* re-reads the inputs and prompts first if the output has unsaved edits. *Recompare* merges again without re-reading (*inferred* distinction, following Text Compare).

- [ ] The output pane offers Text Compare's editing commands: *Increase/Decrease Indent*, the hidden-by-default line and word deletes and inserts, *Select Section*, and *Compare Selection to Clipboard*.
- [ ] *View > Ignore Unimportant Differences* treats differences that the merge session's importance rules class as unimportant as unchanged, in the panes and in the display filters.
- [ ] `/iu` (`/ignoreunimportant`) with `/automerge` turns on Ignore Unimportant Differences for the headless merge. Differences the session's importance rules class as unimportant count as unchanged (*inferred* effect on conflicts).
- [ ] `/favorleft` and `/favorright` draw non-conflicting changes from the favoured side without colouring them. For ignored unimportant conflicts, they take the favoured side instead of the center.
#### Tasks
- **E13-S5-T1** — Align With (one or more lines) and Isolate in the merge view · size: M · depends: —
  - AC: aligning a moved block of three lines with its new position makes the block compare as unchanged.
- **E13-S5-T2** — Format, Importance and Alignment tabs for Text Merge sessions · size: M · depends: —
  - AC: an encoding override on the center reloads it with that encoding. Marking whitespace unimportant changes which rows count as changes. No Replacements tab appears.
- **E13-S5-T3** — Compare to Output and Merge Parent Folders · size: S · depends: —
  - AC: Compare to Output (Right) opens a Text Compare of the right input against the output, including unsaved output edits (*inferred*). Merge Parent Folders opens a Folder Merge on the three parent folders.
- **E13-S5-T4** — Reload and Recompare · size: S · depends: —
  - AC: Reload with unsaved output edits asks first. Recompare keeps the loaded inputs and rebuilds the merge.
- **E13-S5-T5** — Editing commands in the output pane · size: S · depends: —
  - AC: *Increase Indent* on three output lines indents them and marks them as edited (yellow). *Compare Selection to Clipboard* opens a Text Compare of the selected output text against the clipboard.
- **E13-S5-T6** — Ignore Unimportant Differences in the merge view · size: S · depends: E13-S5-T2
  - AC: with whitespace marked unimportant on the Importance tab, a change that is only whitespace doesn't appear under Show Changes.
- **E13-S5-T7** — `/iu`, `/favorleft` and `/favorright` for headless merges · size: S · depends: E13-S5-T2
  - AC: with `/favorleft`, an ignored unimportant conflict takes the left side. With `/iu` and whitespace marked unimportant in the Text Merge session defaults, a whitespace-only conflict no longer blocks the merge.
