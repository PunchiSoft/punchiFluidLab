# electrical.vacuum_switch — conformance review

    status: pending-300dpi
    date: 2026-11-13
    reviewer: automated pass, unreviewed by a second person
    function: closes when its port falls below the -0.4 bar threshold

## Provenance

Hand-written geometry in `assets/catalog/electrical.vacuum_switch.toml`: a rectangular pressure actuator, a perpendicular line, a working stem to the
port, a dashed link and a two-terminal contact. The **blade** is drawn by the renderer from
the switch state (ADR 0024). No third-party asset and no trace of another program's symbol.

## Automated checks — `punchi-fluid-catalog` symbol lint

| Rule | Result |
|---|---|
| Primitives inside the box, no degenerate primitive | pass |
| Every line declares its class | pass: the stem is `working`, the terminals `electrical` |
| Two electrical terminals and one working port | pass: the pressure switch validation |
| The threshold is inside the bounds | pass: -1 to 100 bar |
| Legible at palette size | pass: see the images |

## Visual check

| Requirement | Result | Evidence |
|---|---|---|
| The rectangular actuator and the contact are distinct | pass | `electrical.vacuum_switch-light.png` |
| A visible gap at rest | pass | the open bar above the terminals |
| The class says what each line is | pass | dotted stems for the wires |
| Legible at 300 dpi | **not checked** | needs the maintainer's reader |

## What this review does not prove

- The 300 dpi pass is still owed: the status stays `pending-300dpi` and the export keeps the
  "not verified" note.
- The image shows the rest state. The transition is proven by
  `a_pressure_switch_follows_its_line_up_and_down`, which charges the working line, steps
  until the switch closes and exhausts it until it opens.
- A vacuum switch is the same code with `above = false`; its file is validated and drawn, and
  the condition is the same comparison on the other side of atmosphere.

## Images

- `electrical.vacuum_switch-light.png`, `electrical.vacuum_switch-dark.png` — this symbol at rest.
