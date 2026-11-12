# chart.recorder — conformance review

    status: pending-300dpi
    date: 2026-11-12
    reviewer: automated pass, unreviewed by a second person
    function: charts one quantity of another component over the last 60 s

## Provenance

Hand-written geometry in `assets/catalog/chart.recorder.toml`: the frame of the paper and
one trace inside it. No third-party asset and no trace of another program's symbol.

## Automated checks — `punchi-fluid-catalog` symbol lint

| Rule | Result |
|---|---|
| Primitives inside the box, no degenerate primitive | pass |
| Every line declares its class | pass: `working` |
| No ports, because a recorder observes and is not connected | pass: the recorder validation |
| Window 1–600 s and rate 1–50 samples per second, product bounded | pass |
| Legible at palette size | pass: see the images |

## Visual check

| Requirement | Result | Evidence |
|---|---|---|
| The frame is the instrument; the live curve is the model's, drawn inside it | pass | the drawn trace sits inside the frame and the simulation replaces it with the series |
| The symbol does not promise a connection | pass | no stubs, no numerals, no ports |
| Legible at palette size (24–30 px) | pass | `catalog-dark.png`, sidebar row |
| Legible at 300 dpi | **not checked** | needs M2 |

## Decisions this symbol carries

- **A behaviour of its own** ([ADR 0015](../../docs/adr/0015-a-recorder-samples-a-named-quantity.md)):
  the source is a document setting, not geometry, and the samples are a run artefact that
  never enters the file.
- **No ports.** The catalog rule that a component has at least one port gains this one
  documented exception, tested in the catalog crate.
- **The window and the rate are file values**, so a longer or finer recorder is another
  file; the catalog bounds their product.

## What is still open

- 300 dpi render (M2) and a second reader's signature.
- A moving cursor and exporting the series as data are later improvements; the curve and
  the current value are the evidence today.

## Images

- `chart.recorder-light.png`, `chart.recorder-dark.png`
