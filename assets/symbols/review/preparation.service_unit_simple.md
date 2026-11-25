# preparation.service_unit_simple — conformance review

    status: complete
    date: 2026-11-25
    reviewer: Cristian Muñoz (owner)
    function: service unit, simplified representation, with the owner's automatic-drain purge arrow

## The claim

- **Function, in one sentence**: the same filter, regulator and lubricator as the full
  service unit, drawn as one wide body for a circuit that uses the unit and does not explain
  it.
- **Family**: conditioning (one block with two ports).
- **Normative source**: ISO 1219-1:2006, 7.4.4.17, figure `X12160`, the **simplified
  representation below** the detailed one, read in the owner's copy in the 2026-11-24
  symbology pass; confirmed on the 2012 basis in JIS B 0125-1:2020, 7.4.4.17, which prints
  both representations. Rules: `RF052` and `RF053`. The **automatic-drain mark** is the one
  the plan records as `X12200`/`F065V1`; the owner's reference of 2026-11-24 shows the compact
  with the purge arrow at the bottom.
- **Ports**: `1` supply at the left, outside the body · `2` working at the right, outside the
  body.

## Reference and assumptions

- **Reference read**: the standard's `X12160` simplified figure and the owner's drawn reference
  of 2026-11-24 (disposition and proportion only; ADR 0040). No capture of another
  application was traced.
- **Stated by the standard**: one rectangle with the **dashed element line at the left**, the
  **gauge circle with its needle** in the middle, and a **short line on the top edge at the
  right**. The generic simplified figure prints **no drain**.
- **Owner's decision, 2026-11-24**: this compact carries the **automatic-drain purge arrow**
  in the bottom band, because the unit the owner uses has an automatic drain in the filter.
  The generic figure of `X12160` does not print it; the mark is drawn from the owner's
  reference and the registration `X12200`/`F065V1`, and this record says so instead of
  claiming the simplified figure prints it.
- **Owner's decision, 2026-11-24**: the **short lubricator mark** on the top edge stays, as
  the simplified figure of `X12160` carries it.
- **Owner's decision, 2026-11-25**: the short mark is the **lubricator**; the same body
  **without** it is the separate `preparation.filter_regulator_simple`. The two are
  different components.
- **Owner's decision, 2026-11-24**: the box is **40 × 20 mm (2 : 1)**, derived from the
  owner's reference (its body is ≈2.05 : 1), not a measure of the standard.
- **Owner's decision, 2026-11-24**: the ports sit **outside the body**, and their numerals
  outside it, beside each port.
- **Assumption**: the body is 31.2 × 15.2 mm inside the 40 × 20 mm box, leaving the 4.4 mm
  margins the numerals need outside it; the frame declared in the file is the body, because
  the body is a solid rectangle and not a chain line.

## Measures

The symbol box is 40 × 20 mm; the body is 31.2 × 15.2 mm (x 4.4–35.6, y 2.4–17.6).

| Feature | Requirement | Drawing |
|---|---|---|
| Box | owner's 40 × 20 mm, declared in the file | `box = [40, 20]` |
| Body | one rectangle, wider than tall | x 4.4–35.6, y 2.4–17.6 |
| Element | dashed line at the left, full height | x 8.8, y 2.4–17.6 |
| Gauge | large circle with the needle as a full diagonal from the lower-right edge to the upper-left, filled head at the upper-left | centre (20.8; 10), r 4.5; needle (24.0; 13.2)–(17.6; 6.8), head at the upper-left |
| Lubricator mark | short line from the top edge at the right | x 32, y 2.4–4.0 |
| Purge arrow | automatic drain in the bottom band, pointing out | x 8.8, y 17.6–19.2; head 1.2 mm across |
| Ports | outside the body, stubs crossing its edge | `1` at (1.6; 10), `2` at (38.4; 10) |
| Numerals | from the port table, outside the body, beside the port | `1` at (1.5; 7.3), `2` at (38.5; 12.7); inside the 40 × 20 mm box |

## Evidence

- 300 dpi sheet, print ink: `preparation.service_unit_simple-300.png`.
- Light and dark captures: `preparation.service_unit_simple-light.png`,
  `preparation.service_unit_simple-dark.png`.
- Symbol lint: passes for the built-in.
- Geometry fixture regenerated and read:
  `assets/symbols/fixtures/preparation.service_unit_simple.txt` at the declared 40 × 20 mm
  box (the wide body, the ports and numerals outside, the purge arrow and the retained
  lubricator tick are the new lines).
- Numeral clearance: `the_simplified_service_unit_numerals_sit_outside_its_body` in
  `punchi-fluid-draw`, green in print and dark ink.
- Box: `a_declared_box_is_read_from_the_file` and the built-in box test, green.
- Sheet format: version 15, unchanged by this phase.

## Checklist

| Question | Answer |
|---|---|
| Is every port the standard draws drawn? | yes: `1` and `2`, outside the body |
| Do the numerals match the declared ports? | yes: `1` supply, `2` working |
| Is the element a dashed line at the left? | yes, full height |
| Is the gauge a large circle with the full diagonal needle and the filled head at the upper-left? | yes |
| Is the right-hand lubricator mark drawn? | yes: the short line on the top edge |
| Is the automatic-drain purge arrow drawn? | yes, in the bottom band; the record states the generic `X12160` figure does not print it |
| Are the ports and their numerals outside the body? | yes, beside each port and inside the 40 × 20 mm box |
| Is the symbol legible at palette size and at 300 dpi? | the 300 dpi and both captures are regenerated; the owner's signature is pending |

## What is still open

- **The 2012 text is read through the Japanese adoption**, not the ISO text itself.
- **The box is the owner's derived measure**, not a measure of the standard: 40 × 20 mm from
  the reference's ≈2.05 : 1 body.
- **The purge arrow is the owner's reference's mark**, not a mark the generic simplified
  `X12160` prints; the registration `X12200`/`F065V1` is recorded from the plan, and the
  figure itself was not re-read in this pass.
- The tick's meaning is not stated by the figure; the review records what is drawn and the
  owner's decision to keep it.
- The model is the same fixed restriction as the full unit: it does not regulate, filter or
  lubricate.
- **The signature is pending**: the owner signs after reviewing the three captures and this
  record, and the status returns to `complete` only then.

## Sign-off

- **Status**: `complete`
- **Reviewed by**: Cristian Muñoz (owner)
- **Date**: 2026-11-25
- **What changed since the last record**: the box (40 × 20), the wide body, the ports and
  numerals outside, the automatic-drain purge arrow and the redrawn gauge. The previous
  `complete` record described a 20 mm box with no drain; this record replaces it.

## Images

- `preparation.service_unit_simple-light.png`,
  `preparation.service_unit_simple-dark.png`
- `preparation.service_unit_simple-300.png`
