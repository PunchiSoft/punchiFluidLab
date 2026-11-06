# Delivery plan

How the interface described in [ui-ux.md](ui-ux.md) gets built, in an order where
every step ends with something usable and verifiable. Each milestone names its
acceptance criteria and the level of evidence it needs; "done" means the evidence
exists, not that the code compiles.

Rules that hold for every milestone:

- `./scripts-dev/check.sh` green, no new warnings.
- A behavioural test for anything that can be tested headlessly; screenshots for
  anything visual; a measurement for anything about memory or speed.
- New decisions get an ADR before the code, not after.
- `CHANGELOG.md` updated under `Unreleased` in the same change.

## M0 — Foundation (done)

Domain, catalog of definition files, gas model, canvas, CLI.

**Evidence:** 37 tests, ISO 6358 checked against a hand calculation, mass balance to
1e-9, stroke time bracketed by two flow bounds, screenshots of the running window, a
user catalog file overriding a built-in one in the real session.

## M1 — Application shell

Ribbon, sidebar, sheet frame, status bar, theme layer.

- `punchi-fluid-theme` crate: reads `kdeglobals`, the colour-scheme portal and the
  font, exposes semantic tokens, builds `egui::Visuals`/`Style`. Nothing else may
  build a colour.
- `punchi-fluid-ui`: `Ribbon`, `Sidebar`, `SheetFrame`, `StatusBar` widgets. Ribbon
  collapses below 1100 px.
- `punchi-fluid-desktop`: wires them, enables `accesskit`, handles theme changes at
  runtime.

**Acceptance**

1. The window renders with the desktop's own colours, in light and dark, without
   restarting.
2. `grep` finds no literal `Color32::` outside the theme crate (enforced by a test
   that scans the sources).
3. Every ribbon action is reachable with the keyboard; the shortcuts are listed in a
   help panel.
4. Contrast test: text ≥ 4.5:1, lines ≥ 3:1, for the dark and the light palette.
5. Screenshots in both schemes, in the sibling project's style.

## M1.5 — Symbol conformance (runs with M2)

**Status:** structural half and visual review done; the 300 dpi acceptance (2, 4) and
the export note (5) need M2. See [symbology.md](symbology.md).

Close the gap between what is drawn today and what ISO 1219-1:2012 requires. The
contract is [symbology.md](symbology.md); this milestone is what executes it.

- Renderer: port numerals drawn from the `port` table; line classes with their own
  dash pattern; blocking mark at closed ports; junction dots at three-way junctions.
- Schema: `actuator` and `spring` declared as parts, line class declared per polyline.
- `symbol lint` inside the catalog validation, with the structural checks listed in
  [symbology.md](symbology.md).
- Built-in definitions redrawn to conform: `valve.3_2.spring_return`,
  `supply.workshop`, `cylinder.single_acting_20_100`, `throttle.fixed`.
- One review file per symbol under `assets/symbols/review/`, with light and dark
  screenshots at palette size and at 300 dpi.
- While the review is missing, the export carries the note "symbols not yet verified
  against ISO 1219-1" in its title block.

**Acceptance**

1. `symbol lint` fails on a definition with a missing port numeral, a ragged box, a
   missing spring on a `spring_return` valve, a primitive outside the box, or a
   polyline without a line class. One test per rule.
2. Every built-in definition passes the lint.
3. Every built-in symbol has a review file; the export stops adding the unverified
   note only when all of them do — asserted by a test that reads the review folder.
4. Visual review of the four symbols against Part 1, recorded in the pull request.
5. The legend of line classes and port numerals appears in the exported title block.

## M2 — One drawing, two exports

**Status:** done. The drawing layer, the `Surface` trait, the page geometry, the PDF and
PNG backends, the eight acceptance tests and the export panel are implemented. The note in
the title block stays while the per-symbol 300 dpi reviews are open.

PNG and PDF of the sheet, with standard paper, margins and a title block.

This is the milestone the project owner asked for, and it is cheap **only** if the
drawing is not duplicated: the sheet and the palette draw through a `Surface` trait,
and the export backends implement the same trait.

- New `punchi-fluid-draw` crate: geometry in millimetres, `Surface` trait
  (line, polyline, polygon, rect, circle, arc, text), symbol and sheet drawing.
  `punchi-fluid-ui` implements `Surface` over `egui::Painter`; the canvas keeps
  reporting clicks.
- New `punchi-fluid-export` crate: `Page` (A5/A4/A3/A2/Letter/Legal, portrait or
  landscape, margins, title block), `PngBackend` (`tiny-skia` + `ab_glyph`, text from
  the system font database), `PdfBackend` (`printpdf`, vector, Helvetica).
- Export panel in the ribbon: paper, orientation, margins, title block, DPI (PNG),
  destination, preview of the resulting pixel size, and the estimated file size.

**Acceptance**

1. A4 portrait at 300 dpi produces 2480 × 3508 px; A3 landscape at 300 dpi produces
   4961 × 3508 px. Tested.
2. The margin band is empty and the drawing is inside the content rectangle. Tested
   by sampling pixels in both regions.
3. The PDF opens, is one page, its MediaBox matches the chosen paper within 1 pt, and
   it contains no path, user name or host name. Tested by parsing the header and
   scanning for the strings.
4. Exporting twice with the same inputs produces byte-identical PNG and PDF
   (deterministic, no timestamps beyond the title block date). Tested.
5. The sheet on screen and the exported page differ only in scale: same layout call,
   verified by drawing both through a recording backend and comparing the call lists.
6. A refused export (DPI above 600, paper that would exceed the pixel budget) fails
   with a message and never allocates the bitmap.

## M3 — Document, undo, palette

The editable document.

- New `punchi-fluid-doc` crate: `Document` (components placed by catalog `id`, pipes
  as waypoints), `Command` enum, undo/redo with `Arc` snapshots, dirty tracking,
  autosave.
- File format v2: a placed component references a catalog id. This retires
  `ComponentKind` from the document layer (ADR 0002).
- Sidebar drag-and-drop, symbol thumbnails, placement with grid and port snapping.

**Acceptance**

1. Place, move, delete, undo and redo 100 operations; the document returns to a byte
   identical state at every step back. Property test over random command sequences.
2. A document referencing a missing catalog id opens with a visible, actionable
   error and the component drawn as a placeholder, not as a crash.
3. The undo history never exceeds 100 entries and 32 MB (counting allocator).
4. Saving is atomic: a crash between write and rename leaves the previous file
   intact. Tested with a temporary directory and a simulated failure.
5. Palette thumbnails are the definition's own symbol, proven by a test that draws
   the same definition twice and compares the call list.

## M4 — Connections and the real solver

From a fixed three-component layout to a graph.

- Ports, connections, orthogonal pipe routing with waypoints.
- `punchi-fluid-solver`: network of volumes from the graph, conductance matrix per
  valve position, implicit or substepped explicit integration with the same
  stability bound as today.
- Edit/run mode split; valve actuation recorded as an input trace for replay.

**Acceptance**

1. A 5/2 valve dropped in as a **file only**, with no code change, drives a
   double-acting cylinder. This is the modularity test the project exists for.
2. Mass balance over the whole network to 1e-9 relative, for a circuit with three
   volumes and four restrictions.
3. A circuit with a closed valve holds its pressure for 60 s of simulated time
   without drift beyond 1 %.
4. Replaying an input trace reproduces the same positions bit for bit.
5. 200 components at 60 fps: solver under 4 ms per frame (measured, not estimated).

## M5 — Instruments

Manometer, flow meter, force meter, and a strip chart of pressure and position.

- Instruments are catalog entries with a `behaviour` variant and a `reading` in the
  telemetry struct.
- Telemetry ring buffer: 10 Hz × 60 s, fixed capacity, no allocation after start.

**Acceptance**

1. The chart shows 60 s of history and drops old samples without growing.
2. A gauge reads the same value as the status bar within 0.1 %.
3. No allocation in the simulation loop (counting allocator test over 10k frames).

## M6 — Language, packaging, 1.0

- `i18n/` with Fluent, English and Spanish complete, no string concatenation.
- AppImage and Flatpak manifests, desktop entry, AppStream metadata, Breeze icon in
  hicolor sizes.
- Signed release with checksums; `cargo-deny` and `cargo-audit` clean.
- Documentation: user guide with the three workflows (draw, simulate, export),
  screenshots, and a troubleshooting page for "the catalog file is ignored".

**Acceptance**

1. A fresh machine installs the Flatpak, opens a document, exports a PDF.
2. `cargo deny check` and `cargo audit` clean, licences compatible with GPL-3.0-or-later.
3. Every user-visible string comes from a translation file; a test fails on a
   hardcoded string in the UI crate.
4. Reproducible build: two builds of the same tag produce the same checksum with
   `SOURCE_DATE_EPOCH` set.

## Risks, and what is done about them

| Risk | Mitigation |
|---|---|
| The UI toolkit stops being enough (accessibility, native look) | Drawing is toolkit-agnostic from M2; a QML front end through `cxx-qt` becomes a new `Surface` implementation, not a rewrite |
| Ribbon turns into a maze | Four tabs, overflow menu, every action keyboard-reachable, screenshot review per milestone |
| The catalog schema grows until it is a programming language | Behaviour stays a small closed enum; anything that needs logic is a new variant with an ADR |
| Export drifts from the screen | One `Surface` trait, call-list comparison test |
| Solver slows down with many components | Budget test at 200 components from M4, profiling before optimising |
| Memory creep from caches | Every cache has a bound and a test that fills it past the limit |
