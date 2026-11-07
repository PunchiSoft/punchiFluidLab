# cylinder.double_acting_32_200 — conformance review

    status: pending-300dpi
    date: 2026-11-07
    reviewer: automated pass, unreviewed by a second person
    function: double-acting cylinder, 32 mm bore, 200 mm stroke

## Provenance

Barrel and the two port stubs are hand-written in the definition; the piston and the rod
are drawn by the renderer at the current extension. No third-party asset.

## Automated checks — `punchi-fluid-catalog` symbol lint

| Rule | Result |
|---|---|
| Geometry inside the box, no degenerate primitive | pass |
| Every line declares its class | pass (both stubs are `working`) |
| Every port used and numbered | pass (1 and 2, one at each end) |
| No spring declared on a double-acting cylinder | pass (none declared) |

## Visual check

| Requirement | Result | Evidence |
|---|---|---|
| Barrel, piston and rod, with the rod leaving through one end | pass | catalog sheet |
| Two ports, one drilled in each end cap, no return spring | pass | both leave their cap downwards at the same depth, so the pair reads as one cylinder driven from both ends; `cylinder.double_acting_32_200-light.png` |
| Piston crosses the internal length the file declares and stays inside the barrel | pass | at full stroke it leaves only the declared dead length against the rod-side wall; animation test |
| Rod keeps its length and shows at rest | pass | animation test |
| Legible at palette size | pass | `catalog-dark.png` |
| Legible at 300 dpi | **not checked** | needs a per-symbol render |

## Correction of 2026-11-07

The rod-end port used to leave the barrel's top edge next to the corner, where it read as
part of the outline and crowded the rod. It is now drilled in the rod-end cap and leaves it
downwards, in line with the cap-side one: one port per cap, both drawn at the same depth,
neither over the rod nor over the outline. The numeral of a port on a horizontal edge is
written on the outer side of its own stub, so two neighbouring ports no longer end up with
their numerals side by side between them.

The piston was drawn with a tenth of the bore inset at the top and the bottom and read as a
bar floating inside the barrel; it now meets both walls at every position of the stroke.

## What is still open

- 300 dpi image and a second reader's signature.
- **It is not simulated yet**: driving it needs two working lines, which is the network
  solver of M4. The definition and its symbol are right today; the circuit that drives it
  is the next milestone.
- The 2026-11-06 travel correction shortened the barrel and moved the cap port to the
  centreline; the images next to this file were regenerated after it.
