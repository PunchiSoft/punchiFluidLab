# supply.compressor — conformance review

    status: pending-300dpi
    date: 2026-11-07
    reviewer: automated pass, unreviewed by a second person
    function: compressed-air source drawn as the compressor that makes it

## Provenance

Geometry is hand-written in `assets/catalog/supply.compressor.toml`. No third-party asset, no trace, no
scan: the shape is drawn from the requirement the sheet states, not copied from a picture.

## Automated checks — `punchi-fluid-catalog` symbol lint

| Rule | Result |
|---|---|
| Geometry inside the box, no degenerate primitive | pass |
| Every line declares its class | pass |
| Every port used and numbered | pass (port 1, the delivery) |

## Visual check

| Requirement | Result | Evidence |
|---|---|---|
| The gas source is a circle with an **outline** triangle inside it | pass | a filled triangle is the hydraulic source, so the fill is a medium and not a style |
| The triangle stands on its base line inside the circle | pass | equilateral, base 0.364 of the box, height 0.33 |
| The delivery leaves the bottom through a stem ending in a connection circle | pass | stem below the circle, open circle at the port |
| The triangle inside the circle points at its delivery | pass | apex on the stem, the air leaves through the point |
| The connection circle marks the port | pass | drawn by the renderer at port 1, not by the file |
| Legible at palette size | pass | `catalog-dark.png` |

## What is still open

- 300 dpi render and a second reader's signature.
- The compressor is modelled as a constant-pressure source: the real machine's delivery curve
  is not modelled.

## Images

- `supply.compressor-light.png`, `supply.compressor-dark.png`
