# flow.meter — conformance review

    status: pending-300dpi
    date: 2026-11-12
    reviewer: automated pass, unreviewed by a second person
    function: measures the air that goes through it, in normal litres per minute

## Provenance

Hand-written geometry in `assets/catalog/flow.meter.toml`: a circle for the dial, one stub
on each side where the line enters and leaves, and an arrow from port 1 to port 2. No
third-party asset and no trace of another program's symbol.

## Automated checks — `punchi-fluid-catalog` symbol lint

| Rule | Result |
|---|---|
| Primitives inside the box, no degenerate primitive | pass |
| Every line declares its class | pass: `working` |
| Exactly two working ports | pass: the flow meter validation |
| Conductance and nominal flow finite and positive | pass |
| Legible at palette size | pass: see the images |

## Visual check

| Requirement | Result | Evidence |
|---|---|---|
| The dial is the symbol; the number is a reading, not a primitive | pass | the value is drawn from the model inside the dial |
| The meter interrupts the line, so it has one port on each side | pass | the stubs and the numerals |
| The arrow marks the positive direction of the reading | pass | port 1 to port 2 |
| Legible at palette size (24–30 px) | pass | `catalog-dark.png`, sidebar row |
| Legible at 300 dpi | **not checked** | needs M2 |

## Decisions this symbol carries

- **A behaviour of its own** ([ADR 0014](../../docs/adr/0014-a-flow-meter-is-a-restriction-that-reports.md)):
  a gauge has one port and adds nothing; a meter has two and is the restriction it measures.
- **The conductance and the scale are file values**: a wider meter is another file.
- **The reading is text from the model**, in normal litres per minute at 0 °C and
  1 013.25 hPa, the normal litre ISO 6358 uses; the arrow fixes what counts as positive,
  the number carries the value.

## What is still open

- 300 dpi render (M2) and a second reader's signature.
- The normative shape of the meter in the instrument family is recorded as our contract in
  [docs/symbology.md](../../docs/symbology.md), not as a copy of a published figure; the
  review is `pending` until a second reader checks it.
- A total volume is a later improvement: it belongs with the recorder, not with the meter.

## Images

- `flow.meter-light.png`, `flow.meter-dark.png`
