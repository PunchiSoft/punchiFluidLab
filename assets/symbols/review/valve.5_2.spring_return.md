<!-- SPDX-FileCopyrightText: 2026 PunchiSoft -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Review record: `valve.5_2.spring_return`

    status: pending-300dpi
    date: 2026-11-24
    reviewer: automated pass, unreviewed by a second person
    function: 5/2 directional control valve, spring return, manual lever

## The claim

- **Function, in one sentence**: the spring holds the spool in the rest position, where
  the supply feeds working port `2` and `4` vents; pressing the lever feeds `4`, vents
  `2`, and releasing it returns the spool.
- **Family**: valve (directional control), complete valve with a return and an actuation.
- **Normative source**: ISO 1219-1:2006, 6.1.2.15, figure `X10400` (registration
  `7.1.2.16`) for the **body, ports and both positions**, read in the owner's copy and
  confirmed in JIS B 0125-1:2020 (6.1.2.15 / `7.1.2.16`); the **mechanical spring
  return** is the one of `X10460` (7.1.2.22), the registered 5/2 complete valve with a
  mechanical spring and an air-spring return, read in the same two copies; the **plain
  lever** is `688V1` (8.5.16). This file is the catalog's complete valve (body + lever +
  spring), but no registered X-number combines exactly those modules: `X10400` is
  treadle-controlled and has no spring, and `X10460` is directly operated and returns by
  a mechanical spring **and** an air spring. The record cites the two complete figures it
  is built from and keeps the two module differences explicit. Rules: `RF052`, `RF053`.
- **Ports**: `1` supply · `2`, `4` working · `3`, `5` exhaust — all five drawn once, at
  the rest square (the right one, against the spring).

## Reference and assumptions

- **Reference read**: the standard's figures only. No capture of another application was
  used for the shape, and no figure or text is stored in the repository.
- **Observed**: `X10400` draws two equal squares; the working ports are `4` over the left
  quarter and `2` over the right, the supply `1` in the bottom middle, `5` below the left
  quarter and `3` below the right. In the square the spring holds, the supply goes to
  `2`, `4` vents through `5`, and `3` is shut; in the other square the supply goes to `4`,
  `2` vents through `3`, and `5` is shut. `X10460` draws the same 5/2 body with a
  mechanical spring and an air spring on the return side.
- **Required by the owner**: the variant files are the complete valves with the same body
  as the base; this one fits the spring return and a manual lever.
- **Stated by the standard**: the five ports, their sides and the two positions above;
  `688V1` draws the lever with its pivot circle at the body end and the grip at the far
  end.
- **Assumption**: the file uses the plain lever `688V1` at the actuator end and the
  mechanical spring only at the return end; the air spring of `X10460` is not drawn. The
  port numerals follow `RF053` / ISO 11727.

## Measures

The symbol box is 20 mm; the normalised coordinates map to millimetres on both axes.

| Feature | Requirement | Drawing |
|---|---|---|
| Body | one equal square per position, shared edge | x 1.2 mm, y 5.6 mm, 17.6 × 8.8 mm; two squares 8.8 × 8.8 mm |
| Ports | `4`/`2` working above, `5`/`1`/`3` below, in the figure's order | `4` (11.5, 0.0), `2` (17.3, 0.0), `1` (14.4, 20.0), `5` (11.5, 20.0), `3` (17.3, 20.0) mm; offsets 1.47 / 4.40 / 7.33 mm inside the rest square |
| Rest position | the square against the spring | position 0, right square, drawn active |
| Flow paths | straight inside their own square, arrow at the far end | rest `1`→`2` diagonal (8.28 mm) and `4`→`5` vertical (7.74 mm); actuated `1`→`4` diagonal and `2`→`3` vertical |
| Blocking marks | inside the square that shuts the port | `3` at rest; `5` when actuated; stem 1.76 mm, bar 2.64 mm |
| Spring | right end, outside the body, zigzag of proportional size | starts 1.14 mm clear of the face; coil 3.43 mm long; full amplitude 3.52 mm |
| Lever | left end, pivot circle at the body, grip at the far end | pivot 0.57 mm off the face, radius 0.39 mm; arm 2.40 mm at 20°; grip bar 2.06 mm across the far end |
| Numerals | outside the body, beside the stub, once | 3.0 mm cap; `4`/`2` above, `5`/`1`/`3` below |

## Evidence

- 300 dpi sheet, print ink:
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- valve.5_2.spring_return assets/symbols/review/valve.5_2.spring_return-300.png`
- Light and dark captures, fitted to the drawing:
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- valve.5_2.spring_return assets/symbols/review/valve.5_2.spring_return-light.png --ink light --fit`
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- valve.5_2.spring_return assets/symbols/review/valve.5_2.spring_return-dark.png --ink dark --fit`
- Symbol lint: passes.
- Geometry fixture regenerated and read:
  `assets/symbols/fixtures/valve.5_2.spring_return.txt`. The working ports moved to the
  top edge and the supply/exhausts to the bottom, matching `X10400`; the lever's pivot
  moved to the body end; the rest and actuated paths and both blocking marks follow.
- New test: `the_lever_pivots_against_the_body_and_grips_at_the_far_end`.

## Checklist

| Question | Answer |
|---|---|
| Is the body one equal square per position? | yes: two 8.8 × 8.8 mm squares |
| Is every closed port of every position marked? | yes: `3` at rest, `5` when actuated, inside their own squares |
| Is every port the standard draws drawn? | yes: `1`, `2`, `3`, `4`, `5` |
| Do the numerals match the declared ports (1 inlet, 2/4 outlets, 3/5 exhausts)? | yes: `1` inlet, `4`/`2` outlets, `5`/`3` exhausts |
| If it is a machine, is the drive bar on the standard's side? | not a machine |
| If it is adjustable, is the arrow on the diameter or through the element, and filled? | not adjustable |
| Is every numeral clear of the envelope/frame, the body and the primitive lines? | yes: the numerals sit in the band outside the body, beside their stubs |
| Does the arrow point at the far end of the connection? | yes: each supply path at its working port, each exhaust path at its exhaust |
| Are spring and actuator on the declared sides? | yes: lever at the left, spring at the right, opposite ends |
| Is the symbol legible at palette size? | yes: the light and dark captures |
| Is the symbol legible at 300 dpi? | yes: the 300 dpi sheet; the second reader's signature is owed |

## Sign-off

- **Status**: `pending-300dpi`
- **Reviewed by**: pending (a person, not the assistant)
- **Date**: 2026-11-24
- **What changed since the last record**: the port sides were inverted; they now follow
  `X10400` (`4`/`2` above, `5`/`1`/`3` below), and the file's header comment now states
  the same rest direction as the behaviour (`1`→`2` at rest). The lever's pivot moved to
  the body end with the grip at the far end. The record now cites `X10400` and `X10460`
  and states that no single X-number covers this combination.

## What is still open

- The second reader's signature; until then the record stays `pending-300dpi` and the
  export keeps the unverified note.
- No registered X-number combines the 5/2 body with a plain lever and a mechanical
  spring only: `X10400` is treadle-controlled, `X10460` has the air spring. The review
  cites both and records the substitution instead of inventing a figure.
- The physical conductances are a datasheet-order value for a small valve, not a
  measurement.

## Images

- `valve.5_2.spring_return-light.png`, `valve.5_2.spring_return-dark.png`
- `valve.5_2.spring_return-300.png`
