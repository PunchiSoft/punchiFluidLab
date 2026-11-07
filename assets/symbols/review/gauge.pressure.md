# gauge.pressure — conformance review

    status: pending-300dpi
    date: 2026-11-06
    reviewer: automated pass, unreviewed by a second person
    function: shows the pressure at the port it is connected to, on a 0–10 bar dial

## Provenance

Hand-written geometry in `assets/catalog/gauge.pressure.toml`: a circle for the dial, a
needle, a hub and the stem to port 1. No third-party asset and no trace of another
program's symbol.

## Automated checks — `punchi-fluid-catalog` symbol lint

| Rule | Result |
|---|---|
| Primitives inside the box, no degenerate primitive | pass |
| Every line declares its class | pass: `working` |
| Exactly one working port | pass: the gauge validation |
| Scale finite with `max_bar` above `min_bar` | pass |
| Legible at palette size | pass: see the images |

## Visual check

| Requirement | Result | Evidence |
|---|---|---|
| The dial is the symbol; the number is a reading, not a primitive | pass | the value is drawn from the model at the lower part of the dial |
| One port at the bottom, where a gauge connects | pass | the stem and the numeral |
| Legible at palette size (24–30 px) | pass | `catalog-dark.png`, sidebar row |
| Legible at 300 dpi | **not checked** | needs M2 |

## Decisions this symbol carries

- **A behaviour, not a decoration** ([ADR 0008](../../docs/adr/0008-a-gauge-is-a-behaviour.md)):
  the gauge connects, is checked like any component and reads the pressure at its node;
  a label the drawing put next to arbitrary components would do none of that.
- **The scale is a file value**: a 0–60 bar gauge is another file, not another branch.
- **The reading is text from the model.** A needle angle would need the drawing to know
  the scale and the units; the number is unambiguous and already localised by the panel.

## What is still open

- 300 dpi render (M2) and a second reader's signature.
- A moving needle is a later improvement; the number is the evidence today.
- Flow, force and strip-chart instruments are separate behaviours, each with its own ADR.

## Images

- `gauge.pressure-light.png`, `gauge.pressure-dark.png`
