# 0029 — A two-pressure valve decides by pressure, not by position

- **Status:** accepted
- **Date:** 2026-11-18
- **Deciders:** project owner, implementation session
- **Context crates:** `punchi-fluid-catalog`, `punchi-fluid-solver`, `punchi-fluid-draw`

## Context

A student meets two passive valves before any sequence: the **shuttle** (OR), whose outlet
takes the higher of two inlets, and the **two-pressure** valve (AND), whose outlet only
receives air when both inlets are pressurised. They are the pneumatic version of OR and AND
and the first elements of every control circuit.

Today `Behaviour` has `Supply`, `Valve`, `Cylinder`, `Gauge`, `Throttle`, `FlowMeter` and
`Recorder`. A shuttle valve cannot be a `Valve`: a valve switches positions and connects
ports; these decide from the pressures they see, with one port closed by the higher one.
Modelling them as a `Throttle` would let both inlets feed the outlet at once, which is
exactly the behaviour they exist to prevent.

The two symbols are in the parent application, which the owner authorised as the reference:
both are a square body with two inlets numbered `1` and one outlet numbered `2`; the shuttle
draws two opposing check seats with a common centre, the two-pressure valve a double piston
that the outlet reaches.

## Decision

- **A new `Behaviour::Pressure` with a `kind` of `shuttle` or `two_pressure`.** It is the one
  exception the catalog rules allow: a behaviour variant, with this record behind it. The
  ports are declared like any other file: two `supply` inlets (`1`) and one `working` outlet
  (`2`).
- **The rule is a pressure rule, not a position.** The **shuttle** connects the outlet to the
  inlet with the higher pressure and closes the other. The **two-pressure** valve connects
  the outlet only while both inlets carry pressure above the atmosphere, and the outlet takes
  the **lower** of the two; when either falls, the outlet vents through the same path.
- **It is solved as a pressure boundary, not as a restriction pair.** Each step, the element
  reads its inlets and fixes its outlet: the outlet is one node whose pressure follows the
  rule, with flow taken from the inlet that feeds it. No internal volume is invented, and the
  element adds no state to the document.
- **Both are pure and allocation-free in the step.** No history, no timers, no stored
  position: the same inputs give the same outputs, which keeps the determinism test valid.
- **The geometry is a file**, as every component is: `body`, `[[port]]`, a `check` or
  `piston` part the renderer already draws for a seat and a spool, and the numerals from the
  port table.
- **One check, not two.** A shuttle with both inlets live is normal; a two-pressure valve with
  one inlet live is its normal closed state, not a fault.

## Consequences

- The catalog schema gains one variant and its validation: exactly two `supply` inlets and
  one `working` outlet, no spring, no actuator, no position list.
- The solver gains one element kind with two rules and a reference test each: with two
  pressures, with one, and with neither.
- The files live in `assets/catalog/` and bring their geometry fixtures and their review
  records, like every other symbol.
- The export and the window need no change: both read the same drawing.
- Debt: the two-pressure valve vents through the inlet that lost pressure, which is what the
  symbol shows; a version with a dedicated exhaust port would be another file and another
  variant field, and is not needed to teach the AND.

## Alternatives considered

- **A `Valve` with two positions driven by pressure.** It would need two positions the file
  does not have and a spool that moves with no actuator, and the symbol of a position valve
  is not the symbol of a shuttle. The reader would learn the wrong picture.
- **Two `Throttle` restrictions in parallel.** Both inlets would feed the outlet, so the
  shuttle would mix pressures instead of selecting one, and the AND would pass air with one
  inlet live — the two behaviours the elements exist to prevent.
- **A generic "pressure logic" element with an expression field.** A string language inside a
  catalog file is a second program with its own bugs; two named kinds are read by a student
  and validated by the catalog.

## References

- ISO 1219-1:2012, the check-valve seat and the two-pressure valve; ISO 5598 for the names.
- The parent application, read-only, for the port numerals and the body proportions.
- [ADR 0002](0002-components-are-files.md): the single exception this record uses.
- [docs/symbology.md](../symbology.md) and the review record template.
