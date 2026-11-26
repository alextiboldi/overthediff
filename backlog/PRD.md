# OverTheDiff — product requirements

## What it is
OverTheDiff is a desktop tool for Windows, macOS and Linux. It is for developers, release engineers, IT administrators and analysts who need to see exactly how two files or folders differ, and then fix those differences. It compares text, folder trees, tables, binaries, images and more side by side. It separates differences that matter from those that don't, and lets people edit, copy, synchronise and merge in place, whether they work interactively, from their version-control tool or from scripts.

> **Basis.** This PRD comes from the OverTheDiff feature catalogue, a behaviour-level specification of the product. Every capability here is **spec-derived and not yet validated** with users or a prototype. Statements marked *inferred* are analyst or PO inferences, or details read only from a figure. The epics are the executable source of truth; this document is the readable summary.

## Problem & goals
- **Problem**: Line-diff tools show what changed, but they can't be told what doesn't matter, such as whitespace, comments, timestamps within a tolerance or renamed identifiers. They can't safely reconcile two folder trees, and they stop at plain text. People end up combining several tools and ad-hoc scripts, and they still miss or over-report differences.
- **Goals**:
  1. **Trustworthy differences.** Classify every difference as important or unimportant through explicit, saved rules, and let the user hide the noise.
  2. **Reconcile in place.** Edit, copy, move, delete, synchronise and merge without leaving the comparison.
  3. **One tool for many inputs.** Cover local and network folders, archives, remote services, snapshots, the clipboard and web pages, all treated alike.
  4. **Automation.** Provide a command line with meaningful exit codes, act as the diff and merge tool for version control, and run headless scripts.
  5. **Complete, shipped incrementally.** Deliver every catalogued capability except licensing and support (see non-goals), releasing a credible MVP first and adding capability in independent slices.

## Target users
- **Developers and code reviewers** compare revisions, use the product as `git difftool`/`mergetool`, ignore formatting noise and copy changes across.
- **Release and build engineers** verify build output and archives, keep deployment folders in sync, and produce patches and reports.
- **IT administrators** synchronise and back up folders, compare servers over FTP, SFTP or S3, and compare executables and the registry on Windows.
- **Data analysts** compare CSV and Excel data with key columns and numeric or date tolerances.
- **Designers and QA engineers** compare images pixel by pixel and compare media tags.
- **AI agents** compare and reconcile files through MCP, inside the folders their operator allows.
- **Automation and power users** script comparisons, schedule synchronisation, and customise commands, shortcuts and colours.

## Core capabilities
Each item below links to its epic file; the stories and tasks there are the executable detail.

**Foundations**, built first and invisible on their own:
- **Repository, CI and test harness** ([`epic-27-repository-ci-and-test-harness.md`](epic-27-repository-ci-and-test-harness.md)): the setup story, built alone before everything else. It covers a Cargo workspace whose engines don't depend on the UI, CI through the shared workflows, and a UI harness that drives the keyboard and reads the accessibility tree.
- **Application platform** ([`epic-01-application-platform.md`](epic-01-application-platform.md)): a single-instance app with tabs, a command registry, an Options dialog, a session framework, durable per-user settings and background tasks. Every command runs by id with typed arguments, with or without the window, and all text goes through the translation system. Features plug into it, so no story waits on another story's scaffolding. It also covers installable builds for all three OSes.
- **Pattern and file-type services** ([`epic-02-pattern-and-file-type-services.md`](epic-02-pattern-and-file-type-services.md)): file masks, regular expressions and the ordered file-format registry that decides how each file opens.
- **Comparison engines** ([`epic-03-comparison-engines.md`](epic-03-comparison-engines.md)): a text engine (alignment, character differences, importance classification) and a folder engine (scanning, alignment, quick and content tests, status aggregation).
- **Data sources** ([`epic-04-data-sources.md`](epic-04-data-sources.md)): one contract for local and network paths, zip and other archives, web pages and portable devices.

**MVP features** (Release 1):
- **Text Compare** ([`epic-07-text-compare.md`](epic-07-text-compare.md)): side-by-side panes that scroll together, with full editing, difference navigation, display filters, find and replace, and importance rules for whitespace, case and line endings. Later releases add thumbnails, details panels, alternative alignment algorithms, replacements and more.
- **Folder Compare** ([`epic-08-folder-compare.md`](epic-08-folder-compare.md)): aligned folder trees with status colours, quick and content tests (including rules-based), name and display filters, selection and navigation commands, and drilling down into file pairs.
- **Folder file operations** ([`epic-09-folder-file-operations.md`](epic-09-folder-file-operations.md)): copy, move, delete and synchronise the selection, with confirmations. Later releases add rename, attributes, touch, backups and an operation log.
- **File formats and grammars** ([`epic-10-file-formats-and-grammars.md`](epic-10-file-formats-and-grammars.md)): a File Formats manager, built-in grammars with syntax highlighting, and grammar-based importance (for example, ignoring comments). Later releases add a grammar editor, conversions and external formats.
- **Sessions and workspaces** ([`epic-06-sessions-and-workspaces.md`](epic-06-sessions-and-workspaces.md)): saved and auto-saved sessions on Home, and per-type defaults. Later releases add folders, locking, workspaces and child-session scopes.
- **Command line and VCS integration** ([`epic-11-command-line-and-vcs-integration.md`](epic-11-command-line-and-vcs-integration.md)): waiting launchers, titles, read-only and save-target switches, headless quick compare with exit codes, and difftool setup. Merge-tool switches come with Text Merge.
- **App shell, themes and windows** ([`epic-05-app-shell-themes-and-windows.md`](epic-05-app-shell-themes-and-windows.md)): light and dark themes in the MVP. Later releases add multiple windows, drag and drop, and display-filter toolbar layouts.

**Power-user features** (Release 2) and **the full feature set** (Release 3):
- **Folder Sync** ([`epic-12-folder-sync.md`](epic-12-folder-sync.md)): saved sync sessions with a preview, per-item overrides and custom methods.
- **Text Merge** ([`epic-13-text-merge.md`](epic-13-text-merge.md)) and **Folder Merge** ([`epic-14-folder-merge.md`](epic-14-folder-merge.md)): 3-way merge with automatic output and conflict resolution.
- **Table Compare** ([`epic-15-table-compare.md`](epic-15-table-compare.md)), **Hex Compare** ([`epic-16-hex-compare.md`](epic-16-hex-compare.md)) and **Picture Compare** ([`epic-17-picture-compare.md`](epic-17-picture-compare.md)): specialised viewers for tables, binaries and images.
- **Reports** ([`epic-18-reports.md`](epic-18-reports.md)): HTML, text, patch, XML and printed reports for every view type.
- **Snapshots** ([`epic-19-snapshots.md`](epic-19-snapshots.md)): point-in-time folder listings that compare later like a folder.
- **Scripting** ([`epic-20-scripting.md`](epic-20-scripting.md)): headless scripts over an invisible folder comparison, with file operations, filters, sync and reports.
- **Remote services and profiles** ([`epic-21-remote-services-and-profiles.md`](epic-21-remote-services-and-profiles.md)): FTP, SFTP/FTPS, S3, WebDAV, Dropbox, OneDrive and Subversion through named profiles.
- **Preferences and customisation** ([`epic-22-preferences-and-customisation.md`](epic-22-preferences-and-customisation.md)): menus, toolbars, shortcuts, colours, fonts, Open With applications and tweaks.
- **Settings portability** ([`epic-23-settings-portability.md`](epic-23-settings-portability.md)): export and import packages, factory reset, shared sessions and portable installs.
- **Text Edit and Text Patch** ([`epic-24-text-edit-and-text-patch.md`](epic-24-text-edit-and-text-patch.md)): a single-pane editor with Find in Files, plus a patch viewer and applier.
- **Media, Version and Registry Compare** ([`epic-25-media-version-and-registry-compare.md`](epic-25-media-version-and-registry-compare.md)): audio tags, executable version resources and the Windows registry.
- **OS integrations** ([`epic-26-os-integrations.md`](epic-26-os-integrations.md)): file-manager context menu, clipboard history tray and source-control providers.
- **Automation and agent access** ([`epic-28-automation-and-agent-access.md`](epic-28-automation-and-agent-access.md)): `otd-cli` runs any command without the window and reports JSON. `otd-cli mcp` gives AI agents the same commands, limited to the folders they're given.

## Key domain concepts
Concepts come from catalogue §2. Shared contracts are defined in E01–E04.

**Sessions and views**
- A **session** is one comparison task of a given type (Folder Compare, Text Compare, Text Merge and so on). It holds **specs** (one input per side) and **rules** (that type's settings).
  - Sessions can be named, auto-saved, locked or shared, and each type has default settings.
- A **view** is a session, or the Home launcher, shown in a tab. A **workspace** saves a whole set of windows, tabs and their sessions.
- A **child session** is a file comparison opened from a folder session. It reports its result back, and the parent can remember settings for that file pair.

**Rules and formats**
- **Rules** decide what counts as a difference and whether it is **important** (red) or **unimportant** (blue, or hidden by *Ignore Unimportant Differences*).
- A **file format** is global and ordered, and the first enabled format whose mask matches a file wins. It decides the viewer, any conversion, the **grammar**, tab stops and line weights.
  - Formats define syntax. Session rules decide which syntax elements matter. That is the "rules vs file formats" split.
- **Replacements** and **alignment overrides** make chosen cross-side differences unimportant, or force two items to line up.

**Comparison terms**
- An **orphan** exists on one side only. A **section** is a run of differing lines. A **conflict** is a 3-way change made differently on both sides.
- **Tolerance** (time, number, date, pixel) and **skew tolerance** (alignment search range) bound what is treated as matching.
- **Filters** come in three kinds: **file filters** (name or other criteria), **display filters** (by comparison status) and **folder display modes**.

**Remote sources and snapshots**
- A **profile** holds named credentials and settings for a remote service.
- A **snapshot** is a compressed, read-only listing of a folder tree. It can include CRCs, and it compares later like a folder.

## Release plan
- **Release 1 — MVP: all 41 Must stories.** The setup story (E27-S1) comes first. Then it delivers a credible first release:
  - comparing, editing and saving two text files, with importance rules including ignoring comments by grammar;
  - comparing two folder trees with filters and CRC, binary or rules-based content tests;
  - copying, moving, deleting and synchronising items;
  - opening file pairs from a folder comparison;
  - zip archives as folders;
  - saved sessions with defaults;
  - light and dark themes;
  - acting as a VCS diff tool with exit codes;
  - installers for all three OSes.
- **Release 2 — Power-user features: all Should stories.** Folder Sync, 3-way Text and Folder Merge, Table, Hex and Picture Compare, reports, snapshots, scripting, remote profiles, customisation, settings portability, and the command-line automation tool with MCP for agents.
- **Release 3 — Full feature set: all Could stories.** Text Edit and Text Patch, Media, Version and Registry Compare, OS integrations, and advanced or platform-specific options.

Within a release, stories are built layer by layer. A story never depends on another story of its own layer, so each layer can be built in parallel (see the index).

## Non-functional requirements (proposed by the PO; confirm before Release 1)
- **Stack** (decided): Rust and egui. OverTheDiff is a Craft app, so the shared rules in `AGENTS.md` apply (E01).
- **Platforms** (decided, E01-S2-T1): Windows 10 and 11 on x64 and ARM64; macOS, as one universal build for Apple silicon and Intel on the two newest major versions; and Linux desktop on x64, as an AppImage and a `.deb` (the AppImage waits on a licence question, #79). Windows gets HiDPI and per-monitor scaling. The executable is `otd` (E01-S2).
- **Performance**: the text engine budgets set in E03-S1-T6: a 100,000-line pair shows its first rows within 1 s and completes within 10 s, a single-line edit in a 10,000-line pair recompares within 100 ms, and a 100,000-line comparison takes at most 64 MiB beyond its inputs. Folder scans and comparisons run in the background with progress and never freeze the UI (E01-S1, E03-S2).
- **Data safety**:
  - confirmations before destructive operations;
  - optional backups before overwrites;
  - the recycle bin where available;
  - read-only sources refuse writes before changing anything;
  - settings writes are atomic.
- **Accessibility**: every command is reachable from the keyboard and rebindable (E01-S1, E22-S1), with no keyboard trap. Status is never conveyed by colour alone: every view pairs colour with spots, icons or status text. Text has 4.5:1 contrast and controls 3:1. Drawn panes expose their text, caret, selection and status to screen readers, and a UI harness checks this (E27-S1).
- **Automation**: everything the app does also works without the window, through `otd-cli` and MCP (E28). Partial or unsupported behaviour says so in results.
- **Localisation**: all user-facing text goes through the translation system with English as the key (E01-S1). Translations are added in storytold/craft-i18n.
- **Security**: saved credentials and keys are handled carefully, and where they are stored is an open decision below.

## Non-goals / out of scope
- **Web Resources panel** on Home. Its content isn't specified (Q20). It is not built.
- **Licensing, editions, admin policies, update checks and in-app support** (catalogue §29). OverTheDiff is fully open source: one edition with every feature, no trial, licence keys or Pro tier.
- **Website, online store and add-on downloads** are outside this backlog. The "HTML tables" format ships built in rather than as an add-on (E15-S4).
- **Ideas outside the catalogue** are tracked in the index's Roadmap appendix and not scheduled: modern Git integration in folder views, Finder and Nautilus extensions, a built-in scheduler, nested grammars, and settings sync.
- **Importing snapshots or settings from third-party comparison tools**, unless decided otherwise in Q10 and Q11.

## Open decisions
Each decision sits in its owning story as task T1 with a proposed default. The index lists every question with its owner.

**Business and scope**
- **Registry Compare in scope at all** (Q19, E25-S4).
- **Source-control integration** (Q12, E26-S4): legacy SCC/MSSCCI support versus modern Git, SVN or Perforce integration.
- **Shell integration and clipboard history beyond Windows** (Q13 E26-S1, Q14 E26-S3).

**File formats and compatibility**
- **Snapshot and settings-package formats** (Q10 E19-S1, Q11 E23-S1): own formats, and whether to import third-party tools' files for migration.
- **Built-in language and converter catalogue** (Q8, E10-S2 and E10-S7).
- **Report XML schema** (Q9, E18-S5).

**Unspecified behaviour** (define)
- Custom sync operations (Q1), backups (Q2), confirmations (Q3), media and version importance (Q4), table column types (Q5), the hex layout dialog (Q6), the unimportance list editor (Q7).
- The "similar" exit code (Q17), the automatic-refresh interval (Q18), picture metadata (Q21), registry settings (Q22), patch application (Q23).

**Product-wide conventions**
- **Default keymap and status-bar content** (Q24, Q25, E01-S1): decided on 2026-11-25. Shortcuts follow each OS's standards plus the catalogue's product shortcuts, and views fill shared status-bar fields. Each view decides its own fields in its story. Text Compare's were decided on 2026-11-25 (Q25, E07-S1): the overall result, cursor line and column, insert/overwrite state and load time, and Text Compare Info lists section and line counts.
- **Scheduler** (Q15, E20-S6): rely on OS schedulers or build one in.
- **Password storage** (PO addition): the OS credential store or locally encrypted storage. The lean is the OS credential store where available. Decision task E21-S1-T7 owns it.

---
*Companion to the backlog. Factual and grounded in the catalogue; spec-derived claims are not yet validated. The epics remain the executable source of truth.*
