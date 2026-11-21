# 0031 — The program opens on a start view, and a new project is empty

- **Status:** accepted
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

## How it lands

The fixture stays useful without leaking into the product, which decides the first change:

- **`pf_session_new` creates an empty document**: frame, one circuit, no components. The
  window, the CLI's future editor and any host start from nothing.
- **A new entry point seeds the worked example on request** — `pf_session_load_example` — and
  nothing in the product calls it. The ABI tests' session helper calls it, which keeps the
  fixture those tests were written around, and the example **template** of `Inicio` will call
  it too (or copy a compiled document, whichever the template work finds simpler).
- **The `--selftest` asks for the example explicitly** before its circuit steps, so the battery
  that says "the example has a valve and a cylinder" keeps proving the same gestures.
- **The empty sheet's invitation** is part of the same change, because a blank page with no
  sentence is the state the reader meets first.

Then the start view (the page and its rail) and finally the templates with their thumbnails.

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

## Amendment (2026-11-19, first implementation slice)

The product session now starts with an empty document. The ABI's `pf_session_new` also starts
empty; `pf_session_load_example` is an explicit fixture/template entry point. The ABI test
helper requests that fixture for tests that exercise the worked circuit, and the window's
`--selftest` checks that startup is empty before asking for it. The first-frame renderer and
export no longer seed an example implicitly. The empty-sheet message is present. The `Inicio`
view, its rail, recents and thumbnails remain the next slices; until then the blank-sheet
message names only the catalog, which is already available. Pressure, stroke, velocity and
time readings stay hidden when there is no model, so the empty sheet does not display a
meaningless negative pressure.

- ABI: `a_new_document_is_an_empty_sheet_and_the_example_loads_on_request`.
- Window: `startup: the window opens with an empty sheet`, `startup: the empty sheet says what
  to do`, `startup: model readings stay hidden without a model`, and
  `startup: the example loads on request`.
- Full gate still required before delivery.

## Amendment (2026-11-21, the visual pass P-034a)

The first slice drew Inicio; P-034a makes the page tell the truth about what it offers and
what it stores. This amendment supersedes the parts of the decision that had not held:

- **The ribbon and the status bar are hidden over Inicio**, not reduced to Proyecto and
  Ayuda. A page is not a document: hiding them is what lets the page fill the window down to
  the 1366×768 support floor. The region arithmetic subtracts a bar only while it is visible;
  a hidden bar that still took height left the page about 160 px short and clipped the recent
  rows.
- **The rail offers what exists**: Iniciar (Inicio, Nuevo proyecto, Abrir…) and Aprender
  (Acerca de, Licencia). **Atajos** and **ISO 1219** need a panel and content and are not
  invented here.
- **Three templates are offered, not four.** The compiled `ElectricalControl` template stays
  in the ABI (`PF_TEMPLATE_COUNT` remains four) and its card is hidden until **P-026** lands;
  removing it would change the ABI for a card that returns. The QML list has three entries,
  and a window check asserts the electrical card is absent.
- **The recent list is `recent.json`, not `QSettings`.** `punchi-fluid-file` owns the format
  (`RECENT_VERSION = 1`, paths and names only, `MAX_RECENT = 10`, `MAX_BYTES = 64 KiB`,
  atomic write); the window passes
  `$XDG_DATA_HOME/PunchiSoft/Punchi Fluid Lab/recent.json` to the core. The page shows five
  (`PF_RECENT_PAGE_LIMIT`) and sends the rest to **Archivo ▸ Recientes**.
- **The rows tell the truth with what is available now**: modified time and real size from
  `QFileInfo` on at most five paths, never an invented opened time and never a file parse. A
  path that is gone says **no encontrado** and keeps a **quitar** that forgets the entry and
  never deletes the file. The empty list says the next step.
- **The recent thumbnail and the opened time are P-034b**, not this pass: they need a
  preview-by-path entry (a parse of an untrusted file, with a bound, a message and a
  one-past-the-limit test) and a `recent.json` format change with a migration.
- **The return is `Volver a la hoja`**, and `Esc` leaves the page before it touches anything
  behind it.

Evidence: `--check-layout` at 1280, 1366, 1440 and 1600, with the page measured against the
window; `--check-qml`, `--check-theme` and `--selftest`; light/dark captures at 1366, a
missing row and an empty page in `specs/evidence/`. No ABI, diagram, symbol, ISO 1219, sheet,
zoom or active-fill change.

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
