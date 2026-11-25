# E26 — OS integrations

**Group**: feature
**Label**: `area:os-integration`
**Size**: M
**Depends on**: E07, E08 (views); E12, E13, E14, E16, E24, E25 (for E26-S2 and E26-S3); E18, E24 (reports and Text Edit, for E26-S4)
**Source**: catalogue F-28.1, F-28.2, F-28.4, F-9.11, F-26.1, F-8.15, Appendix C. Spec-derived from the feature catalogue, which specifies Windows behaviour only, and not yet validated. Items marked *inferred* are analyst or PO inferences.

These integrations make the product reachable from outside it: the Explorer context menu, a clipboard-history tray utility, the native shell menu inside views, and legacy source-control providers. The catalogue specifies them for Windows only, so macOS and Linux equivalents are open decisions.

## Behavioral contract
- **Data**:
  - The remembered left item, and the remembered center item, used by later "Compare to" and "Merge with" commands.
  - Which context-menu commands are visible.
  - The clip history: text clips, with time and size limits.
  - Source-control provider bindings per folder, and each item's VCS status and revision.
- **Routes/endpoints**:
  - Explorer context-menu commands, in both the classic and the Windows 11 menus.
  - Startup options page: Include in Explorer context menu with a per-command checklist, and Run Clipboard Compare on Windows startup.
  - The tray icon's menu and the global hotkey Ctrl+Alt+C.
  - The Explorer command inside views.
  - Tools > Source Control Integration, the Source Control submenu, and Check In, Check Out and Undo Check Out in file views.
- **Integrations**:
  - The Windows shell: the context-menu extension, the notification area, global hotkeys and the clipboard.
  - MSSCCI-compatible source-control providers, as the catalogue specifies (Q12).
- **Rules/states**:
  - Context-menu commands are shown or hidden individually.
  - "Compare to <item>" uses the remembered left item.
  - The Source Control submenu appears only when a provider is configured and every selected item is under source control.
  - Clips shorter than the minimum clip length aren't stored, and the two newest clips are always kept.

---

## Stories

### E26-S1 — File-manager context menu
**Priority**: Could · **Layer**: 2 · **Size**: L · **Platforms**: Win
**Needs**: E07-S1 (Text Compare view), E08-S1 (Folder Compare view), E01-S2 (installers, which register the shell extension)
**Traces**: F-28.1, F-26.1 · **Open questions**: Q13
**Not in this story**: Compare Using Hex, Compare to Clipboard, Compare to Registry, Sync, Merge, Select Center, Edit and View Patch → E26-S2.

As a developer working in Explorer, I can compare files and folders from their context menu so that I don't have to open the app and type paths.

**Acceptance criteria**
- [ ] The macOS and Linux scope follows the decision in T1 (Q13).
- [ ] **Compare** compares the selected files or folders in their associated view.
- [ ] **Select Left File** and **Select Left Folder** remember the item. **Compare to <item>** then compares the selection with it; the caption names the remembered item.
- [ ] **Compare Left to <folder>\<file>** appears when a folder is selected. It compares the remembered left file with the same-named file inside that folder.
- [ ] **Compare Using ▸ <view type>** lists the registered view types and forces the chosen one. **Compare to <file> Using ▸** does the same against the remembered left item.
- [ ] **Open for Compare** opens the folder in Folder Compare.
- [ ] The commands appear in the Windows 11 context menu as well as the classic menu.
- [ ] The Startup options page has Include in Explorer context menu, with a checklist that shows or hides each command individually.

#### Tasks
- **E26-S1-T1** — Decide the macOS and Linux file-manager integration (Q13) · size: S · depends: — · flags: `decision`
  - AC: decision recorded. Proposed default: Windows only, as specified. Finder and Nautilus/Dolphin extensions go to the Roadmap appendix.
- **E26-S1-T2** — Shell extension for the classic and Windows 11 menus, registered by the installer · size: L · depends: E26-S1-T1 · flags: `licence-check`
  - AC: after install, right-clicking two files shows Compare in both menus. Uninstalling removes it.
- **E26-S1-T3** — Compare, Compare Using ▸ and Open for Compare · size: M · depends: E26-S1-T2 · flags: `naming-check`
  - AC: Compare on two `.txt` files opens Text Compare. Compare Using ▸ lists the registered view types and opens the one chosen.
- **E26-S1-T4** — Remembered left item: Select Left, Compare to <item>, Compare Left to <folder>\<file>, and Compare to <file> Using ▸ · size: M · depends: E26-S1-T2 · flags: `naming-check`
  - AC: after Select Left File on `a.txt`, right-clicking a folder that contains `a.txt` offers Compare Left to <folder>\a.txt, which compares the two files.
- **E26-S1-T5** — Startup options: Include in Explorer context menu and the per-command checklist · size: S · depends: E26-S1-T2
  - AC: unticking Open for Compare removes only that command from the menu.

### E26-S2 — More context-menu commands
**Priority**: Could · **Layer**: 4 · **Size**: M · **Platforms**: Win
**Needs**: E26-S1 (file-manager context menu), E12-S1 (Folder Sync), E13-S1 (Text Merge), E14-S1 (Folder Merge), E07-S18 (clipboard comparisons), E16-S1 (Hex Compare), E25-S4 (Registry Compare), E24-S1 (Text Edit), E24-S2 (Text Patch)
**Traces**: F-28.1

As a power user working in Explorer, I can sync, merge, edit, view patches, and compare against the clipboard or the registry straight from the context menu so that every workflow starts where my files are.

**Acceptance criteria**
- [ ] **Compare Using Hex** compares archive files byte by byte instead of expanding them.
- [ ] **Compare to Clipboard** compares the selected file with the clipboard text in Text Compare.
- [ ] **Compare to Registry** compares a `.reg` file with the live registry.
- [ ] **Sync** opens Folder Sync for the selected folders. **Sync with <folder>** opens it for the selection and the remembered left folder.
- [ ] **Merge** and **Merge with <item>** open Text Merge for files and Folder Merge for folders. **Select Center File** and **Select Center Folder** remember the ancestor that a later merge uses.
- [ ] **Edit** opens Text Edit, and **View Patch** opens Text Patch.
- [ ] Each new command can be shown or hidden in the Startup checklist.

#### Tasks
- **E26-S2-T1** — Compare Using Hex, Compare to Clipboard and Compare to Registry · size: S · depends: — · flags: `naming-check`
  - AC: Compare Using Hex on two zip files opens Hex Compare rather than Folder Compare.
- **E26-S2-T2** — Sync and Sync with <folder> · size: S · depends: —
  - AC: after Select Left Folder, Sync with <folder> opens Folder Sync with both folders set.
- **E26-S2-T3** — Merge, Merge with <item>, and Select Center File/Folder · size: M · depends: — · flags: `naming-check`
  - AC: after Select Left File and Select Center File, Merge with <item> on a third file opens Text Merge with the left, center and right files set.
- **E26-S2-T4** — Edit and View Patch, with checklist entries for every new command · size: S · depends: —
  - AC: View Patch on a `.diff` file opens Text Patch. Unticking Edit in the checklist hides it.

### E26-S3 — Clipboard Compare tray utility
**Priority**: Could · **Layer**: 4 · **Size**: M · **Platforms**: Win
**Needs**: E07-S1 (Text Compare view), E24-S1 (Text Edit view)
**Traces**: F-28.2, F-26.1 · **Open questions**: Q14

As a support engineer copying snippets out of emails and logs, I can keep a clipboard history and compare the last two clips with one hotkey so that I spot differences without saving files.

**Acceptance criteria**
- [ ] The supported platforms follow the decision in T1 (Q14).
- [ ] A notification-area app records every text clipboard change. Clicking its icon lists the recent entries.
- [ ] Right-clicking an entry offers Save As…, Delete and Open With Text Edit. Deleting an entry doesn't affect the OS clipboard.
- [ ] The global hotkey Ctrl+Alt+C, which can be turned on or off, compares the two most recent clips in Text Compare.
- [ ] The utility's options are:
  - Run on Windows startup.
  - Enable keyboard shortcut.
  - Associated file format mask, for example `*.java`, which picks the file format used for the comparison.
  - Minimum clip length: shorter clips aren't stored, which avoids keeping passwords; 0 stores everything.
  - Storage limits: a maximum number of clips and a maximum size in MB. The two newest clips are always kept.
- [ ] The main app's Startup options page has Run Clipboard Compare on Windows startup, kept in step with the utility's own option (*inferred*).

#### Tasks
- **E26-S3-T1** — Decide the target platforms (Q14) · size: S · depends: — · flags: `decision`
  - AC: decision recorded. Proposed default: Windows only, as specified. A cross-platform version goes to the Roadmap appendix.
- **E26-S3-T2** — Tray app recording text clips, with storage limits and a minimum clip length · size: M · depends: E26-S3-T1 · flags: `licence-check`, `naming-check`
  - AC: with the minimum length at 8, copying `abc` stores nothing. With a maximum of 3 clips, a fourth clip drops the oldest, but never one of the two newest.
- **E26-S3-T3** — Entry list with Save As, Delete and Open With Text Edit · size: S · depends: E26-S3-T2
  - AC: Delete removes the entry while the OS clipboard still holds the text.
- **E26-S3-T4** — Ctrl+Alt+C compares the two newest clips, using the associated format mask · size: S · depends: E26-S3-T2
  - AC: with mask `*.java`, the comparison opens with the file format that `*.java` resolves to.
- **E26-S3-T5** — Startup options in the utility and in the main app · size: S · depends: E26-S3-T2
  - AC: ticking Run on Windows startup in either place starts the utility at the next login.

### E26-S4 — Source control integration
**Priority**: Could · **Layer**: 5 · **Size**: L · **Platforms**: Win
**Needs**: E08-S6 (columns), E08-S12 (other filters), E07-S1 (Text Compare view), E18-S2 (folder reports), E24-S1 (Text Edit view)
**Traces**: F-28.4, F-9.11, F-8.2, F-8.9, F-22.2, F-12.10, F-20.1 · **Open questions**: Q12

As a team lead on a project that uses a legacy source-control system, I can see source-control status in Folder Compare and check files in and out from the comparison so that I reconcile and commit in one place.

**Acceptance criteria**
- [ ] The integration follows the decision in T1 (Q12).
- [ ] Tools > Source Control Integration links folders to an SCC-compatible provider.
- [ ] Once a provider is configured, Folder Compare's Status and Revision columns show each item's version-control status and revision.
- [ ] The Source Control submenu appears only when a provider is configured and every selected item is under source control. It contains:
  - Add to Source Control and Remove from Source Control;
  - Get Latest Version, and Get (a specific version; hidden by default);
  - Check In, Check Out and Undo Check Out;
  - Show Differences (asks the VCS to show its diff), Show History and Show Properties;
  - Run Source Control Application.
- [ ] Other Filters gains Version control status, with Is set or Is not set plus the status.
- [ ] The File menus of Text Compare and Text Edit have Check In, Check Out and Undo Check Out when the file is under source control.
- [ ] Folder reports accept the `vcs` and `revision` columns, written `column-vcs` and `column-revision` in scripts.

#### Tasks
- **E26-S4-T1** — Decide between MSSCCI support and modern VCS integration (Q12) · size: S · depends: — · flags: `decision`
  - AC: decision recorded. Proposed default: MSSCCI support on Windows, as specified. Git status and revision columns, plus diff and history commands on all platforms, go to the Roadmap appendix as the recommended successor.
- **E26-S4-T2** — Provider binding through Tools > Source Control Integration · size: M · depends: E26-S4-T1 · flags: `licence-check`
  - AC: linking a folder to an installed provider persists across restarts. Unlinking it clears that folder's status and revision values.
- **E26-S4-T3** — Status and Revision columns · size: S · depends: E26-S4-T2
  - AC: a checked-out file and a file not under control show different statuses. Revision shows the provider's revision.
- **E26-S4-T4** — Source Control submenu, with its visibility rule · size: M · depends: E26-S4-T2
  - AC: selecting one controlled file and one uncontrolled file hides the submenu. Check Out on controlled files changes their status.
- **E26-S4-T5** — Version control status filter · size: S · depends: E26-S4-T3
  - AC: the filter "Is set: Checked out" shows only checked-out files.
- **E26-S4-T6** — Check In/Check Out/Undo Check Out in Text Compare and Text Edit, and the `vcs` and `revision` report columns · size: S · depends: E26-S4-T2
  - AC: Check Out from Text Compare or Text Edit makes the file editable. A folder report with the `vcs` and `revision` columns shows both.

### E26-S5 — Native shell menu inside views
**Priority**: Could · **Layer**: 3 · **Size**: S · **Platforms**: Win
**Needs**: E07-S1 (Text Compare view), E08-S1 (Folder Compare view), E13-S1 (Text Merge view), E15-S1 (Table Compare view), E16-S1 (Hex Compare view), E17-S1 (Picture Compare view), E25-S1 (Media Compare), E25-S2 (Version Compare), E25-S4 (Registry Compare)
**Traces**: F-8.15, F-12.10, App-C

As a Windows user, I can open the native Explorer context menu for items in a comparison so that I can use shell tools such as Send to and Properties without leaving the view.

**Acceptance criteria**
- [ ] In Folder Compare, the Explorer command shows the native shell context menu for the selected items.
- [ ] In Text Compare, File > Explorer shows the native shell context menu for the focused pane's file.
- [ ] Changes that shell actions make to files are picked up on the next refresh (*inferred*).

- [ ] Text Merge, Table Compare, Hex Compare, Picture Compare, Media Compare, Version Compare and Registry Compare also offer *File > Explorer* for the focused pane's file. In Registry Compare it applies only to a `.reg` file side.
#### Tasks
- **E26-S5-T1** — Native shell context menu for folder selections · size: S · depends: — · flags: `licence-check`
  - AC: Properties from the shell menu opens the OS properties dialog for the selected file.
- **E26-S5-T2** — File > Explorer in Text Compare · size: S · depends: —
  - AC: the menu targets the file shown in the focused pane.
- **E26-S5-T3** — File > Explorer in the other file views · size: S · depends: E26-S5-T2
  - AC: in each listed view, Explorer shows the shell menu for the focused pane's file. It is disabled for a live-registry side.
