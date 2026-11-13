# electrical.relay.latching — conformance review

    status: pending-300dpi
    date: 2026-11-13
    reviewer: automated pass, unreviewed by a second person
    function: sets with one coil, resets with the other, and holds in between

## Provenance

Hand-written geometry in `assets/catalog/electrical.relay.latching.toml`: two coil
rectangles, four coil terminals, one contact and two dashed links. The **bar** is drawn by
the renderer from the latched state (ADR 0025). No third-party asset and no trace of another
program's symbol.

## Automated checks — `punchi-fluid-catalog` symbol lint

| Rule | Result |
|---|---|
| Primitives inside the box, no degenerate primitive | pass |
| Every line declares its class | pass: the stems are `electrical`, the coils `working` |
| Six distinct electrical terminals in three named pairs | pass: the latching relay validation |
| The delay is at zero for an instant relay | pass |
| Legible at palette size | pass with the caveat below |

## Visual check

| Requirement | Result | Evidence |
|---|---|---|
| The two coils are distinguishable | pass | `electrical.relay.latching-light.png` |
| One contact, linked from both windings | pass | the two dashed links |
| A visible gap at rest | pass | the open bar between 11 and 14 |
| The terminal names sit close at palette size | **caveat** | A2/R1 are tight; the 300 dpi pass must confirm |
| Legible at 300 dpi | **not checked** | needs the maintainer's reader |

## What this review does not prove

- The 300 dpi pass is still owed: the status stays `pending-300dpi` and the export keeps the
  "not verified" note.
- The image shows the rest state. The latch is proven by
  `a_latching_relay_holds_until_the_reset_coil`, which sets, releases, holds, resets and
  releases again.
- A coil live at the same time as the other is not a priority rule: the model holds the last
  state, and a circuit that drives both is a fault the check will learn to report.

## Images

- `electrical.relay.latching-light.png`, `electrical.relay.latching-dark.png` — this symbol
  at rest.
