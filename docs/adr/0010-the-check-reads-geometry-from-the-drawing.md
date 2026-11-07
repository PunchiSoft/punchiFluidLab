# 0010 — The design check reads port geometry from the drawing crate

- **Status:** accepted
- **Date:** 2026-11-07
- **Context crates:** `punchi-fluid-check`, `punchi-fluid-draw`, `punchi-fluid-doc`

## Context

`punchi-fluid-check` answers "what is wrong with this circuit before it is run": a component
outside the sheet, two symbols on top of each other, a hose crossing a component, two hoses
over each other, a pneumatic port with nothing attached. Deciding those questions needs the
place of each placement and the route of each hose — the same geometry the drawing uses.

That geometry lives in `punchi-fluid-draw`, in millimetres: the box a turned symbol
occupies, the point where a port sits after a quarter turn and a mirror, and the polyline a
hose follows. `punchi-fluid-check` therefore depends on `punchi-fluid-draw`.

The dependency is the one edge in the workspace that is not obvious from the layer names: a
check is not a drawing, and a reviewer is right to ask why it needs one.

## Decision

`punchi-fluid-check` depends on `punchi-fluid-draw` for geometry only. It uses
`placed::port_point`, `placed::route`, `surface::Pt` and `surface::Rect`; it draws nothing,
opens no window, and calls no `Surface` method. `punchi-fluid-draw` stays free of policy: it
does not know what a design fault is.

`scripts-dev/check-architecture.sh` carries the edge in its allowed graph, so adding a
second consumer of the drawing from a non-rendering crate is a visible decision rather than
a quiet one.

## Alternatives rejected

- **Move the geometry down into `punchi-fluid-doc`.** It is the cleanest layering — a
  placement and its route are document data, and the port position is a property of the
  placement. It is also the largest change in the workspace: `placed.rs` would move, the
  drawing would consume it from below, and every caller of `port_point` and `route` changes
  with it. It buys nothing today, because there is exactly one non-rendering consumer.
- **Duplicate the maths in `punchi-fluid-check`.** Two implementations of "where is the port
  after a turn" is the defect this project cannot afford: a check that disagrees with the
  drawing points at the wrong place, and the screenshot review would not catch it.
- **Put the check inside `punchi-fluid-draw`.** The drawing would gain policy and a fault
  vocabulary, and the crate that must stay free of meaning would start deciding what is
  wrong.

## Consequences

- The check and the drawing cannot disagree about where a port is: there is one function.
- A change to `placed.rs` can break the check, and the check's tests are the second caller
  that keeps that honest.
- If a second non-rendering crate needs the same geometry, the first alternative is no
  longer a wash: move `placed.rs` down into `punchi-fluid-doc`, update the graph and this
  ADR, and leave the drawing as a consumer.
- A `Surface` implementation still has no say in what is checked; the check runs headless,
  with no device attached.
