# filter.air — conformance review

    status: pending-300dpi
    date: 2026-11-07
    reviewer: automated pass, unreviewed by a second person
    function: air filter

## Provenance

Geometry is hand-written in `assets/catalog/filter.air.toml`. No third-party asset, no trace, no
scan: the shape is drawn from the requirement the sheet states, not copied from a picture.

## Automated checks — `punchi-fluid-catalog` symbol lint

| Rule | Result |
|---|---|
| Geometry inside the box, no degenerate primitive | pass |
| Every line declares its class | pass |
| Every port used and numbered | pass (ports 1 and 2) |

## Visual check

| Requirement | Result | Evidence |
|---|---|---|
| The filter is a **square turned 45°**: four equal sides, diagonals meeting at the centre | pass | half-diagonal 0.22 on both axes, so no deformation |
| The flow crosses it from vertex to vertex and is not drawn through the inside | pass | the line stops at each vertex |
| Connection points are the small open circles of the reference set | pass | circles at both ends |
| Legible at palette size | pass | `catalog-dark.png` |

## What is still open

- 300 dpi render and a second reader's signature.
- The filter is modelled as a fixed restriction with a high conductance: it does not clog, and
  nothing separates the particles it traps.

## Images

- `filter.air-light.png`, `filter.air-dark.png`
