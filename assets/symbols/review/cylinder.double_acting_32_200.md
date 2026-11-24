<!-- SPDX-FileCopyrightText: 2026 PunchiSoft -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Review record: `cylinder.double_acting_32_200`

    status: pending-300dpi
    date: 2026-11-24
    reviewer: automated pass, unreviewed by a second person
    function: double-acting cylinder, 32 mm bore, 200 mm stroke, one working port per end

## The claim

- **Function, in one sentence**: the air entering the cap end extends the piston and the
  air entering the rod end retracts it, so the cylinder is driven from both ends and needs
  no return spring.
- **Family**: actuator (linear, double-acting).
- **Normative source**: ISO 1219-1:2006, **7.3.2**, figure **`X11450`**, read in the
  owner's copy (the pneumatic examples; the same symbol is **6.3.2** in the hydraulic
  examples, same registration number) and confirmed on the 2012 basis in
  JIS B 0125-1:2020, **7.3.2**, same figure and registration. The figure is registered as a
  single-rod, double-acting cylinder. Rules: `RF052` and `RF053`; the working ports are `1`
  (cap end, extends) and `2` (rod end, retracts).
- **Ports**: `1` and `2`, both working, one in each end cap, both leaving their cap
  downwards.

## Reference and assumptions

- **Reference read**: the standard's figures only — `X11450` in ISO 1219-1:2006 and in
  JIS B 0125-1:2020. No capture of another application was used and no figure or text is
  stored in the repository.
- **Observed**: `X11450` draws the same barrel and cap end as `X11440`, the piston as a
  full-height line and the rod as an outlined bar through the rod-side wall, with **no
  spring**. One stub leaves each end cap downwards; the two are at the same inset from
  their end walls and the same depth, and no third connection is printed.
- **Required by the owner**: two working ports, one per end cap, at the same depth, and no
  return spring; 32 mm bore, 200 mm stroke and a 20 mm rod.
- **Stated by the standard**: the two connections are the inlet and the outlet of the two
  chambers; the omission of a spring is what makes it double-acting; the figure prints no
  cushioning adjustment and no piston magnet.
- **Assumption**: the figure's STATIC piston stands about a third of the way into the
  barrel. Our piston follows the model, so it rests against the cap's inner face; the
  two ports stay in the caps, outside the chamber it crosses.
- **Assumption**: the figure's cap-plate line is not drawn as a separate primitive. The cap
  is the renderer's 11 % end inset; both ports are inside their caps. The review records
  the missing line with the other open items.

## Measures

The symbol box is **44 × 20 mm**; the normalised coordinates map to the box's millimetres on
each axis. The figure's barrel is 2.23 : 1; the drawing's is 2.2 : 1.

| Feature | Requirement | Drawing |
|---|---|---|
| Barrel | the widest rectangle the file declares, the figure's 44 × 20 mm box, 2.23 : 1 | x 3.52–22.00, y 5.80–14.20 mm; 18.48 × 8.40 mm, 2.2 : 1 |
| End caps | a tenth of the barrel's length at each end, a port inside each | 2.03 mm deep: the chamber runs x 5.55–19.97 mm |
| Port `1` | in the cap end, downwards, clear of the chamber | (4.44, 20.00) mm; 0.92 mm from its wall, 5.80 mm of stub |
| Port `2` | in the rod end, downwards, at the same depth as `1` | (21.08, 20.00) mm; 0.92 mm from its wall, 5.80 mm of stub |
| Port depth | the two leave their caps at the same depth, and neither is over the rod or the outline | both stubs run vertically from the barrel's bottom edge at y 14.20 mm to y 20.00 mm, 5.80 mm below it, each 0.92 mm inside its end wall |
| Spring | none on a double-acting cylinder | none declared; `spring_rate` and `spring_preload` are zero and there is no `[spring]` |
| Piston | a full-height line, crossing the chamber | 5.55 mm at rest, 19.41 mm at full stroke; 13.86 mm of travel = 96.2 % of the 14.41 mm chamber; meets y 5.80 and 14.20 mm |
| Dead length | the stroke-plus-dead length, only the declared dead length left at full stroke | 0.55 mm between the piston and the rod-side cap face at full stroke (the file declares 8 of 208 mm) |
| Rod | one constant length, visible at rest, tip at the box edge at full stroke | 24.59 mm long; 8.14 mm outside the barrel at rest; tip at x 44.00 mm at full stroke |
| Numerals | beside each port's own stub, on the side with no other port | `1` at (-0.04, 16.90) mm and `2` at (25.56, 16.90) mm, 3.0 mm cap: outside the barrel, each on its own outer side |
| Cushioning | none printed on `X11450` (it is the mark of `X11460` and its `101V19`, 8.4.35) | none drawn, none declared |
| Piston magnet | none printed on `X11450` (the magnet symbols are the grippers `X11630`–`X11660` and the rodless `X11540`) | none drawn, none declared |

## Evidence

- 300 dpi sheet, print ink:
  `target/debug/symbol-sheet cylinder.double_acting_32_200 assets/symbols/review/cylinder.double_acting_32_200-300.png`
- Light and dark captures, fitted to the drawing:
  `target/debug/symbol-sheet cylinder.double_acting_32_200 assets/symbols/review/cylinder.double_acting_32_200-light.png --ink light --fit`
  `target/debug/symbol-sheet cylinder.double_acting_32_200 assets/symbols/review/cylinder.double_acting_32_200-dark.png --ink dark --fit`
  (the tool is rebuilt first with `cargo build -p punchi-fluid-export --bin symbol-sheet`,
  because the catalog is compiled in).
- Symbol lint: passes (geometry inside the box, every line classed, every port used and
  numbered; no spring on a double-acting cylinder).
- Geometry fixture regenerated and read: `assets/symbols/fixtures/cylinder.double_acting_32_200.txt`.
  The diff moves the barrel to y 11.6–28.4 mm of the 40 mm fixture box, both ports into their
  caps, and the piston to the cap's inner face; the rod follows.
- Drawing tests: `the_piston_never_leaves_the_barrel_at_either_end_of_the_stroke`,
  `the_piston_meets_both_walls_of_the_bore`, `each_port_of_a_cylinder_is_drilled_in_an_end_cap`,
  `the_piston_crosses_the_whole_stroke_the_file_declares`,
  `the_rod_keeps_its_length_at_every_stroke`, `only_the_single_acting_cylinder_draws_a_spring`.
- New test: `every_cylinder_port_is_drilled_in_a_cap_and_clear_of_the_chamber` — each port
  lies between its end wall and its cap's inner face, the two at the same depth, one per end.
- Model: `a_worked_circuit_builds_and_advances` builds the double-acting cylinder with a
  5/2 valve and two working lines and advances it; the solver turns its two working ports
  into the extend chamber and the retract annulus (`punchi-fluid-solver/src/network.rs`).
  The fixed circuit still draws the single-acting cylinder.

## Checklist

| Question | Answer |
|---|---|
| Is the body one equal square per position? | not a valve: the body is the barrel, 18.48 × 8.40 mm |
| Is every closed port of every position marked? | not applicable: no position and no closed port |
| Is every port the standard draws drawn? | yes: `1` in the cap end and `2` in the rod end, both downwards |
| Do the numerals match the declared ports (1 inlet, 2/4 outlets, 3/5 exhausts)? | yes: `1` and `2`, the two working ports |
| If it is a machine, is the drive bar on the standard's side? | not a machine |
| If it is adjustable, is the arrow on the diameter or through the element, and filled? | not adjustable |
| Is every numeral clear of the envelope/frame, the body and the primitive lines? | yes: each sits below the barrel, beside its own stub, on its own outer side |
| Does the arrow point at the far end of the connection? | not applicable: no flow path |
| Are spring and actuator on the declared sides? | no spring and no actuator: that is the double-acting form |
| Is the symbol legible at palette size? | yes: the light and dark captures |
| Is the symbol legible at 300 dpi? | yes: the 300 dpi sheet; the second reader's signature is owed |

## Sign-off

- **Status**: `pending-300dpi`
- **Reviewed by**: pending (a person, not the assistant)
- **Date**: 2026-11-24
- **What changed since the last record**: both working ports moved from 3.52 mm to 0.92 mm
  from their end walls and the cap's inner face from 4 % to 11 % of the barrel's length, so
  each is drilled in its cap and outside the chamber the piston crosses; the barrel's height
  follows the figure (8.40 mm, 2.2 : 1) and the piston rests against the cap's inner face.
  The earlier record's "not simulated yet" no longer holds: the session model drives this
  cylinder with two working lines and the solver builds both chambers.

## What is still open

- The second reader's signature; until then the record stays `pending-300dpi` and the
  export keeps the unverified note.
- The figure's **cap-plate line** at the cap end is not drawn; the barrel is one rectangle
  and the cap is the renderer's inset. The ports are inside their caps, which is what the
  function needs, but the line the figure prints is missing.
- The figure's **rod** is an outlined bar; the drawing keeps the rod as one thick line, as
  every other moving part is drawn. Its constant length and its box-edge tip are tested;
  its outline is not drawn.
- The **cushioning** and the **piston magnet** are not marks of `X11450`: the adjustable
  end cushioning is `X11460`/`101V19` and the magnet symbols are different X-numbers. If a
  cylinder with either is added, it is a new file and a schema field, not a variant of
  this one.
- The review tool draws **every** symbol in a fixed 40 × 40 mm box, so the captures show
  these normalised coordinates in a square; the sheet's cylinder box is 44 × 20 mm. The
  measures above are the sheet's, and the numerals sit farther from their stubs in the
  captures than on the sheet because the tool scales the offset with the box's height.

## Images

- `cylinder.double_acting_32_200-300.png`
- `cylinder.double_acting_32_200-light.png`, `cylinder.double_acting_32_200-dark.png`
