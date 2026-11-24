<!-- SPDX-FileCopyrightText: 2026 PunchiSoft -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Review record: `valve.3_2.solenoid`

    status: pending-300dpi
    date: 2026-11-24
    reviewer: automated pass, unreviewed by a second person
    function: 3/2 directional control valve, single solenoid, spring return, normally closed

## The claim

- **Function, in one sentence**: while the coil is energised the valve feeds the working
  line and shuts the exhaust; when the coil is released the spring returns the spool to
  the rest position, which vents the working line and shuts the supply.
- **Family**: valve (directional control), complete valve with a return and an electrical
  actuation.
- **Normative source**: ISO 1219-1:2006, 6.1.2.6, figure `X10280` (registration
  `7.1.2.8`), read in the owner's copy; confirmed on the 2012 basis in JIS B
  0125-1:2020, 6.1.2.6 / `7.1.2.8`, same figure and registration. `X10280` is the
  registered **complete valve**: a 3/2 with three ports and two positions, normally
  closed, controlled by a solenoid and spring return. The coil is the winding module
  `212V1` (8.5.29). `X10290` (6.1.2.7) is the neighbouring variant with a manual override
  with detent; this file does not draw that override and does not cite it. Rules:
  `RF052`, `RF053`.
- **Ports**: `1` supply · `2` working · `3` exhaust · `Y1` coil terminal (electrical, no
  printed numeral). All drawn; `Y1` is the control-circuit connection, not a way of the
  valve.

## Reference and assumptions

- **Reference read**: the standard's figures only. No capture of another application was
  used for the shape, and no figure or text is stored in the repository.
- **Observed**: `X10280` draws the two equal squares, the spring against the right square
  and the solenoid at the left as a rectangle with a diagonal. The working port `2` is on
  the top edge, aligned with the supply `1` on the bottom edge; the exhaust `3` is on the
  bottom edge at the other quarter. The rest square connects `2` to `3` and blocks `1`;
  the actuated square connects `1` to `2` and blocks `3`.
- **Required by the owner**: the variant files are the complete valves with the same body
  as the base; this one fits the spring return and the solenoid, and the control circuit
  arrives at its coil terminal (ADR 0018).
- **Stated by the standard**: the positions, the spring side and the flow paths above;
  `212V1` draws one winding as a box with a diagonal.
- **Assumption**: ISO 1219-1 does not draw an electrical terminal on the valve symbol, so
  `Y1` is the project's connection point at the symbol box's left edge (x = 0, the
  smallest position the schema allows), at the actuator end. It prints no numeral because
  its label is the user's connection name, not a port designation. The port numerals
  follow `RF053` / ISO 11727.

## Measures

The symbol box is 20 mm; the normalised coordinates map to millimetres on both axes.

| Feature | Requirement | Drawing |
|---|---|---|
| Body | one equal square per position, shared edge, centred in the box | x 1.2 mm, y 5.6 mm, 17.6 × 8.8 mm; two squares 8.8 × 8.8 mm (the body's y moved from 0.24 to 0.28 to match the other 3/2 files) |
| Ports | `1` supply, `2` working, `3` exhaust, on the figure's sides | `1` (12.2, 20.0), `2` (12.2, 0.0), `3` (16.6, 20.0) mm; 2.2 mm and 6.6 mm inside the rest square |
| Rest position | the square against the spring | position 0, right square, drawn active |
| Flow paths | straight inside their own square, arrow at the far end | rest `2`→`3` diagonal, arrow at `3`; actuated `1`→`2` vertical, arrow at `2` |
| Blocking marks | inside the square that shuts the port | `1` at rest; `3` when actuated; stem 1.76 mm, bar 2.64 mm |
| Spring | right end, outside the body, zigzag of proportional size | starts 1.14 mm clear of the face; coil 3.43 mm long; full amplitude 3.52 mm |
| Solenoid | left end, one winding box with its diagonal, opposite the spring | stem to 0.92 mm; box 0.92 → 3.89 mm from the face; half-height 1.37 mm |
| Coil terminal | at the actuator end, where the control circuit arrives | `Y1` at (0.0, 9.2) mm, no numeral |
| Numerals | outside the body, beside the stub, once | 3.0 mm cap; `1` and `3` below, `2` above |

## Evidence

- 300 dpi sheet, print ink:
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- valve.3_2.solenoid assets/symbols/review/valve.3_2.solenoid-300.png`
- Light and dark captures, fitted to the drawing:
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- valve.3_2.solenoid assets/symbols/review/valve.3_2.solenoid-light.png --ink light --fit`
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- valve.3_2.solenoid assets/symbols/review/valve.3_2.solenoid-dark.png --ink dark --fit`
- Symbol lint: passes, including the electrical port exemption.
- Geometry fixture regenerated and read: `assets/symbols/fixtures/valve.3_2.solenoid.txt`.
  The body is now vertically centred like the other 3/2 files; the ports moved to the
  figure's offsets; the two blocking marks are new, so each square states its own shut
  port instead of leaving it blank.

## Checklist

| Question | Answer |
|---|---|
| Is the body one equal square per position? | yes: two 8.8 × 8.8 mm squares, centred |
| Is every closed port of every position marked? | yes: `1` at rest, `3` when actuated, inside their own squares |
| Is every port the standard draws drawn? | yes: `1`, `2`, `3`; plus the `Y1` coil terminal of the electrical domain |
| Do the numerals match the declared ports (1 inlet, 2/4 outlets, 3/5 exhausts)? | yes: `1`, `2`, `3`; `Y1` prints none by design |
| If it is a machine, is the drive bar on the standard's side? | not a machine |
| If it is adjustable, is the arrow on the diameter or through the element, and filled? | not adjustable |
| Is every numeral clear of the envelope/frame, the body and the primitive lines? | yes: the numerals sit in the band outside the body, beside their stubs |
| Does the arrow point at the far end of the connection? | yes: at `3` at rest, at `2` when actuated |
| Are spring and actuator on the declared sides? | yes: solenoid at the left, spring at the right |
| Is the symbol legible at palette size? | yes: the light and dark captures |
| Is the symbol legible at 300 dpi? | yes: the 300 dpi sheet; the second reader's signature is owed |

## Sign-off

- **Status**: `pending-300dpi`
- **Reviewed by**: pending (a person, not the assistant)
- **Date**: 2026-11-24
- **What changed since the last record**: the body's y moved from 0.24 to 0.28 so it is
  centred like the other 3/2 files; the two positions now carry their blocking marks
  (`1` at rest, `3` when actuated); the ports moved to the figure's offsets; the record
  cites `X10280`, the complete valve this file represents, instead of `X10290`.

## What is still open

- The second reader's signature; until then the record stays `pending-300dpi` and the
  export keeps the unverified note.
- The coil terminal is a project connection point: ISO 1219-1 does not define it on the
  valve symbol, and the schema requires it inside the 0..1 box, so it sits at the left
  edge rather than at the coil's outer end.
- The physical conductances are a datasheet-order value for a small valve, not a
  measurement.

## Images

- `valve.3_2.solenoid-light.png`, `valve.3_2.solenoid-dark.png`
- `valve.3_2.solenoid-300.png`
