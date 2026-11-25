# OverTheDiff — Release 1 (MVP) hand-off: all Must stories

Generated from `backlog/` on 2026-11-25: 41 stories, 200 tasks. The epic files remain the source of truth; if this file and an epic disagree, the epic wins.

## Brief for the implementing agent

You are building the first release of OverTheDiff, a desktop tool in Rust and egui for comparing and reconciling text files and folder trees on Windows, macOS and Linux. No code exists yet.

0. **Repository**: `alextiboldi/overthediff` on GitHub (remote `origin`) is empty, with no `main` branch yet. The first commit has to go straight to `main`, so ask your operator before that first push. After it, every task follows the task workflow below.
1. **Read first**: `AGENTS.md` at the repository root (it overrides your defaults), then the *How to read this backlog* and *Definition of Done* sections below. Every task meets the Definition of Done; its own AC list only what is specific to it. For hub surfaces (which story owns a shared surface), open questions and traceability, see `backlog/00-index.md`.
2. **Scope**: only the stories in this file. Each story's *Not in this story* line points at later stories. Don't build those, and don't add hooks for them beyond the extension points E01-S1 already defines.
3. **Order**: build **E27-S1 (setup) alone first**. Then go layer by layer (0 → 3). Within a layer, stories don't depend on each other and can run in parallel. A story can start as soon as everything in its *Needs* is done. Within a story, follow each task's `depends`.
4. **One task, one worktree, one pull request.** Follow *Task workflow* in `AGENTS.md` exactly: a worktree per task; tests pass; an independent code review; the ticket (the task's entry in its epic file, not this snapshot) updated and closed; squash-merged into `main`; branch and worktree removed. Say in the PR which checks ran and which didn't.
5. **Decisions**: tasks flagged `decision` start by recording a decision, and each has a proposed default. Bring the proposal to your operator and get it confirmed before you build on it; then record the outcome in the story. *inferred* behaviour also needs the operator's confirmation before the story starts.
6. **Flags**: `licence-check` means check the dependency's licence against the allowlist before adding it, and stop if it isn't there. `naming-check` means run the naming check before the name reaches the UI. `craft-libs` means look in storytold/craft-libs first. If the task needs a craft-libs pull request, defer it: open a new ticket that explains why it's needed and links the task, and keep the code in OverTheDiff for now. `accessibility` means the keyboard and screen-reader behaviour is the point of the task.
7. **Stop and ask** before anything a new commit can't undo: publishing, releasing, signing, using secrets or force-pushing (AGENTS.md rule 4). The one standing exception is merging your own task's pull request and deleting its branch, once its tests and review have passed, as the task workflow says.
8. **Report honestly**: if something is partial or unsupported, say so in the app, on the command line and in your report. A task is done only when its AC pass.

## Build order

### Release 1 — MVP (all Must stories) — 41 stories

**Setup** (1 story, before layer 0; build it alone)

| Story | Title | Size | Platforms | Needs |
|---|---|---|---|---|
| E27-S1 | A repository several people can work on at once | M | All | — |

**Layer 0** (5 stories, buildable in parallel)

| Story | Title | Size | Platforms | Needs |
|---|---|---|---|---|
| E01-S1 | Application shell, settings store and extension points | L | All | — |
| E02-S1 | File masks, regular expressions and file-format resolution | M | All | — |
| E03-S1 | Text comparison engine | L | All | — |
| E03-S2 | Folder comparison engine | L | All | — |
| E04-S1 | File-system access layer for local and network paths | M | All | — |

**Layer 1** (5 stories, buildable in parallel)

| Story | Title | Size | Platforms | Needs |
|---|---|---|---|---|
| E01-S2 | Installable builds for Windows, macOS and Linux | M | All | E01-S1 |
| E05-S1 | Light and dark themes | S | All | E01-S1 |
| E07-S1 | Text Compare view | L | All | E01-S1, E03-S1, E04-S1, E02-S1 |
| E08-S1 | Folder Compare view | L | All | E01-S1, E03-S2, E04-S1 |
| E11-S2 | Headless quick compare by size, CRC or binary | S | All | E03-S2, E04-S1, E01-S1 |

**Layer 2** (28 stories, buildable in parallel)

| Story | Title | Size | Platforms | Needs |
|---|---|---|---|---|
| E04-S2 | Zip archives as folders | L | All | E04-S1, E08-S1, E07-S1 |
| E06-S1 | Save and reopen sessions | L | All | E01-S1, E07-S1, E08-S1 |
| E06-S2 | Session defaults per type | M | All | E01-S1, E07-S1, E08-S1 |
| E07-S2 | Navigate between differences | M | All | E07-S1 |
| E07-S3 | Display filters and context | M | All | E07-S1 |
| E07-S4 | Importance rules for whitespace, case and line endings | L | All | E07-S1 |
| E07-S5 | Find and replace | M | All | E07-S1, E02-S1 |
| E07-S6 | Go to line and bookmarks | S | All | E07-S1 |
| E07-S7 | Copy differences to the other side | M | All | E07-S1 |
| E07-S9 | Whitespace, word wrap, layout and font size | M | All | E07-S1 |
| E07-S10 | Per-side file format and encoding overrides | M | All | E07-S1, E02-S1 |
| E08-S2 | Display filters and folder display modes | L | All | E08-S1 |
| E08-S3 | Name filters | M | All | E08-S1, E02-S1 |
| E08-S4 | Quick-test comparison criteria | M | All | E08-S1 |
| E08-S5 | Content comparison by CRC and binary | M | All | E08-S1 |
| E08-S6 | Columns: choose, sort and extra metadata | M | All | E08-S1 |
| E08-S7 | Selection commands | M | All | E08-S1 |
| E08-S8 | Base-folder navigation and history | M | All | E08-S1 |
| E08-S9 | Step through differences and find filenames | M | All | E08-S1, E02-S1 |
| E08-S10 | Open file pairs in file views | L | All | E08-S1, E07-S1 |
| E08-S11 | Folder loading and scanning behaviour | M | All | E08-S1 |
| E09-S1 | Copy to the other side | M | All | E08-S1, E04-S1 |
| E09-S2 | Move to the other side | M | All | E08-S1, E04-S1 |
| E09-S3 | Delete | S | All | E08-S1, E04-S1 |
| E09-S4 | Synchronize the selection | M | All | E08-S1, E04-S1 |
| E10-S1 | File Formats manager | M | All | E02-S1, E07-S1 |
| E10-S2 | Syntax highlighting with built-in grammars | L | All | E07-S1, E02-S1, E05-S1 |
| E11-S1 | Diff tool for version control | L | All | E07-S1, E08-S1 |

**Layer 3** (2 stories, buildable in parallel)

| Story | Title | Size | Platforms | Needs |
|---|---|---|---|---|
| E08-S13 | Rules-based content comparison | M | All | E08-S5, E07-S4, E06-S2, E11-S2 |
| E10-S5 | Grammar-based importance | M | All | E10-S2, E07-S4 |

## How to read this backlog

**IDs.** Epic `E07`, story `E07-S4`, task `E07-S4-T2`. Each story carries one metadata line, then *Needs*, *Traces* (catalogue feature IDs and open questions) and *Not in this story* (scope boundaries pointing at the story that owns that behaviour).

**Priority vs layer.** These are two separate fields.
- *Priority* (MoSCoW) is value. **MVP = every Must story.** Release 2 = every Should story, and Release 3 = every Could story.
- *Layer* is dependency depth, computed rather than chosen: a story with no prerequisites is layer 0, otherwise 1 + the deepest story in its *Needs*.
- One story sits before layer 0: **E27-S1, the setup story** (Cargo workspace, CI, test harness). Every story implicitly needs it, so it isn't listed in *Needs*. Build it alone, before anything else, so later pull requests don't collide with the move.
- Priority closure holds: a Must story needs only Must stories, and a Should story needs only Must or Should stories. So each release is buildable on its own.

**The independence rule, as implemented.**
1. A story depends only on stories in **strictly lower layers**, and lists them explicitly in *Needs*.
2. **No story depends on, or relies in its acceptance criteria on, another story of its own layer.** Within a layer, every story can be picked up, built, tested and shipped individually, in any order or in parallel.
3. Shared surfaces never create hidden coupling.
   - Options pages, Session Settings tabs, menus and toolbars, progress and error UI, the path edit and the Home launcher are **extension points in E01-S1**. Any story adds to them, and none "creates" them.
   - Every other shared surface has **one owner story** in an earlier layer than all of its extenders (see *Hub surfaces*).
4. Foundations (layer 0) are independent of each other through **written contracts**: the data-source contract in E04, and the engine contracts in E03, built against fakes.
5. Tasks may depend on tasks **inside the same story** only. Stories blocked by an unspecified behaviour start with a **decision task** that has a proposed default.

Absolute independence is impossible for a product like this; you can't resolve merge conflicts before a merge view exists. These rules are the strongest form that holds: every story's prerequisites are explicit, shipped and earlier, and nothing in its own layer can block it.

**Sizes.** S has one clear path, M has several parts and some unknowns, and L is cross-cutting or has many unknowns. There are no time estimates. A task is one pull request's worth of work.

**Behaviour, not code.** Stories and tasks say what must be true when they're done. How to build it is left to the task, except where AGENTS.md or an existing craft-libs crate already decides.

**Before starting a task,** search for duplicate work and claim it (AGENTS.md rules 5 and 6).

### Labels

These map one-to-one onto GitHub issue labels when the backlog moves to issues.

- **Type**: `type:epic`, `type:story`, `type:task`, plus `bug` for fixes.
- **Area**, one per epic, shown in each epic's **Label** line: `area:platform` (E01), `area:patterns` (E02), `area:engines` (E03), `area:sources` (E04), `area:shell` (E05), `area:sessions` (E06), `area:text-compare` (E07), `area:folder-compare` (E08), `area:file-ops` (E09), `area:formats` (E10), `area:command-line` (E11), `area:folder-sync` (E12), `area:text-merge` (E13), `area:folder-merge` (E14), `area:table-compare` (E15), `area:hex-compare` (E16), `area:picture-compare` (E17), `area:reports` (E18), `area:snapshots` (E19), `area:scripting` (E20), `area:remote` (E21), `area:customisation` (E22), `area:portability` (E23), `area:text-edit` (E24), `area:specialised-compare` (E25), `area:os-integration` (E26), `area:repo` (E27), `area:automation` (E28). Stories and tasks take their epic's area.
- **Size**, on stories and tasks: `size:S`, `size:M`, `size:L`.
- **Flags**, on tasks, shown as `· flags:` at the end of the task line:
  - `decision`: the task first records a decision, with the proposed default as the starting point.
  - `licence-check`: the task is likely to add a dependency whose licence must be on the licences.md allowlist (rule 26).
  - `naming-check`: the task first puts a catalogue working name into the UI, which must pass the naming.md check (rule 22).
  - `craft-libs`: the code may belong in storytold/craft-libs (rule 9). If it does, the craft-libs pull request is deferred to a new ticket that explains why and links this task (AGENTS.md); the code stays in OverTheDiff until then.
  - `accessibility`: accessibility is central to the task.

**Definition of Done (every story)**

Every story meets these. Its acceptance criteria list only what's specific to it.

1. **Acceptance.** Every acceptance criterion passes. Engines and data sources are covered by automated suites (E03, E04 contract tests), with no regression in either. It works on every platform in the story's *Platforms* field.
2. **One command system.** Every new action is a registered command (stable id, English caption, typed arguments) reachable from a menu or toolbar, and runnable without the window (E01-S1). Default shortcuts follow the keymap conventions (E01-S1) and never use a reserved system chord. Application-level commands go in the macOS application menu.
3. **Text.** User-facing text goes through the translation system, with the English text as the key. Pull requests add English only; translations happen in storytold/craft-i18n.
4. **Look.** Views draw with registered colour roles (E01-S1) that have light and dark values, never fixed colours. Text contrast is at least 4.5:1 and controls at least 3:1, and colour is never the only signal. Icons come from Lucide.
5. **Accessibility.** Everything works from the keyboard, with no trap, and focus really moves (`request_focus`). Each control and drawn surface exposes its role, a translated name that starts with its visible label, its state, and whether it is editable. Changes away from focus are announced. A UI test in the E27-S1 harness covers each new keyboard path, focus move or announcement.
6. **Honesty.** Anything unsupported or partial says so, in the app, on the command line and in automation results (AGENTS.md rule 15).
7. **Safety.** Non-test code doesn't panic. Readers of files, settings and network replies get a hostile-input sweep. Writes stay crash-safe, and a failure never loses data.
8. **Tests.** One invariant per test; relations, not exact values; step counters, not timings. Integration tests go in `tests/it/main.rs`. A bug fix adds a test that fails without it.
9. **Checks.** The checks AGENTS.md lists pass, locally and in CI. Lints are fixed, not silenced.
10. **Names.** A new user-visible feature name passes the naming.md check before it reaches the UI.
11. **Dependencies and assets.** A new dependency's licence is on the licences.md allowlist, and NOTICE changes in the same pull request. Assets are our own or allowed by clean-room.md.
12. **Settings.** New options persist through the settings store. New session settings are contributed through the E01-S1 settings-tab model, so saved sessions persist them without extra work.
13. **Shared code.** Reuse craft-libs where it already does the job. A new reusable parser, codec, protocol or widget is a craft-libs candidate.
14. **Decisions.** Decision tasks record their outcome in the story, and *inferred* behaviour is confirmed or changed by the product owner before the story starts.
15. **Secrets.** Stories that add a profile kind keep every saved secret in E21-S1's shared Save-password control.

**Personas**: developer, code reviewer, release/build engineer, IT administrator, data analyst, designer, QA engineer, automation/power user, AI agent, contributor.

## Behavioral contracts of the epics involved

Shared data, routes, integrations and rules for each epic these stories belong to. The stories below rely on them.

### E27 — Repository, CI and test harness

- **Data**: none for users. The workspace layout, crate names and checks are recorded in AGENTS.md `## This repository`.
- **Routes/endpoints**: none for users. The checks are `cargo fmt`, `cargo clippy` and `cargo test` across the workspace.
- **Integrations**:
  - storytold/craft-common-actions, for CI.
  - egui_kittest, for UI tests.
  - craft-testkit's `hostile` module, for hostile-input sweeps.
- **Rules/states**:
  - CI changes go in pull requests of their own (AGENTS.md rule 17).
  - Engine, data-source and pattern code lives in library crates with no UI dependency, so the E03 and E04 suites run headlessly.
  - Lints are fixed, not silenced (rule 13).

### E01 — Application platform

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

### E02 — Pattern and file-type services

- **Data**:
  - *Mask list*: `;`-separated tokens. Each token may be `-`-prefixed (exclude) and/or `\`-suffixed (folder). Tokens can be simple (`f`, `p\`) or relative (`p\f`, `.\f`, `...\f`).
  - *Regular expression*: pattern plus flags (match case, whole words). A *replacement template* uses `$1`…`$9`.
  - *File format*: name, kind (Text, Table, Hex, Picture, External), masks, enabled, built-in flag, description and kind-specific properties, which the owning feature defines.
  - *Registry*: the single ordered list of formats.
- **Routes/endpoints** (desktop app, so this is a reusable control): the regex input aid, with the *Insert Regular Expression Element* picker and *RE Examples*, offered wherever a feature accepts a regex.
- **Integrations**: features consume the engines, and viewers register format kinds and built-in formats at start-up. User edits to formats and their order belong to E10-S1.
- **Rules/states**:
  - Mask semantics follow A.1. The base folder is always included, and folders containing included files are included.
  - Regex semantics follow A.2. Matching is line by line, so `\A`/`\Z` behave like `^`/`$`.
  - Format resolution: scan from the top, and the first enabled format whose mask matches wins. A format with an empty mask never matches automatically but can be picked manually.
  - Unmatched files fall back by content (T7 decision, *inferred*).

### E03 — Comparison engines

- **Data**: *(text engine)*
  - *Input*: left and right line sequences (decoded text plus each line's ending), the active rule set (ordered classifiers that mark text spans unimportant), and alignment options (Standard here; E07-S13 adds more).
  - *Aligned row*: left line or gap, right line or gap, and a kind (same, changed, left-only or right-only).
  - *Character ranges*: side, start, end and importance, per changed row.
  - *Section*: a run of contiguous non-same rows.
  - *Result*: same, unimportant differences only, or important differences.
  - *Statistics*: important and unimportant difference counts, same lines, orphan lines per side, sections.
- **Data, continued**: *(folder engine)*
  - *Input*: two data sources and base paths, criteria (compare size; compare modified time with a tolerance in seconds), content test (none, CRC or binary), skip-content-if-quick-same, override-quick-results, a per-side visibility predicate, and a flatten flag.
  - *Row*: name, left item or none, right item or none, status, content result, per-side hidden flag, scan state (unscanned, pending, done) and children.
  - *Item metadata* comes from the data-source contract (E04).
  - *Status* is one of same, different, left newer, right newer, left orphan, right orphan, unknown or error.
  - *Folder cues*: the set of child statuses present (for example "contains newer" plus "contains orphans").
  - *Selection resolution*: selected rows and sides go in, and the per-side set of items an operation acts on comes out.
- **Routes/endpoints**: none. The engines are headless and called by views, the CLI and scripts.
- **Integrations**:
  - The folder engine reads only through the data-source contract in E04. Its tests use an in-memory source with the same contract.
  - Both engines report progress to whoever calls them.
- **Rules/states**:
  - Orphan lines are always important.
  - With no rules, every difference is important.
  - Standard alignment shows results progressively.
  - A folder's own timestamp never makes it "newer".
  - "Override quick test results" lets a content match turn a row same.
  - A CRC provided by the source, for example from a zip directory, is used as-is.
  - When a selected folder is open and some of its children are selected, only those children count.

### E04 — Data sources

- **Data**:
  - *Item*: name, kind (file, folder or link), size, modified time, and creation time where the OS has one. A file can carry an **expandable-container** flag, meaning the source can list its children like a folder's, as for archives.
    - Windows attribute codes, or the Unix mode string plus owner and group.
    - Link target, a source-provided CRC (such as an archive's stored CRC), and a per-item error.
  - *Capabilities* per source: read-only, case-sensitive names, recycle-bin support, timestamp resolution (2 s on FAT, for example), attribute support and permission support.
  - *Errors*: not found, access denied, I/O error and unsupported. Each one is attached to the item it concerns.
- **Routes/endpoints** (the data-source contract; every provider implements it):
  - list children, stat, open for reading, open for writing (create or replace), create folder;
  - delete (permanently or to the recycle bin), rename, copy (within a source or across sources);
  - set modified time, set attributes or permissions.
- **Path syntax**:
  - Windows: `C:\…` and `\\server\share\…`.
  - macOS and Linux: `/…`, `~/…` and `smb://server/share/…`.
  - Inside archives: `…\name.zip\inner\file.txt`.
  - Devices: `mtp://Device/Folder` (Windows).
  - Web files: `http://` and `https://`.
- **Integrations**: OS file APIs, the OS recycle bin or trash, OS-mounted network shares, archive formats and OS device (MTP) APIs.
- **Rules/states**:
  - A listing never fails because of one bad item. The item carries the error and the rest of the listing returns.
  - A read-only source refuses every write before changing anything, with a message.
  - Copy keeps the modified time (*inferred*).
  - The recycle bin is used only where the source supports it.
  - A contract test suite defines the contract, and every provider in this backlog must pass it.

### E05 — App shell, themes and windows

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

### E07 — Text Compare

- **Data**:
  - *Text session*: left and right specs (any path the data sources accept), Description, and rules: importance, alignment, format and encoding per side, unimportance list, and replacements.
  - *View state*: display filter, layout, word wrap, visible whitespace, details panels, thumbnail.
  - *Comparison result* (from the text engine): aligned row pairs (a line or a gap on each side), difference sections, character-level ranges, importance per difference, an overall result (same, unimportant differences only, or important differences) and statistics.
  - *Pane document*: decoded text, encoding, line-ending style, file format, read-only flag with its reason, an unsaved-edits flag, undo history, edited-line marks and bookmarks 0–9.
  - *Options*: the Text Editing page (Auto indent, Backspace unindents, Initialize "Text to find" to be current word, Show filtered line counts, Number of context lines), the Next Difference page, and Tweaks (Editor Display group, thumbnail mode, incremental-search reset delay, check for files changed on disk).
- **Routes/endpoints** (desktop app, so these are commands, options and switches):
  - Session, File, Edit, Search, View and Actions commands, plus the toolbar and shortcuts of the Text Compare view.
  - Session Settings tabs: Specs, Importance, Alignment, Format, and Replacements.
  - Options pages: Text Editing, Next Difference and Tweaks.
  - CLI `<file> <file>` opens Text Compare when both files resolve to a Text format. `clipboard://` is accepted as a spec.
- **Integrations**:
  - E03-S1 text comparison engine: alignment, sections and importance classification.
  - E04 data sources: read, write and read-only capability.
  - E02-S1: the regex engine and regex input aid (find, unimportance list, replacements), and the file-format registry (format per side, file info).
  - E01-S1 shell: commands, the Session Settings dialog, options pages, path edits, unsaved-changes prompts and the status bar.
  - E10 grammars for syntax colours.
- **Rules/states**:
  - Default colours (Appendix B): important differences are red text on a light-red line background; unimportant differences are blue text on a light-blue background; red wins on a line.
  - Orphan lines are always important.
  - The comparison updates live as the user types. Recompare Files keeps edits but clears undo history. Reload Files prompts to save first.
  - Editing is disabled when the comparison errored or was cancelled, the location is read-only, the format's conversion can't save, editing is disabled for the session or its parent folder session, or a CLI read-only switch is set. Each condition arrives with the story that introduces it.
  - Navigation, display filters, statistics and the status bar all use the view's current classification of differences, so rule changes and the Ignore Unimportant Differences toggle affect them consistently.

### E08 — Folder Compare

- **Data**:
  - *Folder session*: left and right base folders, description, rules (Comparison, Handling, Name Filters, Other Filters and Misc tabs), display filter, folder display mode, and view state (expanded folders, visible columns, sort).
  - *Comparison row*: a left item and a right item. Either can be absent, which makes the row an orphan. Each item carries name, extension, size, modified time, attributes or Unix mode, owner, group, CRC (when known), link target and error.
  - *Row status*: unknown, same, different, left or right newer or older, orphan, error, or filtered-but-shown (teal).
  - *Content result*, shown in the center column: binary same, binary differences, rules-based same, unimportant differences or important differences.
  - *Folder aggregate cues* summarise a folder's contents.
  - *Filters*: name masks (include and exclude, for files and folders), other-filter items (date, size, content, Unix file type, DOS attribute) and the "Exclude protected operating system files" flag.
  - *Alignment override rules*: left pattern, right pattern, regex flag and folder limit.
  - *Base-folder history* (Back/Forward) and *marked items* (Space) are per view.
- **Routes/endpoints** (desktop app, so these are commands, tabs and switches):
  - Folder Compare's Session, Actions, Edit, Search and View menus, toolbar and context menu.
  - Session Settings tabs Specs, Comparison, Handling, Name Filters, Other Filters and Misc.
  - CLI `otd <folder> <folder>` and `/qc` (rules-based).
  - Options: Startup (Quick Compare), Tweaks (Name Filter Presets), File Operations (Include hidden items by default).
- **Integrations**:
  - E03-S2 folder engine: scan, align, quick and content tests, aggregation, flattening, the visibility hook and selection resolution.
  - E04 data sources supply the items.
  - E02-S1 masks and regular expressions.
  - E07 Text Compare: file pairs open in it, and rules-based content tests use its rules.
  - E01-S1 shell: commands, tabs, Session Settings, path edits and background tasks.
- **Rules/states**:
  - Items align by name within each folder. Case sensitivity follows the data source (*inferred*; for FTP it is a profile setting).
  - Colours follow Appendix B: gray unknown or older, black same, purple orphan, red newer or different, teal filtered-but-shown, yellow folder not yet scanned, hollow folder with its scan pending.
  - A folder's status aggregates its contents, and the folder's own timestamp is ignored.
  - File filters remove items, display filters hide rows by status, and folder display modes decide which folders show. When a filter hides only one side of a pair, both sides stay visible and the hidden side is teal.
  - Selecting a folder resolves to its visible contents for every operation.
  - Child file sessions report their result back to the parent row.

### E11 — Command line and VCS integration

- **Data**:
  - *Command line*: positional arguments and switches. Switches start with `/` on Windows and with `-` on macOS and Linux. A parameter that might contain spaces is quoted.
  - *Launch request*: view type, specs, titles, VCS paths, read-only flags, save target, filters and merge options, forwarded to the running instance.
  - *Exit codes*:

    | Code | Meaning | Owner |
    |---|---|---|
    | 0 | Success | this epic |
    | 1 | Binary same | this epic |
    | 2 | Rules-based same | E08-S13 |
    | 11 | Binary differences | this epic |
    | 12 | Similar | E08-S13 |
    | 13 | Rules-based differences | E08-S13 |
    | 14 | Conflicts detected | this epic |
    | 100 | Unknown error | this epic |
    | 101 | Conflicts detected, merge output not written | this epic |
    | 102 | Waiting helper unable to wait for the main app | this epic |
    | 103 | Waiting helper cannot find the main app | this epic |
    | 105 | Error loading script file | E20-S1 |
    | 106 | Script syntax error | E20-S1 |
    | 107 | Script failed to load folders or files | E20-S1 |
- **Routes/endpoints** (desktop app, so these are launchers, switches and menu commands):
  - *Launchers*: the main command, which hands off to the running instance and returns at once; the waiting commands; *Install Command Line Tools* in the macOS application menu.
  - *Switches*: `/title1..4`, `/lefttitle`, `/righttitle`, `/centertitle`, `/outputtitle`, `/vcs1..4`, `/vcsleft`, `/vcsright`, `/vcscenter`, `/vcsoutput`, `/ro`, `/readonly`, `/ro1`, `/lro`, `/leftreadonly`, `/ro2`, `/rro`, `/rightreadonly`, `/savetarget`, `/fv`, `/fileviewer`, `/solo`, `/?`, `/h`, `/help`, `/qc`, `/quickcompare`, `/filters`, `/expandall`, `/automerge`, `/center`, `/mergeoutput`, `/force`, `/reviewconflicts`. (`/favorleft`, `/favorright` and `/iu`/`/ignoreunimportant` come with merge importance in E13-S5.)
  - *Positional*: `-` for standard input.
- **Integrations**:
  - E01-S1 single-instance hand-off and argument dispatch. Each view type registers how it opens from positional paths.
  - E02-S1 file-format registry, so a format can be picked from a `/vcs` path.
  - E03-S2 content tests for headless quick compare; E13-S1 for merges.
  - Version-control tools that call an external diff or merge tool, such as git difftool/mergetool.
- **Rules/states**:
  - The main command hands the request to a running instance and returns at once, unless `/solo` forces a new instance.
  - A waiting command returns only after the comparison it started is closed.
  - A title replaces the path shown in a path edit. Without a title, a VCS path is shown instead, and it also picks the file format.
  - An unknown switch prints the usage text and returns exit code 100 (*inferred*).

### E06 — Sessions and workspaces

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

### E09 — Folder file operations

- **Data**:
  - *Operation request*: kind, target side or sides, selected rows, resolved item set and per-kind options (direction, recycle bin, path option, rename mask).
  - *Operation result*: outcome per item (done, skipped or failed, with a message), counts and duration.
  - *Operation settings*: one "Confirm …" option per operation kind (Q3), plus "Synchronize confirmations" (Prompt / Yes to All / No to All) on the File Operations options page.
  - *Backup settings*: enabled, naming, location and retention (Q2).
  - *Log entries*: category, time and text.
- **Routes/endpoints** (desktop app, so these are commands, options and switches): Actions menu, context-menu and toolbar commands in Folder Compare; the File Operations, Backups and Log options; the CLI switch `/nobackups`.
- **Integrations**:
  - E03-S2 selection resolution decides which items an operation touches.
  - E04 data sources perform the copy, move, delete, rename, set-time, set-attributes and recycle-bin calls, and report read-only capability.
  - E01-S1 background-task service provides progress, cancel and the end-of-task error summary.
- **Rules/states**:
  - Selecting a folder means its visible (filtered) contents. If an open folder and some of its children are both selected, only the children are processed.
  - A confirmation appears by default, and each kind has its own option to turn it off.
  - A failure on one item doesn't stop the rest (*inferred*). Cancel stops after the current item.
  - After an operation, the affected rows are re-compared and re-coloured without a full refresh.
  - A read-only target, such as a read-only archive or data source, is refused before anything changes.

### E10 — File formats and grammars

- **Data**:
  - *File format* (held in the file-format registry): name, kind (Text, Table, Hex, Picture or External), Mask (`;`-separated masks), Description, enabled flag, built-in flag, and its position in the single ordered list.
  - Text formats add:
    - a *grammar*: ordered grammar items, each with a category, an element name, match fields and flags
    - *elements*, each with "This element is case sensitive" and a line weight
    - *Misc* settings: Insert spaces instead of tabs, Tab stop, Lines are independent, Column-based data
    - *Conversion* settings: method, loading and saving command lines, filename encoding, encoding, Ctrl+Z as end of file, characters per line limit, pre-save edits
  - External formats add four command lines.
  - Folder sessions can carry per-session overrides of which formats are enabled.
- **Grammar item categories**:
  - **Basic**: a text string, with *Match character case* and *Regular expression*.
  - **Delimited**: start and end delimiters, or a start delimiter with *Stop at end of line*, plus an optional **Escape character** and the case and regex flags.
  - **List**: tokens such as keywords, with the case and regex flags.
  - **Columns**: a column range, either start to end or start to end of line.
  - **Lines**: repeating page headers, matched by a string (typically Form Feed) or by line 1, with a number of heading lines and the case and regex flags.
- **Routes/endpoints** (desktop app, so these are commands, options and switches):
  - *Tools > File Formats* opens the manager, with format editor tabs General, Grammar, Misc and Conversion.
  - *View > Syntax Highlighting* in text views.
  - The text Importance tab's element list and its Edit Grammar button.
  - The folder-session Misc tab.
- **Integrations**:
  - E02-S1 file-format registry (resolution), mask engine and regex engine.
  - E07 Text Compare: highlighting, importance, editor behaviour and the file info header.
  - E03-S1 text engine: line weights and importance classification.
  - External programs, called through command lines.
- **Rules/states**:
  - Resolution scans from the top, and the first enabled format whose mask matches the filename wins. A format with an empty mask never matches automatically but can be picked manually.
  - In the manager, red means shadowed (every name it could match is already matched higher up) and gray means disabled.
  - In a grammar, higher items take precedence. There is no nesting and no context-specific rule, so mixed languages such as HTML+PHP are not handled specially.
  - Formats define syntax, meaning which elements exist. Session rules define importance, meaning which elements matter.
  - A conversion succeeds only when the program exits with code 0 and produces a non-empty output file. Picture conversions are load-only.

## Stories, in build order

### E27-S1 — A repository several people can work on at once
**Priority**: Must · **Layer**: setup · **Size**: M · **Platforms**: All
**Needs**: —
**Traces**: AGENTS.md rules 13, 17, 21, 30, 33
**Not in this story**: packaging → E01-S2; the command-line automation tool → E28-S1.

As a contributor, I can work on one feature without colliding with every other open pull request. I can also test interface behaviour without a person at the keyboard.

**Acceptance criteria**
- [ ] The repository is a Cargo workspace. Engines, data sources and pattern services live in library crates that don't depend on the UI, separate from the app crate.
- [ ] CI runs format, lint and test checks on every pull request through storytold/craft-common-actions.
- [ ] UI tests drive the app by keyboard and read its accessibility tree.
- [ ] AGENTS.md `## This repository` describes the layout, the checks and how to run each kind of test.

#### Tasks
- **E27-S1-T1** — Cargo workspace with a UI-free library crate and the app crate · size: S · depends: —
  - AC: the workspace builds. The library crate's manifest has no egui or other UI dependency. AGENTS.md `## This repository` names the crates, the checks, and where commands, colour roles and tests live.
- **E27-S1-T2** — CI through craft-common-actions, in its own pull request · size: S · depends: E27-S1-T1
  - AC: a pull request runs workspace-wide format, clippy and test checks on Linux and macOS, and Windows runs on `main`. A new crate is covered without changing the workflow.
- **E27-S1-T3** — Test layout, a keyboard-and-accessibility UI harness, and a hostile-input sweep · size: M · depends: E27-S1-T1 · flags: `accessibility`, `licence-check`
  - AC: each crate with integration tests has one `tests/it/main.rs` binary. A UI test opens a test command by its shortcut, and after each step it checks where focus is and the focused node's role and name. A hostile-input sweep runs a sample parser over truncated and damaged input without a panic or a hang.

---

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

---

### E02-S1 — File masks, regular expressions and file-format resolution
**Priority**: Must · **Layer**: 0 · **Size**: M · **Platforms**: All
**Needs**: —
**Traces**: F-21.1, F-21.2, App-A
**Not in this story**: File Formats manager UI and shadowing cues → E10-S1; grammars → E10-S2; conversions → E10-S6.

As a developer on the product team, I can match names, text and file types through one tested service so that every filter, search and viewer choice behaves the same way across the product.

**Acceptance criteria**
- [ ] **Wildcards.** `?` matches one character and `*` matches zero or more. `[az]` is a set, `[a-z]` a range, `[!az]` a negated set and `[[]` a literal `[`. `*.*` also matches names without an extension.
- [ ] **No-extension masks.** `*.` matches only names without an extension. In a mask ending in `.`, neither `?` nor `*` matches a period.
- [ ] **Simple masks.**
  - `f` includes files named f, and `p\` includes folders named p and their subfolders.
  - `-f` excludes files and `-p\` excludes folders.
  - The base folder is always included, and folders containing included files are included.
- [ ] **Relative masks.** `p\f` requires f inside a folder named p. `.\f` requires f in the base folder. `...\f` allows zero or more parent folders.
- [ ] **Catalogue examples.** `*.pas;*.dfm;*.dpr`, `Source\`, `-*.dcu;-*.ddp`, `-Backup\`, `Windows\*.txt`, `...\Windows\*.txt` and `.\Windows\*.txt` give the specified include/exclude results on a sample tree.
- [ ] **Regex escapes and classes.**
  - Escapes: `\xnn`, `\x{nnnn}`, `\t`, `\f`, `\a` and `\e`, with `\` escaping metacharacters.
  - Classes: `.`, `\w`, `\W`, `\d`, `\D` and `\s` (space and tab).
  - Bracket classes support ranges, `[^…]` negation, a literal `-` at the start or end, and predefined classes inside.
- [ ] **Regex structure.**
  - Alternation takes the leftmost alternative first (`get|getter` matches "get" in "getter").
  - Greedy (`*` `+` `?` `{n}` `{n,}` `{n,m}`) and lazy (`*?` `+?` `??` `{n}?` `{n,}?` `{n,m}?`) quantifiers work.
  - Groups support backreferences `\1`–`\9`, and `(?i)`/`(?-i)` apply only within the enclosing group.
- [ ] **Line by line.** A pattern never matches across a line break, and `\A`/`\Z` behave like `^`/`$`.
- [ ] **Sample library.** `^abc`, `abc$`, `^abc$`, `^\s*abc`, `^\s*end;?\s*$`, `abc|def`, `a(b|c)d` and `a(b|c)d\1` match exactly the specified strings.
- [ ] **Replacement templates.** `$1`…`$9` insert captured groups. `.*(\d\.txt)` with `xyz$1` turns `abc1.txt` into `xyz1.txt`.
- [ ] **Regex input aid.** *Insert Regular Expression Element* inserts the chosen construct at the cursor, and *RE Examples* inserts a chosen sample. Both work in any field that hosts the aid.
- [ ] **Format resolution.**
  - The registry resolves a file name to the first enabled format whose mask matches, and disabled formats are skipped.
  - An empty-mask format is never chosen automatically but can be chosen by name.
  - Formats registered by a test feature appear in registration order after the built-ins.
- [ ] **Fallback.** A file that matches no format resolves by content: text-like content gets the built-in catch-all Text format, and binary content gets the Hex kind when one is registered, otherwise Text (*inferred*, confirmed in T7).

#### Tasks
- **E02-S1-T1** — Wildcard matcher for `?`, `*`, sets, ranges, negation, `[[]`, `*.*` and `*.` · size: M · depends: — · flags: `craft-libs`
  - AC: a conformance table of mask × name pairs from A.1 passes, including names with no extension and names with several periods.
- **E02-S1-T2** — Mask-list evaluation over tree paths: include/exclude, folder masks, relative masks, base folder always included · size: M · depends: E02-S1-T1 · flags: `craft-libs`
  - AC: the seven catalogue examples give the specified results on a sample tree that has nested `Windows` and `Backup` folders.
- **E02-S1-T3** — Regex engine meeting A.2: escapes, classes, alternation, anchors, quantifiers, groups, backreferences, `(?i)` scoping, line-by-line matching · size: M · depends: — · flags: `licence-check`, `craft-libs`
  - AC: the A.2 conformance cases and the sample library pass. A pattern containing `\n` can't match across lines. How conformance is reached is open.
- **E02-S1-T4** — Replacement templates with `$1`…`$9` · size: S · depends: E02-S1-T3 · flags: `craft-libs`
  - AC: the catalogue rename example passes, and a reference to a missing group produces an empty string (*inferred*).
- **E02-S1-T5** — Regex input aid control: element picker and RE Examples · size: S · depends: E02-S1-T3 · flags: `naming-check`, `craft-libs`
  - AC: a host field gains both menus. Inserting an element at a mid-text cursor keeps the surrounding text.
- **E02-S1-T6** — File-format registry: ordered list, kinds, enable flag, mask resolution, manual pick of empty-mask formats, registration by features · size: M · depends: E02-S1-T2
  - AC: with formats A (`*.txt`) and B (`*.*`) in that order, `x.txt` resolves to A. Disabling A makes it resolve to B. An empty-mask format C resolves only when picked by name.
- **E02-S1-T7** — Decide and implement the fallback for unmatched files (text vs binary detection) · size: S · depends: E02-S1-T6 · flags: `decision`, `craft-libs`
  - AC: decision recorded in this story. Proposed default: a file with NUL bytes or invalid text encoding in its first 8 KB is binary. A UTF-8 sample resolves to Text, and a PNG with an unknown extension resolves to the Hex kind when Hex is registered.

---

### E03-S1 — Text comparison engine
**Priority**: Must · **Layer**: 0 · **Size**: L · **Platforms**: All
**Needs**: —
**Traces**: F-12.7, F-12.6, F-6.3, X2
**Not in this story**: Unaligned, Myers, Patience, skew tolerance, closeness and never-align options → E07-S13; whitespace, case and line-ending rules → E07-S4; grammar elements → E10-S5; replacements → E07-S16; line weights → E10-S3.

As a developer on the product team, I can hand the engine two texts and a rule set and get back aligned rows, character differences, sections and an importance-aware result, so that every text-based view and tool shows the same, trustworthy differences.

**Acceptance criteria**
- [ ] Matching lines pair up. Inserted or deleted lines pair with gaps on the other side. Rows keep the order of both inputs.
- [ ] Paired lines that differ carry character-level ranges on both sides that cover exactly the changed characters.
- [ ] Contiguous differing rows form sections, and the same input always gives the same sections.
- [ ] Standard alignment delivers partial results while it works, so the top of a large comparison can be shown before it finishes. It can be cancelled.
- [ ] Every difference is classified important or unimportant by the active rules.
  - With no rules, all differences are important.
  - An orphan line counts as important even if its text would be unimportant (an inserted blank line, for example).
- [ ] The overall result is "same", "unimportant differences only" or "important differences", and the statistics match the rows.
- [ ] After an edit to either side, recomparing gives the same rows as a full comparison of the edited text. Interactive edits stay within the budget set in T6.
- [ ] Very large inputs stay within the memory and time budgets set in T6.

#### Tasks
- **E03-S1-T1** — Standard line alignment with gap padding and sections · size: L · depends: — · flags: `craft-libs`
  - AC: a corpus of before/after pairs (insertions, deletions, moved blocks, repeated lines) gives the expected row pairings and sections.
- **E03-S1-T2** — Character-level differences inside changed rows · size: M · depends: E03-S1-T1 · flags: `craft-libs`
  - AC: changing one word in a long line marks only that word's characters, and changing whitespace marks only the whitespace.
- **E03-S1-T3** — Importance classification framework: pluggable rules, orphan-always-important, overall result, statistics · size: M · depends: E03-S1-T1
  - AC: a test rule marking digits unimportant turns a digits-only change into "unimportant differences only", while an inserted line of digits stays important.
- **E03-S1-T4** — Progressive results and cancellation · size: M · depends: E03-S1-T1
  - AC: on a 200,000-line pair, the first rows are available before completion, and cancelling stops work within half a second.
- **E03-S1-T5** — Recompare after edits, matching a full comparison · size: M · depends: E03-S1-T3
  - AC: for random edit sequences on the corpus, recompare after each edit equals a full comparison of the edited text.
- **E03-S1-T6** — Set and meet performance budgets for large inputs · size: S · depends: E03-S1-T4, E03-S1-T5 · flags: `decision`
  - AC: budgets recorded in this story. Proposed: a 100,000-line pair with scattered changes shows first rows within 1 s and completes within 10 s on reference hardware; a single-line edit in a 10,000-line pair recompares within 100 ms. The benchmark suite passes them.

---

### E03-S2 — Folder comparison engine
**Priority**: Must · **Layer**: 0 · **Size**: L · **Platforms**: All
**Needs**: — (works against the data-source contract in E04's behavioral contract, tested with an in-memory source)
**Traces**: F-8.1, F-8.3, F-8.4, F-8.7, §9
**Not in this story**: daylight-saving, timezone, filename-case, Unix-metadata, extension and Unicode criteria → E08-S4; rules-based content comparison → E08-S13; Windows attributes → E08-S17; version comparison → E25-S3; 3-way status → E14-S1; sync planning → E12-S1.

As a developer on the product team, I can hand the engine two folder sources and criteria and get back an aligned, status-coloured tree with content results and selection resolution, so that folder views, sync, merge, snapshots and scripts all agree on what differs.

**Acceptance criteria**
- [ ] The engine scans both trees through the data-source contract. A folder is scanned when it is opened and also in the background.
  - Progress events and a per-folder scan state (unscanned, pending, done) are reported.
- [ ] Items are aligned by name within each folder. Matching is case-insensitive on case-insensitive sources and case-sensitive otherwise (*inferred*, from "case sensitivity follows the file system").
- [ ] **Quick tests.**
  - Different sizes mean different.
  - Modified times further apart than the tolerance make the later one newer and the other older.
  - Times within the tolerance count as same.
  - An item present on one side only is an orphan of that side.
- [ ] **Content tests.**
  - CRC reuses a CRC the source provides and otherwise computes CRC32. Binary compares byte by byte.
  - Results are "binary same" or "binary differences".
  - With skip-if-quick-same, content tests run only for pairs the quick tests call different.
  - With override-quick-results, a content match makes the row same even when the timestamps differ. Override-quick-results is on by default (F-8.1).
- [ ] Statuses cover same, different, left newer, right newer, left orphan, right orphan, unknown (not compared yet) and error (unreadable item, with the message kept).
- [ ] A folder carries every cue its visible contents produce, several at once if needed. Its own timestamp never makes it newer.
- [ ] The visibility predicate is evaluated per side.
  - Hidden items are left out of aggregation and selection.
  - An item hidden on one side only stays visible and is flagged as hidden on that side.
- [ ] In flattened mode, files are aligned by name regardless of subfolder.
- [ ] An item flagged as an expandable container compares as a file until it is expanded. Expanding it scans its children like a folder's.
- [ ] **Selection resolution.** A selected folder resolves to its visible contents. An open folder plus some of its children resolves to those children only. Results are per side.

#### Tasks
- **E03-S2-T1** — In-memory data source implementing the E04 contract, used as the engine's test fixture · size: S · depends: —
  - AC: fixtures can declare trees with sizes, times, links and unreadable items. The in-memory source satisfies the written E04 contract for the operations the engine uses (list, stat, read, source-provided CRC), checked by this story's own cases.
- **E03-S2-T2** — Tree scanning on demand and in the background, with progress events and scan states · size: M · depends: E03-S2-T1
  - AC: opening a folder scans it first. Background scanning completes the rest of the tree. Scan states move from unscanned through pending to done.
- **E03-S2-T3** — Name alignment per folder using the source's case sensitivity, plus orphans · size: S · depends: E03-S2-T1
  - AC: `Readme.txt` aligns with `README.txt` on a case-insensitive fixture but not on a case-sensitive one.
- **E03-S2-T4** — Quick tests (size; modified time with tolerance) and the status model · size: M · depends: E03-S2-T3
  - AC: a table of size and time deltas against tolerances gives the expected status in every row, including exactly-at-tolerance cases.
- **E03-S2-T5** — Content tests: CRC with source-provided CRC, binary, skip-if-quick-same, override-quick-results · size: M · depends: E03-S2-T4 · flags: `licence-check`
  - AC: a pair with equal content but different times is same with override on and newer with override off. A source-provided CRC is used without reading content.
- **E03-S2-T6** — Folder aggregation and the per-side visibility predicate · size: M · depends: E03-S2-T4
  - AC: a folder containing one newer file and one orphan reports both cues. Hiding the newer file through the predicate removes that cue. A file hidden on the left only stays visible and is flagged.
- **E03-S2-T7** — Flattened alignment that ignores folder structure · size: S · depends: E03-S2-T3
  - AC: `a/x.txt` on the left aligns with `b/x.txt` on the right in flattened mode and not otherwise.
- **E03-S2-T8** — Selection resolution for operations · size: S · depends: E03-S2-T6
  - AC: unit cases for a closed folder, an open folder with two selected children, and a folder containing hidden items return exactly the specified item sets.
- **E03-S2-T9** — Expandable containers · size: S · depends: E03-S2-T2
  - AC: an in-memory container compares as a file (size, time, content) until expanded. Once expanded, its children align and aggregate like a folder's.

---

### E04-S1 — File-system access layer for local and network paths
**Priority**: Must · **Layer**: 0 · **Size**: M · **Platforms**: All
**Needs**: —
**Traces**: F-7.1, F-9.5
**Not in this story**: archives → E04-S2, E04-S3; web pages → E04-S4; MTP devices → E04-S5; remote services → E21; snapshots → E19-S1; clipboard → E07-S18; stdin → E11-S4.

As a developer on the product team, I can read and change local and network files through one contract with a shared test suite so that every view, operation and future provider behaves the same on every source.

**Acceptance criteria**
- [ ] A contract test suite covers every contract operation and capability flag. It passes against an in-memory reference source and the local provider, and can be run against any later provider.
- [ ] **Listing.** Local listing works on Windows drive paths, Windows UNC paths, macOS and Linux absolute paths, `~` paths, and mounted `smb://` shares.
  - Each item reports name, kind, size and modified time.
  - It also reports Windows attributes, or the Unix mode string, owner and group.
- [ ] Symbolic links and junctions are reported as links with their target. When following links is requested, the target's type, size, time and attributes are reported instead.
- [ ] File content can be read and written. Folders can be created, and items renamed.
- [ ] Items can be deleted permanently, or to the OS recycle bin or trash where the drive supports it. Elsewhere the provider reports that the recycle bin is unavailable.
- [ ] The modified time, Windows attributes and Unix permissions can be set. Copying keeps the modified time (*inferred*).
- [ ] Each source reports its capabilities: read-only (for example a CD or DVD), case-sensitive names, recycle-bin support and timestamp resolution.
- [ ] An unreadable item in a folder produces an error on that item, and the rest of the listing still returns.

- [ ] Items can carry the expandable-container flag, and listing a container's children works through the same contract. The in-memory reference source supports containers, and a contract case covers them.
#### Tasks
- **E04-S1-T1** — Contract definition, in-memory reference source and contract test suite · size: M · depends: —
  - AC: the suite runs against the in-memory source with every case passing, including a container whose children list like a folder's. A deliberately broken fake fails the right cases.
- **E04-S1-T2** — Local provider listing and metadata: Windows attributes, Unix mode, owner and group, links, follow-links · size: M · depends: E04-S1-T1
  - AC: the listing contract cases pass on Windows, macOS and Linux, and a link to a folder is reported as a link unless follow is requested.
- **E04-S1-T3** — Local provider read, write, create folder, rename, and delete with recycle-bin support · size: M · depends: E04-S1-T1 · flags: `licence-check`
  - AC: a file deleted with the recycle bin can be restored from the OS trash. On a network path, the provider reports that the recycle bin is unavailable.
- **E04-S1-T4** — Set modified time, attributes and permissions; copy that keeps the modified time · size: S · depends: E04-S1-T3
  - AC: after a copy, source and target report the same modified time. Setting read-only on Windows and mode 0644 on Unix is visible through the OS.
- **E04-S1-T5** — Network paths (UNC, `smb://`), `~` expansion and capability detection (case sensitivity, timestamp resolution, read-only media) · size: S · depends: E04-S1-T2
  - AC: a FAT-formatted volume reports 2-second timestamp resolution, and a case-sensitive volume reports case-sensitive names.
- **E04-S1-T6** — Per-item errors that never abort a listing · size: S · depends: E04-S1-T2
  - AC: a folder with one access-denied subfolder lists every other item, and the denied one carries "access denied".

---

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

---

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
- **E05-S1-T1** — Light and dark palettes for the chrome and selection, and theme switching for every registered colour role · size: M · depends: — · flags: `accessibility`
  - AC: a harness role with light and dark values switches with the theme. The contrast check passes for chrome and selection in both palettes and flags a deliberately low-contrast harness role.
- **E05-S1-T2** — Appearance page with the three theme choices and the OS setting in parentheses · size: S · depends: E05-S1-T1
  - AC: the label shows the OS setting when the page opens, and the choice persists across restarts.
- **E05-S1-T3** — Follow the operating system setting, including changes while running · size: S · depends: E05-S1-T1
  - AC: while Follow operating system is selected, switching the OS between light and dark recolours open views without a restart.
- **E05-S1-T4** — Preview checkbox applies changes live, and Cancel reverts them · size: S · depends: E05-S1-T2
  - AC: with Preview on, picking Dark recolours open views at once. Cancel returns them to the previous theme.

---

### E07-S1 — Text Compare view
**Priority**: Must · **Layer**: 1 · **Size**: L · **Platforms**: All
**Needs**: E01-S1 (application shell), E03-S1 (text comparison engine), E04-S1 (file-system access layer), E02-S1 (file-format registry)
**Traces**: F-12.1, F-12.3, F-12.10, F-6.1, F-25.2, F-7.1, App-B · **Open questions**: Q25
**Not in this story**: navigation → E07-S2; display filters → E07-S3; importance rules and blue/unimportant → E07-S4; find/replace → E07-S5; go to/bookmarks → E07-S6; copy to other side and Select Section → E07-S7; indent/line/word commands/convert → E07-S8; whitespace/wrap/layout/font size → E07-S9; Format tab → E07-S10; thumbnail → E07-S11; details panels/ruler → E07-S12; alignment options → E07-S13; manual alignment → E07-S14; syntax highlighting → E10-S2; tab stops and Tab key → E10-S4; saving conversions → E10-S6; backups → E09-S10; changed-on-disk → E07-S22; /savetarget and /ro → E11-S1; Disable editing → E06-S8; report → E18-S1; clipboard → E07-S18; colour/font customisation → E22-S3; Merge Files → E13-S1; Compare Parent Folders → E06-S10; Open With → E22-S4.

As a developer, I can open two text files side by side, see every difference highlighted and fix them in place so that I can review and reconcile changes without switching to another editor.

**Acceptance criteria**
- [ ] A Text Compare session opens from:
  - the Home launcher
  - *Session > New Session ▸ Text Compare*
  - *File > Open File* (into the chosen pane)
  - the hidden-by-default *Open File on File System*
  - the command line, with two files that resolve to a Text format
- [ ] Each pane has a path edit with typed paths, Browse for File and history.
- [ ] The two editor panes scroll together. Lines are aligned and gaps are padded, so matching lines sit on the same row.
- [ ] Important differences show in red text, their lines get a light-red background, and the changed characters inside each line are highlighted.
- [ ] Each pane shows line numbers, and gap rows have none.
- [ ] Each pane has a file info header with path, size, modified time, detected file format, encoding and line-ending style.
- [ ] Encoding is detected from a byte-order mark, valid UTF-8 or UTF-16, falling back to the system code page. Line endings (Windows, Mac, Unix) are detected per file, and mixed endings show as mixed (*inferred*).
- [ ] Each pane is a full text editor. Typing, *Undo*/*Redo*, *Cut*/*Copy*/*Paste*/*Delete* and *Select All* (the visible lines in the pane) all work, and the comparison updates live as the user types.
- [ ] *Save File* writes the pane back in its original encoding and line-ending style. *Save File As* writes to a new path, and the hidden-by-default *Save File on File System As* saves to a local path.
- [ ] A pane with unsaved edits is marked, and Close Tab and Exit prompt to save.
- [ ] A pane is read-only, with the reason shown, when its location is read-only or the comparison errored or was cancelled.
- [ ] *Reload Files* prompts to save unsaved edits and then reloads both files. *Recompare Files* reruns the comparison, keeps edits and clears undo history.
- [ ] *Swap Sides* exchanges the left and right files and recompares.
- [ ] *Text Compare Info* shows comparison statistics. The status bar shows the overall result and the load time, as decided in T1.

#### Tasks
- **E07-S1-T1** — Decide the status-bar and Info contents (Q25) · size: S · depends: — · flags: `decision`
  - AC: decision recorded in this story. Proposed default: the status bar shows the overall result ("Same", "Unimportant differences only" or "Important differences"), cursor line and column, insert/overwrite state and load time. Info lists difference sections, changed, inserted and deleted lines, and total lines per side.
- **E07-S1-T2** — Open a Text Compare session from Home, New Session, File > Open File, Open File on File System, the path edits and the command line · size: M · depends: — · flags: `naming-check`
  - AC: launching the product with two text files opens one Text Compare tab. A typed or browsed path loads into its pane. A missing file shows an error in that pane and leaves it read-only.
- **E07-S1-T3** — Load files through the data-source layer with encoding and line-ending detection, and show the file info header · size: M · depends: — · flags: `licence-check`, `craft-libs`
  - AC: UTF-8 with and without a BOM, UTF-16 LE and BE, and a legacy code-page file each display correctly with the detected encoding shown. CRLF, CR and LF files show their line-ending style.
- **E07-S1-T4** — Render the aligned comparison: synchronised scrolling, gap padding, red text, light-red line background, character highlights and line numbers · size: L · depends: E07-S1-T3 · flags: `craft-libs`, `accessibility`
  - AC: for a pair with one inserted, one deleted and one changed line, matching lines share rows and gaps appear opposite the inserted and deleted lines. Only the changed characters are highlighted, and scrolling either pane scrolls both.
- **E07-S1-T5** — Edit with live recompare: typing, Undo/Redo, Cut/Copy/Paste/Delete and Select All · size: M · depends: E07-S1-T4 · flags: `accessibility`
  - AC: typing the other side's text into a changed line removes its highlight without a manual recompare. Undo brings the difference back.
- **E07-S1-T6** — Save File, Save File As, Save File on File System As, and unsaved-changes prompts · size: M · depends: E07-S1-T5
  - AC: saving keeps the file's encoding and line endings, so only the edited bytes change. Close Tab on an edited pane offers save, discard or cancel.
- **E07-S1-T7** — Read-only panes with a visible reason · size: S · depends: E07-S1-T3
  - AC: a file on a read-only data source opens read-only and says why. After a cancelled comparison both panes are read-only.
- **E07-S1-T8** — Session commands: Reload Files, Recompare Files, Swap Sides, Text Compare Info and the status bar · size: M · depends: E07-S1-T1, E07-S1-T4
  - AC: Reload Files with unsaved edits prompts first. Recompare Files keeps edits and empties undo history. Swap Sides moves each file to the other pane. Info and the status bar show what T1 decided.

---

### E08-S1 — Folder Compare view
**Priority**: Must · **Layer**: 1 · **Size**: L · **Platforms**: All
**Needs**: E01-S1 (application shell), E03-S2 (folder comparison engine), E04-S1 (file-system access layer)
**Traces**: F-8.1, F-8.2, F-8.4, F-8.14, F-6.1, F-25.2, F-7.1, F-9.10, App-B
**Not in this story**: display filters → E08-S2; name filters → E08-S3; Comparison tab → E08-S4; content compare and center column → E08-S5; more columns and sorting → E08-S6; selection commands → E08-S7; base-folder navigation → E08-S8; next difference and find → E08-S9; opening pairs → E08-S10; Handling tab and scan cues → E08-S11; file operations → E09; colours/fonts/stripes → E22-S2; Create for a missing base folder → E09-S9.

As a developer, I can open two folders side by side and see at a glance which items are the same, different, newer or missing so that I know where two trees diverge.

**Acceptance criteria**
- [ ] A Folder Compare opens from the Home button, from *Session > New Session ▸ Folder Compare*, or from the CLI with two folders (`otd <folder> <folder>`). The view has a left and a right pane, each with a path edit above it (typed path, Browse for Folder, history).
- [ ] Items align by name within each folder. Partners share a row. An item with no partner is an orphan, with a gap on the other side.
- [ ] *Open Folder* / *Close Folder*, *Open Subfolders* / *Close Subfolders* (recursive), *Expand All* and *Collapse All* work on either side.
- [ ] Name, Size and Modified columns show for both sides.
- [ ] Quick tests compare size and modified time within the default tolerance decided in T1.
- [ ] Rows use the default colours: gray = unknown or older, black = same, purple = orphan, red = newer or different.
- [ ] A folder's colour aggregates its contents. It is "newer" when it contains newer files, its own timestamp is ignored, and it can show several cues at once, such as red for a newer file inside plus purple for an orphan inside.
- [ ] A colour spot left of each name shows the status even when the selection hides the row colour.
- [ ] Symbolic links and junctions carry a small arrow glyph. A folder that couldn't be opened carries a small red "X".
- [ ] Click selects an item on either side, Ctrl+click adds to the selection, and Shift+click selects a range.
- [ ] *Refresh* re-scans open folders, *Full Refresh* redoes the whole comparison, and *Refresh Selection* re-scans the selected items.
- [ ] *Swap Sides* exchanges the left and right base folders and recompares.
- [ ] *Session > Folder Compare Info* shows comparison statistics.
- [ ] *View > Legend* explains every colour and glyph of the folder palette, including the ones that appear only once other features are in use: teal (filtered but shown), yellow folder (not scanned) and hollow folder (scan pending).
- [ ] The status bar shows scan progress and the overall result.

#### Tasks
- **E08-S1-T1** — Decide the default timestamp tolerance · size: S · depends: — · flags: `decision`
  - AC: decision recorded in this story. Proposed default: 2 seconds, which absorbs the 2-second timestamp resolution of FAT volumes and zip archives. Quick tests use it until the user changes it.
- **E08-S1-T2** — Two-pane view with aligned rows, path edits and the entry points (Home, New Session, CLI with two folders) · size: M · depends: — · flags: `naming-check`, `craft-libs`, `accessibility`
  - AC: `otd A B` with two folders opens the view with A and B in the path edits. Rows of a fixture tree align by name, and orphans show gaps.
- **E08-S1-T3** — Tree expansion commands · size: S · depends: E08-S1-T2
  - AC: Open Subfolders expands a folder recursively, Expand All opens every folder (scanning as needed), and Collapse All closes them all.
- **E08-S1-T4** — Status colours, folder aggregation, colour spot and glyphs · size: M · depends: E08-S1-T1, E08-S1-T2 · flags: `accessibility`
  - AC: a fixture with same, newer, older, size-different, orphan, linked and unreadable items shows the specified colour, spot and glyph for each. Parent folders show the aggregated cues.
- **E08-S1-T5** — Click, Ctrl+click and Shift+click selection on either side · size: S · depends: E08-S1-T2 · flags: `accessibility`
  - AC: left and right selections are independent, and Shift+click selects a contiguous range on that side.
- **E08-S1-T6** — Refresh, Full Refresh, Refresh Selection and Swap Sides · size: S · depends: E08-S1-T2
  - AC: after a file changes on disk, Refresh shows its new status. Swap Sides puts the right folder on the left and recompares.
- **E08-S1-T7** — Folder Compare Info, Legend and status bar · size: S · depends: E08-S1-T4
  - AC: the Info counts per status match the rows of a fixture. The Legend lists every colour and glyph the view can show.

---

### E11-S2 — Headless quick compare by size, CRC or binary
**Priority**: Must · **Layer**: 1 · **Size**: S · **Platforms**: All
**Needs**: E03-S2 (folder comparison engine content tests), E04-S1 (file-system access layer), E01-S1 (process entry point and argument dispatch)
**Traces**: F-25.3, F-25.4
**Not in this story**: rules-based `/qc` (no type) and exit codes 2, 12, 13 → E08-S13.

As a release engineer, I can check from a build script whether two files match, without opening any window, so that the script can branch on the exit code.

**Acceptance criteria**
- [ ] `/qc=size`, `/qc=crc` and `/qc=binary` (also spelt `/quickcompare=<type>`) compare two files and exit without showing a window or a taskbar entry.
- [ ] The exit code is 1 when the files are the same by the chosen test and 11 when they differ. These codes are specified for binary and *inferred* for size and CRC.
- [ ] A missing or unreadable file, a folder in place of a file, or a wrong number of files gives exit code 100, with a one-line error on standard error (*inferred* for the message).
- [ ] Any local or network path accepted by the file-system access layer works as an input.

- [ ] `/qc` runs in the launching process. It isn't forwarded to a running instance, and it works whether or not one is running.
- [ ] `/qc` is registered with the run-in-launching-process launch mode.
- [ ] Because `/qc` claims the whole command line, no comparison view opens for the two files.
#### Tasks
- **E11-S2-T1** — Parse `/qc` and `/quickcompare` with the size, crc and binary types · size: S · depends: —
  - AC: every spelling is accepted with `/` on Windows and with `-` on macOS and Linux. An unknown type gives 100.
- **E11-S2-T2** — Run the comparison headlessly and map its result to an exit code · size: S · depends: E11-S2-T1 · flags: `licence-check`
  - AC: identical files give 1 for all three types. Files of equal size with different bytes give 1 for size and 11 for crc and binary. No window appears in any run.

---

### E04-S2 — Zip archives as folders
**Priority**: Must · **Layer**: 2 · **Size**: L · **Platforms**: All
**Needs**: E04-S1 (file-system access layer), E08-S1 (Folder Compare view), E07-S1 (Text Compare view, for file specs inside archives)
**Traces**: F-7.3, F-8.1, F-8.6
**Not in this story**: other archive formats and the Archive Types page → E04-S3; creating archives through Copy/Move to Folder → E09-S5; New Folder with an archive extension → E09-S9; archive contents in snapshots → E19-S1; sync within archives → E12-S1.

As a release engineer, I can open zip-family archives as folders and compare their contents with a folder or another archive so that I can check a build artefact without unpacking it.

**Acceptance criteria**
- [ ] Files matching `*.zip;*.zipx;*.jar;*.ear;*.war` are treated as zip archives and show a zipper icon. Other features can add masks to the zip family.
- [ ] Archive contents are readable and writable through the data-source contract. The contract test suite passes against the zip provider, write cases included, so folder operations work inside archives unchanged.
- [ ] Archives are flagged as expandable containers, so double-clicking one expands it in the Folder Compare tree and compares its contents.
- [ ] The CRC stored in the archive is reported to the folder engine, so CRC comparison doesn't need to decompress.
- [ ] The *Archive handling* option on the folder session's Handling tab offers three modes:
  - **As files**: archives sort with files, show no folder icon and compare as files.
  - **As folders once opened**: archives sort as files with a zipper-folder icon and compare as files until opened; once opened, their status reflects their contents.
  - **As folders**: archives always behave as folders.
  
  The default is "As folders once opened" (*inferred* from "treated as files when copying or comparing; double-clicking expands them").
- [ ] A file spec can point inside an archive, for example `C:\a.zip\docs\readme.txt`. Such paths work in path edits and for file views.
- [ ] A damaged archive shows the folder-error glyph with the error message, and the rest of the comparison still loads.

#### Tasks
- **E04-S2-T1** — Zip provider for listing and reading, with stored CRCs · size: M · depends: — · flags: `licence-check`, `craft-libs`
  - AC: the contract read cases pass against the zip provider, and listing a 10,000-entry archive doesn't decompress any entry.
- **E04-S2-T2** — Zip provider writes: add, replace, delete and rename entries, create folders · size: L · depends: E04-S2-T1
  - AC: the contract write cases pass, and an archive modified through the provider opens in standard zip tools.
- **E04-S2-T3** — Archive detection by mask, zipper icon, and expand-and-compare in Folder Compare · size: M · depends: E04-S2-T1
  - AC: `x.jar` against a folder with the same contents expands and shows every row as same. A mask a test feature adds to the zip family is detected as a zip archive.
- **E04-S2-T4** — "Archive handling" option with As files, As folders once opened and As folders · size: M · depends: E04-S2-T3
  - AC: each mode gives the sorting, icon and status behaviour listed above on the same pair of trees.
- **E04-S2-T5** — Paths inside archives as file and folder specs · size: S · depends: E04-S2-T1
  - AC: typing `…\a.zip\docs` as a base folder loads that inner folder, and an inner file path opens in a file view.

---

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
- **E06-S1-T1** — Store named sessions in the settings store · size: M · depends: —
  - AC: a saved Folder Compare session and a saved Text Compare session reload after a restart with identical type, specs, description and settings. A corrupt session file is skipped with a warning, and the other sessions still load (*inferred*).
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

---

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
- **E06-S2-T1** — Store and load one settings set per session type · size: S · depends: —
  - AC: defaults written for Folder Compare survive a restart and never apply to Text Compare.
- **E06-S2-T2** — *Update session defaults* in the scope dropdown · size: S · depends: E06-S2-T1
  - AC: choosing it and pressing OK updates both the view and the stored defaults. Cancel changes neither.
- **E06-S2-T3** — New sessions start from their type's defaults · size: S · depends: E06-S2-T1
  - AC: after Folder Compare defaults are updated, a Folder Compare opened from Home and one opened from the command line both show the new settings, with specs only from their own inputs.

---

### E07-S2 — Navigate between differences
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E07-S1 (Text Compare view)
**Traces**: F-12.4
**Not in this story**: per-line navigation for independent-line formats → E10-S4; Next/Previous Replacement → E07-S16; Next/Previous Difference Files → E08-S10; Next Difference options → E07-S19.

As a code reviewer, I can jump from one difference to the next so that I can review a change set without scrolling through unchanged text.

**Acceptance criteria**
- [ ] *Next Difference Section* and *Previous Difference Section* jump to the next or previous run of differing lines and scroll both panes to it.
- [ ] *Next Difference* and *Previous Difference* step through individual differences inside lines. When a difference lies beyond the visible width of a long line, the panes scroll horizontally to it.
- [ ] *Next Edit* and *Previous Edit* jump between lines the user has edited since the files were loaded.
- [ ] The navigation arrows are bright yellow while more differences lie in that direction. They turn dull yellow, or disabled, when there are none.
- [ ] Navigation visits differences as the view currently classifies them, so a difference the view treats as same is skipped.
- [ ] Every navigation command has a default keyboard shortcut.

#### Tasks
- **E07-S2-T1** — Find the next or previous section and the next or previous in-line difference from the cursor · size: M · depends: —
  - AC: unit cases cover a cursor before, inside and after a section. A line with three separate changes is visited change by change, in order.
- **E07-S2-T2** — Navigation commands and shortcuts that bring the target into view in both panes · size: S · depends: E07-S2-T1
  - AC: Next Difference Section from the top of the file lands on the first section, with both panes showing it.
- **E07-S2-T3** — Track edited lines, and add Next/Previous Edit · size: S · depends: —
  - AC: after edits on lines 10 and 40, Next Edit from line 1 visits line 10 and then line 40.
- **E07-S2-T4** — Arrow states that follow the cursor and the comparison · size: S · depends: E07-S2-T1
  - AC: at the last section the Next arrow turns dull or disabled. An edit that removes every difference disables both arrows.

---

### E07-S3 — Display filters and context
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E07-S1 (Text Compare view)
**Traces**: F-12.2, F-26.11
**Not in this story**: Ignore Unimportant Differences → E07-S4; Ignored → E07-S17; remembering the filter → E06-S4.

As a code reviewer, I can hide matching lines, or show differences with a few lines of context, so that large files show only what changed.

**Acceptance criteria**
- [ ] *Show All* shows every line. *Show Differences* shows only lines with differences. *Show Same* shows only matching lines. *Show None* (hidden by default) hides every line.
- [ ] *Show Context* shows the differences plus *N* surrounding lines. *N* is the "Number of context lines" option on the Text Editing page.
- [ ] Filters apply to both panes together, so rows stay aligned. The active filter shows as checked.
- [ ] With "Show filtered line counts" on, each run of hidden lines is summarised by the number of lines it hides (*inferred* presentation).
- [ ] A line counts as different or same according to the view's current classification.

#### Tasks
- **E07-S3-T1** — Work out which rows each filter shows (All, Differences, Same, None, Context with *N*) · size: M · depends: —
  - AC: a unit table for a file with two sections and *N* = 2 gives the expected visible rows under every filter, including overlapping context ranges.
- **E07-S3-T2** — Display-filter commands on the View menu and toolbar (Show None hidden by default) · size: S · depends: E07-S3-T1 · flags: `naming-check`
  - AC: switching filters keeps the cursor on the same line whenever that line stays visible.
- **E07-S3-T3** — "Number of context lines" and "Show filtered line counts" options on the Text Editing page · size: S · depends: E07-S3-T1
  - AC: changing *N* updates an open view. With counts on, a hidden run of 37 lines shows a summary saying 37 lines are hidden.

---

### E07-S4 — Importance rules for whitespace, case and line endings
**Priority**: Must · **Layer**: 2 · **Size**: L · **Platforms**: All
**Needs**: E07-S1 (Text Compare view)
**Traces**: F-12.6, F-12.2, F-6.3, App-B
**Not in this story**: grammar elements → E10-S5; Edit Grammar button → E10-S3; unimportance list → E07-S15; replacements → E07-S16.

As a developer, I can mark whitespace, letter-case and line-ending differences as unimportant so that reformatting doesn't hide the changes that matter.

**Acceptance criteria**
- [ ] The Importance tab of Session Settings lists the built-in elements *Leading whitespace*, *Embedded whitespace*, *Trailing whitespace* and *Everything else*. A ticked element is important and an unticked one is unimportant.
- [ ] *Everything else* has a **Character case** checkbox, which decides whether case differences in non-whitespace text are important.
- [ ] **Compare line endings (PC/Mac/Unix)** is off by default, so line-ending differences are ignored. When it is on, line endings are compared line by line.
- [ ] An orphan line is always important, even when it is blank or holds only unimportant text.
- [ ] Unimportant differences show in blue text with a light-blue line background. A line holding both kinds gets the light-red background, because red wins.
- [ ] *View > Ignore Unimportant Differences* treats unimportant differences as same. They are not highlighted, and they count as same wherever the view classifies lines.
- [ ] The status bar and Text Compare Info tell "unimportant differences only" apart from important differences and from same.
- [ ] Rule changes apply through the Session Settings scope dropdown and recompare the open view at once.

#### Tasks
- **E07-S4-T1** — Classify leading, embedded and trailing whitespace, and the case of "everything else" · size: M · depends: —
  - AC: unit cases. An indentation-only change is unimportant when Leading whitespace is unticked. `a  b` vs `a b` follows Embedded whitespace. A case-only change is unimportant only when Character case is unticked.
- **E07-S4-T2** — Compare line endings option · size: S · depends: —
  - AC: a CRLF file and its LF copy compare as same by default. With the option on, every line is flagged as different.
- **E07-S4-T3** — Keep orphan lines important · size: S · depends: E07-S4-T1
  - AC: an inserted blank line stays red with every whitespace element unticked.
- **E07-S4-T4** — Importance tab controls · size: S · depends: E07-S4-T1, E07-S4-T2
  - AC: ticking or unticking an element recompares the open view. The values are kept with the session settings.
- **E07-S4-T5** — Show unimportant differences in blue, with red-over-blue line backgrounds · size: M · depends: E07-S4-T1 · flags: `accessibility`
  - AC: a line with a case-only change and a word change shows blue for the case change, red for the word and a light-red background.
- **E07-S4-T6** — Ignore Unimportant Differences toggle and the overall result · size: M · depends: E07-S4-T5
  - AC: with the toggle on, a pair that differs only in indentation shows no highlights. The status bar reports "unimportant differences only" whether or not they are shown.

---

### E07-S5 — Find and replace
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E07-S1 (Text Compare view), E02-S1 (regular expressions and regex input aid)
**Traces**: F-12.5, F-26.11

As a developer, I can search both files for text or a regular expression and replace the matches so that I can find and fix every occurrence while comparing.

**Acceptance criteria**
- [ ] *Find* opens a find panel whose text box keeps a history of searches.
- [ ] Panel options:
  - **Regular expression**: PCRE subset per Appendix A.2.
  - **Match character case**.
  - **Whole words only**: `apple` doesn't match `apples`, `applesauce` or `snapple`.
  - **Wrap search**.
  - **Sides**: left, right or both.
  - **Close automatically**: hides the panel once the search starts.
- [ ] With Regular expression on, *Insert Regular Expression Element* inserts syntax at the cursor. An invalid pattern shows an error and searches nothing.
- [ ] **Next** finds the next match, and **Select All** selects every line that contains a match.
- [ ] *Find Next* and *Find Previous* repeat the last search forwards or backwards without the panel.
- [ ] *Replace* replaces the current match or all matches on the searched sides. Replacements can be undone and update the comparison live. Read-only panes are skipped with a message (*inferred*).
- [ ] With the option *Initialize "Text to find" to be current word* on, opening Find with no selection fills in the word at the cursor.
- [ ] Typing in the find box searches incrementally. After a pause longer than the tweak "incremental-search reset delay" (ms), the next keystroke starts a new search (*inferred* meaning).

#### Tasks
- **E07-S5-T1** — Match over both panes: plain text, case, whole words, regex, sides, wrap and direction · size: M · depends: — · flags: `licence-check`, `craft-libs`
  - AC: unit cases include the catalogue's whole-word example and a regex with a backreference.
- **E07-S5-T2** — Find panel with history, element picker, options, Next, Select All and Close automatically · size: M · depends: E07-S5-T1
  - AC: Select All for `TODO` selects exactly the lines that contain it on the chosen sides.
- **E07-S5-T3** — Find Next/Previous and the "Initialize Text to find to be current word" option · size: S · depends: E07-S5-T1
  - AC: with the cursor inside `foo` and no selection, Find opens filled with `foo`.
- **E07-S5-T4** — Replace and Replace All, with undo and live recompare · size: M · depends: E07-S5-T1
  - AC: replacing every `a` with `b` on the left changes only the left pane and marks it unsaved. One Undo reverts all the replacements (*inferred*).
- **E07-S5-T5** — Incremental search and its reset-delay tweak · size: S · depends: E07-S5-T2
  - AC: with the delay at 500 ms, typing `ab`, pausing for 1 s, then typing `c` searches for `c`.

---

### E07-S6 — Go to line and bookmarks
**Priority**: Must · **Layer**: 2 · **Size**: S · **Platforms**: All
**Needs**: E07-S1 (Text Compare view)
**Traces**: F-12.5

As a developer, I can jump to a line and column and set numbered bookmarks so that I can move around long files quickly.

**Acceptance criteria**
- [ ] *Go To* moves the cursor in the active pane to a given line and column and scrolls both panes there. Values out of range go to the nearest valid position (*inferred*).
- [ ] *Toggle Bookmark* sets or clears a bookmark numbered **0–9** on the current line. *Go To Bookmark* jumps to it, and *Clear Bookmarks* removes them all.
- [ ] Bookmarked lines are marked in the gutter, and a bookmark moves with its line when lines are inserted or deleted above it (*inferred*).

#### Tasks
- **E07-S6-T1** — Go To line and column · size: S · depends: —
  - AC: Go To 120:5 puts the cursor on line 120, column 5, of the active pane and both panes show it.
- **E07-S6-T2** — Bookmarks 0–9: toggle, go to, clear, gutter marks, and keeping their position through edits · size: S · depends: —
  - AC: after inserting 3 lines above bookmark 2 on line 50, Go To Bookmark 2 lands on line 53.

---

### E07-S7 — Copy differences to the other side
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E07-S1 (Text Compare view)
**Traces**: F-12.3
**Not in this story**: go to next difference after copying → E07-S19; gutter copy buttons → E07-S20.

As a developer merging by hand, I can copy a difference section or selected lines to the other side so that I can take changes across without retyping them.

**Acceptance criteria**
- [ ] *Copy to Other Side* copies the selected lines, or the current difference section when nothing is selected, to the other pane. Its caption shows the direction (Copy to Right or Copy to Left) based on the active pane.
- [ ] *Copy to Right* and *Copy to Left* are hidden by default and run from their default shortcuts.
- [ ] *Copy Line to Right*, *Copy Line to Left* and *Copy Line to Other Side* copy only the current line.
- [ ] After a section is copied, the target section is identical to the source. Lines that exist only on the target side are removed, and lines missing there are inserted.
- [ ] A copy is one undoable edit. It marks the target pane unsaved and updates the comparison live.
- [ ] A copy into a read-only pane is refused with the reason, and nothing changes.
- [ ] *Select Section* selects every line of the current difference section.

#### Tasks
- **E07-S7-T1** — Work out which source and target lines a copy covers · size: M · depends: —
  - AC: unit cases cover a changed section, left-only lines (a gap on the target), right-only lines (removed from the target) and a partial selection inside a section.
- **E07-S7-T2** — Copy to Other Side with a dynamic caption, plus the hidden Copy to Right/Left · size: S · depends: E07-S7-T1
  - AC: the caption reads "Copy to Right" while focus is in the left pane, and "Copy to Left" from the right pane.
- **E07-S7-T3** — Copy Line to Right/Left/Other Side · size: S · depends: E07-S7-T1
  - AC: only the current line is copied, even inside a multi-line section.
- **E07-S7-T4** — Apply a copy as one undoable edit with live recompare, and refuse read-only targets · size: S · depends: E07-S7-T1
  - AC: after a section is copied its rows show as same, and Undo restores the difference. A read-only target shows its reason and nothing changes.
- **E07-S7-T5** — Select Section · size: S · depends: E07-S7-T1
  - AC: with the cursor in a 5-line section, Select Section selects those 5 lines in the active pane.

---

### E07-S9 — Whitespace, word wrap, layout and font size
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E07-S1 (Text Compare view)
**Traces**: F-12.1
**Not in this story**: fonts and colours → E22-S3; editor tweaks → E07-S20; syntax highlighting → E10-S2.

As a developer, I can show whitespace, wrap long lines, stack the panes and resize the text so that I can read any file comfortably on any screen.

**Acceptance criteria**
- [ ] *View > Visible Whitespace* shows spaces, tabs and Windows, Mac and Unix line endings, each with its own glyph.
- [ ] The toolbar *Wrap* button (Word Wrap) wraps lines to the pane width. Continuation rows stay aligned across the panes, and the line number appears only on a line's first row (*inferred*).
- [ ] The *Side-by-Side* and *Over-Under* layouts switch between left/right and top/bottom panes, keeping the scroll position and the cursor.
- [ ] Display font size increase, decrease and reset (hidden by default) change the editor text size in both panes.

#### Tasks
- **E07-S9-T1** — Visible Whitespace glyphs for space, tab, CRLF, CR and LF · size: S · depends: —
  - AC: a file containing all three line-ending styles shows three different end-of-line glyphs.
- **E07-S9-T2** — Word Wrap with aligned continuation rows · size: M · depends: —
  - AC: a 300-character changed line wraps in both panes, and the line after it starts on the same row in both panes.
- **E07-S9-T3** — Side-by-Side and Over-Under layouts · size: M · depends: — · flags: `naming-check`
  - AC: switching layout keeps the current line in view, and both panes still scroll together.
- **E07-S9-T4** — Font size increase, decrease and reset · size: S · depends: —
  - AC: two increases followed by a reset return the text to the configured size.

---

### E07-S10 — Per-side file format and encoding overrides
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E07-S1 (Text Compare view), E02-S1 (file-format registry)
**Traces**: F-12.8
**Not in this story**: Compare Files Using → E06-S9.

As a developer comparing a file whose name or bytes mislead detection, I can choose the file format and encoding for each side so that it is decoded and compared correctly.

**Acceptance criteria**
- [ ] The Format tab of Session Settings has a file-format picker per side. The default, **Detected**, means matched by mask through the file-format registry.
- [ ] The picker lists every registered format, including formats with an empty mask that never match automatically.
- [ ] Each side has an encoding override. The default, **None**, uses the format's encoding, or auto-detection when the format doesn't fix one.
- [ ] Applying an override reloads that side and recompares, prompting first when it has unsaved edits. The file info header shows the format and encoding in use.
- [ ] After an encoding override, saving writes the file in the overridden encoding (*inferred*).

#### Tasks
- **E07-S10-T1** — Format picker per side, listing every registered format with Detected as the default · size: S · depends: —
  - AC: an empty-mask format appears in the list, and once chosen the file info header names it.
- **E07-S10-T2** — Encoding override per side with None as the default · size: S · depends: —
  - AC: a Windows-1252 file mis-detected as UTF-8 shows the correct characters after Windows-1252 is chosen.
- **E07-S10-T3** — Reload with overrides (prompting when there are unsaved edits) and save in the chosen encoding · size: M · depends: E07-S10-T1, E07-S10-T2
  - AC: after the override is set to UTF-16 LE and the file is saved, the file on disk is UTF-16 LE.

---

### E08-S2 — Display filters and folder display modes
**Priority**: Must · **Layer**: 2 · **Size**: L · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view)
**Traces**: F-8.11, F-8.12, F-8.4
**Not in this story**: Ignore Unimportant Differences → E08-S13; Suppress Filters → E08-S15; toolbar layouts → E05-S4; remembering filters → E06-S4.

As a code reviewer, I can show only the rows that matter (differences, orphans, newer files on one side) so that I can focus on what changed in a large tree.

**Acceptance criteria**
- [ ] The View menu offers these display filters, and exactly one is active at a time:
  - *Show All*: everything.
  - *Show Differences*: hides matches.
  - *Show Same*: matches only.
  - *Show No Orphans*: hides orphans.
  - *Show Differences but No Orphans*: hides matches and orphans.
  - *Show Orphans*: orphans only.
  - *Show Left Newer* and *Show Right Newer*.
  - *Show Left Newer and Left Orphans* and *Show Right Newer and Right Orphans*.
  - *Show Left Orphans* and *Show Right Orphans*.
  - *Show None*, hidden by default, hides all files.
- [ ] The toolbar gives one-click access to Show All, Show Differences and Show Same, with the other filters in a dropdown.
- [ ] Folder display modes:
  - Default: a folder shows only if at least one file in it is visible.
  - *Always Show Folders*: every folder shows unless file filters exclude it.
  - *Compare Files and Folder Structure*: folders follow the display filter like files. Under Show Orphans, only orphan folders and folders containing orphans show.
  - *Compare Files Only*: suppresses every empty folder, whether truly empty or empty because of filtering.
  - *Ignore Folder Structure*: a flat list where files align by name regardless of subfolder, and each row shows its relative folder (*inferred*).
- [ ] *Show None* together with *Always Show Folders* shows only the folder structure.
- [ ] Changing a filter or mode updates the view without re-scanning the folders (*inferred*).

#### Tasks
- **E08-S2-T1** — Visibility rules for the 13 display filters · size: M · depends: —
  - AC: a unit table of row status (same, left newer, right newer, different, left orphan, right orphan) × filter gives the specified visibility in every cell.
- **E08-S2-T2** — Display-filter commands and default toolbar access · size: S · depends: E08-S2-T1 · flags: `naming-check`
  - AC: choosing a filter checks it in the View menu and updates the rows. Show None is hidden from menus by default but callable.
- **E08-S2-T3** — Folder visibility for the default mode, Always Show Folders, Compare Files and Folder Structure, and Compare Files Only · size: M · depends: E08-S2-T1 · flags: `naming-check`
  - AC: a fixture with empty folders, folders emptied by filtering and orphan folders gives the specified visibility in each mode, including Show Orphans with Compare Files and Folder Structure.
- **E08-S2-T4** — Ignore Folder Structure (flat list) · size: M · depends: —
  - AC: same-named files from different subfolders align on one row and show their relative folders. Turning the mode off brings back the tree.
- **E08-S2-T5** — Structure-only view with Show None and Always Show Folders · size: S · depends: E08-S2-T2, E08-S2-T3
  - AC: with both set, only folders show.

---

### E08-S3 — Name filters
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view), E02-S1 (file masks)
**Traces**: F-8.8, F-8.7, F-9.10, App-A
**Not in this story**: presets and Add To Presets → E08-S14; other filters → E08-S12; Suppress Filters → E08-S15; /filters → E11-S3.

As a developer, I can include or exclude items by name pattern so that build output, caches and other noise drop out of the comparison.

**Acceptance criteria**
- [ ] The toolbar *Filters* edit takes `;`-separated masks, where a `-` prefix excludes and a trailing `\` means a folder. `*.cs;-obj\;-bin\` shows only `.cs` files outside `obj` and `bin` folders.
- [ ] Masks follow the Appendix A.1 syntax, including `*.` (no extension) and the relative forms `p\f`, `.\f` and `...\f`. The base folder is always included.
- [ ] The *Filters* toolbar button opens Session Settings on the **Name Filters** tab. It has four multi-line lists: *Include Files*, *Exclude Files*, *Include Folders* and *Exclude Folders*, one mask per line and without the `-` prefix. **Clear** empties all four.
- [ ] The toolbar edit and the Name Filters tab always show the same filter (*inferred*).
- [ ] *Actions > Exclude* adds the selected items to the Exclude lists and hides them. When several files of one type are selected, it offers to exclude the whole type, such as `*.log`. Removing the entry from the Exclude lists brings the items back.
- [ ] Excluded items take no part in the comparison and don't affect folder colours.
- [ ] For any operation, a selected folder resolves to its visible contents only, so excluded items are never part of the resolved set.

#### Tasks
- **E08-S3-T1** — Feed the name masks into the comparison's visibility hook · size: M · depends: — · flags: `craft-libs`
  - AC: each Appendix A.1 example shows exactly the expected items on a fixture tree: `*.pas;*.dfm;*.dpr`, `Source\`, `-*.dcu;-*.ddp`, `-Backup\`, `Windows\*.txt`, `...\Windows\*.txt` and `.\Windows\*.txt`.
- **E08-S3-T2** — Toolbar Filters edit · size: S · depends: E08-S3-T1
  - AC: typing masks and pressing Enter re-filters. An invalid mask shows an error and keeps the previous filter (*inferred*).
- **E08-S3-T3** — Name Filters tab with four lists and Clear, opened by the Filters button · size: S · depends: E08-S3-T1
  - AC: a mask in Exclude Folders hides that folder, Clear restores the unfiltered view, and the toolbar edit reflects the tab's contents.
- **E08-S3-T4** — Actions > Exclude, with the offer to exclude a whole type · size: S · depends: E08-S3-T3
  - AC: excluding one file adds its name to Exclude Files. Excluding three `.log` files offers `*.log`. Removing the entry shows them again.
- **E08-S3-T5** — Resolved operation sets leave out filtered items · size: S · depends: E08-S3-T1
  - AC: selecting a folder whose `*.tmp` files are excluded resolves to its other files only.

---

### E08-S4 — Quick-test comparison criteria
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view)
**Traces**: F-8.3, F-8.4
**Not in this story**: Windows attributes → E08-S17; content tests → E08-S5; versions → E25-S3.

As a release engineer, I can choose which metadata decides "different" so that copies made by tools that shift timestamps or change case don't flood the view with false differences.

**Acceptance criteria**
- [ ] The Comparison tab's quick tests, each of which can be turned on or off:
  - **Compare file size**.
  - **Compare timestamps**, with a tolerance in seconds and the sub-options *Ignore daylight saving difference* (ignores an exact 1-hour difference) and *Ignore timezone differences* (ignores differences of a whole number of hours).
  - **Compare filename case**: different when only the capitalisation differs.
  - **Compare Unix metadata**: *Permissions* (user/group/other rwx and sticky), *Owner* ID and *Group* ID.
  - **Align filenames with different extensions**: lines up `a.txt` with `a.csv`.
  - **Align filenames with different Unicode normalization forms**: lines up NFC and NFD equivalents.
- [ ] Timestamps within the tolerance count as equal. Outside it, the later file is newer.
- [ ] A test that is turned off never contributes to "different".
- [ ] Criteria apply to files. Folders take their status from their contents.
- [ ] Closing Session Settings after a change recompares the view.

#### Tasks
- **E08-S4-T1** — Timestamp rules: tolerance, daylight saving and time zone · size: M · depends: —
  - AC: unit cases with a 2 s tolerance:
    - Files 1 s apart are same.
    - Files exactly 3600 s apart are same with Ignore daylight saving on.
    - Files 7200 s apart are same with Ignore timezone on.
    - Files 3599 s apart are different with both on.
    - The tolerance also applies around the 1-hour and whole-hour offsets (*inferred*).
- **E08-S4-T2** — Filename-case and Unix-metadata tests · size: S · depends: —
  - AC: `Readme.md` vs `README.md` differ only when Compare filename case is on. A permissions-only difference shows red only when Permissions is on.
- **E08-S4-T3** — Alignment across extensions and Unicode normalization forms · size: M · depends: —
  - AC: `a.txt` and `a.csv` share a row only with the extension option on. An NFC name and its NFD twin share a row only with the Unicode option on.
- **E08-S4-T4** — Quick-test controls on the Comparison tab; applying recompares · size: S · depends: E08-S4-T1, E08-S4-T2, E08-S4-T3 · flags: `naming-check`
  - AC: each control switches its test, and closing the dialog after a change updates the colours.

---

### E08-S5 — Content comparison by CRC and binary
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view)
**Traces**: F-8.3, F-8.1
**Not in this story**: rules-based → E08-S13; comparison triggered by opening a pair → E08-S10.

As a QA engineer, I can check that files really have the same content, not just the same size and date, so that I can trust "same" before a release.

**Acceptance criteria**
- [ ] The Comparison tab has **Compare contents**, with either *CRC comparison* or *Binary comparison* (byte by byte).
- [ ] With **Skip if quick tests indicate files are the same** on, only pairs that the quick tests say differ are read.
- [ ] **Override quick test results** is on by default: a content match turns the row black even when timestamps differ.
- [ ] A center column between the panes shows each pair's result: *Binary same* or *Binary differences*, each with its own glyph (*inferred* glyphs).
- [ ] With Compare contents on, content tests run when the session loads.
- [ ] *Actions > Compare Contents* runs a content test on demand for the selection, letting the user choose CRC or Binary.
- [ ] Content tests run in the background with progress and can be cancelled. A folder containing a binary-different pair shows as different.

- [ ] Clicking a row's cell in the center column selects both sides of that row.
#### Tasks
- **E08-S5-T1** — Content options on the Comparison tab: Compare contents (CRC or Binary), Skip if quick tests indicate files are the same, Override quick test results · size: S · depends: —
  - AC: the options are kept in the session settings, and changing them recompares.
- **E08-S5-T2** — Run content tests on load and fold the results into row status, applying the skip and override rules · size: M · depends: E08-S5-T1 · flags: `licence-check`
  - AC: a pair with equal content but different timestamps shows black with Override on and red with it off. With Skip on, quick-same pairs get no content result.
- **E08-S5-T3** — Center column with content results · size: S · depends: E08-S5-T2
  - AC: binary-same and binary-different pairs show distinct glyphs, with a tooltip naming the result.
- **E08-S5-T4** — Actions > Compare Contents on the selection, choosing CRC or Binary, with progress and cancel · size: M · depends: E08-S5-T2
  - AC: running it on a selected folder fills the center column for its visible pairs only. Cancel leaves the remaining pairs untested.
- **E08-S5-T5** — Select both sides by clicking the center column · size: S · depends: E08-S5-T3
  - AC: clicking a row's center cell selects its left and right items. Ctrl+click on another center cell adds both of that row's items.

---

### E08-S6 — Columns: choose, sort and extra metadata
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view)
**Traces**: F-8.2
**Not in this story**: Status and Revision → E26-S4; Version → E25-S3; DOS attribute letters in names → E08-S17.

As an IT administrator, I can choose which file details show and sort by any of them so that I can spot size, ownership or permission differences quickly.

**Acceptance criteria**
- [ ] Clicking a column header sorts by that column, and clicking again reverses the order. Rows stay paired while sorted.
- [ ] Right-clicking the header, or *View > Columns*, shows or hides **Name**, **Ext**, **Size**, **CRC**, **Modified**, **Attributes**, **Owner** and **Group**.
- [ ] **Ext** shows the extension.
- [ ] **Size** shows the file size, or for a folder the total size of its contents. A folder's Size stays blank until its contents are fully loaded.
- [ ] **CRC** shows the CRC32. A value the data source already holds, such as one stored in a zip, is used as-is. Otherwise it is computed on demand.
- [ ] **Attributes** shows Windows attribute codes (A archive, C compressed, D directory, E encrypted, H hidden, I not indexed, L symlink or reparse point, M recall on data access, N normal, O offline, P pinned, R read-only, S system, T temporary, U unpinned, Z sparse) or a Unix mode string like `ls` shows.
- [ ] **Owner** and **Group** show the Unix owner and group.
- [ ] Column choices and sort order stay in place through a refresh (*inferred*).

#### Tasks
- **E08-S6-T1** — Column chooser (header right-click and View > Columns) · size: S · depends: —
  - AC: hiding Size removes it from both panes, and the choice survives a refresh.
- **E08-S6-T2** — Sort by any visible column · size: S · depends: —
  - AC: sorting by Size orders files within each folder ascending, then descending on a second click, and pairs stay on one row.
- **E08-S6-T3** — Ext, CRC and folder Size values · size: M · depends: —
  - AC: a folder's Size equals the sum of its files once loaded and is blank before. CRC shows the source-provided value when one exists and computes it otherwise.
- **E08-S6-T4** — Attributes, Owner and Group values per platform · size: S · depends: — · flags: `licence-check`
  - AC: on Windows, a read-only hidden file shows the R and H codes. On Unix, a file shows a mode string such as `-rw-r--r--` plus owner and group names.

---

### E08-S7 — Selection commands
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view)
**Traces**: F-8.13, F-8.15, F-9.10
**Not in this story**: Space marking → E08-S10; selecting both sides by clicking the center column → E08-S5.

As a power user, I can select exactly the items I want with the mouse, the keyboard and selection commands so that later actions hit the right files.

**Acceptance criteria**
- [ ] Press-and-drag ("swipe") selects a range. A swipe that crosses the middle selects both sides.
- [ ] *Select All* selects all visible items, and *Select All Files* selects visible files only.
- [ ] After a both-sides selection, Shift+Left arrow limits it to the left side and Shift+Right arrow to the right side.
- [ ] *Select Newer* (left, right or both) selects the items that are newer on that side. *Select Orphans* (left, right or both) selects orphans on that side.
- [ ] *Invert Selection* selects the visible unselected items and deselects the rest.
- [ ] *Copy Filename* copies the full paths of the selected items to the clipboard, one per line (*inferred* separator).

#### Tasks
- **E08-S7-T1** — Swipe selection across one or both sides · size: M · depends: —
  - AC: a swipe from the left pane across the middle selects rows on both sides. A swipe inside one pane selects that side only.
- **E08-S7-T2** — Select All and Select All Files · size: S · depends: —
  - AC: hidden rows are never selected, and Select All Files leaves folders unselected.
- **E08-S7-T3** — Shift+Left/Right limits the selection to one side · size: S · depends: —
  - AC: after Select All, Shift+Right leaves only right-side items selected.
- **E08-S7-T4** — Select Newer, Select Orphans and Invert Selection · size: S · depends: —
  - AC: on a fixture, Select Newer (left) selects exactly the left-newer items, and Invert flips the selection within the visible items.
- **E08-S7-T5** — Copy Filename · size: S · depends: —
  - AC: with two files selected, both full paths are on the clipboard.

---

### E08-S8 — Base-folder navigation and history
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view)
**Traces**: F-8.14, F-6.1
**Not in this story**: Sync Base Folders → E12-S1; Merge Base Folders → E14-S1.

As a developer, I can re-root the comparison on any pair of subfolders and step back and forth through what I compared so that I can drill into one area and return.

**Acceptance criteria**
- [ ] *Set as Base Folder* makes the selected folder the base on its own side. *Set as Base on Other Side* makes it the base on the opposite side.
- [ ] *Set as Base Folders* makes two selected folders the left and right bases in this view.
- [ ] *Up One Level* (left, right or both) moves the base to its parent.
- [ ] *Browse for Folder* (left or right) picks a new base with a folder dialog.
- [ ] *Compare Parent Folders* opens a new Folder Compare view on the parents of the current bases.
- [ ] *Back* and *Forward* step through the base-folder pairs previously compared in this view.

#### Tasks
- **E08-S8-T1** — Set as Base Folder, Set as Base on Other Side and Set as Base Folders · size: M · depends: —
  - AC: each command loads the expected pair. Set as Base Folders is enabled only with exactly two folders selected (*inferred*).
- **E08-S8-T2** — Up One Level (left, right, both) and Browse for Folder (left, right) · size: S · depends: —
  - AC: Up One Level on both sides shows the two parents. At a root it is disabled (*inferred*).
- **E08-S8-T3** — Compare Parent Folders in a new view · size: S · depends: —
  - AC: a new tab opens on the two parent folders, and the original view is unchanged.
- **E08-S8-T4** — Back/Forward history of base-folder pairs · size: S · depends: E08-S8-T1, E08-S8-T2
  - AC: after three re-roots, three Backs return to the first pair and Forward replays them. A new re-root after Back clears the forward history (*inferred*).

---

### E08-S9 — Step through differences and find filenames
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view), E02-S1 (regular expressions)
**Traces**: F-8.14

As a code reviewer, I can jump from one differing item to the next and find items by name so that I can walk a large comparison without scrolling.

**Acceptance criteria**
- [ ] *Next Difference* and *Previous Difference* move to the next or previous differing item, skipping same and hidden rows. They open collapsed folders when the next difference is inside one (*inferred*).
- [ ] The **Find Filename** dialog has these options: text, *Match character case*, *Whole words only*, *Regular expression* (with the Insert Regular Expression Element picker and RE Examples), *Wrap search*, *Search up* and *Sides* (left, right or both).
- [ ] *Find Next Filename* and *Find Previous Filename* repeat the last search.
- [ ] When no further match or difference exists, the view says so (*inferred*).

#### Tasks
- **E08-S9-T1** — Next/Previous Difference across the tree · size: M · depends: —
  - AC: from the top of a fixture, repeated Next Difference visits every differing item in display order and never a same item.
- **E08-S9-T2** — Find Filename dialog with every option · size: M · depends: —
  - AC: regex `^test_.*\.py$` with Sides = left finds only left-side matches. With Whole words only, `apple` doesn't match `apples`. Matches inside collapsed folders are found and revealed (*inferred*).
- **E08-S9-T3** — Find Next/Previous Filename with wrap and search-up · size: S · depends: E08-S9-T2
  - AC: with Wrap search on, Find Next after the last match returns to the first. Search up reverses the direction.

---

### E08-S10 — Open file pairs in file views
**Priority**: Must · **Layer**: 2 · **Size**: L · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view), E07-S1 (Text Compare view)
**Traces**: F-8.15, F-6.4, F-8.3
**Not in this story**: Open With → E08-S16; Quick Compare → E08-S19; Copy File and Open Next Difference → E08-S18; child scopes and placement → E06-S7; read-only children → E06-S8; limit to current folder → E07-S19; child commands in other views → E06-S11.

As a code reviewer, I can open any file pair from the folder view in a file comparison and step to the next differing pair so that I can review a whole change set file by file.

**Acceptance criteria**
- [ ] *Open* (Enter or double-click) on a file row opens a child session in a new tab, in the view that the file's format chooses. Text formats open in Text Compare. On a row the view can expand (a folder, or an item a data source exposes as a container), Open expands it instead.
- [ ] With exactly two items selected, Open compares them with each other, whatever their rows or names.
- [ ] With one item selected, Open compares it with the item across from it. An orphan opens with the other side empty (*inferred*).
- [ ] *Compare To*: select one item, choose Compare To, then click a second item. The two open in a new view.
- [ ] Space marks an item, and the mark is visible. With two items marked, Open compares them even across rows and names.
- [ ] Opening a pair runs its comparison, and the file view's result (same or different) becomes the row's content result, so the row's colour follows it the same way it follows a content test.
- [ ] Text Compare child sessions offer *Next Difference Files* and *Previous Difference Files*. These open the parent's next or previous pair with differences, in the parent's display order. When none is left in that direction, the view says so (*inferred*).

#### Tasks
- **E08-S10-T1** — Open/double-click a row into a child session, with the view chosen by the file's format · size: M · depends: —
  - AC: double-clicking a `.txt` pair opens a Text Compare tab with both files, and the tab is linked to its parent folder view. Double-clicking a test source's container row expands it instead.
- **E08-S10-T2** — Pair-selection rules: two selected items, one item and its partner across, Compare To · size: M · depends: E08-S10-T1
  - AC: with `a.txt` on the left and `b.txt` on the right of another row selected, Open compares `a.txt` with `b.txt`. Compare To compares the first item with the clicked one.
- **E08-S10-T3** — Space marks and opening two marked items · size: S · depends: E08-S10-T2
  - AC: marking two files in different folders and pressing Open compares them. Marks clear after opening (*inferred*).
- **E08-S10-T4** — Report child results back to the parent row · size: M · depends: E08-S10-T1
  - AC: opening a pair with identical content but different timestamps turns the row black, because override-quick-results is on by default. Child results map to same or different. Saving an edit in the child updates the row (*inferred*).
- **E08-S10-T5** — Next/Previous Difference Files in Text Compare child sessions · size: M · depends: E08-S10-T1, E08-S10-T4
  - AC: from the first differing pair, Next Difference Files opens the next differing pair in display order and skips same pairs. At the last pair it reports that none is left.

---

### E08-S11 — Folder loading and scanning behaviour
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view)
**Traces**: F-8.6, F-8.1
**Not in this story**: Archive handling → E04-S2; FTP touch → E21-S2; Windows options → E08-S17; Automatic refresh → E08-S20.

As an IT administrator comparing large or slow trees, I can control how and when subfolders are scanned and expanded so that I see accurate colours quickly without waiting for everything.

**Acceptance criteria**
- [ ] The Handling tab has these options:
  - *Automatically scan subfolders in background*: reads subfolders after load so folders are coloured before they are opened.
  - *Automatically scan top-level orphan subfolders*: scans orphan folders too, which would otherwise be coloured without scanning.
  - *Expand subfolders when loading session*, with the sub-option *Only expand subfolders with differences*.
  - *Follow symbolic links*: shows symlinks and junctions as their target, with its type, size, modified time and attributes.
- [ ] With background scanning off, a folder that hasn't been opened yet shows as a **yellow folder**.
- [ ] A folder whose background scan hasn't finished shows as a **hollow folder**.
- [ ] The user can browse, expand and select while background scanning runs (*inferred*).

#### Tasks
- **E08-S11-T1** — Background subfolder scanning with the hollow-folder cue · size: M · depends: —
  - AC: on a deep fixture, folders show hollow until scanned and then show their aggregated colour, while other folders can be expanded meanwhile.
- **E08-S11-T2** — Unscanned yellow folders when background scanning is off · size: S · depends: —
  - AC: with the option off, unopened folders are yellow, and opening one scans and colours it.
- **E08-S11-T3** — Scan top-level orphan subfolders · size: S · depends: E08-S11-T1
  - AC: with the option on, an orphan folder is scanned in the background like a paired one. With it off, the folder is coloured as an orphan without being read.
- **E08-S11-T4** — Expand subfolders when loading, optionally only those with differences · size: S · depends: —
  - AC: loading with the sub-option on opens exactly the folders that contain differences.
- **E08-S11-T5** — Follow symbolic links · size: S · depends: —
  - AC: with the option on, a link to a folder shows the target's contents and metadata. With it off, it shows as a link item with the link glyph.

---

### E09-S1 — Copy to the other side
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view), E04-S1 (file-system access layer)
**Traces**: F-9.1, F-26.10 · **Open questions**: Q3
**Not in this story**: Copy/Move to Folder → E09-S5; backups → E09-S10; log panel → E09-S11; including filtered-out items → E08-S15; Copy File and Open Next Difference → E08-S18.

As a developer reconciling two source trees, I can copy the selected items to the other side so that the target matches the source where I choose.

**Acceptance criteria**
- [ ] *Copy to Other Side* adapts to the selection. Its caption and icon read "Copy to Right" for a left-only selection, "Copy to Left" for a right-only selection, and "Copy to Side…" when both sides are selected.
- [ ] *Copy to Right* (Ctrl+R) and *Copy to Left* (Ctrl+L) are hidden by default and run from those shortcuts. *Copy to Side* asks for the direction first.
- [ ] Copying overwrites existing items on the target and creates any missing parent folders.
- [ ] A selected folder copies only its visible contents. With an open folder plus some of its children selected, only those children are copied.
- [ ] A confirmation shows the item count and direction. The "Confirm file copy" option on the File Operations page can skip it for left-only or right-only selections. Copy to Side still asks for the direction.
- [ ] Progress shows while copying, and Cancel stops after the current item. Failed items are listed in the end-of-task summary while the remaining items still copy.
- [ ] Copied rows re-compare immediately. A copied orphan becomes a paired row, and identical results turn black.
- [ ] A copy onto a read-only data source is refused with a message, and nothing changes.

#### Tasks
- **E09-S1-T1** — Turn a selection into a copy plan (items, direction, target paths, folders to create) · size: S · depends: —
  - AC: unit cases cover a single folder, an open folder with selected children, a mixed file-and-folder selection and an orphan folder. Each yields exactly the visible items with the correct target paths.
- **E09-S1-T2** — Copy to Other Side, Copy to Right/Left and Copy to Side commands with a dynamic caption and icon · size: M · depends: E09-S1-T1
  - AC: caption and icon follow the three selection states. Ctrl+R and Ctrl+L work while hidden from menus. Copy to Side prompts for the direction.
- **E09-S1-T3** — Copy confirmation dialog and the "Confirm file copy" option · size: S · depends: E09-S1-T2
  - AC: the dialog shows the count and direction. With the option off, a left-only or right-only copy runs without a dialog.
- **E09-S1-T4** — Run the copy with progress, cancel and error summary · size: M · depends: E09-S1-T1
  - AC: cancelling halfway leaves completed items copied and reports the rest as not done. A locked target file appears in the summary and the other items still complete. A read-only target is refused before any write.
- **E09-S1-T5** — Re-compare affected rows after the copy · size: S · depends: E09-S1-T4
  - AC: copied rows show their new status with no full refresh. Untouched rows keep their state.

---

### E09-S2 — Move to the other side
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view), E04-S1 (file-system access layer)
**Traces**: F-9.2, F-26.10 · **Open questions**: Q3
**Not in this story**: Move to Folder → E09-S5; exchanging both sides → E09-S12; backups → E09-S10.

As a user tidying two copies of a project, I can move items to the other side so that I can relocate files without copying and then deleting them by hand.

**Acceptance criteria**
- [ ] *Move to Other Side* has a dynamic caption that follows the selection: "Move to Right" for a left-only selection, "Move to Left" for a right-only selection, "Move to Side…" when both sides are selected. *Move to Right*, *Move to Left* and *Move to Side* are hidden by default, and Move to Side asks for the direction.
- [ ] Each item is written to the target first and removed from the source only after that write succeeds. When the write fails, the source stays.
- [ ] A selected folder moves only its visible contents. With an open folder plus some of its children selected, only those children move. A source folder is removed only when nothing hidden remains in it (*inferred*).
- [ ] A confirmation appears unless "Confirm file move" is off.
- [ ] Progress shows while moving, and Cancel stops after the current item. Failures are listed in the end-of-task summary while the remaining items still move. Rows refresh afterwards: a moved orphan shows on the target side only.

#### Tasks
- **E09-S2-T1** — Move plan with remove-after-successful-write semantics · size: S · depends: —
  - AC: when the write fails for an item, that item's source still exists and the item is listed in the summary.
- **E09-S2-T2** — Move commands with a dynamic caption, and Move to Side direction prompt · size: S · depends: E09-S2-T1
  - AC: caption follows the selection side. The hidden-by-default commands are callable by shortcut.
- **E09-S2-T3** — Move confirmation and the "Confirm file move" option · size: S · depends: E09-S2-T2
  - AC: with the option off, a one-sided move runs without a dialog.
- **E09-S2-T4** — Run the move with progress, cancel, error summary and row refresh · size: M · depends: E09-S2-T1
  - AC: after a cancel, finished items exist only on the target and unfinished items only on the source.

---

### E09-S3 — Delete
**Priority**: Must · **Layer**: 2 · **Size**: S · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view), E04-S1 (file-system access layer)
**Traces**: F-9.5, F-26.10 · **Open questions**: Q3
**Not in this story**: log panel → E09-S11.

As a user cleaning up a comparison, I can delete items from one or both sides so that I can remove stale files without leaving the tool.

**Acceptance criteria**
- [ ] *Delete* removes the selected items from whichever side or sides they are selected on.
- [ ] The confirmation has a **Use recycle bin if possible** checkbox, which applies to local drives only. Items on sources that report no recycle-bin support, such as network paths, are deleted permanently, and the dialog says so (*inferred*).
- [ ] A selected folder deletes its visible contents. The folder itself goes only when nothing hidden remains in it (*inferred*).
- [ ] The confirmation can be turned off with its own option (Q3).
- [ ] After deletion, rows update: a pair becomes an orphan on the other side, and a deleted orphan disappears.

#### Tasks
- **E09-S3-T1** — Delete plan for one or both sides, with the folder rule · size: S · depends: —
  - AC: unit cases for a left-only selection, a both-sides selection and a folder containing hidden items.
- **E09-S3-T2** — Delete command and confirmation with the recycle-bin checkbox · size: S · depends: E09-S3-T1 · flags: `licence-check`
  - AC: the checkbox is disabled, with an explanation, when every item is on a source without a recycle bin.
- **E09-S3-T3** — Run the delete with progress and errors, then refresh rows · size: S · depends: E09-S3-T1
  - AC: a file in use is reported and the others are deleted. Rows reflect the new state without a full refresh.

---

### E09-S4 — Synchronize the selection
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E08-S1 (Folder Compare view), E04-S1 (file-system access layer)
**Traces**: F-8.18, F-26.10 · **Open questions**: Q3
**Not in this story**: saved, whole-tree sync sessions → E12-S1; script `sync` → E20-S4.

As a user keeping a backup folder current, I can synchronise just the selected part of a comparison in one command so that I don't have to copy and delete item by item.

**Acceptance criteria**
- [ ] The *Actions > Synchronize* submenu offers *Update Right*, *Update Left*, *Update Both*, *Mirror to Right* and *Mirror to Left*. Each acts on the current selection only.
- [ ] *Update Right* copies left-newer and left-orphan items to the right and deletes nothing. *Update Left* does the same in the other direction. *Update Both* copies newer and orphan items in both directions.
- [ ] *Mirror to Right* replaces every different right item with the left version, copies left orphans and deletes right orphans. *Mirror to Left* does the same in the other direction.
- [ ] A confirmation shows the planned copies and deletes as counts before anything changes.
- [ ] During the run, an item whose target is newer than the source or is read-only raises a per-item confirmation (*inferred*). The "Synchronize confirmations" option (Prompt, the default; Yes to All; No to All) decides how those confirmations are answered.
- [ ] Progress shows while synchronising, and Cancel stops after the current item. Failures are listed in the end-of-task summary, and rows refresh afterwards.

#### Tasks
- **E09-S4-T1** — Sync plan per method from row statuses · size: M · depends: —
  - AC: a unit table of status (same, left newer, right newer, different, left orphan, right orphan) × the five methods gives the expected copy or delete action for every cell.
- **E09-S4-T2** — Synchronize submenu and plan confirmation · size: S · depends: E09-S4-T1
  - AC: the confirmation counts match the plan. Choosing No changes nothing.
- **E09-S4-T3** — "Synchronize confirmations" option · size: S · depends: E09-S4-T2
  - AC: Yes to All runs with no further prompts. No to All skips every item that would have prompted.
- **E09-S4-T4** — Run the plan with progress, cancel and error summary, then refresh rows · size: M · depends: E09-S4-T1
  - AC: after Mirror to Right on a fully selected tree, the whole tree compares as same.

---

### E10-S1 — File Formats manager
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E02-S1 (file-format registry and masks), E07-S1 (Text Compare view)
**Traces**: F-21.1, F-21.2
**Not in this story**: Grammar tab → E10-S3; Misc tab → E10-S4; Conversion tab → E10-S6; Table Type/Regional tabs → E15-S7; External kind → E10-S9; per-session enablement → E10-S8.

As a developer who works with uncommon file types, I can manage the ordered list of file formats so that each of my files is recognised and handled the way I want.

**Acceptance criteria**
- [ ] *Tools > File Formats* shows a single ordered list of every format, with its kind, mask and enabled state.
- [ ] **Move Up** and **Move Down** reorder formats. Resolution follows the new order at once: the first enabled format whose mask matches wins.
- [ ] A checkbox enables or disables each format. Disabled formats show gray and never match.
- [ ] A format shadowed by those above it, because every name it could match is already matched higher up, shows red.
- [ ] *New* adds a format of any registered kind, *Duplicate* copies one, *Edit* opens the format editor, and *Delete* removes a custom format. Built-in formats can be edited and disabled but not deleted (*inferred*).
- [ ] The **General** tab holds the **Mask** (semicolon-separated masks) and the **Description** (free text; the built-in descriptions note limitations and requirements).
- [ ] A format with an empty mask never matches automatically.
- [ ] A file whose name matches a custom Text format's mask shows that format's name in the Text Compare file info header.
- [ ] The list, its order and every edit survive a restart.

- [ ] On macOS, *File Formats* is in the application menu. On Windows and Linux, it is under *Tools*.
#### Tasks
- **E10-S1-T1** — Manager list with order, enable checkbox, and kind and mask columns · size: M · depends: —
  - AC: swapping the order of two overlapping formats changes which one a test filename resolves to.
- **E10-S1-T2** — Shadowing detection with red and gray cues · size: M · depends: E10-S1-T1 · flags: `accessibility`
  - AC: `*.txt` below `*.*` shows red, and above it does not. A disabled format shows gray. Complex masks may use a conservative check that never marks a reachable format red (*inferred*).
- **E10-S1-T3** — New, Duplicate, Edit and Delete, with the General tab (Mask, Description) · size: M · depends: E10-S1-T1
  - AC: a new Text format with mask `*.foo` makes `a.foo` show that format in Text Compare's file info header. Deleting it restores the previous match.
- **E10-S1-T4** — Save the format list and edits in the settings store · size: S · depends: E10-S1-T3
  - AC: after a restart, the order, enabled states and custom formats are unchanged.

---

### E10-S2 — Syntax highlighting with built-in grammars
**Priority**: Must · **Layer**: 2 · **Size**: L · **Platforms**: All
**Needs**: E07-S1 (Text Compare view), E02-S1 (file-format registry and regular expressions), E05-S1 (light and dark themes)
**Traces**: F-21.4, F-12.1 · **Open questions**: Q8
**Not in this story**: grammar editing and line weights → E10-S3; importance by element → E10-S5; colour customisation → E22-S3.

As a developer, I can see source code syntax-highlighted in Text Compare so that I can read code changes as easily as in my editor.

**Acceptance criteria**
- [ ] A Text format can carry a grammar: an ordered list of grammar items, each assigned to a named element such as Comment, String, Keyword or a custom name.
- [ ] The grammar supports all five item categories (Basic, Delimited, List, Columns, Lines) with the fields described in the behavioral contract. For example, `'I can\'t'` with `\` as the escape character is one string.
- [ ] Higher items take precedence over lower ones where matches overlap.
- [ ] Each element has **This element is case sensitive**, which governs how its items match.
- [ ] Built-in Text formats with grammars exist for the starter set decided in T1, and each matches its usual masks.
- [ ] *View > Syntax Highlighting* turns highlighting on or off in Text Compare. Every element has a default colour that is readable in the light and dark themes.
- [ ] The grammar has no nesting and no context-specific rules. Mixed-language files are highlighted by their own format's grammar only.

#### Tasks
- **E10-S2-T1** — Decide the starter language set and built-in format list (Q8) · size: S · depends: — · flags: `decision`
  - AC: decision recorded in this story. Proposed default:
    - C/C++, C#, Java, JavaScript/TypeScript, Python, Go, Rust
    - HTML/XML, CSS
    - JSON, YAML, SQL, shell scripts, Markdown, INI/properties
    - a plain Text format
- **E10-S2-T2** — Grammar model and matcher for the five item categories, with precedence and element case sensitivity · size: L · depends: — · flags: `craft-libs`
  - AC: unit cases cover each category: an escaped delimiter, a List keyword with and without case, a Columns range, and a Lines header with 2 heading lines. They also cover overlap precedence.
- **E10-S2-T3** — Built-in formats and grammars for the starter set · size: M · depends: E10-S2-T1, E10-S2-T2 · flags: `licence-check`
  - AC: each built-in format matches its masks and highlights comments, strings and keywords in a sample file of its language.
- **E10-S2-T4** — Syntax highlighting in Text Compare, with the View > Syntax Highlighting toggle · size: M · depends: E10-S2-T2 · flags: `accessibility`
  - AC: turning highlighting off removes element colours but keeps difference highlighting. The default element colours are readable in both themes.

---

### E11-S1 — Diff tool for version control
**Priority**: Must · **Layer**: 2 · **Size**: L · **Platforms**: All
**Needs**: E07-S1 (Text Compare view), E08-S1 (Folder Compare view)
**Traces**: F-25.1, F-25.2, F-25.3, F-25.4, F-28.3
**Not in this story**: `/qc` → E11-S2, E08-S13; `/filters`, `/expandall` → E11-S3; stdin → E11-S4; merge switches → E11-S5; `/sync` → E12-S1; `/edit` → E24-S1; `/nobackups` → E09-S10; `/silent`, `/closescript`, `@script` → E20-S1; `.diff`/`.patch` → E24-S2; snapshots → E19-S1; settings packages → E23-S1; session and workspace names → E06-S1, E06-S6; three or four files → E13-S1.

As a developer, I can set OverTheDiff as my version-control diff tool so that `git difftool` and other VCS commands open comparisons in it, wait until I've finished and get a result code back.

**Acceptance criteria**
- [ ] Waiting commands exist on every platform:
  - On Windows, a GUI-waiting helper waits until the comparison closes. A console that launches this helper doesn't wait.
  - On Windows, a console-waiting helper makes the console or batch file wait. Launched from a VCS, it shows a console window while it waits.
  - On macOS and Linux, a waiting command does the same (*inferred* for Linux).
- [ ] On macOS, *Install Command Line Tools* in the application menu installs a waiting command and a non-waiting command on the PATH, after asking for an administrator password.
- [ ] A waiting command opens the comparison in the running instance, or starts one, and exits with code 0 when the user closes the comparison. It exits with 102 when it can't wait for the main application, 103 when it can't find the main application, and 100 on any other failure.
- [ ] `/title1`…`/title4` (aliases `/lefttitle`, `/righttitle`, `/centertitle`, `/outputtitle`) set the text a path edit shows instead of the path. The effect is checked for the panes the opened view has. `/title3`, `/title4` and their aliases are accepted and passed along for views with a center or output pane.
- [ ] `/vcs1`…`/vcs4` (aliases `/vcsleft`, `/vcsright`, `/vcscenter`, `/vcsoutput`) set the VCS path a path edit shows when no title is given. The file format is then picked from that path instead of from the temporary file's name. As with titles, `/vcs3`, `/vcs4` and their aliases are accepted and passed along for views with a center or output pane.
- [ ] `/ro` (`/readonly`) makes every side read-only. `/ro1` (`/lro`, `/leftreadonly`) makes only the left side read-only, and `/ro2` (`/rro`, `/rightreadonly`) only the right. A read-only pane refuses edits.
- [ ] `/savetarget=<file>` makes Save in the file view write to `<file>` instead of the original.
- [ ] `/fv=<type>` (`/fileviewer=<type>`) forces the view type. Accepted names: "Text Compare", "Text Merge", "Table Compare", "Hex Compare", "Media Compare", "Picture Compare", "Registry Compare", "Version Compare", "Folder Compare", "Folder Merge", "Folder Sync", "Text Edit" and "Text Patch". A name whose view type isn't registered gives an error that lists the registered types.
- [ ] `/solo` requests a new instance instead of handing off to the running one.
- [ ] `/?`, `/h` and `/help` show the usage text. On macOS and Linux every switch also works with a `-` prefix. Parameters that contain spaces work when quoted.
- [ ] The usage text, or a guide shipped with the product, includes a working `git difftool` configuration. In a test repository, `git difftool` run with that configuration opens Text Compare with the VCS paths as titles and returns after the view is closed.

- [ ] The waiting commands request the forward-and-wait launch mode for whichever handler claims their arguments. The comparison opens in the running instance (or a new one) while the launching process waits for it to close.
#### Tasks
- **E11-S1-T1** — Waiting launchers: Windows GUI-waiting and console-waiting helpers, macOS and Linux waiting command · size: L · depends: — · flags: `licence-check`
  - AC: each launcher blocks until its comparison closes, then exits with 0. Ending the main application while a launcher waits gives 102. With no main application installed, the launcher gives 103.
- **E11-S1-T2** — Install Command Line Tools on macOS · size: S · depends: E11-S1-T1
  - AC: after the install (with the admin password), both commands run from a new terminal. The waiting command blocks and the other returns at once.
- **E11-S1-T3** — Title and VCS path switches with all aliases · size: M · depends: —
  - AC: `/title1=Base /vcs2=src/app.c` shows "Base" on the left and "src/app.c" on the right. The right side uses the format a test harness registered for `*.c`, even though its temporary file is named `app_tmp123`.
- **E11-S1-T4** — Read-only switches and `/savetarget` · size: S · depends: —
  - AC: with `/ro1`, the left pane refuses typing and the right pane accepts it. With `/savetarget=out.txt`, Save writes `out.txt` and leaves the original unchanged.
- **E11-S1-T5** — `/fv`, `/solo`, usage text, and the `-` prefix and quoting rules · size: S · depends: —
  - AC: `/fv="Text Compare"` on two files opens Text Compare. `-fv="Text Compare"` works the same on macOS and Linux. `/solo` starts a second process. `/?` prints the usage text, and an unknown switch prints it too and returns 100.
- **E11-S1-T6** — Exit codes 0 and 100 and the git difftool guide · size: S · depends: E11-S1-T1, E11-S1-T3
  - AC: following the guide in a test repository, `git difftool` opens one comparison per changed file and continues after each one closes. Every run exits with 0.

---

### E08-S13 — Rules-based content comparison
**Priority**: Must · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E08-S5 (CRC and binary content comparison), E07-S4 (text importance rules), E06-S2 (session defaults, the source of the text rules), E11-S2 (headless `/qc`)
**Traces**: F-8.3, F-8.11, F-25.3, F-25.4, F-21.1 · **Open questions**: Q17

As a developer, I can have the folder view compare file contents with the same rules as Text Compare so that files differing only in whitespace, line endings or other unimportant ways don't show as changed.

**Acceptance criteria**
- [ ] *Compare contents* offers **Rules-based comparison** alongside CRC and Binary.
- [ ] A rules-based comparison runs a binary check first. Files that differ are then compared through their file formats with the text comparison rules, taken from the Text Compare session defaults (*inferred*). Files without a text format fall back to binary (*inferred*).
- [ ] The center column shows one of four results:
  - *Binary same*.
  - *Rules-based same*: byte differences the rules ignore, such as encoding or line endings.
  - *Unimportant differences*.
  - *Important differences*.
- [ ] *Actions > Compare Contents* offers Rules-based for the selection.
- [ ] *View > Ignore Unimportant Differences* makes pairs with only unimportant differences count as same in colouring and statistics.
- [ ] `/qc` with no type compares two files rules-based without UI. The exit code is 1 for binary same, 2 for rules-based same, 12 for similar and 13 for rules-based differences, per T1. A load error exits with 100.

#### Tasks
- **E08-S13-T1** — Decide when "similar" (exit code 12) applies (Q17) · size: S · depends: — · flags: `decision`
  - AC: decision recorded in this story. Proposed default:
    - 12 when only unimportant differences remain.
    - 13 when any important difference remains.
    - 2 when only byte differences that the rules ignore (encoding, line endings) remain.
- **E08-S13-T2** — Rules-based content test: a binary check, then a text comparison with the file's format and the text rules · size: M · depends: E08-S13-T1
  - AC: in a fixture:
    - Identical files give Binary same.
    - CRLF vs LF only gives Rules-based same.
    - A trailing-whitespace-only change, with trailing whitespace unimportant, gives Unimportant differences.
    - A changed word gives Important differences.
- **E08-S13-T3** — Rules-based option in Compare contents and in Actions > Compare Contents; center-column results · size: S · depends: E08-S13-T2 · flags: `naming-check`
  - AC: the four results show distinct glyphs and tooltips, and an on-demand run on a selection fills them in.
- **E08-S13-T4** — Ignore Unimportant Differences in Folder Compare · size: S · depends: E08-S13-T2
  - AC: with the toggle on, an Unimportant-differences pair is black and counted as same in Folder Compare Info. With it off, the pair counts as different.
- **E08-S13-T5** — Headless rules-based `/qc` with exit codes 1, 2, 12, 13 and 100 · size: S · depends: E08-S13-T2
  - AC: the four fixture pairs from T2 exit with 1, 2, 12 and 13. A missing file exits with 100.

---

### E10-S5 — Grammar-based importance
**Priority**: Must · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E10-S2 (grammar model and built-in grammars), E07-S4 (importance rules)
**Traces**: F-12.6, F-6.3

As a developer, I can mark grammar elements such as comments as unimportant so that comment-only changes don't distract from code changes.

**Acceptance criteria**
- [ ] The **Importance** tab lists the grammar elements of the session's current file format (comments, strings, keywords…) together with the built-in elements.
- [ ] Unticking an element makes differences inside it unimportant: blue, and treated as same under Ignore Unimportant Differences.
- [ ] An element's case sensitivity follows the grammar's *This element is case sensitive* setting, not the *Everything else* Character case box.
- [ ] Orphan lines stay important even when they hold only an unimportant element.
- [ ] When the two sides use different formats, each side's text is classified by its own grammar (*inferred*).
- [ ] The element selection follows the Session Settings scope dropdown like every other importance rule.

#### Tasks
- **E10-S5-T1** — List the session format's grammar elements on the Importance tab · size: S · depends: —
  - AC: for a C-family file, the tab lists that grammar's elements (for example Comment, String, Keyword) below the built-ins. Changing the format changes the list.
- **E10-S5-T2** — Classify differences inside unticked elements as unimportant · size: M · depends: E10-S5-T1
  - AC: a pair that differs only inside a `//` comment shows blue while Comment is unticked and red while it is ticked.
- **E10-S5-T3** — Element case sensitivity taken from the grammar · size: S · depends: E10-S5-T2
  - AC: a case-only change inside a case-insensitive Keyword element is unimportant even when Everything else is case sensitive.

---
