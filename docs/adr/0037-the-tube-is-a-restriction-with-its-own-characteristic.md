# 0037 — The tube is a restriction, and its model needs a measured characteristic

- **Status:** proposed
- **Date:** 2026-11-21
- **Deciders:** project owner, implementation session
- **Context crates:** `punchi-fluid-core`, `punchi-fluid-solver`, `punchi-fluid-check`,
  `punchi-fluid-ffi`

## Context

P-024a stores a nominal outside diameter per connection, draws the line thicker with it and
lists the tube in the parts list. The solver still gives every hose the same constant
conductance (`PIPE_CONDUCTANCE = 1.0e-8`) and the same port volume, so a 4 mm jumper and a
16 mm hose conduct identically and neither stores air. A student who changes the tube sees
nothing change in the simulation.

The spec `specs/tube-physics.md` requires a model with a **reference and a tolerance**, and
it leaves the choice open: a linear resistance, ISO 6358 as a long restriction, or a measured
characteristic. The owner asked for the calculations to be considered.

## The reference case, and why the naive model fails

Take a 10 mm outside diameter tube with a 6.5 mm nominal bore, 1 m long, with 5 bar absolute
at one end and 4 bar absolute at the other, air at 293 K.

- **Isothermal compressible flow** with a smooth-tube friction factor (`f = 0.03`):
  `p1² − p2² = (ṁ/A)² R T (f L/D + 2 ln(p1/p2))` gives `ṁ ≈ 0.0153 kg/s`.
- That is `Q ≈ 774 l/min` in a 33.2 mm² bore, so the mean velocity is **≈ 389 m/s**:
  above the speed of sound. The assumption that the tube is a passive subsonic restriction
  has already broken; the tube is **choked**, and the formula cannot be trusted here.
- A perfect nozzle of the same area, fully choked, would pass `ṁ ≈ 0.039 kg/s` (≈ 1 985
  l/min). The real tube sits between the two: friction lowers the sonic conductance and
  moves the critical pressure ratio.

So a lumped orifice with `C` taken from the area alone **overestimates the flow**, and a
linear resistance with a laminar formula is wrong for compressed air. The reference case is
exactly the one the spec demands, and it rules two of the three candidates out as written.

## Decision (proposed)

- **The tube carries a measured characteristic.** Each nominal size stores its **sonic
  conductance `C`** and **critical ratio `b`** as data, next to the outside diameter, with a
  table per length (or a length correction) taken from a source the owner approves. The
  solver feeds them to the existing ISO 6358 branch, which already handles the choked case.
  A characteristic is data, like a valve's conductance: a new size is a new row, not a new
  code path.
- **The bore comes from the nominal wall**, not from the outside diameter: the tube entry
  gains an inner diameter (10 mm outside, 6.5 mm bore in this example) and the volume is
  `A · L`, split between the two ends of the edge with the split stated and the mass balance
  tested.
- **The sizing warning is a velocity limit, not a physics law**: a named constant (starting
  value 20 m/s, the owner's) reported at the pipe with the size, the flow and the threshold,
  as a warning that never refuses the simulation. A pipe with no size is not checked.
- **Unknown sizes fall back, they do not guess:** a size with no characteristic keeps the
  documented constant and the sheet says the model is approximate for that pipe.

## Open question

The measured table is the missing input. The owner must choose its source: a licensed
datasheet, a measurement, or an accepted analytic long-line model with a stated error bound.
Without it, this ADR stays `proposed` and the solver keeps the constant.

## Consequences

- The tube's size starts to matter: the same circuit with a 4 mm and a 16 mm hose responds
  differently, which is the teaching point.
- The catalog grows a data table per size, and the parts list already shows the size.
- **Debt:** the wall/inside-diameter table must be approved by the owner and recorded in
  `docs/reference/tube.md`.
- **Debt:** the velocity limit is a design choice with a source to confirm, not a universal
  constant.

## Alternatives considered

- **A linear resistance (`Δp = R Q`).** Rejected as written: the reference case is choked,
  and a laminar formula does not describe compressed air in a hose.
- **ISO 6358 with `C` from the area only.** Rejected as written: it overestimates the flow,
  as the reference case shows. It survives only if `C` and `b` come from a measured
  characteristic.
- **No model, keep the constant.** Rejected: it makes the size decorative, which is exactly
  the gap the owner reported.

## References

- The owner's spec `specs/tube-physics.md` (in the owning workspace, outside this
  repository): the candidates, the sizing warning and the approval gate.
- [reference/tube.md](../reference/tube.md): the series and the stored size.
- [ADR 0032](0032-a-connection-carries-its-tube-size.md): the size and its format.
- [solver-validation](../../.agents/skills/solver-validation/SKILL.md): a model needs a
  reference and a tolerance.
