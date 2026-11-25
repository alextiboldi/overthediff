# E04 — Data sources

**Group**: foundation
**Label**: `area:sources`
**Size**: L
**Depends on**: E01 (application shell, for the stories that have UI); E07 and E08 views for the archive and web stories
**Source**: catalogue §7.1–§7.3, F-7.2 (web), F-9.5 (recycle bin), F-26.12, §2 (spec kinds). Spec-derived from the feature catalogue and not yet validated. Items marked *inferred* are analyst or PO inferences.

This epic covers where comparison inputs come from: local disks and network shares first, then archives, web pages and portable devices. All of them sit behind one data-source contract, so every view, operation and script treats them the same way. Remote services (FTP, cloud, Subversion) reuse the same contract in E21, and snapshots in E19.

## Behavioral contract
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

---

## Stories

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
- **E04-S1-T1** — Contract definition, in-memory reference source and contract test suite · size: M · depends: — · status: done
  - AC: the suite runs against the in-memory source with every case passing, including a container whose children list like a folder's. A deliberately broken fake fails the right cases.
  - Implemented: the `DataSource` trait (list, stat, read, write, create folder, delete permanently or to the recycle bin, rename, set modified time and permissions, with per-item errors and reported capabilities) and a cross-source `copy` that keeps modified times, the in-memory reference source `MemorySource` (trees with sizes, times, links, unreadable items, source-provided CRC-32s, attributes and expandable containers) and the contract suite `otd_source::suite::run`, in the new UI-free crate `crates/source` (`otd-source`, no dependencies). `crates/source/tests/it/contract.rs` runs the suite against the memory source in every capability configuration with every case passing, container case included, and checks that each deliberately broken fake fails exactly the cases that guard its flaw. Checks: `cargo xtask ci` on macOS, pull-request CI on Linux and macOS, and the full CI matrix with Windows on the branch passed. Pull request: https://github.com/alextiboldi/overthediff/pull/15
- **E04-S1-T2** — Local provider listing and metadata: Windows attributes, Unix mode, owner and group, links, follow-links · size: M · depends: E04-S1-T1 · status: done
  - AC: the listing contract cases pass on Windows, macOS and Linux, and a link to a folder is reported as a link unless follow is requested.
  - Implemented: `LocalSource`, the local provider, in `crates/source/src/local.rs` (`otd-source`). `list` and `stat` report name, kind, size, modified and creation times, Windows attribute codes or the Unix mode with owner and group names (through uzers; the numeric id when it has no name), and links with their target; with follow-links they report the target's kind, size, times and attributes instead, and a link that can't be followed (a loop) carries its error without aborting the listing. Paths that would leave the root are not found. Reading and changes refuse as unsupported until E04-S1-T3 and T4. `crates/source/tests/it/local.rs` builds each contract fixture on disk and runs the suite: every listing case passes, and a link to a folder (a junction on Windows) is reported as a link unless followed. NOTICE names uzers and tempfile. Checks: `cargo xtask ci` on macOS, `cargo check --target x86_64-pc-windows-msvc -p otd-source`, pull-request CI on Linux and macOS, and the full CI matrix with Windows on the branch passed. Pull request: https://github.com/alextiboldi/overthediff/pull/25
- **E04-S1-T3** — Local provider read, write, create folder, rename, and delete with recycle-bin support · size: M · depends: E04-S1-T1 · flags: `licence-check`
  - AC: a file deleted with the recycle bin can be restored from the OS trash. On a network path, the provider reports that the recycle bin is unavailable.
- **E04-S1-T4** — Set modified time, attributes and permissions; copy that keeps the modified time · size: S · depends: E04-S1-T3
  - AC: after a copy, source and target report the same modified time. Setting read-only on Windows and mode 0644 on Unix is visible through the OS.
- **E04-S1-T5** — Network paths (UNC, `smb://`), `~` expansion and capability detection (case sensitivity, timestamp resolution, read-only media) · size: S · depends: E04-S1-T2
  - AC: a FAT-formatted volume reports 2-second timestamp resolution, and a case-sensitive volume reports case-sensitive names.
- **E04-S1-T6** — Per-item errors that never abort a listing · size: S · depends: E04-S1-T2
  - AC: a folder with one access-denied subfolder lists every other item, and the denied one carries "access denied".

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

### E04-S3 — More archive formats and the Archive Types page
**Priority**: Should · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E04-S2 (zip archives as folders), E04-S1 (data-source contract)
**Traces**: F-7.3, F-26.12
**Not in this story**: the snapshot archive type → E19-S1; plug-ins → E04-S6.

As an IT administrator, I can browse and compare tar, 7-zip, disk images, packages and other archive formats like folders so that I can verify deliverables whatever format they ship in.

**Acceptance criteria**
- [ ] Each format opens as a folder through its default masks:
  - 7-zip `*.7z;*.7z.001`
  - BZip `*.bz;*.bz2`
  - BZipped Tar `*.tbz;*.tbz2;*.tar.bz2`
  - Compiled HTML Help `*.chm`
  - Debian Package `*.deb`
  - Disk Image `*.img;*.iso`
  - GZip `*.gz`
  - GZipped Tar `*.tgz;*.tar.gz`
  - Microsoft Cabinet `*.cab`
  - RAR `*.rar`
  - Red Hat Package `*.rpm`
  - Tar `*.tar`
  - Windows Imaging Format `*.wim;*.swm`
  - Xz `*.xz`
- [ ] 7-zip, CHM and RAR archives are read-only, and their contents can't be edited or written. Writes are refused with a message.
- [ ] The formats decided as writable in T1 pass the contract write cases.
- [ ] Single-file compressors (GZip, BZip, Xz) show the one file they contain.
- [ ] The *Archive Types* options page lists each format with an editable mask (semicolon-separated). A blank mask turns that format off. Changes apply to comparisons loaded afterwards.

#### Tasks
- **E04-S3-T1** — Decide which formats beyond zip are writable · size: S · depends: — · flags: `decision`
  - AC: decision recorded in this story. Proposed default: the tar family and single-file compressors are writable; 7-zip, RAR, CHM, disk images, packages, Cabinet and WIM are read-only.
- **E04-S3-T2** — Tar family and single-file compressors (Tar, GZipped Tar, BZipped Tar, GZip, BZip, Xz) · size: M · depends: E04-S3-T1 · flags: `licence-check`, `craft-libs`
  - AC: the contract read cases pass for every format, and the write cases pass for those decided writable.
- **E04-S3-T3** — Read-only formats: 7-zip (including split `*.7z.001`), RAR and Compiled HTML Help · size: M · depends: E04-S3-T1 · flags: `licence-check`, `craft-libs`
  - AC: browsing works, any write attempt is refused before changing anything, and a split 7-zip opens from its first part.
- **E04-S3-T4** — Packages and images: Debian, Red Hat, Disk Image, Microsoft Cabinet, Windows Imaging Format · size: M · depends: E04-S3-T1 · flags: `licence-check`, `craft-libs`
  - AC: a sample of each format lists the same entries as its native tool.
- **E04-S3-T5** — Archive Types options page with editable masks and blank-to-disable · size: S · depends: —
  - AC: clearing the Tar mask makes `x.tar` load as a plain file. Adding `*.ipa` to Zip makes `.ipa` files open as archives.

### E04-S4 — Web pages as file inputs
**Priority**: Should · **Layer**: 2 · **Size**: S · **Platforms**: All
**Needs**: E04-S1 (file-system access layer), E07-S1 (Text Compare view), E08-S1 (folder path edits)
**Traces**: F-7.2

As a web developer, I can compare a live web page with a local file or another URL so that I can see what a deployment actually serves.

**Acceptance criteria**
- [ ] An `http://` or `https://` URL is accepted as a file spec in file views and on the command line. The fetched content is compared in the view its format chooses, which is Text Compare for HTML.
- [ ] Web inputs are read-only: Save is unavailable, and Save As to a local file is allowed (*inferred*).
- [ ] Using a URL as a base folder is rejected with a message, because web pages are for individual files only.
- [ ] Redirects are followed (*inferred*). Fetch errors, such as not found, timeout or TLS failure, are shown and nothing is compared.
- [ ] The file info shows the URL.

#### Tasks
- **E04-S4-T1** — Read-only web source for single files: fetch, redirects, error reporting · size: S · depends: — · flags: `licence-check`
  - AC: a URL that redirects once loads the final content, and a 404 URL shows a clear error.
- **E04-S4-T2** — Accept URLs in file-view path edits and on the CLI; reject them as base folders · size: S · depends: E04-S4-T1
  - AC: `otd https://example.test/a.html local.html` opens a Text Compare, and a URL in a folder path edit is rejected with a message.

### E04-S5 — Portable devices (MTP) on Windows
**Priority**: Could · **Layer**: 2 · **Size**: M · **Platforms**: Win
**Needs**: E04-S1 (file-system access layer), E08-S1 (Folder Compare view)
**Traces**: F-7.1

As a user backing up a phone or camera, I can compare and copy files on a connected portable device so that I can check what's on it against my computer.

**Acceptance criteria**
- [ ] `mtp://MyDevice/MyFolder` specs open the device's storage as a base folder in Folder Compare.
- [ ] Browse for Folder lists connected portable devices next to local drives (*inferred*).
- [ ] Device storage supports read, write, delete and create folder through the data-source contract, within what the device allows, proven by the contract tests. Device folders compare with local folders in Folder Compare.
- [ ] Disconnecting the device during a scan or a contract write reports an error and leaves the comparison usable.

#### Tasks
- **E04-S5-T1** — MTP provider for listing and reading · size: M · depends: — · flags: `licence-check`
  - AC: the contract read cases pass against a connected test device.
- **E04-S5-T2** — MTP writes (copy to device, delete, create folder) where the device permits · size: M · depends: E04-S5-T1
  - AC: copying a file to the device and back gives identical content, and a write-protected device refuses writes with a message.
- **E04-S5-T3** — `mtp://` path syntax and device browsing · size: S · depends: E04-S5-T1
  - AC: a device picked through Browse fills the path edit with its `mtp://` path.

### E04-S6 — Archive plug-ins on Windows
**Priority**: Could · **Layer**: 4 · **Size**: M · **Platforms**: Win
**Needs**: E04-S3 (more archive formats and the Archive Types page)
**Traces**: F-7.3

As a power user, I can add third-party archive plug-ins so that formats the product doesn't support, such as MSI, open as folders.

**Acceptance criteria**
- [ ] *Add Plug-in* on the Archive Types page registers a Total Commander "packer" plug-in. Its formats appear in the list with editable masks.
- [ ] Archives handled by a plug-in open as folders. They are writable only if the plug-in supports writing.
- [ ] A plug-in that fails to load or crashes reports an error, and the application stays running.
- [ ] Removing a plug-in returns its masks to normal file handling.

#### Tasks
- **E04-S6-T1** — Load and register packer plug-ins from the Archive Types page · size: M · depends: — · flags: `licence-check`
  - AC: adding a sample ISO plug-in lists its format and default masks.
- **E04-S6-T2** — Data source backed by a plug-in, honouring the plug-in's capabilities · size: M · depends: E04-S6-T1
  - AC: the contract read cases pass through the sample plug-in, and write cases pass only when the plug-in supports them.
- **E04-S6-T3** — Isolate plug-in failures · size: S · depends: E04-S6-T2
  - AC: a deliberately crashing plug-in produces an error message, and the comparison window stays open.
