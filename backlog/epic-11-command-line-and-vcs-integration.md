# E11 — Command line and VCS integration

**Group**: feature
**Label**: `area:command-line`
**Size**: L
**Depends on**: E01, E03, E04 (foundations); E07 (Text Compare view), E08 (Folder Compare view), E13 (Text Merge view)
**Source**: catalogue §25, F-28.3, F-7.2. Spec-derived from the feature catalogue and not yet validated. Items marked *inferred* are analyst or PO inferences.

This epic makes the product usable from terminals, build scripts and version-control tools. It covers launching comparisons with switches, waiting for them to close, quick headless comparisons and merges, and meaningful exit codes. That is what lets a developer set OverTheDiff as git's difftool and mergetool.

## Behavioral contract
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

---

## Stories

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

### E11-S3 — Folder comparison switches
**Priority**: Should · **Layer**: 3 · **Size**: S · **Platforms**: All
**Needs**: E08-S3 (name filters)
**Traces**: F-25.3

As an automation/power user, I can open a folder comparison that is already filtered and expanded from the command line so that a launcher script shows exactly the files I care about.

**Acceptance criteria**
- [ ] `/filters=<masks>` sets the initial name filter of the folder comparison it opens. Masks are `;`-separated and follow the name-filter syntax: `-` excludes, and a trailing `\` means a folder. A value that contains spaces works when quoted.
- [ ] The initial filter shows in the toolbar Filters edit and can be changed afterwards like any other filter.
- [ ] `/expandall` expands every subfolder in the initial folder comparison.
- [ ] When the comparison opened isn't a folder comparison, both switches are ignored and a warning is shown (*inferred*).

#### Tasks
- **E11-S3-T1** — `/filters` switch applying the initial name filter · size: S · depends: —
  - AC: launching two folders with `/filters="*.c;-test\"` shows only `.c` files that aren't in a folder named `test`.
- **E11-S3-T2** — `/expandall` switch · size: S · depends: —
  - AC: with `/expandall`, the comparison opens with every folder expanded. Without it, folders open collapsed.

### E11-S4 — Compare standard input
**Priority**: Should · **Layer**: 2 · **Size**: S · **Platforms**: All
**Needs**: E07-S1 (Text Compare view)
**Traces**: F-7.2, F-25.2

As a developer, I can pipe command output into a comparison so that I can compare generated text without saving it to a file first.

**Acceptance criteria**
- [ ] The positional argument `-` reads standard input into a pane. For example, `dir | <main command> - other.txt` compares the directory listing with `other.txt`.
- [ ] With `-` as the only argument, standard input loads into the left pane and the right pane starts empty (*inferred*).
- [ ] The view is chosen as for a file: text content opens in Text Compare. Because there's no file name, the format is detected from the content (*inferred*).
- [ ] The path edit of the standard-input pane shows a label for standard input (*inferred*). Saving that pane asks for a file name.
- [ ] Empty standard input gives an empty pane, not an error.

- [ ] Standard input is read in the launching process and attached to the forwarded request, so `-` works with an instance already running.
#### Tasks
- **E11-S4-T1** — Read standard input for a `-` argument into an in-memory source · size: S · depends: —
  - AC: piping 1 MB of text produces a pane whose content is byte-for-byte identical to the input.
- **E11-S4-T2** — Choose the view and present the standard-input pane · size: S · depends: E11-S4-T1
  - AC: `- other.txt` opens Text Compare with standard input on the left. Save on that pane prompts for a file name.

### E11-S5 — Headless automatic merge for version control
**Priority**: Should · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E13-S1 (Text Merge view), E11-S1 (waiting launcher)
**Traces**: F-25.3, F-25.4, F-28.3, F-25.2
**Not in this story**: `/favorleft`, `/favorright` and `/iu` (`/ignoreunimportant`) → E13-S5.

As a developer, I can let OverTheDiff merge files automatically when git calls it as a mergetool so that only real conflicts need my attention.

**Acceptance criteria**
- [ ] `/automerge` with the positional arguments `<left> <right> <center> <output>` merges without interaction when there are no conflicts. It writes the output and exits with 0 (*inferred* code for success).
- [ ] `/mergeoutput=<path>` sets the merge output in place of the fourth positional argument. `/center=<file>` sets the center (ancestor) file explicitly.
- [ ] When conflicts remain and neither `/force` nor `/reviewconflicts` is given, nothing is written and the exit code is 101 (Conflicts detected, merge output not written).
- [ ] `/force` with `/automerge` writes the output with CVS-style markers (`<<<<<<<`, `=======`, `>>>>>>>`) around each conflict and exits with 14 (Conflicts detected). The code mapping and marker labels are *inferred*.
- [ ] `/reviewconflicts` with `/automerge` opens Text Merge when conflicts remain. The launcher exits with 0 when the user saves the merge output, and with 101 when Text Merge closes without saving (*inferred* codes).
- [ ] The usage text, or a guide shipped with the product, includes a working `git mergetool` configuration. In a test repository, a merge with one conflict run through it exits with 101 and leaves the output file untouched.

- [ ] `/automerge` runs in the launching process (run-in-launching-process). With `/reviewconflicts` and conflicts remaining, it hands off to forward-and-wait: Text Merge opens in the running instance, and the launcher exits after it closes.
#### Tasks
- **E11-S5-T1** — `/automerge` with a positional output, `/mergeoutput` and `/center` · size: M · depends: —
  - AC: a merge with only non-conflicting changes writes the expected output and exits with 0, with no window shown.
- **E11-S5-T2** — Conflict outcomes: 101 by default; markers and 14 with `/force` · size: M · depends: E11-S5-T1
  - AC: with one conflict, the default run writes nothing and exits with 101. With `/force`, the output holds one marker block and the run exits with 14.
- **E11-S5-T3** — `/reviewconflicts` opens Text Merge when conflicts remain · size: S · depends: E11-S5-T1
  - AC: with one conflict, Text Merge opens on the conflict in the running instance. Saving the output makes the launcher exit with 0, and closing without saving gives 101. With no conflicts, no window opens.
- **E11-S5-T5** — git mergetool guide · size: S · depends: E11-S5-T2
  - AC: in a test repository, the specified configuration resolves a non-conflicting merge with no window opened.
