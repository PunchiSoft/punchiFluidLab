# supply.network — conformance review

    status: pending-300dpi
    date: 2026-11-07
    reviewer: automated pass, unreviewed by a second person
    function: compressed-air source drawn as the connection to the network

## Provenance

Geometry is hand-written in `assets/catalog/supply.network.toml`. No third-party asset, no trace, no
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
| The network connection is an **outline triangle**, no circle around it | pass | a circle would be the machine that makes the air, which this symbol is not |
| The triangle is **equilateral**, so its proportions come from the triangle and not from a drawing that happens to look like one | pass | base 0.577 of the box, height 0.50: half the base is tan(30°) of the height |
| The delivery leaves the base on a stem | pass | stem from the middle of the base to the port |
| The apex of the triangle is at the connection, at the top | pass | the point meets the stem, and the hose arrives at its circle |
| The connection circle marks the port | pass | drawn by the renderer at port 1 |
| Legible at palette size | pass | `catalog-dark.png` |

## What is still open

- 300 dpi render and a second reader's signature.
- The source is a constant pressure: a real network sags under load, which is not modelled.

## Images

- `supply.network-light.png`, `supply.network-dark.png`
