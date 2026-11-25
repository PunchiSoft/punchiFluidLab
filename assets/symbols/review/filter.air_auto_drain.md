# filter.air_auto_drain — conformance review

    status: pending-300dpi
    date: 2026-11-24
    reviewer: pending — the owner's signature after the PR review is the closing step
    function: air filter with an automatic drain

## The claim

- **Function, in one sentence**: the air passes through the element, the condensate collects
  in the bowl below it and leaves automatically through the drain.
- **Family**: conditioning (a two-port part with a drain).
- **Normative source**: the plan records the automatic-drain mark as **`X12200`/`F065V1`**.
  The owner's drawn reference of 2026-11-24 shows the same filter body as `X12190`: the
  square turned 45°, the element dashed down to the bowl chord, the solid chord and the stem
  from the bottom vertex, with no separate manual actuation. **The figure itself was not
  re-read in this pass**; the record names the registration from the plan and the reference
  as the authority for the form (ADR 0040). Rules: `RF052` and `RF053`.
- **Ports**: `1` inlet at the left vertex · `2` outlet at the right vertex. The drain is
  drawn and not declared a port: the model keeps the two working ports and has no bowl to
  empty.

## Reference and assumptions

- **Reference read**: the owner's drawn reference of 2026-11-24 (disposition and form only;
  ADR 0040) and the `X12190` filter body from the 2026-11-24 symbology pass. No capture of
  another application was traced.
- **Owner's decision, 2026-11-24**: the automatic drain is a **separate file**, not a
  variant of the manual filter, and the compact FRL carries the purge arrow in its bottom
  band. The per-placement manual/automatic option is a capability of its own (see below),
  not pretended by this file.
- **Stated by the plan**: the automatic-drain mark is `X12200`/`F065V1`; **the figure was
  not re-read here**, and the record says so instead of claiming a reading it did not do.
- **Assumption**: the automatic drain prints the same diamond, chord and stem as the manual
  one in the owner's reference; the difference is the automatic action, which the model
  does not simulate. The review does not invent a mark the figure does not print.
- **Assumption**: the element stops at the chord because the bowl occupies the space below
  it; the chord spans the diamond's lower edges.

## Measures

The symbol box is the canonical 20 mm square (no `box` key).

| Feature | Requirement | Drawing |
|---|---|---|
| Diamond | the same square turned 45° as `filter.air` | vertices (10; 5,6), (14,4; 10), (10; 14,4), (5,6; 10) |
| Element | dashed, from the top vertex to the chord | (10; 5,6)–(10; 12,84), 7,24 mm |
| Bowl | solid chord across the lower part | y 12,84 from x 8,44 to 11,56 |
| Drain | stem from the bottom vertex | (10; 14,4)–(10; 18) |
| Ports | all of them drawn | `1` at (1,2; 10), `2` at (18,8; 10) |
| Numerals | from the port table, clear of the drawing | `1` above the left stub, `2` below the right stub |

## Evidence

- 300 dpi sheet, print ink: `filter.air_auto_drain-300.png`.
- Light and dark captures: `filter.air_auto_drain-light.png`,
  `filter.air_auto_drain-dark.png`.
- Symbol lint: passes for the built-in.
- Geometry fixture regenerated and read: `assets/symbols/fixtures/filter.air_auto_drain.txt`.
- File choice: `the_two_drain_filters_are_separate_files` in `punchi-fluid-catalog` proves
  both files load, keep their own ids and names, and draw the bowl and stem.
- Sheet format: version 15, unchanged by this phase.

## Checklist

| Question | Answer |
|---|---|
| Is every port the standard draws drawn? | yes: `1` and `2` |
| Do the numerals match the declared ports? | yes: `1` inlet, `2` outlet |
| Is the element across the flow, and does it stop at the bowl? | yes |
| Is the drain at the bottom vertex, not somewhere on an edge? | yes |
| Is the bowl a chord of the diamond, not a separate triangle? | yes |
| Is the automatic action stated as not modelled? | yes, in this record and in the file |
| Is the per-placement option claimed by this file? | no: it is a separate capability, recorded as such |

## What is still open

- **The automatic-drain figure was not re-read**: `X12200`/`F065V1` is the registration the
  plan records, and the owner's reference shows the same diamond, chord and stem; the figure
  itself is an open item for the signature pass.
- The automatic action is **not modelled**: the solver has no bowl to empty, and the file
  declares two working ports. The difference between the manual and automatic files is the
  component's action, not a printed mark in this reference.
- The per-placement manual/automatic option is a **separate capability** (same pattern as
  the cylinder variants); this file is the separate automatic component, not the option.
- The 2012 text is read through the Japanese adoption, not the ISO text itself.

## Sign-off

- **Status**: `pending-300dpi`
- **Reviewed by**: pending — the owner signs after the PR review
- **Date**: 2026-11-24
- **What changed since the last record**: new file; no previous record.

## Images

- `filter.air_auto_drain-light.png`, `filter.air_auto_drain-dark.png`
- `filter.air_auto_drain-300.png`
