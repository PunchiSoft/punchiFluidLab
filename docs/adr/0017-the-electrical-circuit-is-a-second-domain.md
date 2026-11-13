# 0017 — The electrical control circuit is a second domain

- **Status:** accepted
- **Date:** 2026-11-13
- **Context crates:** `punchi-fluid-catalog`, `punchi-fluid-solver`, `punchi-fluid-check`, `punchi-fluid-ffi`

## Context

A pneumatic diagram is half a machine. The valve that drives the cylinder is moved by a
solenoid, the solenoid is switched by a button, and the student who draws only the air has
drawn the power half of the circuit and none of its control. F6 in the plan is that library,
and the ledger is honest that it does not exist yet.

The catalog rule is that a component is a file, and the one exception — a behaviour variant
— needs an ADR. Electricity is not air: a port that carries 24 V is not a volume, a contact
does not hold gas, and a check that reports an open pneumatic connection on a terminal would
be teaching the wrong fault.

## Decision

The electrical control circuit is a **second domain inside the same document**, with its own
symbols and its own rules:

- **A terminal is a port role**, `electrical`, not a special component. A coil, a contact
  and a supply rail all declare electrical ports, and the catalog can say so without a new
  file type.
- **Two behaviours arrive first**: `electrical_supply` (a rail at a declared voltage) and
  `contact` (a two-terminal switch, normally open or normally closed). Their files carry the
  symbol, the terminals and the state they start in.
- **The drawing uses the `electrical` line class.** A hose whose two ends are electrical
  terminals is drawn dotted, because the class says what the line is and the reader sees it
  before reading any label.
- **The check leaves electrical terminals alone.** An open terminal is not an escaping
  pneumatic port; the reference has its own fault for a mismatched connection, and this
  build does not implement it yet.
- **The first slice does not energise anything.** The solver builds no volume, pressure or
  force for an electrical component: this ADR covers the library and the drawing. The
  control model — a line that is live or dead, a contact that joins two lines, a coil that
  reads its line and moves the valve — is the next slice, and it gets its own ADR because it
  decides what the model *is*: a boolean state, not a voltage and a current.

## Consequences

- The schema gains `PortRole::Electrical` and the two behaviours; the solver skips them
  whole, so a document with a control circuit simulates exactly as the air part did.
- The check gains one guard: an electrical terminal is never an open pneumatic connection.
- The pipe class is chosen by the class rule: two electrical ends draw a dotted line, and
  the user can still override it — the drawing follows the class, never the definition id.
- The library is drawn and reviewed like every other symbol: hand-drawn geometry, a review
  file and the 300 dpi debt recorded.

## Alternatives considered

- **Voltage and current, a real electrical solver.** Correct and far beyond the teaching
  need of this slice: a student reading a schematic needs to know whether a coil is
  energised, and a boolean line answers that with a model that can be checked by hand.
- **A separate document for the control circuit.** It would break the one thing the student
  must see: the button and the valve it moves on the same sheet, wired together.
- **A `solenoid` behaviour on the valve now.** It is the right destination, but it decides
  how a coil reads its line and what a de-energised spring-return valve does, which is the
  next ADR rather than a detail of this one.

## References

- [ADR 0002](0002-components-are-files.md) and
  [ADR 0014](0014-a-flow-meter-is-a-restriction-that-reports.md) for the behaviour rule.
- [docs/symbology.md](../symbology.md) for the symbols and the line class.
- [P5 in next-steps](../next-steps.md) and the F6 entry in
  [capabilities.md](../capabilities.md).
