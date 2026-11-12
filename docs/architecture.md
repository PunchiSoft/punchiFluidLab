# Architecture

Status: current. This page is the map: what the pieces are, which way the dependencies
point, how a circuit becomes a running model and a printed page, and where a new piece is
allowed to attach. The reasons behind each choice are in the [ADRs](adr/README.md); the
rules for changing the shape are in
[rust-workspace-architecture](../.agents/skills/rust-workspace-architecture/SKILL.md).

## One paragraph

Punchi Fluid Lab is a desktop simulator of pneumatic and hydraulic circuits. The model, the
file format and the drawing are Rust; the window is Qt Quick over a C ABI. A circuit is a
sheet of placements and pipes, the catalog says what each placement is, the solver turns
the sheet into a network of volumes and restrictions, and one drawing serves both the
screen and the exported page. There is no network, no telemetry and no server: the program
reads files, writes files and runs.

## The shape

```text
                    assets/catalog/*.toml        ~/.local/share/punchi-fluid-lab/catalog/
                              │                              │
                              └──────────────┬───────────────┘
                                             ▼
                                        Catalog
                                             │
  user ──▶ Qt Quick window ──▶ C ABI ──▶ PfSession ──┬──▶ Document ──▶ check
             (apps/…/qt)      (punchi-…-ffi)          │        │
                                                      │        ▼
                                                      │   solver::Network ──▶ telemetry
                                                      │        │
                                                      └──▶ draw::Surface ──▶ frame ──▶ screen
                                                                   │
                                                                   └──▶ export ──▶ PDF / PNG

  CLI ──▶ file + catalog + check + solver        (no window, no Qt)
```

The window never links a Rust crate directly: it links the static library built from
`punchi-fluid-ffi` and calls the functions in `include/punchi_fluid.h`. That is why the
window can be replaced without touching the model, and why a test can drive the same
session without a display (ADR 0005).

## The crates

`core` is the base. The edges below are the only allowed ones; `scripts-dev/check-architecture.sh`
fails on any other, and the member list in [AGENTS.md](../AGENTS.md) is checked against
`Cargo.toml` by `scripts-dev/check-docs.sh`.

| Crate | Owns | Never contains | Depends on |
|---|---|---|---|
| `punchi-fluid-core` | components, ports, connections, typed units, the legacy v1 model | numerics, files, widgets | — |
| `punchi-fluid-catalog` | the definition schema, validation, loading, lint, compiled reviews | physics, widgets | `core` |
| `punchi-fluid-file` | the versioned sheet format, atomic write, migrations, recent list | physics, drawing | `core` |
| `punchi-fluid-solver` | ISO 6358 orifice flow, the network of volumes, the cylinder model | editing, drawing, files | `core`, `catalog` |
| `punchi-fluid-doc` | the edited document: placements, pipes, commands, bounded undo | physics, drawing, files | `draw` |
| `punchi-fluid-draw` | ISO 1219 symbols, sheet geometry, the `Surface` trait, hit-testing geometry | windows, files, fonts, solving | `core`, `catalog`, `solver` |
| `punchi-fluid-check` | the design faults a sheet can have before it is run | physics, drawing, UI | `catalog`, `doc`, `draw` |
| `punchi-fluid-export` | page geometry, frame, title block, legend, PNG and PDF writers | physics, widgets | `catalog`, `draw`, `solver` |
| `punchi-fluid-ffi` | the C ABI: one session, one frame, hit tests, layout, export | logic of its own | all of the above |
| `punchi-fluid-cli` | `check`, `validate` and `simulate` without a window | UI | `core`, `file`, `solver`, `catalog`, `check`, `doc`, `draw` |

The window (`apps/punchi-fluid-qt`) is outside Cargo. It owns the ribbon, the sidebar, the
sheet widget, the status bar, the theme adapter, the icon provider and the QML. It never
contains a formula or a catalog id: the catalog comes through the ABI and the palette comes
from the theme adapter. A few pixel sizes are still literals in QML; moving them into the
adapter is a declared debt, not a finished rule.

## A circuit, from file to motion

1. **Start.** `punchi-fluid-ffi` builds a `Catalog` from the compiled-in files and the
   user folder. A malformed file is skipped and reported; the program starts with what is
   valid.
2. **Edit.** The window sends commands (`pf_session_place_component`, `move_selection`,
   `rotate_selected`, `set_component_designation`, …). `punchi-fluid-doc` applies one
   command as one change and pushes a bounded snapshot for undo. The document owns the
   identity of every placement and the route of every pipe.
3. **Run.** `rebuild_model` turns the document into `solver::Network`: each placement
   becomes a volume, a restriction, a valve or a passive part, and each pipe becomes a
   connection between two ports. A valve position selects a set of open paths by **port
   role**, not by a Rust type. A definition the solver cannot model is left out of the
   network and drawn uncoloured; a sheet that names a catalog id which is not installed is
   not yet a finding, which is a known gap.
4. **Draw.** `punchi-fluid-draw` walks the placements and pipes and calls `Surface` for
   every primitive. The same call list is replayed by the window with `QPainter` and by the
   export writers into a raster or a PDF. A circuit that is on screen and a circuit that is
   printed cannot drift apart, because there is one drawing (ADR 0003).
5. **Save.** The sheet is written as JSON in the versioned format (v3 today) through a
   temporary file and a rename. An older sheet reads with defaults; a newer one is refused
   with a message. The same document remembers when it was last saved, which is what the
   `•` in the title and the safety copy use.
6. **Check.** `punchi-fluid-check` reads the document and the port geometry from the
   drawing, and returns findings with the place they happen. The window draws a marker
   there; the CLI returns the same findings as text or JSON (ADR 0010, ADR 0011).

## The three extension points

**1. A component is a file.** Adding a valve, a filter, a cylinder or a gauge is a TOML file
in `assets/catalog/` or in the user catalog. No Rust changes, no new branch in a `match`.
The file declares its ports, its roles, its symbol primitives, its line classes, its
behaviour and its numbers. A new file is a new component. See
[assets/catalog/README.md](../assets/catalog/README.md) and ADR 0002.

**2. A new `behaviour` variant is the exception.** When a part cannot be described with the
existing behaviours, a new variant is added to the catalog schema and to the solver, the
drawing and the check. It needs an ADR, because it is the one place where a component stops
being only a file. The gauge was added this way (ADR 0008); the electrical library will be
the next.

**3. A new device is a new `Surface`.** The drawing speaks millimetres and calls `line`,
`polygon`, `rect`, `circle`, `arc` and `text`. The window implements `Surface` with
`QPainter`; the export implements it with `tiny-skia` (PNG) and `printpdf` (PDF). A new
output — SVG, a plotter, a test recorder — is a new implementation, not a second drawing.

## Units, time and memory

- **Units are typed.** A pressure, a temperature or a volume crosses a crate boundary as
  `Pressure`, `Temperature` or `Volume`, never as a bare `f64`. The model stores SI
  internally and shows bar and millimetres at the edge. The drawing API is millimetres
  (ADR 0003).
- **Time is substepped.** The solver integrates gas mass as the state and derives pressure
  from the ideal gas law, so pressure cannot go negative and the mass balance is exact by
  construction. One call substeps at 2 µs with a bounded maximum, so a frame cannot lock
  the interface. The valve flow is ISO 6358, including the choked branch.
- **The hot path is written to allocate nothing.** The solver reuses its buffers after the
  first step and the draw loop reuses the call buffer; the budget and how it is meant to be
  measured are in [engineering.md](engineering.md).

## Persistence and trust

Sheet files and catalog files are untrusted input. The format rejects an unknown version,
the parser is bounded, and a catalog file that breaks a rule is skipped with a reason
instead of taking the program down. The save path is a temporary file plus a rename, and
the recent list is bounded and written the same way. There is no automatic network access
and no place where a file makes the loader allocate without a bound.

## How the shape is held

| Contract | Check |
|---|---|
| The crates and their edges | `scripts-dev/check-architecture.sh` |
| The documents that describe the crates match `Cargo.toml` | `scripts-dev/check-docs.sh` |
| The drawing on screen and on paper are the same | the caller-drawing and export tests |
| The window behaves without a display | `--check-qml`, `--check-theme`, `--check-icons`, `--selftest` |
| The model conserves mass and stays finite | the solver invariants and the reference curve |
| A capability is not called done without its evidence | [capabilities.md](capabilities.md) |

## Debts that shape the next phases

- **The legacy v1 model** (`ComponentKind`, `Circuit`) still lives in `punchi-fluid-core`
  and `punchi-fluid-file` for the CLI and the first format. The editor and the sheet use
  format v3; the old path must not grow.
- **The ABI owns logic that belongs to the document** in a few places (the starting scene,
  the export composition). It is the one crate allowed to know both, but it is also the
  file over the line budget; splitting it is a named debt in
  `scripts-dev/check-architecture.sh`.
- **The export panel is incomplete**: margins, custom size and PNG resolution have no field
  in the window, and the last paper choice is not remembered. The date and the fit are done;
  the panel is recorded in the ledger.
- **The project of circuits (P1.8)** is the change that will give the document more than
  one sheet; it is the first structural change after this map and will need its own ADR if
  it changes the file format or the ABI.
