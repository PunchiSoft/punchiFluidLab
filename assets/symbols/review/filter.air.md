# filter.air — conformance review

    status: pending-300dpi
    date: 2026-11-24
    reviewer: automated pass, unreviewed by a second person
    function: air filter

## The claim

- **Function, in one sentence**: the air passes through the element and the particles it
  carries stay behind.
- **Family**: conditioning (a two-port part).
- **Normative source**: ISO 1219-1:2006, 7.4.4.1, figure `X11980`, read in the owner's copy;
  confirmed on the 2012 basis in JIS B 0125-1:2020, 6.4.4.1, same registration number. Rules:
  `RF052` (every port is drawn) and `RF053` (`1` inlet, `2` outlet).
- **Ports**: `1` inlet at the left vertex · `2` outlet at the right vertex.

## Reference and assumptions

- **Reference read**: the standard's figure only. No capture of another application was used
  for the shape.
- **Stated by the standard**: `X11980` is a square turned 45° with the flow crossing it from
  vertex to vertex and the **element as a dashed line across the diamond**, perpendicular to
  the flow. The figure prints it with the flow vertical and the element horizontal.
- **Assumption**: the file draws the same figure turned 90°, flow horizontal and element
  vertical, because the standard's own `X12190` prints this diamond with a horizontal flow.
  The rotation keeps the element perpendicular to the flow, which is what carries the
  meaning; no other orientation is used.
- **Assumption**: the working line stops at each vertex, so the inside of the diamond holds
  only the element.

## Measures

The symbol box is 20 mm; the normalised coordinates map to millimetres on both axes.

| Feature | Requirement | Drawing |
|---|---|---|
| Diamond | a square turned 45°, four equal sides | vertices (10,0; 5,6), (14,4; 10), (10; 14,4), (5,6; 10) |
| Element | dashed line across the diamond, vertex to vertex | (10; 5,6)–(10; 14,4), 8,8 mm |
| Flow | vertex to vertex, not through the inside | stub to the left vertex at x 5,6 and from the right vertex at 14,4 |
| Ports | all of them drawn | `1` at (1,2; 10), `2` at (18,8; 10) |
| Numerals | from the port table, clear of the drawing | `1` above the left stub, `2` below the right stub |

## Evidence

- 300 dpi sheet, print ink: `filter.air-300.png`.
- Light and dark captures, fitted: `filter.air-light.png`, `filter.air-dark.png`.
- Symbol lint: passes for the built-in.
- Geometry fixture regenerated and read: `assets/symbols/fixtures/filter.air.txt` (the dashed
  element now spans the full diagonal; the earlier record drew only its upper half).

## Checklist

| Question | Answer |
|---|---|
| Is every port the standard draws drawn? | yes: `1` and `2` |
| Do the numerals match the declared ports? | yes: `1` inlet, `2` outlet |
| Is the element across the flow, not along it? | yes, on the vertical diagonal |
| Is any working line drawn through the element? | no: it stops at the two vertices |
| Is the symbol legible at palette size and at 300 dpi? | yes, both captures |
| Is the drawing inside the 0..1 box with no degenerate primitive? | yes: lint |

## What is still open

- The second reader's signature.
- The 2012 text is read through the Japanese adoption, not the ISO text itself.
- The filter is modelled as a fixed restriction with a high conductance: it does not clog,
  and nothing separates the particles it traps.

## Images

- `filter.air-light.png`, `filter.air-dark.png`
- `filter.air-300.png`
