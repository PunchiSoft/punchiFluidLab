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

## What is still open

- **The lubricator is not drawn.** The unit's name says filter, regulator and lubricator, but
  only the filter (its diamond and its drain) and the regulator's dial are on the symbol. The
  lubricator's own glyph needs a reference to copy the requirement from, and inventing a
  plausible shape for it is worse than leaving it out and saying so.
- 300 dpi render and a second reader's signature.
- The unit is modelled as a **fixed restriction**: it does not regulate, it does not filter
  and it does not lubricate. A regulator that holds a set pressure is a model of its own.
- The three functions are drawn as **one** block. ISO 1219-1 also allows them drawn
  separately, and a circuit that needs only the regulator should eventually be able to say so.

## Images

- `preparation.service_unit-dark.png`, `-light.png`
