# 0015 — A recorder is a component that samples a named quantity

- **Status:** accepted
- **Date:** 2026-11-12
- **Context crates:** `punchi-fluid-catalog`, `punchi-fluid-file`, `punchi-fluid-doc`, `punchi-fluid-draw`, `punchi-fluid-ffi`, `punchi-fluid-check`

## Context

The pressure gauge and the flow meter show an instant. A student also has to see **how** a
value changed: whether the pressure rose and settled, whether two strokes took the same
time, whether a valve change made a step. That is the strip chart of F3, and no other
instrument replaces it: a number at 60 frames per second is unreadable as a history.

Two containers were possible. A **window panel** would not travel with the document and
would make a printed sheet lose the evidence the student was looking at. A **component on
the sheet** keeps the instrument where the circuit is, prints it, and is checkable like
every other part. The owner chose the component on 2026-11-12.

A recorder is not a meter: it has nothing to connect, and what it shows is a choice, not a
port. That makes it the first component with a **setting that names another component**,
which is why the document format changes.

## Decision

- `behaviour = "recorder"` in the catalog, with a time window and a sampling rate as file
  values (`span_s`, default `60`, and `rate_hz`, default `10`). The number of samples is
  their product, bounded by the catalog validation, so a file cannot ask for an unbounded
  buffer.
- A recorder has **no ports**. The catalog rule "a component needs at least one port" gains
  one documented exception: an instrument that observes a named variable is not connected
  to anything. A file with zero ports and any other behaviour is still refused.
- The **source** lives on the placement in the project: `{ "placement": <id>, "quantity":
  "pressure" | "flow" | "position" }`. Format 5 adds it; a version 1–4 file reads with no
  source, and saving writes 5.
- The three quantities are the ones the model can already answer:
  `pressure` at the first port of any definition that has one, `flow` through a flow meter,
  `position` of a cylinder. A source that names a missing placement, a placement in another
  circuit, or a quantity the component cannot report is refused by the document and
  reported by the inspector, never drawn as a zero.
- **Samples are a run artefact, not document state.** The session keeps one bounded ring of
  samples per recorder, filled when the simulation advances; opening a document starts the
  charts empty. Nothing in the file grows with the run.
- The curve is drawn inside the recorder's own box, auto-scaled to the window and labelled
  with its current value and unit. A chart with fewer than two samples draws its frame and
  says it is waiting, instead of a flat line at zero that would be a lie about the data.

## Consequences

- `punchi-fluid-file` gains `StoredSource`, a placement field and a format 5 migration.
- `punchi-fluid-doc` gains a source per placement, one undoable command to set or clear it,
  and the refusal rules for a source that cannot be read.
- `punchi-fluid-draw` gains a series on a placed component and a bounded, allocation-free
  polyline inside the box.
- `punchi-fluid-ffi` keeps the per-recorder samples, samples on each advance, exposes the
  candidate sources for the inspector and refuses a source it cannot honour.
- `punchi-fluid-check` gains no fault: an unset source is an incomplete instrument, not one
  of the nine design faults; the inspector says so where the user is.
- The force meter stays **not planned** in the ledger: a force needs a mechanical link the
  model does not have, and a number that claims to be a load when it is `p·A` would teach
  the wrong lesson.

## Alternatives considered

- **A window panel.** Cheaper and always visible, but it is not part of the circuit, does
  not print, and the owner asked for an instrument on the sheet.
- **Recording a hose instead of a component.** Closer to a real test point, but a hose is
  not addressable in the document and the pressure at its end is the port's anyway.
- **Storing the samples in the file.** It would make a reopened chart look complete while
  the run it measured is gone; a document would grow with every second of simulation, and
  the undo history would carry sample data.
- **A generic "display" behaviour with a quantity field.** It would put a switch in the
  solver for what is a different instrument with different rules, the same reason ADR 0008
  and ADR 0014 refused it for the gauge and the meter.

## References

- [ADR 0008](0008-a-gauge-is-a-behaviour.md) and
  [ADR 0014](0014-a-flow-meter-is-a-restriction-that-reports.md), the two instruments this
  one records.
- [P2.2 in next-steps](../next-steps.md), the F3 entry in
  [capabilities.md](../capabilities.md), and the format in
  [docs/reference/sheet-format.md](../reference/sheet-format.md).
