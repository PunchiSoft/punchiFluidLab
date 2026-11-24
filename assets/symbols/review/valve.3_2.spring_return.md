<!-- SPDX-FileCopyrightText: 2026 PunchiSoft -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Review record: `valve.3_2.spring_return`

    status: pending-300dpi
    date: 2026-11-24
    reviewer: automated pass, unreviewed by a second person
    function: 3/2 directional control valve, spring return, normally closed, manual lever

## The claim

- **Function, in one sentence**: the spring holds the spool in the rest position, where
  the working line vents and the supply is shut; pressing the lever feeds the working line
  and shuts the exhaust, and releasing it returns the spool.
- **Family**: valve (directional control), complete valve with a return and an actuation.
- **Normative source**: ISO 1219-1:2006, 6.1.2.5, figure `X10270` (registration
  `7.1.2.7`), read in the owner's copy; confirmed on the 2012 basis in JIS B
  0125-1:2020, 6.1.2.5 / `7.1.2.7`, same figure and registration. `X10270` is the
  registered **complete valve**: a 3/2, two-position, spring-return valve with a roller
  lever. This file is that complete valve with the **plain lever** `688V1` (8.5.16)
  instead of the roller lever; the body, the ports and the spring are `X10270`'s, and the
  record names that one module difference. The 3/2 body is also the one of `X10280`
  (6.1.2.6). Rules: `RF052` and `RF053`.
- **Ports**: `1` supply · `2` working · `3` exhaust — all three drawn once, at the rest
  square (the right one, against the spring).

## Reference and assumptions

- **Reference read**: the standard's figures only. No capture of another application was
  used for the shape, and no figure or text is stored in the repository.
- **Observed**: `X10270` draws the two equal squares, the spring against the right square
  and the lever at the left. The working port `2` is on the top edge, aligned with the
  supply `1` on the bottom edge; the exhaust `3` is on the bottom edge at the other
  quarter. The rest square connects `2` to `3` and blocks `1`; the actuated square
  connects `1` to `2` and blocks `3`. `X10270`'s lever is a roller lever: a pivot at the
  body end, an arm, and a roller at the far end.
- **Required by the owner**: the variant files are the complete valves with the same body
  as the base; this one fits the spring return and a manual lever.
- **Stated by the standard**: the positions, the spring side and the flow paths above;
  `688V1` draws a lever with its pivot circle at the body end and the grip at the far end.
- **Assumption**: the file uses the plain lever module `688V1` in place of `X10270`'s
  roller lever. No registered X-number combines a plain lever with the 3/2 spring return;
  the record cites the complete figure it is built from and keeps the substitution
  explicit. The port numerals follow `RF053` / ISO 11727, not a printed figure.

## Measures

The symbol box is 20 mm; the normalised coordinates map to millimetres on both axes.

| Feature | Requirement | Drawing |
|---|---|---|
| Body | one equal square per position, shared edge | x 1.2 mm, y 5.6 mm, 17.6 × 8.8 mm; two squares 8.8 × 8.8 mm |
| Ports | `1` supply, `2` working, `3` exhaust, on the figure's sides | `1` (12.2, 20.0), `2` (12.2, 0.0), `3` (16.6, 20.0) mm; 2.2 mm and 6.6 mm inside the rest square |
| Rest position | the square against the spring | position 0, right square, drawn active |
| Flow paths | straight inside their own square, arrow at the far end | rest `2`→`3` diagonal, arrow at `3`; actuated `1`→`2` vertical, arrow at `2` |
| Blocking marks | inside the square that shuts the port | `1` at rest; `3` when actuated; stem 1.76 mm, bar 2.64 mm |
| Spring | right end, outside the body, zigzag of proportional size | starts 1.14 mm clear of the face; coil 3.43 mm long; full amplitude 3.52 mm |
| Lever | left end, pivot circle at the body, grip at the far end | pivot 0.57 mm off the face, radius 0.39 mm; arm 2.40 mm at 20°; grip bar 2.06 mm across the far end |
| Numerals | outside the body, beside the stub, once | 3.0 mm cap; `1` and `3` below, `2` above |

## Evidence

- 300 dpi sheet, print ink:
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- valve.3_2.spring_return assets/symbols/review/valve.3_2.spring_return-300.png`
- Light and dark captures, fitted to the drawing:
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- valve.3_2.spring_return assets/symbols/review/valve.3_2.spring_return-light.png --ink light --fit`
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- valve.3_2.spring_return assets/symbols/review/valve.3_2.spring_return-dark.png --ink dark --fit`
- Symbol lint: passes.
- Geometry fixture regenerated and read:
  `assets/symbols/fixtures/valve.3_2.spring_return.txt`. The lever now has its pivot at
  the body and its grip at the far end; the ports moved to the figure's offsets.
- New test: `the_lever_pivots_against_the_body_and_grips_at_the_far_end`, which fails if
  the pivot circle leaves the body half of the glyph.

## Checklist

| Question | Answer |
|---|---|
| Is the body one equal square per position? | yes: two 8.8 × 8.8 mm squares |
| Is every closed port of every position marked? | yes: `1` at rest, `3` when actuated, inside their own squares |
| Is every port the standard draws drawn? | yes: `1`, `2`, `3` |
| Do the numerals match the declared ports (1 inlet, 2/4 outlets, 3/5 exhausts)? | yes: `1`, `2`, `3` |
| If it is a machine, is the drive bar on the standard's side? | not a machine |
| If it is adjustable, is the arrow on the diameter or through the element, and filled? | not adjustable |
| Is every numeral clear of the envelope/frame, the body and the primitive lines? | yes: the numerals sit in the band outside the body, beside their stubs |
| Does the arrow point at the far end of the connection? | yes: at `3` at rest, at `2` when actuated |
| Are spring and actuator on the declared sides? | yes: lever at the left, spring at the right, opposite ends |
| Is the symbol legible at palette size? | yes: the light and dark captures |
| Is the symbol legible at 300 dpi? | yes: the 300 dpi sheet; the second reader's signature is owed |

## Sign-off

- **Status**: `pending-300dpi`
- **Reviewed by**: pending (a person, not the assistant)
- **Date**: 2026-11-24
- **What changed since the last record**: the file's comment placed position 0 at the
  leftmost square while the spring is on the right; the comment is corrected and the
  record counts position 0 in the right square. The lever's pivot moved from the far end
  to the body end, with the grip at the far end per `688V1`; the ports moved to the
  figure's offsets.

## What is still open

- The second reader's signature; until then the record stays `pending-300dpi` and the
  export keeps the unverified note.
- `X10270`'s lever carries a roller; this file uses the plain lever `688V1`. No
  registered X-number covers that combination, so the review cites `X10270` for the body
  and the return and `688V1` for the actuation.
- The port numerals follow `RF053` / ISO 11727; the figures do not print them.

## Images

- `valve.3_2.spring_return-light.png`, `valve.3_2.spring_return-dark.png`
- `valve.3_2.spring_return-300.png`
