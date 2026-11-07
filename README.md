# Punchi Fluid Lab

**English** | [Español](README.es.md)

[![CI](https://github.com/PunchiSoft/punchiFluidLab/actions/workflows/ci.yml/badge.svg)](https://github.com/PunchiSoft/punchiFluidLab/actions/workflows/ci.yml)
[![License: GPL-3.0-or-later](https://img.shields.io/badge/license-GPL--3.0--or--later-blue.svg)](LICENSE)
[![Status: under construction](https://img.shields.io/badge/status-under%20construction-orange.svg)](#)

> 🚧 **Under construction.** This is a working prototype, not a finished product. It is
> built by one person in the free days left by a full-time job, so it moves steadily but
> slowly, and **no delivery date is promised**: when a capability in
> [docs/capabilities.md](docs/capabilities.md) says `missing` or `partial`, that is exactly
> what it means. If something is not there, it says so; if it is half done, it says which
> half is missing.

![The Punchi Fluid Lab window on a dark desktop: the ribbon with the edit commands, the catalog of components on the left, a 3/2 valve in its rest position feeding a single-acting cylinder on the sheet, and the status bar reading 0,00 bar, 0,0 mm and 0,0 mm/s](assets/screenshots/window-dark.png)

*The window as it is today: ribbon, catalog, sheet and status bar, with the desktop's own
dark palette. The circuit is a 3/2 valve in its rest position feeding a single-acting
cylinder; nothing is running, so the hoses keep the neutral ink and the status bar reads
`0,00 bar`. The interface is Spanish today — [i18n/README.md](i18n/README.md) is the plan for
the other languages.*

A free, open simulator of pneumatic and hydraulic circuits, built so that a student with no
licence, no budget and no connection can draw a schematic, connect the ports, run the model
and watch the actuators move. The model, the format and the drawing are Rust; the window is
Qt Quick.

**Status: `0.1.0`, unreleased.** A working vertical slice: draw and edit a circuit, connect
port to port, save and open it, run the gas model, read the pressure on the hoses, and
export the sheet to PNG or PDF.

No account, no key, no network. GPL-3.0-or-later: download it, teach with it, change it,
share it.

**Who it is for:** vocational and engineering students, teachers building exercises, and any
workshop that cannot justify a commercial licence. The tool that teaches the subject should
not be the thing a student cannot afford.

**What it is not:** a viewer, a course or a cloud service. No telemetry, no subscription, no
"education edition" with the useful features removed.

> **Continuing this project?** [docs/retomar.md](docs/retomar.md) (Spanish) is the
> continuity note: current state, what was built step by step, how to verify it, the
> decisions taken and the exact prompt to resume in a new session.

## What exists today

| Crate | What it does now |
|---|---|
| `punchi-fluid-core` | Components, ports, connections and typed units (bar, litre, kelvin) |
| `punchi-fluid-catalog` | Component definitions read from TOML files: schema, validation, override rules |
| `punchi-fluid-file` | Versioned JSON document, round-trip and version rejection |
| `punchi-fluid-solver` | Gas mass integration, cylinder with spring, damping and friction, ISO 6358 flow |
| `punchi-fluid-doc` | The edited document: placements, commands and bounded undo |
| `punchi-fluid-draw` | The one drawing of a circuit, in millimetres, behind a `Surface` trait |
| `punchi-fluid-export` | Standard sheets, title block, legend and a deterministic PDF writer |
| `punchi-fluid-ffi` | The C ABI the window speaks: one session, one frame of drawing, one hit test |
| `punchi-fluid-check` | Design checks: the faults a circuit can have before it is run |
| `punchi-fluid-cli` | `punchi-fluid-cli validate|simulate <file>` for headless checks |
| `apps/punchi-fluid-qt` | The window: ribbon, catalog sidebar, sheet, status bar and the desktop's own palette, over the C ABI |

## Build and run

```bash
./scripts-dev/run.sh                                    # window; builds only if stale
./scripts-dev/run.sh --demo                             # starts actuated and playing
./scripts-dev/run.sh --no-build --dark                  # already-built binary, no compile
./scripts-dev/run.sh --light                            # pin the palette for a screenshot
cargo run --bin punchi-fluid-cli -- validate circuit.pfluid # headless check
```

`run.sh` compiles only when a source changed and then executes the binary directly;
`--no-build` never invokes Cargo. `PUNCHI_FLUID_THEME=light|dark` pins the palette,
otherwise the window follows the desktop.

In the window: **Play** runs the simulation, **Slower**/**Faster** change the speed (a
20 mm cylinder with this valve finishes its stroke in about 0.13 s of simulated time),
**Actuate valve** or a click on the valve symbol changes its position, and **Reset**
starts again. `F1` lists every shortcut; `Ctrl+F` focuses the catalog search.

**Export PDF** (`Ctrl+Shift+E`) or **Export PNG** (`Ctrl+Shift+P`) opens the export panel:
format, resolution for PNG (72–600 dpi), paper from A5 to A2 plus Letter and Legal,
orientation, margins, the identification block and the destination. The page carries the
frame, the title block, a legend of line classes and port numerals, and — while any symbol
review is open — the note that the symbols are not yet verified against ISO 1219-1. Two
exports of the same circuit produce the same bytes.

Requirements: Rust stable (pinned in `rust-toolchain.toml`), Qt 6.5 or newer with Qt Quick,
CMake and Ninja. The Rust core builds with `cargo build --release -p punchi-fluid-ffi`; the
window with

```bash
cmake -S apps/punchi-fluid-qt -B apps/punchi-fluid-qt/build -G Ninja
cmake --build apps/punchi-fluid-qt/build
apps/punchi-fluid-qt/build/punchi-fluid
```

The core is linked as a static library, so the result is one binary with no runtime search
path. Keeping it in Rust is what also makes a WebAssembly build possible later.

## Components are files, not code

Every valve, cylinder or supply is a TOML definition in
[assets/catalog/](assets/catalog/). The application compiles those in, and loads
`~/.local/share/punchi-fluid-lab/catalog/` on top, where a file with the same `id`
replaces the built-in one:

```toml
id = "valve.3_2.spring_return"
category = "valve"

[behaviour]
type = "valve"
actuation = "toggle"
spring_return = true

[[behaviour.positions]]
name = "rest"
[[behaviour.positions.connection]]
from = "2"
to = "3"
conductance = 1.0e-9
```

A position lists the paths it opens, classified by the **role** of the ports
(`supply`, `working`, `exhaust`), so a 5/2 valve is a new file and not a new match
arm. The symbol is a list of primitives in the same file. A file with a mistake is
skipped, reported in the Catalog panel, and the rest keeps working. The full field
reference is in [assets/catalog/README.md](assets/catalog/README.md).

## The interface

Three regions and nothing else: a **ribbon** for what applies to the whole document,
a **sidebar that is the catalog** (grouped by category, with each component drawn by
its own symbol, drag to the sheet), and the **sheet** where the circuit lives and
reacts. Status bar at the bottom. Edit and simulation are visibly different modes, so
a stray click during a demonstration never destroys work.

The application takes its colours, accent and font size from the desktop: it reads
`kdeglobals` and follows the `org.freedesktop.appearance color-scheme` portal, so it
looks like the rest of Plasma and follows a live scheme change without restarting.
A semantic token layer is the only place a colour exists — a `Color32::` anywhere else
fails a test. Components use their own symbol, never a separate icon set.

**Honest status:** the regions, the tokens, the keyboard map and the help panel are
implemented. Breeze symbolic icons are not loaded yet (action buttons are text), the
desktop font *family* is read but not loaded (size is), and the UI strings are still
literals until the Fluent pass in M6.

The full specification — regions, keyboard map, contrast requirements, what the design
refuses — is in [docs/ui-ux.md](docs/ui-ux.md). The milestones with their acceptance
criteria are in [docs/plan.md](docs/plan.md).

## Symbols follow ISO 1219

A schematic that is *almost* conformant is worse than an obvious sketch: it gets read
as if it were correct. Diagram symbols follow **ISO 1219-1:2012**, in the conventions
of ISO 1219-2:2012 and ISO 1219-3:2016; port numerals follow the fluid-power
port-identification convention.

What that means in practice, and the review that enforces it, is in
[docs/symbology.md](docs/symbology.md): equal squares per valve position, flow arrows
in the direction of flow, blocking marks on closed ports, port numerals drawn from the
definition, spring and actuator at their ends, line classes for working, pilot and
drain lines, junction dots only where three lines meet.

**Honest status:** the structural half is enforced, not hoped for. A valve declares its
body and the renderer splits it into equal squares; the flow path and its arrow come
from the connection, the blocking mark from the position, the numerals from the port
table, and the spring and actuator from declared parts. Eight lint rules run when the
catalog loads, one test each.

The visual half is reviewed per symbol in
[assets/symbols/review/](assets/symbols/review/) at palette and canvas size. The 300 dpi
check needs the export pipeline, so every review says `status: pending-300dpi` and the
export keeps labelling the document as unverified until they do not. One symbol admits a
weaker shape in its own review: the fixed restrictor was drawn without the standard at
hand.

## Why Rust, and why these crates

The product is three different problems: the physics, the file format and the
schematic editor. Keeping them in separate crates means the solver can be tested
without a window, the format can migrate documents while the editor changes, and
the toolkit is a decision confined to one crate. Today the drawing crosses to the window
as a command buffer over a C ABI (`punchi-fluid-ffi`), and Qt Quick replays it with
`QPainter`.

The full reasoning, including what it costs, is in
[docs/adr/0005-qt-quick-over-a-c-abi.md](docs/adr/0005-qt-quick-over-a-c-abi.md).

## The physical model, in one paragraph

Pneumatics is not computational fluid dynamics. A circuit is a network of volumes
joined by restrictions: the ideal gas law for each volume, ISO 6358 for the flow
through a restriction (with the choked branch above the critical pressure ratio),
and a stiff integrator because pressure changes in milliseconds. Getting the model
right matters more than any UI decision.

## Validation

```bash
./scripts-dev/check.sh
```

Format, lints with `-D warnings`, all tests and a CLI smoke test over a temporary
circuit file. A green run proves the crates and the format; the window and the
physics still need eyes and a reference curve.

## Roadmap

Milestones, acceptance criteria and the evidence each one needs live in
[docs/plan.md](docs/plan.md). In short:

- **v0.1** — workspace, domain, format, canvas, CLI, catalog of definition files.
- **v0.2** — vertical slice: supply, 3/2 valve and single-acting cylinder with gas
  mass integration, piston dynamics and a clickable valve. *Done, in review.*
- **M1** — shell: ribbon, sidebar, status bar, theme from the desktop, accessibility. *Done.*
- **M1.5** — symbol conformance against ISO 1219, with a lint and a review per symbol.
- **M2** — one drawing, two exports: PDF and PNG from the same calls, standard paper,
  margins, title block and legend. *Done.*
- **v0.3** — document model with undo, palette built from the catalog, place and
  connect components, orthogonal pipes, save and load. A placed component will
  reference a catalog `id`, which retires the `ComponentKind` enum from the document
  layer (today it still enumerates the kinds of the first format).
- **v0.4** — network solver driven by the circuit graph instead of one fixed layout;
  5/2 valves, flow controls, gauges.
- **v1.0** — hydraulic model, AppImage and Flatpak, translation pipeline.

## Project documentation

| Document | Purpose |
|---|---|
| [AGENTS.md](AGENTS.md) | canonical rules for people and agents |
| [CHANGELOG.md](CHANGELOG.md) | changes by date, under `Unreleased` |
| [docs/retomar.md](docs/retomar.md) | continuity note: everything needed to resume with no context (Spanish) |
| [docs/next-steps.md](docs/next-steps.md) | where the project stands and the work order for what comes next |
| [docs/ui-ux.md](docs/ui-ux.md) | the interface: regions, theming, accessibility |
| [docs/symbology.md](docs/symbology.md) | ISO 1219 conformance and the review checklist |
| [docs/plan.md](docs/plan.md) | milestones with acceptance criteria |
| [docs/engineering.md](docs/engineering.md) | modularisation, memory budget, security, GitHub practices |
| [CONTRIBUTING.md](CONTRIBUTING.md) | how to set up, test and send a change |
| [SECURITY.md](SECURITY.md) | how to report a vulnerability privately |
| [SUPPORT.md](SUPPORT.md) | where to ask, and what this project does not promise |
| [docs/adr/0001-language-and-layout.md](docs/adr/0001-language-and-layout.md) | the stack and the crate split |
| [docs/adr/0002-components-are-files.md](docs/adr/0002-components-are-files.md) | why components are definition files |
| [assets/catalog/README.md](assets/catalog/README.md) | how to add, change or delete a component |
| [packaging/README.md](packaging/README.md) | how a release will be packaged |
| [i18n/README.md](i18n/README.md) | translation plan |
| [assets/symbols/README.md](assets/symbols/README.md) | symbol set and its licence |
| [assets/screenshots/README.md](assets/screenshots/README.md) | the captures used by this file, and their rules |

## Made with love, for the community

Free software is why a student in a workshop with no budget learns this subject on the same
kind of tool a company uses. The code is yours: read it, teach with it, fork it, and send
back what you fix. The project does not ask for an account, a key or a connection, and it
never will.

**Linux for everyone.**

## AI-assisted development

AI coding agents are a development aid here, not an author. They help implement,
investigate, refactor, document and test, and the maintainer remains responsible for the
technical decisions, the review, the validation and everything published.

Their instructions live with the project — [AGENTS.md](AGENTS.md) and
[`.agents/`](.agents/) — so an agent working on this repository follows the same rules as a
person: typed units, no `unwrap` or `panic!` outside tests, no `unsafe` outside the ABI,
symbols drawn by hand against ISO 1219-1, and **nothing taken from another program**. Those
rules were written and tuned by PunchiSoft for this architecture and this workflow, not
inherited from a template.

Because the program is meant to teach, an agent gets no licence to look finished: it may not
present as done what [docs/capabilities.md](docs/capabilities.md) still marks `missing` or
`partial`, and it may not invent a result to close a task. When something cannot be proven,
it says so — that is the same rule a person follows.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).

The licence applies to human-written and machine-generated copies alike: reading, quoting,
reimplementing or redistributing this project with an agent does not waive the notices, the
corresponding source or the attribution.
