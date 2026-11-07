# junction.tee — conformance review

    status: pending-300dpi
    date: 2026-11-06
    reviewer: automated pass, unreviewed by a second person
    function: three hoses meet at one point

## Provenance

Hand-written geometry in `assets/catalog/junction.tee.toml`: one filled circle, radius
0.11 of the symbol box, and nothing else. No third-party asset, no trace of another
program's symbol.

## Automated checks — `punchi-fluid-catalog` symbol lint

| Rule | Result |
|---|---|
| Geometry inside the box, no degenerate primitive | pass |
| Every line declares its class | pass (the circle's outline is a `working` line) |
| Every port used and numbered | not applicable: the file declares no valve position, so the port rules that apply to valves do not apply here |

## Visual check

| Requirement | Result | Evidence |
|---|---|---|
| A junction is a filled dot and carries no shape of its own | pass | screenshots |
| No port numerals, because the three ports are one point and the numerals would overprint | pass | both screenshots: a clean dot, no smudge |
| Legible at palette size | pass | `catalog-dark.png` |
| Legible at 300 dpi | **not checked** | needs M2 |

## Decisions this symbol carries

- **Three ports on one point.** A junction is a node, so every hose that arrives costs
  the same; that is what lets the document stay port-to-port instead of growing a node
  type. [ADR 0006](../../docs/adr/0006-a-junction-is-a-component.md) records it.
- **The renderer draws no numerals for this behaviour.** Three identifiers on one point
  printed on top of each other and read as a smudge, which is not a symbol. ISO 1219-1
  labels junctions by their connections, not by numerals.
- **No box, no wedge, no lettering.** The symbol box is a small square only so the dot
  can be hit and selected; nothing of it is drawn.

## What is still open

- 300 dpi render (M2) and a second reader's signature.
- The dot's radius relative to the hoses it joins was chosen for legibility and was not
  measured against a printed copy of the standard from this machine. If a measurement
  later disagrees, the radius changes in the file and nowhere else.

## Images

- `junction.tee-dark.png`, `junction.tee-light.png`
