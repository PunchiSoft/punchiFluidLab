# electrical.buzzer — conformance review

    status: pending-300dpi
    date: 2026-11-13
    reviewer: automated pass, unreviewed by a second person
    function: sounds while its terminal is live

## Provenance

Hand-written geometry in `assets/catalog/electrical.buzzer.toml`: a round body, two sound
waves and a stem to its single terminal. The **sound** is a generated sample, not a
recording: `scripts-dev/make-sounds.py` writes `assets/sounds/buzzer.wav` from a fixed seed,
and the window loops it while the part is active (ADR 0022). No third-party asset and no
trace of another program's symbol or sound.

## Automated checks — `punchi-fluid-catalog` symbol lint

| Rule | Result |
|---|---|
| Primitives inside the box, no degenerate primitive | pass |
| Every line declares its class | pass: the stem is `electrical`, the body and waves `working` |
| Exactly one electrical terminal | pass: the indicator validation |
| Legible at palette size | pass: see the images |

## Visual check

| Requirement | Result | Evidence |
|---|---|---|
| The body and the sound waves are readable | pass | `electrical.buzzer-light.png` |
| The state is a word, not only a colour | pass | `sonando` / `en silencio` |
| The waves stay clear of the state word | pass | the image above |
| The class says what the line is | pass | dotted stem for the wire |
| Legible at 300 dpi | **not checked** | needs the maintainer's reader |

## Sound check

| Requirement | Result | Evidence |
|---|---|---|
| The sample decodes in the shipped binary | pass | `--check-sounds` includes the buzzer |
| The loop is seamless | pass | 440 Hz and its harmonics over exactly one second |
| It sounds only while the net is live | pass | `pf_session_buzzer_active` and the QML binding |
| A muted window is silent | pass | the mute switch stops the loop |
| It is generated, not copied | pass | `scripts-dev/make-sounds.py`, fixed seed |

## What this review does not prove

- The 300 dpi pass is still owed: the status stays `pending-300dpi` and the export keeps the
  "not verified" note.
- The image shows the active state. The inactive state says `en silencio` and the ABI test
  `a_buzzer_sounds_while_its_line_is_live` proves the transition.
- The sample is synthesised: it is a teaching signal, not a recording of a real buzzer.

## Images

- `electrical.buzzer-light.png`, `electrical.buzzer-dark.png` — this symbol sounding.
