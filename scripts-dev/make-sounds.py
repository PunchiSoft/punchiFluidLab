#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 PunchiSoft
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Generates the two event samples in assets/sounds/ from noise, so the repository never
# carries an audio file of unknown origin. The recipe is here, the output is committed,
# and nothing at build time or run time depends on Python.
#
#   ./scripts-dev/make-sounds.py
#
# The seed is fixed: the same script produces the same bytes, which is what makes the
# committed files verifiable instead of merely plausible. The output was checked with
# `ffprobe` (duration, channels, rate) and by loading it in the application, which is the
# only test that proves the decoder accepts it.

import math
import random
import struct
import wave
from pathlib import Path

OUT = Path(__file__).resolve().parent.parent / "assets" / "sounds"
SEED = 20261106


class Noise:
    """White noise shaped by one-pole filters, which is all a hiss needs.

    Each stage keeps its own state. Sharing one accumulator between the low-pass and the
    high-pass of a band makes the second stage subtract its own input and silence the
    sample — which is exactly what the first run of this script produced.
    """

    def __init__(self, seed, rate):
        self._random = random.Random(seed)
        self._rate = rate
        self._states = {}

    def white(self):
        return self._random.uniform(-1.0, 1.0)

    def low_pass(self, stage, value, cutoff_hz):
        alpha = 1.0 - math.exp(-2.0 * math.pi * cutoff_hz / self._rate)
        state = self._states.get(stage, 0.0) + alpha * (value - self._states.get(stage, 0.0))
        self._states[stage] = state
        return state

    def band(self, stage, cutoff_low_hz, cutoff_high_hz, order=2):
        """Noise with the rumble and the top removed: air through a body, not a hiss.

        Two poles on each side, because a single pole leaves so much energy at the top
        that the result reads as a hiss of steam rather than air moving through metal.
        """
        sample = self.white()
        for pole in range(order):
            sample = self.low_pass(f"{stage}/low{pole}", sample, cutoff_high_hz)
        for pole in range(order):
            sample -= self.low_pass(f"{stage}/high{pole}", sample, cutoff_low_hz)
        return sample


def write_wav(name, rate, samples):
    path = OUT / name
    frames = bytearray()
    for value in samples:
        clipped = max(-1.0, min(1.0, value))
        frames += struct.pack("<h", int(round(clipped * 32767.0)))
    with wave.open(str(path), "wb") as handle:
        handle.setnchannels(1)
        handle.setsampwidth(2)
        handle.setframerate(rate)
        handle.writeframes(bytes(frames))
    return path


def normalise(samples, peak):
    highest = max(abs(value) for value in samples) or 1.0
    return [value * peak / highest for value in samples]


def valve_click(rate):
    """A short burst: the pilot air of a valve that has just been actuated.

    A real 3/2 valve sounds like one syllable — a tick of the spool, then the air behind
    it leaving through the exhaust. Two components with different envelopes read as that
    syllable; a single noise burst reads as a hiss with no cause.
    """
    duration = 0.45
    total = int(duration * rate)
    noise = Noise(SEED, rate)
    out = []
    for index in range(total):
        t = index / rate

        # The spool: a very short, very sharp transient.
        tick = 0.0
        if t < 0.006:
            tick = noise.white() * math.exp(-t / 0.0012)

        # The exhaust: broadband air in the body of the valve, decaying.
        air = noise.band("exhaust", 900.0, 6500.0)
        attack = min(1.0, t / 0.004)
        body = air * attack * math.exp(-t / 0.085)

        out.append(0.55 * tick + 0.9 * body)

    # A tail that reaches zero inside the sample: a hard cut at the end clicks.
    fade = int(0.05 * rate)
    for index in range(fade):
        out[-1 - index] *= index / fade
    return normalise(out, 0.62)


def air_charge(rate):
    """Five seconds of a volume being charged, which is what starting the model is.

    The bed is band-limited noise, slowly modulated so it breathes instead of humming, and
    it falls away as the volume equalises with the supply. That is the shape the model
    actually has: a fast rise, a long approach to equilibrium.
    """
    duration = 5.0
    total = int(duration * rate)
    noise = Noise(SEED, rate)
    out = []
    for index in range(total):
        t = index / rate

        swell = 1.0 + 0.12 * math.sin(2.0 * math.pi * 0.6 * t)
        # 80 ms to full, then a smooth fall that ends the sample where the volume rests.
        attack = min(1.0, t / 0.08)
        settle = 1.0 - smoothstep(3.2, duration, t)
        out.append(noise.band("charge", 260.0, 8000.0) * swell * attack * settle)

    fade = int(0.35 * rate)
    for index in range(fade):
        out[-1 - index] *= index / fade
    return normalise(out, 0.5)


def cylinder_stop(rate):
    """The piston hitting the end of its stroke, which is the sound a circuit really makes.

    Three parts, because that is how a stop sounds: a hard clack of metal on metal, a low
    thud of the barrel taking the blow, and a very short tail of air behind the piston.
    The clack is the event; everything after it is the room. The reference plays its
    `cylinder.wav` at the end positions, and this is that moment.
    """
    duration = 0.40
    total = int(duration * rate)
    noise = Noise(SEED, rate)
    out = []
    for index in range(total):
        t = index / rate

        # The strike: broadband, very short, and the loudest part of the sample.
        clack = noise.band("clack", 1600.0, 11000.0) * math.exp(-t / 0.008)

        # The barrel: a low thud whose resonance sits below everything else here.
        thud = math.sin(2.0 * math.pi * 96.0 * t) * math.exp(-t / 0.055)
        thud += 0.4 * math.sin(2.0 * math.pi * 210.0 * t) * math.exp(-t / 0.030)

        # The air leaving behind the piston, gone in a few tens of milliseconds.
        tail = noise.band("tail", 400.0, 3800.0) * min(1.0, t / 0.003) * math.exp(-t / 0.040)

        out.append(1.0 * clack + 0.75 * thud + 0.28 * tail)

    fade = int(0.04 * rate)
    for index in range(fade):
        out[-1 - index] *= index / fade
    return normalise(out, 0.72)


def smoothstep(edge0, edge1, value):
    x = max(0.0, min(1.0, (value - edge0) / (edge1 - edge0)))
    return x * x * (3.0 - 2.0 * x)


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    for name, rate, samples in (
        ("valve.wav", 44100, valve_click(44100)),
        ("air.wav", 44100, air_charge(44100)),
        ("cylinder.wav", 44100, cylinder_stop(44100)),
    ):
        path = write_wav(name, rate, samples)
        print(f"{path.relative_to(OUT.parent.parent)}  {len(samples) / rate:.2f} s  "
              f"{rate} Hz  {path.stat().st_size} bytes")


if __name__ == "__main__":
    main()
