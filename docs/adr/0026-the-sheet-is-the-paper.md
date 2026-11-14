# 0026 — The sheet is the paper the drawing is printed on

- **Status:** accepted
- **Date:** 2026-11-13
- **Context crates:** `punchi-fluid-core`, `punchi-fluid-doc`, `punchi-fluid-file`, `punchi-fluid-export`, `punchi-fluid-ffi`, `punchi-fluid-cli`, `punchi-fluid-qt`
- **Supersedes:** the "paper is a preference, not a document property" consequence of [ADR 0012](0012-the-paper-a-drawing-is-printed-on.md); its sizes and the rest of its rules stand

## Context

The working canvas was a fixed 200 × 130 mm logical area, inherited from an earlier
front end, while the paper was an export preference chosen per print. The screen and the
printed page therefore measured different sheets: a drawing could fit on screen and be
cropped in the file, or the other way round, and the check's "outside the sheet" referred
to a rectangle nobody could see. A schematic is read and corrected on a sheet, and the
sheet a course hands in is the one the reader has in the printer.

The authority is the paper itself, not another program's page-setup dialog: ISO 216 for
the sizes (A0–A5, plus Letter, Legal and the Chilean Oficio this project is developed
against) and ISO 5457 for the frame — 20 mm on the filing edge and 10 mm on the other
three. The device is a desktop, the interface follows [ADR 0005](0005-qt-quick-over-a-c-abi.md),
and nothing about the page setup is copied from a third-party product.

## Decision

- **Paper, orientation and frame are domain values.** `PaperSize`, `Orientation`,
  `Margins` and `Sheet` live in `punchi-fluid-core::paper`; `punchi-fluid-export`
  re-exports them so its API does not change for a caller. A sheet is a physical
  measurement, like a pressure, and it has one definition.
- **The document stores one sheet.** `Document` gains a `sheet`, defaulting to A4
  portrait with the ISO 5457 frame. Choosing a paper, an orientation or a margin is one
  undoable document change, because it changes what fits and it is saved.
- **The file carries it.** Format 7 stores the paper, the orientation and the four
  margins. A file written before format 7 reads as an **unframed A4**: its geometry was
  drawn on a bare 200 × 130 mm area, so no component becomes "outside the frame" just by
  opening it in a newer build. A new document starts framed.
- **The canvas is the paper.** The front end asks the document for the sheet, fits the
  paper into the viewport, draws its edge and the frame, and keeps the grid inside the
  drawing space.
- **The check measures the frame.** "Outside the sheet" now means outside the drawing
  space the frame leaves free, the same rectangle the canvas draws and the export uses.
- **The export prints the document's sheet.** The paper, orientation and margins come from
  the document. The export panel may override them with a custom size for one print, and
  still owns the raster resolution and the parts list.
- **The new edges are the base crate.** `doc`, `export` and `ffi` now depend on
  `core` directly, because the sheet is a domain value; `core` depends on nothing of
  ours, so no cycle is possible.

## Consequences

- A file saved before this change opens as an unframed A4 with its drawing where it was;
  a new file opens framed. The reader can pick another paper at any time and undo it.
- The default export margins change from 18 mm on four sides to the ISO 5457 20/10, which
  is what ADR 0012 already promised. The margin-band test measures each side on its own.
- The statement of ADR 0012 that "paper is a preference, not a document property" is
  reversed; the paper is now the document's. Remembering the last choice per user is no
  longer the mechanism: the file is.
- The check can report a component in the frame's margin, where it truly is outside the
  drawing space, instead of at the old 200 × 130 mm boundary.
- The worked example's layout uses 0.17 of the drawing width per symbol square, not 0.20,
  so the four boxes fit a portrait A4 and can be spread with positive gaps.
- **Still open:** a sheet per circuit (all circuits of a project share one paper for now),
  a custom size as the working sheet (today it is a print override), the title block drawn
  on the canvas, a user-chosen drawing scale, and the ISO 5457 centring marks and zone
  grid.

## Alternatives considered

- **Keep the paper per export and make the canvas mimic it.** The screen and the file can
  disagree, and the check would keep measuring a rectangle that is not the paper. That is
  the defect this decision removes.
- **Hold the sheet only in the ABI session.** The file would still need it, and the
  document's revision would need a second change counter or a flag to know it was
  modified; two sources of truth for a saved value is the bug that follows.
- **A sheet per circuit now.** A circuit is a sheet, so this is the eventual shape, but it
  multiplies the format surface and the selector's behaviour before the single sheet is
  proven. It is queued, not forgotten.
