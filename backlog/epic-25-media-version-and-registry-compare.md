# E25 — Media, Version and Registry Compare

**Group**: feature
**Label**: `area:specialised-compare`
**Size**: L
**Depends on**: E01, E04 (foundations); E08, E09 (folder views, for E25-S3 and E25-S6); E19, E20 (snapshots and scripts, for E25-S3)
**Source**: catalogue §17, §18, §19, F-7.2, F-8.2, F-8.3, F-23.1, F-24.2. Spec-derived from the feature catalogue, including its Media and Version Compare figures, and not yet validated. Items marked *inferred* are analyst or PO inferences, or details read only from a figure.

This epic covers three specialised comparisons:

- audio tag metadata in FLAC, MP3 and MP4/AAC files;
- the version-information resources of Windows executables;
- the Windows registry, live or as `.reg` files.

All three use one presentation: a Name/Value tree aligned by name, with character-level highlights. They serve QA, release engineering and Windows administration.

## Behavioral contract
- **Data**:
  - *Name/Value tree*: groups of items. Each item has a name, a left value, a right value, a status (same, different or orphan) and an importance.
  - *Media*: the tags of each file, the tag version and the audio data size.
  - *Version*: resource fields in the groups Code, PE Header, Fixed, Translation and Variable.
  - *Registry*: a tree of keys, each holding values (name, type, data). The source is a live registry, local or remote, or a `.reg` file.
  - *Importance settings*: one per field, for Media and Version.
- **Routes/endpoints**:
  - Home and Session > New Session ▸ Media Compare, Version Compare or Registry Compare.
  - The CLI with two media or executable files.
  - Registry specs `reg:\\HKEY_LOCAL_MACHINE\MyKey` and `reg:\\MyComputer\HKEY_USERS\MyKey`.
  - File > Open Registry, and the remote registry selection dialog.
  - Session Settings tabs: Specs, plus Importance for Media and Version.
  - The Folder Compare Version column and Compare versions test, the snapshot Exe versions option, and the script command `criteria version`.
- **Integrations**:
  - Audio tag readers and audio playback.
  - The executable version-resource reader (Windows, including MUI files).
  - The Windows registry: local, or remote through the Remote Registry service.
  - E04-S1: file access.
  - The folder engine and folder views, for E25-S3 and E25-S6.
- **Rules/states**:
  - Items align by name. An orphan shows as a crosshatched gap (*inferred*, Media). Character differences inside a value are highlighted.
  - Item Is Important overrides an item's default importance.
  - These views have no user-editable file-format kind; they're chosen by file type (*inferred*).
  - In Registry Compare, red is a difference and purple an orphan. Each key is coloured by whether it contains differences or orphans.

---

## Stories

### E25-S1 — Media Compare
**Priority**: Could · **Layer**: 1 · **Size**: L · **Platforms**: All
**Needs**: E01-S1 (application shell), E04-S1 (file-system access layer), E02-S1 (file-format resolution)
**Traces**: F-17.1 · **Open questions**: Q4
**Not in this story**: opening pairs from folders and child-session commands → E25-S6; report → E18-S6.

As a QA engineer preparing an audio release, I can compare the tags of two FLAC, MP3 or MP4 files and play them so that I catch wrong metadata before publishing.

**Acceptance criteria**
- [ ] Default importance per field follows the decision in T1 (Q4).
- [ ] Two FLAC, MP3 or MP4/AAC files open in Media Compare from the CLI, from Home, or through Session > New Session ▸ Media Compare. The view is chosen by file type (*inferred*).
- [ ] A Name/Value tree shows both files' tags, side by side or over-under (*inferred*). It has two groups:
  - **Metadata**: Title, Artist, Album, Genre, Track number, Recording year, Album artist, Comment, Private frames, Unique file identifier…
  - **Audio**: Data (byte size).
- [ ] A tag present on one side only shows as a crosshatched gap, and changed characters inside a value are highlighted (*inferred*).
- [ ] The file-info bar shows the timestamp, the size and the tag version, for example ID3v2.3 (*inferred*).
- [ ] Each pane has a play button and a seek bar. Play plays that pane's audio, and Play Both plays both files (*inferred*: at the same time).
- [ ] Item Is Important marks the selected tag item as important or unimportant. The Importance tab keeps the same rules persistently.
- [ ] Display filters are Show All, Show Differences, Show Same and Show None, plus Ignore Unimportant Differences. The toolbar has All, Diffs, Same, Minor, Rules, Next Diff, Prev Diff, Swap, Reload and Play2 (*inferred*); Minor is Ignore Unimportant Differences.
- [ ] Next/Previous Difference Item moves between differing items. Find, Find Next and Find Previous search the tree.
- [ ] Copy and Select All work. View options are Visible Whitespace, display fonts, Side-by-Side and Over-Under, Text Details and File Info.
- [ ] The Specs tab holds the files and a Description. Swap Sides, Reload, Recompare and Media Compare Info all work.

#### Tasks
- **E25-S1-T1** — Decide the default importance of each media field (Q4) · size: S · depends: — · flags: `decision`
  - AC: decision recorded. Proposed default: every Metadata text field and the Audio data size are important; Private frames, Unique file identifier and padding fields are unimportant.
- **E25-S1-T2** — Tag readers for FLAC, MP3 (ID3) and MP4/AAC, including the tag version · size: L · depends: — · flags: `licence-check`, `craft-libs`
  - AC: sample files of each type yield their tags and tag version. A file without tags shows an empty Metadata group.
- **E25-S1-T3** — Name/Value tree aligned by name, with orphan gaps and character highlights · size: M · depends: E25-S1-T2 · flags: `naming-check`
  - AC: a Title that differs by one character highlights only that character. A tag present on one side only faces a crosshatched gap.
- **E25-S1-T4** — Importance: Item Is Important, the Importance tab and Ignore Unimportant Differences · size: M · depends: E25-S1-T1, E25-S1-T3
  - AC: once Comment is marked unimportant, its difference shows as unimportant, and Ignore Unimportant Differences hides it.
- **E25-S1-T5** — Display filters, navigation, Find, Copy, Select All, view options, Specs, and Swap/Reload/Recompare/Info · size: M · depends: E25-S1-T3
  - AC: Show Differences hides items that are the same. Next Difference Item visits each differing item exactly once.
- **E25-S1-T6** — Playback: Play and Play Both, with seek bars · size: M · depends: — · flags: `licence-check`
  - AC: Play starts the pane's audio and its seek bar moves. Seeking jumps playback to that point, and Play Both starts both files.

### E25-S2 — Version Compare view
**Priority**: Could · **Layer**: 1 · **Size**: L · **Platforms**: Win
**Needs**: E01-S1 (application shell), E04-S1 (file-system access layer), E02-S1 (file-format resolution)
**Traces**: F-19.1 · **Open questions**: Q4
**Not in this story**: Version column, Compare versions, snapshot and script support → E25-S3; child-session commands → E25-S6; report → E18-S6.

As a release engineer, I can compare the version resources of two builds of an executable so that I confirm the version numbers, company strings and headers are what I shipped.

**Acceptance criteria**
- [ ] Default importance per field follows the decision in T1 (Q4).
- [ ] Two exe, dll or ocx files open in Version Compare from the CLI, from Home, or through Session > New Session ▸ Version Compare. The view is chosen by file type (*inferred*).
- [ ] A Name/Value tree has these groups (*inferred*):
  - **Code**: Data size.
  - **PE Header**: Operating System, Processor, Creation Date, Flags, Linker Version, Image Base, Checksum, User Interface, DLL Flags.
  - **Fixed**: File Version, Product Version, Flags, Operating System, File Type.
  - **Translation**: Languages.
  - **Variable**: Language, Company Name, File Description, File Version, Legal Copyright, Legal Trademarks…
- [ ] Version resources stored in MUI (multilingual resource) files are found and compared (*inferred* reading of "better MUI handling").
- [ ] Differences inside a value are highlighted at character level.
- [ ] A file with no version information, or one that can't be read, shows a clear error in its own pane, and the other side still loads (*inferred* reading of "better error handling").
- [ ] An Item Is Important toggle and an Importance tab are available.
- [ ] Display filters are Show All, Show Differences, Show Same and Show None, plus Ignore Unimportant Differences. Next/Previous Difference Item, Find, Copy, Select All, display fonts, layouts, Text Details and File Info all work.
- [ ] The Specs tab holds the files and a Description. Swap Sides, Reload, Recompare and Version Compare Info all work.

#### Tasks
- **E25-S2-T1** — Decide the default importance of each version field (Q4) · size: S · depends: — · flags: `decision`
  - AC: decision recorded. Proposed default: every field is important except PE Header Creation Date and Checksum.
- **E25-S2-T2** — Reader for version resources and PE headers, including MUI files · size: L · depends: — · flags: `licence-check`, `craft-libs`
  - AC: a sample dll yields every specified group. A localised dll's MUI resource is read, and a file with no version resource is reported as such.
- **E25-S2-T3** — Name/Value tree with character-level highlights · size: M · depends: E25-S2-T2 · flags: `naming-check`
  - AC: File Version `1.2.3.4` against `1.2.3.5` highlights only the last digit.
- **E25-S2-T4** — Importance: Item Is Important, the Importance tab and Ignore Unimportant Differences · size: S · depends: E25-S2-T1, E25-S2-T3
  - AC: two rebuilds that differ only in Creation Date compare as having unimportant differences only.
- **E25-S2-T5** — Display filters, navigation, Find, Copy, Select All, view options, Specs, and Swap/Reload/Recompare/Info · size: M · depends: E25-S2-T3
  - AC: Next Difference Item visits each differing field exactly once. Swap Sides exchanges the sides and recompares.

### E25-S3 — Version information in folders, snapshots and scripts
**Priority**: Could · **Layer**: 5 · **Size**: M · **Platforms**: Win
**Needs**: E25-S2 (version reader), E08-S4 (Comparison tab), E08-S6 (columns), E19-S1 (snapshots), E20-S1 (script runner), E18-S2 (Folder Compare report)
**Traces**: F-8.2, F-8.3, F-23.1, F-24.2

As a release engineer auditing a deployment folder, I can see and compare file versions across whole folder trees, snapshots and scripts so that I spot mismatched binaries without opening each pair.

**Acceptance criteria**
- [ ] The Folder Compare column chooser offers a Version column, showing major.minor.maintenance.build from exe and dll headers.
- [ ] The Comparison tab's Compare versions option compares the version resources of `.exe`, `.dll` and `.ocx` files. Differing versions mark the pair as different.
- [ ] Save Snapshot has an Exe versions option, and the script command `snapshot save-version` does the same. A snapshot saved this way records versions, and comparisons against it can use version comparison.
- [ ] The script command `criteria version` turns on version comparison for the commands that follow.

- [ ] The Folder Compare report and the `folder-report` script command accept the `version` column, which shows each file's version.
#### Tasks
- **E25-S3-T1** — Version column · size: S · depends: —
  - AC: the column shows `1.2.3.4` for a dll with that version, and is blank for files without one.
- **E25-S3-T2** — Compare versions criterion · size: S · depends: —
  - AC: with the option on, two dlls with the same size and time but different versions show as different.
- **E25-S3-T3** — Snapshot Exe versions option and script `snapshot save-version` · size: S · depends: —
  - AC: a snapshot saved with versions, compared against the live folder, flags a dll that was replaced.
- **E25-S3-T4** — Script `criteria version` · size: S · depends: E25-S3-T2
  - AC: a script running `criteria version` and then `select diff.files` selects the dll whose version changed.
- **E25-S3-T5** — Version column in folder reports · size: S · depends: E25-S3-T1
  - AC: a folder report with the size and version columns lists each exe and dll with its version, and other files with an empty version.

### E25-S4 — Registry Compare
**Priority**: Could · **Layer**: 1 · **Size**: L · **Platforms**: Win
**Needs**: E01-S1 (application shell), E04-S1 (file access for `.reg` files)
**Traces**: F-18.1, F-7.2 · **Open questions**: Q19, Q22
**Not in this story**: editing and export → E25-S5; report → E18-S6; Compare to Registry from Explorer → E26-S2.

As an IT administrator, I can compare two registries, live or exported as `.reg` files, side by side so that I can find configuration drift between machines.

**Acceptance criteria**
- [ ] Scope and session settings follow the decision in T1 (Q19, Q22).
- [ ] A pane can show a live registry, local or on a remote machine chosen in the remote registry selection dialog, or a `.reg` file. File > Open, Open Registry and Open Registry File on File System load them.
- [ ] The specs `reg:\\HKEY_LOCAL_MACHINE\MyKey` and `reg:\\MyComputer\HKEY_USERS\MyKey` open that key, on the local machine or a remote one.
- [ ] The remote registry dialog lists what the remote machine needs:
  - network discovery and file sharing turned on;
  - the Remote Registry service started and set to Automatic;
  - Windows Firewall allowing Windows Remote Management.
- [ ] Keys and values are sorted and aligned by name. Each key is coloured by whether it contains differences or orphans. Values show their character differences. Red means a difference and purple an orphan.
- [ ] Set as Base Keys, Set Both as Base Keys, Set as Base Key on Other Side, Up One Level (left, right or both), Expand All and Collapse All all work.
- [ ] Display filters Show All, Show Differences, Show Same and Show None work. Next/Previous Difference Item and Find work.
- [ ] Visible Whitespace, display fonts, layouts, Text Details, Hex Details and File Info are available.
- [ ] The session settings have only a Specs tab. Swap Sides, Reload, Recompare and Registry Compare Info all work.

#### Tasks
- **E25-S4-T1** — Confirm scope and session settings (Q19, Q22) · size: S · depends: — · flags: `decision`
  - AC: decision recorded. Proposed default: in scope as a Windows-only Could feature, with no comparison rules (Specs tab only), comparing values by type and data.
- **E25-S4-T2** — Registry sources: live local, live remote, and `.reg` file parsing · size: L · depends: E25-S4-T1 · flags: `licence-check`, `craft-libs`
  - AC: a `.reg` export of a key compares as same with the live key. A remote machine that meets the prerequisites loads. One that doesn't shows a message naming the prerequisite to check (*inferred*).
- **E25-S4-T3** — `reg:` specs and the Open commands · size: S · depends: E25-S4-T2
  - AC: each specified `reg:` form opens the named key.
- **E25-S4-T4** — Key and value tree aligned by name, with colours and character differences · size: M · depends: E25-S4-T2 · flags: `naming-check`, `accessibility`
  - AC: a value present only on the right shows purple. A changed DWORD shows red, with the changed digits highlighted.
- **E25-S4-T5** — Base keys, Up One Level, and Expand All/Collapse All · size: S · depends: E25-S4-T4
  - AC: Set Both as Base Keys on two selected keys makes them the roots of the comparison.
- **E25-S4-T6** — Display filters, navigation, Find, details panels, layouts, Specs, and Swap/Reload/Recompare/Info · size: M · depends: E25-S4-T4
  - AC: Hex Details shows a binary value's bytes. Next Difference Item visits each differing value.

### E25-S5 — Registry editing and export
**Priority**: Could · **Layer**: 4 · **Size**: M · **Platforms**: Win
**Needs**: E25-S4 (Registry Compare), E21-S6 (browse using profiles), E22-S4 (Open With applications)
**Traces**: F-18.1

As an IT administrator, I can copy, change and delete registry keys and values between two registries and export them so that I can fix drift and keep a record of what I changed.

**Acceptance criteria**
- [ ] Copy to Right, Copy to Left and Copy to Other Side copy the selected keys or values to the other registry.
- [ ] Delete, Rename, New Key and New Value work. Modify, or double-clicking a value, changes its type and data.
- [ ] Undo and Redo reverse and reapply changes.
- [ ] Changes to a live registry take effect at once (*inferred*). A `.reg`-file side is read-only (*inferred*).
- [ ] Copy copies the selection as text, and Copy Key Name copies the full key path.
- [ ] Export writes the current key to a `.reg` file, and Export All writes the whole side.
- [ ] Delete asks for confirmation first (*inferred*).
- [ ] A change the user isn't permitted to make, for example in HKLM without admin rights, is refused with the system message, and nothing is half-applied (*inferred*).

- [ ] *File > Open Registry File Using Profile* opens a `.reg` file through a named profile. *Open With* lists the configured Open With items for a `.reg` file side.
#### Tasks
- **E25-S5-T1** — Copy keys and values across registries · size: M · depends: —
  - AC: copying a key also copies its subkeys and values, and the pair then compares as same.
- **E25-S5-T2** — Delete, Rename, New Key, New Value and Modify (type and data) · size: M · depends: —
  - AC: changing a value's type from REG_SZ to REG_DWORD stores the new type. Delete asks before deleting.
- **E25-S5-T3** — Undo/Redo for registry edits · size: M · depends: E25-S5-T1, E25-S5-T2
  - AC: undoing a delete restores the key with all its values.
- **E25-S5-T4** — Copy, Copy Key Name, Export and Export All · size: S · depends: —
  - AC: an exported `.reg` file imports cleanly into the OS registry editor and then compares as same with its source.
- **E25-S5-T5** — Permission failures and read-only `.reg` sides · size: S · depends: E25-S5-T1, E25-S5-T2
  - AC: editing HKLM without the rights to do so shows the system error and leaves the key unchanged.
- **E25-S5-T6** — Open Registry File Using Profile and Open With in the Registry view · size: S · depends: —
  - AC: a `.reg` file opened through an FTP profile loads as a registry side. An Open With item launches with that file's path.

### E25-S6 — Media and Version views as child sessions
**Priority**: Could · **Layer**: 3 · **Size**: S · **Platforms**: All
**Needs**: E25-S1 (Media Compare), E25-S2 (Version Compare view), E08-S10 (open pairs from folder views), E09-S1 (copy to the other side)
**Traces**: F-6.4, F-17.1, F-19.1

As a QA engineer walking through a folder of releases, I can open media and executable pairs from Folder Compare and step through the differing ones so that I review them in sequence.

**Acceptance criteria**
- [ ] Opening an MP3, FLAC or MP4 pair from Folder Compare opens Media Compare as a child session. Opening an exe, dll or ocx pair opens Version Compare (Windows only).
- [ ] In these child sessions, Next/Previous Difference Files opens the parent's next or previous pair with differences.
- [ ] Copy File to Right/Left and Open Next Difference copies the current file to the other side and then opens the next pair with differences.
- [ ] The parent folder row updates with the child session's result.

#### Tasks
- **E25-S6-T1** — Route media and executable pairs from folder views to Media Compare and Version Compare · size: S · depends: —
  - AC: double-clicking an MP3 pair opens Media Compare in a new tab, and the folder row then shows that comparison's result.
- **E25-S6-T2** — Next/Previous Difference Files, and Copy File and Open Next Difference, in both views · size: S · depends: E25-S6-T1
  - AC: from the first differing MP3 pair, Next Difference Files opens the second. Copy File to Right copies the file and moves on to the next differing pair.
