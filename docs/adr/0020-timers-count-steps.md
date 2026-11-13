# 0020 — Timers count steps, they do not sleep

- **Status:** accepted
- **Date:** 2026-11-13
- **Context crates:** `punchi-fluid-catalog`, `punchi-fluid-ffi`

## Context

[ADR 0019](0019-a-relay-is-a-coil-and-a-contact.md) gave the control circuit a relay: a
coil, a contact and a bounded settle. The reference's palette also has the two timer relays a
first course uses — an on-delay and an off-delay — and the teaching question is exact: *the
button was pressed two seconds ago; why has the contact not closed yet?*

A timer that sleeps would stop the simulation, and a timer that reads the wall clock would
make a test unreproducible. The model already advances in fixed steps, so the timer has to be
part of that step.

## Decision

A timer is **a relay with a time rule**, not a second kind of part:

```toml
[behaviour]
type = "relay"
coil = ["A1", "A2"]
contact = ["11", "14"]
kind = "ton"          # `instant` (default), `ton` or `tof`
delay_s = 2.0         # 0 to 3600 seconds; an instant relay leaves it at zero
```

- **The names are IEC 61131-3's**: `ton` is the on-delay — the output follows a live input
  after the delay — and `tof` is the off-delay — the output follows a dead input after the
  delay. `instant` is the relay of ADR 0019.
- **The step supplies the time.** Every `advance(dt)` first settles the net with the timer
  outputs it has, updates each timer from the coil liveness and `dt`, and settles again with
  the new contacts. A live `ton` accumulates and closes at the delay; a dead `ton` resets at
  once. A `tof` closes at once on a live coil and accumulates while the coil is dead.
- **The elapsed time is bounded by the delay**, so a long run does not grow a number without
  a bound, and a timer that is held for an hour still costs one comparison per step.
- **A timer feeding another reacts one step later.** Within a step the timers read the net
  as it was at the start; the second settle only applies their new contacts. The step is the
  resolution of the model, and pretending otherwise would need an event queue the gas model
  does not have.
- **The run state is not saved.** Elapsed time and output belong to the simulation, like a
  valve position; a reopened circuit starts with every timer at rest.
- **The contact is the timer output composed with `normally_closed`**, so an on-delay with a
  normally-closed contact opens after the delay instead of closing.
- **No allocation per step.** The control topology — terminals, wires, relays — is cached on
  an edit; a step only clears and refills edges whose capacity was reserved, and the timer
  map is pre-filled when the model is built. The bound is the number of parts, not the run.

## Consequences

- The FFI grows a cached `ControlModel` and a bounded `timers` map; `advance` updates them
  before sampling the recorders, so a recorder sees the contact the step settled on.
- The catalog validates the delay range and refuses an instant relay with a non-zero delay,
  instead of ignoring a field the file declared.
- The drawing does not change: the bar is still the renderer's job, and `ton`/`tof` differ by
  a clock face in the file and the name in the inspector.
- A counter relay, a latching relay and a pressure switch can build on the same model: each
  one adds a state with a bound, not a new solver.

## Alternatives considered

- **Sleeping or reading the wall clock.** It would stop the model or make the test depend on
  the machine; the simulation already owns a clock.
- **An event queue that fires the timer exactly at its deadline.** It would remove the
  one-step lag and add a scheduler, an ordering rule for simultaneous events and a new source
  of non-determinism, for a resolution the fixed step already gives.
- **A separate `timer` behaviour duplicating the relay's four terminals.** Every match on a
  switch would grow two arms and the two files could drift; the time rule belongs to the
  relay, and the file says which one it is.
