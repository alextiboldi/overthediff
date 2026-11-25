# E19 — Snapshots

**Group**: feature
**Label**: `area:snapshots`
**Size**: L
**Depends on**: E01, E03, E04 (foundations); E08 (Folder Compare); E20 (script runner)
**Source**: catalogue §23, F-7.7, F-25.2, F-28.5, F-24.2. Spec-derived from the feature catalogue and not yet validated. Items marked *inferred* are analyst or PO inferences.

A snapshot is a compressed, read-only listing of a folder tree at a point in time. Later it can be compared like a folder to show what changed. When CRCs were saved, it also catches silent content changes or corruption.

## Behavioral contract
- **Data**:
  - *Snapshot file*: source folder path, creation time and, for every item, the relative path, type, size, modified time and attributes.
  - Optional per-item content: CRC32, link target, expanded archive listings and empty folders.
  - The file format is decided in E19-S1 T1 (Q10).
- **Routes/endpoints** (desktop app, so these are commands, inputs and script commands):
  - *Tools > Save Snapshot*.
  - A snapshot path can be given anywhere a base folder can.
  - CLI positional `<snapshot>`, and dropping a snapshot on the application icon.
  - Script command `snapshot [save-crc] [expand-archives] [follow-symlinks] [include-empty] [no-filters] left|right|path:<p> [output:<target>]`.
- **Integrations**:
  - E04 data-source contract: a snapshot is a read-only, archive-like data source.
  - E03-S2 content tests: CRC only.
  - E08 folder view and filters.
  - E20-S1 script runner.
- **Rules/states**:
  - Snapshots are read-only.
  - Binary and rules-based content comparisons are impossible against a snapshot, because the files don't exist. CRC comparison works only when CRCs were saved.
  - The default name is `<folder>_<YYYY-MM-DD>` with a numeric suffix to avoid overwriting.

---

## Stories

### E19-S1 — Save and compare folder snapshots
**Priority**: Should · **Layer**: 3 · **Size**: L · **Platforms**: All
**Needs**: E08-S3 (name filters), E04-S2 (zip archives), E20-S1 (script runner), E05-S3 (drop on the application icon), E08-S5 (CRC and binary content comparison)
**Traces**: F-23.1, F-23.2, F-7.7, F-25.2, F-28.5, F-24.2 · **Open questions**: Q10
**Not in this story**: Exe versions option and the script flag `save-version` → E25-S3.

As an IT administrator, I can save a point-in-time listing of a folder tree and later compare the live folder against it so that I can see what changed and, with CRCs, detect silent corruption.

**Acceptance criteria**
- [ ] *Tools > Save Snapshot* opens a dialog for the folder path. Opened from a folder view, it offers the **Left** or **Right** base folder instead.
- [ ] Dialog options:
  - **CRCs**: CRC32 of contents.
  - **Archive contents**: expanded listings of archives.
  - **Empty folders**, including fully filtered ones.
  - **Symbolic link targets**.
  - **Items excluded by file filters**: from a folder view only. When off, the session's filters apply.
- [ ] A Save As prompt follows, with the default name `<folder>_<YYYY-MM-DD>` and a numeric suffix when that name exists.
- [ ] The snapshot file is compressed and read-only. It records every file's name, timestamp, size and attributes, plus whatever the chosen options add.
- [ ] A snapshot loads anywhere a base folder can, and shows as a read-only, archive-like folder tree.
- [ ] A snapshot source reports file content as unreadable, so every content test other than saved CRCs reports "unavailable". This is checked with Binary comparison and with the script command `compare binary`. CRC comparison works when CRCs were saved and flags files whose content changed. Without saved CRCs, content comparison is reported as unavailable (*inferred* message).
- [ ] A snapshot remembers its source folder. Opening it from the CLI (`otd <snapshot>`), or dropping it on the application icon, opens a Folder Compare of the snapshot against that folder.
- [ ] The script command `snapshot [save-crc] [expand-archives] [follow-symlinks] [include-empty] [no-filters] left|right|path:<p> [output:<target>]` saves a snapshot headlessly, using the same default naming.

#### Tasks
- **E19-S1-T1** — Decide the snapshot file format (Q10) · size: S · depends: — · flags: `decision`
  - AC: decision recorded in this story. Proposed default: the product's own documented, compressed format with a version header and its own extension. Importing snapshot files made by third-party tools stays a separate, optional roadmap item.
- **E19-S1-T2** — Snapshot writer: listing, compression and the options (CRCs, archive contents, empty folders, link targets, excluded items) · size: L · depends: E19-S1-T1 · flags: `licence-check`, `craft-libs`
  - AC: a snapshot of a fixture with every option on contains the expected entries. With *Items excluded by file filters* off, filtered-out items are absent.
- **E19-S1-T3** — Save Snapshot dialog with default naming · size: S · depends: E19-S1-T2 · flags: `naming-check`
  - AC: from a folder view, Left or Right fills in the path. The default name uses today's date and gains a numeric suffix when the name already exists.
- **E19-S1-T4** — Snapshot as a read-only data source · size: M · depends: E19-S1-T1
  - AC: a snapshot path typed into a Folder Compare path edit loads as a tree. Write operations against it are refused as read-only.
- **E19-S1-T5** — Content comparison against a snapshot (CRC only) · size: S · depends: E19-S1-T4
  - AC: with saved CRCs, a CRC comparison flags a file whose content changed while its size and date stayed the same. Binary comparison and the script command `compare binary` against a snapshot side report "unavailable".
- **E19-S1-T6** — Compare with the source folder automatically from the CLI or a drop on the application icon · size: S · depends: E19-S1-T4
  - AC: `otd <snapshot>` opens a Folder Compare with the snapshot on one side and its source folder on the other (*inferred* order: snapshot on the left). If the source folder no longer exists, the user is told (*inferred*).
- **E19-S1-T7** — Script `snapshot` command · size: S · depends: E19-S1-T2
  - AC: `snapshot save-crc left output:"<folder>"` writes a snapshot of the left base folder, with CRCs, into that folder under the default name. With a full file path, it writes exactly that file.
