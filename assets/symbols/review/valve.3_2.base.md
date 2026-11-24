<!-- SPDX-FileCopyrightText: 2026 PunchiSoft -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Review record: `valve.3_2.base`

    status: pending-300dpi
    date: 2026-11-24
    reviewer: automated pass, unreviewed by a second person
    function: 3/2 directional control valve in its base form — two positions, three ports, no return and no actuation

## The claim

- **Function, in one sentence**: at its default position the working line is vented and
  the supply is shut; at the other position the supply feeds the working line and the
  exhaust is shut.
- **Family**: valve (directional control), base form.
- **Normative source**: ISO 1219-1:2006, 6.1.2.6, figure `X10280` (registration
  `7.1.2.8`), read in the owner's copy; confirmed on the 2012 basis in JIS B
  0125-1:2020, 6.1.2.6 / `7.1.2.8`, same figure and registration. **This file is a
  composition**: it is the 3/2 body, ports and both positions of `X10280` with the
  solenoid and the spring removed, so it has no single X-number of its own; the figure
  cited is the complete valve it is composed from. Rules: `RF052` (every port is drawn)
  and `RF053` (`1` inlet, `2`/`4` outlets, `3`/`5` exhausts).
- **Ports**: `1` supply · `2` working · `3` exhaust — all three drawn once, at the
  default square.

## Reference and assumptions

- **Reference read**: the standard's figure only. No capture of another application was
  used for the shape, and no figure or text is stored in the repository.
- **Observed**: `X10280` draws two equal squares side by side, the spring against the
  right square and the solenoid at the left. The working port `2` is on the top edge,
  aligned with the supply `1` on the bottom edge; the exhaust `3` is on the bottom edge at
  the other quarter. The square against the spring connects `2` to `3` and blocks `1`;
  the other square connects `1` to `2` and blocks `3`.
- **Required by the owner**: the base is the body a student places and then configures in
  the valve dialog; the return and the actuation are fitted by the variant files. The
  drawing must be the standard's body, not a simplified valve.
- **Stated by the standard**: the two positions, the spring side and the flow paths above
  are the figure's; `RF053` fixes the port designations.
- **Assumption**: without a return there is no spring square, so the file's
  `default_position = 0` is the rest square and the ports are read at the left square.
  The designations `1`, `2`, `3` are `RF053`'s; the figure itself prints no numerals, so
  the numerals are our placement of the standard's names.

## Measures

The symbol box is 20 mm; the normalised coordinates map to millimetres on both axes.

| Feature | Requirement | Drawing |
|---|---|---|
| Body | one equal square per position, shared edge | body x 1.2 mm, y 5.6 mm, 17.6 × 8.8 mm; two squares 8.8 × 8.8 mm |
| Ports | `1` supply, `2` working, `3` exhaust, on the figure's sides | `1` (3.4, 20.0), `2` (3.4, 0.0), `3` (7.8, 20.0) mm; 2.2 mm and 6.6 mm from the square's left edge |
| Stubs | straight, from the body edge to the box edge | `1` and `3` 5.6 mm below the body (y 14.4→20.0); `2` 5.6 mm above (y 5.6→0.0) |
| Default position | the square the ports are read with | position 0, left square, drawn active |
| Flow paths | straight inside their own square, arrow at the far end | position 0: `2`→`3` diagonal, arrow at `3`; position 1: `1`→`2` vertical, arrow at `2` |
| Blocking marks | inside the square that shuts the port | `1` in position 0; `3` in position 1; stem 1.76 mm, bar 2.64 mm |
| Return and actuator | absent in the base form | none declared |
| Numerals | outside the body, beside the stub, once | 3.0 mm cap; `1` below left, `2` above, `3` below right |

## Evidence

- 300 dpi sheet, print ink:
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- valve.3_2.base assets/symbols/review/valve.3_2.base-300.png`
- Light and dark captures, fitted to the drawing:
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- valve.3_2.base assets/symbols/review/valve.3_2.base-light.png --ink light --fit`
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- valve.3_2.base assets/symbols/review/valve.3_2.base-dark.png --ink dark --fit`
- Symbol lint: passes (the built-in passes `every_builtin_definition_passes_the_lint`).
- Geometry fixture regenerated and read: `assets/symbols/fixtures/valve.3_2.base.txt`.
  The port order moved to the figure's: `1` and `2` now share their offset, `3` sits at
  the other quarter; both paths and both blocking marks follow.

## Checklist

| Question | Answer |
|---|---|
| Is the body one equal square per position? | yes: two 8.8 × 8.8 mm squares |
| Is every closed port of every position marked? | yes: `1` in position 0, `3` in position 1, inside their own squares |
| Is every port the standard draws drawn? | yes: `1`, `2`, `3` |
| Do the numerals match the declared ports (1 inlet, 2/4 outlets, 3/5 exhausts)? | yes: `1`, `2`, `3` |
| If it is a machine, is the drive bar on the standard's side? | not a machine |
| If it is adjustable, is the arrow on the diameter or through the element, and filled? | not adjustable |
| Is every numeral clear of the envelope/frame, the body and the primitive lines? | yes: the numerals sit in the band outside the body, beside their stubs |
| Does the arrow point at the far end of the connection? | yes: at `3` in position 0, at `2` in position 1 |
| Are spring and actuator on the declared sides? | there is no spring and no actuator in the base form |
| Is the symbol legible at palette size? | yes: the light and dark captures |
| Is the symbol legible at 300 dpi? | yes: the 300 dpi sheet; the second reader's signature is owed |

## Sign-off

- **Status**: `pending-300dpi`
- **Reviewed by**: pending (a person, not the assistant)
- **Date**: 2026-11-24
- **What changed since the last record**: the ports moved to the figure's offsets (`1`
  and `2` aligned, `3` at the other quarter), so the two paths and the two blocking marks
  were redrawn; the record now cites `X10280` and says the base is a composition.

## What is still open

- The second reader's signature; until then the record stays `pending-300dpi` and the
  export keeps the unverified note.
- The base is a composition and has no single registered X-number. That is the catalog's
  existing organisation; if the base must itself carry a figure, the catalog would need a
  different split, which is outside this phase.
- ISO 11727 is the reference for the `4`/`2` and `5`/`3` assignment; it was not read in
  this pass, so the numerals follow `RF053` and the catalog convention.

## Images

- `valve.3_2.base-light.png`, `valve.3_2.base-dark.png`
- `valve.3_2.base-300.png`
