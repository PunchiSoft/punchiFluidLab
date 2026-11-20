<!-- SPDX-FileCopyrightText: 2026 PunchiSoft -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Review record: `<component id>`

Copy this file to `<component id>.md` and fill it before touching geometry. A review
without the clause, the measures and the sign-off is not a review; it is an opinion.

## The claim

- **Function, in one sentence** (what a reader must be able to say):
- **Family**: valve / actuator / conditioning / instrument / other
- **Ports**: `1` supply · `2`, `4` working · `3`, `5` exhaust · `12`, `14` pilot — as this
  component declares them
- **Normative source**: standard, edition and clause. If the clause cannot be cited, stop
  here and mark the record `blocked: clause`.

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
| Numerals | which sides carry them | |
| Clearance | every numeral clear of the envelope/frame, the body and the primitive lines | |

## Evidence

- 300 dpi sheet, print ink:
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- <id> review/<id>-300.png`
  (a 60 mm square, the symbol in a fixed 40 mm box; dark scheme is the next step)
- Symbol lint: passes
- Palette legibility at 24–30 px: `<id>-light.png`, `<id>-dark.png`

## Checklist

One line per normative feature, answered, with the number that supports it.

| Question | Answer |
|---|---|
| Is the body one equal square per position? | |
| Is every closed port of every position marked? | |
| Do the numerals match the declared ports? | |
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
