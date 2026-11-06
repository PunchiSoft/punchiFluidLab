# throttle.fixed — conformance review

    status: pending-300dpi
    date: 2026-11-06
    reviewer: automated pass, unreviewed by a second person
    function: fixed restrictor between two ports

## Provenance

Hand-written geometry in `assets/catalog/throttle.fixed.toml`. No third-party asset.

## Automated checks — `punchi-fluid-catalog` symbol lint

| Rule | Result |
|---|---|
| Geometry inside the box, no degenerate primitive | pass |
| Every line declares its class | pass (both stubs are `working` lines) |
| Every port used and numbered | pass (1 and 2) |

## Visual check

| Requirement | Result | Evidence |
|---|---|---|
| The restriction is visible as a narrow throat between two stubs | pass | screenshots |
| Ports numbered outside the symbol | pass | 1 left, 2 right |
| Fixed, not adjustable | pass | no arrow across the throat |
| Legible at palette size | pass | `catalog-dark.png` |
| Legible at 300 dpi | **not checked** | needs M2 |

## What is still open

- 300 dpi render (M2) and a second reader's signature.
- **Shape not confirmed**: Part 1 draws the fixed restriction with a narrower throat
  than this approximation; the two wedges are our own construction and were not
  compared against a copy of the standard from this machine. This is the weakest of the
  four symbols and the first candidate for a redraw once the standard is at hand.

## Images

- `throttle.fixed-dark.png`, `throttle.fixed-light.png`
- `catalog-dark.png`, `catalog-light.png`
