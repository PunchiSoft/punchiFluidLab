# electrical.contact — conformance review

    status: pending-300dpi
    date: 2026-11-13
    reviewer: automated pass, unreviewed by a second person
    function: closes the control circuit while it is pressed

## Provenance

Hand-written geometry in `assets/catalog/electrical.contact.toml`: two terminals with a gap,
the movable bar above it and the push rod with its cap. No third-party asset and no trace of
another program's symbol.

## Automated checks — `punchi-fluid-catalog` symbol lint

| Rule | Result |
|---|---|
| Primitives inside the box, no degenerate primitive | pass |
| Every line declares its class | pass: `electrical`, drawn dotted |
| Exactly two electrical terminals | pass: the contact validation |
| Legible at palette size | pass: see the images |

## Visual check

| Requirement | Result | Evidence |
|---|---|---|
| A visible gap between the terminals | pass | the bar is above the gap, not across it |
| The actuator is drawn, not implied | pass | the push rod and its cap |
| The class says what the line is | pass | `electrical`, dotted |
| Legible at palette size (24–30 px) | pass | `catalog-dark.png`, sidebar row |
| Legible at 300 dpi | **not checked** | needs M2 |

## Decisions this symbol carries

- **A contact is a file with two terminals** ([ADR 0017](../../docs/adr/0017-the-electrical-circuit-is-a-second-domain.md)):
  `normally_closed = true` would be another file, not a branch in the code.
- **The gap is the state.** The symbol shows the rest state; the live state is the model's,
  and it arrives with the control model in the next slice.

## What is still open

- 300 dpi render and a second reader's signature.
- IEC 60617 is the reference for electrical symbols; this geometry is our contract until a
  reader checks it against the edition in force.

## Images

- `electrical.contact-light.png`, `electrical.contact-dark.png`
