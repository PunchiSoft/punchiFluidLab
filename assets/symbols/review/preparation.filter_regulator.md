# preparation.filter_regulator — conformance review

    status: pending-300dpi
    date: 2026-11-24
    reviewer: pending — the owner's signature after the PR review is the closing step
    function: filter and pressure-reducing regulator, without a lubricator

## The claim

- **Function, in one sentence**: the air is filtered and its pressure is held at the set
  value, ready for the circuit, with no oil added.
- **Family**: conditioning (one block with two working ports).
- **Normative source**: ISO 1219-1:2006, figures **`X12150`** (the filter with manual drain
  and the pressure-relieving regulator, **without a gauge**) and **`X12160`** (the same
  regulator with the gauge), read in the owner's copy in the 2026-11-24 symbology pass;
  confirmed on the 2012 basis in JIS B 0125-1:2020, 7.4.4.17 and the `X12150` figure. This
  file **composes the two**: the regulator and its vent/feedback are `X12150`'s, and the
  gauge on its stem is `X12160`'s. Rules: `RF052`, `RF053`, `RF048`.
- **Ports**: `1` working at the left, outside the frame · `2` working at the right, outside
  the frame.

## Reference and assumptions

- **Reference read**: the standard's `X12150` and the gauge of `X12160`; the owner's drawn
  reference of 2026-11-24 for the disposition and proportion (ADR 0040). No capture of
  another application was traced.
- **Composition, stated**: `X12150` prints the regulator without the gauge, and `X12160`
  prints the gauge after the regulator. The file draws the filter with manual drain, the
  regulator with vent and feedback and the gauge, and the record says the figure is composed
  from the two rather than claiming one figure prints all of it.
- **Owner's decision, 2026-11-24**: the box is **46 × 25 mm**, the same wide frame as the
  full service unit, so the family is read at one proportion.
- **Owner's decision, 2026-11-24**: the ports sit **outside the frame** and the numerals
  outside it, beside each port: `1` above the left stub and `2` below the right one.
- **Owner's decision, 2026-11-24**: the frame is the **chain line** (`boundary` class), not a
  dashed drain line.
- **Assumption**: the frame is 36.8 × 20 mm inside the 46 × 25 mm box, leaving the 4.6 mm
  margins the numerals need outside it. The envelope is the whole box because the ports are
  outside the frame. The gauge is placed at x 34 mm to use the right side the lubricator
  occupies in the full unit; the standard fixes the order, not this exact x.

## Measures

The symbol box is 46 × 25 mm; the frame is 36.8 × 20 mm (x 4.6–41.4, y 2.5–22.5).

| Feature | Requirement | Drawing |
|---|---|---|
| Box | owner's 46 × 25 mm, declared in the file | `box = [46, 25]` |
| Frame | chain line around the functions, one group | x 4.6–41.4, y 2.5–22.5; `line_class = "boundary"` |
| Filter | diamond, element dashed down to the chord, bowl and drain stem | centre (9.5; 12.5), half-diagonal 3.2 mm; element to y 14.1; chord (7.9–11.1; 14.1); drain to y 23.5 |
| Regulator | upright square, flow through it, two facing arrows | square (16.3; 9.3)–(22.7; 15.7); arrow tips at 18.2 and 21.0 on y 12.5 |
| Vent | open triangle on the regulator's left | base x 16.3, y 9.9–11.1; apex (14.9; 10.5) |
| Spring | above the square, four peaks | x 18.6–21.4, y 4.8–9.3 |
| Adjustability | arrow through the spring, filled head (`RF048`) | (17; 8.8)–(23; 4.8) |
| Sensing line | dashed, from the outlet to the underside | stub (20; 15.7)–(20; 18); loop to (34; 18) and up to y 12.5 |
| Gauge | circle on a stem, needle as an arrow from the centre | centre (34; 8), r 2; stem from y 12.5; needle to (32.6; 6.9) |
| Tap | filled dot where the stem meets the line | (34; 12.5), r 0.25 |
| Lubricator | **absent**: this is the filter-regulator, not the FRL | no diamond and no oil mark |
| Ports | outside the frame, stubs crossing it | `1` at (1.84; 12.5), `2` at (44.16; 12.5) |
| Numerals | from the port table, outside the frame, beside the port | `1` at (1.5; 9.5), `2` at (44.5; 15.5) |

## Evidence

- 300 dpi sheet, print ink: `preparation.filter_regulator-300.png`.
- Light and dark captures: `preparation.filter_regulator-light.png`,
  `preparation.filter_regulator-dark.png`.
- Symbol lint: passes for the built-in.
- Geometry fixture regenerated and read:
  `assets/symbols/fixtures/preparation.filter_regulator.txt`.
- Numeral clearance: `the_filter_regulator_numerals_sit_outside_its_frame` in
  `punchi-fluid-draw`, green in print and dark ink.
- Family test: `the_filter_regulator_is_the_filter_and_regulator_without_a_lubricator` in
  `punchi-fluid-catalog`, green.
- Sheet format: version 15, unchanged by this phase.

## Checklist

| Question | Answer |
|---|---|
| Is every port the standard draws drawn? | yes: `1` and `2`, outside the frame |
| Do the numerals match the declared ports? | yes: `1` and `2` working |
| Is the filter's element dashed, and does the drain leave the bottom vertex? | yes, with the bowl chord |
| Is the regulator a square with the flow through it and two facing arrows? | yes |
| Is the setting's arrow through the spring, with a filled head? | yes (`RF048`) |
| Is the vent and the feedback line drawn? | yes: open triangle and dashed line |
| Is the gauge drawn on its stem, with the needle from the centre? | yes |
| Is the lubricator absent? | yes: this form is the filter-regulator |
| Is the frame the standard's chain line? | yes: `line_class = "boundary"` |
| Are the ports and their numerals outside the frame? | yes, in the 4.6 mm margin, beside each port |
| Is the symbol legible at palette size and at 300 dpi? | the 300 dpi and both captures are new; the owner's signature is pending |

## What is still open

- **The composition is stated**: `X12150` prints no gauge and `X12160` prints it; this file
  composes them, and the record says so instead of claiming one figure prints all of it.
- **The 2012 text is read through the Japanese adoption**, not the ISO text itself.
- **The box is the owner's 46 × 25 mm**, the family's proportion, not a measure of `X12150`.
- **The gauge's x (34 mm)** is the renderer's placement to use the right side the lubricator
  occupies in the full unit; the standard fixes the order, not this exact coordinate.
- `RF053`'s exact position (above or to the left of the port) remains an open question; the
  numerals are outside the frame and beside their ports, the owner's 2026-11-24 decision.
- The unit is modelled as a **fixed restriction**: it does not regulate or filter.
- **The signature is pending**: the owner signs after reviewing the three captures and this
  record.

## Sign-off

- **Status**: `pending-300dpi`
- **Reviewed by**: pending — the owner signs after the PR review
- **Date**: 2026-11-24
- **What changed since the last record**: new file; no previous record.

## Images

- `preparation.filter_regulator-light.png`, `preparation.filter_regulator-dark.png`
- `preparation.filter_regulator-300.png`
