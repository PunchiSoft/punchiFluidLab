# The Rust migration, in four batches

- **Status:** proposed
- **Date:** 2026-11-22
- **Depends on:** [ADR 0035](adr/0035-the-window-is-rust.md) (the window is Rust) and
  [ADR 0036](adr/0036-the-toolkit-is-egui.md) (the toolkit is egui)
- **Tracks:** [docs/plan.md](plan.md) and [docs/capabilities.md](capabilities.md)

## Why batches

The Qt window is the product until phase 5 (ADR 0035). The Rust window in `apps/punchi-fluid`
already has the shell, the editing commands and the simulation core with the live scene. The
remaining work is grouped into **four large batches**; each leaves the Rust window coherent
and is delivered as one pull request, or a very small number, not one per control. The
capability states stay in [docs/capabilities.md](capabilities.md); this document is the order.

The rule for every batch: the Rust window builds, runs and is driven by `--selftest` (or the
equivalent headless check) at the end of the batch, and nothing in it depends on the Qt app.

## Batch 1 — The Rust window is the product

The batch that ends the migration: the window opens in Rust, does the student's whole loop,
and the Qt app leaves the workspace and the CI.

**Scope**

- **Inicio**: the four start templates (the first is the owner's OR circuit, already composed
  in the ABI), the recent documents and the welcome page, all in the Rust window.
- **Files**: Save, Save As, Open, the recent list, autosave and recovery, the close
  confirmation and the window state, with the Rust window's own dialogs.
- **Retire the Qt app**: remove `apps/punchi-fluid-qt`, its start page and its selftest, and
  the `window` job in CI; the ABI stays only where its tests use it.

**Acceptance**

- The student's path is covered by the Rust window's headless selftest: Inicio → OR template
  → edit → save → reopen → run → export.
- `apps/punchi-fluid-qt` is gone, the `window` job is gone, and the workspace still passes the
  full gate.
- The OR template of the parked pull request is what Inicio offers; the template lives in the
  session, so the Rust window reads it without the Qt.

**Risks:** the native file dialog in the Rust window; the recent list and the autosave timing;
the ABI surface that the Qt tests keep alive.

## Batch 2 — Interface parity and F8

The batch that makes the Rust window feel like the product, not a port.

**Scope**

- **Panels and ribbon**: the catalog panel, the properties/right dock, folding and remembered
  widths, the ribbon's depth and flyouts, the whole icon family, the adaptive layout.
- **Accessibility**: names, contrast, reduced motion, keyboard order.
- **F8**: saved configurations, the Spanish/English translation catalogs, packaging.

**Acceptance**

- The Rust window's `--check-*` checks cover what the Qt window's did; the capability rows of
  [docs/capabilities.md](capabilities.md) move to `covered` with their tests.
- A screen reader reads the same names, and the theme check measures the same contrast
  criteria.

**Risks:** the icon family is a lot of files; the adaptive layout has one breakpoint today and
needs the staged plan; the i18n move changes every user-facing string.

## Batch 3 — The content gaps

The batch that fills the library and the physics, independent of the window.

**Scope**

- The hydraulic model; the rest of the electrical, digital and GRAFCET libraries; FRL and
  compressors; cylinders with cushioning and magnet; port designations; the 300 dpi symbol
  reviews.

**Acceptance**

- Each capability row carries its reference test and its conformance review; the solver's
  invariants and reference curves are stated per model.

**Risks:** the hydraulic model is a new physics domain; the libraries are many files; the
reviews need a second reader.

## Batch 4 — 1.0

The release batch.

**Scope**

- Packaging (AppImage, Flatpak, Windows), the documentation, the version, the final audit and
  the Qt remnants.

**Acceptance**

- The release checklist is green: the packaged app starts on a clean machine, the gate runs on
  the release, and no document links a retired window.

## What each batch does not include

- A batch does not add a capability that belongs to a later batch; the capability plan stays
  the authority for the *what*.
- A batch does not reopen a decision: a change of window, toolkit or format needs its own ADR.
- Working notes stay private: the migration's state and the remaining queue live in the
  owner's untracked notes; this document records only the public order and its evidence.
