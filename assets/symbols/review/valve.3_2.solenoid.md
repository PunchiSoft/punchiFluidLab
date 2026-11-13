# valve.3_2.solenoid — conformance review

    status: pending-300dpi
    date: 2026-11-13
    reviewer: automated pass, unreviewed by a second person
    function: pressurises the working line while its coil is energised

## Provenance

Hand-written geometry in `assets/catalog/valve.3_2.solenoid.toml`: the same two-square 3/2
body as the lever valve, with the solenoid windings drawn by the renderer from `[actuator]`
and a coil terminal at the actuator end. No third-party asset and no trace of another
program's symbol.

## Automated checks — `punchi-fluid-catalog` symbol lint

| Rule | Result |
|---|---|
| Squares equal and aligned, one per position | pass |
| Every position opens or closes something | pass: vent at rest, supply when actuated |
| Spring at the end opposite the actuator | pass |
| The electrical terminal is not an orphan | pass: the lint exempts electrical ports |
| Legible at palette size | pass: see the images |

## Visual check

| Requirement | Result | Evidence |
|---|---|---|
| Two equal squares, counted from the spring end | pass | rest on the right, next to the spring |
| The windings are the solenoid, at the end opposite the spring | pass | drawn by the renderer |
| The coil terminal is where the control circuit arrives | pass | `Y1` on the actuator side, with its numeral |
| The rest position vents the working line | pass | 2→3 in the first square |
| Legible at palette size (24–30 px) | pass | `catalog-dark.png`, sidebar row |
| Legible at 300 dpi | **not checked** | needs M2 |

## Decisions this symbol carries

- **The coil is a terminal, not a label** ([ADR 0018](../../docs/adr/0018-the-control-circuit-is-a-boolean-net.md)):
  the valve is actuated while the terminal is live and the spring returns it when the
  control circuit opens.
- **The electrical port is exempt from the valve rules**: no position opens or closes a
  coil, and the lint says so instead of asking for a connection that would be wrong.

## What is still open

- 300 dpi render and a second reader's signature.
- IEC 60617 is the reference for the coil's drawing; this geometry is our contract until a
  reader checks it.

## Images

- `valve.3_2.solenoid-light.png`, `valve.3_2.solenoid-dark.png`
