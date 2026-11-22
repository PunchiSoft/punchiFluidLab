# valve.two_pressure — conformance review

    status: pending-300dpi
    date: 2026-11-22
    reviewer: automated pass, pending a person's sign-off
    function: two-pressure valve (AND), the outlet follows the lower inlet while both carry pressure

## Provenance

Geometry is hand-drawn in `assets/catalog/valve.two_pressure.toml` from the requirement and
the measures below. No asset, trace, screenshot or scan from another program or from the
standard is used; the owner's reference was read only to confirm which symbol exists and
which side each port is on (ADR 0039).

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
| Body | a square body on the through line | x 0.25, y 0.3, 0.5 × 0.4 |
| Ports | two inlets `1` on opposite sides, outlet `2` above | 1a left, 1b right at mid-height; 2 on top centre |
| Piston | a double piston between the inlets, its rod on the through line | two vertical faces at 0.43 and 0.57, rod between them |
| Outlet | reaches the piston between its faces | line from 0.5, 0.3 to 0.5, 0.5 |
| Numerals | one set, outside the body | 1 on both inlets, 2 on the outlet |

## Visual check

| Requirement | Result | Evidence |
|---|---|---|
| The double piston reads as the AND element | pass | the fixture `valve.two_pressure.txt` |
| The outlet reaches the piston and not the body edge | pass | the fixture |
| The two inlets both read `1` | pass | the fixture and the numeral field |
| Legible at palette size (24–30 px) | pass | `valve.two_pressure-light.png`, `valve.two_pressure-dark.png` |
| Legible at 300 dpi | pass, second reader owed | `valve.two_pressure-300.png` |

## Sign-off

- **Status**: `pending-300dpi`
- **Reviewed by**: pending
- **Date**: 2026-11-22
