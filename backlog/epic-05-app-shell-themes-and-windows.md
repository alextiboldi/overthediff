# E05 — App shell, themes and windows

**Group**: feature
**Label**: `area:shell`
**Size**: M
**Depends on**: E01 (foundation); E07, E08 (comparison views); E22 (command customisation)
**Source**: catalogue §5, F-26.2, F-26.3, F-26.8, F-28.5, §3.2. Spec-derived from the feature catalogue and not yet validated. Items marked *inferred* are analyst or PO inferences.

This epic sets how the application looks and how its windows behave: light and dark themes, several windows with movable tabs, starting comparisons by drag and drop, and how display filters appear on toolbars. It makes the shell from E01 comfortable for daily use on every desktop platform.

## Behavioral contract
- **Data**:
  - *Appearance setting*: Follow operating system, Light theme or Dark theme. Preview is a dialog-only checkbox and isn't stored (*inferred*).
  - *Tab options*: New sessions open in a new window or a new tab; Closing last remaining tab closes the window (on/off); Hide tab bar if single tab (on/off).
  - *Window state*: each window holds an ordered list of tabs, and each tab holds one view.
  - *Display-filter presentation*: DropDown, Favorites or Toggles, plus Large buttons with text labels, stored per view type (*inferred* scope).
- **Routes/endpoints**:
  - Options pages: Appearance and Tabs.
  - Menus and tabs: Session > New Window, the tab right-click *Move Tab to New Window*, and the macOS Window menu.
  - Drop targets: Home, its session-type buttons and the application icon.
  - The toolbar right-click menu.
- **Integrations**:
  - E01-S1 command registry, Options dialog, Home launcher and single-instance argument hand-off.
  - Text Compare and Folder Compare views, for drag and drop and display filters.
  - The customise dialog in E22, for the display-filter mode.
- **Rules/states**:
  - The theme sets the base palette, and colour customisations sit on top of it.
  - With both tab options off, closing the last tab shows Home.
  - Exit closes every window of the instance after the unsaved-changes prompts.
  - A drop on a session-type button forces that type. A drop anywhere else lets the file format decide.
  - Toggles combine any comparison states, while DropDown and Favorites offer fixed combinations.

---

## Stories

### E05-S1 — Light and dark themes
**Priority**: Must · **Layer**: 1 · **Size**: S · **Platforms**: All
**Needs**: E01-S1 (application shell and Options dialog)
**Traces**: F-5.4, F-26.3
**Not in this story**: custom colours and fonts → E22-S2, E22-S3.

As a developer who works on a dark desktop, I can choose a light or dark theme, or follow the operating system, so that the tool matches my environment and stays readable.

**Acceptance criteria**
- [ ] The Appearance options page offers *Follow operating system*, *Light theme* and *Dark theme*. The first shows the current OS setting in parentheses, for example "Follow operating system (Dark)".
- [ ] With *Follow operating system* selected, the app uses the OS light or dark setting at start-up and switches when that setting changes while the app runs (*inferred*).
- [ ] *Light theme* and *Dark theme* apply whatever the OS setting is.
- [ ] With the *Preview* checkbox ticked, changes apply to every open view at once. Cancel restores the previous theme and OK keeps the new one.
- [ ] The window chrome, the selection and every registered colour role take the active theme's value: light values in the light theme, dark values in the dark theme. Acceptance uses harness-registered roles, so it doesn't depend on which views exist. Colour customisations can sit on top.
- [ ] Chrome and selection text stay readable in both themes. An automated check reports any registered colour role whose text falls below a 4.5:1 contrast ratio against its background (*inferred* target).
- [ ] The theme choice persists across restarts.

#### Tasks
- **E05-S1-T1** — Light and dark palettes for the chrome and selection, and theme switching for every registered colour role · size: M · depends: — · flags: `accessibility` · status: needs-verification
  - AC: a harness role with light and dark values switches with the theme. The contrast check passes for chrome and selection in both palettes and flags a deliberately low-contrast harness role.
  - Implemented: nine chrome and selection colour roles with light and dark values (`CHROME`, `CHROME_CONTRAST`) in `crates/ui/src/colour.rs` (`otd-ui`); `install` and `set_palette` rebuild egui's visuals from the registered roles and write them to both of egui's styles, so the OS setting can't swap in egui's own style. The UI-free WCAG 2 `contrast_ratio`, `Contrast` pairs (4.5:1 text, 3:1 controls) and `ColourRoles::low_contrast` are in `crates/core/src/colour.rs` (`otd-core`); `apps/otd/src/colour_roles.rs` registers the chrome roles and checks that every registered role is in a pair and every pair is readable in both palettes. Tests: `crates/ui/tests/it/colour.rs` (palette switch, chrome and selection in both palettes, simulated dark OS) and the `otd-core` unit tests (a deliberately low-contrast role flagged in one palette only). Checks: `cargo xtask ci` (fmt, clippy `-D warnings`, `cargo test --workspace --locked`, ui-free) passed on macOS after rebasing onto main, plus pull-request CI (Linux, macOS); no Windows matrix run (no platform-specific code). craft-libs follow-up: #70. Pull request: https://github.com/alextiboldi/overthediff/pull/69
  - Verify by hand: in the running app on macOS and Windows, set the palette to Light while the OS is in dark mode, then switch the OS setting: the chrome stays in the palette's colours, and `ctx.system_theme()` reports the OS change (the input T2 and T3 rely on).
- **E05-S1-T2** — Appearance page with the three theme choices and the OS setting in parentheses · size: S · depends: E05-S1-T1
  - AC: the label shows the OS setting when the page opens, and the choice persists across restarts.
- **E05-S1-T3** — Follow the operating system setting, including changes while running · size: S · depends: E05-S1-T1
  - AC: while Follow operating system is selected, switching the OS between light and dark recolours open views without a restart.
- **E05-S1-T4** — Preview checkbox applies changes live, and Cancel reverts them · size: S · depends: E05-S1-T2
  - AC: with Preview on, picking Dark recolours open views at once. Cancel returns them to the previous theme.

### E05-S2 — Multiple windows and tab behaviour
**Priority**: Should · **Layer**: 1 · **Size**: M · **Platforms**: All
**Needs**: E01-S1 (application shell, tabs and Options dialog)
**Traces**: F-5.1, F-26.2, §3.2
**Not in this story**: child-session placement option → E06-S7; workspaces → E06-S6.

As a code reviewer comparing several changes at once, I can spread comparisons over several windows and decide where new sessions open so that I can use more than one monitor and keep related work together.

**Acceptance criteria**
- [ ] *Session > New Window* opens a new top-level window showing Home.
- [ ] Right-clicking a tab offers *Move Tab to New Window*, which moves that view and its state into a new window.
- [ ] Dragging a tab onto another window's tab strip moves the view into that window.
- [ ] *Exit* asks about every view with unsaved edits in every window, then closes all windows of the instance. Cancelling any prompt keeps the app open.
- [ ] The Tabs options page offers *New sessions open in* a new window or a new tab. New sessions started from Home, the Session menu and the command line all follow it.
- [ ] *Closing last remaining tab closes the window* and *Hide tab bar if single tab* do what their names say. With both off, closing the last tab shows Home.
- [ ] On macOS the Window menu offers Minimize, Minimize All, Zoom, Zoom All, Cycle Through Windows (forward and back), Select Next Tab, Select Previous Tab, Move Tab to New Window, Merge All Windows (moves every tab onto the active window), Bring All to Front and Arrange in Front.

#### Tasks
- **E05-S2-T1** — New Window command and several windows in one instance · size: M · depends: —
  - AC: two windows each hold their own tabs, and closing one leaves the other running. A test session opened by a second launch through the E01-S1 test handler follows *New sessions open in*.
- **E05-S2-T2** — Move Tab to New Window, and dragging tabs between windows · size: M · depends: E05-S2-T1
  - AC: a moved view keeps its state and its unsaved-changes flag, checked with the E01-S1 test session type. A tab dropped on another window's strip lands at the drop position.
- **E05-S2-T3** — Exit closes every window after the save prompts · size: S · depends: E05-S2-T1
  - AC: with unsaved edits in two windows, Exit prompts for each. Cancel on the second prompt leaves both windows open.
- **E05-S2-T4** — Tabs options page with *New sessions open in*, *Closing last remaining tab closes the window* and *Hide tab bar if single tab* · size: S · depends: E05-S2-T1
  - AC: each option takes effect without a restart. With both tab options off, closing the last tab shows Home.
- **E05-S2-T5** — macOS Window menu · size: S · depends: E05-S2-T1
  - AC: on macOS every listed command is present and works. Merge All Windows leaves one window holding every tab.

### E05-S3 — Drag and drop to start comparisons
**Priority**: Should · **Layer**: 2 · **Size**: S · **Platforms**: All
**Needs**: E07-S1 (Text Compare view), E08-S1 (Folder Compare view)
**Traces**: F-28.5, F-5.2
**Not in this story**: snapshot drop auto-compare → E19-S1.

As a developer with files open in my file manager, I can drag them onto the app to compare them so that I don't have to type or browse for paths.

**Acceptance criteria**
- [ ] Dropping two folders on Home opens a Folder Compare, with the first dropped folder on the left and the second on the right (*inferred* order: the order the file manager supplies).
- [ ] Dropping two files on Home opens the view their file format chooses. Two text files open in Text Compare.
- [ ] Dropping on a session-type button on Home forces that type. For example, two files dropped on the Text Compare button open in Text Compare whatever their format.
- [ ] Dropping items on the application icon (dock icon or shortcut) starts the same comparison. If the app is already running, the comparison opens in the running instance.
- [ ] Drops with another number of items, or a file mixed with a folder, follow the T1 decision and never close the user's open views.

#### Tasks
- **E05-S3-T1** — Decide what dropping one item, more than two items, or a file with a folder does · size: S · depends: — · flags: `decision`
  - AC: decision recorded in this story. Proposed default: one item opens a new session of the chosen type with that item on the left. More than two items, or a file with a folder, shows a message and opens nothing.
- **E05-S3-T2** — Home accepts dropped files and folders and opens the comparison · size: S · depends: E05-S3-T1
  - AC: two dropped folders open Folder Compare, and two dropped text files open Text Compare, with the paths shown in the path edits.
- **E05-S3-T3** — Session-type buttons as drop targets that force the type · size: S · depends: E05-S3-T2
  - AC: the button under the cursor highlights during the drag, and the dropped items open in that type.
- **E05-S3-T4** — Drops on the application icon start a comparison, also while the app is running · size: S · depends: —
  - AC: with the app running, dropping two files on its icon opens one new comparison in the running instance and starts no second instance.

### E05-S4 — Display-filter toolbar layouts
**Priority**: Should · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E07-S3 (Text Compare display filters), E08-S2 (Folder Compare display filters), E22-S1 (customise menus, toolbars and shortcuts)
**Traces**: F-5.3, F-26.8

As a QA engineer who filters comparisons all day, I can choose how display filters appear on the toolbar, including toggles for any combination, so that the filters I use most are one click away.

**Acceptance criteria**
- [ ] Right-clicking the toolbar offers the display-filter presentations *DropDown*, *Favorites* and *Toggles*, plus *Large buttons with text labels*.
- [ ] *DropDown* stays the default. It is the presentation each view's display-filter toolbar already has: Show All, Show Differences and Show Same as buttons, plus a dropdown holding the view's other display filters.
- [ ] *Favorites* shows several common combinations as buttons, chosen per view in T1.
- [ ] *Toggles* shows one toggle per comparison state the view distinguishes. Any combination can be on, for example Left Orphans plus Right Newer only in Folder Compare. The view then shows exactly the rows in the toggled states.
- [ ] *Large buttons with text labels* shows larger toolbar buttons with their captions.
- [ ] The customise dialog offers the same *Display filter mode* and *Large buttons with text labels* settings for each view type. A change in either place shows in the other.
- [ ] The chosen presentation persists per view type across restarts (*inferred* scope).

#### Tasks
- **E05-S4-T1** — Decide the Favorites buttons for each view · size: S · depends: — · flags: `decision`
  - AC: decision recorded in this story. Proposed default: Folder Compare gets Show All, Show Differences, Show Same, Show Orphans and Show Differences but No Orphans. Text Compare gets Show All, Show Differences, Show Same and Show Context.
- **E05-S4-T2** — Folder Compare filters by any set of comparison states · size: M · depends: —
  - AC: toggling Left Orphans and Right Newer shows exactly those rows, and the folder display mode still decides which folders appear.
- **E05-S4-T3** — DropDown, Favorites and Toggles presentations on the toolbar right-click menu · size: M · depends: E05-S4-T1, E05-S4-T2 · flags: `naming-check`
  - AC: switching the presentation redraws the toolbar at once in Text Compare and Folder Compare, and the active filter stays the same.
- **E05-S4-T4** — Large buttons with text labels · size: S · depends: —
  - AC: with the option on, every toolbar button shows its caption. Turning it off restores compact buttons.
- **E05-S4-T5** — The same two settings in the customise dialog, kept in sync · size: S · depends: E05-S4-T3, E05-S4-T4
  - AC: choosing Toggles for Folder Compare in the customise dialog changes the Folder Compare toolbar, and the toolbar right-click menu then shows Toggles as selected.
