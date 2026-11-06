# ADR 0002 — Components are definition files, not Rust types

- **Status:** accepted
- **Date:** 2026-11-06
- **Context crates:** `punchi-fluid-catalog`, `punchi-fluid-solver`, `punchi-fluid-ui`

## Context

Every simulator of this kind grows a component library: valves with different port
counts and actuations, cylinders with different mechanics, silencers, gauges. The
first implementation had them as Rust enums (`ComponentKind::Valve3_2`) and the
drawing hardcoded per type. That makes every new valve a code change: a new variant,
a new match arm in the solver, a new painting function, a new test, a new release.

The requirement from the project owner is explicit: valves must live in a folder of
files, so that a new one is added, changed or deleted without touching the program.

## Decision

A component is a **TOML definition file** validated at load time by
`punchi-fluid-catalog`:

- name, category, summary and localisation;
- ports with a semantic role (`supply`, `working`, `exhaust`, `mechanical`) and a
  position inside the symbol box;
- behaviour: `supply`, `throttle`, `cylinder`, or `valve` as a list of positions,
  where each position lists the *open paths* as `(port, port, conductance)`;
- drawing: geometric primitives normalised to a `0..=1` box, plus one overlay per
  valve position.

Built-in definitions are compiled in with `include_str!` from `assets/catalog/`; the
user folder `~/.local/share/punchi-fluid-lab/catalog/` is loaded on top and wins by
`id`. A file with `disabled = true` removes a built-in definition.

The solver knows only the **roles**: it turns a position into "pressurise the working
line" or "vent it", so a 5/2 valve is a new file and not a new match arm. The canvas
knows only the **primitives**.

## Consequences

Good:

- Adding a valve is a file. Changing a conductance is a number.
- The catalog can be wrong: a broken file is skipped and reported, and the simulator
  keeps running with the rest.
- The validation is one place with its own tests, instead of rules spread over
  constructors.
- Symbols can be tuned by the user without recompiling, which matters while the
  drawing is still rough.

Costs, and how they are paid:

- **No compile-time guarantee** that a definition makes sense. Paid with a
  validation pass at load time, plus a test that fails if a built-in file stops
  being valid.
- **Slower start** by a few milliseconds and a `toml` dependency. Acceptable.
- **Extra indirection** when reading the code. Paid by keeping the schema in one
  module with field-level documentation.
- **Drift between files and physics.** The mechanical fields of a cylinder are data
  too, so a definition can describe a cylinder that starts up impossibly fast. The
  bounds test in the solver (`stroke_time_stays_between_the_two_flow_bounds`) still
  holds for any definition because it is derived from the same numbers.

## Alternatives rejected

- **Rust enum plus a builder.** Still a recompile per valve, and the "folder of
  files" requirement is not met.
- **Embedded scripting language** (Lua, Rhai) for behaviour. Maximum flexibility, but
  it adds a runtime, a sandbox question and a failure mode in the middle of a
  simulation. Not worth it while every valve is a manifold of orifices, which data
  describes exactly.
- **SVG files for symbols.** Closer to how a symbol library is usually shipped, but
  egui has no SVG path support without a rasteriser, and a rasterised thumbnail
  cannot follow the piston. Geometry primitives are enough and are drawn by the
  same painter as the rest of the canvas.
