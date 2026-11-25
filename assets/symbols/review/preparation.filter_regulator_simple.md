<!-- SPDX-FileCopyrightText: 2026 PunchiSoft -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# preparation.filter_regulator_simple — conformance review

    status: complete
    date: 2026-11-25
    reviewer: Cristian Muñoz (owner)
    function: filter-regulator, simplified: filter and regulator without a lubricator

## The claim

- **Function, in one sentence**: the compact filter and pressure regulator, drawn as one
  wide body for a circuit that uses the unit and does not explain it.
- **Family**: conditioning (one block with two ports).
- **Normative source**: ISO 1219-1:2006, 7.4.4.17, figure `X12160`, the **simplified
  representation** of the filter and regulator, read in the owner's copy in the 2026-11-24
  symbology pass; confirmed on the 2012 basis in JIS B 0125-1:2020, 7.4.4.17. Rules:
  `RF052`, `RF053`. The **automatic-drain mark** is the owner's `X12200`/`F065V1` mark, as in
  the full service unit; the owner's reference of 2026-11-25 shows the compact with it.
- **Ports**: `1` supply at the left, outside the body · `2` working at the right, outside the
  body.

## Reference and assumptions

- **Reference read**: the standard's simplified `X12160` figure and the owner's reference of
  2026-11-25 (disposition and proportion only; ADR 0040). No capture of another application
  was traced.
- **Owner's decision, 2026-11-25**: the compact body **without** the short mark on the top
  edge is the **filter-regulator**; the same body **with** the mark is the separate
  `preparation.service_unit_simple` (the mark is the lubricator). They are two components,
  not one; this record is the filter-regulator.
- **Stated by the standard**: one rectangle with the **dashed element line at the left**, the
  **gauge circle with its needle** in the middle. The generic simplified figure prints no
  lubricator tick; the tick belongs to the service unit.
- **Owner's decision, 2026-11-25**: the gauge is the reference's **large circle** with the
  needle as a **full diagonal** from the lower-right edge to the upper-left, the filled head
  at the upper-left.
- **Owner's decision, 2026-11-24**: this compact carries the **automatic-drain purge arrow**
  in the bottom band, because the unit the owner uses has an automatic drain in the filter.
- **Owner's decision, 2026-11-24**: the box is **40 × 20 mm (2 : 1)**, derived from the
  owner's reference, not a measure of the standard.
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
| Lubricator mark | none: this component is the filter-regulator | absent |
| Purge arrow | automatic drain in the bottom band, pointing out | x 8.8, y 17.6–19.2; head 1.2 mm across |
| Ports | outside the body, stubs crossing its edge | `1` at (1.6; 10), `2` at (38.4; 10) |
| Numerals | from the port table, outside the body, beside the port | `1` at (1.5; 7.3), `2` at (38.5; 12.7); inside the 40 × 20 mm box |

## Evidence

- 300 dpi sheet: `preparation.filter_regulator_simple-300.png`.
- Light and dark captures, fitted: `preparation.filter_regulator_simple-light.png`,
  `preparation.filter_regulator_simple-dark.png`.
- Symbol lint: passes for the built-in.
- Geometry fixture regenerated and read:
  `assets/symbols/fixtures/preparation.filter_regulator_simple.txt` at the declared 40 × 20 mm
  box.
- Numeral clearance: the same rule as `preparation.service_unit_simple`; the numerals are
  outside the body and inside the box.
- Box: the declared box is read from the file; the built-in box test covers it.
- Sheet format: version 15, unchanged.

## Checklist

| Question | Answer |
|---|---|
| Is every port the standard draws drawn? | yes: `1` and `2`, outside the body |
| Do the numerals match the declared ports? | yes: `1` supply, `2` working |
| Is the element a dashed line at the left? | yes, full height |
| Is the gauge a large circle with the full diagonal needle and the filled head at the upper-left? | yes |
| Is a lubricator mark drawn? | no: this is the filter-regulator; the mark belongs to `preparation.service_unit_simple` |
| Is the automatic-drain purge arrow drawn? | yes, in the bottom band |
| Are the ports and their numerals outside the body? | yes, beside each port and inside the 40 × 20 mm box |
| Is the symbol legible at palette size and at 300 dpi? | the 300 dpi and both captures are regenerated; the owner's signature is pending |

## What is still open

- **The 2012 text is read through the Japanese adoption**, not the ISO text itself.
- **The box is the owner's derived measure**, not a measure of the standard: 40 × 20 mm from
  the reference's ≈2.05 : 1 body.
- **The purge arrow is the owner's reference's mark**, not a mark the generic simplified
  `X12160` prints; the registration `X12200`/`F065V1` is recorded from the plan.
- **The signature is pending**: the owner signs after reviewing the three captures and this
  record, and the status returns to `complete` only then.

## Sign-off

- **Status**: `complete`
- **Reviewed by**: Cristian Muñoz (owner)
- **Date**: 2026-11-25
- **What changed since the last record**: new component. It is the owner's 2026-11-25
  filter-regulator, split from the service unit: same compact body, no lubricator mark, large
  gauge with the full diagonal needle.

## Images

- `preparation.filter_regulator_simple-light.png`, `-dark.png`
- `preparation.filter_regulator_simple-300.png`
