# electrical.button.mushroom — conformance review

    status: pending-300dpi
    date: 2026-11-13
    reviewer: automated pass, unreviewed by a second person
    function: opens when struck and latches until the next click

## Provenance

Hand-written geometry in `assets/catalog/electrical.button.mushroom.toml`: two electrical terminals with the gap
the renderer bridges, and the actuator its name promises. The blade is drawn from the run
state, and `normally_closed = true` is the rest state the file declares. No third-party
asset and no trace of another program's symbol.

## Automated checks — `punchi-fluid-catalog` symbol lint

| Rule | Result |
|---|---|
| Primitives inside the box, no degenerate primitive | pass |
| Every line declares its class | pass: the stems are `electrical`, the actuator `working` |
| Exactly two electrical terminals | pass: the contact validation |
| Legible at palette size | pass: see the images |

## Visual check

| Requirement | Result | Evidence |
|---|---|---|
| The actuator distinguishes this part from the others | pass | `electrical.button.mushroom-light.png` |
| The rest state matches `normally_closed` | pass | the blade at rest |
| The class says what each line is | pass | dotted stems for the wires |
| Legible at 300 dpi | **not checked** | needs the maintainer's reader |

## What this review does not prove

- The 300 dpi pass is still owed: the status stays `pending-300dpi` and the export keeps the
  "not verified" note.
- The image shows the rest state; the click and the lamp are proven by
  `a_normally_closed_contact_opens_when_clicked`.
- A maintained switch is a click that stays; a true momentary press and release is a separate
  gesture and is not claimed here.

## Images

- `electrical.button.mushroom-light.png`, `electrical.button.mushroom-dark.png` — this symbol at rest.
