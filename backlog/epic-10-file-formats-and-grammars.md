# E10 — File formats and grammars

**Group**: feature
**Label**: `area:formats`
**Size**: L
**Depends on**: E01, E02 (foundations); E07 (Text Compare view); E08, E12, E13, E14 (folder, sync and merge sessions, for the per-session and external-format stories)
**Source**: catalogue §21, F-6.3, F-8.10, F-12.6, F-12.7. Spec-derived from the feature catalogue and not yet validated. Items marked *inferred* are analyst or PO inferences.

A file format is a global, ordered definition that decides how a file is handled: which view opens it, how it is converted, which grammar highlights it, and which of its elements a comparison may treat as unimportant. This epic gives users the File Formats manager, built-in grammars for syntax highlighting and importance, and the conversion and text-behaviour options that let the product handle any text-like format.

## Behavioral contract
- **Data**:
  - *File format* (held in the file-format registry): name, kind (Text, Table, Hex, Picture or External), Mask (`;`-separated masks), Description, enabled flag, built-in flag, and its position in the single ordered list.
  - Text formats add:
    - a *grammar*: ordered grammar items, each with a category, an element name, match fields and flags
    - *elements*, each with "This element is case sensitive" and a line weight
    - *Misc* settings: Insert spaces instead of tabs, Tab stop, Lines are independent, Column-based data
    - *Conversion* settings: method, loading and saving command lines, filename encoding, encoding, Ctrl+Z as end of file, characters per line limit, pre-save edits
  - External formats add four command lines.
  - Folder sessions can carry per-session overrides of which formats are enabled.
- **Grammar item categories**:
  - **Basic**: a text string, with *Match character case* and *Regular expression*.
  - **Delimited**: start and end delimiters, or a start delimiter with *Stop at end of line*, plus an optional **Escape character** and the case and regex flags.
  - **List**: tokens such as keywords, with the case and regex flags.
  - **Columns**: a column range, either start to end or start to end of line.
  - **Lines**: repeating page headers, matched by a string (typically Form Feed) or by line 1, with a number of heading lines and the case and regex flags.
- **Routes/endpoints** (desktop app, so these are commands, options and switches):
  - *Tools > File Formats* opens the manager, with format editor tabs General, Grammar, Misc and Conversion.
  - *View > Syntax Highlighting* in text views.
  - The text Importance tab's element list and its Edit Grammar button.
  - The folder-session Misc tab.
- **Integrations**:
  - E02-S1 file-format registry (resolution), mask engine and regex engine.
  - E07 Text Compare: highlighting, importance, editor behaviour and the file info header.
  - E03-S1 text engine: line weights and importance classification.
  - External programs, called through command lines.
- **Rules/states**:
  - Resolution scans from the top, and the first enabled format whose mask matches the filename wins. A format with an empty mask never matches automatically but can be picked manually.
  - In the manager, red means shadowed (every name it could match is already matched higher up) and gray means disabled.
  - In a grammar, higher items take precedence. There is no nesting and no context-specific rule, so mixed languages such as HTML+PHP are not handled specially.
  - Formats define syntax, meaning which elements exist. Session rules define importance, meaning which elements matter.
  - A conversion succeeds only when the program exits with code 0 and produces a non-empty output file. Picture conversions are load-only.

---

## Stories

### E10-S1 — File Formats manager
**Priority**: Must · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E02-S1 (file-format registry and masks), E07-S1 (Text Compare view)
**Traces**: F-21.1, F-21.2
**Not in this story**: Grammar tab → E10-S3; Misc tab → E10-S4; Conversion tab → E10-S6; Table Type/Regional tabs → E15-S7; External kind → E10-S9; per-session enablement → E10-S8.

As a developer who works with uncommon file types, I can manage the ordered list of file formats so that each of my files is recognised and handled the way I want.

**Acceptance criteria**
- [ ] *Tools > File Formats* shows a single ordered list of every format, with its kind, mask and enabled state.
- [ ] **Move Up** and **Move Down** reorder formats. Resolution follows the new order at once: the first enabled format whose mask matches wins.
- [ ] A checkbox enables or disables each format. Disabled formats show gray and never match.
- [ ] A format shadowed by those above it, because every name it could match is already matched higher up, shows red.
- [ ] *New* adds a format of any registered kind, *Duplicate* copies one, *Edit* opens the format editor, and *Delete* removes a custom format. Built-in formats can be edited and disabled but not deleted (*inferred*).
- [ ] The **General** tab holds the **Mask** (semicolon-separated masks) and the **Description** (free text; the built-in descriptions note limitations and requirements).
- [ ] A format with an empty mask never matches automatically.
- [ ] A file whose name matches a custom Text format's mask shows that format's name in the Text Compare file info header.
- [ ] The list, its order and every edit survive a restart.

- [ ] On macOS, *File Formats* is in the application menu. On Windows and Linux, it is under *Tools*.
#### Tasks
- **E10-S1-T1** — Manager list with order, enable checkbox, and kind and mask columns · size: M · depends: —
  - AC: swapping the order of two overlapping formats changes which one a test filename resolves to.
- **E10-S1-T2** — Shadowing detection with red and gray cues · size: M · depends: E10-S1-T1 · flags: `accessibility`
  - AC: `*.txt` below `*.*` shows red, and above it does not. A disabled format shows gray. Complex masks may use a conservative check that never marks a reachable format red (*inferred*).
- **E10-S1-T3** — New, Duplicate, Edit and Delete, with the General tab (Mask, Description) · size: M · depends: E10-S1-T1
  - AC: a new Text format with mask `*.foo` makes `a.foo` show that format in Text Compare's file info header. Deleting it restores the previous match.
- **E10-S1-T4** — Save the format list and edits in the settings store · size: S · depends: E10-S1-T3
  - AC: after a restart, the order, enabled states and custom formats are unchanged.

### E10-S2 — Syntax highlighting with built-in grammars
**Priority**: Must · **Layer**: 2 · **Size**: L · **Platforms**: All
**Needs**: E07-S1 (Text Compare view), E02-S1 (file-format registry and regular expressions), E05-S1 (light and dark themes)
**Traces**: F-21.4, F-12.1 · **Open questions**: Q8
**Not in this story**: grammar editing and line weights → E10-S3; importance by element → E10-S5; colour customisation → E22-S3.

As a developer, I can see source code syntax-highlighted in Text Compare so that I can read code changes as easily as in my editor.

**Acceptance criteria**
- [ ] A Text format can carry a grammar: an ordered list of grammar items, each assigned to a named element such as Comment, String, Keyword or a custom name.
- [ ] The grammar supports all five item categories (Basic, Delimited, List, Columns, Lines) with the fields described in the behavioral contract. For example, `'I can\'t'` with `\` as the escape character is one string.
- [ ] Higher items take precedence over lower ones where matches overlap.
- [ ] Each element has **This element is case sensitive**, which governs how its items match.
- [ ] Built-in Text formats with grammars exist for the starter set decided in T1, and each matches its usual masks.
- [ ] *View > Syntax Highlighting* turns highlighting on or off in Text Compare. Every element has a default colour that is readable in the light and dark themes.
- [ ] The grammar has no nesting and no context-specific rules. Mixed-language files are highlighted by their own format's grammar only.

#### Decisions
**Starter language set and built-in format list (E10-S2-T1, Q8).** Recorded by E10-S2-T1 and confirmed by the operator on 2026-11-25, as proposed. The feature catalogue says there are built-in formats and grammars but doesn't list them (Q8), so the list is *unspecified* there and is OverTheDiff's own choice. The proposal names the languages. Where it joins two with a slash, the split into formats, the masks and the treatment of plain text are T1's refinements (*inferred*).

- The starter set is 15 built-in Text formats with grammars, plus the plain Text format. Each language pair joined by a slash is **one** format with one grammar, whose keywords cover both languages:

  | Format (working name) | Masks |
  |---|---|
  | C/C++ | `*.c;*.h;*.cc;*.cpp;*.cxx;*.hh;*.hpp;*.hxx` |
  | C# | `*.cs` |
  | Java | `*.java` |
  | JavaScript/TypeScript | `*.js;*.mjs;*.cjs;*.jsx;*.ts;*.mts;*.cts;*.tsx` |
  | Python | `*.py;*.pyw;*.pyi` |
  | Go | `*.go` |
  | Rust | `*.rs` |
  | HTML/XML | `*.html;*.htm;*.xhtml;*.xml;*.xsd;*.xsl;*.xslt;*.svg` |
  | CSS | `*.css` |
  | JSON | `*.json;*.jsonc` |
  | YAML | `*.yaml;*.yml` |
  | SQL | `*.sql` |
  | Shell script | `*.sh;*.bash;*.zsh;*.ksh` |
  | Markdown | `*.md;*.markdown` |
  | INI/properties | `*.ini;*.cfg;*.properties` |

- The built-ins sit in the order shown, at the top of the format list. Their masks don't overlap, so none shadows another and their order doesn't change which one a file resolves to.
- The plain Text format is E02-S1's built-in catch-all Text format (*Plain text*, `otd_format::CATCH_ALL`), with no grammar. It is not a second, listed format: it keeps its empty mask and stays outside the ordered list, and every text file no other format matches, such as `.txt` and `.log` files, gets it (E02-S1 Decisions). So T3 adds no format of that name.
- "Shell scripts" means POSIX-family shells. Windows batch files and PowerShell scripts are not in the starter set; a user can add a format for them.
- Every grammar uses the elements Comment, String and Keyword where its language has them, so each can be highlighted and, later, ignored by element (E10-S5). T3 writes each grammar's items and adds any further elements a language needs, such as tags and attributes for HTML/XML or headings for Markdown.
- The grammars are OverTheDiff's own work, written from the languages' public specifications. Whether T3 can use a library's keyword lists or grammars instead is its `licence-check` (standards/clean-room.md and standards/licences.md).
- The names are working names. Naming.md rule 3 lets a file format or language we support keep its own name, and T3 confirms each name before it reaches the UI.
- E10-S7's built-in document converters, the other half of Q8, stay with E10-S7-T1.

#### Tasks
- **E10-S2-T1** — Decide the starter language set and built-in format list (Q8) · size: S · depends: — · flags: `decision` · status: done
  - Implemented: the decision is recorded in E10-S2's `#### Decisions` section, confirmed by the operator on 2026-11-25 as proposed: 15 built-in Text formats with grammars (each slash-joined pair is one format), non-overlapping masks for each, plain text as E02-S1's existing catch-all format, POSIX-family shells only, and Comment, String and Keyword as the shared elements; `00-index.md` and `PRD.md` mark the E10-S2 half of Q8 decided. Docs only, no code. Checks: `cargo xtask ci` on macOS after rebasing onto main, plus pull-request CI (Linux, macOS). Pull request: https://github.com/alextiboldi/overthediff/pull/122
  - AC: decision recorded in this story. Proposed default:
    - C/C++, C#, Java, JavaScript/TypeScript, Python, Go, Rust
    - HTML/XML, CSS
    - JSON, YAML, SQL, shell scripts, Markdown, INI/properties
    - a plain Text format
- **E10-S2-T2** — Grammar model and matcher for the five item categories, with precedence and element case sensitivity · size: L · depends: — · flags: `craft-libs` · status: done
  - AC: unit cases cover each category: an escaped delimiter, a List keyword with and without case, a Columns range, and a Lines header with 2 heading lines. They also cover overlap precedence.
  - Implemented: the grammar model and matcher in the new UI-free crate `crates/grammar` (`otd-grammar`): elements with their case sensitivity, items in the five categories (Basic, Delimited with escape and stop at end of line, List tokens as whole words, Columns, Lines headers), `Grammar::new` validation and `Grammar::highlight`, which returns ordered, non-overlapping spans per line, carrying unclosed strings and running headers across lines. Precedence (*inferred*): the match that starts first keeps its text, the higher item wins a tie, and column ranges and headers give way to higher items inside them. `otd-regex` gains `Regex::find_at`. `crates/grammar/tests/it/matching.rs` covers each AC case and overlap precedence; `tests/it/hostile.rs` sweeps damaged text. The craft-libs move is deferred to #124. Checks: `cargo xtask ci` on macOS and pull-request CI on Linux and macOS passed. Pull request: https://github.com/alextiboldi/overthediff/pull/127
- **E10-S2-T3** — Built-in formats and grammars for the starter set · size: M · depends: E10-S2-T1, E10-S2-T2 · flags: `licence-check`
  - AC: each built-in format matches its masks and highlights comments, strings and keywords in a sample file of its language.
- **E10-S2-T4** — Syntax highlighting in Text Compare, with the View > Syntax Highlighting toggle · size: M · depends: E10-S2-T2 · flags: `accessibility`
  - AC: turning highlighting off removes element colours but keeps difference highlighting. The default element colours are readable in both themes.

### E10-S3 — Grammar editor and line weights
**Priority**: Should · **Layer**: 3 · **Size**: L · **Platforms**: All
**Needs**: E10-S1 (File Formats manager), E10-S2 (grammar model), E07-S4 (Importance tab)
**Traces**: F-21.4, F-12.6, F-12.7

As a power user with an in-house file format, I can define its grammar myself so that it is highlighted, and its elements can be ignored, just like a built-in language.

**Acceptance criteria**
- [ ] The format editor has a **Grammar** tab that lists the grammar items in order, with their category, element and a match summary.
- [ ] Items of every category (Basic, Delimited, List, Columns, Lines) can be added, edited, deleted and reordered, with all of their fields.
- [ ] Elements can be created and renamed, and *This element is case sensitive* can be set on each.
- [ ] Line weights can be set so that alignment prefers to match higher-weight lines. A line's weight is the highest weight of the elements on it (*inferred* mechanism).
- [ ] After OK, open Text Compare views re-highlight and recompare with the edited grammar.
- [ ] The text **Importance** tab has an **Edit Grammar** button that opens the grammar of the session's current file format.

#### Tasks
- **E10-S3-T1** — Grammar tab item list with reorder and delete · size: M · depends: —
  - AC: moving an item above an overlapping item changes which element wins on that text.
- **E10-S3-T2** — Item editors for the five categories · size: L · depends: E10-S3-T1
  - AC: every specified field of each category can be set, saved and reloaded unchanged.
- **E10-S3-T3** — Element management with case sensitivity · size: S · depends: E10-S3-T1
  - AC: a custom element "Annotation" can be created, assigned to an item and made case-insensitive.
- **E10-S3-T4** — Line weights that bias alignment · size: M · depends: E10-S3-T3
  - AC: for a moved block with two possible alignments, giving declaration lines a higher weight makes alignment choose the candidate that pairs the declarations.
- **E10-S3-T5** — Edit Grammar button on the Importance tab · size: S · depends: E10-S3-T1
  - AC: the button opens the Grammar tab of the format used by the current session.

### E10-S4 — Text format behaviour: tabs, independent lines and column data
**Priority**: Should · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E10-S1 (File Formats manager), E07-S2 (difference navigation), E07-S7 (copy to other side), E07-S8 (tab and space conversions)
**Traces**: F-21.5, F-12.3, F-12.4

As a data analyst comparing record-per-line files, I can tell the product how tabs, lines and columns work in a format so that editing, navigation and comparison suit that data.

**Acceptance criteria**
- [ ] The Text format editor has a **Misc** tab with *Insert spaces instead of tabs*, *Tab stop*, *Lines are independent* and *Column-based data*.
- [ ] **Tab stop** sets the tab display interval in Text Compare and the width used by tab and space conversions.
- [ ] With **Insert spaces instead of tabs** on, the Tab key inserts spaces up to the next tab stop. With it off, Tab inserts a tab character.
- [ ] With **Lines are independent** on, each line is a record, so difference navigation and copying to the other side work per line instead of per section.
- [ ] With **Column-based data** on, character position matters, and lines are compared column by column rather than by best character match (*inferred* detail).

#### Tasks
- **E10-S4-T1** — Misc tab with the four options · size: S · depends: —
  - AC: each option is saved with the format and shown again when the editor reopens.
- **E10-S4-T2** — Tab stop display and Tab key behaviour · size: S · depends: E10-S4-T1
  - AC: with Tab stop 4 and spaces on, Tab at column 2 inserts 2 spaces. A tab character renders 4 columns wide.
- **E10-S4-T3** — Lines are independent: per-line navigation and copying · size: M · depends: E10-S4-T1
  - AC: in a 3-line difference section, Next Difference stops on each line and Copy to Other Side copies only the current line.
- **E10-S4-T4** — Column-based data comparison · size: M · depends: E10-S4-T1
  - AC: in fixed-width records where one character shifted by a column, the differing columns are highlighted rather than a single inserted character.

### E10-S5 — Grammar-based importance
**Priority**: Must · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E10-S2 (grammar model and built-in grammars), E07-S4 (importance rules)
**Traces**: F-12.6, F-6.3

As a developer, I can mark grammar elements such as comments as unimportant so that comment-only changes don't distract from code changes.

**Acceptance criteria**
- [ ] The **Importance** tab lists the grammar elements of the session's current file format (comments, strings, keywords…) together with the built-in elements.
- [ ] Unticking an element makes differences inside it unimportant: blue, and treated as same under Ignore Unimportant Differences.
- [ ] An element's case sensitivity follows the grammar's *This element is case sensitive* setting, not the *Everything else* Character case box.
- [ ] Orphan lines stay important even when they hold only an unimportant element.
- [ ] When the two sides use different formats, each side's text is classified by its own grammar (*inferred*).
- [ ] The element selection follows the Session Settings scope dropdown like every other importance rule.

#### Tasks
- **E10-S5-T1** — List the session format's grammar elements on the Importance tab · size: S · depends: —
  - AC: for a C-family file, the tab lists that grammar's elements (for example Comment, String, Keyword) below the built-ins. Changing the format changes the list.
- **E10-S5-T2** — Classify differences inside unticked elements as unimportant · size: M · depends: E10-S5-T1
  - AC: a pair that differs only inside a `//` comment shows blue while Comment is unticked and red while it is ticked.
- **E10-S5-T3** — Element case sensitivity taken from the grammar · size: S · depends: E10-S5-T2
  - AC: a case-only change inside a case-insensitive Keyword element is unimportant even when Everything else is case sensitive.

### E10-S6 — Conversion through external programs
**Priority**: Should · **Layer**: 3 · **Size**: L · **Platforms**: All
**Needs**: E10-S1 (File Formats manager), E07-S1 (Text Compare view)
**Traces**: F-21.3
**Not in this story**: built-in extractors → E10-S7 and E15-S4.

As a power user, I can plug an external converter into a file format so that files the product can't read directly are compared as text and can be saved back.

**Acceptance criteria**
- [ ] The format editor has a **Conversion** tab whose conversion method is *none* (direct) or *External program*.
- [ ] The **Loading** command line takes the variables `%s` (source file), `%t` (target or temp output) and `%o` (original file), and has a **Filename encoding** choice of Unicode or ANSI.
- [ ] A conversion succeeds only when the program exits with code 0 and produces a non-empty output file. Otherwise the view shows a conversion error for that side, and that side is read-only.
- [ ] Unmarking *Disable editing* and giving a **Saving** command line, with the same variables, writes edits back to the original format. Without a saving command, a converted side is read-only. Picture formats are load-only.
- [ ] Text formats, and Table formats where registered, have **Encoding** (auto-detect or a fixed code page) and **Ctrl+Z indicates end of file** (0x1A as an end-of-file marker).
- [ ] Text formats have a **Characters per line limit**, which splits long lines for display and removes the artificial breaks on save.
- [ ] Text formats have the pre-save edits **Trim trailing whitespace** and **Convert leading spaces to tabs**.

#### Tasks
- **E10-S6-T1** — Conversion tab: method, loading command with `%s`, `%t` and `%o`, and filename encoding · size: M · depends: —
  - AC: a format whose loading command upper-cases its input shows upper-case text in Text Compare.
- **E10-S6-T2** — Run the converter and apply the success rule · size: M · depends: E10-S6-T1
  - AC: exit code 1, or exit code 0 with an empty output, shows a conversion error and leaves that side read-only.
- **E10-S6-T3** — Saving command and the Disable editing round trip · size: M · depends: E10-S6-T2
  - AC: with a saving command, an edited and saved side is written back through the converter. Without one, the side refuses edits.
- **E10-S6-T4** — Encoding and Ctrl+Z end-of-file options · size: S · depends: —
  - AC: in a file with 0x1A part-way through, only the text before it shows while the option is on.
- **E10-S6-T5** — Characters per line limit and pre-save edits · size: M · depends: —
  - AC: a 10,000-character line is split for display at the limit and saved as one line. With *Trim trailing whitespace* on, trailing spaces are gone from the saved file.

### E10-S7 — Built-in document converters
**Priority**: Could · **Layer**: 4 · **Size**: M · **Platforms**: All
**Needs**: E10-S1 (File Formats manager), E10-S6 (Conversion tab and converter framework)
**Traces**: F-21.3 · **Open questions**: Q8

As a team lead reviewing document revisions, I can compare Word and PDF files as text so that I can see wording changes without exporting the documents by hand.

**Acceptance criteria**
- [ ] The Conversion tab offers the built-in methods decided in T1, at least **Word document → text** and **PDF → text**, alongside External program.
- [ ] Built-in Text formats for Word and PDF files use these converters by default (*inferred* masks: `*.docx;*.doc` and `*.pdf`).
- [ ] Only the text is compared. Formatting and embedded images are not, and the format's Description says so.
- [ ] Converted documents are read-only, because there is no saving conversion (*inferred*).

#### Tasks
- **E10-S7-T1** — Decide the built-in converter list (Q8) · size: S · depends: — · flags: `decision`
  - AC: decision recorded in this story. Proposed default: Word (`.docx`, plus `.doc` where feasible) and PDF to text. Other document types go through external programs.
- **E10-S7-T2** — Word document to text · size: M · depends: E10-S7-T1 · flags: `licence-check`, `craft-libs`
  - AC: two `.docx` files that differ in one paragraph show that paragraph as a changed line, and images are ignored.
- **E10-S7-T3** — PDF to text · size: M · depends: E10-S7-T1 · flags: `licence-check`, `craft-libs`
  - AC: two PDFs that differ in one sentence on page 2 show that sentence as a difference, with page order preserved.
- **E10-S7-T4** — Built-in Word and PDF formats wired to the converters · size: S · depends: E10-S7-T2, E10-S7-T3
  - AC: opening two `.docx` files lands in Text Compare, with the Word format shown in the file info header.

### E10-S8 — Per-session file-format enablement
**Priority**: Could · **Layer**: 4 · **Size**: S · **Platforms**: All
**Needs**: E10-S1 (File Formats manager), E12-S1 (Folder Sync session), E14-S1 (Folder Merge view)
**Traces**: F-8.10, F-10.1, F-11.1

As a power user, I can enable or disable file formats for a single folder session so that a special-purpose format applies only where I need it.

**Acceptance criteria**
- [ ] The **Misc** tab of Folder Compare, Folder Sync and Folder Merge session settings has an **Enabled file formats** list that overrides which global formats the session uses.
- [ ] Entries whose state differs from the global setting show in bold.
- [ ] A format disabled globally but enabled for the session is used for that session's files in every format-dependent decision, such as the view a pair opens in and content comparison. Other sessions don't use it.
- [ ] The override is stored with the session's settings.

#### Tasks
- **E10-S8-T1** — Enabled file formats list on the folder-session Misc tab, with bold for entries that differ · size: S · depends: —
  - AC: unticking a globally enabled format shows that entry in bold. Restoring the global state removes the bold.
- **E10-S8-T2** — Session-scoped format resolution in Folder Compare, Folder Sync and Folder Merge · size: S · depends: E10-S8-T1
  - AC: a globally disabled `*.foo` format enabled in one Folder Compare session applies to `.foo` pairs opened from that session only.

### E10-S9 — External formats
**Priority**: Could · **Layer**: 4 · **Size**: M · **Platforms**: All
**Needs**: E10-S1 (File Formats manager), E08-S13 (rules-based content comparison), E13-S1 (Text Merge view), E11-S5 (headless automatic merge)
**Traces**: F-21.7

As a power user, I can hand a file type to another application for comparing and merging so that specialised files are handled by the right tool.

**Acceptance criteria**
- [ ] The **External** format kind has these fields:
  - **Mask**
  - **Quick compare** command line
  - **Compare view** command line
  - **Quick merge** command line
  - **Merge view** command line
  - **Description**
- [ ] Folder comparisons use the **Quick compare** command whenever they need rules-based results for matching files. Exit code 0 means match and 1 means mismatch.
- [ ] Opening a comparison of matching files runs the **Compare view** command instead of a built-in view.
- [ ] The **Quick merge** command gives a merge result without UI, with exit codes 0 and 1. Opening a merge of matching files runs the **Merge view** command (*inferred* exit-code meaning: 0 = merged cleanly, 1 = needs manual merge).
- [ ] The command-line variables follow the decision in T1.

#### Tasks
- **E10-S9-T1** — Decide the command-line variables for the four commands · size: S · depends: — · flags: `decision`
  - AC: decision recorded in this story. Proposed default: the Open With variables, with suffixes 1, 2, 3 and 4 for the left, right, center and output files.
- **E10-S9-T2** — External format kind and its fields in the format editor · size: S · depends: E10-S9-T1
  - AC: an External format with mask `*.dwg` saves, appears in the manager list and resolves for `plan.dwg`.
- **E10-S9-T3** — Quick compare command in rules-based folder comparisons · size: M · depends: E10-S9-T2 · flags: `naming-check`
  - AC: a pair whose quick compare exits 0 shows as matching, and one that exits 1 shows as having important differences.
- **E10-S9-T4** — Compare view and Merge view commands launched on open · size: S · depends: E10-S9-T2
  - AC: opening a matching pair runs the configured program with both paths, and no built-in view opens.
- **E10-S9-T5** — Quick merge command · size: S · depends: E10-S9-T2
  - AC: `/automerge` on files of an External format runs its Quick merge command. Exit 0 writes the output, and exit 1 ends the run with code 101.
