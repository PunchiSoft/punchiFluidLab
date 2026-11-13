# electrical.supply — conformance review

    status: pending-300dpi
    date: 2026-11-13
    reviewer: automated pass, unreviewed by a second person
    function: feeds the electrical control circuit at 24 V

## Provenance

Hand-written geometry in `assets/catalog/electrical.supply.toml`: the two plates of a d.c.
source, a stem to each terminal, and the plus and minus drawn as lines. No third-party asset
and no trace of another program's symbol.

## Automated checks — `punchi-fluid-catalog` symbol lint

| Rule | Result |
|---|---|
| Primitives inside the box, no degenerate primitive | pass |
| Every line declares its class | pass: `electrical`, drawn dotted |
| Two electrical terminals | pass: the supply declares `L` and `N` |
| Voltage finite and positive | pass |
| Legible at palette size | pass: see the images |

## Visual check

| Requirement | Result | Evidence |
|---|---|---|
| Two plates, the long one positive, the short one negative | pass | the pair of vertical lines |
| The polarity is drawn, not typed | pass | the plus and the minus are line primitives |
| Terminals where the wires attach | pass | the stems and the connection circles |
| Legible at palette size (24–30 px) | pass | `catalog-dark.png`, sidebar row |
| Legible at 300 dpi | **not checked** | needs M2 |

## Decisions this symbol carries

- **A second domain in the same document** ([ADR 0017](../../docs/adr/0017-the-electrical-circuit-is-a-second-domain.md)):
  an electrical terminal is a port role, and the check never calls it an open pneumatic
  connection.
- **The class says what the line is**: the electrical class is dotted, and a hose between two
  electrical terminals is drawn that way without the user choosing anything.

## What is still open

- 300 dpi render and a second reader's signature.
- The control model — a live line, a closed contact moving a valve — is the next slice.
- IEC 60617 is the reference for electrical symbols; this geometry is our contract until a
  reader checks it against the edition in force.

## Images

- `electrical.supply-light.png`, `electrical.supply-dark.png`
