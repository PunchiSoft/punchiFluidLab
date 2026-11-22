# 0039 — The pressure element is a flat symbol and a per-step constraint

- **Status:** proposed
- **Date:** 2026-11-22
- **Deciders:** project owner, implementation session
- **Context crates:** `punchi-fluid-catalog`, `punchi-fluid-solver`, `punchi-fluid-draw`,
  `punchi-fluid-export`, `punchi-fluid-session`, `punchi-fluid-ffi`

## Context

[ADR 0029](0029-a-two-pressure-valve-decides-by-pressure.md) decided the `Pressure`
behaviour: a shuttle and a two-pressure valve, each with two `supply` inlets and one
`working` outlet, deciding by pressure and not by position. The decision stands. Three of
its implementation sentences do not hold against the code:

1. **There are no `check` or `piston` parts.** The schema's parts are the spring, the
   return and the actuator. Any definition with a `body` is drawn by the position-valve
   drawer, which splits the body into one square per position and reads
   `Behaviour::Valve`; a pressure element with a body would be drawn as a valve, or not
   at all. Its seats and its double piston have no declarable form today.
2. **The port id is the numeral.** `Port::id` is both the connection key and the text the
   renderer prints, and it must be unique. The standard draws both inlets with the numeral
   `1`, which one id cannot express.
3. **The solver is an edge integrator.** A node has a mass and a volume, an edge a
   conductance, and the step computes flows from the node pressures. There is no
   fixed-pressure boundary, so "the outlet follows the inlets" needs a mechanism the
   current model does not have.

## Decision

- **The pressure element is a flat symbol.** It declares no `body`: its square, its two
  check seats or its double piston, and its port stubs are `[[symbol]]` primitives with
  their line classes, drawn by the same flat path as a gauge or a silencer. The
  position-valve body drawer stays for `Behaviour::Valve`.
- **A port gains an optional `numeral`, defaulting to its id.** The id stays unique and is
  what connections, the document and the solver address; the numeral is what the renderer
  prints. Both supply inlets carry the numeral `1` with distinct ids (`1a` and `1b`), and
  the outlet carries `2`.
- **The solver applies a per-step constraint, not an edge.** After the edge flows of a
  substep, the outlet node is moved toward the rule by transferring mass from the inlet
  that feeds it:
  - the **shuttle** takes from the higher inlet;
  - the **two-pressure** valve takes from the lower inlet while both are above atmosphere,
    and vents the outlet to atmosphere when either inlet falls to it;
  - the transfer is capped by the mass the inlet holds, so no mass is invented;
  - a check seat holds the outlet when it is above the rule, and the circuit consumes it.

  The element stores no state, allocates nothing in the step and stays deterministic.
- **The reference tests** cover two pressures (the shuttle picks the higher, the
  two-pressure valve the lower), one inlet (the shuttle follows it, the two-pressure valve
  vents), neither (both at atmosphere), the mass balance and the determinism.

## Consequences

- The catalog schema gains the optional `numeral`; no file format changes, because ports
  are catalog data.
- The two definitions are flat files with their primitives, and they bring their geometry
  fixtures, their reviews and their three captures like every symbol.
- The solver gains a pressure list and the constraint; the network test helper reads the
  outlet with `port_pressure`.
- The ABI catalog lists the two definitions; the parts list and the drawing use the
  numeral, not the id.
- **This record amends the implementation sentences of ADR 0029; the behaviour decision
  itself is unchanged.**
- **Debt:** the constraint is a projection over one substep, not an implicit solve; at the
  2 µs internal substep the lag is below the reference tolerance, and a test states it.

## Alternatives considered

- **Add `check` and `piston` parts to the renderer.** It is what ADR 0029's text assumed,
  but the geometry would split between the file and the renderer, and the renderer would
  need to know two forms that only these two elements use. The flat file keeps the
  geometry where every other symbol keeps it.
- **A pressure-selected edge.** It reuses the edge model, but the outlet then drops with the
  flow instead of following the rule, which is the behaviour ADR 0029 exists to fix.
- **Duplicate ids and one numeral.** Connections and the document address a port by id;
  two inlets with the same id make a pipe ambiguous.
- **Suppress the inlets' numerals.** The symbol would lose the two `1`s the standard draws
  and the reader would not know which ports are the supply.

## References

- [ADR 0029](0029-a-two-pressure-valve-decides-by-pressure.md), the behaviour this
  implements.
- [ADR 0002](0002-components-are-files.md): the geometry stays in the file.
- [capabilities.md](../capabilities.md), F2: the shuttle and the two-pressure valve.
- ISO 1219-1:2012, the seat and the double piston; ISO 5598 for the names.
