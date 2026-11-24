# filter.air_manual_drain — conformance review

    status: complete
    date: 2026-11-24
    reviewer: Cristian Muñoz (owner), second reader
    function: air filter with a manual drain

## The claim

- **Function, in one sentence**: the air passes through the element, the condensate collects
  in the bowl below it and is let out by hand through the drain.
- **Family**: conditioning (a two-port part with a drain).
- **Normative source**: ISO 1219-1:2006, 7.4.4.20, figure `X12190`, read in the owner's copy;
  confirmed on the 2012 basis in JIS B 0125-1:2020, 7.4.4.20, same registration number
  ("Filter (with manual drain)"). Rules: `RF052` and `RF053`.
- **Ports**: `1` inlet at the left vertex · `2` outlet at the right vertex. The drain is drawn
  as the standard draws it and is not declared a port: the model keeps the two working ports
  and does not empty a bowl.

## Reference and assumptions

- **Reference read**: the standard's `X12190` figure. No capture of another application was
  used for the shape.
- **Stated by the standard**: the same square turned 45° as the plain filter, with the
  element as the dashed line on the vertical diagonal, a **solid chord across the lower
  part** of the diamond (the bowl), and the **stem from the bottom vertex**.
- **Assumption**: the element stops at the chord, because the bowl occupies the space below
  it; that is the form the figure prints. The chord spans the diamond's lower edges.
- **Assumption**: the drain stem is drawn 0,9 of the box and is not a port. A hose cannot be
  attached to it in this build, and the review says so instead of implying a tank connection.

## Measures

| Feature | Requirement | Drawing |
|---|---|---|
| Diamond | the same square turned 45° as `filter.air` | vertices (10; 5,6), (14,4; 10), (10; 14,4), (5,6; 10) |
| Element | dashed, from the top vertex to the chord | (10; 5,6)–(10; 12,84), 7,24 mm |
| Bowl | solid chord across the lower part | y 12,84 from x 8,44 to 11,56 |
| Drain | stem from the bottom vertex | (10; 14,4)–(10; 18) |
| Ports | all of them drawn | `1` at (1,2; 10), `2` at (18,8; 10) |
| Numerals | from the port table, clear of the drawing | `1` above the left stub, `2` below the right stub |

## Evidence

- 300 dpi sheet, print ink: `filter.air_manual_drain-300.png`.
- Light and dark captures: `filter.air_manual_drain-light.png`,
  `filter.air_manual_drain-dark.png`.
- Symbol lint: passes for the built-in.
- Geometry fixture regenerated and read: `assets/symbols/fixtures/filter.air_manual_drain.txt`
  (the earlier record drew a bowl triangle and two inner strokes that the standard does not
  print; both are gone).
- Sheet format: version 15, unchanged by this phase.

## Checklist

| Question | Answer |
|---|---|
| Is every port the standard draws drawn? | yes: `1` and `2` |
| Do the numerals match the declared ports? | yes: `1` inlet, `2` outlet |
| Is the element across the flow, and does it stop at the bowl? | yes |
| Is the drain at the bottom vertex, not somewhere on an edge? | yes |
| Is the bowl a chord of the diamond, not a separate triangle? | yes |
| Is the symbol legible at palette size and at 300 dpi? | yes, both captures |

## What is still open

- The 2012 text is read through the Japanese adoption, not the ISO text itself.
- The drain is drawn and not modelled: the file declares two ports and the solver has no
  bowl to empty.
- The filter does not clog.

## Sign-off

- **Status**: `complete`
- **Reviewed by**: Cristian Muñoz (owner)
- **Date**: 2026-11-24
- **What changed since the last record**: nothing in the record changed; the
  second reader signed its measures and open items on 2026-11-24.

## Images

- `filter.air_manual_drain-light.png`, `filter.air_manual_drain-dark.png`
- `filter.air_manual_drain-300.png`
