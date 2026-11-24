<!-- SPDX-FileCopyrightText: 2026 PunchiSoft -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Review record: `valve.5_2.base`

    status: complete
    date: 2026-11-24
    reviewer: Cristian Muñoz (owner), second reader
    function: 5/2 directional control valve in its base form — two positions, five ports, no return and no actuation

## The claim

- **Function, in one sentence**: at the default position the supply feeds one working
  line while the other vents; at the other position the supply feeds the second working
  line and the first vents, and each position shuts the exhaust it does not use.
- **Family**: valve (directional control), base form.
- **Normative source**: ISO 1219-1:2006, 6.1.2.15, figure `X10400` (registration
  `7.1.2.16`), read in the owner's copy; confirmed on the 2012 basis in JIS B
  0125-1:2020, 6.1.2.15 / `7.1.2.16`, same figure and registration. **This file is a
  composition**: it is the 5/2 body, ports and both positions of `X10400` with the
  treadle removed, so it has no single X-number of its own; the figure cited is the
  complete valve it is composed from. Rules: `RF052` and `RF053`.
- **Ports**: `1` supply · `2`, `4` working · `3`, `5` exhaust — all five drawn once, at
  the default square.

## Reference and assumptions

- **Reference read**: the standard's figures only. No capture of another application was
  used for the shape, and no figure or text is stored in the repository.
- **Observed**: `X10400` draws two equal squares side by side. The working ports are the
  pair above: `4` over the left quarter and `2` over the right; the supply `1` is in the
  middle of the bottom edge, with `5` below the left quarter and `3` below the right. In
  the left square the supply goes to `2`, `4` vents through `5`, and `3` is shut; in the
  right square the supply goes to `4`, `2` vents through `3`, and `5` is shut.
- **Required by the owner**: the base is the body a student places and then configures in
  the valve dialog; the return and the actuation are fitted by the variant files.
- **Stated by the standard**: the five ports, their sides and the two positions above;
  `RF053` fixes the designations.
- **Assumption**: without a return the file's `default_position = 0` is the rest square
  and the ports are read at the left square. The assignment of `4`/`2` and `5`/`3` to the
  left and right pairs follows ISO 11727 / `RF053`; the figures do not print the
  numerals.

## Measures

The symbol box is 20 mm; the normalised coordinates map to millimetres on both axes.

| Feature | Requirement | Drawing |
|---|---|---|
| Body | one equal square per position, shared edge | x 1.2 mm, y 5.6 mm, 17.6 × 8.8 mm; two squares 8.8 × 8.8 mm |
| Ports | `4`/`2` working above, `5`/`1`/`3` below, in the figure's order | `4` (2.7, 0.0), `2` (8.5, 0.0), `1` (5.6, 20.0), `5` (2.7, 20.0), `3` (8.5, 20.0) mm; offsets 1.47 / 4.40 / 7.33 mm inside the square |
| Stubs | straight, from the body edge to the box edge | 5.6 mm above or below the body, per side |
| Default position | the square the ports are read with | position 0, left square, drawn active |
| Flow paths | straight inside their own square, arrow at the far end | position 0: `1`→`2` diagonal (8.28 mm) and `4`→`5` vertical (7.74 mm); position 1: `1`→`4` diagonal and `2`→`3` vertical |
| Blocking marks | inside the square that shuts the port | `3` in position 0; `5` in position 1; stem 1.76 mm, bar 2.64 mm |
| Return and actuator | absent in the base form | none declared |
| Numerals | outside the body, beside the stub, once | 3.0 mm cap; `4`/`2` above, `5`/`1`/`3` below |

## Evidence

- 300 dpi sheet, print ink:
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- valve.5_2.base assets/symbols/review/valve.5_2.base-300.png`
- Light and dark captures, fitted to the drawing:
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- valve.5_2.base assets/symbols/review/valve.5_2.base-light.png --ink light --fit`
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- valve.5_2.base assets/symbols/review/valve.5_2.base-dark.png --ink dark --fit`
- Symbol lint: passes.
- Geometry fixture regenerated and read: `assets/symbols/fixtures/valve.5_2.base.txt`.
  The working ports moved from the bottom edge to the top and the supply/exhausts from the
  top to the bottom, matching `X10400`; the offsets are the figure's quarters and middle.

## Checklist

| Question | Answer |
|---|---|
| Is the body one equal square per position? | yes: two 8.8 × 8.8 mm squares |
| Is every closed port of every position marked? | yes: `3` in position 0, `5` in position 1, inside their own squares |
| Is every port the standard draws drawn? | yes: `1`, `2`, `3`, `4`, `5` |
| Do the numerals match the declared ports (1 inlet, 2/4 outlets, 3/5 exhausts)? | yes: `1` inlet, `4`/`2` outlets, `5`/`3` exhausts |
| If it is a machine, is the drive bar on the standard's side? | not a machine |
| If it is adjustable, is the arrow on the diameter or through the element, and filled? | not adjustable |
| Is every numeral clear of the envelope/frame, the body and the primitive lines? | yes: the numerals sit in the band outside the body, beside their stubs |
| Does the arrow point at the far end of the connection? | yes: the supply path at its working port, the exhaust path at its exhaust |
| Are spring and actuator on the declared sides? | there is no spring and no actuator in the base form |
| Is the symbol legible at palette size? | yes: the light and dark captures |
| Is the symbol legible at 300 dpi? | yes: the 300 dpi sheet |

## Sign-off

- **Status**: `complete`
- **Reviewed by**: Cristian Muñoz (owner)
- **Date**: 2026-11-24
- **What changed since the last record**: the port sides were inverted; they now follow
  `X10400` (`4`/`2` above, `5`/`1`/`3` below), the offsets are the figure's, and the two
  positions' paths and blocking marks follow. The record now cites `X10400` and says the
  base is a composition.

## What is still open

- The base is a composition and has no single registered X-number; that is the catalog's
  existing organisation, not a change made here.
- ISO 11727 is the reference for the `4`/`2` and `5`/`3` assignment; it was not read in
  this pass, so the numerals follow `RF053` and the catalog convention.

## Images

- `valve.5_2.base-light.png`, `valve.5_2.base-dark.png`
- `valve.5_2.base-300.png`
