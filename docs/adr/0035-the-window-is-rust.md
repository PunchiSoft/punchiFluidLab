# ADR 0035 — The window is Rust, with a fixed layout

- **Status:** proposed
- **Date:** 2026-11-21
- **Deciders:** project owner, implementation session
- **Context crates:** `punchi-fluid-draw`, `punchi-fluid-export`, `punchi-fluid-doc`, `punchi-fluid-ffi` (retires)

## Context

Since [ADR 0005](0005-qt-quick-over-a-c-abi.md) the product is split in two: a Rust model
behind a C ABI and a Qt Quick front end that owns everything the user touches. That split
bought a real editor, and it costs two toolchains for the life of the project — the record
says so itself.

Three things have changed.

**The seam is complete on the Rust side.** The drawing is one recording behind the
`Surface` trait ([ADR 0003](0003-one-drawing-two-devices.md)), and the PNG, the PDF and the
SVG surfaces are already Rust (`punchi-fluid-export`). A window that consumes the same
recording needs no C ABI and no `QPainter`; the reason the ABI existed — carrying a command
buffer across two runtimes — disappears with the second runtime.

**The Qt dependency keeps asking to be installed.** The window needs development packages
and runtime image plugins that a machine may not have; this session found the SVG format
plugin present and its development package absent, which is precisely the failure
[ADR 0028](0028-the-ribbon-draws-its-own-icons.md) wrote down when it rejected asset files.
A missing plugin does not fail the build; it blanks an icon.

**The owner has decided.** On 2026-11-21 the project owner directed that the whole product
migrates to Rust, that the interface stops being adaptive and becomes fixed, and that the
iconography is a separate matter from the ISO 1219 symbology, which is not touched.

## Decision

**The product is one Rust binary.** The window, its regions and its input path are Rust
crates; `punchi-fluid-ffi` and `apps/punchi-fluid-qt` retire, and no C ABI is shipped.

**The layout is fixed.** One layout, authored at the support floor of
[ui-ux.md](../ui-ux.md#1-ribbon-top) — 1366×768 at the default scale — that fills a larger
window with the same regions and the same rows. There is no fold, no mode chosen from the
room, and no responsive threshold: below the floor the window has a minimum size and refuses
the size instead of rearranging itself. The adaptation section of the contract
([§2.2](../ui-ux.md#22-the-window-adapts)) is superseded by this paragraph for the window,
and a fixed interface is a simplification the owner chose, not an accident of the toolkit.

**The drawing is the existing one.** The window replays `punchi-fluid-draw`'s recording; it
draws no symbol of its own, and the ISO 1219 geometry, `assets/symbols` and the symbology
review are untouched by this record.

**The toolkit is chosen by evidence, in a second record.** Candidates: `egui` (fastest to a
window, immediate mode, the option ADR 0005 rejected) and `slint` (declarative, fixed
layouts, retained state). The spike measures five things before the choice is written:
in-place text editing with IME, the accessibility tree and keyboard order, one 16 px SVG
icon tinted by the theme, a 200-component frame's cost, and a printed page from the Rust
PDF surface. A toolbox that fails one of the five is not chosen.

**The iconography is files, and the symbology is not.** Action icons become SVG files under
`assets/icons` (the decision and its rules are their own record); a valve, a cylinder or a
port keeps its ISO 1219 drawing in `punchi-fluid-draw` and its review record, and neither
surface borrows from the other.

## Consequences

- **Two toolchains become one.** CMake, Qt, the QML checks and the C ABI leave CI; a build
  needs `cargo` and, at most, the system libraries the chosen toolkit links.
- **`punchi-fluid-ffi`, `apps/punchi-fluid-qt` and their tests retire.** The ABI tests,
  `--check-qml`, `--check-theme` and `--selftest` are replaced by their Rust equivalents in
  the same change that deletes them; a check is not deleted before its replacement is green.
- **`punchi-fluid-export` stays and gains the print path.** It already writes PNG, PDF and
  SVG from the same surface; the window stops being the printer.
- **Fixed layout is a smaller window, not a responsive one.** `ui-ux.md`'s adaptation
  sections and the nodes that paid for them (P-027, P-028) close as obsolete, and the
  minimum size becomes a hard rule instead of a measurement.
- **The migration is a debt with phases.** Phases: (1) the ADR and the icon files; (2) the
  toolkit spike and its record; (3) the window with the sheet, the ribbon and the status bar;
  (4) the dialogs, the start view and the theme; (5) the retirement of Qt and the ABI. Each
  phase is one branch and one pull request, and the product keeps working at the end of each.
- **Accessibility, IME and text editing are the named risk.** They are the reason ADR 0005
  chose Qt; the spike measures them first, and if they are not met the migration stops with
  the evidence written down.

## Alternatives considered

- **Stay on Qt.** It works and it is the status quo, but it keeps two toolchains and a
  runtime plugin surface that can blank an icon without failing a build; the owner decided
  against it on 2026-11-21.
- **`cxx-qt` instead of the C ABI.** It removes the hand-written ABI while keeping Qt and
  QML; it still leaves two languages, the plugin surface and the Qt packages, so it does not
  answer the decision.
- **Keep the ABI and write only a Rust shell.** The ABI becomes a boundary between two
  Rust runtimes with a serialisation between them; the seam that already exists
  (`Surface`) makes the ABI unnecessary rather than preservable.
- **A custom `winit` + `wgpu` window.** It maximises control and reimplements text, IME and
  accessibility from zero; not for a project that must teach a subject.

## References

- [ADR 0003](0003-one-drawing-two-devices.md): one drawing behind a `Surface`.
- [ADR 0005](0005-qt-quick-over-a-c-abi.md), superseded here; its spike is the evidence for
  what Qt bought and what the migration must replace.
- [ADR 0028](0028-the-ribbon-draws-its-own-icons.md): the family and the "no asset files"
  rule the icon record supersedes for action icons only.
- [ADR 0009](0009-identity-and-standards.md): the symbols are ours and the standards rule;
  this record touches neither.
- The owner's direction of 2026-11-21: one Rust binary, a fixed interface, and the
  symbology untouched.
