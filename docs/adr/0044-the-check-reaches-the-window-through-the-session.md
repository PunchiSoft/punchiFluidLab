# 0044 — The design check reaches the window through the session

- **Status:** accepted
- **Date:** 2026-11-23
- **Deciders:** project owner, implementation session
- **Context crates:** `punchi-fluid-session`, `punchi-fluid-check`, `apps/punchi-fluid`

## Context

The ribbon's **Comprobar** command shows the design faults of the circuit in view. The rule
that finds them already exists in `punchi-fluid-check` — `check(document, catalog, sheet,
circuit) -> Vec<Finding>`, with `Finding::describe()` and a coordinate to point at — and the
CLI runs it today. The window cannot: its allowed dependencies are `catalog`, `doc`, `draw`,
`export`, `file` and `session`, and `scripts-dev/check-architecture.sh` fails on any edge not
in its list. Adding `punchi-fluid-window → punchi-fluid-check` would be a new dependency
direction, which [AGENTS.md](../../AGENTS.md) says needs an ADR.

The check is **domain logic**: it reads the document, the drawing bounds and the sheet frame
and decides what is wrong. The window must paint the list and take the reader to the fault, not
own the rule. Two candidates were considered: the window depending on the check crate directly,
or the session — which already owns the document's life between the file and the window —
exposing it.

## Decision

1. **The session exposes the check; the window keeps its dependency list.** `punchi-fluid-check`
   joins the session's dependencies and the session offers one thin function:
   `pub fn check_circuit(document: &Document, catalog: &Catalog, sheet: Rect, circuit: u32)
   -> Vec<punchi_fluid_check::Finding>`. It adds no policy of its own: it forwards the inputs
   and returns the check's answer.
2. **No `punchi-fluid-window → punchi-fluid-check` edge.** The window already depends on the
   session; it calls the function, paints `Finding::describe()` and selects the fault's
   coordinate. The sheet rectangle it passes is the drawing area it already computes from the
   export page and the block, so the session does not gain an export dependency.
3. **The implementation updates the contract in the same change as the panel**: the session's
   `Cargo.toml`, the `ALLOWED_EDGES` list in `scripts-dev/check-architecture.sh`, the workspace
   table in [AGENTS.md](../../AGENTS.md) and the reasoning in
   [rust-workspace-architecture](../../.agents/skills/rust-workspace-architecture/SKILL.md).
   A dependency added without its contract line is an unbuilt decision.
4. **The CLI keeps depending on the check crate directly.** The session function is the
   window's path, not a replacement for the domain crate.
5. **The panel does not fake the answer.** An empty list says the circuit passes; a fault that
   cannot be located is still listed with its explanation.

## Consequences

- The graph gains one edge, `punchi-fluid-session → punchi-fluid-check`, which is acyclic: the
  check does not depend on the session.
- The session grows a domain concern over a document it already owns; it already depends on
  `doc`, `draw` and `solver`, so the layer is coherent.
- The window gains no new dependency and keeps its rule that interface code paints, it does
  not decide. The panel's tests use the session function and the check's own tests keep
  proving the rules.
- The `Comprobar` command and its panel can be built in the phase after this ADR; until then
  the ribbon keeps the command disabled, as it does today.

## Alternatives considered

- **The window depends on `punchi-fluid-check` directly.** It would put domain logic behind an
  interface crate and make the window's tests the place where a design rule is proven; the
  layering rule in AGENTS rejects it without a stronger reason than convenience.
- **A new crate just for the panel.** A crate that only forwards one function is a layer with
  no content; the session is the crate that already holds this kind of document work.

## References

- [AGENTS.md](../../AGENTS.md) § Workspace layout and § Branches and delivery.
- [scripts-dev/check-architecture.sh](../../scripts-dev/check-architecture.sh), the
  `ALLOWED_EDGES` contract.
- `crates/punchi-fluid-check/src/lib.rs`, the `check` function the session forwards.
- [ADR 0035](0035-the-window-is-rust.md) and [ADR 0036](0036-the-toolkit-is-egui.md): the
  window's place in the layering.
