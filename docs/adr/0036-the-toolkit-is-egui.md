# 0036 — The window's toolkit is egui

- **Status:** proposed
- **Date:** 2026-11-22
- **Deciders:** project owner, implementation session
- **Context crates:** `punchi-fluid-draw`, `punchi-fluid-export`, `punchi-fluid-doc`

## Context

[ADR 0035](0035-the-window-is-rust.md) directed one Rust binary with a fixed layout and
named two candidates, egui and Slint, with five measurements before the choice: in-place
text with IME, the accessibility tree and keyboard order, a themed 16/24 px icon, the cost
of a 200-component frame, and a printed page from the Rust PDF surface. A toolbox that fails
one of the five is not chosen.

The spike measured both against the same document — an A2 sheet with 200 components, 180
square symbols of eight built-in definitions and 20 cylinders — the same drawing and the
same machine. On the GPU, egui turned the drawing into 4 503 shapes in 0.67 ms and
tessellated them in 5.15 ms; the window ran at 88–89 fps with 9.98 ms of CPU, and the
drawing itself is 0.85 ms of that. Slint's first declarative integration produced 1 866
paths and 959 text runs and ran at 29 fps with the text and 36 fps without it. The frame
cost is the toolkit's own pass, not the paint: 5.8 ms for egui at 200 components.

The other four: egui composes á through a dead key and ñ through its IME events, takes the
text field, the action icons and the sheet into the accessibility tree (the frame node is
unnamed by eframe's adapter), embeds and tints the SVG family, and prints the page
byte-identical to the product — 470 089 bytes, three writers — from the same document.
Slint's accessibility tree is better at the frame and leaks the page's text runs as
hundreds of labels, and it shares the same print path.

## Decision

The window's toolkit is **egui**, behind eframe. The five measurements are its acceptance
and are recorded rather than re-derived: the frame budget is 60 fps at 200 components with
one text run per port numeral, and egui meets it on the reference machine while Slint's
measured integration does not.

The rules that follow:

- No Slint dependency enters the workspace. The migration phases of ADR 0035 continue with
  egui and the C ABI retires as that record says.
- The window replays `punchi-fluid-draw`'s recording; it draws no symbol of its own.
- The action icons are the SVG files under `assets/icons`, loaded by the window and tinted
  by its theme; no other icon set is added.
- The page is printed by `punchi-fluid-export`; the window is not a printer.
- A change that moves the frame budget states the measurement at 200 components with the
  text runs, gives the drawing and the tessellation, and names the GPU.
- The frame node's missing accessible name is a debt of eframe's adapter to close in the
  window; it is not a reason to change the toolkit.
- The IME pipe through a real input method on Wayland remains to be driven; the toolkit's
  composition handling is proven and is not re-opened by this record.

## Consequences

- The ribbon, the dialogs, the start view and the theme are written in egui. The ribbon
  prototype with the five tabs, the large transport commands and the per-group collapse at
  1366 is the starting point for the phase-3 window.
- The Qt window and the C ABI keep the product working until the phase that replaces their
  checks; a check is not deleted before its replacement is green (ADR 0035).
- Slint is not evaluated further. Its accessibility result stays as evidence that a
  declarative toolkit can name the frame, which the egui window must reach another way.
- The frame node's name and the real-IME check are debts, closed with the window and
  measured the same way.
- The spike is evidence and never merges; the numbers in this record are the durable copy,
  and the commands that produced them are kept with the spike.

## Alternatives considered

- **Slint.** Better accessibility at the frame with no code, but it failed the 200-component
  frame at 29 fps with the text and 36 without, on the same GPU where egui ran at 88–89.
  A different Slint integration — fewer items, text batched differently — was not the one
  measured, and the criterion is the integration the project would write.
- **Stay on Qt.** Superseded by ADR 0035: two toolchains and a runtime plugin surface that
  can blank an icon without failing a build.
- **A custom `winit` + `wgpu` window.** Rejected in ADR 0035: it reimplements text, IME and
  accessibility from zero.

## References

- [ADR 0035](0035-the-window-is-rust.md): the window is Rust, with a fixed layout.
- [ADR 0003](0003-one-drawing-two-devices.md): one drawing behind a `Surface`.
- The measurement on `spike/rust-ui` (2026-11-21/22): `--measure 200`, `--parity`,
  `--seconds 5`; the Slint candidate's `SLINT_DEBUG_PERFORMANCE` runs. The spike never
  merges.
