# silencer.pneumatic — conformance review

    status: pending-300dpi
    date: 2026-11-12
    reviewer: automated pass, unreviewed by a second person
    function: vents an exhaust to atmosphere through a restriction

## Provenance

Hand-written geometry in `assets/catalog/silencer.pneumatic.toml`: a cone widening away
from the port with two relief lines across its mouth. No third-party asset and no trace of
another program's symbol.

## Automated checks — `punchi-fluid-catalog` symbol lint

| Rule | Result |
|---|---|
| Primitives inside the box, no degenerate primitive | pass |
| Every line declares its class | pass: `working`; the body is a component, not a drain line |
| Exactly one exhaust port | pass: the silencer validation |
| Conductance finite and positive | pass |
| Legible at palette size | pass: see the images |

## Visual check

| Requirement | Result | Evidence |
|---|---|---|
| The cone widens away from the port: the air leaves | pass | the port is on the narrow side |
| Two relief lines distinguish it from a plain nozzle | pass | the mouth of the cone |
| The body is drawn like a component, not like a drain line | pass | `working`, solid |
| Legible at palette size (24–30 px) | pass | `catalog-dark.png`, sidebar row |
| Legible at 300 dpi | **not checked** | needs M2 |

## Decisions this symbol carries

- **A behaviour of its own** ([ADR 0016](../../docs/adr/0016-a-termination-is-a-component.md)):
  the silencer is a restriction to atmosphere, so it changes the circuit it is fitted to —
  a cylinder with one retracts more slowly than one venting through an open port.
- **No decibel is drawn or stored.** The acoustic part is not modelled, and a number that
  claimed to be a noise level would be a lie; the file says so in its summary.

## What is still open

- 300 dpi render (M2) and a second reader's signature.
- The normative figure was not traced; the requirement is our contract in
  [docs/symbology.md](../../docs/symbology.md).

## Images

- `silencer.pneumatic-light.png`, `silencer.pneumatic-dark.png`
