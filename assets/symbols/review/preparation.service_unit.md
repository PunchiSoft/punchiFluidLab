# preparation.service_unit — conformance review

    status: pending-300dpi
    date: 2026-11-07
    reviewer: automated pass, unreviewed by a second person
    function: service unit — filter, regulator and lubricator — between the compressor and the circuit

## Provenance

Geometry is hand-written in `assets/catalog/preparation.service_unit.toml`. No third-party
asset, no trace, no scan.

## Automated checks — `punchi-fluid-catalog` symbol lint

| Rule | Result |
|---|---|
| Geometry inside the box, no degenerate primitive | pass |
| Every line declares its class | pass (through line, drain and needle are `working`) |
| Every port used and numbered | pass (1 in, 2 out) |
| A two-port component declares one restriction | pass (`throttle`, conductance 5.0e-9) |

## Visual check

| Requirement | Result | Evidence |
|---|---|---|
| The unit is a **box** with the air passing through it, not a bottle and not a valve | pass | rectangle with the working line through it |
| The body is one box with the working line **through** it | pass | a box with no path through it reads as a vessel |
| The filter is the diamond the flow crosses, element dashed | pass | diamond at the centre of the box, dashed chord inside it |
| The drain leaves the body at the bottom, ending in a drain triangle | pass | stem from the bottom of the diamond out of the box |
| The regulator's dial is the gauge it is, on a stem from the top | pass | circle with its needle outside the box; inside it would read as part of the filter |
| It sits between the compressor and the first valve | pass | the worked example wires compressor → unit → valve |
| Legible at palette size | pass | `catalog-dark.png` |

| Legible at 300 dpi | pass | `preparation.service_unit-300.png`: automated pass; the second reader's signature is still owed |

## Correction of 2026-11-20 — P-018a, port designation clearance

The numerals 1 and 2 were centred on the dashed frame's vertical segments, so each glyph
straddled the line and its halo erased the dash under it. The file now declares the frame
as the symbol's functional outer boundary,
`envelope = [0.06, 0.12, 0.94, 0.88]`, and the renderer places each numeral against that
boundary. The frame leaves only 1.2 mm outside its left and right edges, so both numerals
go **inside** the frame, 1.2 mm clear of the dashed line, lateral to the connection and
clear of the filter, the regulator and the dial.

| Requirement | Result | Evidence |
|---|---|---|
| Numeral 1 inside the frame, clear of the dashed segment | pass | `preparation.service_unit-300.png`, 1.2 mm inside the left edge, above the working line |
| Numeral 2 inside the frame, clear of the dashed segment | pass | `preparation.service_unit-300.png`, 1.2 mm inside the right edge, below the working line |
| No numeral on the envelope, the body or a primitive line | pass | `the_service_unit_numerals_stay_clear_of_the_frame_and_the_drawing` in `punchi-fluid-draw` |
| 300 dpi sheet regenerated | pass | `preparation.service_unit-300.png`, 2026-11-20 |
| Light and dark palette-size captures regenerated | pass | `preparation.service_unit-light.png`, `preparation.service_unit-dark.png`, 2026-11-20 |

## What is still open

- **The lubricator is not drawn.** The unit's name says filter, regulator and lubricator, but
  only the filter (its diamond and its drain) and the regulator's dial are on the symbol. The
  lubricator's own glyph needs a reference to copy the requirement from, and inventing a
  plausible shape for it is worse than leaving it out and saying so.
- The 300 dpi sheet is in this folder; only a second reader's signature is owed.
- The unit is modelled as a **fixed restriction**: it does not regulate, it does not filter
  and it does not lubricate. A regulator that holds a set pressure is a model of its own.
- The three functions are drawn as **one** block. ISO 1219-1 also allows them drawn
  separately, and a circuit that needs only the regulator should eventually be able to say so.

## Images

- `preparation.service_unit-dark.png`, `-light.png`
- `preparation.service_unit-300.png`
