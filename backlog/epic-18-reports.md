# E18 — Reports

**Group**: feature
**Label**: `area:reports`
**Size**: L
**Depends on**: E01 (foundation); E07, E08 (comparison views); E15, E16, E17, E25 (other viewers); E20 (script runner)
**Source**: catalogue §22 (F-22.2 is the authoritative list, taken from the script reference) and F-24.2. Spec-derived from the feature catalogue and not yet validated. Items marked *inferred* are analyst or PO inferences.

Reports save comparison results so they can be read, shared or archived without re-running the comparison. The layouts are side-by-side, interleaved, summaries, patches and statistics, and the output is HTML, plain text or print. Every report type is available from its view and as a script command.

## Behavioral contract
- **Data**:
  - *Report request*:
    - report type, layout, options and title;
    - output target (`output-to`): printer, clipboard or a file name;
    - output options: `html-color`, `html-mono` or `html-custom` with a stylesheet file or URL; `print-color` or `print-mono`; `print-portrait` or `print-landscape`; `wrap-none`, `wrap-character` or `wrap-word`.
  - *Comparison source*: the current view or the selected file pairs. A script can also name a saved session or two files.
  - *Layouts by type* (F-22.2):

    | Type | Layouts |
    |---|---|
    | Folder | side-by-side, summary, xml |
    | Text | side-by-side, summary, interleaved, patch, statistics, xml |
    | Table (`data-report`) | side-by-side, summary, interleaved |
    | Hex | side-by-side, summary, interleaved |
    | Picture | side-by-side, summary |
    | Media | side-by-side, summary |
    | Version | side-by-side, summary |
    | Registry | side-by-side, summary |
    | File | side-by-side, summary |
- **Routes/endpoints** (desktop app, so these are menu commands and script commands):
  - *Session > <Type> Report* in each comparison view, and *Actions > File Compare Report* in Folder Compare.
  - Script commands `text-report`, `folder-report`, `file-report`, `data-report`, `hex-report`, `picture-report`, `media-report`, `version-report` and `registry-report`.
  - Common script syntax: `<type>-report layout:<layout> [options:<list>] [title:<text>] output-to:(printer|clipboard|<file>) [output-options:<list>] [<comparison>]`. The `layout:`, `options:`, `output-to:` and `output-options:` keywords come from the catalogue's samples; `title:` is *inferred*.
- **Integrations**:
  - Comparison views and engines produce the content.
  - The system browser shows *View in Browser*.
  - The clipboard and printers are output targets.
  - The script runner runs the commands headlessly.
- **Rules/states**:
  - HTML output needs `html-color`, `html-mono` or `html-custom`. Without one of them, the output is plain text.
  - `wrap-character` applies only to printer output.
  - Deprecated layout names still parse: `over-under` and `composite` become interleaved, `stats-descriptive` becomes summary with a warning, and `stats-tabular` is ignored.
  - When a script names a session as the comparison, the session's type must match the report type.

---

## Stories

### E18-S1 — Text Compare report
**Priority**: Should · **Layer**: 3 · **Size**: L · **Platforms**: All
**Needs**: E07-S3 (display filters and context), E07-S4 (importance rules), E20-S1 (script runner), E06-S1 (saved sessions, for script reports by session name)
**Traces**: F-22.1, F-22.2, F-24.2
**Not in this story**: printing → E18-S3; XML layout → E18-S5.

As a code reviewer, I can save a text comparison as an HTML page, a plain-text listing or a patch so that others can read the differences without running the comparison.

**Acceptance criteria**
- [ ] *Session > Text Compare Report* opens a report dialog with choices for layout, options, title and output, and a preview.
- [ ] The layouts are *side-by-side*, *summary*, *interleaved*, *patch* (in the variants *patch-normal*, *patch-context* or *patch-unified*) and *statistics* (CSV).
- [ ] The options are:
  - *ignore-unimportant*;
  - one of *display-all*, *display-mismatches*, *display-context* or *display-matches*;
  - *line-numbers*;
  - *strikeout-left-diffs* and *strikeout-right-diffs*, for the interleaved layout.
- [ ] Output is HTML with *html-color*, *html-mono* or *html-custom* (an external stylesheet file or URL). Without one of these, it is plain text. Output goes to a file or to the clipboard, with an optional title.
- [ ] Wrapping is *wrap-none* (the default) or *wrap-word*.
- [ ] *View in Browser* opens the HTML result in the system browser.
- [ ] A *patch-unified* report of two files applies cleanly with a standard patch tool and turns the left file into the right one.
- [ ] The script command `text-report layout:<layout> [options:<list>] [title:<text>] output-to:(clipboard|<file>) [output-options:<list>] [<comparison>]` produces the same report headlessly. `<comparison>` is a saved session's name or two file names. Without it, the selected file pairs are reported, in one combined report (*inferred*).
- [ ] Deprecated layout names are accepted: *over-under* and *composite* give interleaved, *stats-descriptive* gives summary with a warning, and *stats-tabular* is ignored.
- [ ] Sample script 4, an HTML text report with context (`text-report … options:ignore-unimportant,display-context … output-options:html-color`), produces the expected report.

#### Tasks
- **E18-S1-T1** — Text report content model: rows, sections, display options and ignore-unimportant · size: M · depends: —
  - AC: *display-context* includes, around each difference, the number of context lines set in the Text Editing options. *ignore-unimportant* leaves out differences classed as unimportant.
- **E18-S1-T2** — Side-by-side, summary and interleaved layouts, with line numbers and strikeout · size: M · depends: E18-S1-T1
  - AC: interleaved with *strikeout-left-diffs* strikes through the changed left text. The summary gives totals only (*inferred* content).
- **E18-S1-T3** — Patch layouts (normal, context, unified) and the statistics layout (CSV) · size: M · depends: E18-S1-T1
  - AC: the unified patch applies cleanly with a standard patch tool. The statistics CSV opens in a spreadsheet with one row per file pair (*inferred* columns).
- **E18-S1-T4** — HTML and plain-text output to a file or the clipboard, with title, wrapping and View in Browser · size: M · depends: E18-S1-T2 · flags: `licence-check`, `craft-libs`
  - AC: *html-custom* links the given stylesheet. Without an html option, the output is plain text. *wrap-word* breaks long lines at word boundaries.
- **E18-S1-T5** — Report dialog in Text Compare · size: S · depends: E18-S1-T4
  - AC: every layout and option can be chosen in the dialog, and each produces the same output as the script command given the same arguments.
- **E18-S1-T6** — `text-report` script command, deprecated layout names and sample script 4 · size: M · depends: E18-S1-T4
  - AC: sample script 4 writes an HTML file with colour and context. `layout:over-under` produces the interleaved layout. `layout:stats-descriptive` logs a warning and produces the summary.

### E18-S2 — Folder Compare and file-list reports
**Priority**: Should · **Layer**: 4 · **Size**: L · **Platforms**: All
**Needs**: E18-S1 (report dialog and output targets), E08-S2 (display filters), E08-S6 (columns), E20-S1 (script runner), E06-S1 (saved sessions, for script reports by session name), E20-S3 (script filters, used by sample script 8)
**Traces**: F-22.1, F-22.2, F-8.17, F-24.2
**Not in this story**: vcs and revision columns → E26-S4; version column → E25-S3; xml layout → E18-S5; printing → E18-S3; the unmodified sample script 6 (rules-based criterion) → E20-S5.

As a QA engineer, I can report a whole folder comparison, or selected file pairs, so that I can archive release differences and attach them to a ticket.

**Acceptance criteria**
- [ ] *Session > Folder Compare Report* offers the layouts *side-by-side* and *summary*.
- [ ] The display-filter options are *display-all*, *display-mismatches*, *display-no-orphans*, *display-mismatches-no-orphans*, *display-orphans*, *display-left-newer*, *display-right-newer*, *display-left-newer-orphans*, *display-right-newer-orphans*, *display-left-orphans*, *display-right-orphans* and *display-matches*.
- [ ] The column options are *column-size*, *column-crc*, *column-timestamp*, *column-attributes*, *column-owner* and *column-group*. By default the report has size and timestamp, and *column-none* clears the defaults.
- [ ] *include-file-links* (side-by-side HTML only) also generates a file report for each differing pair and links each row to its file report.
- [ ] The report covers the loaded folders in their current expansion state. This is specified for scripts and *inferred* for the dialog.
- [ ] *Actions > File Compare Report* reports the selected file pairs, with the layouts side-by-side and summary and the options *ignore-unimportant*, *display-\** and *line-numbers*.
- [ ] Each pair's report type is chosen from its file format. A type without a dedicated report uses the summary layout.
- [ ] The script command `folder-report layout:(side-by-side|summary) [options:<list>] [title:<text>] output-to:(clipboard|<file>) [output-options:<list>]` reports the currently loaded folders. `file-report …` does the same for the named or selected file pairs.
- [ ] Sample script 3 (`file-report layout:side-by-side options:display-mismatches output-to:"%3" "%1" "%2"`) produces the expected report.
- [ ] Sample script 8, a report of timestamp differences, produces the expected summary. It uses `criteria timestamp:2sec`, `filter "-*.log"`, `select newer.files older.files` and `folder-report layout:summary options:display-mismatches …`.
- [ ] Sample script 6's logging and summary-report steps (`log verbose append:"…%date%.txt"`, then a summary folder report) pass with a CRC or binary comparison criterion. The rules-based criterion in the catalogue's version of the script isn't part of this story.

#### Tasks
- **E18-S2-T1** — Folder report content: rows by display-filter option, columns and current expansion · size: M · depends: —
  - AC: *display-left-orphans* lists only left orphans. *column-none* with *column-crc* shows only the CRC column.
- **E18-S2-T2** — Side-by-side and summary folder layouts in HTML and plain text · size: M · depends: E18-S2-T1
  - AC: the side-by-side HTML report colours each row by status, as the view does.
- **E18-S2-T3** — include-file-links · size: M · depends: E18-S2-T2
  - AC: a side-by-side HTML folder report over three differing text files links to three generated file reports, and each opens in a browser.
- **E18-S2-T4** — File Compare Report for the selected pairs, typed by file format · size: M · depends: —
  - AC: a text pair gets a text report. A pair whose type has no dedicated report gets the summary layout.
- **E18-S2-T5** — `folder-report` and `file-report` script commands; sample scripts 3 and 8, and sample script 6 with a CRC criterion · size: M · depends: E18-S2-T2, E18-S2-T4
  - AC: all three samples run headlessly and write the expected files. Sample 6 runs with a CRC criterion.

### E18-S3 — Print reports
**Priority**: Should · **Layer**: 4 · **Size**: M · **Platforms**: All
**Needs**: E18-S1 (report dialog and output targets)
**Traces**: F-22.1, F-22.2

As a QA engineer, I can preview and print any report with colour highlighting so that I can review differences on paper or through a PDF printer.

**Acceptance criteria**
- [ ] Every report dialog offers *Printer* as an output target, with colour highlighting.
- [ ] *Print Preview* shows the pages before anything prints.
- [ ] The output options are *print-color* or *print-mono* (the default is mono), and *print-portrait* (the default) or *print-landscape*.
- [ ] Wrapping is *wrap-none* (the default), *wrap-character* (printer only) or *wrap-word*.
- [ ] Report script commands accept `output-to:printer` with the same output options.

#### Tasks
- **E18-S3-T1** — Printer output target with colour or mono rendering · size: M · depends: — · flags: `licence-check`
  - AC: printing a text report with *print-color* shows the difference colours. *print-mono* uses no colour.
- **E18-S3-T2** — Print Preview, orientation and wrapping · size: M · depends: E18-S3-T1
  - AC: the preview in landscape shows wider pages. *wrap-character* breaks long lines at any character.
- **E18-S3-T3** — `output-to:printer` in report script commands · size: S · depends: E18-S3-T1
  - AC: a `text-report … output-to:printer` script prints without a dialog, to the default printer (*inferred*).

### E18-S4 — Table, Hex and Picture reports
**Priority**: Should · **Layer**: 4 · **Size**: M · **Platforms**: All
**Needs**: E18-S1 (report dialog and output targets), E15-S1 (Table Compare view), E16-S1 (Hex Compare view), E17-S1 (Picture Compare view), E20-S1 (script runner), E06-S1 (saved sessions, for script reports by session name)
**Traces**: F-22.2, F-14.7, F-15.3, F-16.4, F-24.2

As a data analyst, I can save table, binary and image comparisons as reports so that the differences can be reviewed without the original files.

**Acceptance criteria**
- [ ] *Table Compare Report* offers the layouts side-by-side, summary and interleaved, and the options *ignore-unimportant*, *display-all*, *display-mismatches*, *display-context* and *display-matches*, and *line-numbers* (row numbers). That *display-\** means the same four options here as for text is *inferred*.
- [ ] *Hex Compare Report* offers the layouts side-by-side, summary and interleaved, and the options *display-\** and *line-numbers* (byte addresses).
- [ ] *Picture Compare Report* offers the layouts side-by-side and summary, with *ignore-unimportant* for the summary.
- [ ] Output targets and options are the same as for text reports: HTML or plain text, to a file or the clipboard, with a title and wrapping.
- [ ] The script commands `data-report`, `hex-report` and `picture-report` produce the same reports headlessly from a session name, two file names or the selected pairs. A session of the wrong type is refused with an error.

#### Tasks
- **E18-S4-T1** — Table report layouts and options · size: M · depends: —
  - AC: *display-mismatches* lists only the rows that differ, with row numbers when *line-numbers* is set.
- **E18-S4-T2** — Hex report layouts and options · size: S · depends: —
  - AC: *line-numbers* shows byte addresses. *display-mismatches* lists only the rows of bytes that differ.
- **E18-S4-T3** — Picture report layouts · size: M · depends: —
  - AC: the side-by-side HTML report embeds the left, right and difference images (*inferred*). The summary says whether the images match, and respects *ignore-unimportant*.
- **E18-S4-T4** — `data-report`, `hex-report` and `picture-report` script commands · size: S · depends: E18-S4-T1, E18-S4-T2, E18-S4-T3
  - AC: each command, given two file names, writes its report. `hex-report` given a Text Compare session's name fails with a type-mismatch error.

### E18-S5 — XML report layouts
**Priority**: Could · **Layer**: 5 · **Size**: M · **Platforms**: All
**Needs**: E18-S1 (text report), E18-S2 (folder report)
**Traces**: F-22.2 · **Open questions**: Q9

As an automation/power user, I can export folder and text comparison results as XML so that other tools can process them.

**Acceptance criteria**
- [ ] Folder Compare and Text Compare reports offer an *xml* layout, both in their dialogs and through the `folder-report` and `text-report` script commands.
- [ ] The XML follows the schema decided in T1 and validates against it.
- [ ] Folder XML carries each item's path, status and selected columns. Text XML carries each difference's line ranges and text. Both are proposals until T1 settles the schema.
- [ ] The display-filter and *ignore-unimportant* options affect XML content as they do in the other layouts.

#### Tasks
- **E18-S5-T1** — Define the XML report schema (Q9) · size: S · depends: — · flags: `decision`
  - AC: decision recorded in this story. Proposed default: a documented, versioned schema of the product's own, published as a schema file.
- **E18-S5-T2** — Folder XML layout · size: M · depends: E18-S5-T1 · flags: `licence-check`
  - AC: a folder report in *xml* validates against the schema and lists every visible item with its status.
- **E18-S5-T3** — Text XML layout · size: M · depends: E18-S5-T1
  - AC: a text report in *xml* validates against the schema. Applying the recorded differences to the left text gives the right text (*inferred* completeness check).

### E18-S6 — Media, Version and Registry reports
**Priority**: Could · **Layer**: 4 · **Size**: M · **Platforms**: All
**Needs**: E18-S1 (report dialog and output targets), E25-S1 (Media Compare), E25-S2 (Version Compare), E25-S4 (Registry Compare), E20-S1 (script runner)
**Traces**: F-22.2, F-17.1, F-18.1, F-19.1, F-24.2

As a QA/support engineer, I can report differences in audio tags, version resources and the registry so that I can document releases and system changes.

**Acceptance criteria**
- [ ] *Media Compare Report* offers the layouts side-by-side and summary, and the options *ignore-unimportant* and *display-\**.
- [ ] *Version Compare Report* (Windows) offers the layouts side-by-side and summary, and the options *ignore-unimportant* and *display-\**.
- [ ] *Registry Compare Report* (Windows) offers the layouts side-by-side and summary, and the option *display-\**.
- [ ] Output targets and options are the same as for the other reports.
- [ ] The script commands `media-report`, `version-report` (Windows) and `registry-report` (Windows) produce the same reports headlessly.

#### Tasks
- **E18-S6-T1** — Media report · size: S · depends: —
  - AC: *display-mismatches* lists only the tags that differ.
- **E18-S6-T2** — Version report · size: S · depends: —
  - AC: a report on two builds of one DLL lists the version fields that differ.
- **E18-S6-T3** — Registry report · size: S · depends: —
  - AC: a report on two `.reg` files lists the keys and values that differ and those present on one side only.
- **E18-S6-T4** — `media-report`, `version-report` and `registry-report` script commands · size: S · depends: E18-S6-T1, E18-S6-T2, E18-S6-T3
  - AC: each command writes its report. `version-report` and `registry-report` run on Windows.
