# Application icon

The plate the window shows and the packaging will install: a **P** built from a pipe that
rises on the left, loops around a manometer as its bowl and returns down the outlet leg,
with the graduation, the needle and the flow chevrons of a pressure gauge on it.

## Files

| Path | What |
|---|---|
| `punchi-fluid-lab.svg` | the source: `512 × 512`, plain shapes and one gradient, `3.6 kB` |
| `punchi-fluid-lab-16.png` … `-512.png` | the raster set: 16, 24, 32, 48, 64, 128, 256 and 512 px |

The window takes one of them, the **256** px plate, through
[`apps/punchi-fluid/src/branding.rs`](../../apps/punchi-fluid/src/branding.rs), because
`ViewportBuilder::with_icon` wants pixels and the first frame does not exist yet. The rest
are the sizes a launcher, an AppImage or a `.desktop` file asks for.

## Where it comes from

Drawn for this project on 2026-11-23 from the owner's reference sheet and his choice of its
**cobalt dual-tone** variant. The geometry was measured off that reference — plate, ring,
legs, graduation, needle, collars — and written here as our own vector: **no path, asset,
name or byte is taken from another program or from an icon theme.** A theme was the visual
brief; it is not a source.

The whole tree's licence applies: **GPL-3.0-or-later**.

## Rules it keeps

- **Full-bleed plate**: `viewBox="0 0 512 512"`, the squircle from edge to edge with `rx 88`
  and a 12 px rim, so a launcher that adds no margin does not cut the drawing and one that
  does adds its own.
- **Four colours**: plate `#1B489E`, rim `#2D68C3`, light tone `#90C0F0`, medium tone
  `#2484CC`. The two tones are the icon's language: light for the inlet and the upper left,
  medium for the outlet and the lower right, split at twelve o'clock and at the outlet leg.
- **No filter, no `<style>`, no `<script>`, no external reference, no text.** Nothing here
  depends on a renderer feature another one lacks; Inkscape, librsvg and a browser draw the
  same picture, and only the edge antialiasing differs. That is the trap the first draft fell
  into: a `feDropShadow` on the plate made Inkscape drop the plate entirely.
- **Legible at 48 px** and recognisable down to 32. At 16 px the drawing is a plate with a
  lighter ring, which is what any detailed symbol becomes at that size; a compact 16/24
  drawing is the change that would fix it if it is ever needed.

## Regenerating the raster set

```bash
for size in 16 24 32 48 64 128 256 512; do
  inkscape --export-type=png \
    --export-filename="assets/branding/punchi-fluid-lab-$size.png" \
    --export-width="$size" --export-height="$size" \
    assets/branding/punchi-fluid-lab.svg
done
```

A new size is a row in the table above and, once `packaging/` exists, a file in its hicolor
tree. The window's own test decodes the 256 px plate and fails if it stops being the square
with cut corners and an opaque middle.
