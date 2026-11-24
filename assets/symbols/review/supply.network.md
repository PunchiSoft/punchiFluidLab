# supply.network — conformance review

    status: complete
    date: 2026-11-24
    reviewer: Cristian Muñoz (owner), second reader
    function: compressed-air network, 4 bar: the air arrives from the plant

## The claim

- **Function, in one sentence**: the connection to the workshop network, drawn when the
  machine that makes the air is elsewhere in the plant.
- **Family**: supply.
- **Normative source**: ISO 1219-1:2006, 9.6.4.1, rule `RF059` (pneumatic source of energy),
  read in the owner's copy and confirmed on the 2012 basis in JIS B 0125-1:2020; `RF060` is
  the filled hydraulic source.
- **Ports**: `1`, the supply line.

## Reference and assumptions

- **Reference read**: the standard's rule `RF059` and the project's own drawing.
- **Observed**: the rule draws the outline triangle with its point at the fluid supply line
  and a side dimension of `4M`; the triangle's height is the dimension (4M = 10 mm with
  M = 2,5 mm).
- **Required by the owner**: the source symbol in the air line, before the service unit.
- **Stated by the standard**: the outline triangle is a pneumatic source of energy; the
  filled one is the hydraulic source.
- **Assumption**: the port prints `1` because the source's line is the circuit's supply line;
  the standard's figure prints no numeral. The triangle is rotated to the vertical line, so
  its point is up: the standard's form turned with the line it feeds.

## Measures

| Feature | Requirement | Drawing |
|---|---|---|
| Triangle | outline, equilateral, point at the line | apex (0,500; 0,300); base (0,211; 0,800)–(0,789; 0,800) |
| Height | `4M` = 10 mm of the 20 mm box | 0,500 of the box |
| Stem | the connection, clear of the drawing | (0,500; 0,300) → (0,500; 0,080) |
| Port | the supply line | `1` at (0,500; 0,080) |

## Evidence

- 300 dpi sheet: `supply.network-300.png`; light and dark captures fitted.
- Symbol lint: passes.
- Fixture: `assets/symbols/fixtures/supply.network.txt` (unchanged: the form was already the
  rule's).

## Checklist

| Question | Answer |
|---|---|
| Is the body one equal square per position? | not a valve: the body is the triangle |
| Is every closed port of every position marked? | not applicable |
| Is every port the standard draws drawn? | one port, the supply line |
| Do the numerals match the declared ports? | `1`, the supply line |
| If it is a machine, is the drive bar on the standard's side? | not a machine: the drive is drawn elsewhere |
| If it is adjustable, is the arrow on the diameter? | not adjustable |
| Is every numeral clear of the frame, the body and the lines? | yes |
| Is the symbol legible at palette size and at 300 dpi? | yes, both captures |

## What is still open

- Whether a source's single connection should print `1` (the supply line) or `2` (an outlet,
  `RF053`) is a contract question, recorded here rather than decided alone.
- The 2012 text is read through the Japanese adoption (MOD).

## Sign-off

- **Status**: `complete`
- **Reviewed by**: Cristian Muñoz (owner)
- **Date**: 2026-11-24
- **What changed since the last record**: nothing in the record changed; the
  second reader signed its measures and open items on 2026-11-24.

## Images

- `supply.network-300.png`
- `supply.network-light.png`, `supply.network-dark.png`
