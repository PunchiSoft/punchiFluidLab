# Screenshots

Images of the running application, used by the README and by pull request reviews.

| File | What it shows |
|---|---|
| `window-dark.png` | The window on a dark desktop, on the sheet the program opens with: compressor, service unit, 3/2 valve and cylinder, every port with its connection circle |
| `window-light.png` | The same window on a light desktop, taken in the same run |
| `run.mp4` | Fourteen seconds of the same sheet in motion: the valve actuated, the cylinder travelling and the hoses coloured by pressure |

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
  apps/punchi-fluid-qt/build/punchi-fluid --theme dark --grab /tmp/window-dark.png
```

The video is a screen recording of the window, transcoded to MP4 with the same content:

```bash
ffmpeg -i recording.webm -c:v libx264 -crf 30 -preset slow -pix_fmt yuv420p \
  -movflags +faststart -an assets/screenshots/run.mp4
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
