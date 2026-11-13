# 0021 — An indicator reads the net, it does not drive it

- **Status:** accepted
- **Date:** 2026-11-13
- **Context crates:** `punchi-fluid-catalog`, `punchi-fluid-draw`, `punchi-fluid-ffi`

## Context

[ADR 0018](0018-the-control-circuit-is-a-boolean-net.md) made liveness the only quantity
the control circuit carries. A student can see a valve move, but a line that only lights a
lamp has no moving part, and a diagram with no visible result teaches nothing. The reference
palette has an indicator lamp and an audible indicator; the lamp is the part that makes a
line's state visible without a load that moves.

## Decision

A lamp is **a load of the boolean net**, not a source and not a switch:

- **One electrical terminal** named by the file; the return is the rail the net already has.
  The validation refuses a second port, a pneumatic role, or no port at all.
- **Lit means the terminal is live**, with the same settle the relays use. The lamp adds no
  edge and does not change liveness: it observes it, and a circuit behaves the same with or
  without the lamp.
- **The state is a run artefact**, like a valve position: not saved, and a reopened circuit
  starts dark.
- **The drawing is never colour alone.** A lit lamp gains eight rays around the circle its
  file declares, and the word `encendida` beside it; a dark lamp has neither and says
  `apagada`. The centre and radius come from the file, so a larger lamp is another file.
- **The interface can ask.** `pf_session_indicator_active` returns `1`, `0` or a negative
  error, so the status bar and a future audible part read the same state as the sheet.
- **An audible indicator is a different part.** Its continuous sound needs the audio path and
  its own tests; it is queued separately and does not change this decision. Calling a silent
  lamp a buzzer would be exactly the pretence this repository forbids.

## Consequences

- The catalog gains `Behaviour::Lamp`, its validation and `electrical.lamp`; the solver
  treats it as an electrical part that contributes nothing to the gas.
- The control model collects the lamp terminals, and the frame carries the lit state as a
  `Reading::Indicator`, which the drawing turns into rays and a word.
- A lamp in series with a coil does **not** model a voltage drop or a current: the boolean
  net has neither. It reads the same line the coil reads, which is the model the course can
  check by hand.

## Alternatives considered

- **A lamp as a coil on a dummy valve.** It would move a part to show a state, and the parts
  list would carry a valve the student did not draw.
- **A colour-only dot.** A reader with a monochrome print or a colour-vision difference
  would lose the state; rays and a word survive both.
- **A real current through the lamp.** It needs resistances and a voltage model the boolean
  net does not have, and would make a first course check Kirchhoff to read a lamp.
