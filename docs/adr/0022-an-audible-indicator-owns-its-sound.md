# 0022 — An audible indicator owns its sound path

- **Status:** accepted
- **Date:** 2026-11-13
- **Context crates:** `punchi-fluid-catalog`, `punchi-fluid-draw`, `punchi-fluid-ffi`, `apps/punchi-fluid-qt`

## Context

[ADR 0021](0021-an-indicator-reads-the-net.md) gave the control circuit a lamp: one terminal,
lit while the net reaches it. An audible indicator asks the same electrical question — *is the
line live?* — but the answer has to be heard, not seen, and the sound has to stop when the
line does. The window already plays event samples (valve, air, cylinder); a buzzer is the
first sound that **lasts**, and the model has to say when to start and stop it.

## Decision

- **The part is `Behaviour::Buzzer`**, one electrical terminal, lit by the same settle as the
  lamp. It is a load of the boolean net and adds no edge; a circuit behaves the same with or
  without it.
- **The sheet says the state as a word**: a sounding buzzer reads `sonando`, a silent one
  `en silencio`, through `Reading::Audible`. The body and its two waves are in the file, so
  the drawing is not a colour-only signal.
- **The ABI reports an aggregate, not a stream.** `pf_session_buzzer_active` answers `1` when
  any buzzer of the session sounds and `0` when none does. The mixer needs one answer per
  refresh, not one per placement, and the session already holds the state.
- **The sound lives in the window, not in the core.** The FFI and the solver stay silent and
  free of an audio dependency; the Qt session exposes `buzzerActive`, polls the ABI after
  every refresh, and emits `buzzerChanged` only when the value changes.
- **The QML loops the sample.** `SoundEffect` with an infinite loop starts on a rising edge
  and stops on a falling one; the mute switch stops it, and unmuting restarts it only if the
  buzzer is still sounding. The same rule applies to the air bed.
- **The sample is generated.** `scripts-dev/make-sounds.py` writes `assets/sounds/buzzer.wav`
  from a fixed seed, with a fundamental and odd harmonics and an integer number of cycles in
  one second so the loop joins without a click. No recording of unknown origin enters the
  repository.
- **The binary proves it decodes.** `--check-sounds` waits for every sample, the buzzer
  included, to reach `SoundEffect.Ready`; a truncated or missing resource fails the check.

## Consequences

- The catalog gains `electrical.buzzer` and its review; the drawing gains four words in
  `Reading` and no new geometry beyond the file.
- The session gains one boolean property and one signal; the QML gains one looping effect.
  Neither the solver nor the document changes.
- A future horn or siren is another file and another sample, not another audio path.
- A machine with no audio device still runs: the sound is a notification, the sheet carries
  the state, and the window obeys mute.

## Alternatives considered

- **A beep on every change.** It would not model a buzzer held on by a live line, and a
  listener could not tell a held alarm from a short pulse.
- **Play the sound from the FFI.** The core would depend on an audio backend, an allocation
  and a device that a headless run does not have.
- **One ABI query per buzzer.** The mixer would poll every placement to answer one question;
  the aggregate is the question it actually asks.
- **A recording of a real buzzer.** Its licence and origin would have to be recorded, and a
  synthesised signal is deterministic and reproducible.
