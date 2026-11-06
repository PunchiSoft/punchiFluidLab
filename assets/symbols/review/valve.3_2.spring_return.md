# valve.3_2.spring_return — conformance review

    status: pending-300dpi
    date: 2026-11-06
    reviewer: automated pass, unreviewed by a second person
    function: 3/2 directional control valve, spring return, normally closed

Three squares of judgement go into this file: the automated `symbol lint`, the visual
check recorded below, and the 300 dpi render. The first two are done. The third needs
the export pipeline (M2), so the status is `pending-300dpi` and the export keeps
labelling the document as unverified.

## Provenance

Geometry is written by hand in `assets/catalog/valve.3_2.spring_return.toml` and in the
renderer's part functions. No asset, trace, screenshot or scan from another program or
from the standard is used, and the standard's text is not reproduced.

## Automated checks — `punchi-fluid-catalog` symbol lint

| Rule | Result |
|---|---|
| Body splits into one equal square per position | pass (2 positions, 0.44 × 0.44 each) |
| Every position opens or closes something | pass (rest vents 2→3 and blocks 1; actuated feeds 1→2 and blocks 3) |
| Every port is used, every numeral unique | pass (1, 2, 3) |
| `spring_return` implies a declared spring | pass (`[spring] side = "right"`) |
| `actuation` implies a declared actuator | pass (`[actuator] lever, left`) |
| No primitive outside the box, none degenerate | pass |
| Every line declares its class | pass (no free lines: stubs, paths and marks are rendered) |
| Nothing drawn inside the body by hand | pass (the symbol list is empty) |

One test per rule lives in `crates/punchi-fluid-catalog/src/lint.rs`; a built-in that
breaks one fails `cargo test --workspace`.

## Visual check, read against the requirements of ISO 1219-1

| Requirement | Result | Evidence |
|---|---|---|
| Two positions, equal squares sharing an edge, side by side | pass | screenshot, both themes |
| Rest position drawn in the square against the spring | pass | rest is the right square, spring at the right end |
| Actuator at one end, spring at the other, both outside the body | pass | lever left, spring right |
| Port stubs straight, attached where the definition puts the port | pass | 1 and 3 below the rest square, 2 above it |
| Port numerals outside the body, at their stub, identical in both squares | pass | 1 bottom-left, 2 top, 3 bottom-right |
| Blocked port marked inside its own square, not omitted | pass | T over port 1 at rest, T over port 3 when actuated |
| Flow path a straight arrow inside its own square, pointing along the connection | pass | rest: 2→3 to the exhaust; actuated: 1→2 |
| Spool position readable without colour | pass | the active square is filled *and* the other position's path is dimmed |
| Legible at palette size (24–30 px) | pass | `catalog-dark.png`, sidebar row |
| Legible when printed at 300 dpi | **not checked** | needs M2 |

## Correction of 2026-11-06

The first drawing put each position's flow path across the boundary into the
neighbouring square and marked the blocked port inside a square that did not always hold
it. Both are fixed: a position is now drawn entirely inside its own square, and the
blocking mark crosses the port's own stub. The images next to this file were regenerated
after the fix, with the same drawing code as the sheet.

## Correction of 2026-11-06, second pass

The symbol was drawn mirrored and its shut-off mark sat outside the body, so it did not
read like the standard's: the spring was at the left with the rest square against it and
the lever at the right, the stubs came out slanted because the port positions were read
in the body's frame instead of the symbol box's, and the closed port was a bar across the
stub rather than the T of a shut-off inside the square. Now the actuator is at the left
and the spring at the right, positions are counted from the spring end so the rest square
is the one the spring holds, ports are read in the symbol's own frame and their stubs are
straight, the flow path is a single straight arrow from port to port inside the square,
and every position draws its own blocking T in the ink of its own square. Two tests hold
this: one that the bar of each position falls inside the square of that position, and one
that the position at rest is the square against the spring.

## What is still open

- The 300 dpi render, and therefore the signed-off status.
- The edition of the port-identification standard used for the numerals (1/2/3/4/5/12/14)
  could not be confirmed from this machine; the numbering follows the fluid-power
  convention assumed in [docs/symbology.md](../../../docs/symbology.md).
- A second person has not signed this. Until then the status stays `pending-300dpi`
  and the export keeps the unverified note.

## Images

- `valve.3_2.spring_return-dark.png`, `valve.3_2.spring_return-light.png` — this symbol.
- `catalog-dark.png`, `catalog-light.png` — the whole catalog on one sheet
  (`PUNCHI_FLUID_PREVIEW=1 ./scripts-dev/run.sh --dark`).
