# Architecture

Status: current. This page is the map: what the pieces are, which way the dependencies
point, how a circuit becomes a running model and a printed page, and where a new piece is
allowed to attach. The reasons behind each choice are in the [ADRs](adr/README.md); the
rules for changing the shape are in
[rust-workspace-architecture](../.agents/skills/rust-workspace-architecture/SKILL.md).

## One paragraph

Punchi Fluid Lab is a desktop simulator of pneumatic and hydraulic circuits. The model, the
file format, the drawing and the window are Rust. A circuit is a sheet of placements and
pipes, the catalog says what each placement is, the solver turns the sheet into a network
of volumes and restrictions, and one drawing serves both the screen and the exported page.
There is no network, no telemetry and no server: the program reads files, writes files and
runs.

The product is **one Rust binary** in `apps/punchi-fluid` (ADR 0035), built with egui
(ADR 0036). The C ABI and the Qt Quick window it served are retired; the crate
`punchi-fluid-ffi` still exists in the workspace, has no product consumer and is scheduled
for removal. Nothing in the product speaks C.

## The shape

```text
                    assets/catalog/*.toml        ~/.local/share/punchi-fluid-lab/catalog/
                              │                              │
                              └──────────────┬───────────────┘
                                             ▼
                                         Catalog
                                             │
  user ──▶ apps/punchi-fluid (egui) ──┬──▶ core
                                      └──▶ session ──┬──▶ Document ──▶ check
                                                 │        │
                                                 │        ▼
                                                 │   solver::Network ──▶ telemetry
                                                 │        │
                                                 └──▶ draw::Surface ──▶ frame ──▶ screen
                                                          │
                                                          └──▶ export ──▶ PDF / PNG / SVG

  CLI ──▶ file + catalog + check + solver        (no window)
```

The window links the crates directly: it carries the document through `punchi-fluid-session`,
replays `punchi-fluid-draw` behind `Surface`, and prints through `punchi-fluid-export`.
There is no serialisation between two runtimes and no command buffer to keep in step.

## The crates

`core` is the base. The edges below are the only allowed ones;
`scripts-dev/check-architecture.sh` fails on any other, and the member list in
[AGENTS.md](../AGENTS.md) is checked against `Cargo.toml` by `scripts-dev/check-docs.sh`.

| Crate | Owns | Never contains | Depends on |
|---|---|---|---|
| `punchi-fluid-core` | components, ports, connections, typed units, paper and the sheet frame, the legacy v1 model | numerics, files, widgets | — |
| `punchi-fluid-i18n` | the language, the embedded TOML catalogs and the key lookup | physics, widgets, files, UI | — |
| `punchi-fluid-catalog` | the definition schema, validation, loading, lint, compiled reviews | physics, widgets | `core` |
| `punchi-fluid-file` | the versioned sheet format, atomic write, migrations, recent list | physics, drawing | `core` |
| `punchi-fluid-solver` | ISO 6358 orifice flow, the network of volumes, the cylinder model | editing, drawing, files | `core`, `catalog` |
| `punchi-fluid-doc` | the edited document: placements, pipes, commands, bounded undo, the sheet | physics, drawing, files | `core`, `draw` |
| `punchi-fluid-draw` | ISO 1219 symbols, sheet geometry, the `Surface` trait, hit-testing geometry | windows, files, fonts, solving | `catalog`, `solver` |
| `punchi-fluid-check` | the design faults a sheet can have before it is run | physics, drawing, UI | `catalog`, `doc`, `draw` |
| `punchi-fluid-export` | page geometry, frame, title block, PNG, PDF and SVG writers | physics, widgets | `core`, `catalog`, `draw`, `solver` |
| `punchi-fluid-session` | the document's life between the file and the window: the sheet↔document conversion, the live model, recents, safety copy and templates | physics, widgets | `core`, `catalog`, `doc`, `draw`, `file`, `solver` |
| `punchi-fluid-cli` | `check`, `validate` and `simulate` without a window | UI | `catalog`, `check`, `doc`, `draw`, `file`, `solver` |
| `punchi-fluid-window` | the Rust window: ribbon, sidebar, sheet, status bar, start view, theme | physics, the document | `core`, `catalog`, `doc`, `draw`, `export`, `file`, `i18n`, `session` |

The retired `punchi-fluid-ffi` crate is still in the workspace and still has its C header,
but no product code depends on it. It exists only for its own tests and an untracked spike;
removing it is a named debt in [engineering.md](engineering.md#the-c-abi-retires).

## A circuit, from file to motion

1. **Start.** The window builds a `Catalog` from the compiled-in files and the user folder.
   A malformed file is skipped and reported; the program starts with what is valid. A
   document named on the command line is read through `punchi-fluid-file` and converted by
   `punchi-fluid-session`.
2. **Edit.** The window applies one command to the `Document`; `punchi-fluid-doc` pushes a
   bounded snapshot for undo. The document owns the identity of every placement and the
   route of every pipe.
3. **Run.** `punchi-fluid-session::Model` turns the document into a `solver::Network`: each
   placement becomes a volume, a restriction, a valve or a passive part, and each pipe a
   connection between two ports. A valve position selects its open paths by **port role**,
   not by a Rust type. A definition the solver cannot model is left out of the network and
   drawn uncoloured; a sheet that names a catalog id which is not installed is not yet a
   finding, which is a known gap.
4. **Draw.** `punchi-fluid-draw` walks the placements and pipes and calls `Surface` for every
   primitive. The window replays that recording with egui and the export writers turn it
   into a raster, a PDF or an SVG. A circuit on screen and a circuit printed cannot drift
   apart, because there is one drawing (ADR 0003).
5. **Save.** The sheet is written as JSON in the versioned format (v14) through a temporary
   file and a rename. An older sheet reads with defaults; a newer one is refused with a
   message. The same document remembers when it was last saved, which is what the `•` in the
   title and the safety copy use.
6. **Check.** `punchi-fluid-check` reads the document and the port geometry from the drawing,
   and returns findings with the place they happen. The CLI returns them as text or JSON.
   The Rust window has no check command yet: that is a product gap, not a completed
   capability.

## The three extension points

**1. A component is a file.** Adding a valve, a filter, a cylinder or a gauge is a TOML file
in `assets/catalog/` or in the user catalog. No Rust changes, no new branch in a `match`.
The file declares its ports, its roles, its symbol primitives, its line classes, its
behaviour and its numbers. A new file is a new component. See
[assets/catalog/README.md](../assets/catalog/README.md) and ADR 0002.

**2. A new `behaviour` variant is the exception.** When a part cannot be described with the
existing behaviours, a new variant is added to the catalog schema and to the solver, the
drawing and the check. It needs an ADR, because it is the one place where a component stops
being only a file.

**3. A new device is a new `Surface`.** The drawing speaks millimetres and calls `line`,
`polygon`, `rect`, `circle`, `arc` and `text`. The window implements `Surface` with egui;
the export implements it with `tiny-skia` (PNG), `printpdf` (PDF) and an SVG writer. A new
output is a new implementation, not a second drawing.

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
the recent list and the window state are bounded and written the same way. There is no
automatic network access. The parser still has values that only need to be finite rather
than bounded, and the export's own atomic writer is weaker than the sheet's; both are
recorded debts in [engineering.md](engineering.md#security-and-privacy).

## How the shape is held

| Contract | Check |
|---|---|
| The crates and their edges | `scripts-dev/check-architecture.sh` |
| The documents that describe the crates match `Cargo.toml` | `scripts-dev/check-docs.sh` |
| The drawing on screen and on paper are the same | the caller-drawing and export tests |
| The window behaves without a display | `cargo test -p punchi-fluid-window` (egui_kittest) |
| The model conserves mass and stays finite | the solver invariants and the reference curve |
| A capability is not called done without its evidence | [capabilities.md](capabilities.md) |

## Debts that shape the next phases

- **The legacy v1 model** (`ComponentKind`, `Circuit`) still lives in `punchi-fluid-core`
  and `punchi-fluid-file` for the CLI and the first format. The editor and the sheet use
  format 14; the old path must not grow.
- **The C ABI retires.** `punchi-fluid-ffi` has no product consumer and the Qt window is
  gone. Its 96 tests must be inventoried, the unique ones ported, and the crate, its header
  and the `unsafe` exemption removed (ADR 0035).
- **The Rust window is not feature-complete.** It has no check command, its export does not
  print the parts list, and it has no panel for margins, custom paper or PNG resolution.
  Those are product gaps, not hidden behaviour.
- **i18n is not started.** The window's strings are Spanish literals; there is no catalog
  runtime. The English-first plan needs an ADR after ADR 0035.
- **A project is one document with several circuits** (ADR 0013): a circuit list, project-level
  placements and pipes with a circuit id, and one active sheet.
