# 0031 — The program opens on a start view, and a new project is empty

- **Status:** proposed
- **Date:** 2026-11-19
- **Deciders:** project owner, implementation session
- **Context crates:** `punchi-fluid-ffi`, `apps/punchi-fluid-qt`

## Context

The window opens with a circuit already drawn: a compressor, a service unit, a valve and a
cylinder, seeded by the session before the first frame. It began as a demonstration of the
model, and it is useful as a **test fixture** — a circuit with every element lets a test drive
the solver, the check and the gestures. As a product it teaches the wrong first lesson: that
the program comes with a circuit, that a student edits somebody else's example instead of
starting one, and that the saved file is whatever was on screen.

The owner has asked for what the references do: a **start screen** (Inventor, AutoCAD, the
sibling CAD in the reference set) that offers a new project, an open one, the recent ones and
a few ready-made examples, each with a picture. The default circuit is not deleted: it becomes
one of those examples, chosen on purpose, while the fixture stays internal.

Two facts today:

- `ensure_seeded` in the ABI draws the example into every new session, and the export and the
  `--selftest` use it as their circuit.
- A circuit with no components says "No hay un circuito que dibujar", which names the state
  and not the next step.

## Decision

- **The program opens on a start view, `Inicio`**, drawn in the drawing region — not a fourth
  region and not a ribbon tab — with the ribbon reduced to **Proyecto** and **Ayuda** while no
  project is open. The start view covers the sheet the way a start screen covers the viewport
  in the references.
- **The left region carries the start navigation** while the start view is shown: the
  catalog's place is taken by a rail with **Iniciar**, **Nuevo**, **Abrir** and **Aprender**
  (shortcuts, ISO 1219, About, licence), the region the window already has instead of a second
  sidebar. The right dock is hidden: with no project it has no inspector and no block.
- **`Inicio` is reachable at any time** from **Archivo ▸ Inicio**: with a project open, the
  view covers the sheet and **Volver** or `Esc` returns to it, as the references do; with no
  project it is the only view. The commands that need a document are not shown greyed: they
  appear when a project is open.
- **A new project is an empty sheet**: A4 landscape with the ISO frame, **no components**, and
  the sheet says what to do: *Arrastra símbolos a la hoja para empezar*, with the catalog and
  the start view as the two ways in. A new project is never a copy of the example.
- **The seeded example becomes a template.** `Inicio` offers four of our own circuits, each
  drawn by our renderer as a thumbnail: a 3/2 valve with a single-acting cylinder, a 5/2 valve
  with a double-acting one, a service unit feeding a valve, and an electrical control (rail,
  button, relay, coil). Choosing one copies it into a new project; choosing **Nuevo** starts
  empty. The fixture remains in the tests and is never shown to the reader.
- **The start view offers what a student needs to begin**: Nuevo, Abrir, the **recent
  projects** with a thumbnail, name and folder (a path that is gone stays in the list, greyed,
  with *quitar*), the templates, and **Aprender** — the shortcuts, what ISO 1219 conformance
  means, and the About panel.
- **Closing the last project returns to `Inicio`**, after the confirmation when the project has
  unsaved work; closing the window from `Inicio` does not ask, because nothing is lost there.
- **Everything is local**: templates are compiled in, thumbnails are rendered by the same
  drawing, bounded and cached, and the recent list is view state in `QSettings`. No network,
  no telemetry, no auto-open of the last file.

## Consequences

- The ABI stops seeding on creation; the template is loaded explicitly. The tests and
  `--selftest` steps that depend on "the example has a valve and a cylinder" must build their
  circuit, which is the risky part and gets its own change.
- The empty sheet needs its message and the keyboard path through the catalog, so a reader
  without a pointer can start too.
- The start view is interface, not a capability of the model: no format change, no new field.
- The export keeps working on whatever the project holds; a template is copied into the
  document like any other circuit.
- The ribbon, the panels and the status bar hide what cannot act on an empty project; a tab
  whose commands cannot run is not shown.

## Alternatives considered

- **Keep seeding the example and offer the start view after it.** The first lesson stays
  "somebody else's circuit"; closing it to start your own is a step nobody should need.
- **A start window separate from the main window.** A second window needs its own layout,
  theme, accessibility tree and close behaviour, and the references put the start view inside
  the one window.
- **A ribbon tab called Inicio.** The ribbon holds commands; a tab would compete with Archivo,
  and the start view must be reachable with no project, when the editing tabs have nothing to
  offer.
- **Deleting the example.** It is the fixture every test and capture uses, and the best
  teaching circuit we have; it becomes a template instead.

## References

- The reference CAD's start page, read as behaviour: new, open, recents and built-in samples
  with a picture, over the viewport, closed when a document opens. No text, picture or code
  taken from it.
- [ui-ux.md](../ui-ux.md) §2.3 (the start view) and §3 (the empty sheet).
- [ADR 0026](0026-the-sheet-is-the-paper.md): the sheet is the paper and what it holds.
