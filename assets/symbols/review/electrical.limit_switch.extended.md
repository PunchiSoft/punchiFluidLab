# electrical.limit_switch.extended — conformance review

    status: pending-300dpi
    date: 2026-11-13
    reviewer: automated pass, unreviewed by a second person
    function: closes when the chosen cylinder reaches the far end

## Provenance

Hand-written geometry in `assets/catalog/electrical.limit_switch.extended.toml`: a two-terminal contact with a roller
lever on top, mirrored so the two ends read differently. The **blade** is drawn by the
renderer from the switch state, and the cylinder it watches is the placement source the
inspector writes (ADR 0023). No third-party asset and no trace of another program's symbol.

## Automated checks — `punchi-fluid-catalog` symbol lint

| Rule | Result |
|---|---|
| Primitives inside the box, no degenerate primitive | pass |
| Every line declares its class | pass: the stems are `electrical`, the lever `working` |
| Exactly two electrical terminals | pass: the contact validation |
| The watched end is declared | pass: `end = "extended"` |
| Legible at palette size | pass: see the images |

## Visual check

| Requirement | Result | Evidence |
|---|---|---|
| The roller is drawn, not implied | pass | `electrical.limit_switch.extended-light.png` |
| A visible gap at rest | pass | the open bar above the terminals |
| The two ends are distinguishable | pass | the lever points to the watched side |
| The class says what each line is | pass | dotted stems for the wires |
| Legible at 300 dpi | **not checked** | needs the maintainer's reader |

## What this review does not prove

- The 300 dpi pass is still owed: the status stays `pending-300dpi` and the export keeps the
  "not verified" note.
- The image shows the rest state; the transition is proven by
  `a_limit_switch_closes_at_the_end_of_its_cylinder`, which binds a cylinder, presses its
  valve and steps until the rod is out.
- The switch closes at `>= 0.98` or `<= 0.02` of the stroke, the same margins the telemetry
  uses to call the rod out or home.

## Images

- `electrical.limit_switch.extended-light.png`, `electrical.limit_switch.extended-dark.png` — this symbol at rest.
