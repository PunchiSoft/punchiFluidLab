# 0041 — The shuttle's ball has three seats

- **Status:** proposed
- **Date:** 2026-11-22
- **Deciders:** project owner, implementation session
- **Context crates:** `punchi-fluid-solver`, `punchi-fluid-catalog`, `punchi-fluid-draw`

## Context

The shuttle symbol draws a ball between two seats and the centre dot. [ADR 0029](0029-a-two-pressure-valve-decides-by-pressure.md)
says the outlet follows the higher inlet, and ADR 0039 gave the constraint its form. The
owner's reference and circuit show the ball moving with the signals: **1A alone** pushes it
to the right seat and closes 1B; **1B alone** pushes it to the left and closes 1A; **both
signals at once** hold it **centred**, closing both seats so that port 2 is isolated and the
cylinder does not move.

Today the symbol is static TOML: the ball is a primitive, not a moving part, and the solver
picks one inlet when the two carry the same pressure.

## Decision

1. **The solver's rule.** An inlet is live above atmosphere + 5 kPa. Two live inlets within
   2 kPa of each other hold the ball centred and isolate the outlet; otherwise the outlet
   follows the higher inlet, and when both fall it follows the higher and vents (ADR 0039's
   correction).
2. **The ball has three positions**: seated on the right seat (1B closed) when 1A wins,
   seated on the left (1A closed) when 1B wins, and centred when both live inlets are equal.
3. **The symbol declares its moving part and the three positions**; the renderer places it
   from the element's state, which travels with the live frame. The symbol's form stays the
   standard's; the motion is the simulator's state, not a symbol variant.
4. **The two-pressure valve's piston is not part of this decision.**

## Consequences

- The bands are constants with the tests; the both-live case blocks port 2; the vent of
  ADR 0039 still passes.
- The schema declares the moving part and its three seats, and the draw, the session and the
  FFI pass the state; the review captures the three positions.
- **Debt:** the bands are absolute pressures; when both inlets fall the ball follows the
  vent side.

## Alternatives considered

- **Keep the higher-only rule.** Rejected: the owner's two-signal case would pass air and
  move the cylinder.
- **Animate by matching the symbol id in the draw.** Rejected: the catalog is data-driven,
  and the symbol has no code path of its own.

## References

- [ADR 0029](0029-a-two-pressure-valve-decides-by-pressure.md), the pressure rule.
- [ADR 0039](0039-the-pressure-element-is-flat-and-constrained.md), the constraint.
- The owner's circuit of 2026-11-22: 1A, 1B, the orb/shuttle and the single-acting cylinder.
