# cylinder.single_acting_20_100 — conformance review

    status: pending-300dpi
    date: 2026-11-07
    reviewer: automated pass, unreviewed by a second person
    function: single-acting cylinder with return spring, 20 mm bore, 100 mm stroke

## Provenance

Barrel and port are hand-written in the definition file; the piston, the rod and the
return spring are drawn by the renderer at the current extension. No third-party asset.

## Automated checks — `punchi-fluid-catalog` symbol lint

| Rule | Result |
|---|---|
| Geometry inside the box, no degenerate primitive | pass |
| Every line declares its class | pass (port stub is a `working` line) |
| Declared spring on a component that moves | pass (`[spring] side = "right"`) |
| Every port used and numbered | pass (port 1 at the cap end) |

## Visual check

| Requirement | Result | Evidence |
|---|---|---|
| Barrel, piston, rod and return spring all visible | pass | screenshots at half stroke |
| Spring at the rod side, inside the barrel | pass | coil ends inside the barrel wall |
| Piston crosses the internal length the file declares | pass | at full stroke it leaves only the declared dead length against the rod-side wall; `the_piston_crosses_the_whole_stroke_the_file_declares` |
| Piston travels with the model, not a decoration | pass | `draw_cylinder` reads the solver's extension |
| Rod keeps its length and shows at rest | pass | tip outside the barrel at rest, at the symbol box edge at full stroke; `the_rod_keeps_its_length_at_every_stroke` |
| Spring compresses with the stroke | pass | span shrinks to the dead length at full stroke; `the_return_spring_compresses_with_the_stroke` |
| Compressed coil stays a coil | pass | turns drop from six to two as the span tightens, so the spring is never a filled band; same test |
| Proportions: rod concentric with the barrel, piston square to it | pass | screenshots |
| The piston meets both walls of the bore | pass | it is drawn as the seal between the two chambers; `the_piston_meets_both_walls_of_the_bore` |
| One port, drilled in the cap end, leaving it downwards | pass | the rod end carries no port at all: that is what makes it single-acting; `each_port_of_a_cylinder_is_drilled_in_an_end_cap` |
| Legible at palette size | pass | `catalog-dark.png` |
| Legible at 300 dpi | **not checked** | needs M2 |

## Correction of 2026-11-07

The piston was drawn with a tenth of the bore inset at the top and the bottom and read as a
bar floating inside the barrel; it now meets both walls at every position of the stroke. The
coil is drawn in the same ink and weight as the other functional lines — a grey part is not
a line class in ISO 1219 and is the first thing to disappear in a photocopy. The port leaves
its cap downwards, and its numeral is written beyond the stub.

## What is still open

- 300 dpi render (M2) and a second reader's signature.
- The spring is drawn as a six-coil zigzag that drops to two turns when the piston is
  out: Part 1 does not fix a coil count, but the end turns are not distinguished from the
  active turns. Left as is and recorded.
- The 2026-11-06 travel correction changed the barrel proportion and the spring's
  compressed span; the images next to this file were regenerated after it.

## Images

- `cylinder.single_acting_20_100-dark.png`, `-light.png`
- `catalog-dark.png`, `catalog-light.png`
