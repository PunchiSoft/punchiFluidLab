<!-- SPDX-FileCopyrightText: 2026 PunchiSoft -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Review record: `<component id>`

Copy this file to `<component id>.md` and fill it before touching geometry. A review
without the clause, the measures and the sign-off is not a review; it is an opinion.

## The claim

- **Function, in one sentence** (what a reader must be able to say):
- **Family**: valve / actuator / conditioning / instrument / other
- **Normative source**: standard, **edition read**, clause and the **registration number**
  (`X…` for the figure, `RF…` for a rule). If the clause cannot be cited, stop here and mark
  the record `blocked: clause`.
- **Ports**: `1` inlet · `2`, `4` outlets · `3`, `5` exhausts · `12`, `14` pilots — as this
  component declares them, and all of them drawn.

## Reference and assumptions

- **Reference read** (a normative figure, an owner drawing, a capture of a reference
  application): a capture is classified — it identifies the component and its arrangement,
  it is not traced, and it is not stored in the repository.
- **Observed**:
- **Required by the owner**:
- **Stated by the standard**:
- **Assumption** (what the review could not settle):

## Measures

Numbers, not adjectives. The drawing is checked against this table.

| Feature | Requirement | Drawing |
|---|---|---|
| Body | one equal square per position | |
| Ports | position on the side, in this order | |
| Arrow | sense and end | |
| Spring | side and form | |
| Actuator | side and form | |
| Blocking marks | which ports, which position | |
| Drive | the shaft bar's side and form, for a machine | |
| Adjustability | arrow on the diameter, or through the element | |
| Numerals | which sides carry them | |
| Clearance | every numeral clear of the envelope/frame, the body and the primitive lines | |

## Evidence

- 300 dpi sheet, print ink:
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- <id> review/<id>-300.png`
  (a 60 mm square, the symbol in a fixed 40 mm box)
- Light and dark captures, fitted to the drawing:
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- <id> review/<id>-light.png --ink light --fit`
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- <id> review/<id>-dark.png --ink dark --fit`
- Every built-in at once, with the same three names and options:
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- --all assets/symbols/review`
- Symbol lint: passes
- Palette legibility at 24–30 px: `<id>-light.png`, `<id>-dark.png`

## Checklist

One line per normative feature, answered, with the number that supports it.

| Question | Answer |
|---|---|
| Is the body one equal square per position? | |
| Is every closed port of every position marked? | |
| Is every port the standard draws drawn? | |
| Do the numerals match the declared ports (1 inlet, 2/4 outlets, 3/5 exhausts)? | |
| If it is a machine, is the drive bar on the standard's side? | |
| If it is adjustable, is the arrow on the diameter or through the element, and filled? | |
| Is every numeral clear of the envelope/frame, the body and the primitive lines? | |
| Does the arrow point at the far end of the connection? | |
| Are spring and actuator on the declared sides? | |
| Is the symbol legible at palette size? | |
| Is the symbol legible at 300 dpi? | |

## Sign-off

- **Status**: `blocked` / `pending-300dpi` / `complete`
- **Reviewed by**: (a person, not the assistant)
- **Date**:
- **What changed since the last record**:
