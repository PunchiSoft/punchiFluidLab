# preparation.service_unit_simple — conformance review

    status: pending-300dpi
    date: 2026-11-07
    reviewer: automated pass, unreviewed by a second person
    function: service unit drawn in its compact form

## Provenance

Geometry is hand-written in `assets/catalog/preparation.service_unit_simple.toml`. No third-party asset, no trace, no
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
| One body with the **dashed element line** and the **dial** inside it | pass | a plain rectangle is not this symbol |
| The **drain leaves the bottom** as an arrow | pass | the only part of the unit that goes somewhere else |
| Connection points are the small open circles of the reference set | pass | circles at both ends |
| Legible at palette size | pass | `catalog-dark.png` |

## What is still open

- 300 dpi render and a second reader's signature.
- Same model as the full unit: a fixed restriction. It does not regulate, filter or lubricate.

## Images

- `preparation.service_unit_simple-light.png`, `preparation.service_unit_simple-dark.png`
