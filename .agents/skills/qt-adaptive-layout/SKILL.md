---
name: qt-adaptive-layout
description: Choose and prove the window's adaptive layout in QML: breakpoints on available room, Layout versus anchors, per-panel minimum and preferred widths, and the size and DPI test matrix. Use when resizing the shell, adding a panel or a view strip, or proving the adaptive stage.
license: GPL-3.0-or-later
---

# Adaptive layout: the room decides, the content sets the limits

The window has to work on a 1280 px laptop and on a 1920 px desk, at 100 % and at 200 %,
with the catalog open or folded, without the sheet disappearing. The rules that make that
possible are few, and the trap is always the same: a number that belonged to one screen.

The contract lives in [docs/ui-ux.md](../../../docs/ui-ux.md) §2.2 and the stage in
[docs/interface-plan.md](../../../docs/interface-plan.md) 6a. This skill is the procedure,
not a second copy of the contract.

## The rule

**A region declares its limits; the room decides the mode.** The breakpoint is measured on
the space the shell actually has — the window's content width after the frame, in
device-independent pixels — never on a resolution remembered from a screenshot. The
sheet's minimum outranks a panel's preferred width: the panel narrows, then becomes a rail,
then a drawer, and the drawing is never the region that gives way.

**Layouts or anchors, never both in the same item.** A region placed by a `RowLayout`,
`ColumnLayout` or `GridLayout` does not set `x`, `y`, `width` or `height`, and an item that
is anchored does not declare `Layout.*`. Mixing the two produces geometry conflicts and
binding cycles, and the window is the worst place to debug one.

## Procedure

1. **Measure before choosing.** Write the number down: the sheet floor in millimetres at
   the smallest readable zoom, the catalog's usable width with two tile columns, the right
   dock's with its narrowest editor. Convert each to a content minimum in logical pixels.
   Those three numbers are the layout; everything else follows from them.
2. **Give every region a minimum and a preferred width** and let one layout own the row.
   The mode is not a stack of `if` in QML: it is the consequence of the minimums, so a new
   panel does not need a new breakpoint.
3. **Choose the mode from the available room**: wide when all preferred widths fit, medium
   when only the minimums do, compact when they do not. A region that cannot fit becomes a
   rail or an overlay with a handle, never a squeezed version of itself.
4. **Fold by rule, not by hand.** The view strip, the ribbon and the docks fold at the same
   thresholds; the fold is a state of the mode and stays remembered as view state, never as
   document state.
5. **Let heights follow the font.** A row height, a control size and an icon size are
   derived from the spacing unit and the font, so a desktop with a larger font does not
   clip the chrome. No 34, 44 or 62 in a widget.
6. **Prove it at sizes and at scales.** Resize headlessly, capture, and measure; then repeat
   at another device pixel ratio. A layout that only passes at 100 % is not done.

## Gotchas

- **A fixed width is not wrong; an unexplained one is.** The catalog's 220 px minimum is a
  measurement. The 248 px that happened to fit a screenshot is a leftover.
- **The window's width is not the workspace's width.** A dock, a rail or a scrolled sidebar
  changes the room the sheet has; the sheet's mode is decided from the room, not the window.
- **HiDPI does not need multiplication.** Qt scales logical pixels. Only a resource
  rasterised by hand — an icon pixmap, a buffer — takes `devicePixelRatio`. The drawing is
  in millimetres and the replay applies one scale; scaling that scale again is a bug.
- **Resolution is not logical size.** A 2560 × 1440 screen at 150 % gives fewer logical
  pixels than 1920 × 1080 at 100 %. Test logical sizes, then test scales.
- **A breakpoint must not change the document.** Folding a panel, hiding a grid or fitting
  the view is view state: no undo entry, no modified mark, no file field.
- **Reduced motion changes the easing, never the threshold.** A fold is instant, not
  skipped.
- **Do not copy a layout table from another program or from a guide.** Thresholds follow
  our own content minimums; the table in the contract is the only one.

## Validation

- `--check-layout` (stage 6a) resizes headlessly to the sizes named in
  [docs/capabilities.md](../../../docs/capabilities.md) and asserts the mode, the sheet
  minimum and that no two panels overlap.
- A run at 2× density and a run with a large font, plus `--check-qml`.
- One capture per mode, light and dark, with the sheet still centred.
- The zoom, pan and check self-tests stay green: a layout change must not move the document.
- `./scripts-dev/check.sh` green before the pull request.
