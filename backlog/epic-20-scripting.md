# E20 — Scripting

**Group**: feature
**Label**: `area:scripting`
**Size**: L
**Depends on**: E01, E02, E03, E04 (foundations); E06 (sessions), E08 (folder criteria), E19 (snapshots), E21 (remote sources) for full scripts
**Source**: catalogue §24 and F-25.2–F-25.4. Spec-derived from the feature catalogue and not yet validated. Items marked *inferred* are analyst or PO inferences.

Scripting runs comparisons and file operations without a window. A script loads folders into an invisible folder session, then filters, selects, compares, copies, syncs and reports. Teams use it to automate backups, deployments and audit reports, from the command line or an OS scheduler.

## Behavioral contract
- **Data**:
  - *Script syntax*: a text file with one command per line. Commands are case-insensitive, and blank lines are ignored. `#` starts a comment. Arguments are separated by spaces, and an argument with spaces is quoted. `&` at the end of a line continues the command on the next line.
  - *Script variables*:
    - `%1`–`%9`: CLI positional arguments, not counting the script name or switches.
    - `%ENVVAR%`: an environment variable; the name is case-sensitive.
    - `%date%` (yyyy-mm-dd), `%time%` (locale format) and `%fn_time%` (hh-mm-ss).
    - `lt` and `rt` stand for `left` and `right`.
  - *Script session state*: base folders, criteria, filters, expansion, selection, log target and level, and options (stop-on-error, confirm).
  - *Exit codes*: 105 (error loading the script file), 106 (script syntax error), 107 (script failed to load folders or files).
- **Routes/endpoints** (desktop app, so these are the CLI form, switches, a status window and tweaks):
  - CLI `@<script> [args] [/switches]`, with a `-` prefix for switches on macOS and Linux.
  - Switches `/silent` and `/closescript`.
  - The Scripting Status window and its taskbar entry.
  - Tweaks > Scripts: Beep when finished, Close when finished, Shared scripts folder.
- **Integrations**:
  - E03-S2 folder engine runs the invisible Folder Compare session.
  - E04 data sources supply any folder: local, network, an archive, remote or a snapshot.
  - Reports and snapshots add their own commands to the runner.
  - The OS scheduler starts scripts on a schedule.
- **Rules/states**:
  - File operations act on the selection.
  - A folder must be expanded before its contents can be selected.
  - Failing to load a base folder ends the script with exit code 107.
  - With `/silent`, no window or taskbar entry appears, and any dialog the script would raise becomes a logged error.
  - `beep` beeps even with `/silent`.

---

## Stories

### E20-S1 — Script runner and core commands
**Priority**: Should · **Layer**: 1 · **Size**: L · **Platforms**: All
**Needs**: E01-S1 (application shell and argument dispatch), E03-S2 (folder comparison engine), E04-S1 (file-system access layer)
**Traces**: F-24.1, F-24.2, F-25.2, F-25.3, F-25.4
**Not in this story**: `option confirm:` and file operations → E20-S2; `filter` → E20-S3; `sync` → E20-S4; `load <session>`, `load <default>`, full criteria and rules-based compare → E20-S5; scheduling → E20-S6; report commands → E18-S1, E18-S2, E18-S4, E18-S6; `snapshot` → E19-S1.

As an automation/power user, I can run a script file from the command line that loads two folders, selects items and compares them, with no window, so that repetitive checks run unattended.

**Acceptance criteria**
- [ ] `@<script> [args] [/switches]` runs the script over an invisible Folder Compare session and opens no comparison view. On macOS and Linux, switches also take a `-` prefix.
- [ ] Syntax: one command per line; commands are case-insensitive; blank lines are ignored; `#` starts a comment; arguments are separated by spaces and quoted when they contain spaces; `&` at the end of a line continues the command on the next line.
- [ ] Variables expand before a command runs:
  - `%1`…`%9` are the CLI positional arguments after the script name, not counting switches;
  - `%ENVVAR%` is an environment variable, with a case-sensitive name;
  - `%date%` gives yyyy-mm-dd, `%time%` the locale time and `%fn_time%` hh-mm-ss;
  - `lt` and `rt` work anywhere `left` and `right` do.
- [ ] `load [create:(all|left|right)] <left> [<right>]` loads base folders with the current settings. `create:` creates missing base folders. If a base folder fails to load, the script ends with exit code 107.
- [ ] `expand all`, `expand <path>…`, `collapse all` and `collapse <path>…` change which folders are expanded. Paths are relative to the base and take no wildcards.
- [ ] `select <mask>…` builds the selection as a union of masks. A mask is one of:
  - `all`;
  - `[(left|right|all).][(exact|diff|newer|older|orphan|all).][(files|folders|all)]`;
  - `empty.folders`.
  
  `diff` means the items differ in criteria other than timestamps. A folder is selected only if it holds nothing but the given result type.
- [ ] `criteria [timestamp[:<n>sec]] [size|CRC|binary]` replaces all comparison criteria at once.
- [ ] `compare [CRC|binary]` compares the contents of the selection once. It uses the last content type when none is given.
- [ ] `log [none|normal|verbose] [[append:]<file>]` controls logging. Logging is off by default. The default file is `Log.txt` in the current folder, and `append:` adds to an existing file instead of replacing it.
- [ ] `option stop-on-error` stops the script and prompts at the first error. `beep` beeps, even with `/silent`.
- [ ] A Scripting Status window and a taskbar entry show progress and errors. `/silent` hides both and logs any dialog as an error. `/closescript` closes the window when the script ends, overriding the tweak.
- [ ] Tweaks > Scripts has *Beep when finished*, *Close when finished* and *Shared scripts folder*. A script name with no path that isn't in the current folder is looked up in the shared scripts folder.
- [ ] A missing or unreadable script file exits with 105. A syntax error exits with 106 and reports the line number.

- [ ] `@script` is registered with the run-in-launching-process launch mode. The script runs in the launching process even when an instance is already running, and the process exits with the script's result code.
#### Tasks
- **E20-S1-T1** — Script parser: lines, comments, quoting, continuation, case-insensitivity and variables · size: M · depends: — · flags: `licence-check`, `craft-libs`
  - AC: a fixture that uses every syntax rule parses into the expected command list. An unknown command exits with 106 and reports its line number.
- **E20-S1-T2** — Invisible folder session with `load` (including `create:`) and exit code 107 · size: M · depends: E20-S1-T1
  - AC: loading two existing folders succeeds. `load create:right a b` with `b` missing creates `b`. A missing base folder without `create:` exits with 107.
- **E20-S1-T3** — `expand`, `collapse` and the `select` masks · size: M · depends: E20-S1-T2
  - AC: `select left.orphan.files` on a fixture selects exactly the left orphan files. `select left.newer.folders` doesn't select a folder whose contents have mixed results.
- **E20-S1-T4** — `criteria` (timestamp[:Nsec], size, CRC, binary) and `compare` (CRC, binary) · size: M · depends: E20-S1-T2
  - AC: with `criteria timestamp:2sec`, a 1-second difference counts as same. `compare binary` marks pairs with identical content as same.
- **E20-S1-T5** — `log`, `option stop-on-error` and `beep` · size: S · depends: E20-S1-T1
  - AC: `log verbose append:"x.txt"` adds detailed lines to `x.txt`. With `option stop-on-error`, a failing command halts the script.
- **E20-S1-T6** — Scripting Status window, taskbar entry, `/silent`, `/closescript`, Scripts tweaks, shared scripts folder and exit code 105 · size: M · depends: E20-S1-T1 · flags: `naming-check`
  - AC: with `/silent`, nothing appears on screen and a would-be dialog is logged. `@name.txt` that isn't in the current folder runs from the shared scripts folder. A missing script exits with 105.

### E20-S2 — Script file operations
**Priority**: Should · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E20-S1 (script runner), E04-S2 (zip archives), E02-S1 (masks and regular expressions)
**Traces**: F-24.2

As a release engineer, I can copy, move, delete, rename and retime selected items from a script so that deployment and clean-up steps run without manual work.

**Acceptance criteria**
- [ ] `option confirm:(prompt|yes-to-all|no-to-all)` sets how later commands answer confirmations. The default is `prompt`.
- [ ] `copy (left->right|right->left)` copies the selection, and `move (left->right|right->left)` moves it.
- [ ] `delete [recyclebin=(yes|no)] (left|right|all)` deletes the selection. The recycle bin applies to local drives only.
- [ ] `copyto [left|right|all] [path:(relative|base|none)] <path>` and `moveto [left|right|all] [path:(relative|base|none)] <path>` copy or move the selection to a folder or an archive. The defaults are `all` and `path:none`, and a missing archive target is created.
- [ ] `rename [regexpr <old>] <new>` renames the selection. It uses DOS-style masks by default, or a regular expression with `$n` templates when `regexpr` is given.
- [ ] `touch (left->right|right->left)` copies timestamps across. `touch (left|right|all):(now|<timestamp>)` sets them. Both need a selection.
- [ ] On Windows, `attrib (+|-)<[a][s][h][r]> […]` sets or clears DOS attributes on the selection.
- [ ] A command run with an empty selection does nothing and logs a warning (*inferred*). Errors follow `option stop-on-error`.

#### Tasks
- **E20-S2-T1** — `option confirm:` · size: S · depends: —
  - AC: with `no-to-all`, a copy that would overwrite asks nothing and skips the overwrites. With `yes-to-all`, it overwrites.
- **E20-S2-T2** — `copy`, `move` and `delete` · size: M · depends: E20-S2-T1
  - AC: `delete recyclebin=yes left` moves local files to the recycle bin. `move left->right` leaves no source files behind.
- **E20-S2-T3** — `copyto` and `moveto`, with path options and archive targets · size: M · depends: E20-S2-T1 · flags: `licence-check`
  - AC: `copyto left path:relative out.zip` creates `out.zip` holding the selected files at their relative paths.
- **E20-S2-T4** — `rename`, `touch` and `attrib` · size: M · depends: —
  - AC: `rename regexpr "(.*)\.txt" "$1.bak"` turns `.txt` names into `.bak`. `touch all:now` sets the time on both sides. On Windows, `attrib +r` sets read-only.

### E20-S3 — Script filters
**Priority**: Should · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E20-S1 (script runner), E02-S1 (masks)
**Traces**: F-24.2

As an automation/power user, I can limit a script to the files that matter by name, date, size or type so that later commands act only on them.

**Acceptance criteria**
- [ ] `filter <masks>` sets name filters. `-` excludes, and a trailing `\` means a folder.
- [ ] `filter cutoff:([<|>](<timestamp>|<n>days)|none)` keeps files older (`<`, the default) or newer (`>`) than a timestamp or a number of days. `none` clears the cutoff. Dates are parsed leniently, and the locale format is recommended.
- [ ] `filter size:([<|>]<n>[KB|MB|GB|TB]|none)` keeps files smaller or larger than a size. `none` clears the limit.
- [ ] On Windows, `filter attrib:((+|-)<[a][c][e][h][i][l][o][p][r][s][t][u][z]>|none)` filters by DOS attributes. `+` keeps only items with the attribute and `-` only items without it (*inferred*).
- [ ] On Windows, `filter exclude-protected` excludes protected operating-system files and `filter include-protected` includes them.
- [ ] `filter unixtype:((+|-)<[b][c][l][p][r][s]>|none)` filters by Unix file type.
- [ ] Items that a filter excludes can't be selected, and `expand all` skips them.

#### Tasks
- **E20-S3-T1** — `filter` with masks · size: S · depends: —
  - AC: after `filter "*.c;-test\"`, `select all` selects only `.c` files that aren't in a folder named `test`.
- **E20-S3-T2** — `cutoff:` and `size:` filters · size: M · depends: —
  - AC: `filter cutoff:<7days` keeps only files modified more than 7 days ago. `filter size:>1MB` keeps only files over 1 MB.
- **E20-S3-T3** — `attrib:`, `exclude-protected` and `include-protected` (Windows), and `unixtype:` · size: M · depends: —
  - AC: on Windows, `filter attrib:-h` leaves out hidden files. On macOS and Linux, `filter unixtype:-l` leaves out symbolic links.

### E20-S4 — Script sync
**Priority**: Should · **Layer**: 2 · **Size**: S · **Platforms**: All
**Needs**: E20-S1 (script runner)
**Traces**: F-24.2

As an IT administrator, I can synchronise two folders from a script so that backups and mirrors run unattended.

**Acceptance criteria**
- [ ] `sync [visible] [create-empty] (update|mirror):(left->right|right->left|all)` synchronises the loaded base folders and ignores the selection.
- [ ] `update:left->right` copies newer and orphan items to the right, and `update:all` copies in both directions.
- [ ] `mirror:left->right` makes the right side match the left, including deleting right orphans.
- [ ] `visible` limits the sync to files in expanded folders. `create-empty` also creates empty folders.
- [ ] `mirror:all` is rejected as a syntax error, with exit code 106.

#### Tasks
- **E20-S4-T1** — `sync` with the update and mirror directions · size: S · depends: —
  - AC: after `sync mirror:left->right`, the right tree compares as identical to the left.
- **E20-S4-T2** — `visible`, `create-empty` and rejecting `mirror:all` · size: S · depends: E20-S4-T1
  - AC: with `visible`, files in collapsed folders are left untouched. `mirror:all` fails with exit code 106.

### E20-S5 — Script sessions, full criteria and sample scripts
**Priority**: Should · **Layer**: 5 · **Size**: M · **Platforms**: All
**Needs**: E20-S2 (script file operations), E20-S3 (script filters), E20-S4 (script sync), E06-S1 (saved sessions), E06-S2 (session defaults), E08-S4 (quick-test criteria), E08-S13 (rules-based content comparison), E08-S17 (Windows file details), E19-S1 (snapshots), E21-S1 (FTP sites), E20-S1 (script runner), E18-S2 (folder report), E06-S4 (sessions remember display filters)
**Traces**: F-24.2
**Not in this story**: `criteria version` → E25-S3; report commands and the sample scripts that run them → E18-S1, E18-S2.

As an IT administrator, I can drive scripts from saved sessions and use every comparison criterion so that the catalogue's automation recipes work as written.

**Acceptance criteria**
- [ ] `load <session>` loads a saved Folder Compare session with all its settings except the display filter.
- [ ] `load <default>` loads the Folder Compare defaults.
- [ ] The full form `criteria [attrib:<set>] [timestamp[:[<n>sec][;IgnoreDST]]] [size|CRC|binary|rules-based] [timezone:(ignore|(left|right)(+|-)<0..12>)] [follow-symlinks] [ignore-unimportant] [owner] [group] [permissions]` replaces all criteria at once. For example: `criteria attrib:sh timestamp:2sec;IgnoreDST rules-based timezone:left+6`. Attributes apply on Windows only.
- [ ] `compare rules-based` compares the contents of the selection using the session rules.
- [ ] Sample script 1 (`load "Sync Laptop"`, then `sync update:left->right`) updates the right side from a saved session.
- [ ] Sample script 2, a mirror to an FTP site with logging (`load <default>`, a `filter`, then `sync mirror:left->right`), mirrors a local folder to an FTP site.
- [ ] Sample script 5, a web-site sync (`criteria timestamp size`, a filter that excludes `CVS\`, then `sync create-empty mirror:left->right`), gives the specified result.
- [ ] Sample script 7, a snapshot delta, produces the delta archive and a new snapshot. It runs `option confirm:yes-to-all`, `criteria crc`, loads a folder against a snapshot, then runs `expand all`, `select left.diff.files left.orphan.files`, `copyto left path:relative x.zip` and `snapshot save-crc left output:…`.

- [ ] Sample script 6 runs unmodified. It is a rules-based summary report logged to a dated file: `log verbose append:"…%date%.txt"`, rules-based criteria, then `folder-report layout:summary`. It writes the report and the dated log.
- [ ] Filters set with `filter` limit what `sync` synchronises (*inferred*).
#### Tasks
- **E20-S5-T1** — `load <session>` and `load <default>` · size: S · depends: —
  - AC: loading a session that has a name filter and binary criteria applies both, but not the session's display filter.
- **E20-S5-T2** — Full `criteria` options and `compare rules-based` · size: M · depends: —
  - AC: `timezone:left+6` treats a 6-hour offset as same. `ignore-unimportant` with `rules-based` treats differences that are only whitespace as same. On Windows, `attrib:sh` compares the system and hidden attributes.
- **E20-S5-T3** — Sample scripts 1, 2, 5 and 7 as end-to-end acceptance tests · size: M · depends: E20-S5-T1, E20-S5-T2
  - AC: each sample runs headlessly against fixture folders, with a test FTP server for sample 2, and gives the specified outcome with exit code 0 (*inferred* success code).
- **E20-S5-T4** — Sample script 6 end to end · size: S · depends: E20-S5-T2
  - AC: after the run, the summary report and the dated log file both exist, and the log holds verbose entries for the run.

### E20-S6 — Scheduled scripts
**Priority**: Could · **Layer**: 2 · **Size**: S · **Platforms**: Win
**Needs**: E20-S1 (script runner)
**Traces**: F-24.3 · **Open questions**: Q15

As an IT administrator, I can run a script on a schedule so that synchronisations and reports happen without anyone starting them.

**Acceptance criteria**
- [ ] The decision between a built-in scheduler and the OS scheduler is recorded in T1.
- [ ] The help documents running a script from Windows Task Scheduler, with Program set to the application's executable, Arguments to `@Sync.txt` and Start in to the script's folder.
- [ ] The documentation recommends a `log` command and points to the scheduler's own error log.
- [ ] A script that the scheduler starts with `/silent` runs with no visible window and returns its exit code to the scheduler.

#### Tasks
- **E20-S6-T1** — Decide between a built-in scheduler and the OS scheduler (Q15) · size: S · depends: — · flags: `decision`
  - AC: decision recorded in this story. Proposed default: no built-in scheduler; document Windows Task Scheduler, and add the macOS and Linux equivalents if this story is extended to those platforms.
- **E20-S6-T2** — Task Scheduler guide and a verified scheduled run · size: S · depends: E20-S6-T1
  - AC: following the guide, a scheduled `@Sync.txt` run with `/silent` and a `log` command starts at the set time and writes its log.
