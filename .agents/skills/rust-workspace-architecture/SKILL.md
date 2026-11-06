---
name: rust-workspace-architecture
description: Keeps the Punchi Fluid crates acyclic and the three extension points intact. Use when adding a crate, module or dependency edge, moving code between crates, exposing a public API, or deciding whether a change needs an ADR.
license: GPL-3.0-or-later
---

# Workspace architecture

The workspace is ten crates with one allowed dependency graph and exactly three
extension points. Both are decisions recorded in
[docs/adr/](../../../docs/adr/); a new edge or a fourth extension point is an ADR,
not a refactor.

## The graph

```text
core ← file
core, catalog ← solver
catalog, core, solver ← draw
catalog, draw, solver ← export
catalog, core, draw, solver, theme ← ui
catalog, core, draw, export, file, solver, theme, ui ← desktop
core, file, solver ← cli
```

`catalog` and `theme` have no internal dependency: the catalog schema is self-contained
and the theme only wraps `egui`. Verify the graph with `cargo tree` before adding an
edge; the manifests are the source of truth when a summary and the code disagree.

Read [AGENTS.md](../../../AGENTS.md) for the owns / never-contains table and
[docs/engineering.md](../../../docs/engineering.md) for the milestone version. The
short form:

| Crate | Owns | Never contains |
|---|---|---|
| `punchi-fluid-core` | components, ports, connections, typed units | numerics, files, widgets |
| `punchi-fluid-catalog` | definition schema, validation, loading | physics, widgets |
| `punchi-fluid-solver` | lumped model, orifice flow, integration | editing, UI, file I/O |
| `punchi-fluid-file` | versioned document format, migrations | physics |
| `punchi-fluid-theme` | semantic colours, desktop theme reading | widgets |
| `punchi-fluid-draw` | mm geometry, `Surface`, symbol drawing | windows, I/O |
| `punchi-fluid-export` | page geometry, PNG/PDF backends | interaction |
| `punchi-fluid-ui` | widgets, ribbon, sidebar, canvas | file I/O, physics |
| `punchi-fluid-desktop` | window, menus, wiring | physics, primitives |
| `punchi-fluid-cli` | headless validate and simulate | UI |

## Procedure: deciding where code goes

1. **Name the responsibility in one sentence.** If it needs "and", it belongs in two
   crates or the sentence is wrong.
2. **Find the crate that owns that responsibility** in the table above. If none does,
   stop: a new crate needs an ADR and a row in `AGENTS.md` and `docs/engineering.md`.
3. **Check the edge direction** against the graph before adding a dependency. A
   shortcut ("the solver needs one struct from the UI") is the start of a cycle.
4. **Check the extension-point rule.** New behaviour should arrive as a catalog file
   (first choice), a `Surface` implementation (new drawing backend) or a `behaviour`
   variant with an ADR (genuinely new physics). Anything else is a change to an existing
   module.
5. **Keep the public surface small.** `pub(crate)` is the default; re-export only what
   the next crate needs. A public module is an API with a doc comment and tests.
6. **Test helpers stay out of production crates.** Use `#[cfg(test)]`, a
   `dev-dependency`, or a fixture file under the crate's `tests/` directory.

## Gotchas

- **`cargo` prevents cycles, not layering.** `ui` depending on `file` compiles and is
  still wrong: the canvas would gain file I/O. Review the edge, not the build.
- **Dev-dependencies may cross layers** because they never reach the binary, but do not
  let a test helper become a back door for production code.
- **No `common`, `utils` or `shared` crate.** It becomes the cycle in the middle. A
  shared type belongs in the crate that owns the concept (`core` for the domain).
- **`core` must stay free of numerics.** A formula there means the solver no longer
  owns the physics and cannot be validated headlessly.
- **The renderer knows no valve by name.** No `match` on a valve id, no `Valve3_2`
  struct, no hardcoded geometry; that is [ADR 0002](../../../docs/adr/0002-components-are-files.md).
- **`DEFAULT_SUPPLY`, `DEFAULT_VALVE`, `DEFAULT_CYLINDER`** in the solver identify the
  demo circuit. They are not a licence to hardcode a component in the model.
- **A new public dependency is a decision**, not a convenience; see
  [rust-dependencies-and-supply-chain](../rust-dependencies-and-supply-chain/SKILL.md).

## Validation

```bash
cargo tree --workspace --edges normal
cargo metadata --no-deps --format-version 1 >/dev/null
cargo test --workspace
```

Read the `cargo tree` output: every line is an edge you are promising to maintain. If a
change adds one, the pull request states why, and an ADR records it when it changes a
rule.

## Related skills

- Language rules: [rust-idioms](../rust-idioms/SKILL.md)
- Tests per layer: [rust-testing-and-verification](../rust-testing-and-verification/SKILL.md)
