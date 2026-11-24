# preparation.service_unit — conformance review

    status: pending-300dpi
    date: 2026-11-24
    reviewer: automated pass, unreviewed by a second person
    function: service unit — filter, regulator and lubricator — between the compressor and the circuit

## The claim

- **Function, in one sentence**: the air is filtered, its pressure is held at the set value
  and oil is added, so what leaves the unit is the line air the circuit is designed around.
- **Family**: conditioning (one block with two ports).
- **Normative source**: ISO 1219-1:2006, 7.4.4.17, figure `X12160`, read in the owner's copy;
  confirmed on the 2012 basis in JIS B 0125-1:2020, 7.4.4.17, same registration number. The
  standard describes the figure as a filter with manual drain, a pressure-relieving regulator
  with manual adjustment, a pressure gauge and a **lubricator**. Rules: `RF052`, `RF053`,
  `RF048` (the adjustable element's arrow goes through it).
- **Ports**: `1` supply at the left edge of the frame · `2` working at the right edge.

## Reference and assumptions

- **Reference read**: the standard's detailed `X12160` figure. No capture of another
  application was used for the shape.
- **Stated by the standard**: the functions are drawn in the order the air meets them inside
  a frame; the filter is the diamond with the element, the bowl chord and the drain stem; the
  regulator is the square with the flow through it, the two facing arrows, the spring with the
  diagonal adjusting arrow, the vent triangle on its left and the dashed sensing line from
  the outlet back to its underside; the gauge is the circle on a stem with an arrow from its
  centre; the lubricator is the diamond with the plain line at its top vertex.
- **Assumption**: the **ports sit on the frame's edges**. The standard draws the working line
  crossing the frame to ports outside it; inside a 20 mm box the 1,2 mm margin outside the
  frame is the only room a numeral has, so the frame is the envelope and the connection point
  is the unit's edge. The review records it instead of pretending the placement is the
  figure's.
- **Assumption**: the standard's FRL is drawn wider than tall; the project's square box
  compresses the band, so the lubricator is smaller than the filter. Both are the same shape,
  and the marks that carry the meaning are all present.
- **Deviation, recorded**: the standard draws the frame as an assembly boundary, a chain
  line; the renderer's class set has no chain pattern, and the contract says "dashed frame",
  so the frame uses the dashed class. Open item below.

## Measures

The symbol box is 20 mm; the frame is 17,6 × 15,2 mm.

| Feature | Requirement | Drawing |
|---|---|---|
| Frame | around the functions, one group | x 1,2–18,8, y 2,4–17,6 |
| Filter | diamond, element dashed to the chord, bowl, drain stem | centre (4,4; 10), half-diagonal 2,4 mm; element to y 11,55; chord y 11,55; drain to y 19 |
| Regulator | upright square, flow through it, two facing arrows | square (7,4; 8)–(11; 11,6); arrow tips at x 7,7 and 10,7 on y 10 |
| Vent | open triangle on the regulator's left | base x 7,4, y 8,6–9,4; apex (6,7; 9) |
| Spring | above the square, four peaks | x 8,4–10,8, y 5,2–8 |
| Adjustability | arrow through the spring, filled head | (8; 7,6)–(12; 4,9) |
| Sensing line | dashed, from the outlet to the underside | stub (8,4; 11,6)–(8,4; 12,8); loop to (11,7; 12,8) and up to y 10 |
| Gauge | circle on a stem, needle as an arrow from the centre | centre (12,4; 6,3), r 1,1; stem from y 10; needle to (11,7; 5,6) |
| Tap | filled dot where the stem meets the line | (12,4; 10), r 0,18 |
| Lubricator | diamond with the plain line at the top vertex | centre (15,4; 10), half-diagonal 1,8 mm; line to y 8,84 |
| Numerals | from the port table, clear of the frame and the parts | `1` and `2` inside the frame, clear of the edges |

## Evidence

- 300 dpi sheet, print ink: `preparation.service_unit-300.png`.
- Light and dark captures: `preparation.service_unit-light.png`,
  `preparation.service_unit-dark.png`.
- Symbol lint: passes for the built-in.
- Geometry fixture regenerated and read: `assets/symbols/fixtures/preparation.service_unit.txt`
  (the lubricator, the vent, the sensing line, the bowl chord, the tap dot and the needle
  arrow are new; the two file-drawn connection circles are gone, because the renderer draws
  one per port).
- Numeral clearance: `the_service_unit_numerals_stay_clear_of_the_frame_and_the_drawing`
  in `punchi-fluid-draw`, green.
- Sheet format: version 15, unchanged by this phase.

## Checklist

| Question | Answer |
|---|---|
| Is every port the standard draws drawn? | yes: `1` and `2` |
| Do the numerals match the declared ports? | yes: `1` supply, `2` working |
| Is the filter's element dashed, and does the drain leave the bottom vertex? | yes |
| Is the regulator a square with the flow through it and two facing arrows? | yes |
| Is the setting's arrow through the spring, with a filled head? | yes (`RF048`) |
| Is the gauge a circle on a stem, with its needle as an arrow from the centre? | yes |
| Is the lubricator drawn, with its own mark? | yes: the earlier record said it was missing |
| Is every numeral clear of the frame, the body and the lines? | yes, the draw test proves it |
| Is the symbol legible at palette size and at 300 dpi? | yes, both captures |

## What is still open

- **The frame is dashed, not chain.** `X12160` draws the assembly boundary as a chain line;
  the renderer has working, pilot, drain and electrical classes only, and the contract says
  "dashed frame". Adding a chain class touches the draw pattern, the catalog schema, the
  sheet's pipe classes and the export labels, which is a change of its own; it is recorded
  here instead of smuggled into an air-treatment phase.
- **The ports sit on the frame edge**, not outside it as the standard draws them; the 20 mm
  box has no margin for a numeral outside the frame.
- **The parts are compressed**: the standard's FRL is wider than tall, the project's box is
  square, and the lubricator is the part that gives up the most size.
- The second reader's signature.
- The 2012 text is read through the Japanese adoption, not the ISO text itself.
- The unit is modelled as a **fixed restriction**: it does not regulate, filter or lubricate.
- The three functions are drawn as one block; ISO 1219-1 also allows them separately, and a
  circuit that needs only the regulator should eventually be able to say so.

## Images

- `preparation.service_unit-light.png`, `preparation.service_unit-dark.png`
- `preparation.service_unit-300.png`
