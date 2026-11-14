# Screenshots

Images of the running application, used by the README and by pull request reviews.

| File | What it shows |
|---|---|
| `window-light.png` | The window on a light desktop, on the **A4 landscape** sheet the program opens with: the ISO 5457 frame with its four centring marks, compressor, service unit, 3/2 valve and cylinder, every port with its connection circle, the two help links **Acerca de** and **Licencia** at the right of the tab row, and the status bar reading `A4 · 297 × 210 mm` |
| `window-dark.png` | The same window on a dark desktop, to check the paper edge, the frame and the help links in both themes |
| `a3-reference-zones-light.png` | An **A3 landscape** sheet on a light desktop: the optional ISO 5457 / NCh 13.Of93 reference zones, with numerals down the left and right bands and letters across the top and bottom, all inside the margins |
| `zoom-light.png` | The same sheet at 150 % (`--zoom 1.5`), the status bar reading `Zoom 150 %`: the capture for the wheel zoom and pan review |
| `zoom-dark.png` | The same magnified sheet on a dark desktop, to show the readout and the sheet edge in both themes |
| `simular-light.png` | The **Simular** tab open on a light desktop: its six transport tiles, the three speed tiles and the state, with the group names underneath |
| `simular-dark.png` | The same tab on a dark desktop, to check the tiles and the marked speed in both themes |
| `title-block-light.png` | The **docked Cajetín editor** open on a light desktop, industrial model: the content combo, four position miniatures, the twelve guide fields and the 170 × 45 mm block drawn on the sheet beside it |
| `title-block-dark.png` | The same editor on a dark desktop, to check the fields, the derived sheet number and the block in both themes |
| `title-block-escolar-light.png` | The docked editor with the **Escolar** model, showing the shorter field set, the **Nota:** box with its placeholder and the compact block |
| `title-block-zoom-light.png` | The A4 industrial block at **2×** from `window-light.png`: the 5 mm title and the 3.5 mm values sit inside their rows, and the title does not touch the Nº de plano row |
| `about-light.png` | The **Acerca de** popup on a light desktop: the name and the core's version, the unreleased note, the repository and issue links, the copyright with **Leer la licencia**, the author and **Cerrar** |
| `about-dark.png` | The same popup on a dark desktop, to check the links and the panel in both themes |
| `license-light.png` | The **Licencia** popup: the GPL summary and the start of the full text that travels inside the binary |
| `cajetin-tab-light.png` | The **Cajetín** ribbon tab open on a light desktop: the Contenido summary, the positions and the Datos group |
| `cajetin-portrait-light.png` | The docked editor on **A4 portrait**, with the content and position combos and the three positions the portrait sheet offers |
| `cajetin-landscape-light.png` | The same editor on **A4 landscape**, with the two corner positions the landscape sheet offers |
| `run.gif` | The same clip as an inline image for the README (720 px, 12 fps, 924 KB), because GitHub does not play a repository video inside a page |
| `run.mp4` | The clip at 4× playback: 44 seconds of the sheet in motion in 11, the valve actuated, the cylinder travelling and the hoses coloured by pressure |

## Origin and licence

Every file here is a capture of **this** program running on the maintainer's machine, taken
from the window itself — never a mock-up, never a composition of another program, never a
screenshot of a competitor. They are distributed under the same licence as the project,
GPL-3.0-or-later.

## How they are taken

```bash
# A clean profile, so no safety copy or recent list appears in the frame.
mkdir -p /tmp/punchi-capture
XDG_DATA_HOME=/tmp/punchi-capture QT_QPA_PLATFORM=offscreen \
  apps/punchi-fluid-qt/build/punchi-fluid --theme light --grab /tmp/window-light.png
# The magnified pair, at the zoom the review is about.
XDG_DATA_HOME=/tmp/punchi-capture QT_QPA_PLATFORM=offscreen \
  apps/punchi-fluid-qt/build/punchi-fluid --theme light --zoom 1.5 --grab /tmp/zoom-light.png
# The docked title-block editor, light and dark; 2 is the industrial model, 0 escolar.
XDG_DATA_HOME=/tmp/punchi-capture QT_QPA_PLATFORM=offscreen \
  apps/punchi-fluid-qt/build/punchi-fluid --theme light --tab 3 --block --block-model 2 \
  --grab /tmp/title-block-light.png
# The Cajetín tab itself.
XDG_DATA_HOME=/tmp/punchi-capture QT_QPA_PLATFORM=offscreen \
  apps/punchi-fluid-qt/build/punchi-fluid --theme light --tab 3 \
  --grab /tmp/cajetin-tab-light.png
# Portrait, to show the third position in the combo.
XDG_DATA_HOME=/tmp/punchi-capture QT_QPA_PLATFORM=offscreen \
  apps/punchi-fluid-qt/build/punchi-fluid --theme light --open /tmp/plano-a4-portrait.pfluid \
  --tab 3 --block --block-model 2 --grab /tmp/cajetin-portrait-light.png
# The A3 sheet with its reference zones: open a saved A3 document first.
XDG_DATA_HOME=/tmp/punchi-capture QT_QPA_PLATFORM=offscreen \
  apps/punchi-fluid-qt/build/punchi-fluid --theme light --open /tmp/plano-a3.pfluid \
  --grab /tmp/a3-reference-zones-light.png
# The two help popups.
XDG_DATA_HOME=/tmp/punchi-capture QT_QPA_PLATFORM=offscreen \
  apps/punchi-fluid-qt/build/punchi-fluid --theme light --about --grab /tmp/about-light.png
XDG_DATA_HOME=/tmp/punchi-capture QT_QPA_PLATFORM=offscreen \
  apps/punchi-fluid-qt/build/punchi-fluid --theme dark --about --grab /tmp/about-dark.png
XDG_DATA_HOME=/tmp/punchi-capture QT_QPA_PLATFORM=offscreen \
  apps/punchi-fluid-qt/build/punchi-fluid --theme light --license --grab /tmp/license-light.png
```

`plano-a3.pfluid` and `plano-a4-portrait.pfluid` are written with
`--save`, then their `sheet.paper` or `sheet.landscape` field is set to the sheet the
capture is about: the format is the program's own and the drawing stays the opening one.

The still captures in this folder are taken from the window itself with `--grab`, so they
show the content without window decorations. A decorated capture is still the maintainer's
preferred hero image; when one is taken it replaces `window-light.png` and this paragraph
says so.

The video follows what a repository can serve cheaply, as GitHub recommends for clips:
**MP4/H.264**, **30 frames per second**, `yuv420p`, `faststart` so it starts before it is fully
downloaded, no audio track, and **under 1 MB** — it is 208 KB for fourteen seconds.

```bash
# The full clip at 4x, the way a repository serves one cheaply.
ffmpeg -i recording.webm -vf "setpts=PTS/4,fps=30,scale=1280:-2:flags=lanczos" \
  -c:v libx264 -profile:v high -level 4.0 -crf 26 -preset slow -pix_fmt yuv420p \
  -movflags +faststart -an assets/screenshots/run.mp4
# The inline one: a GIF, because a README shows an image and not a video.
ffmpeg -i recording.webm -vf "setpts=PTS/4,fps=12,scale=720:-2:flags=lanczos,split[a][b];\
[a]palettegen=max_colors=128[p];[b][p]paletteuse=dither=sierra2_4a" assets/screenshots/run.gif
```

## Rules

- **A capture shows what the build does.** If the ribbon changes, the image is regenerated in
  the same change; a screenshot that lies about the interface is worse than no screenshot.
- **No private content.** No path, document name, user name or other window in the frame.
  The circuit is one of ours, drawn here, not a file opened from anywhere else.
- **Light and dark** are captured when both matter for a review, as
  [assets/symbols/review](../symbols/review) does for the symbols.
- **Compressed, not resized.** The images are stripped of metadata and kept at their real
  resolution, so text stays legible on a phone.
