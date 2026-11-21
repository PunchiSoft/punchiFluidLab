# 0033 — A port closure belongs to the port, not to the sheet

- **Status:** accepted
- **Date:** 2026-11-21
- **Deciders:** project owner, implementation session
- **Context crates:** `punchi-fluid-catalog`, `punchi-fluid-doc`, `punchi-fluid-file`,
  `punchi-fluid-solver`, `punchi-fluid-draw`, `punchi-fluid-check`,
  `punchi-fluid-export`, `punchi-fluid-ffi`, `apps/punchi-fluid-qt`

## Context

[ADR 0016](0016-a-termination-is-a-component.md) made a plug and a silencer
**one-port components**: a box on the sheet, joined to the valve by a hose. That
described the physics correctly and the drawing wrongly. A blind plug or a
silencer is mounted **on the port**: it needs no hose, it takes no sheet space of
its own, and a parts list that counts it as a free component tells the workshop
to buy a part and a piece of tubing that the circuit does not use.

The owner reported it on 2026-11-20 and split the correction on 2026-11-21:
**P-021a** covers the plug and the silencer, **P-021b** the flow regulator and
the silencer + regulator chain. A closure is not a new `behaviour`; `Plug` and
`Silencer` already exist and keep their physics. What changes is **where the
choice lives**: on a port of a placement, not on a placement of its own.

The port hit test, the port geometry and the per-port network node already
exist. The missing pieces are the document field, the format, the network
termination, the drawing, the parts row and the menu.

## Decision

- **A placement carries a per-port closure map**: `port_closures: BTreeMap<port
  id, closure catalog id>`. One closure per port in this node; an absent key is
  no closure. The map is a port-level decision, so it is not part of
  `PlacementEdits`: it is stored in the sheet, undone as its own `Change::Closured`
  and drawn at the port.
- **The role decides the choices**: an exhaust accepts none, a plug or a
  silencer; a supply or working port accepts none or a plug; a mechanical or an
  electrical port accepts none and has no menu. The catalog is data; the role
  table is the interface and the check.
- **A closed port is terminated**: the document and the file reader refuse a pipe
  that ends on it, the canvas does not start one there, and the menu disables a
  closure while a hose is present. The user removes the hose first; nothing is
  deleted silently.
- **Only an exhaust gets a solver termination.** A plug on an exhaust traps the
  air; a silencer replaces the free atmosphere edge with a restriction of its
  conductance; a supply or working plug changes no node (the port is already
  sealed with no pipe), and its effect is the pipe refusal, the drawing and the
  parts row.
- **The closure is drawn at the port**, outside the body, with the closure
  definition's own hand-drawn ISO 1219 symbol. Its own port numeral is
  suppressed, so the plug's bar or the silencer's cone does not read as a
  labelled component; the host port's numeral stays visible and clear of the
  closure box. The drawn footprint grows with the closure.
- **The parts list names the parent and the port**: one flat row after the
  components and before the tubes, for example
  `Silenciador · 1V1 · escape 3`. Equal closure/parent/port groups
  share a row with their quantity, as the component rows do.
- **The palette stops offering `plug.blind` and `silencer.pneumatic` as
  placements.** They remain catalog definitions and stay in the ABI catalog, so
  an old sheet that already places one loads, simulates and prints as before.
- **The menu is non-modal**: a repeated press on a port (double click) or
  `Alt+C` for the selected component opens it; it lists the placement's closable
  ports, offers **Sin cierre**, **Tapón** and, only on an exhaust,
  **Silenciador**, marks the current choice with a check and text, and closes
  with `Esc` or a click outside. The **Propiedades** panel gains a read-only
  “Cierres de puerto” block.
- **Format 14** stores the map as an optional field, omitted when empty; a
  format-13 sheet reads with no closures. **`PF_ABI_VERSION` is 12.**
- **This ADR amends ADR 0016, it does not erase it.** The two behaviours keep the
  physics and the symbols that ADR 0016 gave them; this decision changes only
  where a placement stores its choice, and it retires the placement path from
  the palette while old documents keep reading it.
- **The electrical connection stays out** (P-026): an electrical port never
  offers a closure, the menu hides the section and no parts row is written.

## Consequences

- A sheet can say “this exhaust is silenced” without a hose and without a
  component count that lies; the drawing shows the accessory on the port.
- A new placement field bumps the format to 14 and adds a migration test and a
  round trip; the writer omits the key for a placement with no closure, so a
  format-14 sheet with none differs from a format-13 one only in the version.
- The document refuses a pipe on a closed port and the file reader rejects the
  same sheet as malformed; the check reports a silencer on a non-exhaust port
  and an unresolvable closure, and the solver keeps the free boundary when the
  closure cannot be resolved instead of guessing.
- The ABI grows the port and closure queries and the setter; the palette filter
  lives in `FluidSession::readCatalog()`, so the ABI catalog index that
  `placeCatalogAt` uses is untouched.
- The parts list joins the component rows; the closure rows take room from the
  tube rows under `MAX_ROWS`, and both counters say what was cut.
- **Debt:** one closure per port. The chain and the flow regulator are P-021b,
  with the next free format and an explicit migration of the single id to a
  one-element chain.
- **Debt:** `canvasitem.cpp` grew past the 800-line budget for the input path of
  P-021a; it is listed in `BIG_ALLOWED` with a follow-up node that splits it.

## Alternatives considered

- **Keep the separate placement and add an “attach” gesture.** Rejected: the
  document still stores a placement and a hose, the solver still gives the
  silencer its own node, and the parts list still counts it as a free component.
- **A child placement with a parent id.** Rejected for this node: it adds a
  parent/child relation to every placement, a second selection model and a second
  hit-test rule for a case only ports need.
- **A `Behaviour::PortClosure` variant.** Rejected: it duplicates the plug and
  silencer physics, cannot express the chain later without more variants, and
  needs the catalog's one exception.
- **A chain of closures now.** Rejected for P-021a: the owner split the node and
  the interface cannot create a regulator yet. P-021b adds the chain and its
  setting with the next free format and an explicit migration.
- **Hide the closure inside `PlacementEdits`.** Rejected: edits are numeric
  overrides of the definition, not a per-port decision; the undo, the bounds and
  the parts row would be wrong.
- **Silently delete the hose when a closure is set.** Rejected: the user's hose
  would disappear without a word; the menu asks to remove it first.

## References

- [ADR 0016](0016-a-termination-is-a-component.md): the plug and the silencer
  behaviours and their symbols, amended here.
- [reference/sheet-format.md](../reference/sheet-format.md): format 14, the field
  and the bounds.
- [reference/catalog.md](../reference/catalog.md): the two definitions as closure
  definitions.
- [ui-ux.md](../ui-ux.md): the port menu, the panel block and the palette filter.
- [symbology.md](../symbology.md): the closure drawn at the port and the numeral
  rule.
- [capabilities.md](../capabilities.md): the F1 row and the F6 exclusion.
- [work-tree.md](../work-tree.md): P-021a, P-021b and the `canvasitem.cpp`
  follow-up.
- The owner's report of 2026-11-20, the split of 2026-11-21 and the approved
  specification `specs/port-closures.md`.
