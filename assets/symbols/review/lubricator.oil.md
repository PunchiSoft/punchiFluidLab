# lubricator.oil — conformance review

    status: complete
    date: 2026-11-24
    reviewer: Cristian Muñoz (owner), second reader
    function: oil lubricator

## The claim

- **Function, in one sentence**: oil is added to the air stream so the moving parts
  downstream are lubricated.
- **Family**: conditioning (a two-port part).
- **Normative source**: ISO 1219-1:2006, 7.4.4.25, figure `X12240`, read in the owner's copy;
  confirmed on the 2012 basis in JIS B 0125-1:2020, 7.4.4.25, same registration number. Rules:
  `RF052` and `RF053`.
- **Ports**: `1` inlet at the left vertex · `2` outlet at the right vertex.

## Reference and assumptions

- **Reference read**: the standard's figure only. The owner's capture of a reference
  application had been used before this phase; it is not a shape authority and its arrow was
  removed.
- **Stated by the standard**: `X12240` is the same square turned 45° as the filter, with the
  flow crossing it from vertex to vertex and a **plain line from the top vertex toward the
  centre**. There is no dashed element, because nothing is filtered here.
- **Assumption**: the line carries **no arrowhead**. The standard prints a plain line; the
  head that an earlier record copied from the reference would make the mark read as a flow
  direction, and a flow arrow inside a diamond is a different statement.
- **Assumption**: the line stops before the centre, at 0,4 of the box, the length the figure
  prints (a little over half the top-to-centre distance).

## Measures

| Feature | Requirement | Drawing |
|---|---|---|
| Diamond | a square turned 45°, four equal sides | vertices (10; 5,6), (14,4; 10), (10; 14,4), (5,6; 10) |
| Oil mark | plain line from the top vertex toward the centre, no head | (10; 5,6)–(10; 8), 2,4 mm |
| Element | none: no dashed line | the diamond is empty below the oil mark |
| Flow | vertex to vertex, not through the inside | stubs to the left and right vertices |
| Ports | all of them drawn | `1` at (1,2; 10), `2` at (18,8; 10) |
| Numerals | from the port table, clear of the drawing | `1` above the left stub, `2` below the right stub |

## Evidence

- 300 dpi sheet, print ink: `lubricator.oil-300.png`.
- Light and dark captures: `lubricator.oil-light.png`, `lubricator.oil-dark.png`.
- Symbol lint: passes for the built-in.
- Geometry fixture regenerated and read: `assets/symbols/fixtures/lubricator.oil.txt` (the
  arrowhead and its two strokes are gone; the mark is one line).

## Checklist

| Question | Answer |
|---|---|
| Is every port the standard draws drawn? | yes: `1` and `2` |
| Do the numerals match the declared ports? | yes: `1` inlet, `2` outlet |
| Is the mark a line, with no arrowhead? | yes |
| Is there a dashed element, as on a filter? | no: the lubricator has none |
| Is the symbol legible at palette size and at 300 dpi? | yes, both captures |

## What is still open

- The 2012 text is read through the Japanese adoption, not the ISO text itself.
- The model adds no oil: the file is a two-port restriction, and the review does not claim
  lubrication.
- The earlier record described a "short arrow ... as the reference draws a lubricator"; the
  standard does not print the head, and the reference is not a shape authority
  ([ADR 0040](../../docs/adr/0040-the-owner-reference-authorizes-the-reconstruction.md)).

## Sign-off

- **Status**: `complete`
- **Reviewed by**: Cristian Muñoz (owner)
- **Date**: 2026-11-24
- **What changed since the last record**: nothing in the record changed; the
  second reader signed its measures and open items on 2026-11-24.

## Images

- `lubricator.oil-light.png`, `lubricator.oil-dark.png`
- `lubricator.oil-300.png`
