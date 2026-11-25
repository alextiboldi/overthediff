# E16 — Hex Compare

**Group**: feature
**Label**: `area:hex-compare`
**Size**: M
**Depends on**: E01, E02, E04 (foundations)
**Source**: catalogue §15. Spec-derived from the feature catalogue, including its Hex Compare figure, and not yet validated. Items marked *inferred* are analyst or PO inferences, or details read only from a figure.

Hex Compare shows the raw bytes of two files side by side in hex. It aligns inserted and deleted byte runs, and lets the user edit bytes and copy them across. It's the view for binary files, and for checking what really differs when a text comparison can't show it.

## Behavioral contract
- **Data**:
  - *Byte sequences*: the contents of the two files.
  - *Aligned byte runs*: matching runs, differing runs, and gaps where alignment inserted padding.
  - *Grid rows*: an address, the bytes and a character panel. A row may hold fewer bytes where a gap splits it.
  - *Hex format*: file-format kind Hex, with masks.
  - *Session settings*:
    - Specs: files, Description and Disable editing.
    - Comparison: alignment (Complete, Fast or None), and a size threshold below which files load without file locks.
    - Format: hex format per side.
- **Routes/endpoints** (desktop app: commands, settings and CLI):
  - Home and Session > New Session ▸ Hex Compare.
  - The CLI with two files that resolve to a Hex format (binary files by default).
  - Session Settings tabs Specs, Comparison and Format.
  - Edit, Search and View commands.
  - The status bar.
- **Integrations**:
  - E02-S1: the file-format registry, for the Hex kind and the binary fallback.
  - E04-S1: reads and writes the files.
  - E01-S1: the shell and the session framework.
- **Rules/states**:
  - Alignment None compares byte N with byte N.
  - Complete aligns insertions and deletions, padding them with crosshatched gaps.
  - Fast is a simpler, quicker alignment for very large files.
  - Files up to the lock threshold load without file locks, so other programs can still change them while they're open.
  - A binary file with no other matching format opens in Hex Compare (*inferred*).

---

## Stories

### E16-S1 — Hex Compare view
**Priority**: Should · **Layer**: 1 · **Size**: L · **Platforms**: All
**Needs**: E01-S1 (application shell), E02-S1 (file-format registry), E04-S1 (file-system access layer)
**Traces**: F-15.1, F-15.2, F-15.3
**Not in this story**: byte editing and Disable editing → E16-S2; find/replace, value inspector, Layout dialog and thumbnail → E16-S3; child-session commands → E06-S11; report → E18-S4; Compare Files Using → E06-S9; Compare Parent Folders → E06-S10; Hex font → E22-S3.

As a QA engineer checking two firmware images, I can see their bytes side by side with insertions aligned so that I find exactly which bytes changed and where.

**Acceptance criteria**
- [ ] A Hex format kind and a built-in Hex format are registered. Two binary files that match no other format open in Hex Compare, whether passed on the CLI or chosen through Home or Session > New Session ▸ Hex Compare (*inferred* binary fallback).
- [ ] Each row shows an address, the bytes in hex, and a character panel (*inferred*). View > Byte Addresses toggles the address column, and the current byte address is shown.
- [ ] Differing bytes are highlighted. Where alignment inserts a gap, the row is split and the gap is crosshatched, so a row can hold fewer bytes than usual (*inferred*).
- [ ] The Comparison tab offers three alignments: Complete (aligns insertions and deletions), Fast (simpler and quicker, for very large files) and None (byte N against byte N).
- [ ] The Comparison tab setting "Files up to this size are loaded without file locks" lets other programs edit smaller files while they are open.
- [ ] Display filters Show All, Show Differences and Show Same work.
- [ ] Next/Previous Difference Bytes moves between differing runs, and Go To moves to a byte offset.
- [ ] Side-by-Side and Over-Under layouts are available. File Info shows the timestamp, size and a format dropdown (*inferred*).
- [ ] The Format tab sets a hex format per side ("Detected" by default). The Specs tab holds the files and a Description.
- [ ] Swap Sides, Reload, Recompare and Hex Compare Info all work.
- [ ] The status bar shows the overall result (for example "Binary differences"), the insert/overwrite state and the load time (*inferred*). The Insert key toggles the insert/overwrite state.

#### Tasks
- **E16-S1-T1** — Register the Hex kind, a built-in Hex format and the binary fallback · size: S · depends: — · flags: `naming-check`
  - AC: two `.bin` files passed on the CLI open in Hex Compare, while a text file still opens in its text format.
- **E16-S1-T2** — Byte alignment: Complete, Fast and None · size: L · depends: —
  - AC: inserting 3 bytes in the middle of a file gives one 3-byte gap with Complete. With None, the same edit gives a run of differences to the end of the file.
- **E16-S1-T3** — Hex grid: addresses, character panel, highlights and crosshatched gaps · size: M · depends: E16-S1-T2 · flags: `craft-libs`, `accessibility`
  - AC: the grid shows the inserted run facing a crosshatched gap, and every other row as matching.
- **E16-S1-T4** — Display filters, Next/Previous Difference Bytes and Go To · size: M · depends: E16-S1-T3
  - AC: Show Differences shows only rows with differing bytes. Go To `0x100` puts the cursor on byte 256.
- **E16-S1-T5** — Load without file locks up to the configured size · size: S · depends: —
  - AC: when the file is smaller than the threshold, another program can overwrite it while it's open; when it's larger, it can't (*inferred*).
- **E16-S1-T6** — Layouts, File Info, Specs and Format tabs, Swap/Reload/Recompare/Info, and the status bar · size: M · depends: E16-S1-T3
  - AC: changing the right side's format reloads that side. Identical files show "Binary same" in the status bar, and the Insert key flips the insert/overwrite indicator.

### E16-S2 — Hex editing
**Priority**: Should · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E16-S1 (Hex Compare view)
**Traces**: F-15.3, F-6.6
**Not in this story**: profile-based save variants (not yet placed for hex views); backups → E09-S10.

As a developer patching a binary resource, I can edit bytes and copy byte runs to the other side so that I can make one file match the other and save it.

**Acceptance criteria**
- [ ] Typing hex digits changes bytes; typing in the character panel does too (*inferred*). Typing follows the insert/overwrite mode shown in the status bar.
- [ ] Copy to Right, Copy to Left and Copy to Other Side copy the selected bytes to the other side.
- [ ] Cut, Copy, Paste, Delete and Select All work on byte selections. Copy puts the bytes on the clipboard (*inferred* format: hex text).
- [ ] Undo and Redo cover every edit, and the comparison updates after each one (*inferred*).
- [ ] Save and Save As write the bytes. Unsaved changes prompt on close.
- [ ] Disable editing on the Specs tab makes both sides read-only. Editing is also unavailable when the location is read-only or loading failed.

#### Tasks
- **E16-S2-T1** — Byte editing in overwrite and insert modes · size: M · depends: —
  - AC: in overwrite mode, typing `FF` replaces one byte. In insert mode, it adds one byte and the file grows by one.
- **E16-S2-T2** — Cut, Copy, Paste, Delete and Select All on byte selections · size: S · depends: E16-S2-T1
  - AC: copying 4 bytes and pasting them in insert mode adds exactly those 4 bytes.
- **E16-S2-T3** — Copy to Right, Copy to Left and Copy to Other Side for selected bytes · size: S · depends: E16-S2-T1
  - AC: copying a differing run to the other side makes that run match.
- **E16-S2-T4** — Undo/Redo and recompare after edits · size: S · depends: E16-S2-T1, E16-S2-T2, E16-S2-T3
  - AC: undo restores the original bytes and the original comparison.
- **E16-S2-T5** — Save, Save As, Disable editing and the read-only conditions · size: S · depends: E16-S2-T1
  - AC: a saved file is byte-identical to what the grid shows. With Disable editing on, every edit command is greyed out.

### E16-S3 — Hex search, value inspection and layout
**Priority**: Should · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E16-S1 (Hex Compare view), E16-S2 (hex editing and saving)
**Traces**: F-15.1, F-15.3 · **Open questions**: Q6

As a developer decoding an unknown file format, I can search for byte patterns, read the values under the cursor and change the grid layout so that I can interpret the data, not just see that it differs.

**Acceptance criteria**
- [ ] A Layout dialog offers the options settled in T1 (Q6) and applies them to both grids.
- [ ] Find searches for a byte sequence in hex, or for text (*inferred*). Find Next and Find Previous step through the matches.
- [ ] Replace replaces the matched bytes. Replaced bytes count as unsaved changes (*inferred*).
- [ ] The Little Endian Values and Big Endian Values panels interpret the bytes at the cursor (*inferred* value types: 8-, 16-, 32- and 64-bit integers, signed and unsigned, and 32- and 64-bit floats).
- [ ] The Thumbnail shows one colour row per grid row and marks the viewport; clicking it jumps there (*inferred*: as in Text Compare).

#### Tasks
- **E16-S3-T1** — Decide the Layout dialog options (Q6) · size: S · depends: — · flags: `decision`
  - AC: decision recorded. Proposed default:
    - bytes per row: 8, 16 (default) or 32;
    - byte grouping: 1, 2, 4 or 8;
    - character panel encoding: ASCII, system code page or UTF-8;
    - address format: hex (default) or decimal.
- **E16-S3-T2** — Layout dialog applied to both grids · size: M · depends: E16-S3-T1
  - AC: switching to 32 bytes per row halves the row count, and alignment gaps still split rows.
- **E16-S3-T3** — Find, Find Next/Previous and Replace for bytes and text · size: M · depends: —
  - AC: searching for `DE AD BE EF` finds every occurrence in order. A text search finds the bytes that spell that text in ASCII.
- **E16-S3-T4** — Little- and big-endian value inspector · size: S · depends: —
  - AC: with the cursor on bytes `01 00 00 00`, the little-endian 32-bit value reads 1 and the big-endian value reads 16777216.
- **E16-S3-T5** — Thumbnail with click-to-jump · size: S · depends: —
  - AC: differing regions show as coloured bands, and clicking a band scrolls to it.
