# lubricator.oil — conformance review

    status: pending-300dpi
    date: 2026-11-07
    reviewer: automated pass, unreviewed by a second person
    function: oil lubricator

## Provenance

Geometry is hand-written in `assets/catalog/lubricator.oil.toml`. No third-party asset, no trace, no
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
| The lubricator is a square turned 45° with the flow crossing it from vertex to vertex | pass | half-diagonal 0.22 on both axes |
| It carries **no dashed element**, which is what tells it from the filter | pass | the filter and the lubricator are the same diamond otherwise |
| Connection points are the small open circles of the reference set | pass | circles at both ends |
| Legible at palette size | pass | `catalog-dark.png` |

## What is still open

- 300 dpi render and a second reader's signature.
- The oil mist is drawn and not modelled: the air that leaves it carries no oil in the solver.

## Images

- `lubricator.oil-light.png`, `lubricator.oil-dark.png`
