# E06 — Sessions and workspaces

**Group**: feature
**Label**: `area:sessions`
**Size**: L
**Depends on**: E01 (foundation); E05, E07, E08, E09, E15, E16, E17
**Source**: catalogue §6, F-5.2, F-12.8, F-12.10, F-25.2, F-26.1, F-26.2, F-26.11, Appendix C. Spec-derived from the feature catalogue and not yet validated. Items marked *inferred* are analyst or PO inferences.

This epic treats comparisons as reusable sessions. Users save, find, lock and reopen them, set defaults for each session type, and restore whole window layouts as workspaces. It also covers how folder sessions hand file pairs to child sessions, which turns one-off comparisons into repeatable tasks.

## Behavioral contract
- **Data**:
  - *Session*: name (named sessions only), type, specs, every Session Settings value, description, folder (named sessions), locked flag, remembered display filter and last-used time. Auto-saved sessions have no name and are listed by their specs (*inferred*).
  - *Session defaults*: one settings set per session type. Specs and description are not part of it (*inferred*).
  - *Child-session override*: held by a parent folder session, either for one file pair or for every new child of a given type.
  - *Workspace*: name, each window's size and position, the tabs in order, and each tab's session. A workspace can have a keyboard shortcut. There is also a startup workspace name and an exit workspace name.
- **Routes/endpoints**:
  - Session menu: Save Session, Save Session As, Lock and Unlock, the Session Settings scope dropdown, Compare Files Using, Compare Parent Folders, Save Workspace As, Load Workspace and Manage Workspaces.
  - Home sessions tree: the New, Saved, Auto-saved and folder branches, with right-click Edit, Rename, Delete and Lock.
  - View menu: Show Session Management and Maximize Session Management.
  - Options: the Startup and Tabs pages, and the Tweaks *Max auto-saved sessions* and *Sticky session selection*.
  - Command line: a session name or a workspace name as the positional argument.
  - Child-session commands in file views.
- **Integrations**:
  - E01-S1 session model, Session Settings dialog, settings store, command-line dispatch and Home launcher.
  - Folder and file views provide specs, settings, display filters and child sessions.
  - E05 multiple windows, for workspaces and child-session placement.
- **Rules/states**:
  - Session settings are stored with saved sessions.
  - Changing a named session's primary parameters (its specs) prompts to save on exit.
  - Auto-saved sessions are capped by *Max auto-saved sessions*. The oldest go first, and 0 turns auto-save off.
  - The display filter is saved when a session is left, except for locked sessions.
  - Scope choices are *Use for this view only*, *Update session defaults*, *Use for these files within parent session* and *Use for all files within parent session*. The last two appear in child sessions only.
  - If editing is disabled in a parent folder session, its child sessions are read-only.

---

## Stories

### E06-S1 — Save and reopen sessions
**Priority**: Must · **Layer**: 2 · **Size**: L · **Platforms**: All
**Needs**: E01-S1 (session model, settings store, Home launcher), E07-S1 (Text Compare view), E08-S1 (Folder Compare view)
**Traces**: F-6.1, F-5.2, F-25.2, F-26.11
**Not in this story**: folders and search → E06-S3; lock and display-filter memory → E06-S4; management panel, editing defaults from Home and sticky selection → E06-S5; shared sessions branch → E23-S3; workspaces → E06-S6.

As a developer who compares the same folders every week, I can save a comparison as a named session and reopen it from Home so that I don't have to set it up again.

**Acceptance criteria**
- [ ] *Session > Save Session As* asks for a name, then saves the session's type, specs, description and every Session Settings value. *Save Session* saves a named session in place, and behaves like Save Session As for an unnamed one.
- [ ] Saving under a name that already exists asks before overwriting (*inferred*).
- [ ] Home shows a sessions tree with three branches:
  - *New*: one entry per session type. Double-clicking it opens a new session with that type's current defaults.
  - *Saved*: named sessions.
  - *Auto-saved*: recent unnamed sessions.
- [ ] Recent sessions are auto-saved when their view closes (*inferred* trigger). The tweak *Max auto-saved sessions* caps the Auto-saved branch, dropping the oldest entries first. A value of 0 turns auto-save off.
- [ ] Double-clicking a saved or auto-saved session opens it in its view with its specs and settings restored.
- [ ] Right-clicking a saved session offers *Rename* and *Delete*. Delete asks first (*inferred*).
- [ ] On exit, the app prompts to save each open named session whose primary parameters (specs) changed since it was opened or last saved.
- [ ] Passing a saved session's name as the command-line argument opens that session in its view. An existing path wins over a session with the same name (*inferred* precedence). An argument that is neither gives the usage error.

- [ ] Acceptance includes a settings tab contributed by a test harness. Every registered setting round-trips through save and reopen, whichever real settings exist yet.
#### Tasks
- **E06-S1-T1** — Store named sessions in the settings store · size: M · depends: — · status: done
  - AC: a saved Folder Compare session and a saved Text Compare session reload after a restart with identical type, specs, description and settings. A corrupt session file is skipped with a warning, and the other sessions still load (*inferred*).
  - Implemented: `otd_settings::sessions` (`crates/settings/src/sessions.rs`) saves each named session as its own TOML file in the settings folder's `sessions` folder (name, type, specs, description and settings by tab), written crash-safely with `craft-atomic`; file names encode the name so no two names share a file on a case-insensitive file system and none reaches outside the folder; `load` reads settings against the registered ones for the session's type, and skips a corrupt file with a warning, moving it aside so a later save can't replace it, while the other sessions still load. Tests in `crates/settings/tests/it/sessions.rs` and the hostile-input sweep (`tests/it/hostile.rs`). Checks: `cargo xtask ci` (fmt, clippy, tests, ui-free) on macOS, pull-request CI on Linux and macOS, and the full matrix with Windows on the branch. https://github.com/alextiboldi/overthediff/pull/130
- **E06-S1-T2** — Save Session and Save Session As, with overwrite confirmation · size: S · depends: E06-S1-T1
  - AC: Save Session on an unnamed session prompts for a name. Saving under an existing name asks before overwriting.
- **E06-S1-T3** — Home sessions tree with New, Saved and Auto-saved branches · size: M · depends: E06-S1-T1
  - AC: the New branch lists every registered session type, and double-clicking any entry opens the right view. The tree updates when a session is saved, without reopening Home.
- **E06-S1-T4** — Auto-save recent sessions within the *Max auto-saved sessions* cap · size: S · depends: E06-S1-T1
  - AC: with the cap at 2, closing three sessions keeps the two newest. With the cap at 0, closing a session adds nothing.
- **E06-S1-T5** — Rename and Delete from the sessions tree · size: S · depends: E06-S1-T3
  - AC: a renamed session opens under its new name. A deleted session disappears from Home and from the settings store.
- **E06-S1-T6** — Prompt on exit for named sessions whose specs changed · size: S · depends: E06-S1-T2
  - AC: changing the left folder of a named session and then exiting prompts to save. Exiting without changes doesn't prompt.
- **E06-S1-T7** — Open a saved session by name from the command line · size: S · depends: E06-S1-T1
  - AC: passing a saved session's name opens it, including when the app is already running. An unknown name gives the usage error.

### E06-S2 — Session defaults per type
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E01-S1 (Session Settings dialog), E07-S1 (Text Compare view), E08-S1 (Folder Compare view)
**Traces**: F-6.2
**Not in this story**: editing defaults from Home → E06-S5; child scopes → E06-S7.

As a team lead who wants every folder comparison to use our standard rules, I can make a session's settings the default for its type so that every new session of that type starts with them.

**Acceptance criteria**
- [ ] The Session Settings scope dropdown offers *Update session defaults* in addition to *Use for this view only*.
- [ ] With *Update session defaults* selected, OK applies the settings to the current view and stores them as the defaults for that session type.
- [ ] Every new session of that type starts with the stored defaults, whether it is opened from Home, the Session menu or the command line.
- [ ] Defaults are per type: updating Text Compare defaults doesn't change new Folder Compare sessions.
- [ ] *Use for this view only* changes the current view and leaves the defaults untouched.
- [ ] Defaults cover the session settings but not the specs or description (*inferred*), and they persist across restarts.

- [ ] Acceptance uses settings tabs contributed by a test harness on Text Compare and Folder Compare, so it doesn't depend on which real settings exist yet.
#### Tasks
- **E06-S2-T1** — Store and load one settings set per session type · size: S · depends: — · status: done
  - AC: defaults written for Folder Compare survive a restart and never apply to Text Compare.
  - Implemented: `session_defaults::read(store, settings, type_id)` and `write(store, session)` in `crates/settings/src/session_defaults.rs` (`otd-settings`) keep each session type's settings in the settings store's `session-defaults` section as `[session-defaults.<type>.<tab>]` tables, sharing the Options dialog's value codec (`options::to_toml`/`from_toml`); only settings are stored, never specs or description, and a missing or ill-fitting stored value falls back to the setting's own default. Tests in `crates/settings/tests/it/session_defaults.rs` (restart survival per type, isolation from Text Compare, reset back to defaults) and the hostile-input sweep. Checks: `cargo xtask ci` on macOS passed after rebasing onto `main`; pull-request CI on Linux and macOS passed. Pull request: https://github.com/alextiboldi/overthediff/pull/129
- **E06-S2-T2** — *Update session defaults* in the scope dropdown · size: S · depends: E06-S2-T1
  - AC: choosing it and pressing OK updates both the view and the stored defaults. Cancel changes neither.
- **E06-S2-T3** — New sessions start from their type's defaults · size: S · depends: E06-S2-T1 · status: done
  - AC: after Folder Compare defaults are updated, a Folder Compare opened from Home and one opened from the command line both show the new settings, with specs only from their own inputs.
  - Implemented: `Shell::load_session_defaults(store)` and `apply_session_defaults` in `crates/ui/src/session_defaults.rs` (`otd-ui`) read each session type's stored defaults with `otd_settings::session_defaults::read` (E06-S2-T1); `Shell::prepare` in `crates/ui/src/session.rs`, which every new view goes through (Home, New Session, Clear Session, command-line opens), applies them before the settings' own defaults, never to specs or description; `apps/otd/src/main.rs` loads them at start. Test in `crates/ui-folder-compare/tests/it/defaults.rs`: defaults written and reloaded, then Home's Folder Compare and an `otd <left> <right>` launch both open with the new value and only their own specs. Hookup left for E06-S2-T2: after it writes the defaults, the app must call `shell.load_session_defaults` so later sessions in the same run see them. Checks: `cargo xtask ci` on macOS passed; `cargo check --target x86_64-pc-windows-msvc -p otd-ui -p otd-ui-folder-compare --all-targets` passed; Linux and Windows tests run in main's CI. Merged straight into `main` (no pull request).

### E06-S3 — Session folders and search
**Priority**: Should · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E06-S1 (saved sessions and the Home sessions tree)
**Traces**: F-5.2, F-6.1

As a QA engineer with dozens of saved comparisons, I can group sessions into folders and search them so that I find the one I need quickly.

**Acceptance criteria**
- [ ] In the Saved branch, the user can create, rename and delete session folders. Folders can nest (*inferred*).
- [ ] A session can move into or out of a folder by drag and drop or a Move command (*inferred* commands). Save Session As can save straight into a folder.
- [ ] Deleting a folder that holds sessions asks first, then deletes the folder and its sessions (*inferred*).
- [ ] A search box above the sessions tree filters every branch to sessions whose name contains the typed text, ignoring case (*inferred*). Clearing the box restores the full tree.
- [ ] Folders and their contents persist across restarts. A session opens from a folder the same way it opens from the Saved branch.

#### Tasks
- **E06-S3-T1** — Session folders: create, rename, delete and nest · size: M · depends: —
  - AC: a folder holding two sessions survives a restart. Deleting it, after confirming, removes both sessions.
- **E06-S3-T2** — Move sessions between folders, and Save Session As into a folder · size: S · depends: E06-S3-T1
  - AC: a session dragged into a folder opens from there with unchanged settings.
- **E06-S3-T3** — Search box that filters the sessions tree · size: S · depends: —
  - AC: typing "release" shows only sessions with that text in their name, with their folders expanded. Clearing the box shows everything again.

### E06-S4 — Lock sessions and remember display filters
**Priority**: Should · **Layer**: 3 · **Size**: S · **Platforms**: All
**Needs**: E06-S1 (saved sessions), E07-S3 (Text Compare display filters), E08-S2 (Folder Compare display filters)
**Traces**: F-6.1

As a team lead who shares a carefully set-up session, I can lock it so that nobody changes it by accident, and every session reopens with the display filter I last used.

**Acceptance criteria**
- [ ] *Lock* and *Unlock* are on the Home right-click menu and the Session menu for named sessions. A locked session shows a lock indicator in the sessions tree (*inferred*).
- [ ] A locked session can't be overwritten. Save Session is unavailable, and the exit prompt for changed specs doesn't offer to save over it. Save Session As still works (*inferred* protection).
- [ ] When the user leaves a session (closes its view or loads another session in its tab), its current display filter is saved with it. Reopening the session restores that filter, in both Text Compare and Folder Compare.
- [ ] A locked session doesn't save display-filter changes: it reopens with the filter it had when it was locked.
- [ ] Unlocking restores normal saving.

#### Tasks
- **E06-S4-T1** — Lock and Unlock commands, with a lock indicator · size: S · depends: —
  - AC: the lock state survives a restart, and the indicator shows in the sessions tree.
- **E06-S4-T2** — Locked sessions refuse in-place saves · size: S · depends: E06-S4-T1
  - AC: changing a locked session's settings and then exiting leaves the saved session unmodified. Save Session As creates a new, unlocked session.
- **E06-S4-T3** — Save the display filter when leaving a session, and restore it on reopen · size: S · depends: —
  - AC: a Folder Compare left on Show Orphans reopens on Show Orphans. A Text Compare left on Show Differences reopens on Show Differences.
- **E06-S4-T4** — Locked sessions keep the display filter they had when locked · size: S · depends: E06-S4-T1, E06-S4-T3
  - AC: lock a session on Show All, switch to Show Differences, then close and reopen it: it shows Show All.

### E06-S5 — Session management panel and editing defaults from Home
**Priority**: Should · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E06-S1 (Home sessions tree), E06-S2 (session defaults)
**Traces**: F-5.2, F-26.11 · **Open questions**: Q20
**Not in this story**: Web Resources panel (View > Show Web Resources). It is not built and is a non-goal (Q20).

As a team lead maintaining shared settings, I can edit a saved session or a type's defaults straight from Home so that I can adjust them without loading large folders.

**Acceptance criteria**
- [ ] Right-clicking a saved session and choosing *Edit* opens the session management panel on Home, showing that session's settings on the same tabs as Session Settings. Saving the panel updates the saved session without opening it.
- [ ] *Pin Session Management Panel* keeps the panel visible. While it is pinned, selecting another session in the tree shows that session's settings.
- [ ] *View > Show Session Management* shows or hides the panel, and *View > Maximize Session Management* gives it the whole Home area.
- [ ] Under the New branch, right-clicking a session type and choosing *Edit* opens that type's defaults in the panel. Saving them changes the defaults for every future session of that type.
- [ ] With the tweak *Sticky session selection* on, Home keeps the session the user last picked selected, instead of the session updated most recently.
- [ ] The Web Resources panel is not built (Q20), so there is no *Show Web Resources* command.

#### Tasks
- **E06-S5-T1** — Session management panel that edits a saved session's settings without opening it · size: M · depends: — · flags: `naming-check`
  - AC: settings changed in the panel for a saved Folder Compare session are there when the session is opened, and no comparison runs while the panel is used.
- **E06-S5-T2** — Pin, Show and Maximize Session Management · size: S · depends: E06-S5-T1
  - AC: a pinned panel follows the tree selection. Maximize hides the sessions tree until the panel is restored.
- **E06-S5-T3** — Edit a type's defaults from the New branch · size: S · depends: E06-S5-T1
  - AC: editing Text Compare defaults in the panel changes the settings of the next new Text Compare session.
- **E06-S5-T4** — *Sticky session selection* tweak · size: S · depends: —
  - AC: with the tweak on, after the user picks session A and session B is then saved, Home shows A selected on return. With the tweak off, it shows B.

### E06-S6 — Workspaces
**Priority**: Should · **Layer**: 3 · **Size**: L · **Platforms**: All
**Needs**: E06-S1 (saved sessions), E05-S2 (multiple windows)
**Traces**: F-6.5, F-26.1, F-25.2

As a developer who works with the same set of comparisons across two monitors, I can save and restore my whole window layout as a workspace so that I pick up exactly where I left off.

**Acceptance criteria**
- [ ] *Save Workspace As* takes a new unique name, or overwrites an existing workspace picked from the list. It stores every window and tab, their sizes and positions, and each tab's session.
- [ ] *Load Workspace* lists the saved workspaces. Choosing one closes all current views, after the unsaved-changes prompts, and restores the saved windows, tabs and sessions.
- [ ] Tabs holding unnamed sessions come back with their specs and settings (*inferred*). Home tabs come back as Home.
- [ ] *Manage Workspaces* renames (F2) and deletes (Del) workspaces, and assigns each one a keyboard shortcut that loads it.
- [ ] *On startup, load* and *On exit, save as* can be set in Manage Workspaces and on the Startup options page, and both places show the same values. Using the same name for both reopens the last state automatically.
- [ ] Passing a workspace name as the command-line argument opens that workspace.
- [ ] A window that would be restored off-screen, for example on a monitor that is no longer connected, is moved onto a visible screen (*inferred*).

#### Tasks
- **E06-S6-T1** — Workspace model and storage: windows, sizes, positions, tabs and sessions · size: M · depends: —
  - AC: a workspace with two windows, holding three tabs and two tabs, round-trips through save and load with the same layout.
- **E06-S6-T2** — Save Workspace As and Load Workspace · size: M · depends: E06-S6-T1 · flags: `naming-check`
  - AC: Load closes the current views after prompting about unsaved edits. Cancelling a prompt stops the load and leaves the current views open.
- **E06-S6-T3** — Manage Workspaces: rename, delete and keyboard shortcut · size: S · depends: E06-S6-T1
  - AC: F2 renames and Del deletes the selected workspace. The assigned shortcut loads the workspace from any view.
- **E06-S6-T4** — *On startup, load* and *On exit, save as*, in the dialog and on the Startup page · size: S · depends: E06-S6-T2
  - AC: with both set to the same name, quitting with two windows open and relaunching restores both windows.
- **E06-S6-T5** — Open a workspace by name from the command line · size: S · depends: E06-S6-T2
  - AC: passing a workspace name opens it. A name that matches both a session and a workspace opens the session (*inferred* precedence).
- **E06-S6-T6** — Keep restored windows on visible screens · size: S · depends: E06-S6-T1
  - AC: a workspace saved with a window on a monitor that is now missing opens that window on the main screen.

### E06-S7 — Child-session settings scope and placement
**Priority**: Should · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E08-S10 (child sessions from folder views), E05-S2 (multiple windows and Tabs options), E06-S1 (saved sessions, which keep the per-pair overrides)
**Traces**: F-6.2, F-26.2, F-6.4

As a developer reviewing a folder of generated files, I can make settings chosen in one file comparison stick for that pair or for every file in the folder session so that I don't have to repeat them for each file.

**Acceptance criteria**
- [ ] In a child session, the Session Settings scope dropdown also offers *Use for these files within parent session* and *Use for all files within parent session*. Sessions not opened from a folder view don't show them.
- [ ] *Use for these files within parent session*: the parent folder session remembers the settings for this file pair and reuses them whenever the pair is compared again from that parent.
- [ ] *Use for all files within parent session*: the parent applies the settings to every new child session of this type it opens. Child sessions already open keep their settings (*inferred*).
- [ ] When the parent folder session is saved, its remembered child settings are saved with it and apply after reopening (*inferred*).
- [ ] The Tabs options page offers *Child sessions open in* a new window or a new tab, and opening a pair from a folder view follows it.

#### Tasks
- **E06-S7-T1** — Child-only scope choices in the scope dropdown · size: S · depends: —
  - AC: the two choices appear only in sessions opened from a folder view.
- **E06-S7-T2** — Per-pair settings remembered by the parent · size: M · depends: E06-S7-T1
  - AC: after a pair's rules are set with "these files", closing and reopening that pair from the parent shows those rules. Another pair keeps the defaults.
- **E06-S7-T3** — Settings for all children of a type · size: S · depends: E06-S7-T1
  - AC: after "all files", every newly opened text pair from that parent starts with the chosen settings.
- **E06-S7-T4** — Save child overrides with the parent session · size: S · depends: E06-S7-T2, E06-S7-T3
  - AC: a saved parent session, reopened after a restart, still applies both kinds of override.
- **E06-S7-T5** — *Child sessions open in* a new window or a new tab · size: S · depends: —
  - AC: with "new window" selected, opening a pair from a folder view opens a new window holding the child session.

### E06-S8 — Disable editing and read-only child sessions
**Priority**: Should · **Layer**: 3 · **Size**: S · **Platforms**: All
**Needs**: E07-S1 (Text Compare view), E08-S10 (child sessions), E09-S1 (copy), E09-S2 (move), E09-S3 (delete), E09-S4 (synchronize selection), E09-S6 (rename), E09-S7 (attributes), E09-S8 (touch)
**Traces**: F-6.6, F-6.4, F-12.3
**Not in this story**: Table and Hex sessions → E15-S5, E16-S2.

As a code reviewer, I can mark a session as not editable so that I can't change the files on disk by accident while I review them.

**Acceptance criteria**
- [ ] The Specs tab of Folder Compare and Text Compare sessions has a *Disable editing* checkbox.
- [ ] In a Text Compare session with editing disabled, both panes are read-only. Typing, paste and the other editing commands are unavailable, Save is disabled, and the view shows that it is read-only (*inferred* indicator).
- [ ] In a Folder Compare session with editing disabled, Copy, Move, Delete, Synchronize, Rename, Attributes and Touch are unavailable (*inferred*). File pairs opened from it open as read-only child sessions.
- [ ] Clearing the checkbox restores editing immediately.
- [ ] The setting is part of the session settings, so it travels with them.

#### Tasks
- **E06-S8-T1** — *Disable editing* checkbox on the Folder Compare and Text Compare Specs tabs · size: S · depends: —
  - AC: the checkbox state is stored with the session settings.
- **E06-S8-T2** — Read-only Text Compare when editing is disabled · size: S · depends: E06-S8-T1
  - AC: with the box ticked, typing does nothing, Save is disabled and a read-only indicator shows. Unticking the box makes the panes editable again.
- **E06-S8-T3** — Folder sessions with editing disabled block disk-changing commands and open read-only children · size: S · depends: E06-S8-T1
  - AC: in such a session, opening a text pair gives a read-only Text Compare. Copy, Move, Delete, Synchronize, Rename, Attributes and Touch are disabled.

### E06-S9 — Switch view type (Compare Files Using)
**Priority**: Should · **Layer**: 2 · **Size**: S · **Platforms**: All
**Needs**: E07-S1 (Text Compare view), E16-S1 (Hex Compare view)
**Traces**: F-12.8, App-C

As a developer whose "text" file turns out to be binary, I can reopen the same files in another view type so that I see the comparison that makes sense for them.

**Acceptance criteria**
- [ ] *Session > Compare Files Using* lists every registered file view type except the current one. In Text Compare it offers Hex Compare.
- [ ] Choosing a type reopens the same two files in that view type in the same tab, keeping the specs (*inferred*: same tab).
- [ ] Before switching, the app offers to save any unsaved edits. Cancelling keeps the current view.
- [ ] The command also works from Hex Compare back to Text Compare.
- [ ] The Text Compare Format tab offers the same *Compare Files Using* choice.

#### Tasks
- **E06-S9-T1** — *Compare Files Using* command listing the other registered file view types · size: S · depends: — · flags: `naming-check`
  - AC: in Text Compare the list contains Hex Compare but not Text Compare. Folder views don't show the command.
- **E06-S9-T2** — Reopen the current specs in the chosen view type · size: S · depends: E06-S9-T1
  - AC: switching Text → Hex → Text shows the same two files each time. With unsaved edits, the save prompt appears first.
- **E06-S9-T3** — *Compare Files Using* on the Text Compare Format tab · size: S · depends: E06-S9-T2
  - AC: choosing Hex Compare there and pressing OK reopens the files in Hex Compare.

### E06-S10 — Compare parent folders from file views
**Priority**: Should · **Layer**: 2 · **Size**: S · **Platforms**: All
**Needs**: E07-S1 (Text Compare view), E08-S1 (Folder Compare view), E15-S1 (Table Compare view), E16-S1 (Hex Compare view), E17-S1 (Picture Compare view)
**Traces**: F-12.10, App-C

As a developer looking at two changed files, I can jump to a comparison of the folders that hold them so that I see what else changed around them.

**Acceptance criteria**
- [ ] *Session > Compare Parent Folders* in Text Compare, Table Compare, Hex Compare and Picture Compare opens a new Folder Compare of the folders holding the left and right files.
- [ ] The Folder Compare opens as a new view, and the file view stays open.
- [ ] If an input on either side isn't a file inside a folder, the command is unavailable (*inferred*).

#### Tasks
- **E06-S10-T1** — Compare Parent Folders command in the four file views · size: S · depends: —
  - AC: from Text, Table, Hex and Picture views, the command opens Folder Compare with the two parent folders as left and right.
- **E06-S10-T2** — Disable the command when a side has no parent folder · size: S · depends: E06-S10-T1
  - AC: for an input without a folder path, the command is greyed out.

### E06-S11 — Child-session commands in Table, Hex and Picture views
**Priority**: Should · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E08-S10 (child sessions), E09-S1 (copy to the other side), E15-S1 (Table Compare view), E16-S1 (Hex Compare view), E17-S1 (Picture Compare view)
**Traces**: F-6.4, F-14.6, F-15.3, F-16.4

As a QA engineer working through a folder of changed spreadsheets, images and binaries, I can move to the next differing file and copy files across from inside any file view so that I don't have to keep going back to the folder view.

**Acceptance criteria**
- [ ] Table Compare, Hex Compare and Picture Compare views opened from a folder view offer *Next Difference Files* and *Previous Difference Files*. These open the parent's next or previous pair of files with differences.
- [ ] The next pair opens in the view its file format chooses, which can be a different view type from the current one (*inferred*). It replaces the current child view (*inferred*).
- [ ] *Copy File to Right and Open Next Difference* and *Copy File to Left and Open Next Difference* copy the current file across, using the parent's copy confirmation, then open the next differing pair. The parent folder view updates the copied pair's status.
- [ ] When there are no more differing pairs in that direction, the command says so and the current view stays open (*inferred*).
- [ ] Views not opened from a folder view don't show these commands.

#### Tasks
- **E06-S11-T1** — Next and Previous Difference Files in Table, Hex and Picture child views · size: M · depends: —
  - AC: from a Hex child view, Next Difference Files opens the parent's next pair with differences. At the last pair, it reports that there are no more and stays.
- **E06-S11-T2** — Copy File to Right/Left and Open Next Difference · size: M · depends: E06-S11-T1
  - AC: in a Picture child view, the command copies the left image over the right one, the parent row turns black, and the next differing pair opens.
- **E06-S11-T3** — Hide the child-only commands outside child sessions · size: S · depends: —
  - AC: a Table Compare opened from Home shows neither command.
