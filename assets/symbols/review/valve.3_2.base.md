# valve.3_2.base — conformance review

    status: pending-300dpi
    date: 2026-11-22
    reviewer: automated pass, pending a person's sign-off
    function: 3/2 directional control valve in its base form, two positions, no return, no actuation

This is the body a student places and configures in the valve dialog. The review checks
the base form only: the return and the actuation are fitted later and are their own
variants.

## Provenance

Geometry is written by hand in `assets/catalog/valve.3_2.base.toml`: the same equal-square
body this repository uses for its 3/2 valves, with the spring and the lever removed. No
asset, trace, screenshot or scan from another program or from the standard is used.

## Automated checks — `punchi-fluid-catalog` symbol lint

| Rule | Result |
|---|---|
| Body splits into one equal square per position | pass (2 positions, 0.44 × 0.44 each) |
| Every position opens or closes something | pass (position 1 vents 2→3 and blocks 1; position 2 feeds 1→2 and blocks 3) |
| Every port is used, every numeral unique | pass (1, 2, 3) |
| No return declared and none promised | pass (`spring_return` absent; no `[spring]`) |
| No actuator declared and none promised | pass (`actuation = "none"`; no `[actuator]`) |
| No primitive outside the box, none degenerate | pass (the definition declares no free primitive) |
| Every line declares its class | pass (the renderer draws the stubs and the paths) |
| Nothing drawn inside the body by hand | pass (the symbol list is empty) |

## Measures

| Feature | Requirement | Drawing |
|---|---|---|
| Body | two equal squares sharing an edge, side by side | x 0.06, y 0.28, 0.88 × 0.44; each square 0.44 |
| Ports | 1 supply, 2 working, 3 exhaust, on the declared sides | 1 and 3 below, 2 above, read at the default square |
| Default position | the square the ports are read with | position 1 (left), supply blocked |
| Blocking marks | the port each position closes, inside its own square | 1 in position 1; 3 in position 2 |
| Return and actuator | absent in the base form | none declared |
| Numerals | outside the body, at their stub, once | 1, 2, 3 at their offsets |

## Visual check

| Requirement | Result | Evidence |
|---|---|---|
| Two equal squares, side by side | pass | the fixture `valve.3_2.base.txt` |
| No spring, no actuator, no lever | pass | the fixture has no spring or actuator call |
| Port stubs and numerals at their declared positions | pass | the fixture |
| Blocked port marked inside its own square | pass | the fixture |
| Legible at palette size (24–30 px) | pass | `valve.3_2.base-light.png`, `valve.3_2.base-dark.png` |
| Legible at 300 dpi | pass, second reader owed | `valve.3_2.base-300.png` |

## Sign-off

- **Status**: `pending-300dpi`
- **Reviewed by**: pending
- **Date**: 2026-11-22
