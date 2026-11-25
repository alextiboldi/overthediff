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
  - Unmatched files fall back by content (T7 decision, *inferred*).

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

#### Tasks
- **E02-S1-T1** — Wildcard matcher for `?`, `*`, sets, ranges, negation, `[[]`, `*.*` and `*.` · size: M · depends: — · flags: `craft-libs` · status: done
  - AC: a conformance table of mask × name pairs from A.1 passes, including names with no extension and names with several periods.
  - Implemented: `Wildcard::new(mask, case)` and `Wildcard::matches(name)` in the new UI-free crate `crates/mask` (`otd-mask`, no dependencies), with the caller's `Case` and `WildcardError` for unclosed and empty sets; the A.1 conformance table, case, period-set and error tests and a hostile-input sweep are in `crates/mask/tests/it/`. Checks: `cargo xtask ci` on macOS and pull-request CI on Linux and macOS passed. craft-libs follow-up: #4. Pull request: https://github.com/alextiboldi/overthediff/pull/6
- **E02-S1-T2** — Mask-list evaluation over tree paths: include/exclude, folder masks, relative masks, base folder always included · size: M · depends: E02-S1-T1 · flags: `craft-libs`
  - AC: the seven catalogue examples give the specified results on a sample tree that has nested `Windows` and `Backup` folders.
- **E02-S1-T3** — Regex engine meeting A.2: escapes, classes, alternation, anchors, quantifiers, groups, backreferences, `(?i)` scoping, line-by-line matching · size: M · depends: — · flags: `licence-check`, `craft-libs`
  - AC: the A.2 conformance cases and the sample library pass. A pattern containing `\n` can't match across lines. How conformance is reached is open.
- **E02-S1-T4** — Replacement templates with `$1`…`$9` · size: S · depends: E02-S1-T3 · flags: `craft-libs`
  - AC: the catalogue rename example passes, and a reference to a missing group produces an empty string (*inferred*).
- **E02-S1-T5** — Regex input aid control: element picker and RE Examples · size: S · depends: E02-S1-T3 · flags: `naming-check`, `craft-libs`
  - AC: a host field gains both menus. Inserting an element at a mid-text cursor keeps the surrounding text.
- **E02-S1-T6** — File-format registry: ordered list, kinds, enable flag, mask resolution, manual pick of empty-mask formats, registration by features · size: M · depends: E02-S1-T2
  - AC: with formats A (`*.txt`) and B (`*.*`) in that order, `x.txt` resolves to A. Disabling A makes it resolve to B. An empty-mask format C resolves only when picked by name.
- **E02-S1-T7** — Decide and implement the fallback for unmatched files (text vs binary detection) · size: S · depends: E02-S1-T6 · flags: `decision`, `craft-libs`
  - AC: decision recorded in this story. Proposed default: a file with NUL bytes or invalid text encoding in its first 8 KB is binary. A UTF-8 sample resolves to Text, and a PNG with an unknown extension resolves to the Hex kind when Hex is registered.
