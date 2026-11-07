# Project instructions: Punchi Fluid Lab

This file is canonical for this repository. Skills live in `.agents/skills/` and
supplement it; user instructions take precedence. The catalog and the reading order are
in [.agents/skills/README.md](.agents/skills/README.md).

**Resuming work with no context: [docs/retomar.md](docs/retomar.md)** (Spanish) is the
continuity note: state, history step by step, how to verify it, decisions, open questions
and the prompt to continue in a new session.

**Resuming work: [docs/next-steps.md](docs/next-steps.md)** is the work order — current
state, the next milestone step by step, the files it touches, its acceptance tests and
the questions that belong to the project owner.

**Before a capability: [docs/capabilities.md](docs/capabilities.md)** is the plan — what a
student must be able to do, how it behaves, and the test that proves it. The authority is
the standard and the teaching goal, never another product; the procedure is
[.agents/skills/capability-plan](.agents/skills/capability-plan/SKILL.md).

Where the rest lives: [docs/ui-ux.md](docs/ui-ux.md) (interface),
[docs/symbology.md](docs/symbology.md) (ISO 1219 conformance),
[docs/plan.md](docs/plan.md) (milestones and acceptance criteria),
[docs/engineering.md](docs/engineering.md) (memory, security, GitHub practices),
[assets/catalog/README.md](assets/catalog/README.md) (authoring a component).

## Scope and product

A desktop simulator of pneumatic and hydraulic circuits: draw a schematic, connect
ports, run the model, watch the actuators move. Free and open, built so that a student with
no licence and no budget can learn the subject. **No asset, symbol set, manual, example
circuit or file format from any other program enters this repository.**

Status: `0.1.0`, unreleased, **under construction** — advanced in the maintainer's free
time, one capability at a time, with no promised date. A capability that is `missing` or
`partial` in [docs/capabilities.md](docs/capabilities.md) is not there yet, and nothing may
pretend otherwise. There is no editor and no hydraulic model yet.

## Language and stack

Rust, stable toolchain pinned in `rust-toolchain.toml`. See
[docs/adr/0001-language-and-layout.md](docs/adr/0001-language-and-layout.md) for the
decision and its consequences; changing it needs a new ADR, not a silent edit.

## Workspace layout

| Crate | Owns | Never contains |
|---|---|---|
| `crates/punchi-fluid-core` | components, ports, connections, units | numerics, files, widgets |
| `crates/punchi-fluid-catalog` | component definition files: schema, validation, loading | physics, widgets |
| `crates/punchi-fluid-solver` | lumped-parameter model, orifice flow, integration | editing, UI |
| `crates/punchi-fluid-file` | versioned documents: the sheet format and its migrations | physics |
| `crates/punchi-fluid-draw` | the single drawing of a circuit, in millimetres, behind a `Surface` trait | windows, files, fonts, solving |
| `crates/punchi-fluid-doc` | the edited document: placements, commands, bounded undo | physics, drawing, files |
| `crates/punchi-fluid-check` | design faults a circuit can have before it is run | physics, drawing, UI |
| `crates/punchi-fluid-export` | standard sheets, title block, the PNG and PDF writers | physics, widgets |
| `crates/punchi-fluid-ffi` | the C ABI a native front end speaks: one session, one frame of drawing | logic of its own |
| `crates/punchi-fluid-cli` | headless validate and simulate | UI |
| `apps/punchi-fluid-qt` | the window: ribbon, sidebar, sheet, status bar, theme adapter | physics, the document |

Dependency direction: `core` is the base; `catalog` and `file` depend on it and on no
other crate of ours; `solver` depends on `core` and `catalog`; `draw` on `catalog`,
`core` and `solver`; `doc` on `draw`; `check` on `catalog`, `doc` and `draw` (it reads
port geometry from `draw`, because that is where a millimetre lives); `export` on
`catalog`, `draw` and `solver`; `ffi` on all of the above; `cli` on `core`, `file` and
`solver`. Nothing in Rust depends on the window, and the window depends only on
`punchi-fluid-ffi`. A new dependency direction needs an ADR; the `Surface` layer that
made `draw` and `export` possible is [ADR 0003](docs/adr/0003-one-drawing-two-devices.md),
and the Qt window over the C ABI is
[ADR 0005](docs/adr/0005-qt-quick-over-a-c-abi.md). `scripts-dev/check-docs.sh` fails if
this table and `Cargo.toml` disagree.

Non-code directories: `assets/catalog/` (the component definitions and their guide),
`assets/symbols/` (ISO 1219-1 reference), `docs/` (ADRs and the record), `i18n/`
(translations), `packaging/` (AppImage, Flatpak, Windows), `scripts-dev/` (developer
tooling, never a runtime dependency).

## Code rules

- **Units are typed.** Pressures, temperatures and volumes go through
  `punchi-fluid-core::units`; a bare `f64` crossing a crate boundary for a physical
  quantity is a bug. Store SI internally, accept the unit the user thinks in.
- **No `unwrap`, `expect` or `panic!` outside tests.** The workspace lints deny
  them. Recoverable failures are `thiserror` enums; a value that cannot be trusted
  falls back to a documented default instead of aborting.
- **No `unsafe`.** Forbidden by the workspace lints.
- **The domain is total.** A component kind knows its ports; a circuit validates
  connections; the file format never produces an invalid domain object silently.
- **Files are versioned.** Any change to the document shape bumps `FORMAT_VERSION`
  and adds a migration test.
- Comments explain a decision or a trap, never restate the line, and are in English.
- Every source file starts with the SPDX headers used in this repository.

## Component catalog rules

- **A component is a file, never a Rust type.** No `match` on a valve id, no
  `Valve3_2` struct, no hardcoded geometry. See
  [docs/adr/0002-components-are-files.md](docs/adr/0002-components-are-files.md).
- Behaviour is expressed with **port roles and open paths**, so an unknown valve is a
  new file and not a new code path. A new `behaviour` variant is the only exception,
  and it needs an ADR.
- A malformed file is **skipped and reported**, never fatal: the simulator must start
  with whatever is valid.
- Built-in definitions live in `assets/catalog/` and are compiled in with
  `include_str!`; the same text is what a user copies to override one.
- The catalog is validated once, in `punchi-fluid-catalog`, with a test per rule. Do
  not duplicate those checks in the solver or the UI.
- New field: document it in `assets/catalog/README.md` in the same change.

## Symbology rules

- Symbols follow **ISO 1219-1:2012**, in the diagram conventions of ISO 1219-2:2012
  and ISO 1219-3:2016. The contract, the guarantees and the review checklist are in
  [docs/symbology.md](docs/symbology.md).
- **Never trace or copy** a symbol from another program, a screenshot, a manual or the
  standard itself. Geometry is written by hand to conform, and the review records
  which requirement was checked.
- A symbol change needs a **conformance review**: the automated `symbol lint` plus the
  manual checklist, with light/dark screenshots attached to the pull request.
- Normative details are not negotiable for taste: box proportions, arrow direction,
  port numerals, blocking marks, spring and actuator ends, line classes and junction
  dots all come from the standard.
- When a normative question is open, the code says so and the export labels the
  document as unverified. A plausible-looking diagram that is not conformant is the
  worst outcome this project can ship.

## Physics rules

- Pneumatics is a **network of lumped volumes joined by restrictions**, not CFD.
- Orifice flow follows ISO 6358 (`punchi-fluid-solver::orifice`), including the
  choked branch: above the critical ratio the flow stops growing.
- A model is not "done" because it compiles: it needs invariants (mass
  conservation, pressures never negative) and a reference curve or a hand
  calculation. State the reference and the tolerance in the test.
- The solver never touches the UI and never reads files; it receives validated
  definitions and reads numbers from them, so a conductance is a value in a file.
- A definition-driven model is still a model: invariants and reference tests use the
  numbers from the definition, never a copy of them.

## Security and privacy

- Circuit files are untrusted input: parse defensively, bound sizes, never index
  with unvalidated data.
- No telemetry, no network at runtime, no shell-outs.
- `assets/` may only contain files whose licence and origin are recorded; ISO
  symbols are drawn from the standard, never traced from another program.
- Catalog files are untrusted input too: they are parsed with unknown keys rejected,
  every number is range-checked, and no file can make the loader allocate or loop
  beyond a bound.

## Interface rules

- The interface is specified in [docs/ui-ux.md](docs/ui-ux.md). Three regions: ribbon,
  sidebar that mirrors the catalog, sheet. Adding a fourth needs an ADR.
- **Colour, size and font come from the theme adapter**
  (`apps/punchi-fluid-qt/src/theme.cpp`), never from a literal in a widget or in QML.
  `--check-theme` measures the contrast and fails below the criterion.
- Follow the desktop: the application palette, which already carries `kdeglobals`, plus
  the `org.freedesktop.appearance color-scheme` portal, read on start and on change,
  never per frame.
- Components are shown by **their own symbol**, never by a separate icon set.
- State is never colour-only, hover never changes the document, and there is no modal
  on the normal path.
- Accessibility is part of the definition of done: keyboard reachable, contrast
  measured, names exposed to the accessibility layer, reduced motion honoured.

## Memory and performance rules

- Budgets and their verification are in [docs/engineering.md](docs/engineering.md).
- No allocation in the solver step or the per-frame draw loop; reuse buffers.
- Every cache has a bound and an eviction rule; a cache without one is a leak.
- `Arc` for shared immutables, never `Rc<RefCell<…>>`, no global mutable state.
- A change that moves a budget says so in the pull request, with the measurement.

## Validation

```bash
./scripts-dev/check.sh
```

Builds run with two cargo jobs and every test binary under a memory ceiling derived from
this machine, because a single runaway allocation has taken the whole session down before.
The profile is [.cargo/config.toml](.cargo/config.toml) and
[scripts-dev/limits.sh](scripts-dev/limits.sh); the procedure for diagnosing a kill is
[.agents/skills/machine-resource-budget](.agents/skills/machine-resource-budget/SKILL.md).

It runs `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
`scripts-dev/test.sh` (which covers the workspace), the documentation drift check
`scripts-dev/check-docs.sh`, the architecture contract `scripts-dev/check-architecture.sh`
and a CLI smoke test over a temporary circuit file.

The window is built and driven headlessly when `apps/punchi-fluid-qt/build` has been
configured: `--check-qml`, `--check-theme`, `--check-icons` and `--selftest` under
`QT_QPA_PLATFORM=offscreen`. A machine without Qt skips that step with a message on
stderr, and CI does not configure it yet — so a green gate there still does not prove
the window.

During a session, `./scripts-dev/test.sh` runs the same tests and
`./scripts-dev/run.sh` opens the window; both compile only when a source changed and
accept `--no-build` to execute the previous build directly.
A green run proves the crates and the format; it does not prove the window draws
correctly or that a model matches reality.

Behavioral tests are protected: never delete, disable or weaken one to make a change
pass.

## Versioning and changelog

`CHANGELOG.md` is in Spanish. Work accumulates under `## Unreleased` in dated `###
<hito> — <YYYY-MM-DD>` subsections; the declared version changes only when a release
is prepared, updating `Cargo.toml` (`workspace.package.version`), both READMEs and
this file. Documentation, tests and tooling alone never justify a bump.

## Adding a skill

Create `.agents/skills/<name>/SKILL.md` in English: `name` equal to the folder in
kebab-case, `license: GPL-3.0-or-later` in the frontmatter, a description under 300
characters with a `Use when` clause, a procedure, gotchas and a validation section.
Register it in `.agents/skills/README.md`.
