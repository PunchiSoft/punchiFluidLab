# ADR 0005 — The sheet is Qt Quick over a C ABI, the model stays Rust

- **Status:** accepted — 2026-11-06, on the evidence in the spike below
- **Date:** 2026-11-06
- **Supersedes:** [0001](0001-language-and-layout.md)
- **Context crates:** `punchi-fluid-ffi`, `punchi-fluid-ui`, `punchi-fluid-desktop`,
  `punchi-fluid-export`, `punchi-fluid-theme`

## Context

ADR 0001 chose Rust for the whole product and kept the toolkit decision open: "egui
today, QML through `cxx-qt` tomorrow, confined to `punchi-fluid-ui` plus a new
`desktop` crate". Two things have changed since.

**The editor is the remaining cost.** M0–M2 are done; M3 and M4 — selection, snapping,
drag and drop, undo, orthogonal pipes, in-place text — are the largest part of the
project that is left, and they are exactly what an immediate-mode toolkit makes you
write yourself. `egui` gives a frame, a painter and hit-testing over its own widgets;
it gives no scene graph, no undo stack, no accessibility tree beyond `accesskit`, and
no text input worth the name.

**The seam already exists.** ADR 0003 built `punchi-fluid-draw` against a six-method
`Surface` trait with no toolkit in it. `EguiSurface` — the entire involvement of egui in
the drawing — is 317 lines. Replacing the front end is therefore a new `Surface`
implementation plus a window, not a rewrite of the drawing, and not a rewrite of the
model.

## Decision

Split by the nature of the problem, not by taste:

| Layer | Language | Owns |
|---|---|---|
| `core`, `catalog`, `file`, `solver`, `draw` | **Rust** | the model, the format, the drawing in millimetres |
| `ffi` | **Rust**, C ABI | one session, one frame of drawing, telemetry, hit test |
| shell, ribbon, sidebar, sheet, dialogs, i18n | **Qt Quick** | everything the user touches |
| export | `QPainter` + `QPdfWriter` + `QImage` | the two backends the Rust export crate implemented by hand |

The drawing crosses the boundary as a **command buffer**, not as callbacks: one call per
frame fills an array of `PfCall` and the front end replays it onto a `QPainter`. The
screen, the PNG and the PDF then consume the same primitive list, which is the promise
`Recording` already tests.

Colour crosses the other way. `PfInk` is filled by the front end from `QPalette`, which
the platform plugin has already assembled from `kdeglobals` and the
`org.freedesktop.appearance color-scheme` portal. Nothing in Rust picks a colour, and
the rule that a `Color32::` outside the theme crate is a bug becomes "a colour literal
outside the front end's theme adapter is a bug".

## Evidence from the spike

`spike/qt-qml/` on branch `spike/qt-qml`, built on 2026-11-06 with Qt 6.11.2, Rust
1.98.1, Fedora 44:

| Claim | Evidence |
|---|---|
| A C ABI can drive the model | `cargo test -p punchi-fluid-ffi`: 7 tests, including a 400-step run that moves the piston and pressurises the cylinder |
| The C and Rust layouts agree | `pf_abi()` returns `sizeof` for every structure; the test asserts them against the Rust types |
| The drawing survives the boundary | the frame carries the primitives and the text runs; a test walks every call and asserts none escaped the area |
| A `QPainter` can draw it | `QT_QPA_PLATFORM=offscreen ... --grab` renders 1100×720 from the same call list |
| The valve responds | `--toggle` goes through the same path as a click: the active square moves, the readouts follow (0 → 5.79 bar, 0 → 100 mm) |
| The theme comes from the desktop | the front end reads `QPalette` and pushes `PfInk`; nothing in Rust knows a colour |
| One binary, no search path | 7.0 MB executable, the Rust core linked as a static library |

What the spike has *not* proven: a real click event end to end (the hit test is a Rust
unit test and the two use one mapping function), Qt Quick Controls in a real ribbon at
the sizes `docs/ui-ux.md` asks for, and frame cost with 200 components.

## Consequences

- **`punchi-fluid-ui`, `punchi-fluid-desktop` and the egui half of `punchi-fluid-theme`
  retire.** `punchi-fluid-export` retires when `QPdfWriter` and `QImage` replace it;
  until then both exist and only one is the product.
- **Two toolchains for the life of the project**: `cargo` for the core, CMake and Qt for
  the front end, in CI, in the Flatpak and in the debugger. The alternative — one
  language — costs the model its guarantees, so this is accepted deliberately.
- **The world does not forbid `unsafe` in `punchi-fluid-ffi`.** The workspace sets
  `forbid`, which a crate cannot relax, so that crate opts out of the inherited lints
  and restates them; `undocumented_unsafe_blocks` is denied in exchange, so every
  pointer has a written justification. The exception is one crate wide and visible.
- **`QQuickPaintedItem` rasterises whole frames on the CPU.** Acceptable for a
  schematic; if the M4 budget (200 components, 4 ms) is missed, the next step is
  `QSGGeometry`, and the `Surface` impl is the only thing that changes.
- **Qt 6 under GPLv3.** The project is GPL-3.0-or-later, so the licence is compatible at
  no cost; a future relicensing to a permissive licence would not be.
- **The reference is still not a fork.** Qt makes it easier to imitate FluidSIM's look;
  the symbols stay hand-drawn against ISO 1219-1 and the reference binary stays a
  source of *behaviour* only.

## What would reverse this

If the spike's timing is misleading — if a real ribbon, a real 200-component frame and a
real text field cost more than the `egui` versions did — then the honest conclusion is
that the toolkit was never the problem, and the fallback is to return to `main` and keep
`egui`. That is why this ADR is *proposed* and why the spike lives on a branch.
