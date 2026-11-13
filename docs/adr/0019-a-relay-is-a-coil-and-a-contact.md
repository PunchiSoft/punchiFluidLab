# 0019 — A relay is a coil and a contact in one file

- **Status:** accepted
- **Date:** 2026-11-13
- **Context crates:** `punchi-fluid-catalog`, `punchi-fluid-draw`, `punchi-fluid-check`, `punchi-fluid-ffi`

## Context

[ADR 0018](0018-the-control-circuit-is-a-boolean-net.md) made the control circuit a boolean
net: terminals are nodes, wires and closed contacts are edges, and a rail marks its positive
terminal live. A push button is two terminals whose edge appears when the user clicks.

A relay is the first part that **moves a contact from inside the net**. It is also the part
that makes the net non-trivial: the contact of one relay may feed the coil of another, and a
relay may even feed its own coil. The model needs a rule for that feedback that terminates.

## Decision

A relay is one definition file with **four electrical terminals in two pairs**:

- `coil = ["A1", "A2"]` and `contact = ["11", "14"]` are named by the behaviour, because all
  four terminals share the `electrical` role and the pair is behaviour, not geometry. The
  validation requires four distinct electrical terminals; a shared or missing terminal is
  refused.
- **The coil is energised when exactly one of its terminals is live.** With a single live
  rail and an unseeded return, that is the boolean shadow of a potential difference across
  the coil: two live terminals mean the coil is shorted, no live terminal means it has no
  supply.
- **The contact follows the coil.** A normally-open contact is an edge while the coil is
  energised; a normally-closed one is an edge while it rests. `normally_closed` defaults to
  `false`, and the state is a run artefact: it is not saved, and a relay cannot be clicked.
- **The net settles in bounded passes.** Each pass rebuilds the edges from the wires, the
  clicked contacts and the relay states, energises the net, and recomputes every relay from
  its coil. The loop stops when the states stop changing, and it is bounded by
  `2 · relays + 2`, so a circuit that oscillates still ends. The bound is a safety rail, not
  an expected path: a well-formed circuit settles in one or two passes.
- **The check reads the same net.** The ninth fault, the rail joined to its return, settles
  relays before it looks for a path, so a short through a relay contact is found.
- **No delay.** A timer is a different part: `TON` and `TOF` follow IEC 61131-3 and are
  counted in steps, never slept. This decision gives the relay no time constant of its own.

The drawing follows the file: a coil rectangle with A1 and A2, the contact terminals 11 and
14, and a dashed mechanical link between them. The contact bar is drawn by the renderer from
the state the net settled on, like the bar of a push button.

## Consequences

- The catalog schema gains `Behaviour::Relay`; a relay file is validated once, with a test
  per rule, and the solver treats it as an electrical part that contributes nothing to the
  gas.
- The ABI control pass becomes a small fixed-point loop, and the settled state is kept for
  the frame so the sheet draws the bar the run is using.
- The check's electrical net grows the same settle step, which keeps the ninth fault honest
  when a relay is in the path.
- The next slices build on the same file shape: a latching relay adds a second coil, a timer
  adds a declared delay, a counter adds a preset. None of them changes the four-terminal
  idea.

## Alternatives considered

- **Model a relay as a `Contact` with two extra coil ports.** The contact validation and the
  renderer would then have to guess which pair is which from port order, and a file with a
  different order would silently draw or close the wrong pair.
- **Two separate components, a coil and a contact, wired together.** The document would hold
  two placements for one physical part, the parts list would say two, and nothing would stop
  the user wiring the coil of one relay to the contact of another.
- **A full electrical solver.** It would answer voltage and current questions this course
  does not ask yet, at the cost of a model no first-year student can check by hand.
