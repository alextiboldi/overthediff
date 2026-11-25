# E22 — Preferences and customisation

**Group**: feature
**Label**: `area:customisation`
**Size**: L
**Depends on**: E01 (foundation); E07, E08, E10, E13, E16
**Source**: catalogue §26 (F-26.4, F-26.5, F-26.8, F-26.11, F-26.13), F-5.3, F-7.1, F-8.1, F-8.15, F-12.10, Appendix B. Spec-derived from the feature catalogue and not yet validated. Items marked *inferred* are analyst or PO inferences.

This epic lets each user fit the product to how they work. It covers which commands appear where and which keys run them, the colours and fonts of folder and file views, external applications launched through Open With, and small interface and performance tweaks.

## Behavioral contract
- **Data**:
  - *Command customisation*: for each view type and command, whether it shows in the menu, whether it shows on the toolbar, and its shortcut (if any). Every command also keeps a factory default.
  - *Folder view colours*: font (system or custom); selection colour (medium blue, or the system colour); one colour per compare status; Filtered out items; Unknown; Use stripes.
  - *File view colours*: an element list, where each element has Text color, Background color and, for some, Style. Default inherits, and lower elements override higher ones. Also Editor Font, Hex Font and Listing Font.
  - *Open With item*: Description, Shortcut, Command line, Working folder, Path delimiter, Accepts files, Accepts folders, Refresh when finished, Multiple instances, Wait for previous instance to finish.
  - *Tweaks*:
    - hints: Show hints for toolbar buttons, Show keyboard shortcuts in hints, and the hint show and hide delays
    - views: Esc closes file views, Single vertical scrollbar, Sticky splitter position
    - Max most recently used items
    - comparison: Comparison priority, Buffer size for binary compare
- **Routes/endpoints**:
  - Options pages: menu, toolbar and shortcut customisation per view type; Folder Views colours and fonts; File Views colours and fonts; Open With; Tweaks.
  - *Toolbar Options* on the toolbar right-click menu.
  - The *Open With* submenu in folder and file views.
- **Integrations**:
  - E01-S1 command registry and Options dialog.
  - The active theme (E05) as the base palette.
  - Folder and file views, which read the colours and fonts.
  - The operating system, which launches Open With applications.
- **Rules/states**:
  - Every view type has its own menus, toolbars and shortcuts.
  - Assigning a shortcut that is already in use within the view type raises a conflict warning, which offers to reassign it.
  - Lower colour elements override higher ones, and Default inherits.
  - An Open With variable without a number means the first file. The exception is a second `%f` or `%l`, which means the second file.
  - A folder view refreshes after an Open With application exits only when that item has Refresh when finished.

---

## Stories

### E22-S1 — Customise menus, toolbars and shortcuts
**Priority**: Should · **Layer**: 1 · **Size**: L · **Platforms**: All
**Needs**: E01-S1 (command registry and Options dialog)
**Traces**: F-26.8, F-5.3
**Not in this story**: display-filter modes and large buttons → E05-S4.

As a power user, I can choose which commands appear on each view's menus and toolbar and which keys run them so that the tool fits my habits.

**Acceptance criteria**
- [ ] The menu, toolbar and shortcut customisation page has a view-type picker. It lists every view type with its own menus and toolbars, including Home.
- [ ] A *Search* box filters the command list by name or description, and selecting a command shows its description.
- [ ] *Menu* and *Toolbar* checkbox columns set where each command appears. Commands that are hidden by default can be added this way.
- [ ] The *Shortcut* field records the key combination the user presses. *Remove* clears the shortcut, and *Default* restores the factory shortcut and placement.
- [ ] Assigning a shortcut that another command in the same view type already uses shows a conflict warning offering to reassign it. Declining leaves both commands unchanged.
- [ ] Right-clicking a toolbar offers *Toolbar Options*, which opens this page for the current view type.
- [ ] Changes apply to open views without a restart and persist across restarts.

#### Tasks
- **E22-S1-T1** — Customisation page with view-type picker, command list, search and description · size: M · depends: — · flags: `craft-libs`
  - AC: searching "copy" lists only commands whose name or description contains it. The selected command's description shows below the list.
- **E22-S1-T2** — Menu and Toolbar placement checkboxes per command · size: M · depends: E22-S1-T1
  - AC: ticking Toolbar for a command that is hidden by default adds its button to that view type's toolbar only.
- **E22-S1-T3** — Shortcut capture, Remove and Default · size: S · depends: E22-S1-T1 · flags: `accessibility`
  - AC: pressing Ctrl+Shift+K stores that shortcut, and the shortcut then runs the command. Default restores the factory shortcut.
- **E22-S1-T4** — Shortcut conflict warning with reassignment · size: S · depends: E22-S1-T3
  - AC: assigning a shortcut used by another command in the same view type shows a warning. Accepting moves the shortcut, and declining changes nothing. The same shortcut in a different view type gives no warning.
- **E22-S1-T5** — *Toolbar Options* on the toolbar right-click menu · size: S · depends: E22-S1-T1
  - AC: the command opens the customisation page with the current view type selected.
- **E22-S1-T6** — Apply customisations live and keep them across restarts · size: S · depends: E22-S1-T2, E22-S1-T3
  - AC: a placement change shows in open views at once and still applies after a restart.

### E22-S2 — Folder view colours and fonts
**Priority**: Should · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view)
**Traces**: F-26.4, F-8.1
**Not in this story**: Merge colors → E14-S1.

As a designer with a colour-vision deficiency, I can change the colours and font of folder views so that I can tell the comparison states apart.

**Acceptance criteria**
- [ ] On the Folder Views colours page, the font can be *Use system font* or a custom font and size.
- [ ] *Selection* defaults to medium blue, so difference colours stay readable on selected rows. *Use system* switches to the OS selection colour. Selected rows then lose their difference colour, but the colour spot still shows it.
- [ ] *Compare colors* sets a colour for each comparison status the folder view shows: unknown or older, same, orphan, and newer or different. It also sets *Filtered out items* and *Unknown*.
- [ ] *Use stripes* tints every other row.
- [ ] Changes apply to open folder views and to the Legend without a restart. They sit on top of the active theme and persist.

#### Tasks
- **E22-S2-T1** — Folder Views colours page: font, selection, compare colours, Filtered out items, Unknown and stripes · size: M · depends: — · flags: `accessibility`
  - AC: every setting on the page persists across restarts.
- **E22-S2-T2** — Folder views draw with the chosen font and colours · size: M · depends: E22-S2-T1
  - AC: choosing green for orphans immediately recolours orphan rows, their colour spots and the Legend in open views.
- **E22-S2-T3** — Medium-blue default selection versus system selection · size: S · depends: E22-S2-T2
  - AC: with the default, a selected red row stays readable as red text on blue. With Use system, the row takes the OS colours and its colour spot still shows red.
- **E22-S2-T4** — Use stripes · size: S · depends: E22-S2-T2
  - AC: with stripes on, alternate rows are tinted in both themes. With stripes off, all rows look the same.

### E22-S3 — File view colours and fonts
**Priority**: Should · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E07-S4 (importance rules), E10-S2 (syntax highlighting), E13-S1 (Text Merge view), E16-S1 (Hex Compare view), E15-S1 (Table Compare view, Editor Font), E17-S1 (Picture Compare view, Listing Font), E07-S12 (Hex Details panel)
**Traces**: F-26.5, App-B

As a developer who reads diffs for hours, I can tune the colours and fonts of file views so that differences and syntax stand out the way I like.

**Acceptance criteria**
- [ ] The File Views colours page lists elements. Each has a *Text color* and a *Background color*, and some also have a *Style* (bold, italic). *Default* inherits, and elements lower in the list override higher ones.
- [ ] *Important difference* and *Unimportant difference* each have two settings: a *Background* that tints the whole line, and a *Text background* that highlights the exact changed characters.
- [ ] The list includes *Selection*, every syntax-highlighting element, and the Text Merge colours for left change, right change, conflict and manually edited output.
- [ ] *Use stripes* tints alternate lines in text-based file views (*inferred* scope).
- [ ] Three fonts can be set. *Editor Font* covers Text Compare, Text Merge and Table Compare. *Hex Font* covers Hex Compare and the hex details panels. *Listing Font* covers every other file view.
- [ ] Changes apply to open views without a restart. They sit on top of the active theme and persist.

#### Tasks
- **E22-S3-T1** — Element list with Text color, Background color, Style and Default inheritance · size: M · depends: — · flags: `accessibility`
  - AC: setting the Comment element to italic green shows comments in italic green in Text Compare. Setting an element back to Default makes it inherit from the element above.
- **E22-S3-T2** — Background and Text background for Important and Unimportant differences · size: S · depends: E22-S3-T1
  - AC: changing the Text background of Unimportant difference recolours only the changed characters of unimportant differences.
- **E22-S3-T3** — Text Merge colours for left change, right change, conflict and edited output · size: S · depends: E22-S3-T1
  - AC: changing the conflict colour recolours conflict sections in open Text Merge views.
- **E22-S3-T4** — Editor, Hex and Listing fonts · size: S · depends: —
  - AC: a new Editor Font shows in Text Compare and Text Merge. A new Hex Font shows only in Hex Compare.
- **E22-S3-T5** — Use stripes in file views · size: S · depends: E22-S3-T1
  - AC: with stripes on, alternate lines in Text Compare are tinted.

### E22-S4 — Open With applications
**Priority**: Should · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view), E07-S1 (Text Compare view), E15-S1 (Table Compare view), E16-S1 (Hex Compare view), E17-S1 (Picture Compare view)
**Traces**: F-26.13, F-8.15, F-12.10

As a developer, I can launch my own tools on the selected files straight from a comparison so that I can open them in my editor or run a script without leaving the tool.

**Acceptance criteria**
- [ ] The Open With options page holds a list of items. Each item has *Description* (its menu caption), *Shortcut*, *Command line*, *Working folder*, *Path delimiter*, *Accepts files*, *Accepts folders*, *Refresh when finished*, *Multiple instances* and *Wait for previous instance to finish*.
- [ ] Items appear under *Open With* in Folder Compare for the item kinds they accept (files, folders) and in the Text Compare File menu. Each item's shortcut launches it.
- [ ] Command-line variables expand as follows. Appending `1` or `2` to a variable refers to the left (first) or right (second) file, for example `tool %F2 %F1`.

  | Variable | Expands to |
  |---|---|
  | `%f` | absolute path and filename |
  | `%l` | current line number in a file view |
  | `%n` | filename only |
  | `%p` | absolute path |
  | `%x` | extension, including the dot |
  | `%b` | filename without extension |
  | `%F` | path and filename relative to the working folder |
  | `%P` | path relative to the working folder |

- [ ] Without a number, a variable means the first file. The exception is a second `%f` or `%l`, which means the second file.
- [ ] *Working folder* left blank uses the app's working folder. It can also be a specific folder, `<parent folder>` or `<base folder>`.
- [ ] *Path delimiter* replaces the path separators in expanded paths with the given string.
- [ ] *Multiple instances* runs the command once per selected item, or once per pair if the command line uses a "2" variable. *Wait for previous instance to finish* runs those instances one after another.
- [ ] Folder Compare refreshes after the application exits only if *Refresh when finished* is set. Otherwise, changes the application makes aren't picked up automatically.

- [ ] Open With items also appear in the File menu of Table Compare, Hex Compare and Picture Compare. In those views, `1` and `2` variables refer to the left and right files.
#### Tasks
- **E22-S4-T1** — Open With item list and item editor on the options page · size: S · depends: — · flags: `naming-check`
  - AC: items, with every field filled in, persist across restarts and can be reordered and deleted (*inferred* reorder).
- **E22-S4-T2** — Variable expansion with side suffixes and the second-variable rule · size: M · depends: —
  - AC: unit cases cover every variable, the 1 and 2 suffixes, relative variables against a working folder, a second `%f`, and Path delimiter replacement.
- **E22-S4-T3** — Open With submenu entries and shortcuts in Folder Compare and Text Compare · size: S · depends: E22-S4-T1
  - AC: an item with only Accepts folders set appears for a selected folder but not for a selected file.
- **E22-S4-T4** — Launch with working folder, multiple instances and waiting · size: M · depends: E22-S4-T2, E22-S4-T3
  - AC: with three files selected and Multiple instances on, the tool runs three times. With Wait for previous instance to finish on, each run starts after the previous one exits. A command line using `%f2` runs once per pair.
- **E22-S4-T5** — Refresh when finished · size: S · depends: E22-S4-T4
  - AC: a tool that edits a selected file triggers a fast refresh of the folder view when it exits, but only if Refresh when finished is set.
- **E22-S4-T6** — Open With in Table, Hex and Picture views · size: S · depends: E22-S4-T3
  - AC: an item with the command line `tool %f1 %f2` launches with the two files of the active Table, Hex or Picture view.

### E22-S5 — Interface tweaks
**Priority**: Could · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E07-S1 (Text Compare view), E08-S1 (Folder Compare view)
**Traces**: F-26.11, F-5.3, F-7.1
**Not in this story**: thumbnail mode → E07-S11; incremental search delay → E07-S5; max auto-saved sessions → E06-S1.

As a power user, I can adjust small interface behaviours, such as hints, Esc handling and scrollbars, so that the tool behaves the way I expect.

**Acceptance criteria**
- [ ] *Show hints for toolbar buttons* turns toolbar tooltips on or off. *Show keyboard shortcuts in hints* adds each command's shortcut to its tooltip.
- [ ] The hint show and hide delays, in milliseconds, control when tooltips appear and disappear.
- [ ] With *Esc closes file views* on, Esc closes the current file view, such as Text Compare. It never closes a folder view.
- [ ] *Single vertical scrollbar* replaces the two vertical scrollbars in Text Compare with one shared scrollbar (*inferred* placement).
- [ ] *Sticky splitter position* keeps the splitter between the panes where the user put it when the window resizes or a new comparison loads, instead of re-centring it (*inferred* meaning).
- [ ] *Max most recently used items* sets how many entries the path-edit history dropdowns keep.

#### Tasks
- **E22-S5-T1** — Hint options: show hints, shortcuts in hints, and show and hide delays · size: S · depends: —
  - AC: with shortcuts in hints on, hovering a toolbar button shows its caption and shortcut after the show delay and hides them after the hide delay.
- **E22-S5-T2** — Esc closes file views · size: S · depends: —
  - AC: with the tweak on, Esc closes a Text Compare tab (after the unsaved-changes prompt) and does nothing in a Folder Compare tab.
- **E22-S5-T3** — Single vertical scrollbar in Text Compare · size: S · depends: —
  - AC: with the tweak on, one scrollbar scrolls both panes together.
- **E22-S5-T4** — Sticky splitter position · size: S · depends: —
  - AC: after the splitter is dragged to 30/70, resizing the window keeps it at 30/70.
- **E22-S5-T5** — Max most recently used items for path-edit history · size: S · depends: —
  - AC: with the limit set to 3, the history dropdown keeps the three most recent paths.

### E22-S6 — Comparison performance settings
**Priority**: Could · **Layer**: 3 · **Size**: S · **Platforms**: All
**Needs**: E08-S5 (content comparison by CRC and binary)
**Traces**: F-26.11

As an IT administrator comparing huge trees on slow storage, I can tune the order of background comparison work and the binary-compare buffer so that comparisons finish faster on my hardware.

**Acceptance criteria**
- [ ] The tweak *Comparison priority* sets the order in which background comparison work runs. The choices are decided in T1.
- [ ] The tweak *Buffer size for binary compare* sets how much of each file a binary comparison reads at a time.
- [ ] Results are the same for every setting value. Only speed and order change.
- [ ] New values apply to comparisons that start after the change.

#### Tasks
- **E22-S6-T1** — Decide the Comparison priority choices · size: S · depends: — · flags: `decision`
  - AC: decision recorded in this story. Proposed default: "Visible items first" as the default, plus "Folder order".
- **E22-S6-T2** — Comparison priority setting drives the order of background work · size: S · depends: E22-S6-T1
  - AC: with Visible items first, rows on screen get their content results before rows that are scrolled out of view.
- **E22-S6-T3** — Buffer size for binary compare · size: S · depends: —
  - AC: binary results for the same pairs are identical at the smallest and the largest buffer size, and the value persists.
