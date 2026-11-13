# 0025 — A latching relay holds its last pulse

- **Status:** accepted
- **Date:** 2026-11-13
- **Context crates:** `punchi-fluid-catalog`, `punchi-fluid-draw`, `punchi-fluid-ffi`

## Context

[ADR 0019](0019-a-relay-is-a-coil-and-a-contact.md) made the relay follow its coil: live, the
contact closes; dead, it opens. A sequence also needs a memory — a start button that keeps a
motor running after the finger leaves, an emergency stop that stays open. That is a relay
with two windings: one sets the contact and one resets it, and neither has to stay live.

## Decision

- **The part is `Behaviour::Relay` with an optional `reset = ["R1", "R2"]` pair.** The plain
  relay keeps working with the field absent; a latching relay has three named pairs (set
  coil, reset coil, contact) and six electrical terminals.
- **Set closes, reset opens, neither holds.** The state is a run artefact, initialised to
  `normally_closed`, and it is not saved.
- **Both coils live is not a priority rule.** The settle keeps the last state; a circuit that
  drives both is a fault a later check can report, not a tie-break invented here.
- **The latch is part of the same fixed point.** Each settle pass reads the coils, updates
  the latch and re-fills the edges until the net stops changing, bounded by the number of
  relays. The cached topology keeps the step allocation-free.
- **The drawing reuses the relay bar.** The file holds two coil rectangles and the two
  dashed links; the renderer draws the contact from the latched state.

## Consequences

- The control model's relay state has three kinds now: instantaneous, timed and latched, all
  in the same list and the same settle.
- The catalog validation accepts four or six terminals and refuses a shared or missing one;
  the review records the port numerals that sit close at palette size.
- A counter relay and a flip-flop in the digital library can follow the same shape: a state
  with a bound, not a new solver.

## Alternatives considered

- **Two separate relays wired as a latch.** The parts list would carry two parts for one
  physical device, and the wiring would be a circuit the student has to draw twice.
- **A digital flip-flop for the electrical panel.** It belongs to the digital library, which
  is a different symbol set and a different teaching step; the relay panel must stand alone.
- **Saving the state in the document.** A saved circuit would open with the motor already
  running; the state belongs to the run, like a valve position.
