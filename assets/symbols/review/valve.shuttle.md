# valve.shuttle — conformance review

    status: pending-300dpi
    date: 2026-11-22
    reviewer: automated pass, pending a person's sign-off
    function: shuttle valve (OR), the outlet follows the higher of the two inlets

## Provenance

Geometry is hand-drawn in `assets/catalog/valve.shuttle.toml` from the requirement and the
measures below. No asset, trace, screenshot or scan from another program or from the
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
| Check seats | two opposing seats meeting at the centre | two filled triangles, apexes at 0.5, 0.5 |
| Junction | one dot where the seats meet | filled circle 0.025 |
| Numerals | one set, outside the body | 1 on both inlets, 2 on the outlet |

## Visual check

| Requirement | Result | Evidence |
|---|---|---|
| The two check seats and the junction read as a shuttle | pass | the fixture `valve.shuttle.txt` |
| The outlet leaves from the centre, not from a side | pass | the fixture |
| The two inlets both read `1` | pass | the fixture and the numeral field |
| Legible at palette size (24–30 px) | pass | `valve.shuttle-light.png`, `valve.shuttle-dark.png` |
| Legible at 300 dpi | pass, second reader owed | `valve.shuttle-300.png` |

## Sign-off

- **Status**: `pending-300dpi`
- **Reviewed by**: pending
- **Date**: 2026-11-22
