# 0038 — A port closure is a chain: the regulator and the silencer in series

- **Status:** proposed
- **Date:** 2026-11-22
- **Deciders:** project owner, implementation session
- **Context crates:** `punchi-fluid-catalog`, `punchi-fluid-doc`, `punchi-fluid-file`,
  `punchi-fluid-solver`, `punchi-fluid-draw`, `punchi-fluid-check`, `punchi-fluid-export`,
  `punchi-fluid-ffi`, `apps/punchi-fluid-qt`

## Context

[ADR 0033](0033-a-port-closure-belongs-to-the-port.md) gave one port one closure id: a plug
or a silencer, stored in the placement's closure map and drawn at the port. An exhaust in a
real circuit often carries a **flow regulator and a silencer in series**: the regulator sets
how fast the air leaves, the silencer makes it quiet. One id cannot say that there are two
parts, which one is nearer the port, or how far the regulator is open.

The owner split the correction on 2026-11-21. P-021a delivered the single closure
([ADR 0033](0033-a-port-closure-belongs-to-the-port.md)); P-021b owns the chain and the
adjustable regulator. The physics already exists: `Throttle` is a restriction with a
conductance solved through ISO 6358, and `Silencer` is an exhaust restriction to
atmosphere. A new behaviour would copy both and still not express the order.

The format is at 14, where the closure is one optional string per port.

## Decision

- **A port closure is an ordered chain of elements, from the port outward.** An empty chain
  is no closure; a one-element chain is today's plug or silencer. The placement stores a
  chain per port, and the chain's order is the physical order: the first element is fitted
  to the port, the next to that element.
- **An element is a catalog id plus an optional setting.** Only the adjustable flow
  regulator carries a setting. The setting is the conductance it presents, stored in SI
  (`m³ / (s · Pa)`), bounded by the range its definition declares; the menu shows it as the
  fraction of that range, so the file never carries a unit the user cannot check.
- **The regulator is a catalog definition, not a behaviour.** `throttle.adjustable` declares
  the existing `Throttle` behaviour plus a conductance range; the chain element's setting
  overrides the nominal conductance for that port. A separate behaviour variant is not
  needed and would duplicate the ISO 6358 model.
- **The role table of [ADR 0033](0033-a-port-closure-belongs-to-the-port.md) is extended,
  not replaced**: an exhaust accepts none, a plug, a silencer, a regulator, or a chain; a
  supply or working port accepts none or a plug; a mechanical or electrical port has no
  menu. A plug must be the **last** element: it ends the chain.
- **The solver composes the chain as restrictions in series** between the port node and the
  atmosphere: the regulator's conductance from its setting, the silencer's fixed
  conductance, the total by the series rule. A plug closes the node and adds no edge. The
  composition is pure and allocation-free in the step and adds no state to the document.
- **The file takes the next free format version** (15 unless another change takes it first)
  and publishes the migration from the single closure id to a one-element chain; a
  format-14 sheet reads as a chain of one, and the writer omits an empty chain.
- **The menu edits the chain**: add an element, remove one, reorder it, and set the
  regulator's opening. The **Propiedades** block lists the chain read-only, and the parts
  list names each element in order with its parent and its port.
- **The drawing draws each element in order at the port**, with the host numeral clear, and
  the footprint grows with the chain, as [ADR 0033](0033-a-port-closure-belongs-to-the-port.md)
  established for one element.

## Consequences

- A sheet can say "this exhaust has a regulator and a silencer" with two parts in the list,
  the order they are fitted in and the regulator's opening.
- The document, the file, the solver, the drawing, the export, the check, the ABI and the
  Qt menu grow by one shape; each needs its round-trip, migration or reference test.
- The format bumps and the migration from one id to a chain is explicit, with a test that
  reads a format-14 sheet and writes a one-element chain.
- The regulator's file brings its geometry fixture and its review, like every symbol; the
  acoustic part of the silencer stays unmodelled and the review says so.
- **Debt:** the chain has no per-element orientation or spacing rule; it is drawn in the
  order it is declared, and the drawing's footprint is the sum of the elements.
- **Debt:** the regulator's setting is a conductance, not a flow; a student sets an opening,
  not litres per minute. A flow scale would need a measured reference and is not claimed.

## Alternatives considered

- **A second closure key beside the first.** Rejected: two independent values cannot say
  "in series", order the parts or refuse a plug in the middle.
- **One combined `silencer.regulated` definition.** Rejected: the setting is per instance,
  the parts list must count two parts, and the order is a property of the port, not of the
  definition.
- **A new behaviour for the regulator.** Rejected: the physics is the existing `Throttle`;
  a variant would duplicate ISO 6358 and need the catalog's single exception for no gain.
- **A free graph of closures.** Rejected: a port is a line, not a network; an ordered list
  is the smallest shape that says what a student sees on the sheet.
- **Keeping the single id and putting the regulator in the definition.** Rejected: the
  opening varies per port, and a catalog file is shared by every placement.

## References

- [ADR 0033](0033-a-port-closure-belongs-to-the-port.md), the per-port closure this extends.
- [ADR 0016](0016-a-termination-is-a-component.md), the plug and silencer physics kept.
- [capabilities.md](../capabilities.md), F1: the silencer with a regulator is part of the
  essential core, and the flow regulator and the chain are the node P-021b in the owner's
  working notes.
- ISO 6358, the restriction model; ISO 1219-1, the symbols of the regulator and the
  silencer.
