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

#### Decisions
Recorded by E01-S1-T1 and confirmed by the operator on 2026-11-25, as proposed: follow each OS's standard shortcuts, keep the product shortcuts named in the catalogue, and have views fill shared status-bar fields. The catalogue names only the product shortcuts and the three status-bar fields. Everything else below is *unspecified* there and is OverTheDiff's own choice.

**Default keymap conventions (Q24)**
- **Modifiers.** The catalogue's Ctrl is the command modifier: Cmd on macOS and Ctrl on Windows and Linux (craft-keymap's `Cmd`, egui's `Modifiers::COMMAND`), so Copy to Right is Cmd+R on macOS. Alt is Option on macOS. Ctrl+Tab and Ctrl+Shift+Tab keep the Control key on macOS, as macOS tabbed apps do.
- **Platform-standard shortcuts.** These commands take their OS's chord:

  | Command | macOS | Windows | Linux |
  |---|---|---|---|
  | New Tab | Cmd+T | Ctrl+T | Ctrl+T |
  | Close Tab | Cmd+W | Ctrl+W, Ctrl+F4 | Ctrl+W |
  | Next / previous tab | Ctrl+Tab / Ctrl+Shift+Tab, Cmd+Shift+] / Cmd+Shift+[ | Ctrl+Tab / Ctrl+Shift+Tab | Ctrl+Tab / Ctrl+Shift+Tab |
  | Open Session | Cmd+O | Ctrl+O | Ctrl+O |
  | Save / Save As | Cmd+S / Cmd+Shift+S | Ctrl+S / Ctrl+Shift+S | Ctrl+S / Ctrl+Shift+S |
  | Undo | Cmd+Z | Ctrl+Z | Ctrl+Z |
  | Redo | Cmd+Shift+Z | Ctrl+Y, Ctrl+Shift+Z | Ctrl+Shift+Z, Ctrl+Y |
  | Cut / Copy / Paste / Select All | Cmd+X / C / V / A | Ctrl+X / C / V / A | Ctrl+X / C / V / A |
  | Find | Cmd+F | Ctrl+F | Ctrl+F |
  | Find Next / Find Previous | Cmd+G / Cmd+Shift+G | F3 / Shift+F3 | F3 / Shift+F3 |
  | Replace | Cmd+Alt+F | Ctrl+H | Ctrl+H |
  | Options | Cmd+, (application menu) | none | none |
  | Help | Cmd+? | F1 | F1 |
  | Exit | Cmd+Q (application menu) | none: Alt+F4 is the OS's and closes the window | Ctrl+Q |
  | Hide / Hide Others / Minimize | Cmd+H / Cmd+Alt+H / Cmd+M | — | — |
  | Next / previous region, also out of an editable pane | F6 / Shift+F6 | F6 / Shift+F6 | F6 / Shift+F6 |
  | Menu bar, context menu | the OS's own | Alt or F10, Shift+F10 or the Menu key | Alt or F10, Shift+F10 or the Menu key |
  | Close a menu, popup or dialog / its default button | Esc / Enter | Esc / Enter | Esc / Enter |

  While a text field or an editable pane has focus, the platform's text-editing chords go to it (craft-keymap `text_editing_chords`: undo, redo, select all, and moving and deleting by word, line and text), and plain keys type. An editable pane may take Tab to indent (E10-S4, E24-S6), so F6 / Shift+F6 are the way out of it on every OS: no pane or field captures them. On Mac keyboards whose top row is media keys, they are Fn+F6 / Fn+Shift+F6.
- **Product shortcuts named in the catalogue.** Each applies only where shown:

  | Chord | Command | Where |
  |---|---|---|
  | Ctrl+R / Ctrl+L | Copy to Right / Copy to Left, hidden by default (E09-S1) | Folder Compare |
  | F2 | Rename the selected item | manager lists, such as Manage Workspaces (E06) |
  | Del | Delete the selected item | manager lists. On macOS, Cmd+Backspace too, since many Mac keyboards have no forward Delete |
  | Space | Mark the focused item (E08-S10) | the Folder Compare list, while it has focus. Elsewhere Space activates the focused control |
  | Shift+Left / Shift+Right | Limit a both-sides selection to the left / right side (E08-S7) | the Folder Compare list. In text panes Shift+arrow extends the selection as usual |
  | Esc | Close the current file view (E22-S5, when *Esc closes file views* is on) | file views such as Text Compare, never a folder view |
  | Insert | Toggle insert/overwrite | editable panes. On every OS the command is also in the Edit menu and the status-bar field toggles it, so keyboards without an Insert key, such as Mac ones, still reach it |
  | Ctrl+arrow | Nudge the crop offset by a larger step (E17) | the Picture Compare difference pane |
  | Ctrl+Alt+C | Compare the two newest clips, a global hotkey that can be turned off (E26-S3) | Windows only |
- **Esc, innermost first.** Esc closes an open menu, popup or dialog and puts focus back, then cancels a pending action (Align With in E07), then leaves full screen (E17), then closes the find bar. Only when none of these is open does it close a file view, and then the unsaved-changes prompt still applies.
- **Reserved chords.** No default uses a chord the OS takes. The keymap reserves craft-keymap's `reserved::system` list for the current OS:
  - macOS: Cmd+Tab, Cmd+Shift+Tab, Cmd+Alt+Esc, Cmd+\`;
  - Windows: Alt+Tab, Alt+Shift+Tab, Alt+F4, Ctrl+Alt+Del, Ctrl+Esc, Ctrl+Shift+Esc;
  - Linux: Alt+Tab, Alt+Shift+Tab, Alt+F4, Ctrl+Alt+Del, Ctrl+Alt+F1 to F12.

  Defaults also avoid:
  - macOS: Cmd+Space, Ctrl+Space, Control+arrow keys, Cmd+Shift+3, 4 and 5, Ctrl+Cmd+Q;
  - Windows: every chord with the Windows key;
  - Linux: every chord with Super, Ctrl+Alt+arrow keys, Ctrl+Alt+T.

  On Windows, AltGr arrives as Ctrl+Alt, so in-app defaults don't use Ctrl+Alt+<a key that types a character>. The E26-S3 global hotkey is the one catalogue-named exception. Shortcuts that users assign (workspaces in E06, customisation in E22-S1) can't take a reserved chord either.
- **Conflicts.** Within one view type, two commands never share a default chord, and an application-level command holds its chord in every view type. Where a catalogue shortcut clashes with a platform habit in a view type, the catalogue shortcut wins there. A command this list doesn't cover gets a default only when its story names one or a platform standard exists. Otherwise it ships without one, and users can assign one (E22-S1).
- **Showing chords.** Menus, tooltips and accessibility nodes show a chord in the current platform's form (craft-keymap `Chord::display`: ⇧⌘Z on macOS, Ctrl+Shift+Z elsewhere). Every application or view command with a default chord also has a menu entry, except the hidden-by-default commands the catalogue names. Keys that act on the focused list, pane or dialog (F2 and Del in manager lists, Space, Shift+Left/Right, Ctrl+arrow, Esc), focus and menu navigation, and the text-editing chords need none. Single-key shortcuts (Space, F2, Del, Insert, Esc) can be remapped or removed through E22-S1.

**Status-bar field model (Q25)**
- The shell owns the status bar and shows the active view's fields; views never draw it. Switching tabs swaps the fields. Home fills none, so its bar is empty.
- A view publishes an ordered list of fields. Each field has an id, a translated label and a text value. Every field is optional, and they show in this order:
  1. **Overall result**: the view's own words, such as Text Compare's result or "Binary differences" in Hex Compare.
  2. **The view's own fields**, in the view's order, such as cursor line and column, or Folder Compare's scan progress.
  3. **Insert/overwrite**: "Insert" or "Overwrite", filled only by views that edit. Activating the field toggles it, the same as the Insert key.
  4. **Load time**: how long the last load and comparison took, in seconds. It is set when a load finishes and kept until the next one.
- The text carries the meaning. A view may colour the result with a registered colour role, but never with colour alone.
- Accessibility: each field is a text node whose name starts with its label, and the insert/overwrite field is a button. A change of the overall result that happens away from focus, such as at the end of a load or recompare, is announced through a live region. Load-time changes are not announced.
- Each view's own contents are decided in its story: Text Compare in E07-S1-T1 (Q25), Folder Compare in E08-S1-T7, Table Compare in E15-S1-T6 and Hex Compare in E16-S1-T6.

#### Tasks
- **E01-S1-T1** — Decide the default keymap conventions and status-bar field model (Q24, Q25) · size: S · depends: — · flags: `decision` · status: done
  - AC: a decision in this story lists platform-standard shortcuts, the product shortcuts named in the catalogue (Ctrl+R/Ctrl+L copy, F2 and Del in managers, Space to mark, Shift+arrow to limit a selection, Esc to close file views) and how views fill status-bar fields (overall result, insert/overwrite, load time). Proposed default: follow each OS's standard shortcuts and keep the product shortcuts named in the catalogue.
  - Implemented: a `#### Decisions` section in this story records Q24 (modifier mapping with Ctrl as Cmd on macOS, the platform-standard shortcuts per OS, the catalogue's product shortcuts and where each applies, Esc precedence, reserved chords from craft-keymap's `reserved::system` at the pinned rev, the conflict rule and chord display) and Q25 (the shell owns the status bar; the active view publishes ordered optional fields: overall result, view-defined fields, insert/overwrite, load time; accessibility exposure), as confirmed by the operator on 2026-11-25. `backlog/00-index.md` and `backlog/PRD.md` mark Q24 and the E01-S1 part of Q25 decided. No code. `cargo xtask ci` passed on macOS, and the pull request's CI passed on Linux and macOS. https://github.com/alextiboldi/overthediff/pull/5
- **E01-S1-T2** — Single-instance launch, argument hand-off and argument dispatch · size: M · depends: — · status: needs-verification
  - AC: two launches in quick succession end with one running instance, which handled both argument sets. An unclaimed argument gives the usage message and exit code 100. With a running instance, a run-in-launching-process test handler exits with its own code and nothing reaches the instance. A forward-and-wait test handler's launcher exits only after the test view closes, with the code the handler reported. A launch requesting forward-and-wait for a default test handler waits for its view to close, and a launch requesting a new instance starts a second process. A test handler that hands off from the launching process to forward-and-wait opens its view in the running instance, and the launcher exits after it closes. Content attached by a test handler reaches the running instance. With a switch handler and a bare-path handler both matching, only the switch handler runs. A run-in-launching-process test handler can show its own progress window while no main window opens.
  - Implemented: the UI-free crate `crates/launch` (`otd-launch`, on craft-ipc at the pinned rev): `Handler<S>` (switch or bare-paths pattern, default mode forward-and-exit, run-here or forward-and-wait, usage line, `attach`, `run_here` with hand-off to forward-and-wait, `open` in the instance), `Registry::claim` (a switch handler claims the whole line before bare paths; an unclaimed argument gives the usage message and exit code 100) and `launch`, which elects the instance by an exclusive lock on `otd-<major>.primary` in the per-user runtime folder, publishes a craft-ipc record, forwards `otd.open` requests (arguments, working folder, attachment, wait) and lets waiting launchers exit with the code reported when the view closes. `apps/otd/src/main.rs` calls it before any window opens, and `apps/otd/src/handlers.rs` is the one-line-per-feature handler list. Tests in `crates/launch/tests/it/launch.rs` cover each criterion, plus a craft-testkit hostile-input sweep of the request decoder. `cargo xtask ci` passed on macOS after rebasing onto `main`, the full-matrix CI run passed on Linux, macOS and Windows, and the pull request's CI passed. https://github.com/alextiboldi/overthediff/pull/16
  - Verify by hand: once a run-here handler with its own progress window exists (the first is E11-S2's `/qc`), launch it with no instance running and confirm its progress window shows while no main window opens.
- **E01-S1-T3** — Main window with tab strip, Home launcher tab and status bar · size: M · depends: E01-S1-T2 · status: needs-verification
  - AC: New Tab, Close Tab and tab switching work. Home lists registered session types. Status-bar fields are filled from the active view.
  - Implemented: the UI crate `crates/ui` (`otd-ui`): `Shell` (tab strip as a TabList of Tab nodes with selected state and position, a close button per tab and a New Tab button drawn from Lucide's `x` and `plus`; `new_tab`, `close_tab`, `next_tab`, `previous_tab` and `open(view)` for launch handlers), the Home launcher tab with one button per registered `SessionType`, and the shell-drawn status bar filled from the active `View`'s `Status` in Q25's order, the overall result as a polite live region. The Q24 tab shortcuts (Cmd/Ctrl+T, Cmd/Ctrl+W, Ctrl+Tab and Ctrl+Shift+Tab, Cmd+Shift+] and [ on macOS, Ctrl+F4 on Windows) need their exact modifiers, and a `CtrlTab` egui plugin keeps Ctrl+Shift+Tab from also moving focus. Focus moves to the selected tab after every tab action. `apps/otd/src/session_types.rs` is the one-line-per-feature session-type list; the app enables eframe's `default_fonts`, with egui's fonts and the Lucide icons in `NOTICE` and `ATTRIBUTION.md`. Keyboard and accessibility-tree tests in `crates/ui/tests/it/shell.rs` cover each criterion. `cargo xtask ci` passed on macOS after rebasing onto `main`, the full-matrix CI run passed on Linux, macOS and Windows, and the pull request's CI passed. https://github.com/alextiboldi/overthediff/pull/49
  - Verify by hand: launch `otd` with no instance running and confirm the window opens on Home with the tab strip and an empty status bar, and that text draws in egui's default fonts; then, with a screen reader and once a real view exists (E07-S1 or E08-S1), confirm tabs read as "Home, tab, selected, N of M" and that a change of the view's overall result is announced.
- **E01-S1-T4** — Command registry driving menus, toolbars and shortcuts per view type · size: L · depends: E01-S1-T3 · status: needs-verification
  - AC: switching to another view type swaps the menu and toolbar contents. Disabled, hidden-by-default and dynamic-caption commands behave as specified. On macOS, Options sits in the application menu.
  - Implemented: the UI-free command model `otd_core::command` in `crates/core/src/command.rs` (`Command<C>`: stable id, English caption, Lucide icon name, `Views` it applies to, `Placement` as a menu path, `Hidden` or `Application` for the macOS application menu and Tools or Help elsewhere, a toolbar flag, default `Shortcut`s per platform, a `state` function giving enabled, dynamic caption and dynamic icon, and `run`), and the shell side in `crates/ui/src/commands.rs` (`otd-ui`): New Tab, Close Tab, Next Tab and Previous Tab become the shell's own commands on their Q24 chords; `Shell::with_commands`, `Shell::run(id)` (runs only an applicable, enabled command, and none while the Session Settings dialog is open) and `Shell::menu_layout(os)`; the in-window menu bar (craft-menu), a Toolbar row of buttons, and shortcut dispatch through a craft-keymap keymap with one scope per view type, the OS's reserved chords left out, and shortcuts waiting only while a text field has the keys or a modal dialog is open. `View::view_type` names a view's type and `Shell::view_mut::<V>()` lets a command reach its view. `apps/otd/src/commands.rs` is the one-line-per-feature command list, whose unit test fails on a shared default chord within a view type, a shadowed application-wide chord, a repeated id or a reserved chord; on macOS `apps/otd/src/main.rs` installs and syncs the native menu bar, keeping the in-window bar if AppKit refuses it. Captions and menu titles go through the translation lookup. craft-keymap, craft-menu and craft-quit come from the pinned craft-libs rev, recorded in `NOTICE` and `ATTRIBUTION.md`, and AGENTS.md records how a feature uses the extension point. Keyboard and accessibility-tree tests in `crates/ui/tests/it/commands.rs` cover each criterion (view-type swap, disabled, hidden-by-default, dynamic caption, application-menu placement, shortcuts while typing, keyboard path into the in-window bar). `cargo xtask ci` and `cargo check --target x86_64-pc-windows-msvc -p otd` passed on macOS, the pull request's CI passed on Linux and macOS, and the full-matrix CI run passed on Linux, macOS and Windows. https://github.com/alextiboldi/overthediff/pull/56
  - Verify by hand: on macOS, launch `otd` and confirm the native menu bar shows the OverTheDiff application menu, Session (New Tab ⌘T, Close Tab ⌘W) and Window (Next Tab ⌃⇥, Previous Tab ⌃⇧⇥), that choosing each item and pressing each key equivalent acts on the tabs, and, once E01-S1-T6 adds Options, that it appears in the application menu; on Windows and Linux, launch `otd` and confirm the window draws its own menu bar, that Alt or F10 moves the keyboard to it, and that arrow keys and Enter run an item; once a real command with a changing icon exists, confirm by eye that its toolbar icon changes with its dynamic caption.
- **E01-S1-T5** — Settings store in a per-user, per-major-version folder with atomic writes and corrupt-file recovery · size: M · depends: — · flags: `craft-libs`
  - AC: killing the process during a write leaves the previous settings readable. A deliberately corrupted file is set aside, and the app starts with defaults and a notice.
- **E01-S1-T6** — Options dialog shell with contributed pages and options · size: M · depends: E01-S1-T4, E01-S1-T5
  - AC: an option contributed by a test feature appears on its page and persists after Apply and a restart. An uncontributed page doesn't show. Cancel restores the previous values.
- **E01-S1-T7** — Session model and Session Settings dialog shell (contributed tabs, Specs tab with Description, "Use for this view only" scope), New Session, Open Session and Clear Session · size: L · depends: E01-S1-T3 · flags: `naming-check` · status: needs-verification
  - AC: a test session type registered for a test format kind is returned when the shell looks up that kind. A test session type contributing two tabs shows Specs plus those two tabs. Two test features contributing options to the same named tab show one tab holding both sets of options. A change applies only to the current view. Clear Session gives an empty session of the same type.
  - Implemented: the UI-free session model `otd_core::session` in `crates/core/src/session.rs` (`Session`: type id, one input per side, rules by tab and setting id, description; `Setting` and `Value`; `check_unique`, which rejects two settings with one id on one tab of one session type as `DuplicateSetting`), and the shell side in `crates/ui/src/session.rs` (`otd-ui`): `SessionType` gains the format kinds it opens (`Shell::session_type_for`, the first registered type wins) and its sides' captions; the Session Settings dialog, titled Comparison Settings, shows the Inputs tab (the backlog's Specs tab, with an input per side and the Description) and then the contributed tabs in first-contributed order, settings naming the same tab sharing it, with the "Use for this view only" scope; OK writes its copy back to the view it was opened on only, Cancel and Esc drop it, focus stays inside it and the window's shortcuts wait while it is open; `edit_value` is the editor for each value type; and `Shell::new_session`, `open_session`, `clear_session` and `session_settings`, which return a `SessionError` when there is nothing to act on, for E01-S1-T4 to bind to the Session menu and the Rules toolbar button. `apps/otd/src/session_settings.rs` is the one-line-per-feature settings list; the app exits with an error, never a panic, on a duplicate setting. All dialog text goes through the translation lookup. The naming check is recorded in `docs/feature-names.md` (Comparison Settings, Inputs, Use for this view only), and AGENTS.md records how a feature uses the extension point. Keyboard and accessibility-tree tests in `crates/ui/tests/it/session.rs` cover each criterion, plus focus, outside-click, shortcut, value-editor and pseudo-locale checks. `cargo xtask ci` passed on macOS after rebasing onto `main`, and the pull request's CI passed on Linux and macOS (no platform-dependent code, so no full-matrix run). https://github.com/alextiboldi/overthediff/pull/54
  - Verify by hand: once E01-S1-T4 binds the commands, open Session > Session Settings and the Rules toolbar button in the running app on a real session type and confirm the dialog opens with Inputs and the contributed tabs, and that Clear Session, Open Session and New Session work from the menu; with a screen reader, confirm the dialog reads as "Comparison Settings, dialog", its tabs read with selected state and position, and each field reads by its label.
- **E01-S1-T8** — Background-task service with progress, cancel and end-of-task error summary · size: M · depends: E01-S1-T3 · status: needs-verification
  - AC: a test task reporting progress can be cancelled midway, and a task with three failing items lists all three in the summary.
  - Implemented: the UI-free task engine `otd_core::task` in `crates/core/src/task.rs` (`Task::spawn(title, cancellable, work)` runs the work on its own thread; its `Worker` reports `progress(done, total)`, `fail(item, message)` for each failed item and `cancelled()` at each safe point; a task ends `Completed`, `Cancelled` (only when the work saw the request) or `Failed` (item errors, or a caught panic), with every item error in order, and `wait()` returns the outcome without a window), and the shell side `otd_ui::show_task(ctx, task)` in `crates/ui/src/tasks.rs`: a progress window per running task (a ProgressIndicator named "N of M" and Cancel, disabled once pressed), and, when a task ends cancelled or with errors, a modal summary dialog named after the task that lists every failed item, takes focus on Close, gives it back on Esc or Close and blocks the tab shortcuts while open; a clean completion only closes the progress window. All its text goes through the translation lookup (`trn` for the failed-item count in `crates/core/src/i18n.rs`). Nothing needs registering; AGENTS.md records how a feature uses it. Tests in `crates/core/tests/it/task.rs` and `crates/ui/tests/it/tasks.rs` drive each criterion by keyboard and the accessibility tree, plus never-reported-cancelled, panic, focus-restore, one-window-per-task and pseudo-locale lookup checks. `cargo xtask ci` passed on macOS, the pull request's CI passed on Linux and macOS, and the full-matrix CI run on the branch passed on Linux, macOS and Windows. https://github.com/alextiboldi/overthediff/pull/52
  - Verify by hand: with a screen reader (VoiceOver, Narrator or Orca), run a task that ends with failed items and confirm the summary is read as a modal dialog named after the task and that focus lands on Close; the tree tests cover the roles and names, not what a screen reader actually says.
- **E01-S1-T9** — Path edit control with typed path, Browse for File/Folder and a recent-paths dropdown · size: S · depends: E01-S1-T5 · flags: `craft-libs`
  - AC: recent paths persist per field across restarts, and Browse returns the chosen path into the edit.
- **E01-S1-T10** — Unsaved-changes prompts on Close Tab and Exit · size: S · depends: E01-S1-T3
  - AC: a test view flagged as unsaved triggers Save / Don't Save / Cancel. Cancel keeps the view open, and Exit prompts once for each unsaved view.
- **E01-S1-T11** — Colour roles with light and dark values · size: S · depends: E01-S1-T3 · status: done
  - AC: a harness view drawing with a registered role shows its light value. Switching the active palette from code re-colours the view without a restart.
  - Implemented: the UI-free colour-role table in `crates/core/src/colour.rs` (`otd-core`: `Rgba`, `Palette` with `Light` by default and `Dark`, `ColourRole` with a stable id and light and dark values, and `ColourRoles`, which rejects a second role with the same id by name and iterates for E05-S1's contrast check), and `crates/ui/src/colour.rs` (`otd-ui`: `install`, `set_palette`, which requests a repaint, `palette` and `colour(ctx, &ROLE)`, which keeps the table and the active palette in the egui context and draws an unregistered role as the conspicuous `UNREGISTERED`). `apps/otd/src/colour_roles.rs` is the one-line-per-feature role list; the app installs it when the window opens and exits with an error, never a panic, on a duplicate id. Tests in `crates/ui/tests/it/colour.rs` drive a harness view in a `Shell` tab for each criterion, plus the duplicate-id tests in `otd-core` and the app. `cargo xtask ci` passed on macOS, and the pull request's CI passed on Linux and macOS (no platform-dependent code, so no full-matrix run). https://github.com/alextiboldi/overthediff/pull/51
- **E01-S1-T12** — Commands with typed arguments and results, runnable by id without the window · size: M · depends: E01-S1-T4 · status: done
  - AC: a test command run by id with valid arguments returns its result in a headless test. Missing, unknown or out-of-range arguments return an error naming the argument. A test command declared as needing the window reports that when run headlessly.
  - Implemented: typed commands in `otd_core::command` (`crates/core/src/command.rs`, UI-free): `Command::typed(id, caption, PARAMS, run)` declares `Param`s (name, `Kind`: bool, number with an inclusive range, choice, text, path or colour; required or optional) and returns an `Outcome` with an optional `otd_core::session::Value`; `.needs_window()` marks commands that act on the window; `otd_core::command::run(commands, ctx, id, args, window_open)` runs one by id and returns a `CommandError`, never a panic, for an unknown or disabled command, a command needing the window when none is open, and a missing, unknown, repeated, wrong-type or out-of-range (NaN included) argument, each argument error naming the argument through the translation lookup. `Command::bind` does the same for a command list inside its context; `Shell::run(id)` in `crates/ui/src/commands.rs` now goes through it with the window open, and the four tab commands are marked `needs_window`. AGENTS.md records how a feature declares a typed command and runs it without the window. Headless tests in `crates/core/tests/it/command.rs` cover each criterion plus unknown and disabled commands. `cargo xtask ci` passed on macOS, and the pull request's CI passed on Linux and macOS (no platform-dependent code, so no full-matrix run). https://github.com/alextiboldi/overthediff/pull/59
- **E01-S1-T13** — Translation lookup for captions, labels and messages · size: S · depends: E01-S1-T3 · status: done
  - AC: a harness caption with a test translation shows translated, and one without shows its English key. No caption in the shell is a literal outside the lookup.
  - Implemented: the translation lookup `otd_core::i18n` in `crates/core/src/i18n.rs` (`t(english)`, `fmt` for `{name}` placeholders, `measure` for numbers with units, and a per-thread language set with `set_language`), on craft-libs' `craft-i18n` at the pinned rev. `crates/core/build.rs` embeds the catalogs from an optional storytold/craft-i18n checkout named by `CRAFT_I18N_DIR` (English only without it; `CRAFT_I18N_REQUIRED=1` makes that an error). Every shell caption, label, accessible name and status-field text in `crates/ui/src/lib.rs` goes through `t`, with session type names as English keys; the app sets the system language on the UI thread. AGENTS.md rule 16 records how a feature uses it, and `NOTICE` and `ATTRIBUTION.md` list craft-i18n, the ICU4X crates and CLDR data. Tests in `crates/ui/tests/it/translation.rs` cover each criterion: a one-locale test catalog for the translated and English-key captions, and craft-i18n's pseudo locale to show that no text in the shell skips the lookup. `cargo xtask ci` passed on macOS after rebasing onto `main`, `cargo check --target x86_64-pc-windows-msvc -p otd-core -p otd-ui` passed, and the pull request's CI passed. https://github.com/alextiboldi/overthediff/pull/53

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
