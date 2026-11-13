# 0024 — A pressure switch reads its own port

- **Status:** accepted
- **Date:** 2026-11-13
- **Context crates:** `punchi-fluid-catalog`, `punchi-fluid-draw`, `punchi-fluid-check`, `punchi-fluid-ffi`

## Context

[ADR 0023](0023-a-limit-switch-follows-a-cylinder-end.md) made the control circuit read a
mechanical position: the limit switch follows a chosen cylinder. The other quantity a
sequence watches is the pressure of the line it is on — the compressor has charged the
receiver, the clamp has gripped, the vacuum cup has held. That part does not need to follow
another placement: it is screwed into the line it measures.

## Decision

- **The part is `Behaviour::PressureSwitch { threshold_bar, above, normally_closed }`**, with
  **one working port and two electrical terminals**. The working port is the line it reads;
  the electrical pair is the contact it switches.
- **The comparison is gauge bar**, the unit the gauges and the threshold use. With
  `above = true` the contact closes at `pressure >= threshold`; with `above = false` it closes
  at `pressure <= threshold`, which is a vacuum switch with a threshold below atmosphere.
- **No binding.** The switch reads `Network::port_pressure` at its own port, so it needs no
  source and no identity in the file. The cylinder of a limit switch is chosen because a
  roller can sit away from the part it watches; a pressure switch cannot.
- **The state is a run artefact.** On an edit or before a run the contact rests at
  `normally_closed`; every `advance` reads the port and re-settles the control net.
- **The design check is static.** It has no run, so it reads the rest state and never
  pretends to know the line pressure.
- **The drawing reuses the contact bar.** A sensing circle, a stem to the working port and a
  dashed link are in the file; the renderer adds the bar from the switch state.

## Consequences

- The control model gains a `pressures` list beside relays, timers and limits, with its own
  `state` slice; the cached topology keeps the step allocation-free.
- A pressure switch is also a legitimate pressure source for a recorder: it has a working
  port and reports the pressure the model already knows.
- A vacuum switch is one more file with `above = false` and a negative threshold; the code
  path is the same comparison, and the review records that its file is validated and drawn.

## Alternatives considered

- **Bind the switch to a placement, as a recorder or a limit switch does.** It would add a
  hidden rule about which line is read and a second source to keep in sync; the port is the
  line.
- **Compare absolute pressure.** The gauges, the thresholds a student reads and the catalog
  dials are gauge; using absolute would make every threshold a number nobody can check by
  hand.
- **A threshold edited per instance instead of per file.** A pressure switch is a part with
  a rating; a file per rating is the same rule as a gauge scale, and the parts list keeps the
  name.
- **A voltage/current model for the contact.** The boolean net already answers whether the
  line is live; the pressure switch only decides when the contact is an edge.
