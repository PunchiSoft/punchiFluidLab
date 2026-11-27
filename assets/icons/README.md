# Action icons

The window's action-icon family: one SVG file per command, in the flat 2D line style the
owner asked for on 2026-11-21 ("iconografía estilo 2D con estilo flat"), inspired by the
open sets and drawn here.

## Why files

A command icon is a drawing that is edited with a drawing tool, reviewed at 16 px and
scaled to any density. Keeping it as geometry inside `actionicons.cpp` made it impossible to
edit without a compiler and impossible to scale past the sizes the renderer knew. The
record and its consequences are in [ADR 0035](../../docs/adr/0035-the-icons-are-svg-files.md),
which supersedes ADR 0028's "no asset files" rule.

## Rules

- **Box**: `viewBox="0 0 24 24"`, square, the drawing centred inside a 2-unit margin.
- **Stroke**: 2 units, `stroke-linecap="round"`, `stroke-linejoin="round"`, `fill="none"`.
  A filled dot or triangle is allowed only where the metaphor is solid (play, stop, a
  junction); never a second colour.
- **Ink**: `stroke="currentColor"`. The window tints the icon with the theme's tone; a file
  never carries a literal colour, a gradient or a shadow.
- **No text**, no raster image, no external reference, no script, no `<style>` block.
- **Names**: kebab-case, the id the ribbon table asks for (`document-save`, `pipe-pilot`).
  One file per id, and the file name is the id.
- **Nothing copied**: the geometry is written here from the command's meaning. Lucide (ISC),
  Tabler (MIT) and Fluent (MIT) are references for breadth and legibility, never a source;
  no path, asset or name is taken from them.

## Layout

| Path | What |
|---|---|
| `actions/<id>.svg` | one drawing per command |
| `review/` | the rendered sheets used as evidence (16, 24 and 32 px, light and dark) |

## Adding or changing one

1. Draw it in the box and the stroke above; keep one concept per drawing.
2. Save it as `actions/<id>.svg`.
3. Run `cargo test -p punchi-fluid-window every_action_icon`: the test resolves every icon
   the ribbon asks for, compares the drawings with the inventory, and checks the 24-point
   canvas and the state ink.
4. Read the 16 px row in `review/action-icons-light.png` and `review/action-icons-dark.png`;
   an icon that does not read there is redrawn, not shipped. The four state tones are in
   `review/action-icon-states-*.png`.
5. Update the inventory in [`docs/reference/icons.md`](../../docs/reference/icons.md) in the
   same change.
