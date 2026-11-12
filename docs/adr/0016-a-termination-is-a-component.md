# 0016 — A termination is a component: a plug closes, a silencer vents

- **Status:** accepted
- **Date:** 2026-11-12
- **Context crates:** `punchi-fluid-catalog`, `punchi-fluid-solver`, `punchi-fluid-check`, `punchi-fluid-ffi`

## Context

A schematic needs to say two things about a port that a hose does not: that it is
**deliberately closed**, and that an exhaust **breathes through a silencer**. Today the
model has neither. An unconnected working port is a design fault the check reports, so a
student who caps a line to leave it out of service gets a fault they cannot silence; and an
open exhaust always vents at the full conductance of the port, so a silencer cannot slow a
cylinder down the way a real one does.

Both are one-port components. The catalog rule "a component is a file" applies, but a file
still needs a behaviour, and a behaviour is the one thing that needs an ADR.

## Decision

Two behaviours, one per job:

- **`plug`** — exactly one port, no numbers, no physics of its own. The network keeps the
  port's node and nothing else: a plug closes a line, it does not conduct, store or meter.
  Attaching one is what turns "this port is open" into "this port is capped", and the check
  accepts it because the port is connected.
- **`silencer`** — exactly one exhaust port and one `conductance`. The solver gives the port
  an atmosphere reservoir and a restriction of that conductance, so the air leaves through
  the silencer instead of through an open port. The acoustic part is **not** modelled: the
  file's summary and the review say so, because a number that claims to be a noise level
  would be a lie.

Both are `category = "accessory"`. The symbol is the file's geometry, drawn by hand to our
own contract: a stub ending in the blocking bar for the plug, a cone with relief lines for
the silencer.

## Consequences

- The catalog validates the port count and the conductance; two new built-in files and two
  reviews, the first with no numbers at all.
- The solver gains two arms: the plug adds nothing, the silencer adds an atmosphere edge.
  The exhaust rule that closes an open port keeps working; a silencer port is already closed
  by its own edge.
- The check gains a test, not a rule: a capped port is not an open connection.
- A silencer on an exhaust changes the circuit — that is the point — so the restriction is a
  file value a user can edit, never a constant in the solver.

## Alternatives considered

- **Treating a capped port as "no connection" and letting the check know.** It would need a
  second meaning for an absent pipe, invisible on the sheet and impossible to undo as one
  action; the drawing would show nothing where the student capped a line.
- **A single "termination" behaviour with a `silencer` flag.** The two have different ports
  (any pneumatic one versus an exhaust), different numbers (none versus a conductance) and
  different physics; a flag would move the difference into the solver for no gain.
- **Modelling the noise.** Out of scope and unverifiable here: the model has no acoustic
  domain, and the ledger would rather say so than show a decibel.

## References

- [ADR 0002](0002-components-are-files.md), the rule this follows.
- [docs/symbology.md](../symbology.md) for the two symbols and the open 300 dpi review.
- [P3.2 in next-steps](../next-steps.md) and the F1 entry in
  [capabilities.md](../capabilities.md).
