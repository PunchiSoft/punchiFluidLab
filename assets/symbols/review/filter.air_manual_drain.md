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
| The element is the **dashed line on the upper diagonal** | pass | dashed chord from the top vertex to the centre |
| The **bowl and the drain** below the lower vertex | pass | the bowl, its hatching and the stem, as the reference draws a manual purge |
| Connection points are the small open circles of the reference set | pass | drawn by the renderer at both ports, once |
| Legible at palette size | pass | `catalog-dark.png` |

| Legible at 300 dpi | pass | `filter.air_manual_drain-300.png`: automated pass; the second reader's signature is still owed |
## What is still open

- The 300 dpi sheet is in this folder; only a second reader's signature is owed.
- The drain is drawn and not modelled: emptying it is not part of the simulation.

## Images

- `filter.air_manual_drain-light.png`, `filter.air_manual_drain-dark.png`
- `filter.air_manual_drain-300.png`
