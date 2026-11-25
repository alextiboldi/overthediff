# E01 — Application platform

**Group**: foundation
**Label**: `area:platform`
**Size**: L
**Depends on**: —
**Source**: catalogue §1, §2, §3.2, F-5.1, F-5.2, F-6.1, F-6.2, F-7.1, F-25.1, F-27.1, Q24, Q25. Spec-derived from the feature catalogue and not yet validated. Items marked *inferred* are analyst or PO inferences.

This epic is the desktop frame every other story plugs into: a single-instance application with windows, tabs, commands, dialogs, settings storage and background tasks, plus installable builds for each OS. It exists so that feature stories can add commands, options, settings tabs and session types without touching each other's code. That is what makes the rest of the backlog independently buildable.

**Stack**: Rust and egui. OverTheDiff is a Craft app, so the shared rules in `AGENTS.md` apply.

## Behavioral contract
- **Data**:
  - *Session*: type, specs (one input per side), rules (type-specific settings keyed by tab and option), description.
  - *View*: a tab showing a session or the Home launcher. It has a title, an unsaved-changes flag and status-bar fields.
  - *Command*: stable id, caption, description, icon, default shortcut, view types, menu path (including the macOS application menu), toolbar slot, hidden-by-default flag, enabled/checked state, optional dynamic caption and icon, and typed arguments and result.
  - *Session type*: name, Home button, settings tabs and the **file-format kinds** it opens (for example Text Compare opens Text-kind formats). The shell can look up the session type for a format kind.
  - *Argument handler*: the argument pattern it claims (a switch or bare paths), and its **default launch mode**. There are three launch modes:
    - forward-and-exit, the default;
    - run-in-launching-process: no main window and nothing forwarded; the handler may show its own progress window and prompts, and its result is the exit code;
    - forward-and-wait: the launching process waits until the opened view closes, then exits with the code the handler reports.
  - *Colour role*: id, light value and dark value. Features register the colours they draw with as roles (status colours, line backgrounds, syntax elements and so on).
  - *Option contribution*: page path, option id, label, value type (boolean, number, choice, text, path, colour), default.
  - *Settings-tab contribution*: session type, tab name, option id, label, value type, default.
  - *Background task*: title, progress, cancellable flag, result (completed, cancelled or failed with per-item errors).
  - *Settings store*: a per-user, per-major-version folder holding program options, program state (window geometry, MRU lists) and the named collections other epics persist (sessions, formats, profiles).
- **Routes/endpoints** (desktop app, so these are commands and process entry points):
  - Session menu: New Session ▸ <type>, New Tab, Open Session, Clear Session, Session Settings (plus the Rules toolbar button), Close Tab, Exit.
  - Tools > Options.
  - The process entry point `otd [arguments]` (final executable name is decided in E01-S2). It hands arguments to registered handlers.
- **Integrations**: the OS file and folder pickers, the per-user configuration folders of each OS, and the OS mechanism for a second process to reach a running instance.
- **Rules/states**:
  - **Single instance and launch modes.**
    - By default a second launch forwards its arguments and working folder to the running instance and exits (forward-and-exit).
    - A launch can request forward-and-wait, or a **new instance**, for whichever handler claims its arguments. Waiting launchers and `/solo` use this.
    - A running handler can hand off from run-in-launching-process to forward-and-wait, for example when a headless merge needs review.
    - A handler can attach content read in the launching process, such as standard input, to the forwarded request.
    - A handler matched by a switch claims the whole command line before bare-path handlers.
  - **Extension points instead of shared ownership.** Any story may add commands to a view type, options to a named Options page, or options to a named Session Settings tab of a session type. A page or tab appears once something contributes to it. No feature story "creates" a shared page.
  - **Unsaved changes.** Closing a view with unsaved changes, or exiting, prompts Save / Don't Save / Cancel for each affected view.
  - **Settings writes are atomic.** An interrupted write never leaves a half-written settings file. A corrupt file is set aside, defaults are used and the user is told.
  - **Major versions** keep separate settings folders and can run side by side (F-27.1).
  - **One way to act.** Menus, toolbars, shortcuts, scripts and the automation interface (E28) all run registered commands. A command runs by id with typed arguments and returns a result, with or without the window. A command that needs the window reports that instead of claiming success.
  - **Translated text.** Every caption, label and message goes through the translation system, with the English text as the key.
  - **Colour roles.** Views draw only with registered colour roles, never fixed colours. The active palette picks each role's light or dark value. Until a theme choice exists, light values apply.
  - **Platform menu placement.** On macOS, application-level commands (Options and similar) go in the application menu. On Windows and Linux, they stay in their usual menus.

---

## Stories

### E01-S1 — Application shell, settings store and extension points
**Priority**: Must · **Layer**: 0 · **Size**: L · **Platforms**: All
**Needs**: —
**Traces**: F-5.1, F-5.2, F-6.1, F-6.2, F-6.6, F-7.1, F-25.1, F-25.2, F-27.1 · **Open questions**: Q24, Q25
**Not in this story**: themes → E05-S1; multiple windows and tab options → E05-S2; saving sessions → E06-S1; "Update session defaults" → E06-S2; child-session scopes → E06-S7; Disable editing → E06-S8; customising commands → E22-S1; MRU length tweak → E22-S5; installers → E01-S2; comparison views → E07-S1, E08-S1; `otd-cli` and MCP → E28.

As a developer on the product team, I can rely on one application frame with registered commands, contributed options and session settings, background tasks and durable settings so that every feature story ships on its own without editing another story's code.

Every AC below is verified with a test session type and test commands registered by the acceptance harness. No real comparison view exists yet.

**Acceptance criteria**
- [ ] With no instance running, launching the app opens a main window on the Home tab. Launching it again forwards the arguments to the running instance, which acts on them, and the second process exits at once.
- [ ] Each launch argument goes to the feature that registered a matching pattern. An argument nobody claims shows a usage message, and the process exits with code 100 (*inferred*).
- [ ] Launch modes work as declared by the handler or requested by the launch:
  - **forward-and-exit** (default): the arguments go to the running instance, or a new one, and the launching process exits at once with 0;
  - **run-in-launching-process**: no main window opens and nothing is forwarded. The handler may show its own progress window and prompts, and its result becomes the exit code;
  - **forward-and-wait**: the launching process waits until the opened view closes, then exits with the code the handler reports;
  - a launch can request forward-and-wait, or a new instance, for whichever handler claims its arguments;
  - a handler running in the launching process can hand off to forward-and-wait;
  - a handler can attach content read in the launching process, such as standard input, to the forwarded request.
- [ ] A handler matched by a switch claims the whole command line before bare-path handlers.
- [ ] Session types declare the file-format kinds they open, and the shell returns the session type for a given kind. This is checked with test types.
- [ ] Home shows one button per registered session type. A button, or *Session > New Session ▸ <type>*, opens an empty session of that type in a new tab.
- [ ] *New Tab* opens Home in a new tab and *Open Session* shows Home. *Close Tab* closes the current tab, and closing the last tab shows Home.
- [ ] Each command appears in the menus and toolbar slots declared for the current view type:
  - disabled commands are greyed out;
  - hidden-by-default commands have no menu entry but run from their shortcut;
  - a command with a dynamic caption updates its caption and icon when its state changes.
  - on macOS, commands declared as application-level (Options, for example) appear in the application menu. On Windows and Linux they appear under Tools or Help.
- [ ] *Tools > Options* shows a page tree. Options contributed to a page appear on it, and pages with nothing contributed are absent. Apply saves without closing, OK saves and closes, and Cancel discards changes.
- [ ] *Session > Session Settings* and the Rules toolbar button open a dialog with the tabs contributed for that session type.
  - The Specs tab shows the session's inputs and a free-text Description.
  - The scope dropdown offers "Use for this view only", which applies changes to the current view only.
  - Several features can contribute options to the same named tab, which then shows all of them.
- [ ] *Clear Session* replaces the current view with an empty session of the same type.
- [ ] A long task shows progress with Cancel. Cancel stops at the next safe point, and failures are listed in an end-of-task summary.
- [ ] A path edit accepts a typed path, offers Browse for File or Browse for Folder through the OS picker, and keeps a dropdown of recent paths for that field.
- [ ] Settings survive restarts in a per-user, per-major-version folder in the OS's standard location.
  - An interrupted write doesn't corrupt them.
  - A corrupt settings file is set aside, defaults are used and a notice is shown.
- [ ] Closing a tab or exiting with unsaved changes prompts Save / Don't Save / Cancel for each affected view. Cancel aborts the close.
- [ ] Features can register colour roles with light and dark values. A view drawing with a role shows the role's value from the active palette, light by default.
- [ ] Any registered command runs by id with typed arguments and returns a result, with no window open. Bad arguments return an error, never a panic, and a command that needs the window says so.
- [ ] Captions, labels and messages are looked up through the translation system with English as the key. A missing translation shows the English text.
- [ ] The default keymap conventions (Q24) and the status-bar field model (Q25) are recorded in this story.

#### Tasks
- **E01-S1-T1** — Decide the default keymap conventions and status-bar field model (Q24, Q25) · size: S · depends: — · flags: `decision`
  - AC: a decision in this story lists platform-standard shortcuts, the product shortcuts named in the catalogue (Ctrl+R/Ctrl+L copy, F2 and Del in managers, Space to mark, Shift+arrow to limit a selection, Esc to close file views) and how views fill status-bar fields (overall result, insert/overwrite, load time). Proposed default: follow each OS's standard shortcuts and keep the product shortcuts named in the catalogue.
- **E01-S1-T2** — Single-instance launch, argument hand-off and argument dispatch · size: M · depends: —
  - AC: two launches in quick succession end with one running instance, which handled both argument sets. An unclaimed argument gives the usage message and exit code 100. With a running instance, a run-in-launching-process test handler exits with its own code and nothing reaches the instance. A forward-and-wait test handler's launcher exits only after the test view closes, with the code the handler reported. A launch requesting forward-and-wait for a default test handler waits for its view to close, and a launch requesting a new instance starts a second process. A test handler that hands off from the launching process to forward-and-wait opens its view in the running instance, and the launcher exits after it closes. Content attached by a test handler reaches the running instance. With a switch handler and a bare-path handler both matching, only the switch handler runs. A run-in-launching-process test handler can show its own progress window while no main window opens.
- **E01-S1-T3** — Main window with tab strip, Home launcher tab and status bar · size: M · depends: E01-S1-T2
  - AC: New Tab, Close Tab and tab switching work. Home lists registered session types. Status-bar fields are filled from the active view.
- **E01-S1-T4** — Command registry driving menus, toolbars and shortcuts per view type · size: L · depends: E01-S1-T3
  - AC: switching to another view type swaps the menu and toolbar contents. Disabled, hidden-by-default and dynamic-caption commands behave as specified. On macOS, Options sits in the application menu.
- **E01-S1-T5** — Settings store in a per-user, per-major-version folder with atomic writes and corrupt-file recovery · size: M · depends: — · flags: `craft-libs`
  - AC: killing the process during a write leaves the previous settings readable. A deliberately corrupted file is set aside, and the app starts with defaults and a notice.
- **E01-S1-T6** — Options dialog shell with contributed pages and options · size: M · depends: E01-S1-T4, E01-S1-T5
  - AC: an option contributed by a test feature appears on its page and persists after Apply and a restart. An uncontributed page doesn't show. Cancel restores the previous values.
- **E01-S1-T7** — Session model and Session Settings dialog shell (contributed tabs, Specs tab with Description, "Use for this view only" scope), New Session, Open Session and Clear Session · size: L · depends: E01-S1-T3 · flags: `naming-check`
  - AC: a test session type registered for a test format kind is returned when the shell looks up that kind. A test session type contributing two tabs shows Specs plus those two tabs. Two test features contributing options to the same named tab show one tab holding both sets of options. A change applies only to the current view. Clear Session gives an empty session of the same type.
- **E01-S1-T8** — Background-task service with progress, cancel and end-of-task error summary · size: M · depends: E01-S1-T3
  - AC: a test task reporting progress can be cancelled midway, and a task with three failing items lists all three in the summary.
- **E01-S1-T9** — Path edit control with typed path, Browse for File/Folder and a recent-paths dropdown · size: S · depends: E01-S1-T5 · flags: `craft-libs`
  - AC: recent paths persist per field across restarts, and Browse returns the chosen path into the edit.
- **E01-S1-T10** — Unsaved-changes prompts on Close Tab and Exit · size: S · depends: E01-S1-T3
  - AC: a test view flagged as unsaved triggers Save / Don't Save / Cancel. Cancel keeps the view open, and Exit prompts once for each unsaved view.
- **E01-S1-T11** — Colour roles with light and dark values · size: S · depends: E01-S1-T3
  - AC: a harness view drawing with a registered role shows its light value. Switching the active palette from code re-colours the view without a restart.
- **E01-S1-T12** — Commands with typed arguments and results, runnable by id without the window · size: M · depends: E01-S1-T4
  - AC: a test command run by id with valid arguments returns its result in a headless test. Missing, unknown or out-of-range arguments return an error naming the argument. A test command declared as needing the window reports that when run headlessly.
- **E01-S1-T13** — Translation lookup for captions, labels and messages · size: S · depends: E01-S1-T3
  - AC: a harness caption with a test translation shows translated, and one without shows its English key. No caption in the shell is a literal outside the lookup.

### E01-S2 — Installable builds for Windows, macOS and Linux
**Priority**: Must · **Layer**: 1 · **Size**: M · **Platforms**: All
**Needs**: E01-S1 (application shell)
**Traces**: F-25.1, §3.2
**Not in this story**: waiting launchers and macOS "Install Command Line Tools" → E11-S1; portable installation → E23-S4; packaging `otd-cli` → E28-S1.

As a release engineer, I can install the product on Windows, macOS and Linux from a standard package so that teams can run it on the machines they already use.

**Acceptance criteria**
- [ ] A Windows installer installs the application, adds a Start menu entry, and uninstalls cleanly.
- [ ] On Windows, the UI is per-monitor DPI aware: moving the window between monitors with different scaling re-renders it crisply. It runs on Windows 11.
- [ ] A macOS build runs natively on both Apple silicon and Intel Macs and opens on a clean machine without security warnings (*inferred* distribution requirement).
- [ ] A Linux package installs the application with a desktop entry and icon.
- [ ] Uninstalling on any OS removes the program but keeps the user's settings folder.
- [ ] Two major versions install side by side, and each keeps its own settings.
- [ ] The supported OS versions, CPU architectures and package formats are recorded (T1) and each one is covered by a smoke test.
- [ ] Packages are built by the storytold/craft-common-actions release workflow, in a pull request of their own (AGENTS.md rule 17). NOTICE and the licence texts ship inside every package. Nothing is signed or published without the maintainer's go-ahead (rule 4).

#### Tasks
- **E01-S2-T1** — Decide supported OS versions, CPU architectures, package formats and the executable name · size: S · depends: — · flags: `decision`
  - AC: decision recorded in this story. Proposed default: Windows 10/11 x64 and ARM64; macOS universal build on the two newest major versions; Linux x64 as a distribution-neutral package plus one native package format. Start with unsigned dry-run builds; signing needs secrets and the maintainer's go-ahead.
- **E01-S2-T2** — Windows installer with Start menu entry, per-monitor DPI awareness and clean uninstall · size: M · depends: E01-S2-T1
  - AC: after install, the app launches from the Start menu and stays crisp on a 100%/200% monitor pair. After uninstall, the program files are gone and the settings remain.
- **E01-S2-T3** — macOS application bundle native on Apple silicon and Intel, distributable without security warnings · size: M · depends: E01-S2-T1
  - AC: on a clean Mac of each architecture, the downloaded app opens with no warning and reports its native architecture.
- **E01-S2-T4** — Linux package with desktop entry and icon · size: M · depends: E01-S2-T1
  - AC: after install on a clean machine, the app shows in the desktop's application launcher and starts from it.
- **E01-S2-T5** — Side-by-side major versions and settings kept on uninstall · size: S · depends: E01-S2-T2, E01-S2-T3, E01-S2-T4
  - AC: with two major versions installed, changing any persisted setting in one (for example a path edit's recent paths) leaves the other unchanged. Reinstalling restores the earlier settings.
