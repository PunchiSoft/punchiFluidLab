# ADR 0003 — One drawing behind a `Surface` trait

- **Status:** accepted
- **Date:** 2026-11-06
- **Context crates:** `punchi-fluid-draw`, `punchi-fluid-ui`, `punchi-fluid-export`,
  `punchi-fluid-desktop`

## Context

The window draws the circuit with `egui`. M2 asks for the same circuit on a standard
sheet, as PNG and PDF. The cheapest wrong answer is a second renderer: two drawings
drift, and the first one to be improved is the one the user sees, so the printed
diagram quietly becomes a worse version of the screen.

The second wrong answer is to tessellate `egui` shapes and emit triangles: the PDF
would carry every glyph as a path, be enormous, and lose selectable text.

## Decision

Drawing happens once, in **`punchi-fluid-draw`**, in millimetres, against a `Surface`
trait with six methods (line, polygon, rect, circle, arc, text). Two devices implement
it:

- `punchi-fluid-ui::surface::EguiSurface` — one millimetre is one point on screen;
- `punchi-fluid-export::pdf::PdfSurface` — millimetres converted to points, Helvetica,
  hexadecimal strings for text.

A `Recording` implementation keeps every call in order, which is how a test proves the
screen and the paper are the same drawing: with proportional areas the calls match
coordinate for coordinate and differ only in scale.

The drawing crate depends on `catalog`, `core` and `solver` and on nothing else of
ours; it does not know about windows, files, fonts or colours outside `Ink`.

## Consequences

- A new device (SVG, PNG, a QML front end) is a new `Surface` implementation.
- Colour arrives as `Ink`, so the drawing crate never reads a theme and the printed
  sheet can force black on white.
- Dashing is expanded into solid segments before the device sees it, in drawing
  millimetres, so screen and paper put the gaps in the same places.
- Text is positioned by the device: `egui` measures with its own fonts and the PDF
  estimates Helvetica widths, so a centred label may be off by a fraction of a
  millimetre on paper. The call list is identical; only the glyph metrics differ.
- The PDF writer is ours, without a dependency, which keeps `deny.toml` simple and the
  output reproducible: same calls, same bytes.
- The PNG backend is not written yet; the trait is what makes it a small change.
