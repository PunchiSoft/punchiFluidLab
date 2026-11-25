# supply.network — conformance review

    status: complete
    date: 2026-11-25
    reviewer: Cristian Muñoz (owner)
    function: compressed-air network, 4 bar: the air arrives from the plant

## The claim

- **Function, in one sentence**: the connection to the workshop network, drawn when the
  machine that makes the air is elsewhere in the plant.
- **Family**: supply.
- **Normative source**: ISO 1219-1:2006, 9.6.4.1, rule `RF059` (pneumatic source of energy),
  read in the owner's copy and confirmed on the 2012 basis in JIS B 0125-1:2020; `RF060` is
  the filled hydraulic source.
- **Ports**: `1`, the supply line, **without a printed numeral**: the owner's reference of
  2026-11-25 shows the bare triangle, and the source is the connection, not a numbered port.

## Reference and assumptions

- **Reference read**: the standard's rule `RF059` and the owner's reference of 2026-11-25
  (a scene with the source next to a double-acting cylinder). The reference identifies the
  **size** relative to the cylinder; no trace or pixel copy enters the repository.
- **Owner's decision, 2026-11-25**: the source is the **small triangle at the plant's scale**:
  an **8 × 8 mm box** in the sheet, not the 20 mm square the previous record used. The
  triangle is 4.6 × 4.0 mm inside that box; the reference's triangle is about 4.4 × 4.2 mm
  against the cylinder's 18.48 mm barrel.
- **Owner's decision, 2026-11-25**: the source prints **no numeral**. The previous record
  printed `1` as an assumption; the reference shows none, and the port id stays `1` for the
  solver and the hoses (`show_numeral = false`).
- **Correction, not deviation**: the previous 20 mm box and the `1` were project assumptions,
  not the standard's size; the owner corrected them. The symbol is conformant at the
  reference's size.
- **Stated by the standard**: the outline triangle is a pneumatic source of energy; the
  filled one is the hydraulic source; its point is at the supply line.
- **Assumption**: the triangle is turned with the vertical line, so its point is up; the
  standard's form is turned with the line it feeds.

## Measures

The symbol box is **8 × 8 mm**; the triangle is 4.6 × 4.0 mm.

| Feature | Requirement | Drawing |
|---|---|---|
| Box | owner's reference, 8 × 8 mm | `box = [8, 8]` |
| Triangle | outline, equilateral, point at the line | apex (4.00; 2.40); base (1.69; 6.40)–(6.31; 6.40) |
| Height | 4 mm in the 8 mm box (4M with M = 1 mm) | 2.40 … 6.40 |
| Stem | the connection, clear of the drawing | (4.00; 2.40) → (4.00; 0.64) |
| Port | the supply line | `1` at (4.00; 0.64), **no numeral** |

## Evidence

- 300 dpi sheet: `supply.network-300.png`; light and dark captures fitted.
- Symbol lint: passes.
- Fixture: `assets/symbols/fixtures/supply.network.txt`, regenerated at the 8 mm box.
- Layout: `the_composition_uses_the_boxes_the_files_declare` checks that the source takes
  its declared 8 × 8 mm box in the worked example, not the 20 mm square.
- Numeral: `the_network_source_prints_no_numeral` proves the source draws no designation.
- The source in the start template reads the same catalog box through `symbol_box_mm`.

## Checklist

| Question | Answer |
|---|---|
| Is the body one equal square per position? | not a valve: the body is the triangle |
| Is every closed port of every position marked? | not applicable |
| Is every port the standard draws drawn? | one port, the supply line |
| Do the numerals match the declared ports? | the port id is `1`; the symbol prints no numeral, as the reference |
| If it is a machine, is the drive bar on the standard's side? | not a machine: no drive mark |
| If it is adjustable, is the arrow on the diameter? | not adjustable |
| Is the symbol the reference's size? | yes: 8 × 8 mm box, triangle 4.6 × 4.0 mm |
| Is the symbol legible at palette size and at 300 dpi? | yes, both captures |

## What is still open

- The 2012 text is read through the Japanese adoption (MOD), not the ISO text itself.
- The numeral is absent by the owner's reference; a reader who needs the port id sees it in
  the document, not on the sheet.
- The source is modelled as a fixed 4 bar supply; it does not model the plant's network.

## Sign-off

- **Status**: `complete`
- **Reviewed by**: Cristian Muñoz (owner)
- **Date**: 2026-11-25
- **What changed since the last record**: the box is 8 × 8 mm, the triangle is 4.6 × 4.0 mm,
  and the numeral is gone; the previous 20 mm box and the `1` were project assumptions the
  owner corrected. The record is not a deviation: it is the reference's size.

## Images

- `supply.network-300.png`
- `supply.network-light.png`, `supply.network-dark.png`
