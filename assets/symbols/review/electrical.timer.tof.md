# electrical.timer.tof — conformance review

    status: pending-300dpi
    date: 2026-11-13
    reviewer: automated pass, unreviewed by a second person
    function: opens its contact after the coil has been dead for the delay

## Provenance

Hand-written geometry in `assets/catalog/electrical.timer.tof.toml`: the relay body — a coil rectangle with
A1 and A2, a contact with terminals 11 and 14, and a dashed mechanical link — plus a small
clock face on the coil that marks a delay. The **contact bar** is drawn by the renderer from
the state the net settled on (ADR 0019 and ADR 0020). No third-party asset and no trace of
another program's symbol.

## Automated checks — `punchi-fluid-catalog` symbol lint

| Rule | Result |
|---|---|
| Primitives inside the box, no degenerate primitive | pass |
| Every line declares its class | pass: the stems are `electrical`, the coil and clock are `working` |
| The two pairs are distinct electrical terminals | pass: the relay validation |
| The delay is inside the declared bounds | pass: 0 to 3600 seconds |
| Legible at palette size | pass: see the images |

## Visual check

| Requirement | Result | Evidence |
|---|---|---|
| The coil, the contact and the link are readable | pass | `electrical.timer.tof-light.png` |
| The clock face marks the delay, not a second coil | pass | the circle on the coil |
| A visible gap at rest, with the bar tilted away | pass | `electrical.timer.tof-light.png` |
| The terminals are named in the file, not in code | pass | A1, A2, 11, 14 |
| The class says what each line is | pass | dotted stems for the wires |
| Legible at 300 dpi | **not checked** | needs the maintainer's reader |

## What this review does not prove

- The 300 dpi pass is still owed: the status stays `pending-300dpi` and the export keeps the
  "not verified" note.
- The image shows the rest state; the time behaviour is proven by
  `a_tof_contact_opens_after_its_delay` with a fixed step count.
- The dashed link uses the `drain` class for its pattern. It is a mechanical link, not a
  fluid drain; the class is only the pattern supplier until a dedicated class exists.

## Images

- `electrical.timer.tof-light.png`, `electrical.timer.tof-dark.png` — this symbol at rest.
