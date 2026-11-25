# E03 — Comparison engines

**Group**: foundation
**Label**: `area:engines`
**Size**: L
**Depends on**: E04 behavioral contract (data-source contract, built against a fake). No story dependency.
**Source**: catalogue §2 (glossary: section, orphan, skew, CRC), F-8.1, F-8.3, F-8.4, F-8.7, §9 common rules, F-12.6, F-12.7, F-6.3, X2. Spec-derived from the feature catalogue and not yet validated. Items marked *inferred* are analyst or PO inferences.

This epic is the two headless engines that decide what differs. The text engine aligns lines and characters and classifies each difference as important or unimportant. The folder engine scans two trees, aligns items, runs quick and content tests, and aggregates status up the tree. Every viewer, the CLI quick compare and scripting are thin layers over these two engines. Building them first, test-first and without UI, lets views, automation and reports proceed in parallel.

## Behavioral contract
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

---

## Stories

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
- **E03-S1-T1** — Standard line alignment with gap padding and sections · size: L · depends: — · flags: `craft-libs` · status: done
  - AC: a corpus of before/after pairs (insertions, deletions, moved blocks, repeated lines) gives the expected row pairings and sections.
  - Implemented: `compare(left, right) -> Comparison { rows, sections }` over `Line { text, ending }` in the new UI-free crate `crates/text-engine` (`otd-text-engine`, no dependencies): a forward walk that anchors on the nearest three-line match, pairs single matching lines nearest-first between anchors, breaks equally near points by how long the sides then agree, pads the rest with gaps, and groups differing rows into maximal sections; lines align on text, and a different ending makes the row changed. The before/after corpus (each case also swapped), order, row-kind, section and repeated-line relation tests are in `crates/text-engine/tests/it/alignment.rs`. Checks: `cargo xtask ci` on macOS and pull-request CI on Linux and macOS passed. craft-libs follow-up: #9. Pull request: https://github.com/alextiboldi/overthediff/pull/10
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
