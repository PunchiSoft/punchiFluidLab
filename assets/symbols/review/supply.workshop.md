# supply.workshop — conformance review

    status: pending-300dpi
    date: 2026-11-06
    reviewer: automated pass, unreviewed by a second person
    function: compressed air source, constant pressure

## Provenance

Hand-written geometry in `assets/catalog/supply.workshop.toml`. No third-party asset.

## Automated checks — `punchi-fluid-catalog` symbol lint

| Rule | Result |
|---|---|
| Geometry inside the box, no degenerate primitive | pass |
| Every line declares its class | pass (the outlet is a `working` line) |
| Every port used and numbered | pass (port 1, drawn below the outlet) |

## Visual check

| Requirement | Result | Evidence |
|---|---|---|
| Source shown as a circle with an outline triangle and an outlet — a filled triangle means hydraulic | pass | screenshots |
| Numerals outside the symbol, at the stub | pass | "1" below the outlet |
| Symbol keeps its proportions in a square box | pass | circle and triangle concentric |
| Legible at palette size | pass | `catalog-dark.png` |
| Legible at 300 dpi | **not checked** | needs M2 |

## What is still open

- 300 dpi render (M2) and a second reader's signature.
- One known deviation: the outlet line is drawn straight down for clarity of the
  single-port layout, where Part 1 shows the source with its working connection
  leaving the circle. This is recorded rather than quietly accepted.

## Images

- `supply.workshop-dark.png`, `supply.workshop-light.png`
- `catalog-dark.png`, `catalog-light.png`
