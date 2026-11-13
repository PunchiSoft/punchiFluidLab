# 0018 — The control circuit is a boolean net, not a voltage

- **Status:** accepted
- **Date:** 2026-11-13
- **Context crates:** `punchi-fluid-catalog`, `punchi-fluid-doc`, `punchi-fluid-solver`, `punchi-fluid-draw`, `punchi-fluid-ffi`

## Context

[ADR 0017](0017-the-electrical-circuit-is-a-second-domain.md) put the electrical components
in the document and left the model for this one. The teaching question is small and exact:
**is the coil energised?** A student who wires a 24 V rail to a button and the button to a
solenoid needs to see the valve move when the button is pressed, and to understand why it
does not when a contact is open.

A real electrical solver — voltage sources, resistances, Kirchhoff — would answer the same
question with a model a first course cannot check by hand, and it would need numbers for
every wire and coil that no catalog file carries yet.

## Decision

The control circuit is a **boolean net**:

- **Terminals are nodes.** A terminal is a port whose role is `electrical`; a wire is a pipe
  between two terminals, and a **closed contact** is an edge between its own two terminals.
- **A rail is a source.** An `electrical_supply` marks its positive terminal live. Liveness
  propagates through wires and closed contacts, and nothing else: there is no current, no
  voltage drop and no short-circuit fault in this model.
- **A coil reads its net.** A valve whose file declares the **solenoid** actuator and one
  electrical terminal is energised when that terminal is live: an energised solenoid valve
  sits in the position the actuator selects, and a de-energised one returns to its rest
  position through its spring.
- **The contact's state is a run state**, like a valve position: it starts at whatever the
  file declares (`normally_closed`) and the user toggles it by clicking. It is not saved in
  the document, and a reopened circuit starts with its contacts at rest.
- **The drawing shows the state.** A contact's movable bar is drawn by the renderer — the
  same rule that lets a cylinder's piston move — so the sheet shows an open gap or a closed
  bridge without the file carrying two symbols.
- **The check stays out of it.** No open-terminal fault and no short-circuit fault: the
  reference's mismatched-connection fault needs the rest of the electrical library, and a
  check that guesses is worse than no check.

## Consequences

- The catalog gains the solenoid actuator on a valve file and `Kinematics::Contact`; the
  drawing gains the contact's bar and the coil terminal.
- The session owns the contact states and computes the live nets after every edit and every
  toggle, then sets the solenoid valves' positions before the network steps.
- A solenoid valve is not clickable: its position belongs to its coil, and a click that
  fought the coil would teach the wrong cause.
- The next slices can add a relay (a coil that closes a contact), a limit switch and the
  digital library on the same net, without changing this decision.

## Alternatives considered

- **A full electrical solver.** Honest but far beyond the question: the student needs to know
  whether the coil is energised, and a boolean net is a model they can check by hand.
- **Storing the contact state in the document.** It would make a saved circuit open with a
  button held down; the state of a switch belongs to the run, like the position of a valve.
- **Drawing the closed contact as a second symbol file.** Two files for one component means
  two catalog entries and a document that has to swap one for the other on a click.

## References

- [ADR 0017](0017-the-electrical-circuit-is-a-second-domain.md), the library this models.
- [docs/symbology.md](../symbology.md) for the contact's bar and the coil terminal.
- [P5.2 in next-steps](../next-steps.md) and the F6 entry in
  [capabilities.md](../capabilities.md).
