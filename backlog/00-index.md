# OverTheDiff — product backlog

**Source**: the OverTheDiff feature catalogue, a behaviour-level specification of the product. F-IDs, Q-IDs and § numbers refer to it.
**Mode**: spec-only (greenfield; no code exists yet). Every story is spec-derived and not yet validated. *inferred* marks an analyst or PO inference, or a detail read only from a figure.
**Scope**: OverTheDiff, a desktop comparison tool for Windows, macOS and Linux, covering 127 of the 130 catalogue features (F-5.1 … F-28.5). Licensing, admin policies, updates and support (F-29.1–F-29.3) are out of scope: the product is fully open source.
**Size**: 28 epics · 175 stories (41 Must · 92 Should · 42 Could) · 734 tasks. Companion: [`PRD.md`](PRD.md).

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

## Epics (dependency order)

### Foundation
- **E27 — Repository, CI and test harness** ([`epic-27-repository-ci-and-test-harness.md`](epic-27-repository-ci-and-test-harness.md)) — The setup story: a Cargo workspace with UI-free engine crates, CI through the shared workflows, and the UI test harness. · size: M · depends: — · 1 story (1 Must / 0 Should / 0 Could), 3 tasks, setup (before layer 0)
- **E01 — Application platform** ([`epic-01-application-platform.md`](epic-01-application-platform.md)) — The desktop frame every feature plugs into: single-instance launch, windows and tabs, commands, dialogs, settings storage, background tasks, typed commands that run without the window, and translated text. · size: L · depends: — · 2 stories (2 Must / 0 Should / 0 Could), 19 tasks, layers 0–1
- **E02 — Pattern and file-type services** ([`epic-02-pattern-and-file-type-services.md`](epic-02-pattern-and-file-type-services.md)) — Shared matching engines: file masks, regular expressions and the ordered file-format registry that decides how each file is handled. · size: M · depends: — · 1 story (1 Must / 0 Should / 0 Could), 7 tasks, layer 0
- **E03 — Comparison engines** ([`epic-03-comparison-engines.md`](epic-03-comparison-engines.md)) — Headless engines that decide what differs: line and character diff for text; scan, align and test for folder trees. · size: L · depends: — · 2 stories (2 Must / 0 Should / 0 Could), 15 tasks, layer 0
- **E04 — Data sources** ([`epic-04-data-sources.md`](epic-04-data-sources.md)) — Where inputs come from: local and network file systems first, then archives, web pages and devices, all behind one contract so every view treats them alike. · size: L · depends: E07, E08 · 6 stories (2 Must / 2 Should / 2 Could), 24 tasks, layers 0–4

### Features
- **E05 — App shell, themes and windows** ([`epic-05-app-shell-themes-and-windows.md`](epic-05-app-shell-themes-and-windows.md)) — How the application looks and how its windows behave: themes, multiple windows, drag and drop, toolbar presentation. · size: M · depends: E01, E07, E08, E22 · 4 stories (1 Must / 3 Should / 0 Could), 18 tasks, layers 1–3
- **E06 — Sessions and workspaces** ([`epic-06-sessions-and-workspaces.md`](epic-06-sessions-and-workspaces.md)) — Comparisons as reusable sessions: saving, defaults, organisation, workspaces, and how folder sessions hand file pairs to child sessions. · size: L · depends: E01, E05, E07, E08, E09, E15, E16, E17 · 11 stories (2 Must / 9 Should / 0 Could), 43 tasks, layers 2–3
- **E07 — Text Compare** ([`epic-07-text-compare.md`](epic-07-text-compare.md)) — Side-by-side comparison and editing of text files: the core of the product. · size: L · depends: E01, E02, E03, E04, E08, E10, E15, E16, E17 · 22 stories (9 Must / 10 Should / 3 Could), 87 tasks, layers 1–3
- **E08 — Folder Compare** ([`epic-08-folder-compare.md`](epic-08-folder-compare.md)) — Side-by-side comparison of folder trees: display, criteria, filters, selection, navigation and drilling down into file pairs. · size: L · depends: E01, E02, E03, E04, E06, E07, E09, E11, E14 · 22 stories (12 Must / 9 Should / 1 Could), 87 tasks, layers 1–4
- **E09 — Folder file operations** ([`epic-09-folder-file-operations.md`](epic-09-folder-file-operations.md)) — Reconciling folders: copying, moving, deleting, renaming and synchronising items from the folder view, with confirmations, backups and a log. · size: L · depends: E02, E04, E07, E08 · 12 stories (4 Must / 7 Should / 1 Could), 44 tasks, layers 2–3
- **E10 — File formats and grammars** ([`epic-10-file-formats-and-grammars.md`](epic-10-file-formats-and-grammars.md)) — Global file-format definitions: which view a file opens in, its grammar for highlighting and importance, conversions and text behaviour. · size: L · depends: E02, E05, E07, E08, E11, E12, E13, E14 · 9 stories (3 Must / 3 Should / 3 Could), 36 tasks, layers 2–4
- **E11 — Command line and VCS integration** ([`epic-11-command-line-and-vcs-integration.md`](epic-11-command-line-and-vcs-integration.md)) — Launching comparisons from scripts and version-control tools: waiting launchers, switches and exit codes. · size: L · depends: E01, E03, E04, E07, E08, E13 · 5 stories (2 Must / 3 Should / 0 Could), 16 tasks, layers 1–3
- **E12 — Folder Sync** ([`epic-12-folder-sync.md`](epic-12-folder-sync.md)) — A dedicated session that plans and runs copies and deletes to synchronise two folders, saved for reuse. · size: L · depends: E04, E08, E09, E21 · 4 stories (0 Must / 2 Should / 2 Could), 19 tasks, layers 3–5
- **E13 — Text Merge** ([`epic-13-text-merge.md`](epic-13-text-merge.md)) — 3-way merge of text files into an output file, with automatic merging and manual conflict resolution. · size: L · depends: E03, E06, E07, E11, E14 · 5 stories (0 Must / 5 Should / 0 Could), 24 tasks, layers 2–4
- **E14 — Folder Merge** ([`epic-14-folder-merge.md`](epic-14-folder-merge.md)) — 3-way merge of folder trees into an output folder. · size: L · depends: E08, E09, E11, E13, E22 · 2 stories (0 Must / 2 Should / 0 Could), 13 tasks, layers 3–4
- **E15 — Table Compare** ([`epic-15-table-compare.md`](epic-15-table-compare.md)) — Cell-by-cell comparison of tabular data: CSV and delimited text, fixed width, Excel and HTML tables. · size: L · depends: E01, E02, E03, E04, E07, E10 · 8 stories (0 Must / 8 Should / 0 Could), 38 tasks, layers 1–4
- **E16 — Hex Compare** ([`epic-16-hex-compare.md`](epic-16-hex-compare.md)) — Byte-level comparison and editing of binary files. · size: M · depends: E01, E02, E04 · 3 stories (0 Must / 3 Should / 0 Could), 16 tasks, layers 1–3
- **E17 — Picture Compare** ([`epic-17-picture-compare.md`](epic-17-picture-compare.md)) — Visual comparison of images with a difference pane. · size: M · depends: E01, E02, E04 · 5 stories (0 Must / 3 Should / 2 Could), 18 tasks, layers 1–2
- **E18 — Reports** ([`epic-18-reports.md`](epic-18-reports.md)) — Saving comparison results to read later: HTML, plain text, patch and print output. · size: L · depends: E06, E07, E08, E15, E16, E17, E20, E25 · 6 stories (0 Must / 4 Should / 2 Could), 25 tasks, layers 3–5
- **E19 — Snapshots** ([`epic-19-snapshots.md`](epic-19-snapshots.md)) — Point-in-time listings of folder trees that can be compared later like a folder. · size: L · depends: E04, E05, E08, E20 · 1 story (0 Must / 1 Should / 0 Could), 7 tasks, layer 3
- **E20 — Scripting** ([`epic-20-scripting.md`](epic-20-scripting.md)) — Headless automation through script files over an invisible folder comparison. · size: L · depends: E01, E02, E03, E04, E06, E08, E18, E19, E21 · 6 stories (0 Must / 5 Should / 1 Could), 21 tasks, layers 1–5
- **E21 — Remote services and profiles** ([`epic-21-remote-services-and-profiles.md`](epic-21-remote-services-and-profiles.md)) — Comparing against FTP, SFTP, cloud storage and repositories through named profiles. · size: L · depends: E04, E07, E08, E09, E15, E16 · 9 stories (0 Must / 5 Should / 4 Could), 34 tasks, layers 2–4
- **E22 — Preferences and customisation** ([`epic-22-preferences-and-customisation.md`](epic-22-preferences-and-customisation.md)) — Tailoring commands, colours, fonts, external tools and interface details. · size: L · depends: E01, E07, E08, E10, E13, E15, E16, E17 · 6 stories (0 Must / 4 Should / 2 Could), 29 tasks, layers 1–3
- **E23 — Settings portability** ([`epic-23-settings-portability.md`](epic-23-settings-portability.md)) — Moving, sharing and resetting settings. · size: L · depends: E01, E04, E05, E06, E10, E21, E22 · 4 stories (0 Must / 2 Should / 2 Could), 15 tasks, layers 2–4
- **E24 — Text Edit and Text Patch** ([`epic-24-text-edit-and-text-patch.md`](epic-24-text-edit-and-text-patch.md)) — A single-pane editor and a patch-file viewer built on the text components. · size: L · depends: E02, E03, E07, E08, E10, E15, E21, E26 · 6 stories (0 Must / 0 Should / 6 Could), 20 tasks, layers 3–4
- **E25 — Media, Version and Registry Compare** ([`epic-25-media-version-and-registry-compare.md`](epic-25-media-version-and-registry-compare.md)) — Specialised comparisons: audio tags, executable version resources and the Windows registry. · size: L · depends: E01, E02, E04, E08, E09, E18, E19, E20, E21, E22 · 6 stories (0 Must / 0 Should / 6 Could), 30 tasks, layers 1–5
- **E26 — OS integrations** ([`epic-26-os-integrations.md`](epic-26-os-integrations.md)) — Reaching the product from the operating system: file-manager menus, clipboard history and source-control providers. · size: M · depends: E01, E07, E08, E12, E13, E14, E15, E16, E17, E18, E24, E25 · 5 stories (0 Must / 0 Should / 5 Could), 23 tasks, layers 2–5
- **E28 — Automation and agent access** ([`epic-28-automation-and-agent-access.md`](epic-28-automation-and-agent-access.md)) — Every command without the window: a command-line automation tool, and MCP for agents limited to given folders. · size: M · depends: E01 · 2 stories (0 Must / 2 Should / 0 Could), 5 tasks, layers 2–3

## Release plan

Each release is built layer by layer. Within a layer, every story in that release can run in parallel. A story can start as soon as everything in its *Needs* has shipped; it doesn't have to wait for the whole previous layer.

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

### Release 2 — Power-user features (all Should stories) — 92 stories

**Layer 1** (6 stories, buildable in parallel)

| Story | Title | Size | Platforms | Needs |
|---|---|---|---|---|
| E05-S2 | Multiple windows and tab behaviour | M | All | E01-S1 |
| E15-S1 | Table Compare view | L | All | E01-S1, E02-S1, E03-S1, E04-S1 |
| E16-S1 | Hex Compare view | L | All | E01-S1, E02-S1, E04-S1 |
| E17-S1 | Picture Compare view | L | All | E01-S1, E02-S1, E04-S1 |
| E20-S1 | Script runner and core commands | L | All | E01-S1, E03-S2, E04-S1 |
| E22-S1 | Customise menus, toolbars and shortcuts | L | All | E01-S1 |

**Layer 2** (31 stories, buildable in parallel)

| Story | Title | Size | Platforms | Needs |
|---|---|---|---|---|
| E04-S4 | Web pages as file inputs | S | All | E04-S1, E07-S1, E08-S1 |
| E05-S3 | Drag and drop to start comparisons | S | All | E07-S1, E08-S1 |
| E06-S9 | Switch view type (Compare Files Using) | S | All | E07-S1, E16-S1 |
| E06-S10 | Compare parent folders from file views | S | All | E07-S1, E08-S1, E15-S1, E16-S1, E17-S1 |
| E07-S8 | Editing commands and file conversions | M | All | E07-S1 |
| E07-S11 | Thumbnail overview | M | All | E07-S1 |
| E07-S12 | Details panels and ruler | M | All | E07-S1 |
| E07-S13 | Alignment algorithms and options | M | All | E07-S1 |
| E07-S14 | Manual alignment | M | All | E07-S1 |
| E07-S18 | Clipboard comparisons | S | All | E07-S1 |
| E07-S22 | Detect files changed on disk | M | All | E07-S1, E15-S1, E16-S1, E17-S1 |
| E08-S12 | Other filters | M | All | E08-S1, E02-S1 |
| E08-S20 | Automatic refresh | S | All | E08-S1 |
| E08-S22 | Alignment overrides and manual alignment | M | All | E08-S1, E02-S1 |
| E09-S6 | Rename one or many items | M | All | E08-S1, E02-S1 |
| E09-S7 | Change attributes and permissions | M | All | E08-S1 |
| E09-S8 | Touch timestamps | S | All | E08-S1 |
| E11-S4 | Compare standard input | S | All | E07-S1 |
| E13-S1 | Text Merge view with automatic merge | L | All | E03-S1, E07-S1 |
| E15-S2 | Key columns for row alignment | M | All | E15-S1 |
| E15-S4 | Multiple sheets: Excel workbooks and HTML tables | L | All | E15-S1 |
| E15-S5 | Edit and save tables | M | All | E15-S1 |
| E16-S2 | Hex editing | M | All | E16-S1 |
| E17-S2 | Mismatch-range and blend modes | M | All | E17-S1 |
| E17-S3 | Geometric alignment | M | All | E17-S1 |
| E20-S3 | Script filters | M | All | E20-S1, E02-S1 |
| E20-S4 | Script sync | S | All | E20-S1 |
| E21-S1 | FTP sites and the Profiles manager | L | All | E04-S1, E08-S1 |
| E22-S2 | Folder view colours and fonts | M | All | E08-S1 |
| E22-S4 | Open With applications | M | All | E08-S1, E07-S1, E15-S1, E16-S1, E17-S1 |
| E28-S1 | Every command runs without the window | M | All | E01-S1, E01-S2 |

**Layer 3** (46 stories, buildable in parallel)

| Story | Title | Size | Platforms | Needs |
|---|---|---|---|---|
| E04-S3 | More archive formats and the Archive Types page | M | All | E04-S2, E04-S1 |
| E05-S4 | Display-filter toolbar layouts | M | All | E07-S3, E08-S2, E22-S1 |
| E06-S3 | Session folders and search | M | All | E06-S1 |
| E06-S4 | Lock sessions and remember display filters | S | All | E06-S1, E07-S3, E08-S2 |
| E06-S5 | Session management panel and editing defaults from Home | M | All | E06-S1, E06-S2 |
| E06-S6 | Workspaces | L | All | E06-S1, E05-S2 |
| E06-S7 | Child-session settings scope and placement | M | All | E08-S10, E05-S2, E06-S1 |
| E06-S8 | Disable editing and read-only child sessions | S | All | E07-S1, E08-S10, E09-S1, E09-S2, E09-S3, E09-S4, E09-S6, E09-S7, E09-S8 |
| E06-S11 | Child-session commands in Table, Hex and Picture views | M | All | E08-S10, E09-S1, E15-S1, E16-S1, E17-S1 |
| E07-S15 | Unimportance list | M | All | E07-S4, E02-S1 |
| E07-S16 | Text replacements | M | All | E07-S4, E07-S2, E02-S1 |
| E07-S19 | Next Difference options | M | All | E07-S2, E07-S7, E08-S10 |
| E08-S14 | Name filter presets | S | All | E08-S3 |
| E08-S15 | Suppress filters | S | All | E08-S3, E09-S1, E09-S2, E09-S3, E09-S4, E08-S2 |
| E08-S16 | Open With an associated application or a chosen view | S | All | E08-S10 |
| E08-S17 | Windows file details | M | Win | E08-S4, E08-S6, E08-S12, E09-S1, E07-S1 |
| E08-S18 | Copy a file across and open the next difference | S | All | E08-S10, E09-S1 |
| E09-S5 | Copy or move to a folder or archive | M | All | E08-S1, E04-S2 |
| E09-S9 | New folders and missing base folders | S | All | E08-S1, E04-S2 |
| E09-S10 | Backups before overwriting | M | All | E09-S1, E09-S2, E07-S1 |
| E09-S11 | Operation log panel | M | All | E09-S1, E09-S2, E09-S3 |
| E10-S3 | Grammar editor and line weights | L | All | E10-S1, E10-S2, E07-S4 |
| E10-S4 | Text format behaviour: tabs, independent lines and column data | M | All | E10-S1, E07-S2, E07-S7, E07-S8 |
| E10-S6 | Conversion through external programs | L | All | E10-S1, E07-S1 |
| E11-S3 | Folder comparison switches | S | All | E08-S3 |
| E11-S5 | Headless automatic merge for version control | M | All | E13-S1, E11-S1 |
| E12-S1 | Folder Sync session: set up, preview and run | L | All | E08-S3, E08-S4, E08-S10, E08-S11, E08-S12, E04-S2, E08-S6, E08-S7, E08-S8, E08-S9 |
| E13-S2 | Conflict definition and conflict flags | M | All | E13-S1 |
| E13-S3 | Merge navigation, search and line-level takes | M | All | E13-S1, E07-S5, E07-S6 |
| E13-S4 | Merge display filters and view options | M | All | E13-S1, E07-S3, E07-S9, E07-S11, E07-S12 |
| E14-S1 | Folder Merge view with 3-way status | L | All | E08-S2, E08-S3, E08-S4, E08-S5, E08-S11, E08-S12, E22-S2 |
| E15-S3 | Column alignment and per-column handling | M | All | E15-S1, E15-S2 |
| E15-S6 | Table search, navigation and view options | M | All | E15-S1, E02-S1, E15-S5 |
| E15-S8 | Row alignment options (Rows tab) | M | All | E15-S1, E15-S2, E07-S13 |
| E16-S3 | Hex search, value inspection and layout | M | All | E16-S1, E16-S2 |
| E18-S1 | Text Compare report | L | All | E07-S3, E07-S4, E20-S1, E06-S1 |
| E19-S1 | Save and compare folder snapshots | L | All | E08-S3, E04-S2, E20-S1, E05-S3, E08-S5 |
| E20-S2 | Script file operations | M | All | E20-S1, E04-S2, E02-S1 |
| E21-S3 | SFTP and FTPS with key and certificate authentication | L | All | E21-S1 |
| E21-S4 | Amazon S3 profiles | M | All | E21-S1 |
| E21-S5 | WebDAV profiles | M | All | E21-S1 |
| E21-S6 | Browse using profiles and Quick Connect | M | All | E21-S1, E07-S1, E15-S5, E16-S2 |
| E22-S3 | File view colours and fonts | M | All | E07-S4, E10-S2, E13-S1, E16-S1, E15-S1, E17-S1, E07-S12 |
| E23-S1 | Export and import settings packages | L | All | E06-S1, E10-S1, E21-S1, E22-S1, E22-S2, E04-S2, E05-S1 |
| E23-S2 | Restore factory defaults | M | All | E06-S1, E10-S1, E21-S1, E22-S1, E22-S2, E05-S1 |
| E28-S2 | Agents work through MCP, limited to given folders | M | All | E28-S1 |

**Layer 4** (8 stories, buildable in parallel)

| Story | Title | Size | Platforms | Needs |
|---|---|---|---|---|
| E08-S19 | Quick Compare dialog | M | All | E08-S10, E08-S13 |
| E12-S2 | Per-item overrides and sync review commands | M | All | E12-S1, E08-S13, E08-S15, E08-S16, E09-S11 |
| E13-S5 | Merge alignment, settings and session commands | M | All | E13-S1, E07-S4, E07-S13, E07-S14, E14-S1, E07-S7, E07-S8, E07-S10, E07-S18, E13-S4, E11-S5, E06-S2 |
| E14-S2 | Folder Merge actions | L | All | E14-S1, E09-S1, E09-S3, E09-S6, E09-S7, E09-S8, E09-S9, E13-S1, E08-S13, E08-S10, E08-S3, E08-S7, E11-S1, E11-S5 |
| E15-S7 | Table format definitions | M | All | E15-S1, E10-S1, E15-S3 |
| E18-S2 | Folder Compare and file-list reports | L | All | E18-S1, E08-S2, E08-S6, E20-S1, E06-S1, E20-S3 |
| E18-S3 | Print reports | M | All | E18-S1 |
| E18-S4 | Table, Hex and Picture reports | M | All | E18-S1, E15-S1, E16-S1, E17-S1, E20-S1, E06-S1 |

**Layer 5** (1 stories, buildable in parallel)

| Story | Title | Size | Platforms | Needs |
|---|---|---|---|---|
| E20-S5 | Script sessions, full criteria and sample scripts | M | All | E20-S2, E20-S3, E20-S4, E06-S1, E06-S2, E08-S4, E08-S13, E08-S17, E19-S1, E21-S1, E20-S1, E18-S2, E06-S4 |

### Release 3 — Full feature set (all Could stories) — 42 stories

**Layer 1** (3 stories, buildable in parallel)

| Story | Title | Size | Platforms | Needs |
|---|---|---|---|---|
| E25-S1 | Media Compare | L | All | E01-S1, E04-S1, E02-S1 |
| E25-S2 | Version Compare view | L | Win | E01-S1, E04-S1, E02-S1 |
| E25-S4 | Registry Compare | L | Win | E01-S1, E04-S1 |

**Layer 2** (10 stories, buildable in parallel)

| Story | Title | Size | Platforms | Needs |
|---|---|---|---|---|
| E04-S5 | Portable devices (MTP) on Windows | M | Win | E04-S1, E08-S1 |
| E07-S17 | Ignored lines | S | All | E07-S1 |
| E07-S21 | Webpages panel | M | Win, Mac | E07-S1 |
| E09-S12 | Exchange sides | S | All | E08-S1, E04-S1 |
| E17-S4 | Colour replacements | S | All | E17-S1 |
| E17-S5 | Extra image formats | M | All | E17-S1 |
| E20-S6 | Scheduled scripts | S | Win | E20-S1 |
| E22-S5 | Interface tweaks | M | All | E07-S1, E08-S1 |
| E23-S4 | Portable installation on Windows | M | Win | E01-S2 |
| E26-S1 | File-manager context menu | L | Win | E07-S1, E08-S1, E01-S2 |

**Layer 3** (8 stories, buildable in parallel)

| Story | Title | Size | Platforms | Needs |
|---|---|---|---|---|
| E07-S20 | Editor display tweaks | M | All | E07-S7, E10-S2, E07-S9 |
| E21-S7 | Dropbox profiles | M | All | E21-S1 |
| E21-S8 | OneDrive profiles | M | All | E21-S1 |
| E22-S6 | Comparison performance settings | S | All | E08-S5 |
| E24-S1 | Text Edit view | L | All | E07-S5, E07-S6, E07-S8, E07-S9, E10-S2 |
| E24-S2 | Text Patch view | L | All | E07-S1, E03-S1, E07-S2, E07-S5, E07-S6, E07-S7, E07-S9, E07-S11, E07-S12, E10-S2 |
| E25-S6 | Media and Version views as child sessions | S | All | E25-S1, E25-S2, E08-S10, E09-S1 |
| E26-S5 | Native shell menu inside views | S | Win | E07-S1, E08-S1, E13-S1, E15-S1, E16-S1, E17-S1, E25-S1, E25-S2, E25-S4 |

**Layer 4** (17 stories, buildable in parallel)

| Story | Title | Size | Platforms | Needs |
|---|---|---|---|---|
| E04-S6 | Archive plug-ins on Windows | M | Win | E04-S3 |
| E08-S21 | Ignored items | S | All | E08-S1, E14-S1 |
| E10-S7 | Built-in document converters | M | All | E10-S6, E10-S1 |
| E10-S8 | Per-session file-format enablement | S | All | E10-S1, E12-S1, E14-S1 |
| E10-S9 | External formats | M | All | E10-S1, E08-S13, E13-S1, E11-S5 |
| E12-S3 | Custom sync operations | M | All | E12-S1 |
| E18-S6 | Media, Version and Registry reports | M | All | E18-S1, E25-S1, E25-S2, E25-S4, E20-S1 |
| E21-S2 | Advanced FTP settings | M | All | E21-S1, E09-S11, E21-S3 |
| E21-S9 | Subversion repositories | M | All | E21-S1, E21-S6 |
| E23-S3 | Shared sessions | M | All | E23-S1, E06-S1 |
| E24-S3 | Find in Files | M | All | E24-S1, E02-S1 |
| E24-S4 | Apply patches | M | All | E24-S2 |
| E24-S5 | Open With Text Edit and Text Patch | S | All | E24-S1, E24-S2, E08-S16, E15-S1 |
| E24-S6 | Text Edit extras: tab stops, profiles, shell menu and Webpages panel | S | All | E24-S1, E10-S4, E21-S6, E26-S5, E07-S21 |
| E25-S5 | Registry editing and export | M | Win | E25-S4, E21-S6, E22-S4 |
| E26-S2 | More context-menu commands | M | Win | E26-S1, E12-S1, E13-S1, E14-S1, E07-S18, E16-S1, E25-S4, E24-S1, E24-S2 |
| E26-S3 | Clipboard Compare tray utility | M | Win | E07-S1, E24-S1 |

**Layer 5** (4 stories, buildable in parallel)

| Story | Title | Size | Platforms | Needs |
|---|---|---|---|---|
| E12-S4 | Advanced sync handling | S | All | E12-S1, E08-S17, E08-S20, E08-S22, E21-S2 |
| E18-S5 | XML report layouts | M | All | E18-S1, E18-S2 |
| E25-S3 | Version information in folders, snapshots and scripts | M | Win | E25-S2, E08-S4, E08-S6, E19-S1, E20-S1, E18-S2 |
| E26-S4 | Source control integration | L | Win | E08-S6, E08-S12, E07-S1, E18-S2, E24-S1 |


## Hub surfaces (one owner, extenders strictly later)

| Surface | Owner (layer) | Extenders (layers) |
|---|---|---|
| Options pages, Session Settings tabs (several contributors per tab), command registry and menus/toolbars (with macOS application-menu placement), colour roles, argument handlers with launch modes (forward-and-exit, run-in-launching-process, forward-and-wait; requested modes, hand-off, attached stdin, switch precedence), Home launcher, progress/cancel/error summary, path edit, unsaved-changes prompts, commands runnable by id with typed arguments, translation lookup | E01-S1 (0) | any story (contribution points) |
| Headless `/qc` quick compare | E11-S2 (1) | E08-S13 (3) |
| `otd-cli` automation tool | E28-S1 (2) | E28-S2 (3) |
| Data-source contract (incl. expandable containers) | E04-S1 (0) | E04-S2 (2), E04-S3 (3), E04-S4 (2), E04-S5 (2), E21-S1 (2), E08-S1 (1), E07-S1 (1) |
| Zip-family archive masks | E04-S2 (2) | E23-S1 (3) |
| Merge display filters and toggles | E13-S4 (3) | E13-S5 (4) |
| Headless automatic-merge switches | E11-S5 (3) | E13-S5 (4), E10-S9 (4), E14-S2 (4) |
| Key columns | E15-S2 (2) | E15-S3 (3), E15-S8 (3) |
| Home sessions tree (Saved and Auto-saved branches) | E06-S1 (2) | E06-S3 (3), E06-S4 (3), E06-S5 (3), E23-S3 (4) |
| Session Settings scope dropdown beyond "this view only" | E06-S2 (2) | E06-S5 (3) |
| Child-session mechanism (open pairs from folder views) | E08-S10 (2) | E06-S7 (3), E06-S8 (3), E06-S11 (3), E07-S19 (3), E08-S16 (3), E08-S18 (3), E08-S19 (4), E14-S2 (4), E25-S6 (3) |
| File Formats manager and format editor tabs | E10-S1 (2) | E10-S3 (3), E10-S4 (3), E10-S6 (3), E10-S7 (4), E10-S8 (4), E10-S9 (4), E15-S7 (4) |
| Name Filters tab | E08-S3 (2) | E08-S14 (3), E11-S3 (3), E14-S2 (4) |
| Other Filters tab | E08-S12 (2) | E08-S17 (3), E26-S4 (5) |
| Profiles dialog | E21-S1 (2) | E21-S2 (4), E21-S3 (3), E21-S4 (3), E21-S5 (3), E21-S6 (3), E21-S7 (3), E21-S8 (3), E21-S9 (4) |
| Browse Using Profile / profile-based open and save | E21-S6 (3) | E21-S9 (4), E24-S6 (4), E25-S5 (4) |
| Report dialog and output targets | E18-S1 (3) | E18-S2 (4), E18-S3 (4), E18-S4 (4), E18-S5 (5), E18-S6 (4) |
| Folder Compare report columns | E18-S2 (4) | E18-S5 (5), E20-S5 (5), E25-S3 (5), E26-S4 (5) |
| Operation log panel | E09-S11 (3) | E12-S2 (4), E21-S2 (4) |
| Script runner | E20-S1 (1) | E20-S2 (3), E20-S3 (2), E20-S4 (2), E20-S5 (5), E20-S6 (2), E18-S1 (3), E18-S2 (4), E18-S4 (4), E18-S6 (4), E19-S1 (3), E25-S3 (5) |
| Text Merge view | E13-S1 (2) | E13-S2 (3), E13-S3 (3), E13-S4 (3), E13-S5 (4), E11-S5 (3), E14-S2 (4), E10-S9 (4), E22-S3 (3), E26-S2 (4), E26-S5 (3) |
| Folder Sync view | E12-S1 (3) | E12-S2 (4), E12-S3 (4), E12-S4 (5), E10-S8 (4) |
| Folder Merge view | E14-S1 (3) | E14-S2 (4), E13-S5 (4), E08-S21 (4), E10-S8 (4) |
| Folder view colours page | E22-S2 (2) | E14-S1 (3), E23-S1 (3) |
| Customise menus/toolbars/shortcuts dialog | E22-S1 (1) | E05-S4 (3), E23-S1 (3) |
| Open With applications | E22-S4 (2) | E25-S5 (4) |
| Native shell menu in views | E26-S5 (3) | E24-S6 (4) |
| Table Compare view | E15-S1 (1) | E15-S2 (2), E15-S3 (3), E15-S4 (2), E15-S5 (2), E15-S6 (3), E15-S7 (4), E15-S8 (3), E06-S10 (2), E06-S11 (3), E07-S22 (2), E18-S4 (4), E22-S3 (3), E22-S4 (2), E26-S5 (3) |
| Table editing and saving | E15-S5 (2) | E15-S6 (3), E21-S6 (3) |
| Hex Compare view | E16-S1 (1) | E16-S2 (2), E16-S3 (3), E06-S9 (2), E06-S10 (2), E06-S11 (3), E07-S22 (2), E18-S4 (4), E22-S3 (3), E22-S4 (2), E26-S2 (4), E26-S5 (3) |
| Hex editing and saving | E16-S2 (2) | E16-S3 (3), E21-S6 (3) |
| Picture Compare view | E17-S1 (1) | E17-S2 (2), E17-S3 (2), E17-S4 (2), E17-S5 (2), E06-S10 (2), E06-S11 (3), E07-S22 (2), E18-S4 (4), E22-S3 (3), E22-S4 (2), E26-S5 (3) |
| Text Edit view | E24-S1 (3) | E24-S3 (4), E24-S5 (4), E24-S6 (4), E26-S2 (4), E26-S3 (4), E26-S4 (5) |
| Text Patch view | E24-S2 (3) | E24-S4 (4), E24-S5 (4), E26-S2 (4) |

## Dependency notes

- **Setup** comes first: E27-S1 makes the workspace, CI and test harness, alone, so the layer-0 foundations can then run in parallel without colliding.
- **Layer 0** has five foundation stories that share no code dependency: the shell (E01-S1), pattern services (E02-S1), the text engine (E03-S1), the folder engine (E03-S2) and the file-system layer (E04-S1). E03-S2 is built against the data-source contract written in E04, using an in-memory source, so it doesn't wait for E04-S1.
- **Layer 1** is the walking skeleton: Text Compare (E07-S1), Folder Compare (E08-S1), installers (E01-S2), themes (E05-S1) and headless `/qc` (E11-S2). Standalone viewers that need only foundations (Table, Hex, Picture, Media, Version, Registry), the script runner and the customise dialog also sit here, by priority, in later releases.
- **The MVP critical path** is three hops deep. Text: E03-S1 → E07-S1 → E07-S4 → {E08-S13 rules-based folder comparison, E10-S5 grammar-based importance}. Folder: E03-S2 → E08-S1 → E08-S5 → E08-S13. Everything else in the MVP is layer 2 or lower.
- **Release 2 and 3 depth** comes from views built on views: Folder Sync and Folder Merge reuse the Folder Compare settings tabs, reports reuse the script runner, and Text Merge actions reuse Folder Merge.
- **Engines are reused, not re-implemented.** Table rows, Text Merge, Text Patch and rules-based folder comparison all use the E03-S1 text engine. Sync, merge, snapshots and scripts all use the E03-S2 folder engine.

## Open questions in the catalogue

No code exists yet, so there is no as-built divergence. What remains open is **what the catalogue leaves for the product to decide**. The table maps every open question in catalogue §30 to its owning story and the proposed default recorded there.

| Question | Owning story | Proposed default / status |
|---|---|---|
| **Q1** Folder Sync custom operations: the per-status matrix (copy, delete, leave alone) isn't specified. | E12-S3 | E12-S3: Rows are statuses: left newer, right newer, different with no newer side, left orphan, right orphan, same; Columns are actions: copy left → right, copy right → left, delete left, delete right, leave alone; Each status gets exactly one action, and the five presets are expressed in this matrix |
| **Q2** Backups: naming, location options and retention aren't specified. | E09-S10 | E09-S10: a timestamped copy in a per-user backups folder that mirrors the original path, keeping the last 5 versions per file; `<name>.bak` next to the original is the alternative |
| **Q3** Confirmations: the full list of confirmation options isn't specified. | E09-S1, E09-S2, E09-S3, E09-S4 | E09-S1: one "Confirm …" option per operation kind on the File Operations page (E09 behavioral contract); each operation story adds its own; E09-S2: same rule; E09-S3: same rule; E09-S4: same rule, plus "Synchronize confirmations" (Prompt / Yes to All / No to All) |
| **Q4** Media and Version Compare importance: each field's default importance and the Importance tab layout aren't specified. | E25-S1, E25-S2 | E25-S1: every Metadata text field and the Audio data size are important; Private frames, Unique file identifier and padding fields are unimportant.; E25-S2: every field is important except PE Header Creation Date and Checksum. |
| **Q5** Table column types: only General and Text are named; number, date and currency types are implied by the tolerances. | E15-S3 | E15-S3: General (auto-detect), Text, Number and Date/time, with Currency parsed as Number. General detects numbers and dates cell by cell. |
| **Q6** Hex Compare Layout dialog: bytes per row, grouping and character encoding aren't specified. | E16-S3 | E16-S3: bytes per row: 8, 16 (default) or 32; byte grouping: 1, 2, 4 or 8; character panel encoding: ASCII, system code page or UTF-8; address format: hex (default) or decimal |
| **Q7** Unimportance list: the item editor (regular expressions? per side?) isn't specified. | E07-S15 | E07-S15: each item is text or a regular expression, with Match character case and Whole words only flags, and applies to both sides. |
| **Q8** Built-in file formats: the languages, grammars and converters (Word, Excel, PDF) aren't listed. | E10-S2, E10-S7 | E10-S2: C/C++, C#, Java, JavaScript/TypeScript, Python, Go, Rust; HTML/XML, CSS; JSON, YAML, SQL, shell scripts, Markdown, INI/properties; a plain Text format; E10-S7: Word (`.docx`, plus `.doc` where feasible) and PDF to text. Other document types go through external programs. |
| **Q9** Report XML schema for folder and text reports isn't specified. | E18-S5 | E18-S5: a documented, versioned schema of the product's own, published as a schema file. |
| **Q10** Snapshot file format isn't specified. | E19-S1 | E19-S1: the product's own documented, compressed format with a version header and its own extension. Importing snapshot files made by third-party tools stays a separate, optional roadmap item. |
| **Q11** Settings, session and workspace file formats and the settings package aren't specified; decide whether third-party settings formats import. | E23-S1 | E23-S1: The package is a zip with a manifest (format version, categories, items) and one file per item; Every package records its format version. Importing older format versions is added once a second format version exists; Third-party settings formats are not imported; Saved passwords are left out unless the user ticks an explicit option |
| **Q12** Source control: legacy SCC/MSSCCI is specified; decide whether to offer modern VCS integration (Git status column, blame) instead. | E26-S4 | E26-S4: MSSCCI support on Windows, as specified. Git status and revision columns, plus diff and history commands on all platforms, go to the Roadmap appendix as the recommended successor. |
| **Q13** Shell integration is specified for Windows only; decide whether to add Finder or Nautilus integration. | E26-S1 | E26-S1: Windows only, as specified. Finder and Nautilus/Dolphin extensions go to the Roadmap appendix. |
| **Q14** Clipboard Compare is specified for Windows only; decide whether to make it cross-platform. | E26-S3 | E26-S3: Windows only, as specified. A cross-platform version goes to the Roadmap appendix. |
| **Q15** Scheduler: rely on OS schedulers, or build one in? | E20-S6 | E20-S6: no built-in scheduler; document Windows Task Scheduler, and add the macOS and Linux equivalents if this story is extended to those platforms. |
| **Q17** Exit code 12 ("similar"): when is it returned rather than 13 (rules-based differences)? | E08-S13 | E08-S13: 12 when only unimportant differences remain; 13 when any important difference remains; 2 when only byte differences that the rules ignore (encoding, line endings) remain |
| **Q18** Automatic refresh: the interval and how it is configured aren't specified. | E08-S20 | E08-S20: react to file-system change notifications where the data source supports them, otherwise poll every 5 seconds, with the interval configurable on the Handling tab. |
| **Q19** Registry Compare is Windows-only by nature; confirm it's in scope at all. | E25-S4 | E25-S4: in scope as a Windows-only Could feature, with no comparison rules (Specs tab only), comparing values by type and data. |
| **Q20** Web Resources panel on Home: its content isn't specified. | E06-S5 | E06-S5: not built (non-goal); see PRD |
| **Q21** Picture Compare metadata: which metadata (EXIF?) is compared, and how it is shown. | E17-S2 | E17-S2: EXIF, IPTC and XMP fields, plus DPI and colour profile name. Metadata differences are listed in Picture Compare Info and count towards the overall result while the toggle is on. |
| **Q22** Registry Compare session settings: only a Specs tab is specified; confirm there are no comparison rules. | E25-S4 | E25-S4: in scope as a Windows-only Could feature, with no comparison rules (Specs tab only), comparing values by type and data. |
| **Q23** Apply Patch: target resolution, dry run and reject handling aren't specified. | E24-S4 | E24-S4: targets resolve relative to a chosen base folder, with a count of leading path components to strip, like `patch -p`; a dry-run preview is always shown first; a file with any failed hunk is left untouched and listed in a rejects report |
| **Q24** Default keymap: there is no default shortcut table, and only a few shortcuts are named. | E01-S1 | E01-S1: **decided**, confirmed by the operator on 2026-11-25 (E01-S1-T1): follow each OS's standard shortcuts and keep the product shortcuts named in the catalogue. The story's Decisions list both sets of shortcuts and the reserved chords. |
| **Q25** Status bar: figures show an overall result, an insert/overwrite indicator and the load time; the content per view isn't specified. | E01-S1, E07-S1 | E01-S1: **decided**, confirmed by the operator on 2026-11-25 (E01-S1-T1): views fill shared status-bar fields (overall result, insert/overwrite state, load time) plus fields of their own; E07-S1: **decided**, confirmed by the operator on 2026-11-25 (E07-S1-T1): the status bar shows the overall result ("Same", "Unimportant differences only" or "Important differences"), cursor line and column, insert/overwrite state and load time. Info lists difference sections, changed, inserted and deleted lines, and total lines per side. |

### Further decisions found while writing the backlog

These are behaviours the catalogue leaves open beyond §30. Each one is a decision task inside its story, with a proposed default.

| Story | Decision task |
|---|---|
| E01-S2-T1 | Decide supported OS versions, CPU architectures, package formats and the executable name: **decided**, confirmed by the operator on 2026-11-25 as proposed (E01-S2 Decisions). Windows 10 and 11 on x64 and ARM64 (MSI); a universal macOS build for the two newest major versions; Linux x64 as an AppImage plus a `.deb`. Builds start unsigned, and the executable is `otd`. |
| E02-S1-T7 | Decide and implement the fallback for unmatched files (text vs binary detection): **decided**, confirmed by the operator on 2026-11-25. A NUL or invalid text encoding in the first 8 KB makes a file binary, and an unmatched binary file gets a Hex format when the Hex kind is registered; any other unmatched file gets the built-in catch-all Text format. |
| E03-S1-T6 | Set the text engine's performance budgets for large inputs: **decided**, confirmed by the operator on 2026-11-25 as proposed, with a memory and a per-row budget added (E03-S1 Decisions) |
| E04-S3-T1 | Decide which formats beyond zip are writable |
| E05-S3-T1 | Decide what dropping one item, more than two items, or a file with a folder does |
| E05-S4-T1 | Decide the Favorites buttons for each view |
| E08-S1-T1 | Decide the default timestamp tolerance: **decided**, confirmed by the operator on 2026-11-25, as proposed. Quick tests default to a 2-second tolerance, which absorbs the 2-second timestamps of FAT volumes and zip archives; times exactly 2 seconds apart count as same. |
| E10-S9-T1 | Decide the command-line variables for the four commands |
| E21-S1-T7 | Decide where saved passwords and keys are stored (PO addition; not in the catalogue) |
| E21-S3-T1 | Decide how host keys and server certificates are verified (not covered in the catalogue) |
| E22-S6-T1 | Decide the Comparison priority choices |

## PO priority changes vs the catalogue

The catalogue's P1/P2/P3 are the analyst's suggestions. These are the deliberate changes, all **downgrades** to keep the MVP lean. Otherwise, a story takes the catalogue priority of the part of a feature it covers. Some features bundle parts of different priority, such as F-25.2/F-25.3/F-25.4 (each switch, exit code and positional form follows the feature it belongs to) and F-8.14 (Sync/Merge Base Folders follow Folder Sync and Folder Merge). Those parts are prioritised one by one and aren't listed here.

| Catalogue feature (suggested) | Backlog story (priority) | Reason |
|---|---|---|
| F-5.1 multiple windows, tab moving (P1) | E05-S2 (Should) | One window with tabs covers the MVP workflows, and nothing in the MVP needs a second window. |
| F-5.2 / F-6.1 session folders, search, lock, management panel (P1) | E06-S3, E06-S4, E06-S5 (Should) | Saving, auto-saving and reopening sessions (E06-S1) is the must-have. Organising them is convenience. |
| F-6.1 remembering the display filter per session (P1) | E06-S4 (Should) | Convenience. It also needs display filters and saved sessions, so it would deepen the MVP. |
| F-6.2 child-session scopes (P1) | E06-S7 (Should) | Per-type defaults (E06-S2, Must) cover the main need. |
| F-6.4 Copy File and Open Next Difference; read-only children (P1) | E08-S18, E06-S8 (Should) | Workflow accelerators. Opening pairs and stepping through difference files stay Must (E08-S10). |
| F-8.6 automatic refresh, Windows-only handling (P1/P2) | E08-S20, E08-S17 (Should) | The refresh interval is unspecified (Q18), and the Windows-only metadata options are platform polish. |
| F-8.15 Open With, Quick Compare, native Explorer menu (P1) | E08-S16, E08-S19 (Should); E26-S5 (Could) | Opening pairs in the built-in views is the must-have. |
| F-12.1 thumbnail, details panels, ruler (P1); Webpages panel; editor tweaks | E07-S11, E07-S12 (Should); E07-S21, E07-S20 (Could) | These are display aids. The core display, wrap, whitespace and layouts stay Must (E07-S1, E07-S9). |
| F-12.2 Ignored lines (P1) | E07-S17 (Could) | Rarely used, and importance rules cover the common need. |
| F-12.3 indent, line/word commands, Convert File (P1) | E07-S8 (Should) | Basic editing, save and copy-to-other-side stay Must (E07-S1, E07-S7). |
| F-12.6 unimportance list (P1) | E07-S15 (Should) | Its editor is unspecified (Q7). Whitespace, case, line-ending and grammar rules stay Must. |
| F-12.7 alignment algorithm choices, manual alignment (P1) | E07-S13, E07-S14 (Should) | Standard alignment ships in the MVP engine (E03-S1), and the alternatives are power-user tuning. |
| F-12.8 Compare Files Using (P1) | E06-S9 (Should) | It needs a second file view type, which arrives in Release 2. |
| F-21.4 grammar editor (P1) | E10-S3 (Should) | Built-in grammars, highlighting and grammar-based importance stay Must (E10-S2, E10-S5). |
| F-21.5 text format Misc tab (P1) | E10-S4 (Should) | Tab stops and independent-line handling refine behaviour that already works with defaults. |
| F-25.3 `/filters`, `/expandall`, stdin (P1/P2) | E11-S3, E11-S4 (Should) | Difftool use, `/qc` and exit codes stay Must (E11-S1, E11-S2, E08-S13). |
| F-28.3 merge-tool half of VCS integration (P1) | E11-S5 (Should) | It depends on Text Merge, which the catalogue rates P2. |
| F-7.1 MTP portable devices (part of a P1 feature) | E04-S5 (Could) | Windows-only device access. Local and network paths stay Must (E04-S1). |
| F-7.3 archive plug-ins (P1/P2) | E04-S6 (Could) | Windows-only, and only for third-party formats. Zip stays Must (E04-S2) and the other built-in formats are Should (E04-S3). |
| F-7.4 advanced FTP settings (P2) | E21-S2 (Could) | Basic FTP and profiles are Should (E21-S1). Proxy, listing and transfer tuning is niche. |
| F-16.1 extra codecs, F-16.4 colour replacements (P2) | E17-S5, E17-S4 (Could) | Platform-dependent or niche. Picture Compare itself is Should. |
| F-21.3 built-in document converters (P2) | E10-S7 (Could) | The converter list is unspecified (Q8). External-program conversion is Should (E10-S6). |
| F-22.2 XML layouts; media, version and registry reports (P2) | E18-S5, E18-S6 (Could) | The XML schema is unspecified (Q9), and those reports follow their P3 views. |

## Risks

- **Drawn surfaces and accessibility.** The text, hex, table and picture panes are drawn, not stock widgets, so each one has to expose its text, caret, selection and status to screen readers itself (DoD 5). AccessKit's Windows adapter is known to report editable documents as read-only (`storytold/wordcraft#3174`).
- **Licence gates.** Many stories need third-party crates: archives, image codecs, Excel and PDF reading, SFTP, S3, WebDAV, cloud authorisation, Subversion, credential stores, audio tags. Each needs an allowlist check (tasks flagged `licence-check`), and one that fails needs an RFC and can stall its story.
- **The webview departure.** The Webpages panel (E07-S21, E24-S6) needs a webview, which rule 11 rules out. Settle it before those stories start.
- **Windows-only features.** Registry Compare, shell integration, MTP devices, archive plug-ins, clipboard history and source-control providers need Windows hardware and testing that CI alone won't give.
- **Catalogue names.** View, command and switch names are working names from the catalogue. A name that fails the naming check changes the UI, the docs and possibly the command line (tasks flagged `naming-check`).
- **Large folder trees.** Folder scans, CRC tests and remote listings must stay responsive on very large trees. E03-S2 and E08-S1 carry the budgets; regressions show up late if nobody measures early.

## Repository rules → stories

Behaviour that comes from AGENTS.md rather than the catalogue.

| Rule | Stories |
|---|---|
| 13, 17, 21, 30, 33: lints, CI in its own PRs, keyboard and accessibility tests, one test binary per crate, checks | E27-S1 |
| 14: every feature works without the window | E01-S1, E28-S1, E28-S2 |
| 15: partial or unsupported behaviour says so | E01-S1, E28-S1 |
| 16: text through the translation system | E01-S1 |
| 17, 4: release packages through craft-common-actions, nothing signed or published without the maintainer | E01-S2, E28-S1 |
| 18: agent entry point limited to read and write roots | E28-S2 |

## Roadmap appendix (not in this backlog)

Ideas outside the catalogued feature set. The catalogue raises them as product decisions, and none is scheduled:
- **Modern VCS integration** (Git status and blame in folder views) instead of, or alongside, legacy SCC (Q12, E26-S4).
- **Finder and Nautilus file-manager integration** on macOS and Linux (Q13, E26-S1).
- **Cross-platform clipboard history** (Q14, E26-S3).
- **A built-in script scheduler** (Q15, E20-S6, proposed default: document OS schedulers instead).
- **Nested or context-aware grammars** for mixed languages such as HTML+PHP (F-21.4 keeps grammars deliberately flat).
- **Migration import** of snapshots, settings packages and settings files from third-party comparison tools (Q10, Q11).
- **Settings sync across machines.** The catalogue has no source for it. (UI text goes through the translation system from E01-S1; the translations themselves are added in storytold/craft-i18n.)
- **Web Resources panel** (Q20): explicitly not built.

## Assumptions to verify (every *inferred* item)

Settle each one in product review, for example with a prototype or a user test, or confirm it as a product decision.

| Story | Inferred behaviour |
|---|---|
| E01-S1 | Each launch argument goes to the feature that registered a matching pattern. An argument nobody claims shows a usage message, and the process exits with code 100 (*inferred*). |
| E01-S2 | A macOS build runs natively on both Apple silicon and Intel Macs and opens on a clean machine without security warnings (*inferred* distribution requirement). |
| E02-S1 | **Fallback.** A file that matches no format resolves by content: text-like content gets the built-in catch-all Text format, and binary content gets the Hex kind when one is regi… |
| E02-S1 | AC: the catalogue rename example passes, and a reference to a missing group produces an empty string (*inferred*). |
| E02-S1 | A.1 doesn't say whether masks ignore case or what an edge `-` or an empty set in `[...]` means. Proposed default: a mask compares names with the data source's case sensitivity, so `*.txt` matches `README.TXT` on a case-insens… |
| E02-S1 | A.1 doesn't say where a path mask starts, how file and folder masks combine, or how excludes rank. Proposed default: a path mask without `.\` may start at any depth, so `Windows\*.txt` and `...\Windows\*.txt` select the same files. A file is incl… |
| E03-S2 | Items are aligned by name within each folder. Matching is case-insensitive on case-insensitive sources and case-sensitive otherwise (*inferred*, from "case sensitivity follows t… |
| E04-S1 | The modified time, Windows attributes and Unix permissions can be set. Copying keeps the modified time (*inferred*). |
| E04-S2 | The default is "As folders once opened" (*inferred* from "treated as files when copying or comparing; double-clicking expands them"). |
| E04-S4 | Web inputs are read-only: Save is unavailable, and Save As to a local file is allowed (*inferred*). |
| E04-S4 | Redirects are followed (*inferred*). Fetch errors, such as not found, timeout or TLS failure, are shown and nothing is compared. |
| E04-S5 | Browse for Folder lists connected portable devices next to local drives (*inferred*). |
| E05-S1 | With *Follow operating system* selected, the app uses the OS light or dark setting at start-up and switches when that setting changes while the app runs (*inferred*). |
| E05-S1 | Chrome and selection text stay readable in both themes. An automated check reports any registered colour role whose text falls below a 4.5:1 contrast ratio against its backgroun… |
| E05-S3 | Dropping two folders on Home opens a Folder Compare, with the first dropped folder on the left and the second on the right (*inferred* order: the order the file manager supplies). |
| E05-S4 | The chosen presentation persists per view type across restarts (*inferred* scope). |
| E06-S1 | Saving under a name that already exists asks before overwriting (*inferred*). |
| E06-S1 | Recent sessions are auto-saved when their view closes (*inferred* trigger). The tweak *Max auto-saved sessions* caps the Auto-saved branch, dropping the oldest entries first. A … |
| E06-S1 | Right-clicking a saved session offers *Rename* and *Delete*. Delete asks first (*inferred*). |
| E06-S1 | Passing a saved session's name as the command-line argument opens that session in its view. An existing path wins over a session with the same name (*inferred* precedence). An a… |
| E06-S1 | AC: a saved Folder Compare session and a saved Text Compare session reload after a restart with identical type, specs, description and settings. A corrupt session file is skippe… |
| E06-S2 | Defaults cover the session settings but not the specs or description (*inferred*), and they persist across restarts. |
| E06-S3 | In the Saved branch, the user can create, rename and delete session folders. Folders can nest (*inferred*). |
| E06-S3 | A session can move into or out of a folder by drag and drop or a Move command (*inferred* commands). Save Session As can save straight into a folder. |
| E06-S3 | Deleting a folder that holds sessions asks first, then deletes the folder and its sessions (*inferred*). |
| E06-S3 | A search box above the sessions tree filters every branch to sessions whose name contains the typed text, ignoring case (*inferred*). Clearing the box restores the full tree. |
| E06-S4 | *Lock* and *Unlock* are on the Home right-click menu and the Session menu for named sessions. A locked session shows a lock indicator in the sessions tree (*inferred*). |
| E06-S4 | A locked session can't be overwritten. Save Session is unavailable, and the exit prompt for changed specs doesn't offer to save over it. Save Session As still works (*inferred* … |
| E06-S6 | Tabs holding unnamed sessions come back with their specs and settings (*inferred*). Home tabs come back as Home. |
| E06-S6 | A window that would be restored off-screen, for example on a monitor that is no longer connected, is moved onto a visible screen (*inferred*). |
| E06-S6 | AC: passing a workspace name opens it. A name that matches both a session and a workspace opens the session (*inferred* precedence). |
| E06-S7 | *Use for all files within parent session*: the parent applies the settings to every new child session of this type it opens. Child sessions already open keep their settings (*in… |
| E06-S7 | When the parent folder session is saved, its remembered child settings are saved with it and apply after reopening (*inferred*). |
| E06-S8 | In a Text Compare session with editing disabled, both panes are read-only. Typing, paste and the other editing commands are unavailable, Save is disabled, and the view shows tha… |
| E06-S8 | In a Folder Compare session with editing disabled, Copy, Move, Delete, Synchronize, Rename, Attributes and Touch are unavailable (*inferred*). File pairs opened from it open as … |
| E06-S9 | Choosing a type reopens the same two files in that view type in the same tab, keeping the specs (*inferred*: same tab). |
| E06-S10 | If an input on either side isn't a file inside a folder, the command is unavailable (*inferred*). |
| E06-S11 | The next pair opens in the view its file format chooses, which can be a different view type from the current one (*inferred*). It replaces the current child view (*inferred*). |
| E06-S11 | When there are no more differing pairs in that direction, the command says so and the current view stays open (*inferred*). |
| E07-S1 | Encoding is detected from a byte-order mark, valid UTF-8 or UTF-16, falling back to the system code page. Line endings (Windows, Mac, Unix) are detected per file, and mixed endi… |
| E07-S3 | With "Show filtered line counts" on, each run of hidden lines is summarised by the number of lines it hides (*inferred* presentation). |
| E07-S5 | *Replace* replaces the current match or all matches on the searched sides. Replacements can be undone and update the comparison live. Read-only panes are skipped with a message … |
| E07-S5 | Typing in the find box searches incrementally. After a pause longer than the tweak "incremental-search reset delay" (ms), the next keystroke starts a new search (*inferred* mean… |
| E07-S5 | AC: replacing every `a` with `b` on the left changes only the left pane and marks it unsaved. One Undo reverts all the replacements (*inferred*). |
| E07-S6 | *Go To* moves the cursor in the active pane to a given line and column and scrolls both panes there. Values out of range go to the nearest valid position (*inferred*). |
| E07-S6 | Bookmarked lines are marked in the gutter, and a bookmark moves with its line when lines are inserted or deleted above it (*inferred*). |
| E07-S8 | *Increase Indent* and *Decrease Indent* shift the selected lines by one indent unit (*inferred*: a tab, or the current tab width in spaces). |
| E07-S8 | *Tabs to Spaces*, using the built-in tab width of 4 columns (*inferred*) |
| E07-S9 | The toolbar *Wrap* button (Word Wrap) wraps lines to the pane width. Continuation rows stay aligned across the panes, and the line number appears only on a line's first row (*in… |
| E07-S10 | After an encoding override, saving writes the file in the overridden encoding (*inferred*). |
| E07-S14 | Pressing Esc while an Align With is pending cancels it (*inferred*). |
| E07-S17 | Choosing *Ignored* again on ignored lines removes the suppression, and Undo also restores it (*inferred*). |
| E07-S18 | A clipboard pane has no file behind it, so Save File asks for a path, as Save File As does (*inferred*). |
| E07-S21 | The rendering updates after Recompare or Reload (*inferred*). |
| E07-S22 | Answering yes reloads that file and recompares. Answering no keeps the loaded version and doesn't ask again until the file changes again (*inferred*). |
| E08-S2 | *Ignore Folder Structure*: a flat list where files align by name regardless of subfolder, and each row shows its relative folder (*inferred*). |
| E08-S2 | Changing a filter or mode updates the view without re-scanning the folders (*inferred*). |
| E08-S3 | The toolbar edit and the Name Filters tab always show the same filter (*inferred*). |
| E08-S3 | AC: typing masks and pressing Enter re-filters. An invalid mask shows an error and keeps the previous filter (*inferred*). |
| E08-S4 | The tolerance also applies around the 1-hour and whole-hour offsets (*inferred*). |
| E08-S5 | A center column between the panes shows each pair's result: *Binary same* or *Binary differences*, each with its own glyph (*inferred* glyphs). |
| E08-S6 | Column choices and sort order stay in place through a refresh (*inferred*). |
| E08-S7 | *Copy Filename* copies the full paths of the selected items to the clipboard, one per line (*inferred* separator). |
| E08-S8 | AC: each command loads the expected pair. Set as Base Folders is enabled only with exactly two folders selected (*inferred*). |
| E08-S8 | AC: Up One Level on both sides shows the two parents. At a root it is disabled (*inferred*). |
| E08-S8 | AC: after three re-roots, three Backs return to the first pair and Forward replays them. A new re-root after Back clears the forward history (*inferred*). |
| E08-S9 | *Next Difference* and *Previous Difference* move to the next or previous differing item, skipping same and hidden rows. They open collapsed folders when the next difference is i… |
| E08-S9 | When no further match or difference exists, the view says so (*inferred*). |
| E08-S9 | AC: regex `^test_.*\.py$` with Sides = left finds only left-side matches. With Whole words only, `apple` doesn't match `apples`. Matches inside collapsed folders are found and r… |
| E08-S10 | With one item selected, Open compares it with the item across from it. An orphan opens with the other side empty (*inferred*). |
| E08-S10 | Text Compare child sessions offer *Next Difference Files* and *Previous Difference Files*. These open the parent's next or previous pair with differences, in the parent's displa… |
| E08-S10 | AC: marking two files in different folders and pressing Open compares them. Marks clear after opening (*inferred*). |
| E08-S10 | AC: opening a pair with identical content but different timestamps turns the row black, because override-quick-results is on by default. Child results map to same or different. … |
| E08-S11 | The user can browse, expand and select while background scanning runs (*inferred*). |
| E08-S12 | An item must pass every active filter to stay visible (*inferred*). |
| E08-S12 | AC: "Older than 2 days ago" is evaluated against midnight on the day the comparison runs. "Larger than 1 MB" hides smaller files, with 1 KB = 1024 bytes (*inferred*). |
| E08-S12 | AC: "Containing TODO" keeps only files containing TODO. Regex mode uses the Appendix A.2 syntax. An unreadable file is reported and treated as not matching (*inferred*). |
| E08-S13 | A rules-based comparison runs a binary check first. Files that differ are then compared through their file formats with the text comparison rules, taken from the Text Compare se… |
| E08-S14 | Presets are available in every Folder Compare session (*inferred*). |
| E08-S16 | AC: each selected item opens in its OS default application. For a pair, both items open (*inferred*). |
| E08-S18 | Unsaved edits in the child are saved or discarded first, after a prompt (*inferred*). |
| E08-S18 | At the last differing pair, the copy still happens, and the view reports that no differences are left (*inferred*). |
| E08-S19 | The dialog can open the full view for the same pair (*inferred*). |
| E08-S20 | An automatic refresh keeps folder expansion and the selection for items that still exist (*inferred*). |
| E08-S21 | Running Ignored again on an ignored item restores its real status (*inferred* toggle). |
| E08-S21 | Ignored items stay ignored until the view is closed or refreshed in full (*inferred*; persistence isn't specified). |
| E08-S21 | AC: an ignored conflicting row shows as unchanged and needs no merge action (*inferred*). |
| E08-S22 | Align With works only when the items' parent folders are already aligned. Otherwise it is refused with an explanation (*inferred* message). |
| E09-S2 | A selected folder moves only its visible contents. With an open folder plus some of its children selected, only those children move. A source folder is removed only when nothing… |
| E09-S3 | The confirmation has a **Use recycle bin if possible** checkbox, which applies to local drives only. Items on sources that report no recycle-bin support, such as network paths, … |
| E09-S3 | A selected folder deletes its visible contents. The folder itself goes only when nothing hidden remains in it (*inferred*). |
| E09-S4 | During the run, an item whose target is newer than the source or is read-only raises a per-item confirmation (*inferred*). The "Synchronize confirmations" option (Prompt, the de… |
| E09-S6 | *Regular expressions* mode takes an Old mask, a regex matched against each name, and a New mask template using `$1` etc. For example, `.*(\d\.txt)` → `xyz$1` renames `abc1.txt` … |
| E09-S7 | Checkboxes are tri-state. Unmarked clears the attribute on every item, marked sets it on every item, and gray means mixed. Leaving a box gray leaves each item unchanged (*inferr… |
| E09-S12 | A confirmation appears first. Failures are reported, and no item is lost when one move of a pair fails (*inferred*). |
| E10-S1 | *New* adds a format of any registered kind, *Duplicate* copies one, *Edit* opens the format editor, and *Delete* removes a custom format. Built-in formats can be edited and disa… |
| E10-S1 | AC: `*.txt` below `*.*` shows red, and above it does not. A disabled format shows gray. Complex masks may use a conservative check that never marks a reachable format red (*infe… |
| E10-S3 | Line weights can be set so that alignment prefers to match higher-weight lines. A line's weight is the highest weight of the elements on it (*inferred* mechanism). |
| E10-S4 | With **Column-based data** on, character position matters, and lines are compared column by column rather than by best character match (*inferred* detail). |
| E10-S5 | When the two sides use different formats, each side's text is classified by its own grammar (*inferred*). |
| E10-S7 | Built-in Text formats for Word and PDF files use these converters by default (*inferred* masks: `*.docx;*.doc` and `*.pdf`). |
| E10-S7 | Converted documents are read-only, because there is no saving conversion (*inferred*). |
| E10-S9 | The **Quick merge** command gives a merge result without UI, with exit codes 0 and 1. Opening a merge of matching files runs the **Merge view** command (*inferred* exit-code mea… |
| E11-S1 | On macOS and Linux, a waiting command does the same (*inferred* for Linux). |
| E11-S2 | The exit code is 1 when the files are the same by the chosen test and 11 when they differ. These codes are specified for binary and *inferred* for size and CRC. |
| E11-S2 | A missing or unreadable file, a folder in place of a file, or a wrong number of files gives exit code 100, with a one-line error on standard error (*inferred* for the message). |
| E11-S3 | When the comparison opened isn't a folder comparison, both switches are ignored and a warning is shown (*inferred*). |
| E11-S4 | With `-` as the only argument, standard input loads into the left pane and the right pane starts empty (*inferred*). |
| E11-S4 | The view is chosen as for a file: text content opens in Text Compare. Because there's no file name, the format is detected from the content (*inferred*). |
| E11-S4 | The path edit of the standard-input pane shows a label for standard input (*inferred*). Saving that pane asks for a file name. |
| E11-S5 | `/automerge` with the positional arguments `<left> <right> <center> <output>` merges without interaction when there are no conflicts. It writes the output and exits with 0 (*inf… |
| E11-S5 | `/force` with `/automerge` writes the output with CVS-style markers (`<<<<<<<`, `=======`, `>>>>>>>`) around each conflict and exits with 14 (Conflicts detected). The code mappi… |
| E11-S5 | `/reviewconflicts` with `/automerge` opens Text Merge when conflicts remain. The launcher exits with 0 when the user saves the merge output, and with 101 when Text Merge closes … |
| E12-S1 | **Sync Now** and *Session > Run Sync* run every planned action, with progress, cancel and an error summary, after a confirmation showing the counts (*inferred*). The preview ref… |
| E12-S1 | AC: Home and New Session offer Folder Sync. The setup accepts any two folder specs, and Accept is needed only after a change (*inferred*). |
| E12-S1 | AC: `otd /sync A B` opens Folder Sync with A and B. On Linux the switch is reported as unsupported (*inferred*). |
| E12-S2 | **Next Conflict** and **Previous Conflict** jump between rows whose action the method can't decide on its own, such as a pair that differs with neither side newer (*inferred* de… |
| E12-S3 | A custom method can be saved as a named preset and picked again later (*inferred*). |
| E13-S1 | *Text Merge Info* shows merge statistics: counts of left, right, same and different changes and of conflicts (*inferred* fields). *Swap Sides* exchanges the left and right inputs. |
| E13-S1 | Without a center file, the view still opens: left and right are compared directly, and every difference is flagged as a conflict for manual choice (*inferred*). |
| E13-S2 | Changing the setting rebuilds the merge with the new set of conflicts. If the output has unsaved edits, the user is asked first (*inferred*). |
| E13-S5 | Changing these settings merges the inputs again. If the output has unsaved edits, the user is asked first (*inferred*). |
| E13-S5 | *Reload* re-reads the inputs and prompts first if the output has unsaved edits. *Recompare* merges again without re-reading (*inferred* distinction, following Text Compare). |
| E13-S5 | `/iu` (`/ignoreunimportant`) with `/automerge` turns on Ignore Unimportant Differences for the headless merge. Differences the session's importance rules class as unimportant co… |
| E13-S5 | AC: Compare to Output (Right) opens a Text Compare of the right input against the output, including unsaved output edits (*inferred*). Merge Parent Folders opens a Folder Merge … |
| E14-S1 | AC: unit cases cover add left, add right, delete left, delete right, change left, change right, same change and different change, plus the no-center case, in which every differi… |
| E14-S2 | A file changed on both sides is merged line by line when that merge has no conflicts. Otherwise it is left for a manual merge and listed in the summary (*inferred*). |
| E14-S2 | `/fv="Folder Merge"` with left, right and optional center folders opens a Folder Merge, and `/mergeoutput=<folder>` sets its output folder (*inferred*). |
| E14-S2 | AC: `otd /fv="Folder Merge" L R C /mergeoutput=OUT` opens Folder Merge with L, R and C as inputs and OUT as the Merge to folder (*inferred* argument order, as for Text Merge). |
| E15-S1 | Built-in Table formats are registered for comma-separated and tab-separated files (masks `*.csv` and `*.tsv;*.tab`, *inferred*). Two such files open in Table Compare whether pas… |
| E15-S1 | Two grids, left and right, scroll together. Their columns are comparison columns, and in this story they pair up by position (*inferred* default). |
| E15-S1 | A file that can't be parsed as a table shows an error in its own pane, and the other pane still loads (*inferred*). |
| E15-S1 | AC: fixtures cover comma, tab and fixed-width files, each with and without a header row, and all are detected correctly. A quoted field containing a delimiter stays one cell (te… |
| E15-S2 | Any combination of comparison columns can be marked as key columns. Marking is done from the column header's context menu, and key columns carry a marker (*inferred* placement; … |
| E15-S2 | Changing the keys recompares immediately (*inferred*). |
| E15-S3 | Numbers within the Numeric tolerance, and date-times within the Date tolerance, are unimportant differences. Values are parsed using the system's regional settings (*inferred*). |
| E15-S4 | Built-in Excel → table conversion opens Excel workbooks in Table Compare (masks `*.xlsx;*.xls`, *inferred*). Cell values are compared; formatting and embedded images are not. |
| E15-S4 | The built-in conversion is load-only: edits can't be saved back to a workbook (*inferred*). |
| E15-S4 | An "HTML tables" format turns each table in an HTML file into a sheet. It ships built in rather than as a separate add-on (PO decision, *inferred*). |
| E15-S4 | A file with several sheets shows tabs along the bottom, one per comparison sheet. Each tab has a status spot: red when the sheet has important differences, blue when it has only… |
| E15-S4 | AC: a workbook with three sheets yields three tables with their sheet numbers and names. Formula cells compare by their stored values (*inferred*). |
| E15-S5 | Modify toggles edit mode on the current cell, and typing changes its value. The comparison updates after the edit (*inferred*: live recompare, as in Text Compare). |
| E15-S5 | AC: two forced rows stay aligned until a recompare resets the alignment (*inferred*). |
| E15-S6 | Find searches cell text on the chosen sides, as plain text or a regular expression, with case and whole-word options (*inferred*: the same options as Text Compare's find). Find … |
| E15-S6 | Replace replaces the matched cell text. A replaced cell counts as an unsaved change (*inferred*). |
| E15-S6 | The Thumbnail shows one colour row per comparison row and marks the viewport; clicking it jumps there (*inferred*: as in Text Compare). |
| E15-S7 | Under the General column type, values that parse as the same number or date compare as equal (*inferred*). |
| E15-S7 | AC: with decimal separator `,`, the cells `1,5` and `1,50` compare as the same number (*inferred* numeric comparison). |
| E15-S8 | Unaligned compares row N with row N. *Sort rows before alignment* sorts both sides by all columns, left to right, before aligning (*inferred* sort key). |
| E15-S8 | Changing a Rows setting recompares immediately (*inferred*). |
| E16-S1 | A Hex format kind and a built-in Hex format are registered. Two binary files that match no other format open in Hex Compare, whether passed on the CLI or chosen through Home or … |
| E16-S1 | Each row shows an address, the bytes in hex, and a character panel (*inferred*). View > Byte Addresses toggles the address column, and the current byte address is shown. |
| E16-S1 | Differing bytes are highlighted. Where alignment inserts a gap, the row is split and the gap is crosshatched, so a row can hold fewer bytes than usual (*inferred*). |
| E16-S1 | Side-by-Side and Over-Under layouts are available. File Info shows the timestamp, size and a format dropdown (*inferred*). |
| E16-S1 | The status bar shows the overall result (for example "Binary differences"), the insert/overwrite state and the load time (*inferred*). The Insert key toggles the insert/overwrit… |
| E16-S1 | AC: when the file is smaller than the threshold, another program can overwrite it while it's open; when it's larger, it can't (*inferred*). |
| E16-S2 | Typing hex digits changes bytes; typing in the character panel does too (*inferred*). Typing follows the insert/overwrite mode shown in the status bar. |
| E16-S2 | Cut, Copy, Paste, Delete and Select All work on byte selections. Copy puts the bytes on the clipboard (*inferred* format: hex text). |
| E16-S2 | Undo and Redo cover every edit, and the comparison updates after each one (*inferred*). |
| E16-S3 | Find searches for a byte sequence in hex, or for text (*inferred*). Find Next and Find Previous step through the matches. |
| E16-S3 | Replace replaces the matched bytes. Replaced bytes count as unsaved changes (*inferred*). |
| E16-S3 | The Little Endian Values and Big Endian Values panels interpret the bytes at the cursor (*inferred* value types: 8-, 16-, 32- and 64-bit integers, signed and unsigned, and 32- a… |
| E16-S3 | The Thumbnail shows one colour row per grid row and marks the viewport; clicking it jumps there (*inferred*: as in Text Compare). |
| E17-S1 | Only 32 bpp data is compared. An image in an unsupported colour space (HDR, wide gamut) reports that it can't be compared exactly (*inferred* message). |
| E17-S1 | Zoom In, Zoom Out, Actual Size and Fit to Window apply to all panes together (*inferred*). |
| E17-S1 | Swap Sides, Reload, Recompare and Picture Compare Info work. Info shows the dimensions and pixel difference counts (*inferred* content). |
| E17-S1 | File Info shows each image's details (*inferred*: dimensions, file size, modified time). Display Font for listings (hidden by default) changes the font of the view's text. |
| E17-S3 | These transformations change only the comparison, never the files (*inferred*). |
| E17-S5 | On a platform with no codec for a format, that format isn't offered, and its files fall back to another view (*inferred*). |
| E18-S1 | The script command `text-report layout:<layout> [options:<list>] [title:<text>] output-to:(clipboard/<file>) [output-options:<list>] [<comparison>]` produces the same report hea… |
| E18-S1 | AC: interleaved with *strikeout-left-diffs* strikes through the changed left text. The summary gives totals only (*inferred* content). |
| E18-S1 | AC: the unified patch applies cleanly with a standard patch tool. The statistics CSV opens in a spreadsheet with one row per file pair (*inferred* columns). |
| E18-S2 | The report covers the loaded folders in their current expansion state. This is specified for scripts and *inferred* for the dialog. |
| E18-S3 | AC: a `text-report … output-to:printer` script prints without a dialog, to the default printer (*inferred*). |
| E18-S4 | *Table Compare Report* offers the layouts side-by-side, summary and interleaved, and the options *ignore-unimportant*, *display-all*, *display-mismatches*, *display-context* and… |
| E18-S4 | AC: the side-by-side HTML report embeds the left, right and difference images (*inferred*). The summary says whether the images match, and respects *ignore-unimportant*. |
| E18-S5 | AC: a text report in *xml* validates against the schema. Applying the recorded differences to the left text gives the right text (*inferred* completeness check). |
| E19-S1 | A snapshot source reports file content as unreadable, so every content test other than saved CRCs reports "unavailable". This is checked with Binary comparison and with the scri… |
| E19-S1 | AC: `otd <snapshot>` opens a Folder Compare with the snapshot on one side and its source folder on the other (*inferred* order: snapshot on the left). If the source folder no lo… |
| E20-S2 | A command run with an empty selection does nothing and logs a warning (*inferred*). Errors follow `option stop-on-error`. |
| E20-S3 | On Windows, `filter attrib:((+/-)<[a][c][e][h][i][l][o][p][r][s][t][u][z]>/none)` filters by DOS attributes. `+` keeps only items with the attribute and `-` only items without i… |
| E20-S5 | Filters set with `filter` limit what `sync` synchronises (*inferred*). |
| E20-S5 | AC: each sample runs headlessly against fixture folders, with a test FTP server for sample 2, and gives the specified outcome with exit code 0 (*inferred* success code). |
| E21-S1 | `ftp://username@ftp.hostname.com/subfolder` (relative to the login folder, *inferred*) |
| E21-S1 | `ftp://username@ftp.hostname.com//subfolder` (absolute, *inferred*) |
| E21-S1 | A connection or login failure shows as a folder error carrying the server's message. When no password is saved, the user is asked for one on connect (*inferred*). |
| E21-S2 | Aggressive uploads to OpenSSH servers, which uses larger buffers and pipelining on SSH connections (*inferred* scope). |
| E21-S2 | AC: with FTP trace on, a connection logs its commands and replies with passwords masked (*inferred*). With IPv6 on, an IPv6-only test server connects. |
| E21-S3 | E21-S3-T3 — FTPS data source (explicit TLS, *inferred*) with an optional client certificate · size: M · depends: E21-S3-T1 |
| E21-S4 | `profile:MyS3` opens as a folder. Buckets show as top-level folders, or the configured bucket's contents show directly, and key prefixes show as folders (*inferred*). |
| E21-S4 | The S3 kind declares itself a remote service. *Saved S3 credentials* go through the shared Save-password control, so they are stored the same way as saved pass… |
| E21-S5 | With Recursive listings on, the whole tree is fetched in one listing request (*inferred* mechanism). With it off, folders are listed one at a time. |
| E21-S5 | An authentication failure shows as a folder error. Save password works as it does for FTP profiles (*inferred*). |
| E21-S5 | AC: with Recursive listings on, expanding a deep tree makes one listing request (*inferred*). With it off, it makes one request per folder. |
| E21-S7 | An expired or revoked authorisation shows an error that offers a way to authorise again (*inferred*). |
| E21-S8 | An expired or revoked authorisation shows an error that offers a way to authorise again (*inferred*). |
| E21-S9 | With no revision given, the latest revision is used (*inferred*). |
| E22-S3 | *Use stripes* tints alternate lines in text-based file views (*inferred* scope). |
| E22-S4 | AC: items, with every field filled in, persist across restarts and can be reordered and deleted (*inferred* reorder). |
| E22-S5 | *Single vertical scrollbar* replaces the two vertical scrollbars in Text Compare with one shared scrollbar (*inferred* placement). |
| E22-S5 | *Sticky splitter position* keeps the splitter between the panes where the user put it when the window resizes or a new comparison loads, instead of re-centring it (*inferred* me… |
| E23-S1 | Every package records its format version. A package from a newer format version is refused with a message. Migrating older format versions is deferred until a second format vers… |
| E23-S1 | AC: an export with System Info includes the OS version, product version and display settings (*inferred* contents). Import ignores this category. |
| E23-S2 | A confirmation lists what will be reset or deleted before anything changes (*inferred*). |
| E23-S3 | Shared sessions open like saved sessions but are read-only: they can't be renamed, deleted or saved in place. Save Session As makes a personal copy (*inferred*). |
| E23-S3 | After the package is re-exported to the same path, the Shared branch shows the new contents the next time Home refreshes or the app starts (*inferred* timing). |
| E23-S3 | If the shared file is unreachable or invalid, the Shared branch shows a message and personal sessions are unaffected (*inferred*). |
| E23-S4 | If the program folder isn't writable, the app says so at start-up rather than silently losing changes (*inferred*). |
| E24-S2 | *Next/Previous Difference Files* moves between the file pairs of a multi-file patch (*inferred*). |
| E24-S2 | A malformed patch shows an error that names the first line that couldn't be read (*inferred*). |
| E24-S3 | The results list every matching line, with its file and line number (*inferred* layout). |
| E25-S1 | Two FLAC, MP3 or MP4/AAC files open in Media Compare from the CLI, from Home, or through Session > New Session ▸ Media Compare. The view is chosen by file type (*inferred*). |
| E25-S1 | A Name/Value tree shows both files' tags, side by side or over-under (*inferred*). It has two groups: |
| E25-S1 | A tag present on one side only shows as a crosshatched gap, and changed characters inside a value are highlighted (*inferred*). |
| E25-S1 | The file-info bar shows the timestamp, the size and the tag version, for example ID3v2.3 (*inferred*). |
| E25-S1 | Each pane has a play button and a seek bar. Play plays that pane's audio, and Play Both plays both files (*inferred*: at the same time). |
| E25-S1 | Display filters are Show All, Show Differences, Show Same and Show None, plus Ignore Unimportant Differences. The toolbar has All, Diffs, Same, Minor, Rules, Next Diff, Prev Dif… |
| E25-S2 | Two exe, dll or ocx files open in Version Compare from the CLI, from Home, or through Session > New Session ▸ Version Compare. The view is chosen by file type (*inferred*). |
| E25-S2 | A Name/Value tree has these groups (*inferred*): |
| E25-S2 | Version resources stored in MUI (multilingual resource) files are found and compared (*inferred* reading of "better MUI handling"). |
| E25-S2 | A file with no version information, or one that can't be read, shows a clear error in its own pane, and the other side still loads (*inferred* reading of "better error handling"). |
| E25-S4 | AC: a `.reg` export of a key compares as same with the live key. A remote machine that meets the prerequisites loads. One that doesn't shows a message naming the prerequisite to… |
| E25-S5 | Changes to a live registry take effect at once (*inferred*). A `.reg`-file side is read-only (*inferred*). |
| E25-S5 | Delete asks for confirmation first (*inferred*). |
| E25-S5 | A change the user isn't permitted to make, for example in HKLM without admin rights, is refused with the system message, and nothing is half-applied (*inferred*). |
| E26-S3 | The main app's Startup options page has Run Clipboard Compare on Windows startup, kept in step with the utility's own option (*inferred*). |
| E26-S5 | Changes that shell actions make to files are picked up on the next refresh (*inferred*). |

## Traceability (catalogue feature → stories)

| Feature | Stories |
|---|---|
| F-5.1 Multi-window, multi-tab shell | E01-S1, E05-S2 |
| F-5.2 Home view | E01-S1, E05-S3, E06-S1, E06-S3, E06-S5, E23-S3 |
| F-5.3 Menus, toolbars and display-filter toolbar layouts | E05-S4, E22-S1, E22-S5 |
| F-5.4 Light/dark theme | E05-S1 |
| F-6.1 Session lifecycle: save, auto-save, lock, rename, delete, folders | E01-S1, E06-S1, E06-S3, E06-S4, E07-S1, E08-S1, E08-S8 |
| F-6.2 Session settings dialog and scope of changes | E01-S1, E06-S2, E06-S7 |
| F-6.3 Rules vs. file formats separation | E03-S1, E07-S4, E10-S5 |
| F-6.4 Child sessions and folder↔file coordination | E06-S7, E06-S8, E06-S11, E08-S10, E08-S18, E14-S2, E25-S6 |
| F-6.5 Workspaces | E06-S6 |
| F-6.6 Session-level "Disable editing" and description | E01-S1, E06-S8, E15-S5, E16-S2 |
| F-7.1 Local and network paths | E01-S1, E04-S1, E04-S5, E07-S1, E08-S1, E09-S9, E22-S5 |
| F-7.2 Other spec kinds: web, clipboard, stdin, registry | E04-S4, E07-S18, E11-S4, E25-S4 |
| F-7.3 Archives as folders | E04-S2, E04-S3, E04-S6, E09-S5, E09-S9 |
| F-7.4 FTP / SFTP / FTPS | E12-S4, E21-S1, E21-S2, E21-S3 |
| F-7.5 Cloud and other remote profiles | E21-S1, E21-S4, E21-S5, E21-S7, E21-S8, E21-S9 |
| F-7.6 Browse using profiles / Quick Connect | E21-S1, E21-S6, E21-S9 |
| F-7.7 Snapshots as a data source | E19-S1 |
| F-8.1 Folder Compare display: tree, colours, icons | E03-S2, E04-S2, E08-S1, E08-S5, E08-S11, E08-S17, E22-S2 |
| F-8.2 Folder columns | E08-S1, E08-S6, E08-S17, E25-S3, E26-S4 |
| F-8.3 Comparison criteria (quick tests and content tests) | E03-S2, E08-S4, E08-S5, E08-S10, E08-S13, E08-S17, E25-S3 |
| F-8.4 Alignment of items | E03-S2, E08-S1, E08-S2, E08-S4 |
| F-8.5 Alignment overrides and manual align | E08-S22 |
| F-8.6 Folder handling options | E04-S2, E08-S11, E08-S17, E08-S20, E21-S2 |
| F-8.7 Filters: overview | E03-S2, E08-S3, E08-S15 |
| F-8.8 Name filters and the toolbar filter box | E08-S3, E08-S14 |
| F-8.9 Other filters | E08-S12, E08-S17, E26-S4 |
| F-8.10 Misc tab: per-session file-format enablement | E08-S22, E10-S8 |
| F-8.11 Display filters (Folder Compare) | E08-S2, E08-S13 |
| F-8.12 Folder display filters | E08-S2 |
| F-8.13 Selection model | E08-S7 |
| F-8.14 Navigation and structure commands | E08-S1, E08-S8, E08-S9, E12-S1, E14-S1 |
| F-8.15 Opening items in file views | E08-S7, E08-S10, E08-S16, E08-S19, E22-S4, E24-S5, E26-S5 |
| F-8.16 "Ignored" items | E08-S21 |
| F-8.17 Folder Compare Info and Report | E18-S2 |
| F-8.18 Synchronize submenu (inside Folder Compare) | E09-S4 |
| F-9.1 Copy to Left / Right / Side / Other Side | E09-S1 |
| F-9.2 Move to Left / Right / Side / Other Side | E09-S2 |
| F-9.3 Exchange | E09-S12 |
| F-9.4 Copy to Folder / Move to Folder | E09-S5 |
| F-9.5 Delete | E04-S1, E09-S3 |
| F-9.6 Rename (single and multi-file) | E09-S6 |
| F-9.7 Attributes | E09-S7 |
| F-9.8 Touch | E09-S8 |
| F-9.9 New Folder | E09-S9 |
| F-9.10 Exclude, Ignored, Copy Filename, Refresh Selection | E08-S1, E08-S3, E08-S7, E08-S21 |
| F-9.11 Source-control actions in folder views | E26-S4 |
| F-10.1 Folder Merge specs and settings | E10-S8, E14-S1 |
| F-10.2 Folder Merge display and status colours | E14-S1 |
| F-10.3 Folder Merge commands | E08-S21, E14-S1, E14-S2 |
| F-11.1 Sync setup: specs, method, rules, filters | E10-S8, E12-S1, E12-S3, E12-S4 |
| F-11.2 Sync preview and per-item overrides | E12-S1, E12-S2 |
| F-11.3 Running the sync | E12-S1 |
| F-12.1 Text Compare display | E07-S1, E07-S9, E07-S11, E07-S12, E07-S20, E07-S21, E10-S2 |
| F-12.2 Display filters and context | E07-S3, E07-S4, E07-S17 |
| F-12.3 Editing in Text Compare | E06-S8, E07-S1, E07-S7, E07-S8, E10-S4 |
| F-12.4 Walking through differences | E07-S2, E07-S16, E07-S19, E08-S18, E10-S4 |
| F-12.5 Search, replace, go to, bookmarks | E07-S5, E07-S6 |
| F-12.6 Importance rules (Importance tab) | E03-S1, E07-S4, E07-S15, E10-S3, E10-S5 |
| F-12.7 Alignment algorithms (Alignment tab) | E03-S1, E07-S13, E07-S14, E10-S3 |
| F-12.8 File format and encoding overrides (Format tab) | E06-S9, E07-S10 |
| F-12.9 Text replacements | E07-S16 |
| F-12.10 Other Text Compare commands | E06-S10, E07-S1, E07-S18, E13-S1, E22-S4, E26-S4, E26-S5 |
| F-13.1 Merge model and automatic merge | E13-S1, E13-S2, E13-S5 |
| F-13.2 Resolving: Take commands and output editing | E13-S1, E13-S2, E13-S3, E13-S5 |
| F-13.3 Merge navigation | E13-S1, E13-S3 |
| F-13.4 Merge display filters and view options | E13-S4 |
| F-13.5 Merge session commands | E13-S1, E13-S5 |
| F-14.1 Grid display | E15-S1, E15-S6 |
| F-14.2 Multiple sheets and HTML tables | E15-S4 |
| F-14.3 Rows: keys, alignment and sorting | E15-S2, E15-S8 |
| F-14.4 Columns: alignment and per-column handling | E15-S3 |
| F-14.5 Table editing | E15-S5 |
| F-14.6 Table navigation and search | E06-S11, E15-S1, E15-S6 |
| F-14.7 Table session commands and settings | E15-S1, E15-S7, E18-S4 |
| F-15.1 Hex display and options | E16-S1, E16-S3 |
| F-15.2 Hex comparison settings | E16-S1 |
| F-15.3 Hex editing and navigation | E06-S11, E16-S1, E16-S2, E16-S3, E18-S4 |
| F-16.1 Supported image types | E17-S1, E17-S5 |
| F-16.2 Difference visualisation modes | E17-S1, E17-S2 |
| F-16.3 Geometric alignment | E17-S1, E17-S3 |
| F-16.4 Picture session settings and commands | E06-S11, E17-S1, E17-S4, E18-S4 |
| F-17.1 Media Compare | E18-S6, E25-S1, E25-S6 |
| F-18.1 Registry display and editing | E18-S6, E25-S4, E25-S5 |
| F-19.1 Version Compare | E18-S6, E25-S2, E25-S6 |
| F-20.1 Text Edit (single-pane editor) | E24-S1, E24-S3, E24-S5, E24-S6 |
| F-20.2 Text Patch (patch viewer and applier) | E24-S2, E24-S4, E24-S5 |
| F-21.1 File format manager | E02-S1, E08-S13, E10-S1 |
| F-21.2 Common format fields: General tab | E02-S1, E10-S1 |
| F-21.3 Conversion (load and save filters) | E10-S6, E10-S7, E15-S4 |
| F-21.4 Grammar (text formats) | E10-S2, E10-S3 |
| F-21.5 Text format: Misc tab | E10-S4 |
| F-21.6 Table format: Type and Regional tabs | E15-S1, E15-S7 |
| F-21.7 External format (delegate to another application) | E10-S9 |
| F-22.1 Interactive reports | E18-S1, E18-S2, E18-S3 |
| F-22.2 Report layouts and options (from the script reference, the authoritative list) | E18-S1, E18-S2, E18-S3, E18-S4, E18-S5, E18-S6, E26-S4 |
| F-23.1 Save snapshot | E19-S1, E25-S3 |
| F-23.2 Compare against snapshot | E19-S1 |
| F-24.1 Script execution model | E20-S1 |
| F-24.2 Script command reference | E18-S1, E18-S2, E18-S4, E18-S6, E19-S1, E20-S1, E20-S2, E20-S3, E20-S4, E20-S5, E25-S3 |
| F-24.3 Scheduled scripts | E20-S6 |
| F-25.1 Executables and the single-instance model | E01-S1, E01-S2, E11-S1 |
| F-25.2 Positional parameters | E01-S1, E06-S1, E06-S6, E07-S1, E08-S1, E11-S1, E11-S4, E11-S5, E13-S1, E19-S1, E20-S1, E23-S1, E24-S2 |
| F-25.3 Switches | E08-S13, E09-S10, E11-S1, E11-S2, E11-S3, E11-S5, E12-S1, E20-S1, E23-S1, E24-S1 |
| F-25.4 Exit codes | E08-S13, E11-S1, E11-S2, E11-S5, E20-S1 |
| F-26.1 Startup | E06-S6, E08-S19, E26-S1, E26-S3 |
| F-26.2 Tabs | E05-S2, E06-S7 |
| F-26.3 Appearance: theme and preview | E05-S1 |
| F-26.4 Folder view colours and fonts | E14-S1, E22-S2 |
| F-26.5 File view colours and fonts | E22-S3 |
| F-26.6 Picture Compare colours | E17-S2 |
| F-26.7 Log panel | E09-S11, E21-S2 |
| F-26.8 Toolbar, shortcut and menu customisation | E05-S4, E22-S1 |
| F-26.9 Backups | E09-S10 |
| F-26.10 File operation confirmations | E08-S15, E09-S1, E09-S2, E09-S3, E09-S4 |
| F-26.11 Text Editing, Next Difference and Tweaks | E06-S1, E06-S5, E07-S3, E07-S5, E07-S8, E07-S11, E07-S19, E07-S20, E07-S22, E08-S14, E08-S17, E09-S11, E13-S4, E21-S2, E22-S5, E22-S6, E23-S3 |
| F-26.12 Archive Types | E04-S3 |
| F-26.13 "Open With" applications | E22-S4 |
| F-27.1 Settings storage locations | E01-S1 |
| F-27.2 Export settings | E23-S1 |
| F-27.3 Import settings and restore defaults | E23-S1, E23-S2 |
| F-27.4 Shared sessions | E23-S3 |
| F-27.5 Portable installation | E23-S4 |
| F-28.1 File-manager context menu (Explorer) | E26-S1, E26-S2 |
| F-28.2 Clipboard Compare tray utility | E26-S3 |
| F-28.3 VCS / external-tool integration (diff and merge tool) | E11-S1, E11-S5 |
| F-28.4 Source control integration (SCC / MSSCCI) | E26-S4 |
| F-28.5 Drag and drop | E05-S3, E19-S1 |

