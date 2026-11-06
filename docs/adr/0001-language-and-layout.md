# 0001 — Rust and a crate-per-layer workspace

- **Status:** accepted
- **Date:** 2026-11-06

## Context

The product is a desktop simulator of pneumatic and hydraulic circuits, inspired
by the workflow of Festo FluidSIM: draw a schematic, connect ports, run the model,
watch the actuators move. The work splits into three very different problems:

1. the physical model and its numerics;
2. the file format that keeps a user's circuit safe across releases;
3. the schematic editor, which is the largest and most interactive part.

The reference tool is proprietary Windows software. Only its behaviour is a
reference: no asset, symbol set or manual from it enters this repository.

## Decision

Rust, in a Cargo workspace with one crate per layer:

| Crate | Owns | Never contains |
|---|---|---|
| `punchi-fluid-core` | components, ports, connections, units | numerics, files, widgets |
| `punchi-fluid-solver` | lumped-parameter model, orifice flow, integration | domain editing, UI |
| `punchi-fluid-file` | versioned document format and migrations | physics |
| `punchi-fluid-ui` | schematic canvas and widgets | file I/O, solving |
| `punchi-fluid-desktop` | window, menus, wiring | physics, drawing primitives |
| `punchi-fluid-cli` | headless validate/simulate | UI |

Dependencies point one way only: `core` ← `file`, `solver`, `ui` ← `desktop`.

## Reasons

- **The solver is the differentiator and the cheapest thing to get right.** A
  pure crate can be tested headlessly, benchmarked and property-tested; a solver
  embedded in a window cannot.
- **The file format outlives the UI.** Keeping it separate lets the format migrate
  a user's documents while the editor changes underneath.
- **The editor is the risky part.** Isolating it in one crate means the toolkit
  decision (egui today, QML through `cxx-qt` tomorrow) can change without
  touching physics or persistence.
- **A CLI is not a luxury.** `punchi-fluid validate` gives CI and bug reports a
  way to reproduce a circuit without a desktop session.
- **Rust over C++ here** because the project is standalone rather than a Plasma
  component: no KDE integration is required, memory safety is worth more than the
  maturity of `QGraphicsScene`, and the same code can later run in a browser
  through WebAssembly for teaching material.

## Consequences

- The schematic editor has to be built rather than assembled from a framework;
  selection, snapping, undo and hit-testing are our code.
- KDE theming and accessibility are not automatic. If the product must look native
  on Plasma, the path is a QML front end through `cxx-qt`, which is a change
  confined to `punchi-fluid-ui` plus a new `desktop` crate.
- The first release prioritises a **viewer** (load, simulate, animate) over an
  editor, so the simulation proves itself before the drawing tools exist.
