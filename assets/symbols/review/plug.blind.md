# plug.blind — conformance review

    status: pending-300dpi
    date: 2026-11-12
    reviewer: automated pass, unreviewed by a second person
    function: caps a port so the line ends there on purpose

## Provenance

Hand-written geometry in `assets/catalog/plug.blind.toml`: a short stub ending in the
blocking bar. No third-party asset and no trace of another program's symbol.

## Automated checks — `punchi-fluid-catalog` symbol lint

| Rule | Result |
|---|---|
| Primitives inside the box, no degenerate primitive | pass |
| Every line declares its class | pass: `working` |
| Exactly one port, because a plug caps one line | pass: the plug validation |
| No number of its own to range-check | pass: the behaviour declares none |
| Legible at palette size | pass: see the images |

## Visual check

| Requirement | Result | Evidence |
|---|---|---|
| The bar across the end reads as "closed", not as a joint | pass | the same mark a valve draws for a blocked way |
| One port, where the hose arrives | pass | the stub and its numeral |
| No stub on the far side, because nothing leaves | pass | the bar ends the line |
| Legible at palette size (24–30 px) | pass | `catalog-dark.png`, sidebar row |
| Legible at 300 dpi | **not checked** | needs M2 |

## Decisions this symbol carries

- **A behaviour of its own** ([ADR 0016](../../docs/adr/0016-a-termination-is-a-component.md)):
  a plug closes a line and says so to the check, which is why a capped working port stops
  being reported as open.
- **The mark is shared with the valve's blocked way**, so a reader learns one shape for
  "closed" and reads it everywhere.

## What is still open

- 300 dpi render (M2) and a second reader's signature.
- The normative figure was not traced; the requirement is our contract in
  [docs/symbology.md](../../docs/symbology.md).

## Images

- `plug.blind-light.png`, `plug.blind-dark.png`
