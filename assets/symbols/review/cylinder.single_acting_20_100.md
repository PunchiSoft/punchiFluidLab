<!-- SPDX-FileCopyrightText: 2026 PunchiSoft -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Review record: `cylinder.single_acting_20_100`

    status: complete
    date: 2026-11-24
    reviewer: Cristian Muñoz (owner), second reader
    function: single-acting cylinder with a return spring, 20 mm bore, 100 mm stroke

## The claim

- **Function, in one sentence**: the air entering the cap end pushes the piston out and
  compresses the spring in the rod-side chamber; when the pressure falls, the spring
  returns the piston, and the rod end carries no working line.
- **Family**: actuator (linear, single-acting, spring return).
- **Normative source**: ISO 1219-1:2006, **7.3.1**, figure **`X11440`**, read in the
  owner's copy (the pneumatic examples; the same symbol is **6.3.1** in the hydraulic
  examples, same registration number) and confirmed on the 2012 basis in
  JIS B 0125-1:2020, **7.3.1**, same figure and registration. The figure is registered as a
  single-rod, single-acting cylinder whose return stroke is by spring force, with the spring
  chamber carrying its own connection. Rules: `RF052` (every port of a symbol is drawn) and
  `RF053` (the numerals; this file declares the working port as `1`).
- **Ports**: `1` working, in the cap end, drawn downwards from the barrel's bottom edge.
  The figure also prints the **spring-chamber connection** at the rod end; this file
  declares the working port only (see *What is still open*, and the `RF052` omission).

## Reference and assumptions

- **Reference read**: the standard's figures only — `X11440` in ISO 1219-1:2006 and in
  JIS B 0125-1:2020. No capture of another application was used for the shape, and no
  figure or text is stored in the repository.
- **Observed**: `X11440` draws the barrel as a long rectangle with the **end cap** as a
  second vertical line inset about a tenth of the barrel's length from the cap-end wall;
  the piston is a full-height line; the **spring** sits in the rod-side chamber, between
  the piston and the rod-side wall, and its coils reach both walls; the **rod** leaves
  through the rod-side wall as an outlined bar. The working port leaves the cap end
  downwards, between the cap-end wall and the cap's inner face; a second stub leaves the
  rod-side cap the same way. The coil is drawn across the bore and the port stubs are about
  half the bore deep.
- **Required by the owner**: a single-acting cylinder with the working port in the cap end,
  the return spring at the rod side, no second working line, 20 mm bore and 100 mm stroke.
- **Stated by the standard**: the cap end carries the working connection; the spring is the
  return element of the rod-side chamber and fills it; the figure registers the spring chamber as
  carrying a connection, so that connection is part of the printed symbol.
- **Assumption**: the spring-chamber connection is **not declared**. The model gives a
  single-acting cylinder one working chamber and the schema has no breather role; declaring
  it as `working` would make the solver build a retract chamber and declaring it `exhaust`
  would change the port table of the protected part. The review records the omission
  instead of inventing a role. The figure's STATIC piston stands about a third of the way
  into the barrel; our piston follows the model, so it rests against the cap's inner face
  and the spring spans the whole rod-side chamber at rest.
- **Assumption**: the figure's cap-plate line is not drawn as a separate primitive. The
  cap is the renderer's 11 % end inset; the port is inside it. The review records the
  missing line with the other open items.

## Measures

The symbol box is **44 × 20 mm**; the normalised coordinates map to the box's millimetres on
each axis. The figure's barrel is 2.23 : 1; the drawing's is 2.2 : 1.

| Feature | Requirement | Drawing |
|---|---|---|
| Barrel | the widest rectangle the file declares, the figure's 44 × 20 mm box, 2.23 : 1 | x 3.52–22.00, y 5.80–14.20 mm; 18.48 × 8.40 mm, 2.2 : 1 |
| End cap | a tenth of the barrel's length at each end, the port inside it | 2.03 mm deep: the chamber runs x 5.55–19.97 mm |
| Port `1` | in the cap end, downwards, clear of the chamber the piston crosses | (4.44, 20.00) mm; 0.92 mm from the cap-end wall, 5.80 mm of stub; the figure's 0.05 of the barrel's width |
| Second port | none at the rod end (a working port there would make it double-acting) | none declared; the figure's spring-chamber connection is the recorded omission |
| Piston | a full-height line, crossing the chamber | 5.55 mm at rest, 19.28 mm at full stroke; 13.73 mm of travel = 95.2 % of the 14.41 mm chamber; meets y 5.80 and 14.20 mm |
| Dead length | the stroke-plus-dead length, only the declared dead length left at full stroke | 0.69 mm between the piston and the rod-side cap face at full stroke (the file declares 5 of 105 mm) |
| Rod | one constant length, visible at rest, tip at the box edge at full stroke | 24.72 mm long; 8.27 mm outside the barrel at rest; tip at x 44.00 mm at full stroke |
| Spring | the return element, spanning the rod-side chamber, thinning with the stroke, floor two turns | from the piston to x 19.97 mm; 5 turns at rest, 2 at full stroke; amplitude ±2.18 mm at rest, the figure's coil filling the bore |
| Numerals | beside the port's own stub, in the band below the barrel | `1` at (8.92, 16.90) mm, 3.0 mm cap; the only port, so either side is clear |
| Cushioning | none printed on `X11440` (it is the mark of `X11460` and its `101V19`, 8.4.35) | none drawn, none declared |
| Piston magnet | none printed on `X11440` (the magnet symbols are the grippers `X11630`–`X11660` and the rodless `X11540`) | none drawn, none declared |

## Evidence

- 300 dpi sheet, print ink:
  `target/debug/symbol-sheet cylinder.single_acting_20_100 assets/symbols/review/cylinder.single_acting_20_100-300.png`
- Light and dark captures, fitted to the drawing:
  `target/debug/symbol-sheet cylinder.single_acting_20_100 assets/symbols/review/cylinder.single_acting_20_100-light.png --ink light --fit`
  `target/debug/symbol-sheet cylinder.single_acting_20_100 assets/symbols/review/cylinder.single_acting_20_100-dark.png --ink dark --fit`
  (the tool is rebuilt first with `cargo build -p punchi-fluid-export --bin symbol-sheet`,
  because the catalog is compiled in).
- Symbol lint: passes (geometry inside the box, every line classed, the spring declared on a
  moving part, every port used and numbered).
- Geometry fixture regenerated and read: `assets/symbols/fixtures/cylinder.single_acting_20_100.txt`.
  The diff moves the barrel to y 11.6–28.4 mm of the 40 mm fixture box, the port into the
  cap, and the piston to the cap's inner face; the spring and the rod follow.
- Drawing tests: `the_piston_never_leaves_the_barrel_at_either_end_of_the_stroke`,
  `the_piston_meets_both_walls_of_the_bore`, `each_port_of_a_cylinder_is_drilled_in_an_end_cap`,
  `the_piston_crosses_the_whole_stroke_the_file_declares`,
  `the_return_spring_compresses_with_the_stroke`, `the_rod_keeps_its_length_at_every_stroke`,
  `only_the_single_acting_cylinder_draws_a_spring`.
- New test: `every_cylinder_port_is_drilled_in_a_cap_and_clear_of_the_chamber` — the port
  lies between the end wall and the cap's inner face, so the piston never sweeps it.
- Model: the built-in definition is the fixed circuit's cylinder; the stroke and dead length
  in the measures are the file's, not a copy in the test.

## Checklist

| Question | Answer |
|---|---|
| Is the body one equal square per position? | not a valve: the body is the barrel, 18.48 × 8.40 mm |
| Is every closed port of every position marked? | not applicable: no position and no closed port |
| Is every port the standard draws drawn? | the working port yes; the figure's spring-chamber connection is the recorded `RF052` omission |
| Do the numerals match the declared ports (1 inlet, 2/4 outlets, 3/5 exhausts)? | yes: `1`, the working port |
| If it is a machine, is the drive bar on the standard's side? | not a machine |
| If it is adjustable, is the arrow on the diameter or through the element, and filled? | not adjustable |
| Is every numeral clear of the envelope/frame, the body and the primitive lines? | yes: `1` sits in the band below the barrel, beside its stub |
| Does the arrow point at the far end of the connection? | not applicable: no flow path |
| Are spring and actuator on the declared sides? | yes: the spring is at the rod side (`[spring] side = "right"`) and there is no actuator |
| Is the symbol legible at palette size? | yes: the light and dark captures |
| Is the symbol legible at 300 dpi? | yes: the 300 dpi sheet |

## Sign-off

- **Status**: `complete`
- **Reviewed by**: Cristian Muñoz (owner)
- **Date**: 2026-11-24
- **What changed since the last record**: the working port moved from x 7.04 mm to
  4.44 mm, into the end cap, and the cap's inner face from 4 % to 11 % of the barrel's
  length, so the port is no longer inside the chamber the piston crosses. The barrel's
  height follows the figure (8.40 mm, 2.2 : 1), and the piston now rests against the cap's
  inner face. The earlier record's claim that the rod end carries no port at all is
  narrowed to no **working** port: the figure prints a spring-chamber connection, which
  this file does not declare and the record keeps open.

## What is still open

- **`RF052`**: `X11440` prints a connection in the spring chamber at the rod end and this
  catalog declares the working port only. The model gives a single-acting cylinder one
  working chamber, so a second `working` port would be read as a retract chamber, and the
  schema has no breather role. Closing it needs a schema field (a non-working fluid port)
  in a later phase, not a change smuggled into a drawing.
- The figure's **cap-plate line** at the cap end is not drawn; the barrel is one rectangle
  and the cap is the renderer's inset. The port is inside the cap, which is what the
  function needs, but the line the figure prints is missing.
- The figure's **rod** is an outlined bar; the drawing keeps the rod as one thick line, as
  every other moving part is drawn. The rod's constant length and its box-edge tip are
  tested; its outline is not drawn.
- The figure's **coil** is drawn across the bore; the drawing's is thinner (2.18 mm each
  side of a 8.40 mm bore) so the piston and the rod stay distinguishable, as the contract
  asks. The standard fixes the spring's form, not its amplitude; the record gives the
  measure instead of claiming the figure's.
- The review tool draws **every** symbol in a fixed 40 × 40 mm box, so the captures show
  these normalised coordinates in a square; the sheet's cylinder box is 44 × 20 mm. The
  measures above are the sheet's, and the numeral distance differs between the two boxes
  because the tool scales it with the box's height.

## Images

- `cylinder.single_acting_20_100-300.png`
- `cylinder.single_acting_20_100-light.png`, `cylinder.single_acting_20_100-dark.png`
