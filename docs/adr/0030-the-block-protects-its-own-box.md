# 0030 — The title block protects its own box, not a band

- **Status:** accepted
- **Date:** 2026-11-18
- **Deciders:** project owner, implementation session
- **Context crates:** `punchi-fluid-ffi`, `punchi-fluid-export`, `punchi-fluid-check`

## Context

The session reserves the title block by subtracting its **height across the whole inner
width** of the sheet: `content_rect` in the ABI and `drawing_rect` in the export both do it,
and a drop or a move is clamped above that band. The reason is sound — a symbol printed
behind the block is a symbol nobody reads — but the shape is wrong.

On A4 landscape the content area is 267 mm wide and the industrial block is **170 × 45 mm**
in the standard bottom-right position. Reserving 45 mm across the full 267 mm protects
**7,650 mm²** with a **12,015 mm²** band: the 97 mm strip to the left of the block, 4,365 mm²
or **36 % of the band**, is dead paper that nothing will ever cover. The owner reported it
while drawing: the lower-left corner of the sheet cannot take a symbol, and in the references
it can.

The open question in [ui-ux.md](../ui-ux.md) — whether hiding the block releases its band —
is part of the same defect: the reservation is not the block's footprint, so it cannot follow
the block's position, width or a parts list that grows upward from it.

## Decision

- **The protected region is the block's own rectangle**, from the resolved layout: its width,
  its height and its position (top right, bottom right, or edge to edge at the bottom). The
  parts list, which hangs above the block, protects its own rectangle too, plus the same
  clearance. A shape is not a quantity: the block is 170 mm wide or it is not.
- **The free drawing area is the frame minus those rectangles**, an L when the block is in a
  corner, a band only when the position is genuinely edge to edge.
- **The clamp follows the width.** A drop or a move is pushed out of a protected rectangle
  only when the box falls inside its horizontal range; outside it, the symbol may go down to
  the frame. The user's horizontal choice is never moved.
- **The export draws the circuit in the whole frame** and paints the block and the list over
  it, instead of clipping the drawing to a band.
- **`Sin cajetín` frees the frame.** Nothing is printed there, so nothing is reserved: a
  phantom reservation is a rule the reader cannot see. If a block is enabled later over a
  component, the **check reports it** — a fault shown where it happens, with the component
  named — and the student moves it; that is cheaper to understand than a space that stays
  forbidden after the block is gone.
- **The reserve is dynamic and stores nothing.** Width, height and position come from the
  layout that is already resolved each time the sheet changes; a shorter model, a hidden
  field, an empty row or a wider parts list moves the protected rectangle with no new field
  and no format bump.
- **The example is seeded clear of the protected rectangles**, not above the band, so the
  first circuit shows what the rule allows.

## Amendment (2026-11-18, with the implementation)

The parts list is **not on the canvas**: it exists only on the printed page, where the export
already gives it a column and fits the drawing into what remains. Reserving it live would
forbid paper the student can see, which is the phantom rule this record rejects, so the canvas
protects the block alone and the page keeps its own fitting. The starting circuit is composed
in the largest rectangle the block leaves free, so the worked example keeps the shape it was
drawn with while a user's own drop may still use the strip beside the block.

## Consequences

- `content_rect` and the canvas clamp in the ABI change their meaning; the export's
  `drawing_rect` and the clipping change with them. The tests that assert the band become
  tests of the rectangle.
- The check gains one fault: a component whose box overlaps the block or the parts list.
  The ledger's "nine faults" becomes ten, and the new one is only reachable when a block is
  enabled over an existing drawing.
- The parts list already borrows its width from the block; it now also has to reserve its
  height while it grows.
- No document field, no format version: the rule is derived from what the sheet already
  stores.
- Symbols, fixtures and the 300 dpi review are untouched.

## Alternatives considered

- **Keep the band.** Simple and already written, but it spends 36 % of the A4 band on paper
  no block covers, and the owner met the cost while drawing.
- **Reserve the footprint only while the block is visible, and keep it afterwards.** A
  phantom reservation prevents the drawing from drifting when the block returns, but it
  forbids paper nobody prints on for a reason the reader cannot see; the check states the
  collision in words instead.
- **Allow drawing under the block with no fault.** It matches a CAD's freedom and teaches the
  wrong lesson: the printed sheet hides half a valve, and the student finds out after
  printing.

## References

- ISO 5457 (frame and title block) and `NCh 14.Of93` / ISO 7200 for the block's fields, as
  recorded in [reference/title-block.md](../reference/title-block.md).
- [ADR 0026](0026-the-sheet-is-the-paper.md): the sheet is the paper the drawing is printed
  on, which is why the protection exists.
- [ui-ux.md](../ui-ux.md) §3, the sheet, and its open question on the band.
