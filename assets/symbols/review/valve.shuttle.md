# valve.shuttle — conformance review

    status: pending-300dpi
    date: 2026-11-22
    reviewer: automated pass, pending a person's sign-off
    function: shuttle valve (OR), the outlet follows the higher of the two inlets

## Provenance

Geometry is hand-drawn in `assets/catalog/valve.shuttle.toml` from the owner's reference and
the measures below. No asset, trace, screenshot or scan from another program or from the
standard is stored in the repository; the reference was read only to fix the form and the
position of each part.

## Automated checks — `punchi-fluid-catalog` symbol lint and validation

| Rule | Result |
|---|---|
| Two supply inlets and one working outlet | pass (`1a`, `1b` supply; `2` working) |
| No return and no actuator | pass (none declared) |
| Primitives inside the box, none degenerate | pass |
| Every line declares its class | pass (`working`) |
| The two inlets print the numeral 1 | pass (`numeral = "1"`; ids stay distinct) |

## Measures

| Feature | Requirement | Drawing |
|---|---|---|
| Body | a square body on the through line | x 0.20, y 0.25, 0.60 × 0.50 |
| Ports | two inlets `1` on opposite sides, outlet `2` above | 1a left, 1b right at mid-height; 2 on top centre |
| Left check | a ball on its V seat, the seat opening toward the inlet | ball at 0.35, 0.5, r 0.05; seat 0.375,0.40 → 0.283,0.50 → 0.375,0.60 |
| Right check | a V seat opening toward the outlet, connected to the body | seat 0.63,0.40 → 0.72,0.50 → 0.63,0.60; passage 0.72 → 0.80 |
| Junction | one dot where the checks meet and the outlet arrives | filled circle r 0.022 at 0.5, 0.5 |
| Numerals | one set, outside the body | 1 on both inlets, 2 on the outlet |

## Visual check

| Requirement | Result | Evidence |
|---|---|---|
| The ball, the two V seats and the junction read as a shuttle | pass | the fixture `valve.shuttle.txt` |
| The right seat reaches the body edge and the right port | pass | the fixture, the passage 0.72 → 0.80 |
| The outlet arrives at the junction | pass | the fixture |
| The two inlets both read `1` | pass | the fixture and the numeral field |
| Legible at palette size (24–30 px) | pass | `valve.shuttle-light.png`, `valve.shuttle-dark.png` |
| Legible at 300 dpi | pass, second reader owed | `valve.shuttle-300.png` |

## Sign-off

- **Status**: `pending-300dpi`
- **Reviewed by**: pending
- **Date**: 2026-11-22
