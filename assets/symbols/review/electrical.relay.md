# electrical.relay — conformance review

    status: pending-300dpi
    date: 2026-11-13
    reviewer: automated pass, unreviewed by a second person
    function: closes its contact while the coil is live

## Provenance

Hand-written geometry in `assets/catalog/electrical.relay.toml`: a coil rectangle with A1
and A2, a contact with terminals 11 and 14, and a dashed mechanical link between them. The
**contact bar** is drawn by the renderer from the state the net settles on, the same rule
that draws the push button's bar (ADR 0019). No third-party asset and no trace of another
program's symbol.

## Automated checks — `punchi-fluid-catalog` symbol lint

| Rule | Result |
|---|---|
| Primitives inside the box, no degenerate primitive | pass |
| Every line declares its class | pass: the stems are `electrical`, the coil is `working` |
| The two pairs are distinct electrical terminals | pass: the relay validation |
| The behaviour names its coil and contact terminals | pass: `coil = ["A1", "A2"]`, `contact = ["11", "14"]` |
| Legible at palette size | pass: see the images |

## Visual check

| Requirement | Result | Evidence |
|---|---|---|
| The coil is drawn apart from the contact | pass | `electrical.relay-light.png` |
| A visible gap at rest, with the blade tilted away | pass | `electrical.relay-light.png` |
| The mechanical link is dashed, not a wire | pass | the dashes between coil and contact |
| The terminals are named in the file, not in code | pass | A1, A2, 11, 14 |
| The class says what each line is | pass | dotted stems for the wires |
| Legible at 300 dpi | **not checked** | needs the maintainer's reader |

## What this review does not prove

- The 300 dpi pass is still owed: the status stays `pending-300dpi` and the export keeps the
  "not verified" note.
- The blade is drawn for **both** the rest state and the live state; the image shows the rest
  state. The live state is proven by the ABI test
  `a_relay_closes_its_contact_while_its_coil_is_live`, which moves a solenoid valve through
  the relay's contact.
- The dashed link uses the `drain` class for its pattern. It is a mechanical link, not a
  fluid drain; the class is only the pattern supplier until a dedicated class exists.

## Images

- `electrical.relay-light.png`, `electrical.relay-dark.png` — this symbol at rest.
