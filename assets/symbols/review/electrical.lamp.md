# electrical.lamp — conformance review

    status: pending-300dpi
    date: 2026-11-13
    reviewer: automated pass, unreviewed by a second person
    function: lights while its terminal is live

## Provenance

Hand-written geometry in `assets/catalog/electrical.lamp.toml`: a circle with a cross and a
stem to its single terminal. The **rays** and the word `encendida` are drawn by the renderer
from the state of the net (ADR 0021). No third-party asset and no trace of another program's
symbol.

## Automated checks — `punchi-fluid-catalog` symbol lint

| Rule | Result |
|---|---|
| Primitives inside the box, no degenerate primitive | pass |
| Every line declares its class | pass: the stem is `electrical`, the glass `working` |
| Exactly one electrical terminal | pass: the lamp validation |
| Legible at palette size | pass: see the images |

## Visual check

| Requirement | Result | Evidence |
|---|---|---|
| The glass and the cross are readable | pass | `electrical.lamp-light.png` |
| A lit lamp gains rays, not only a colour | pass | the eight rays in the image |
| The state is also a word | pass | `encendida` beside the symbol |
| A dark lamp keeps both away | pass | `a_lamp_shows_its_state_as_rays_not_only_colour` |
| The class says what the line is | pass | dotted stem for the wire |
| Legible at 300 dpi | **not checked** | needs the maintainer's reader |

## What this review does not prove

- The 300 dpi pass is still owed: the status stays `pending-300dpi` and the export keeps the
  "not verified" note.
- The image shows the lit state. The dark state has no rays and says `apagada`, proven by the
  drawing test and by the ABI query `pf_session_indicator_active`.
- The lamp is a load of the boolean net: it reads one terminal and does not add a voltage or
  a current, which is the model of ADR 0018.

## Images

- `electrical.lamp-light.png`, `electrical.lamp-dark.png` — this symbol lit.
