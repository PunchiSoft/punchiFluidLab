# 0008 — A pressure gauge is a behaviour, not a draw-time decoration

- Status: accepted
- Date: 2026-11-06
- Deciders: project owner, implementation session

## Context

The reference shows a pressure gauge as a component of the schematic: it connects to a
port like a hose and its dial reads the pressure at that point while the simulation
runs. The catalog rule is "a component is a file, never a Rust type", and a behaviour
variant is the one exception that needs an ADR.

Two shortcuts were possible and both are wrong:

1. **A text label the drawing puts next to any component.** It would look like a gauge
   without being one: it would not connect, would not be part of the circuit check and
   would have no port to read.
2. **A special catalog id the UI recognises.** That is exactly the `match` on an id the
   catalog rule forbids.

## Decision

Add `behaviour = "gauge"` to the catalog schema, with a declared scale (`min_bar`,
`max_bar`). A gauge is a passive one-port component: the network gives its port a node
like any other, adds no volume, restriction or force of its own, and the interface reads
the pressure at that port. The dial is a symbol in the file; the number is drawn beside
it from the model, because a value is a reading and not a drawing primitive.

The gauge is the first instrument of F3. A flow meter, a force meter and a strip chart
each read a different quantity and will get their own variant, one ADR at a time, rather
than one generic "display" that would have to guess what it shows.

## Consequences

- A new `behaviour` variant: the catalog lint, the schema validation and the solver each
  get one arm, and the drawing gets one reading.
- The gauge adds nothing to the physics; a circuit with a gauge simulates exactly as it
  did without it.
- The scale is a file value, so a 0–10 bar gauge and a 0–60 bar one are two files, not
  two code paths.
- A gauge must declare exactly one working port; the catalog validation enforces it.
