# preparation.service_unit_simple — conformance review

    status: complete
    date: 2026-11-24
    reviewer: Cristian Muñoz (owner), second reader
    function: service unit, simplified representation

## The claim

- **Function, in one sentence**: the same filter, regulator and lubricator as the full
  service unit, drawn as one body for a circuit that uses the unit and does not explain it.
- **Family**: conditioning (one block with two ports).
- **Normative source**: ISO 1219-1:2006, 7.4.4.17, figure `X12160`, the **simplified
  representation below** the detailed one, read in the owner's copy; confirmed on the 2012
  basis in JIS B 0125-1:2020, 7.4.4.17, which prints both representations. Rules: `RF052`
  and `RF053`.
- **Ports**: `1` supply at the left · `2` working at the right.

## Reference and assumptions

- **Reference read**: the standard's `X12160` simplified figure. No capture of another
  application was used for the shape.
- **Stated by the standard**: one rectangle with the **dashed element line at the left**, the
  **gauge circle with its needle** in the middle, and a **short line on the top edge at the
  right**. It prints no drain in this representation.
- **Assumption**: the short line at the right is the mark the standard carries for the
  lubricator, reduced to a tick; the standard's figure does not label it, and the review
  records the position and the form without inventing a meaning for it.
- **Assumption**: the tick hangs from the top edge at x 16 mm, a quarter of the body's height
  long.
- **Correction**: the earlier file drew its own circles at the two ports (the renderer draws
  one per port, so each capture showed a double ring) and a drain arrow leaving the bottom,
  which the standard does not print in this form. Both are gone.

## Measures

| Feature | Requirement | Drawing |
|---|---|---|
| Body | one rectangle, wider than tall | x 2,8–17,2, y 6,8–13,2 (14,4 × 6,4 mm) |
| Element | dashed line at the left, full height | x 5,6, y 6,8–13,2 |
| Gauge | circle with the needle as an arrow from its centre | centre (12; 10), r 2,3 mm; needle to (10,5; 8,5) |
| Lubricator mark | short line from the top edge at the right | x 16, y 6,8–8,4 |
| Drain | none in this representation | — |
| Ports | both drawn, one per side | `1` at (1,2; 10), `2` at (18,8; 10) |
| Numerals | from the port table, clear of the body | `1` above the left stub, `2` below the right stub |

## Evidence

- 300 dpi sheet, print ink: `preparation.service_unit_simple-300.png`.
- Light and dark captures: `preparation.service_unit_simple-light.png`,
  `preparation.service_unit_simple-dark.png`.
- Symbol lint: passes for the built-in.
- Geometry fixture regenerated and read:
  `assets/symbols/fixtures/preparation.service_unit_simple.txt` (the drain arrow and the
  file-drawn connection circles are gone; the needle is an arrow from the centre; the tick at
  the right is new).

## Checklist

| Question | Answer |
|---|---|
| Is every port the standard draws drawn? | yes: `1` and `2` |
| Do the numerals match the declared ports? | yes: `1` supply, `2` working |
| Is the element a dashed line at the left? | yes, full height |
| Is the gauge a circle with an arrow-needle? | yes |
| Is the right-hand mark drawn? | yes: the tick on the top edge |
| Is a drain drawn that the standard does not print? | no: removed |
| Is the symbol legible at palette size and at 300 dpi? | yes, both captures |

## What is still open

- The 2012 text is read through the Japanese adoption, not the ISO text itself.
- The tick's meaning is not stated by the figure; the review records what is drawn.
- The model is the same fixed restriction as the full unit: it does not regulate, filter or
  lubricate.

## Sign-off

- **Status**: `complete`
- **Reviewed by**: Cristian Muñoz (owner)
- **Date**: 2026-11-24
- **What changed since the last record**: nothing in the record changed; the
  second reader signed its measures and open items on 2026-11-24.

## Images

- `preparation.service_unit_simple-light.png`,
  `preparation.service_unit_simple-dark.png`
- `preparation.service_unit_simple-300.png`
