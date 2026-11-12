# 0014 — A flow meter is an inline restriction that reports its flow

- **Status:** accepted
- **Date:** 2026-11-12
- **Context crates:** `punchi-fluid-catalog`, `punchi-fluid-solver`, `punchi-fluid-draw`, `punchi-fluid-ffi`

## Context

[ADR 0008](0008-a-gauge-is-a-behaviour.md) made the pressure gauge a behaviour and left the
flow meter, the force meter and the strip chart to their own decisions. A student reading a
pneumatic schematic needs the second of those: how much air passes a point is as much part
of the lesson as the pressure at it, and the status bar cannot show a quantity it does not
know where to read.

A flow meter is not a gauge with another scale. The gauge hangs from one port and adds
nothing to the circuit; a meter sits **in** the line, so the air goes through it, and the
only honest way to know the flow is to keep the flow that went through it. Two shortcuts
are wrong for the reasons ADR 0008 already gave: a draw-time text label would not connect,
would not be checked and would read nothing, and a catalog id the interface recognises is
the `match` the catalog rule forbids.

## Decision

Add `behaviour = "flow_meter"` to the catalog schema:

- **Two working ports**, inline, port 1 and port 2. The port order defines the sign of the
  reading: positive means gas from port 1 to port 2.
- **`conductance`** is the physics: the solver models the meter as an ISO 6358 restriction
  between its two ports, exactly like a fixed restrictor. A meter does obstruct the line it
  measures, and pretending otherwise would make the model disagree with the circuit.
- **`nominal_l_min`** is the declared full scale, in normal litres per minute. It is a file
  value like the gauge's scale, so a 100 l/min meter and a 2 000 l/min one are two files.
- The solver keeps the mass flow through the meter on the last substep. The interface turns
  it into **normal litres per minute** at 20 °C and 1 013.25 hPa — our
  `Temperature::STANDARD` and `Pressure::ATMOSPHERE` — and draws it beside the symbol as a
  reading, in the same way the gauge draws its number.
- The symbol is the file's geometry. The needle, like the gauge's, is not a later promise
  this ADR makes; the number is the evidence today.

## Consequences

- The catalog lint, the schema validation, the solver and the drawing each gain one arm.
  The solver's restriction edges now remember which part owns them, which is what lets the
  meter read its own flow.
- Inserting a meter changes the circuit, because a real one does. The file states the
  conductance, so the change is a number the user can read and edit, not a hidden constant.
- A meter whose definition the solver cannot build reads nothing rather than zero: an
  absent reading and a reading of zero are different claims.
- The force meter and the strip chart stay their own ADRs. A force needs a mechanical
  domain the model does not have yet, and a chart is a recorder of a reading rather than an
  instrument in the line.

## Alternatives considered

- **A gauge with a `quantity` field.** It would make the two instruments one behaviour with
  a switch, but a gauge has one port and no restriction while a meter has two ports and
  one: the switch would only move the difference into the solver, and the port count cannot
  be a field.
- **A meter that reads the flow of the hose it is plugged into.** Simpler to model, but it
  would read a hose the drawing cannot name without a second kind of connection, and it
  would silently ignore the restriction a real meter adds.
- **Showing the total volume as well.** The reference offers it and it is a later
  improvement, not the first claim: the strip chart (P2) is where a quantity over time
  belongs, and a totaliser needs a reset that the instrument row has not specified.

## References

- [ADR 0008](0008-a-gauge-is-a-behaviour.md), the gauge this follows.
- ISO 6358 through `punchi-fluid-solver::orifice`, the same restriction model as the
  throttle; [docs/symbology.md](../symbology.md) for the symbol contract.
- [P2 in next-steps](../next-steps.md) and the F3 entry in
  [capabilities.md](../capabilities.md).
