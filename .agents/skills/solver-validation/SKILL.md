---
name: solver-validation
description: 'Validating the lumped-parameter pneumatic and hydraulic model: units, invariants, ISO 6358 and reference curves. Use when implementing or tuning solver behaviour, adding a component model, or writing a physics test.'
license: GPL-3.0-or-later
---

# Solver validation

The solver is a network of lumped volumes joined by restrictions, not CFD. Its
correctness is argued with units, invariants and a reference value whose source and
tolerance are written in the test. "It converged" is not a result.

## Procedure

1. **Draw the model before coding it.** Name the control volumes, the restrictions
   between them, the mechanical degree of freedom, and which catalog parameters feed
   each. If a number is not in a definition, it is a hidden constant and a bug.
2. **Fix the units.** Store SI internally: Pa absolute, K, kg, m, m³, s, kg/s. Convert
   at the boundary with `punchi-fluid-core::units`; a bare `f64` for a physical
   quantity crossing a crate boundary is a defect. Gauge pressure never enters the
   gas law.
3. **Implement the restriction once.** Pneumatic flow is ISO 6358:
   `m = C · p_up · ρ_n · sqrt(T_n / T_up)` in the choked branch (`p_down / p_up ≤ b`),
   and that value times `sqrt(1 − ((r − b)/(1 − b))²)` above it. `C` is the sonic
   conductance, `b` the critical ratio, normal conditions 0 °C and 1.01325 bar. Clamp
   the pressure ratio to `0..=1`; reject non-finite, negative or out-of-range inputs
   with `None` rather than propagating a NaN.
4. **State the integrator and its bounds.** The current model integrates gas mass and
   piston motion with explicit Euler, `MAX_INTERNAL_SUBSTEP = 2e-6 s`, at most
   `MAX_SUBSTEPS_PER_CALL = 500_000` per call. A step that is zero, negative,
   non-finite or too large is a `SolverError`, not an extrapolation.
5. **Derive the reference by hand.** For a choked flow, compute the expected mass flow
   from the formula and write the number and the tolerance in the test. For a stroke,
   bound it between the two flow limits rather than asserting one magic duration.
6. **Assert the invariants every step**, not only at the end: mass is conserved (it is
   the integrated state, so the balance is exact by construction), pressure is never
   negative, state stays finite, the piston stays inside the barrel and the velocity
   changes sign only through a physical event.
7. **Prove determinism.** Same definition, same inputs, same sequence of states. No
   wall clock, no hash-map iteration order in a calculation, no floating-point
   accumulation whose order changes.
8. **Check the hot path.** The step reuses its buffers and allocates nothing after the
   first call; see [rust-performance-and-memory](../rust-performance-and-memory/SKILL.md).

## Reference tests this repository already has

Use them as the pattern for a new one:

| Test | What it proves |
|---|---|
| `starts_at_rest_at_atmospheric_pressure` | initial state is physical |
| `inflow_matches_the_choked_hand_calculation` | ISO 6358 choked branch against arithmetic |
| `stroke_time_stays_between_the_two_flow_bounds` | the result lies between the two analytic limits |
| `mass_balance_is_exact` | the integrated state conserves mass |
| `state_stays_finite_and_physical_over_a_long_run` | no NaN, no negative pressure |
| `releasing_the_valve_lets_the_spring_retract_the_piston` | the mechanical model responds to the valve |
| `the_line_flow_changes_sign_between_filling_and_venting` | direction follows the pressure difference |
| `a_spring_valve_only_holds_while_pressed` | the valve model follows the definition |
| `valve_positions_come_from_the_definition` | no valve is hardcoded in the solver |

## Gotchas

- **Absolute vs gauge pressure.** A supply of 6 bar is about 700 000 Pa absolute at
  sea level, not 600 000. The gas law and the orifice both use absolute pressure.
- **Bar, kPa and Pa are different numbers.** The catalog may speak bar; the solver
  stores Pa. Convert once, at the boundary.
- **`b` is a ratio, not a pressure.** It must be in `0..1`; `b = 0` means no choked
  branch, `b` near 1 means the quarter ellipse is very steep.
- **Clamp the ratio before dividing.** `p_down > p_up` happens during a transient or
  when a port is mis-assigned; do not produce a negative flow from it.
- **Normal conditions are 0 °C, not 20 °C.** The `sqrt(T_n / T)` factor is not a
  cosmetic correction; using 293.15 K changes the flow.
- **Explicit Euler is stable only under the substep cap.** Shortening `dt` is not a
  fix; the internal substep is chosen from the pneumatic time constant. A new volume
  with a much smaller time constant needs the bound revisited.
- **Friction is not damping.** The model has `friction` (a force) and `damping` (N·s/m);
  swapping them changes the stroke without failing a plausibility check.
- **Dead volume is a length in the definition** (`dead_length_mm`), converted to a
  residual volume. Ignoring it makes the initial pressure spike disappear.
- **A toleranced test is not a weak test.** State the tolerance and why; "within 1 %" is
  a claim about the model, "approximately" is not.
- **Hydraulics is not this model.** Liquid is incompressible; ISO 6358 is for
  compressible flow. A hydraulic solver needs its own model and an ADR before it can
  claim a reference.

## Validation

```bash
cargo test -p punchi-fluid-solver
cargo test -p punchi-fluid-solver -- --nocapture   # when a state sequence needs detail
./scripts-dev/check.sh
```

The pull request states the reference (formula, standard clause or data sheet), the
tolerance and the new test names. A solver change without an invariant test is not
finished, even when the UI looks plausible.
