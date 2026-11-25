# preparation.service_unit — conformance review

    status: complete
    date: 2026-11-25
    reviewer: Cristian Muñoz (owner)
    function: service unit — filter, regulator and lubricator — between the compressor and the circuit

## The claim

- **Function, in one sentence**: the air is filtered, its pressure is held at the set value
  and oil is added, so what leaves the unit is the line air the circuit is designed around.
- **Family**: conditioning (one block with two ports).
- **Normative source**: ISO 1219-1:2006, 7.4.4.17, figure `X12160`, read in the owner's copy
  in the 2026-11-24 symbology pass; confirmed on the 2012 basis in JIS B 0125-1:2020,
  7.4.4.17, same registration number. The standard describes the figure as a filter with
  manual drain, a pressure-relieving regulator with manual adjustment, a pressure gauge and a
  **lubricator**. Rules: `RF052`, `RF053`, `RF048` (the adjustable element's arrow goes through
  the element).
- **Ports**: `1` supply at the left, outside the frame · `2` working at the right, outside the
  frame.

## Reference and assumptions

- **Reference read**: the standard's detailed `X12160` figure and the owner's drawn reference
  of 2026-11-24 (disposition and proportion only; ADR 0040). No capture of another
  application was traced.
- **Stated by the standard**: the functions are drawn in the order the air meets them inside
  a frame; the filter is the diamond with the element, the bowl chord and the drain stem; the
  regulator is the square with the flow through it, the two facing arrows, the spring with the
  diagonal adjusting arrow, the vent triangle on its left and the dashed sensing line from
  the outlet back to its underside; the gauge is the circle on a stem with an arrow from its
  centre; the lubricator is the diamond with the plain line at its top vertex.
- **Owner's decision, 2026-11-24**: the **box is 46 × 25 mm**, derived from the owner's
  reference with a cylinder as the scale, not a measure of the standard. The standard draws
  the unit wider than tall; the previous 20 mm square compressed it, and the owner rejected
  that drawing.
- **Owner's decision, 2026-11-24**: the ports sit **outside the frame** (x 1.84 and 44.16 mm),
  their stubs cross it, and the **numerals are outside it, beside each port**: `1` above the
  left stub and `2` below the right one.
- **Closed debt**: the frame is now the standard's **chain line** (the `boundary` line class:
  a long dash, a gap, a dot and a gap), not the dashed drain class the previous record
  carried.
- **Assumption**: the frame is 36.8 × 20 mm inside the 46 × 25 mm box, leaving the 4.6 mm
  margins the numerals need outside it. The envelope is the whole box because the ports are
  outside the frame.

## Measures

The symbol box is 46 × 25 mm; the frame is 36.8 × 20 mm (x 4.6–41.4, y 2.5–22.5).

| Feature | Requirement | Drawing |
|---|---|---|
| Box | owner's 46 × 25 mm, declared in the file | `box = [46, 25]` |
| Frame | chain line around the functions, one group | x 4.6–41.4, y 2.5–22.5; `line_class = "boundary"` |
| Filter | diamond, element dashed with short dashes down to the chord, bowl, short drain stem inside the frame | centre (9.5; 12.5), half-diagonal 3.2 mm; element to y 14.1 (`pilot` dash); chord (7.9–11.1; 14.1); drain to y 18.75, inside the frame |
| Regulator | upright square, flow through it, two facing arrows | square (16.3; 9.3)–(22.7; 15.7); arrow tips at 18.2 and 21.0 on y 12.5 |
| Vent | open triangle on the regulator's left | base x 16.3, y 9.9–11.1; apex (14.9; 10.5) |
| Spring | above the square, four peaks | x 18.6–21.4, y 4.8–9.3 |
| Adjustability | arrow through the spring, filled head (`RF048`) | (17; 8.8)–(23; 4.8) |
| Sensing line | dashed: diagonal down-right from the working line at the body's right edge, right leg down, bottom left, left leg up to the underside | diagonal (22.7; 12.5)–(25.9; 16.2), right leg to (25.9; 19.25), bottom to (19.3; 19.25), left leg up to (19.3; 15.7); all `pilot` |
| Gauge | circle on a stem, needle crossing from the lower-left edge to the upper-right, filled head | centre (30; 8), r 2; stem from y 12.5; needle (28.6; 9.4)–(31.4; 6.6), head at the upper-right |
| Tap | filled dot where the stem meets the line | (30; 12.5), r 0.25 |
| Lubricator | diamond with the plain line at the top vertex | centre (37; 12.5), half-diagonal 2.8 mm; line to y 11 |
| Ports | outside the frame, stubs crossing it | `1` at (1.84; 12.5), `2` at (44.16; 12.5) |
| Numerals | from the port table, outside the frame, beside the port | `1` at (1.5; 9.5), `2` at (44.5; 15.5); inside the 46 × 25 mm box, clear of the frame, the stubs and the parts |

## Evidence

- 300 dpi sheet, print ink: `preparation.service_unit-300.png`.
- Light and dark captures: `preparation.service_unit-light.png`,
  `preparation.service_unit-dark.png`.
- Symbol lint: passes for the built-in.
- Geometry fixture regenerated and read: `assets/symbols/fixtures/preparation.service_unit.txt`
  at the declared 46 × 25 mm box (the frame chain, the ports outside, the numerals outside and
  the redrawn filter, regulator, gauge and lubricator are the new lines).
- Numeral clearance: `the_service_unit_numerals_stay_clear_of_the_frame_and_the_drawing` in
  `punchi-fluid-draw`, green in print and dark ink; it asserts the numeral is outside the
  frame, beside its port, inside the box and clear of every primitive.
- Chain line: `a_chain_line_draws_a_dash_a_gap_a_dot_and_a_gap` (draw) and
  `a_chain_line_writes_its_four_dash_values` (SVG), green.
- Box: `a_declared_box_is_read_from_the_file` and the built-in box test, green.
- Sheet format: version 15, unchanged by this phase.

## Checklist

| Question | Answer |
|---|---|
| Is every port the standard draws drawn? | yes: `1` and `2`, outside the frame |
| Do the numerals match the declared ports? | yes: `1` supply, `2` working |
| Is the filter's element dashed, and does the drain leave the bottom vertex inside the frame? | yes, with the bowl chord; the stem ends at y 18.75, inside the frame (owner's 2026-11-25 correction) |
| Is the regulator a square with the flow through it and two facing arrows? | yes |
| Is the setting's arrow through the spring, with a filled head? | yes (`RF048`) |
| Is the gauge a circle on a stem, with its needle as an arrow from the centre? | yes |
| Is the lubricator drawn, with its own mark? | yes |
| Is the frame the standard's chain line? | yes: `line_class = "boundary"`, four-phase pattern |
| Are the ports and their numerals outside the frame? | yes, in the 4.6 mm margin, beside each port |
| Is every numeral clear of the frame, the parts and the box edge? | yes, the draw test proves it in both inks |
| Is the symbol legible at palette size and at 300 dpi? | the 300 dpi and both captures are regenerated; the owner's signature is pending |

## What is still open

- **The 2012 text is read through the Japanese adoption**, not the ISO text itself.
- **The box is the owner's derived measure**, not a measure of the standard: 46 × 25 mm from
  the reference with a cylinder as the scale.
- **`RF053`'s position** — above or to the left of the port — remains an open question of the
  contract; the numerals are outside the frame and beside their ports, which is the owner's
  2026-11-24 decision, and the review records it instead of claiming the standard fixes the
  exact spot.
- The unit is modelled as a **fixed restriction**: it does not regulate, filter or lubricate.
- **The manifest signature is pending**: the review is complete; the owner signs the batch
  manifest, not this record.

## Sign-off

- **Status**: `complete`
- **Reviewed by**: Cristian Muñoz (owner)
- **Date**: 2026-11-25
- **What changed since the last record**: the box (46 × 25), the chain-line frame, the ports
  and numerals outside, the internal forms redrawn at the reference's proportion, the filter
  element with short dashes, the drain shortened so it stays inside the frame, the sensing
  line leaving the working line at the body's right edge with the diagonal down-right, the
  right leg down outside the body, the bottom, and the left leg up to the underside, and the
  gauge needle crossing from the lower-left to the upper-right (owner's 2026-11-25
  correction).
  The previous `complete` record described the 20 mm
  square and the dashed frame; this record replaces it.

## Images

- `preparation.service_unit-light.png`, `preparation.service_unit-dark.png`
- `preparation.service_unit-300.png`
