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
- [x] Matching lines pair up. Inserted or deleted lines pair with gaps on the other side. Rows keep the order of both inputs.
- [x] Paired lines that differ carry character-level ranges on both sides that cover exactly the changed characters.
- [x] Contiguous differing rows form sections, and the same input always gives the same sections.
- [x] Standard alignment delivers partial results while it works, so the top of a large comparison can be shown before it finishes. It can be cancelled.
- [x] Every difference is classified important or unimportant by the active rules.
  - With no rules, all differences are important.
  - An orphan line counts as important even if its text would be unimportant (an inserted blank line, for example).
- [x] The overall result is "same", "unimportant differences only" or "important differences", and the statistics match the rows.
- [ ] After an edit to either side, recomparing gives the same rows as a full comparison of the edited text. Interactive edits stay within the budget set in T6.
  - Not ticked: the recompare equals a full comparison (T5's tests), and the edit budget passed `cargo bench -p otd-text-engine --bench budgets` only on a busy Apple M4 Max under macOS; it needs a run on an idle machine and on slower hardware (see T6's Verify by hand).
- [ ] Very large inputs stay within the memory and time budgets set in T6.
  - Not ticked: the budgets passed `cargo bench -p otd-text-engine --bench budgets` only on a busy Apple M4 Max under macOS; it needs a run on an idle machine, on slower hardware and on Windows (see T6's Verify by hand).

#### Decisions
Recorded by E03-S1-T6. The catalogue sets no performance figures, so every budget here is *unspecified* there and is OverTheDiff's own choice.

- **Time budgets**, confirmed by the operator on 2026-11-25, as proposed:
  - A 100,000-line pair with scattered changes shows its first rows within 1 s and completes within 10 s.
  - A single-line edit in a 10,000-line pair recompares within 100 ms.
- **Further budgets**, added by E03-S1-T6 because the acceptance criteria also ask for a memory budget and character differences are part of what a view shows. They are not part of the confirmed proposal:
  - Comparing a 100,000-line pair grows the process by at most 64 MiB beyond the two inputs. The engine's memory grows in step with the number of lines.
  - The character differences of one changed row come within 100 ms. The character search keeps its limits (2,048 differing characters, 2^26 steps) and past either one marks the whole differing middle as approximate.
- **Reference hardware** is a current laptop or desktop running a release build, otherwise idle. The budgets were measured on an Apple M4 Max machine under macOS with other builds running. First rows (about 12 ms), completion (about 15 ms) and the slowest single-line recompare (about 1.5 ms) came in under a tenth of their budgets. Memory, about 27 MiB of 64 MiB, used about 40% of its budget, and the slowest row's character differences, about 40 to 45 ms of 100 ms, about 45%: on hardware a little over twice as slow, that budget would be missed first. Slower hardware has not been measured.
- **How they are checked.** `cargo bench -p otd-text-engine --bench budgets` runs each budget and exits with an error when one is missed. It reads memory with `ps` on macOS and Linux and with `tasklist` on Windows, and fails rather than pass when it can't read it; it has run only on macOS so far. It also runs a 100,000-line pair of repetitive source-like text (blank lines and braces) against the same time budgets. Timings stay out of `cargo test` (AGENTS.md rule 28); the integration test `twice_the_lines_take_about_twice_the_polls` counts polls instead, so work that grows faster than the input fails in `cargo test`.

#### Tasks
- **E03-S1-T1** — Standard line alignment with gap padding and sections · size: L · depends: — · flags: `craft-libs` · status: done
  - AC: a corpus of before/after pairs (insertions, deletions, moved blocks, repeated lines) gives the expected row pairings and sections.
  - Implemented: `compare(left, right) -> Comparison { rows, sections }` over `Line { text, ending }` in the new UI-free crate `crates/text-engine` (`otd-text-engine`, no dependencies): a forward walk that anchors on the nearest three-line match, pairs single matching lines nearest-first between anchors, breaks equally near points by how long the sides then agree, pads the rest with gaps, and groups differing rows into maximal sections; lines align on text, and a different ending makes the row changed. The before/after corpus (each case also swapped), order, row-kind, section and repeated-line relation tests are in `crates/text-engine/tests/it/alignment.rs`. Checks: `cargo xtask ci` on macOS and pull-request CI on Linux and macOS passed. craft-libs follow-up: #9. Pull request: https://github.com/alextiboldi/overthediff/pull/10
- **E03-S1-T2** — Character-level differences inside changed rows · size: M · depends: E03-S1-T1 · flags: `craft-libs` · status: done
  - AC: changing one word in a long line marks only that word's characters, and changing whitespace marks only the whitespace.
  - Implemented: `char_differences(left, right) -> CharDifferences { ranges, exact }` in `crates/text-engine/src/chars.rs` (`otd-text-engine`, no new dependencies) trims the two lines' common start and end, then runs an O(ND) shortest-edit search over the rest, so the marked `CharRange`s (side plus byte offsets on character boundaries, left then right, none touching) are exactly the characters a shortest edit changes; a caller asks only for the rows it shows. Past fixed search limits it returns one range per side over the differing middle with `exact = false`, so a caller can say the marks are approximate (rule 15); E03-S1-T6 sets the real budgets. Tests in `crates/text-engine/tests/it/chars.rs` cover one changed word in a long line, whitespace-only changes, shortest-edit coverage, side swapping and the inexact fallback. Checks: `cargo xtask ci` on macOS and pull-request CI on Linux and macOS passed. craft-libs follow-up: #21. Pull request: https://github.com/alextiboldi/overthediff/pull/24
- **E03-S1-T3** — Importance classification framework: pluggable rules, orphan-always-important, overall result, statistics · size: M · depends: E03-S1-T1 · status: done
  - AC: a test rule marking digits unimportant turns a digits-only change into "unimportant differences only", while an inserted line of digits stays important.
  - Implemented: `classify(comparison, left, right, rules) -> Classification { rows, outcome, statistics }` with the pluggable `Rule` trait (byte ranges of a line's text that are unimportant) in `crates/text-engine/src/importance.rs`: a changed row is unimportant when its two lines are equal once the marked parts are left out, an orphan line and a different line ending are always important, and with no rules every difference is important; `Outcome` is same, unimportant differences only, or important, and `Statistics` counts important, unimportant and same rows, orphans per side and sections. The digits-rule AC, orphan, no-rules, mixed-line, line-ending, statistics-partition and out-of-range-rule tests are in `crates/text-engine/tests/it/importance.rs`. Checks: `cargo xtask ci` on macOS and pull-request CI on Linux and macOS passed. Pull request: https://github.com/alextiboldi/overthediff/pull/20
- **E03-S1-T4** — Progressive results and cancellation · size: M · depends: E03-S1-T1 · status: done
  - AC: on a 200,000-line pair, the first rows are available before completion, and cancelling stops work within half a second.
  - Implemented: `compare_progressive(left, right, step) -> ControlFlow<B, Comparison>` in `crates/text-engine/src/lib.rs` (`otd-text-engine`, no new dependencies) hands each batch of final rows to the caller's `step`, polling after every row of the same-line walk and once per left line the anchor and single-match searches look at, and stops at once with the caller's `Break`, never a partial result; `compare` runs it to the end. Poll-counting tests in `crates/text-engine/tests/it/progressive.rs` cover batches equal to a full comparison, first rows before the last, a poll per line before any row on inputs with nothing in common, and a break with no further poll. The timings are a benchmark outside `cargo test` (`cargo bench -p otd-text-engine --bench progress`, documented in AGENTS.md): on three 200,000-line pairs on macOS, first rows came at 32 to 46 ms, completion at 45 to 52 ms, and the longest gap between polls (a cancel's wait) was at most 33 ms against the 0.5 s limit. Checks: `cargo xtask ci` on macOS after rebasing onto `main`, the benchmark, the full Linux, macOS and Windows matrix and pull-request CI on Linux and macOS passed. Pull request: https://github.com/alextiboldi/overthediff/pull/22
- **E03-S1-T5** — Recompare after edits, matching a full comparison · size: M · depends: E03-S1-T3 · status: done
  - AC: for random edit sequences on the corpus, recompare after each edit equals a full comparison of the edited text.
  - Implemented: `Pair::new(left, right)` with `edit_left` and `edit_right` (replace a line range, so insert and delete too) in `crates/text-engine/src/recompare.rs` keeps two texts and their comparison current; each edit recompares the edited texts in full, so its rows are exactly what `compare` gives, and an edit outside a side's lines returns `OutOfRange` and changes nothing. Reusing old rows past the edit is left for E03-S1-T6 if its budget needs it. Fixed-seed random edit sequences on the alignment corpus, checked against a full comparison after every edit, are in `crates/text-engine/tests/it/recompare.rs`. Checks: `cargo xtask ci` on macOS and pull-request CI on Linux and macOS passed. Pull request: https://github.com/alextiboldi/overthediff/pull/27
- **E03-S1-T6** — Set and meet performance budgets for large inputs · size: S · depends: E03-S1-T4, E03-S1-T5 · flags: `decision` · status: needs-verification
  - AC: budgets recorded in this story. Proposed: a 100,000-line pair with scattered changes shows first rows within 1 s and completes within 10 s on reference hardware; a single-line edit in a 10,000-line pair recompares within 100 ms. The benchmark suite passes them.
  - Implemented: the budgets are recorded in this story's Decisions section (the time budgets confirmed by the operator on 2026-11-25 as proposed, plus a 64 MiB memory budget for a 100,000-line pair and a 100 ms budget for one row's character differences). The benchmark `crates/text-engine/benches/budgets.rs` (`cargo bench -p otd-text-engine --bench budgets`, `harness = false`, documented in AGENTS.md) checks each one on a 100,000-line scattered pair, a 100,000-line repetitive pair, 18 single-line edits of a 10,000-line `Pair` and two hard character rows, reads memory with `ps` on macOS and Linux and `tasklist` on Windows, and exits with an error on a miss or when memory can't be read. Every budget passed without an engine change, so T5's full recompare stays. The integration test `twice_the_lines_take_about_twice_the_polls` in `crates/text-engine/tests/it/progressive.rs` fails when work grows faster than the input. Checks: `cargo xtask ci` on macOS after rebasing onto `main`, the `budgets` and `progress` benchmarks on macOS, pull-request CI on Linux and macOS and the full Linux, macOS and Windows matrix passed. Pull request: https://github.com/alextiboldi/overthediff/pull/39
  - Verify by hand: run `cargo bench -p otd-text-engine --bench budgets` in a release build on an otherwise idle machine, on hardware slower than an Apple M4 Max, and once on Windows (its `tasklist` memory read has never run), and confirm it exits 0 with every figure inside its budget.

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
- **E03-S2-T1** — In-memory data source implementing the E04 contract, used as the engine's test fixture · size: S · depends: — · status: done
  - AC: fixtures can declare trees with sizes, times, links and unreadable items. The in-memory source satisfies the written E04 contract for the operations the engine uses (list, stat, read, source-provided CRC), checked by this story's own cases.
  - Implemented: the new UI-free crate `crates/folder-engine` (`otd-folder-engine`, depending only on `otd-source`) uses `otd_source::MemorySource` unchanged as the engine's fixture; its builder already declares sizes, times, links, unreadable items, source-provided CRCs and containers, so no second contract was defined. `crates/folder-engine/tests/it/fixture.rs` runs the E04 contract suite's listing and read cases on both case rules, checks that an unreadable file lists with its access-denied message and refuses reading while its neighbours read, and checks that a declared CRC lists as declared even when it disagrees with the content. Checks: `cargo xtask ci` on macOS and pull-request CI on Linux and macOS passed; no `cfg(windows)` code, so the Windows matrix was not run. Pull request: https://github.com/alextiboldi/overthediff/pull/23
- **E03-S2-T2** — Tree scanning on demand and in the background, with progress events and scan states · size: M · depends: E03-S2-T1 · status: done
  - AC: opening a folder scans it first. Background scanning completes the rest of the tree. Scan states move from unscanned through pending to done.
  - Implemented: `TreeScan` in `crates/folder-engine/src/scan.rs` (`otd-folder-engine`) scans one tree through the `DataSource` contract. `open()` puts a folder at the front of the queue, so it is listed before anything queued ahead of it; with background scanning on, each listing queues its subfolders so the rest of the tree follows. Each folder moves `Unscanned` → `Pending` → `Done` (a folder that cannot be listed ends `Done` with its error), and every listing sends a `ScanEvent` with the folders done and still pending. `step()` lists one folder on the caller's thread for headless use and tests; `start()` runs the same steps on a worker thread that stops when the scan is dropped. `crates/folder-engine/tests/it/scan.rs` checks open-first ordering, the state sequence with background scanning off, background completion with every folder listed once and progress ending with nothing pending, base-relative paths, an unlistable folder not stopping the rest of the tree, and the worker thread. Links are reported, not followed (E08-S11-T5); containers are listed only when opened (E03-S2-T9). Checks: `cargo xtask ci` on macOS (after rebasing onto `main`), pull-request CI on Linux and macOS, and the full matrix with Windows on the branch. Pull request: https://github.com/alextiboldi/overthediff/pull/32
- **E03-S2-T3** — Name alignment per folder using the source's case sensitivity, plus orphans · size: S · depends: E03-S2-T1 · status: done
  - AC: `Readme.txt` aligns with `README.txt` on a case-insensitive fixture but not on a case-sensitive one.
  - Implemented: `align()` and `case_sensitive()` in `crates/folder-engine/src/align.rs` (`otd-folder-engine`) pair one folder's left and right listings by name: exact names first, then, when either source is case-insensitive (*unspecified*, so matching follows the more permissive side), names that differ only in case. Unpaired items become orphans of their side, and rows come out in name order whatever the listing order. `crates/folder-engine/tests/it/align.rs` checks the AC on `MemorySource` fixtures of both case rules, orphans on each side, the exact-match preference on a case-sensitive side holding two case variants, and order independence. Checks: `cargo xtask ci` on macOS (after rebasing onto `main`) and pull-request CI on Linux and macOS; no `cfg(windows)` code, so the Windows matrix was not run. Pull request: https://github.com/alextiboldi/overthediff/pull/30
- **E03-S2-T4** — Quick tests (size; modified time with tolerance) and the status model · size: M · depends: E03-S2-T3 · status: done
  - AC: a table of size and time deltas against tolerances gives the expected status in every row, including exactly-at-tolerance cases.
  - Implemented: `Status`, `Criteria` and `quick_test()` in `crates/folder-engine/src/status.rs` (`otd-folder-engine`) give one aligned row its status from metadata alone: same, different, left newer, right newer, left orphan, right orphan, unknown (two folders, whose result comes from their contents) and error (an unreadable item, keeping the source's message). Modified times further apart than the tolerance make the later side newer; a gap exactly at the tolerance counts as the same. Then different sizes make the row different; a file against a folder or link is different; a criterion that is off never decides. Newer winning over a size difference, the left error kept when both sides are unreadable, and links compared by their listed metadata are *unspecified* and documented on `quick_test()`. `crates/folder-engine/tests/it/status.rs` checks the AC's table of size and time deltas against tolerances, including exactly-at-tolerance rows, plus criteria switched off, orphans, unreadable items, file against folder and folder timestamps, on `MemorySource` fixtures. Checks: `cargo xtask ci` on macOS (after rebasing onto `main`) and pull-request CI on Linux and macOS; no `cfg(windows)` code, so the Windows matrix was not run. Pull request: https://github.com/alextiboldi/overthediff/pull/43
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
