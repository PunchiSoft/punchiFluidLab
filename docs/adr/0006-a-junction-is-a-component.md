# ADR 0006 — A T junction is a component, not a document node

- **Status:** accepted
- **Date:** 2026-11-06
- **Context crates:** `punchi-fluid-catalog`, `punchi-fluid-doc`, `punchi-fluid-draw`,
  `punchi-fluid-check`

## Context

Pneumatic circuits branch. The reference creates a T junction when a hose end is
dropped on an existing hose (§3 of its manual), and from that moment three hoses meet
at one point. Our document is port-to-port: a connection names a component and one of
its ports at each end, and a pipe's route is computed from those two ports.

A third hose arriving on a pipe therefore needs somewhere to arrive *at*. Two designs
were on the table:

1. **A document node.** `Connection` ends grow a second shape — a port or a junction —
   and the document keeps a list of junctions with positions. Faithful to what the
   reference does mechanically, and the junction has no physics of its own.
2. **A component.** A catalog file with three ports and a `junction` behaviour, placed
   at the meeting point. The document model does not change at all: the T is a
   component with three ports, and the three hoses are three ordinary connections.

## Decision

A junction is **a component**: `assets/catalog/junction.tee.toml`, a filled circle with
three ports on the same point, and `Behaviour::Junction` — the eighth variant of the
enum that ADR 0002 fixed as the exception that needs this ADR.

The reasons, in the order they weighed:

- **The document stays port-to-port.** Option 1 rewrites `Connection`, the undo
  history, `route()`, the hit test, the pipe check and the C ABI — five places that
  are correct today and would all need new tests. Option 2 rewrites none of them.
  A feature that costs one file instead of five refactors is the one to take.
- **It is what ADR 0002 already decided.** "A component is a file, never a Rust type"
  exists so that a new piece of hardware is a new file. A T junction *is* a piece of
  hardware; the fact that it carries no air of its own does not make it a node.
- **Three ports on one point is the honest model of a node.** Every path to the
  junction costs the same, because every port is at the same place, so the solver and
  the drawing never see a difference between the three.
- **A passive behaviour is a statement, not a gap.** `Behaviour::Junction` says "this
  component adds nothing to the network": no volume, no restriction, no force. The
  solver skips it instead of guessing a large conductance.

The gesture is the reference's: dropping a hose end on an existing hose inserts the
junction at the drop point, splits that hose in two and joins the third port to the
hose being drawn. The junction is also offered in the palette, which the reference
does not do — a documented improvement: a user who wants to lay out a branch before
drawing its third hose can.

## Consequences

- `Behaviour` gains a unit variant. Every exhaustive `match` on it must handle it; the
  compiler lists them, and the range check for a junction is trivially satisfied.
- `assets/catalog/README.md` documents the new behaviour in the same change, as the
  catalog rules require.
- The design check must not report a junction's ports as open pneumatic connections.
  A junction is a point where hoses meet, not a device that has to be wired: a port of
  it with nothing on it is an artefact of modelling a node as three ports, so the rule
  is exempted for this behaviour and the exemption is written where the rule lives.
- A junction adds no volume to the model, so the solver's mass balance is unaffected by
  how many junctions a circuit has — which is the test that proves it is passive.
- If a real pneumatic part ever needs a behaviour that shares air between ports
  passively (a manifold block with several outlets), it is **not** a junction and must
  not reuse this variant: a manifold is a volume and this is a point.
