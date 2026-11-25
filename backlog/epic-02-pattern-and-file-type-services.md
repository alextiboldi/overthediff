# E02 — Pattern and file-type services

**Group**: foundation
**Label**: `area:patterns`
**Size**: M
**Depends on**: —
**Source**: catalogue Appendix A (A.1 file masks, A.2 regular expressions), F-21.1, F-21.2. Spec-derived from the feature catalogue and not yet validated. Items marked *inferred* are analyst or PO inferences.

This epic is the shared matching layer: file masks, regular expressions and the ordered file-format registry that decides how each file is treated. Name filters, find, rename, grammars, replacements, alignment overrides, scripts and the CLI all depend on it. Building it once, with conformance cases taken from the catalogue, means none of those stories reinvents matching.

## Behavioral contract
- **Data**:
  - *Mask list*: `;`-separated tokens. Each token may be `-`-prefixed (exclude) and/or `\`-suffixed (folder). Tokens can be simple (`f`, `p\`) or relative (`p\f`, `.\f`, `...\f`).
  - *Regular expression*: pattern plus flags (match case, whole words). A *replacement template* uses `$1`…`$9`.
  - *File format*: name, kind (Text, Table, Hex, Picture, External), masks, enabled, built-in flag, description and kind-specific properties, which the owning feature defines.
  - *Registry*: the single ordered list of formats.
- **Routes/endpoints** (desktop app, so this is a reusable control): the regex input aid, with the *Insert Regular Expression Element* picker and *RE Examples*, offered wherever a feature accepts a regex.
- **Integrations**: features consume the engines, and viewers register format kinds and built-in formats at start-up. User edits to formats and their order belong to E10-S1.
- **Rules/states**:
  - Mask semantics follow A.1. The base folder is always included, and folders containing included files are included.
  - A.1 doesn't say whether masks ignore case or what an edge `-` or an empty set in `[...]` means. Proposed default: a mask compares names with the data source's case sensitivity, so `*.txt` matches `README.TXT` on a case-insensitive source, as names align in E03-S2. A `-` at the start or end of a set is literal, and an empty set (`[]`, `[!]`) is an error (*inferred*; E02-S1-T1 takes the case from its caller).
  - A.1 doesn't say where a path mask starts, how file and folder masks combine, or how excludes rank. Proposed default: a path mask without `.\` may start at any depth, so `Windows\*.txt` and `...\Windows\*.txt` select the same files. A file is included when it passes the file masks and sits in an included folder; the base folder's own files count as in one. An exclude wins over any include, and a kind with no include masks includes everything of that kind not excluded. Spaces around a token are dropped and empty tokens skipped (*inferred*; E02-S1-T2).
  - Regex semantics follow A.2. Matching is line by line, so `\A`/`\Z` behave like `^`/`$`.
  - Format resolution: scan from the top, and the first enabled format whose mask matches wins. A format with an empty mask never matches automatically but can be picked manually.
  - Unmatched files fall back by content: binary (a NUL or invalid text encoding in the first 8 KB) gets the Hex kind when it is registered, anything else Text (T7 decision, *inferred*, confirmed by the operator on 2026-11-25; see the story's Decisions).

---

## Stories

### E02-S1 — File masks, regular expressions and file-format resolution
**Priority**: Must · **Layer**: 0 · **Size**: M · **Platforms**: All
**Needs**: —
**Traces**: F-21.1, F-21.2, App-A
**Not in this story**: File Formats manager UI and shadowing cues → E10-S1; grammars → E10-S2; conversions → E10-S6.

As a developer on the product team, I can match names, text and file types through one tested service so that every filter, search and viewer choice behaves the same way across the product.

**Acceptance criteria**
- [ ] **Wildcards.** `?` matches one character and `*` matches zero or more. `[az]` is a set, `[a-z]` a range, `[!az]` a negated set and `[[]` a literal `[`. `*.*` also matches names without an extension.
- [ ] **No-extension masks.** `*.` matches only names without an extension. In a mask ending in `.`, neither `?` nor `*` matches a period.
- [ ] **Simple masks.**
  - `f` includes files named f, and `p\` includes folders named p and their subfolders.
  - `-f` excludes files and `-p\` excludes folders.
  - The base folder is always included, and folders containing included files are included.
- [ ] **Relative masks.** `p\f` requires f inside a folder named p. `.\f` requires f in the base folder. `...\f` allows zero or more parent folders.
- [ ] **Catalogue examples.** `*.pas;*.dfm;*.dpr`, `Source\`, `-*.dcu;-*.ddp`, `-Backup\`, `Windows\*.txt`, `...\Windows\*.txt` and `.\Windows\*.txt` give the specified include/exclude results on a sample tree.
- [ ] **Regex escapes and classes.**
  - Escapes: `\xnn`, `\x{nnnn}`, `\t`, `\f`, `\a` and `\e`, with `\` escaping metacharacters.
  - Classes: `.`, `\w`, `\W`, `\d`, `\D` and `\s` (space and tab).
  - Bracket classes support ranges, `[^…]` negation, a literal `-` at the start or end, and predefined classes inside.
- [ ] **Regex structure.**
  - Alternation takes the leftmost alternative first (`get|getter` matches "get" in "getter").
  - Greedy (`*` `+` `?` `{n}` `{n,}` `{n,m}`) and lazy (`*?` `+?` `??` `{n}?` `{n,}?` `{n,m}?`) quantifiers work.
  - Groups support backreferences `\1`–`\9`, and `(?i)`/`(?-i)` apply only within the enclosing group.
- [ ] **Line by line.** A pattern never matches across a line break, and `\A`/`\Z` behave like `^`/`$`.
- [ ] **Sample library.** `^abc`, `abc$`, `^abc$`, `^\s*abc`, `^\s*end;?\s*$`, `abc|def`, `a(b|c)d` and `a(b|c)d\1` match exactly the specified strings.
- [ ] **Replacement templates.** `$1`…`$9` insert captured groups. `.*(\d\.txt)` with `xyz$1` turns `abc1.txt` into `xyz1.txt`.
- [ ] **Regex input aid.** *Insert Regular Expression Element* inserts the chosen construct at the cursor, and *RE Examples* inserts a chosen sample. Both work in any field that hosts the aid.
- [ ] **Format resolution.**
  - The registry resolves a file name to the first enabled format whose mask matches, and disabled formats are skipped.
  - An empty-mask format is never chosen automatically but can be chosen by name.
  - Formats registered by a test feature appear in registration order after the built-ins.
- [ ] **Fallback.** A file that matches no format resolves by content: text-like content gets the built-in catch-all Text format, and binary content gets the Hex kind when one is registered, otherwise Text (*inferred*, confirmed in T7).

#### Decisions
**Fallback for unmatched files (E02-S1-T7).** Recorded by E02-S1-T7 and confirmed by the operator on 2026-11-25, as proposed: a file with NUL bytes or invalid text encoding in its first 8 KB is binary. The catalogue doesn't say how unmatched files are treated, so this is OverTheDiff's own choice (*inferred*). It reads as follows:
- Only the first 8 KB (8,192 bytes) are looked at. A character that byte 8,192 cuts in two still counts as text; one cut at the end of a shorter file doesn't.
- A UTF-16 BOM (`FF FE` or `FE FF`) makes the sample UTF-16: it is text when its code units decode with no unpaired surrogate and no U+0000. Otherwise the sample must be valid UTF-8, with or without a BOM, and hold no NUL byte. An empty file is text.
- So UTF-16 without a BOM, UTF-32 and legacy code-page text count as binary. A text format still takes them by mask, and any format can be picked by name.
- A file no enabled format matches gets the Hex kind when its sample is binary and the Hex kind is registered, and the Text kind otherwise. The view of that kind opens it with its own catch-all format; the registry doesn't hold one, and no feature registers built-in formats before E07-S1 and E16-S1.

#### Tasks
- **E02-S1-T1** — Wildcard matcher for `?`, `*`, sets, ranges, negation, `[[]`, `*.*` and `*.` · size: M · depends: — · flags: `craft-libs` · status: done
  - AC: a conformance table of mask × name pairs from A.1 passes, including names with no extension and names with several periods.
  - Implemented: `Wildcard::new(mask, case)` and `Wildcard::matches(name)` in the new UI-free crate `crates/mask` (`otd-mask`, no dependencies), with the caller's `Case` and `WildcardError` for unclosed and empty sets; the A.1 conformance table, case, period-set and error tests and a hostile-input sweep are in `crates/mask/tests/it/`. Checks: `cargo xtask ci` on macOS and pull-request CI on Linux and macOS passed. craft-libs follow-up: #4. Pull request: https://github.com/alextiboldi/overthediff/pull/6
- **E02-S1-T2** — Mask-list evaluation over tree paths: include/exclude, folder masks, relative masks, base folder always included · size: M · depends: E02-S1-T1 · flags: `craft-libs` · status: done
  - AC: the seven catalogue examples give the specified results on a sample tree that has nested `Windows` and `Backup` folders.
  - Implemented: `MaskList::new(list, case)` in `crates/mask/src/list.rs` (`otd-mask`, no dependencies) parses `;`-separated tokens with the `-` exclude prefix, the `\` folder suffix and `.`/`...` path parts over T1 `Wildcard`s; `includes_file(path)` and `folder(path)` (`Included`, `Excluded`, `IfContainsIncluded`) evaluate pre-split paths, with the base folder always included and excludes winning. The catalogue examples run against a sample tree with nested `Windows` and `Backup` folders in `crates/mask/tests/it/list.rs`, and the hostile-input sweep covers mask lists. Checks: `cargo xtask ci` on macOS and pull-request CI on Linux and macOS passed. craft-libs follow-up: #18. Pull request: https://github.com/alextiboldi/overthediff/pull/19
- **E02-S1-T3** — Regex engine meeting A.2: escapes, classes, alternation, anchors, quantifiers, groups, backreferences, `(?i)` scoping, line-by-line matching · size: M · depends: — · flags: `licence-check`, `craft-libs` · status: done
  - Implemented: UI-free crate `otd-regex` (`crates/regex`) reads A.2 patterns, rejects undefined constructs with a positioned syntax error, translates them to fancy-regex (MIT) with `(?i)`/`(?-i)` scoped to their group, and matches line by line (LF, CRLF, CR) so no match crosses a line break; runaway backtracking is reported as too complex. A.2 conformance cases, the sample library and a craft-testkit hostile sweep are in `crates/regex/tests/it/`. Checks: `cargo xtask ci` (fmt, clippy `-D warnings`, `cargo test --workspace --locked`, ui-free) passed on macOS after rebasing onto main, plus pull-request CI (Linux, macOS). craft-libs follow-up: #7. Pull request: https://github.com/alextiboldi/overthediff/pull/8
  - AC: the A.2 conformance cases and the sample library pass. A pattern containing `\n` can't match across lines. How conformance is reached is open.
- **E02-S1-T4** — Replacement templates with `$1`…`$9` · size: S · depends: E02-S1-T3 · flags: `craft-libs` · status: done
  - AC: the catalogue rename example passes, and a reference to a missing group produces an empty string (*inferred*).
  - Implemented: `Match::expand(text, template)` and `Regex::replace(text, template)` in `crates/regex/src/lib.rs` (`otd-regex`, no new dependency): `$1`…`$9` insert the match's groups, a missing or non-participating group inserts nothing, any other `$` is literal and a reference takes one digit; `replace` rewrites every match on every line and keeps the rest. The catalogue rename example, missing-group, literal-`$` and multi-line tests are in `crates/regex/tests/it/replace.rs`, and the hostile sweep runs damaged templates and patterns through `replace`. Checks: `cargo xtask ci` (fmt, clippy `-D warnings`, `cargo test --workspace --locked`, ui-free) on macOS and pull-request CI (Linux, macOS) passed. craft-libs follow-up: #33. Pull request: https://github.com/alextiboldi/overthediff/pull/34
- **E02-S1-T5** — Regex input aid control: element picker and RE Examples · size: S · depends: E02-S1-T3 · flags: `naming-check`, `craft-libs` · status: needs-verification
  - AC: a host field gains both menus. Inserting an element at a mid-text cursor keeps the surrounding text.
  - Implemented: `regex_aid(ui, field_id, &mut text)` in the new UI library crate `crates/ui-regex` (`otd-ui-regex`, depends on `egui` only) draws two menu buttons, **Regex elements** (grouped A.2 constructs) and **Regex examples** (the story's sample library), beside a host `TextEdit`; a chosen entry replaces the selection or goes in at the cursor, and focus returns to the field (or to the menu button after Esc). egui_kittest tests in `crates/ui-regex/tests/it/aid.rs` drive it by keyboard: two hosted fields, mid-text insertion in multi-byte text, Esc focus return, and every entry compiling with `otd-regex`. The naming check is in `docs/feature-names.md`. Checks: `cargo xtask ci` (fmt, clippy `-D warnings`, `cargo test --workspace --locked`, ui-free) on macOS after rebasing onto main, plus pull-request CI (Linux, macOS). craft-libs follow-up: #35. Pull request: https://github.com/alextiboldi/overthediff/pull/36
  - Verify by hand: finish the naming.md check for "Regex elements" and "Regex examples" (USPTO, EUIPO/TMview and WIPO register searches, and the three-product descriptive check for "Regex elements"), record the results in `docs/feature-names.md`, and change the label constants in `crates/ui-regex/src/lib.rs` if either name fails.
- **E02-S1-T6** — File-format registry: ordered list, kinds, enable flag, mask resolution, manual pick of empty-mask formats, registration by features · size: M · depends: E02-S1-T2 · status: done
  - AC: with formats A (`*.txt`) and B (`*.*`) in that order, `x.txt` resolves to A. Disabling A makes it resolve to B. An empty-mask format C resolves only when picked by name.
  - Implemented: the UI-free crate `crates/format` (`otd-format`, depends only on `otd-mask`, no new third-party dependency): `Kind` (Text, Table, Hex, Picture, External) with `Registry::register_kind`/`has_kind`; `Format::new(name, kind, masks)` compiles the A.1 mask list and rejects a bad mask; `Registry::register` appends below earlier formats and refuses an unregistered kind or a taken name; `resolve(file_name, case)` returns the first enabled format whose mask matches, with the data source's case; an empty mask never resolves automatically; `by_name` is the manual pick; `set_enabled` turns a format on or off. Tests for the AC, registration order, case and refusals are in `crates/format/tests/it/registry.rs`. Checks: `cargo xtask ci` (fmt, clippy `-D warnings`, `cargo test --workspace --locked`, ui-free) on macOS and pull-request CI (Linux, macOS) passed. Pull request: https://github.com/alextiboldi/overthediff/pull/31
- **E02-S1-T7** — Decide and implement the fallback for unmatched files (text vs binary detection) · size: S · depends: E02-S1-T6 · flags: `decision`, `craft-libs`
  - AC: decision recorded in this story. Proposed default: a file with NUL bytes or invalid text encoding in its first 8 KB is binary. A UTF-8 sample resolves to Text, and a PNG with an unknown extension resolves to the Hex kind when Hex is registered.
