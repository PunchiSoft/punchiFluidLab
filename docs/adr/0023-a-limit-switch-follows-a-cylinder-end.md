# 0023 — A limit switch follows a cylinder end

- **Status:** accepted
- **Date:** 2026-11-13
- **Context crates:** `punchi-fluid-catalog`, `punchi-fluid-doc`, `punchi-fluid-draw`, `punchi-fluid-check`, `punchi-fluid-ffi`

## Context

The control circuit can switch on a hand (the push button), on a coil (the relay) and on time
(the timers). A sequence also needs to know where the machine is: the rod is out, so the next
step may start. That is a contact driven by a mechanical position, and it is the first part of
the electrical library whose state comes from the gas model.

The document already binds one placement to another: a recorder stores a `Source { placement,
quantity }` in the format. A limit switch needs exactly that shape — *which cylinder* and
*which end* — and inventing a second binding would duplicate a field the file already carries.

## Decision

- **The part is `Behaviour::LimitSwitch { end, normally_closed }`**, with two electrical
  terminals like a contact. `end` is `extended` or `retracted`; a file per end keeps the
  symbol readable and the palette explicit.
- **The cylinder is chosen, not guessed.** The inspector writes the same source a recorder
  uses, with `Quantity::Position`; no format change and no migration. The ABI exposes
  `pf_session_placement_id` so a front end can name the target, and the existing source
  selector lists only cylinders for a limit switch.
- **The contact closes at the end, not near it**: `extension >= 0.98` for the extended end and
  `extension <= 0.02` for the retracted one, the same margins the telemetry uses to call a rod
  out or home. `normally_closed` inverts the contact for a switch that is closed while free.
- **The state is a run artefact.** On an edit or before a run the switch rests at
  `normally_closed`; every `advance` reads the bound cylinder's extension and re-settles the
  control net, exactly as a timer reads its elapsed time.
- **The design check stays static.** It has no run, so it reads the rest state and never
  pretends to know where the rod is.
- **The drawing reuses the contact bar.** The roller lever is in the file; the renderer adds
  the bar between the two terminals from the switch state.

## Consequences

- The control model gains a `limits` list beside relays and timers, with its own `state`
  slice; the cached topology keeps the step allocation-free.
- The source selector becomes a general "bind this placement to a quantity" control: a
  recorder takes every quantity, a limit switch only position.
- A sequence built from a relay, a timer and a limit switch now runs without a script: the
  parts carry the order.

## Alternatives considered

- **Mechanical ports and mechanical pipes.** It would model the roller as a link between two
  parts, but it adds a third connection domain, its drawing and its checks for one state the
  source already stores.
- **Binding by proximity on the sheet.** It looks automatic until the user moves a symbol and
  the logic changes; the source is visible and undoable, proximity is a hidden guess.
- **A new `limit_source` field in the format.** It would duplicate `source`, bump the format
  and need a migration for no new information.
- **An analogue position threshold per switch.** The file would carry a number the teaching
  question does not ask: a limit switch is at an end, not at 97 % of the stroke.
