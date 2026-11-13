# supply.compressor_adjustable — conformance review

    status: pending-300dpi
    date: 2026-11-07
    reviewer: automated pass, unreviewed by a second person
    function: compressed-air source with an adjustable delivery

## Provenance

Geometry is hand-written in `assets/catalog/supply.compressor_adjustable.toml`. No third-party asset, no trace, no
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
| The delivery leaves the top through a stem ending in a connection circle | pass | stem above the circle, open circle at the port |
| The triangle inside the circle points up, to its delivery | pass | apex up, the air leaves through the point |
| The connection circle marks the port | pass | drawn by the renderer at ports 1 and 2 |
| The diagonal arrow marks the adjustable delivery | pass | the arrow through the circle |
| The bottom intake mark is drawn, as the reference does | pass | the stem and circle, declared as a mark, not a port |
| Legible at palette size | pass | `catalog-dark.png` |

## What is still open

- 300 dpi render and a second reader's signature.
- The compressor is modelled as a constant-pressure source: the real machine's delivery curve
  is not modelled.

## Images

- `supply.compressor_adjustable-light.png`, `supply.compressor_adjustable-dark.png`
