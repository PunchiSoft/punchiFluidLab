# filter.air_manual_drain — conformance review

    status: pending-300dpi
    date: 2026-11-07
    reviewer: automated pass, unreviewed by a second person
    function: air filter with a manual drain

## Provenance

Geometry is hand-written in `assets/catalog/filter.air_manual_drain.toml`. No third-party asset, no trace, no
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
| The filter is a square turned 45°, the same shape as the plain filter | pass | half-diagonal 0.20 on both axes |
| The element is the **dashed line on the vertical diagonal** | pass | dashed chord from top vertex to bottom vertex |
| The **drain** leaves the bottom vertex on a stem | pass | without the stem the symbol is a filter, not a separator with a drain |
| Connection points are the small open circles of the reference set | pass | circles at both ends |
| Legible at palette size | pass | `catalog-dark.png` |

## What is still open

- 300 dpi render and a second reader's signature.
- The drain is drawn and not modelled: emptying it is not part of the simulation.

## Images

- `filter.air_manual_drain-light.png`, `filter.air_manual_drain-dark.png`
