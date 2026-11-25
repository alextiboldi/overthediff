# E23 — Settings portability

**Group**: feature
**Label**: `area:portability`
**Size**: L
**Depends on**: E01 (foundation); E06, E10, E21, E22
**Source**: catalogue §27 (F-27.2–F-27.5), F-25.2, F-25.3, F-26.11, F-5.2. Spec-derived from the feature catalogue and not yet validated. Items marked *inferred* are analyst or PO inferences.

This epic covers moving, sharing and resetting settings. Users can export chosen settings to a package and import it on another computer, restore factory defaults, and share a read-only set of sessions with a team. On Windows, a portable installation keeps its settings next to the executable.

## Behavioral contract
- **Data**:
  - *Settings package*: one file holding the chosen categories and items. It has a format version so that newer releases can import older packages (Q11).
  - *Categories*: Program Options; Theme, Colors, Fonts; Toolbars, Shortcuts, Menus; Program State (window positions, MRU lists); Sessions; File Formats; Profiles; System Info.
  - *Shared Sessions File*: the path of a package whose sessions appear read-only.
  - *Portable marker*: an empty file with a documented name, placed in the program folder.
- **Routes/endpoints**:
  - Tools > Export Settings, Tools > Import Settings and Tools > Restore Factory Defaults.
  - Tweaks > Shared Sessions File.
  - Command line: a package path as the argument, optionally with `/silent`.
- **Integrations**: every category is read from and written to the E01-S1 settings store. The categories come from sessions (E06), file formats (E10), profiles (E21) and command customisation and colours (E22).
- **Rules/states**:
  - On import, items whose names already exist are shown in red, and they are replaced.
  - The delete-all checkboxes (sessions, file formats, profiles) clear those categories before importing.
  - Shared sessions are read-only. To change them, re-export the package.
  - A portable installation keeps one set of settings next to the executable, shared by every user of that copy.

---

## Stories

### E23-S1 — Export and import settings packages
**Priority**: Should · **Layer**: 3 · **Size**: L · **Platforms**: All
**Needs**: E06-S1 (saved sessions), E10-S1 (File Formats manager), E21-S1 (profiles), E22-S1 (command customisation), E22-S2 (folder view colours), E04-S2 (zip archives, for the package mask), E05-S1 (themes)
**Traces**: F-27.2, F-27.3, F-25.2, F-25.3 · **Open questions**: Q11

As a developer moving to a new computer, I can export my settings to one package and import it on the other machine so that I don't have to set everything up again.

**Acceptance criteria**
- [ ] *Tools > Export Settings* opens a wizard with these categories: Program Options; Theme, Colors, Fonts; Toolbars, Shortcuts, Menus; Program State; Sessions; File Formats; Profiles; System Info.
- [ ] *Next* lets the user pick individual named sessions, file formats and profiles ("some or all"). The result is one settings package file.
- [ ] *Tools > Import Settings* accepts a settings package, plus any other inputs decided in T1. The user then picks categories and items.
- [ ] Items whose names already exist are shown in red and are replaced on import.
- [ ] The checkboxes *Delete all existing sessions*, *Delete all existing file formats* and *Delete all existing profiles* clear those categories before importing.
- [ ] Opening a settings package from the command line forwards it to the running instance, or starts one, which shows the import wizard. With `/silent`, everything in the package is imported without showing the wizard. With no instance running, this happens in the launching process (run-in-launching-process). With one running, it happens there.
- [ ] Every package records its format version. A package from a newer format version is refused with a message. Migrating older format versions is deferred until a second format version exists (*inferred* from "older major versions").

- [ ] The settings-package extension is added to the zip-family archive masks, so a package opens as a folder in folder views.
#### Tasks
- **E23-S1-T1** — Decide the package format, the accepted import inputs, and whether third-party settings formats import (Q11) · size: S · depends: — · flags: `decision`
  - AC: decision recorded in this story. Proposed default:
    - The package is a zip with a manifest (format version, categories, items) and one file per item.
    - Every package records its format version. Importing older format versions is added once a second format version exists.
    - Third-party settings formats are not imported.
    - Saved passwords are left out unless the user ticks an explicit option.
- **E23-S1-T2** — Export wizard: categories first, then named items · size: M · depends: E23-S1-T1 · flags: `licence-check`, `naming-check`, `craft-libs`
  - AC: exporting only two named sessions and one profile produces a package whose manifest lists exactly those items. The exported package opens as a folder in Folder Compare.
- **E23-S1-T3** — Import wizard: categories, items, red replacements and delete-all checkboxes · size: M · depends: E23-S1-T1
  - AC: in a package containing a session named like an existing one, that session shows in red and replaces the existing one. With *Delete all existing profiles* ticked, profiles not in the package are gone after the import.
- **E23-S1-T4** — Command-line import, with and without `/silent` · size: S · depends: E23-S1-T3
  - AC: opening a package from the command line shows the import wizard. With `/silent` and no instance running, every item imports and no window appears. With an instance running, `/silent` imports there without the wizard.
- **E23-S1-T5** — Record the format version and refuse newer packages · size: S · depends: E23-S1-T3
  - AC: a package marked with a newer format version is refused with a message naming both versions.
- **E23-S1-T6** — System Info category for support · size: S · depends: E23-S1-T2
  - AC: an export with System Info includes the OS version, product version and display settings (*inferred* contents). Import ignores this category.

### E23-S2 — Restore factory defaults
**Priority**: Should · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E06-S1 (saved sessions), E10-S1 (File Formats manager), E21-S1 (profiles), E22-S1 (customised toolbars, shortcuts, menus), E22-S2 (folder view colours), E05-S1 (themes)
**Traces**: F-27.3

As a QA engineer whose settings got into a bad state, I can restore factory defaults for chosen categories and items so that I can start clean without reinstalling.

**Acceptance criteria**
- [ ] *Tools > Restore Factory Defaults* offers the same category and item selection as import: Program Options; Theme, Colors, Fonts; Toolbars, Shortcuts, Menus; Program State; Sessions; File Formats; Profiles.
- [ ] Restoring a category resets its settings to factory values. Restoring individual built-in file formats resets only those formats.
- [ ] The checkboxes *Delete all existing sessions*, *Delete all existing file formats* and *Delete all existing profiles* remove every user-created item in that category.
- [ ] A confirmation lists what will be reset or deleted before anything changes (*inferred*).
- [ ] Categories that aren't selected stay as they are.

#### Tasks
- **E23-S2-T1** — Factory values for each category and each built-in item · size: M · depends: —
  - AC: after Program Options is reset, every option shows its factory value and sessions are unchanged.
- **E23-S2-T2** — Restore Factory Defaults dialog with categories, items and delete-all checkboxes · size: M · depends: E23-S2-T1 · flags: `naming-check`
  - AC: resetting one built-in file format restores its masks and description and leaves the other formats alone. Ticking *Delete all existing sessions* removes every saved session.
- **E23-S2-T3** — Confirmation before resetting · size: S · depends: E23-S2-T2
  - AC: the confirmation names the selected categories, and Cancel changes nothing.

### E23-S3 — Shared sessions
**Priority**: Could · **Layer**: 4 · **Size**: M · **Platforms**: All
**Needs**: E23-S1 (settings packages), E06-S1 (Home sessions tree)
**Traces**: F-27.4, F-26.11, F-5.2

As a team lead, I can publish a set of sessions in a package on a shared path so that everyone on the team sees the same read-only sessions.

**Acceptance criteria**
- [ ] The tweak *Shared Sessions File* takes the path of a settings package.
- [ ] The sessions in that package appear in a separate *Shared* branch of the Home sessions tree.
- [ ] Shared sessions open like saved sessions but are read-only: they can't be renamed, deleted or saved in place. Save Session As makes a personal copy (*inferred*).
- [ ] After the package is re-exported to the same path, the Shared branch shows the new contents the next time Home refreshes or the app starts (*inferred* timing).
- [ ] If the shared file is unreachable or invalid, the Shared branch shows a message and personal sessions are unaffected (*inferred*).

#### Tasks
- **E23-S3-T1** — *Shared Sessions File* tweak, and reading sessions from the package · size: S · depends: — · flags: `naming-check`
  - AC: pointing the tweak at a package that holds three sessions makes those three sessions available.
- **E23-S3-T2** — Shared branch in the sessions tree with read-only sessions · size: M · depends: E23-S3-T1
  - AC: Rename and Delete are unavailable on shared sessions, but opening one works. Save Session As saves a personal copy under Saved.
- **E23-S3-T3** — Pick up re-exported packages, and report unreachable files · size: S · depends: E23-S3-T1
  - AC: after the package is replaced with one holding a fourth session, four sessions show following a restart. An unreachable path shows a message in the Shared branch.

### E23-S4 — Portable installation on Windows
**Priority**: Could · **Layer**: 2 · **Size**: M · **Platforms**: Win
**Needs**: E01-S2 (installable builds)
**Traces**: F-27.5

As an IT administrator who carries tools on a USB drive, I can run a portable installation that keeps its settings next to the executable so that the settings travel with it and serve every user of that copy.

**Acceptance criteria**
- [ ] When an empty marker file with the documented name is in the program folder, the app reads and writes all its settings there instead of in the per-user settings folder.
- [ ] Every user who runs that copy shares the same settings.
- [ ] Moving or copying the program folder to another drive or computer keeps its settings.
- [ ] Without the marker file, settings stay in the per-user folder as usual.
- [ ] If the program folder isn't writable, the app says so at start-up rather than silently losing changes (*inferred*).

#### Tasks
- **E23-S4-T1** — Detect the portable marker, switch the settings location, and document the marker · size: S · depends: —
  - AC: with the marker present, saving an option writes it under the program folder and nothing under the per-user folder. A guide shipped with the product names the marker file and where to put it.
- **E23-S4-T2** — Settings survive moving the portable copy · size: S · depends: E23-S4-T1
  - AC: after the program folder is copied to another drive and run there, it shows the same options and program state.
- **E23-S4-T3** — Report an unwritable program folder in portable mode · size: S · depends: E23-S4-T1
  - AC: a read-only program folder that contains the marker gives a clear message at start-up.
