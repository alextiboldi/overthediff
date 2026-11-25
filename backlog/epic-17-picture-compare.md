# E17 — Picture Compare

**Group**: feature
**Label**: `area:picture-compare`
**Size**: M
**Depends on**: E01, E02, E04 (foundations)
**Source**: catalogue §16, F-26.6. Spec-derived from the feature catalogue and not yet validated. Items marked *inferred* are analyst or PO inferences.

Picture Compare puts two images side by side with a difference pane that shows where they differ and by how much. It offers tolerance, mismatch-range and blend modes, plus tools to line up images that were shifted, scaled or rotated. Designers and QA use it to check visual changes and regressions.

## Behavioral contract
- **Data**:
  - *Images*: decoded to 32 bpp pixel data. HDR and wide gamut are not supported.
  - *Picture format*: file-format kind Picture, with masks.
  - *Per-pixel result*: match, unimportant difference (within the Tolerance), or important difference.
  - *View state*: difference mode, Tolerance, Blend percentage, zoom, rotation and flip per image, and the crop offset of the right image.
  - *Session settings*:
    - Specs: files and Description.
    - Format: picture format per side.
    - Replacements: colour pairs.
- **Routes/endpoints** (desktop app: commands, settings and CLI):
  - Home and Session > New Session ▸ Picture Compare.
  - The CLI with two files that resolve to a Picture format.
  - Session Settings tabs Specs, Format and Replacements.
  - View commands: modes, zoom, rotate, flip and full screen.
  - The Picture colours options page.
  - Copy, and File > Open Clipboard.
- **Integrations**:
  - E02-S1: the file-format registry, for the Picture kind.
  - E04-S1: reads the files.
  - The platform's image codecs and the clipboard.
- **Rules/states**:
  - Only 32 bpp data is compared.
  - Tolerance mode classes every pixel. Ignore Unimportant Differences counts differences within the Tolerance as matches.
  - Picture formats are load-only.
  - Scale, rotate, flip and offset change only the comparison, never the files (*inferred*).

---

## Stories

### E17-S1 — Picture Compare view
**Priority**: Should · **Layer**: 1 · **Size**: L · **Platforms**: All
**Needs**: E01-S1 (application shell), E02-S1 (file-format registry), E04-S1 (file-system access layer)
**Traces**: F-16.1, F-16.2, F-16.3, F-16.4
**Not in this story**: Mismatch Range, Blend, checkerboard transparency and metadata → E17-S2; scale, rotate, flip, offset and full screen → E17-S3; colour replacements → E17-S4; extra formats → E17-S5; child-session commands → E06-S11; report → E18-S4; Compare Files Using → E06-S9; Compare Parent Folders → E06-S10.

As a designer, I can compare two versions of an image with a difference pane so that I see which pixels changed and by how much.

**Acceptance criteria**
- [ ] Built-in Picture formats are registered for GIF, ICO, JPEG (`*.jpg`), PNG, TIFF (`*.tif`) and BMP. Two such files open in Picture Compare from the CLI, from Home, or through Session > New Session ▸ Picture Compare.
- [ ] Only 32 bpp data is compared. An image in an unsupported colour space (HDR, wide gamut) reports that it can't be compared exactly (*inferred* message).
- [ ] There are left, right and difference panes. The difference pane sits between or beside the images.
- [ ] In Tolerance mode, each pixel is either a match (shades of gray), an unimportant difference within the Tolerance (blue) or an important difference (red). Changing the Tolerance value updates the difference pane.
- [ ] View > Ignore Unimportant Differences counts differences within the Tolerance as matches.
- [ ] Zoom In, Zoom Out, Actual Size and Fit to Window apply to all panes together (*inferred*).
- [ ] The Specs tab holds the files and a Description. The Format tab sets a picture format per side.
- [ ] Swap Sides, Reload, Recompare and Picture Compare Info work. Info shows the dimensions and pixel difference counts (*inferred* content).
- [ ] Copy puts the current image on the clipboard. File > Open Clipboard loads a clipboard image into the chosen pane, and File > Open File loads a file into the chosen pane.
- [ ] File Info shows each image's details (*inferred*: dimensions, file size, modified time). Display Font for listings (hidden by default) changes the font of the view's text.

#### Tasks
- **E17-S1-T1** — Register Picture formats and decode the default types to 32 bpp · size: M · depends: — · flags: `licence-check`, `craft-libs`
  - AC: a PNG, a GIF and a BMP each load, and an unsupported colour space shows a message.
- **E17-S1-T2** — Pixel comparison in Tolerance mode with a Tolerance setting · size: M · depends: E17-S1-T1 · flags: `craft-libs`
  - AC: in two images that differ by 3 levels in one pixel, that pixel is blue at Tolerance 5 and red at Tolerance 2.
- **E17-S1-T3** — Left, right and difference panes with synchronised zoom · size: M · depends: E17-S1-T2 · flags: `naming-check`
  - AC: Fit to Window fits both images, and Actual Size shows 1:1 pixels in every pane.
- **E17-S1-T4** — Ignore Unimportant Differences and Info statistics · size: S · depends: E17-S1-T2
  - AC: with the toggle on, the blue pixel shows as a match and Info counts it as same.
- **E17-S1-T5** — Specs and Format tabs, Swap Sides, Reload, Recompare, Open File, Open Clipboard, Copy and File Info · size: M · depends: E17-S1-T3
  - AC: an image copied from another app opens through Open Clipboard. Copy then pastes the same image into another app.

### E17-S2 — Mismatch-range and blend modes
**Priority**: Should · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E17-S1 (Picture Compare view)
**Traces**: F-16.2, F-26.6 · **Open questions**: Q21

As a QA engineer chasing a visual regression, I can switch the difference pane to show how much pixels differ, or blend the two images, so that subtle shifts and colour changes become obvious.

**Acceptance criteria**
- [ ] Mismatch Range Mode shows matches in black and differences in yellow, with the brightness showing the degree of difference.
- [ ] Blend Mode overlays the images at a Blend percentage; a lower percentage lets the right image dominate. Blend Toggle flips between left and right.
- [ ] Show transparency as checkerboarding draws transparent areas as a checkerboard.
- [ ] The Compare Metadata toggle compares or ignores image metadata, as settled in T1.
- [ ] The Picture colours options page sets the Tolerance-mode backgrounds for same, different and similar pixels, and holds the Show transparency as checkerboarding option.

#### Tasks
- **E17-S2-T1** — Decide which metadata Compare Metadata covers and how it is shown (Q21) · size: S · depends: — · flags: `decision`
  - AC: decision recorded. Proposed default: EXIF, IPTC and XMP fields, plus DPI and colour profile name. Metadata differences are listed in Picture Compare Info and count towards the overall result while the toggle is on.
- **E17-S2-T2** — Mismatch Range Mode · size: S · depends: — · flags: `naming-check`
  - AC: a pixel that differs by 10% shows a darker yellow than one that differs by 90%.
- **E17-S2-T3** — Blend Mode with Blend percentage and Blend Toggle · size: M · depends: — · flags: `naming-check`
  - AC: at 0% the right image fills the pane. Blend Toggle swaps which image dominates.
- **E17-S2-T4** — Checkerboard transparency and the Picture colours page · size: S · depends: —
  - AC: a PNG with an alpha channel shows a checkerboard where it's transparent. Changing the colour for different pixels recolours the important pixels.
- **E17-S2-T5** — Compare Metadata toggle · size: M · depends: E17-S2-T1 · flags: `licence-check`, `craft-libs`
  - AC: two pixel-identical JPEGs with different EXIF dates compare as different with the toggle on, and as same with it off.

### E17-S3 — Geometric alignment
**Priority**: Should · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E17-S1 (Picture Compare view)
**Traces**: F-16.3

As a designer comparing a resized or re-exported image, I can scale, rotate, flip and shift the images so that the difference pane compares the content rather than the framing.

**Acceptance criteria**
- [ ] Auto Scale enlarges the smaller image to match the larger one in height or width.
- [ ] Rotate Clockwise and Rotate Counterclockwise turn the current image by 90°. Flip Horizontally and Flip Vertically flip it.
- [ ] Dragging on the difference pane moves the right image's top-left corner (the crop offset). With the difference pane focused, the arrow keys nudge the offset by 1 px and Ctrl+arrow nudges it by a larger step.
- [ ] Reset Difference Offset, or clicking the offset label, resets the offset to (0,0).
- [ ] Full Screen shows the chosen pane full screen, and Esc exits.
- [ ] These transformations change only the comparison, never the files (*inferred*).

#### Tasks
- **E17-S3-T1** — Auto Scale · size: S · depends: —
  - AC: a 100×100 and a 200×200 version of the same image compare as matching, within the Tolerance, after Auto Scale.
- **E17-S3-T2** — Rotate and Flip the current image · size: S · depends: —
  - AC: an image rotated 90° in an editor matches the original after Rotate Counterclockwise.
- **E17-S3-T3** — Crop offset by dragging and keyboard nudges, with reset · size: M · depends: —
  - AC: an image shifted 3 px to the right matches after three Left-arrow nudges. The offset label shows the offset and resets it when clicked.
- **E17-S3-T4** — Full Screen per pane · size: S · depends: —
  - AC: Full Screen on the difference pane fills the screen, and Esc returns to the normal layout.

### E17-S4 — Colour replacements
**Priority**: Could · **Layer**: 2 · **Size**: S · **Platforms**: All
**Needs**: E17-S1 (Picture Compare view)
**Traces**: F-16.4

As a designer checking a re-themed icon set, I can declare that one colour on one side corresponds to another colour on the other side so that intentional colour swaps don't count as important differences.

**Acceptance criteria**
- [ ] The Replacements tab lists colour pairs: colour X on one side, colour Y on the other. Pairs can be added, edited and deleted.
- [ ] A pixel that is X on one side and Y on the other counts as an unimportant difference. It shows blue in Tolerance mode, and as a match while Ignore Unimportant Differences is on.
- [ ] Replacements are saved with the session settings.

#### Tasks
- **E17-S4-T1** — Replacements tab with colour-pair items · size: S · depends: —
  - AC: an added `#FF0000` ↔ `#0000FF` pair shows in the list, and it's still there after the settings are reopened.
- **E17-S4-T2** — Apply replacements during comparison · size: S · depends: E17-S4-T1
  - AC: an image whose red pixels have all turned blue shows only unimportant differences.

### E17-S5 — Extra image formats
**Priority**: Could · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E17-S1 (Picture Compare view)
**Traces**: F-16.1

As a designer working with source and print formats, I can compare any other image format the platform can decode so that I don't have to convert files first.

**Acceptance criteria**
- [ ] Codec-dependent formats are registered as Picture formats only where the platform can decode them:
  - PDF (Mac), Alias/Wavefront, ICNS (Mac), Autodesk, camera RAW formats (Mac), Dr. Halo, EPS, FAX (Mac);
  - JPEG variants, JPEG 2000 (Mac), JPEG XR/HD Photo (Win), Kodak Photo-CD;
  - PICT (Mac), MacPaint (Mac), misc (`cur`, `epi`, `exr`, `hdr`, `jps`; Mac), Netpbm, Photoshop PSD/PDD, PostScript (Mac), QuickTime image (Mac), SGI;
  - TIFF (`*.tiff`), Truvision TGA, Windows DIB/RLE, EMF/WMF, XPM (Mac) and PCX.
- [ ] On a platform with no codec for a format, that format isn't offered, and its files fall back to another view (*inferred*).
- [ ] Decoded images go through the same 32 bpp comparison as the default types.

#### Tasks
- **E17-S5-T1** — Codec discovery and per-platform registration · size: M · depends: — · flags: `licence-check`, `craft-libs`
  - AC: on each platform, the registered Picture formats are exactly those whose sample file loads.
- **E17-S5-T2** — Per-format sample conformance · size: M · depends: E17-S5-T1
  - AC: a sample of each registered format loads and compares as same against itself.
