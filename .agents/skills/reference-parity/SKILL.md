---
name: reference-parity
description: Review the running parent app as the reference for symbols, animations and workflow, then implement and prove parity by hand without copying geometry. Use when a symbol, an animation or a workflow must match the parent app, before changing one, or when the owner says the result does not look like the reference.
license: GPL-3.0-or-later
---

# Reference parity

This project replicates the **behaviour and the symbology** of a parent application the owner
runs on the same machine. That application is the reference; this repository is the
implementation. The reference is looked at, measured and understood, never traced: every
symbol is written by hand to conform to the convention (ADR 0009), and no image, sound,
manual or file from it enters the repository.

A change is not finished because it compiles. It is finished when the reference and the
implementation show the same thing, the evidence is in the repository, and the gate is green.

## Before touching a symbol or an animation

1. **Find the reference window.** It runs through Proton and its window class is
   `steam_proton`, not `FluidSIM`. It may be behind other windows or minimised. List windows
   with a KWin script; do not guess:

   ```bash
   cat > /tmp/opencode/log-windows.js <<'JS'
   for (const c of workspace.windowList()) {
       console.info("PWIN|" + c.caption + "|" + c.resourceClass + "|" + c.minimized);
   }
   JS
   gdbus call --session --dest org.kde.KWin --object-path /Scripting \
     --method org.kde.kwin.Scripting.loadScript /tmp/opencode/log-windows.js pw-log
   gdbus call --session --dest org.kde.KWin --object-path /Scripting \
     --method org.kde.kwin.Scripting.start
   journalctl --user -n 80 --no-pager | grep PWIN
   ```

2. **Raise it and capture it.** Bring the `steam_proton` window to the front with another
   script, then take a full screenshot and crop the left palette:

   ```bash
   cat > /tmp/opencode/raise-reference.js <<'JS'
   for (const c of workspace.windowList()) {
       if (String(c.resourceClass) === "steam_proton") {
           c.minimized = false;
           workspace.activeWindow = c;
       }
   }
   JS
   gdbus call --session --dest org.kde.KWin --object-path /Scripting \
     --method org.kde.kwin.Scripting.loadScript /tmp/opencode/raise-reference.js raise-ref
   gdbus call --session --dest org.kde.KWin --object-path /Scripting \
     --method org.kde.kwin.Scripting.start
   spectacle -b -n -f -o /tmp/opencode/reference.png
   magick /tmp/opencode/reference.png -crop 430x440+0+220 +repage -resize 400% /tmp/opencode/relays.png
   ```

3. **Read the reference before writing.** Identify the family, the actuator, the contact
   blade, the coil mark and the terminal numbering. Write the finding in the review file
   under *Provenance* in words: "a rectangle with a latch square", "a semicircular actuator
   with a dashed link". Never copy the pixels or the file.

## The audit, symbol by symbol

For every symbol the capability touches:

| Question | Evidence |
|---|---|
| Does the family match the reference? | the captured palette crop |
| Is the actuator the reference's (rod, lever, roller, dome, semicircle)? | the file geometry |
| Is the contact a blade, not a bar? | `control_drawing.rs` |
| Are the terminals named and placed like the reference? | the file's `[[port]]` |
| Is the mark (latch, delay, counter) the reference's? | the file's primitives |
| Does the review image show the rest state? | regenerated light/dark PNG |

A symbol with a plausible drawing that does not match the family is a defect, not a taste.

## The audit, animation by animation

Run the reference and watch the circuit, not only the palette. Record what moves, what
carries the direction and what stops when the run pauses. Then implement it **from the
model**:

- A flow chevron follows the solver's signed flow through that hose, never a pressure
  gradient dressed as flow.
- A moving part follows the model's position; a lamp and a bar follow the run state.
- Starting resets the phase, pausing freezes it, reset clears it.
- Reduced motion (`AnimationDurationFactor = 0`) still shows the state, with a static arrow
  or a fixed position.
- No allocation in the per-frame draw or the solver step.

An animation that cannot be tied to a model quantity is not implemented: write the missing
quantity in the queue and the continuity note, and do not fake it.

## The loop

1. Read `docs/work-queue.md` and `docs/parity-run.md`; take the first `ready` entry.
2. Audit the reference for that entry (palette and running behaviour).
3. Implement in layer order: catalog file, model, drawing, ABI, window, format only if
   unavoidable.
4. Regenerate every affected review image with
   `PUNCHI_FLUID_WRITE_SAMPLE=1 cargo test -p punchi-fluid-export --test generate`.
5. Write the evidence and the documentation in the same commit; run
   `./scripts-dev/check.sh` until green.
6. Commit once, signed, and update the queue and the continuity note.
7. Take the next entry. The pull request waits until only `blocked` entries remain.

## Stopping

The owner's order is to continue without stopping until the behaviour and the symbology are
correct. Stopping without progress is not allowed. A session may stop only when:

- the gate fails twice for the same reason and the reason is written down;
- an entry needs an owner decision (a missing product rule, a licence question, a domain
  that needs its own ADR);
- every remaining entry is `blocked`;
- the context is exhausted, and then the last action is a gated commit and a note that says
  the exact next step. A reset is not a stop.

**Progress means**: a gated commit with its test and its documentation, or a written blocker
with the exact missing input. A plan alone is not progress.

## Gotchas

- The reference window class is `steam_proton`; searching for "FluidSIM" finds nothing.
- `spectacle` captures the current screen, so raise the window first.
- A KWin script runs once; load it under a new name when it changes.
- The review image generator draws the rest state; do not claim a run state from it.
- Arc angles are degrees, not box coordinates; only the centre and radius are `0..=1`.
- The contact is a **blade**, not a bar; a bar reads as a different switch.
- A latching relay, a timer and a counter carry a **mark** on the coil; two floating coils
  are not the reference's family.
- Regenerating all review images changes many files: commit them with the geometry, never
  apart.

## Validation

```bash
gdbus call --session --dest org.kde.KWin --object-path /Scripting \
  --method org.kde.kwin.Scripting.isScriptLoaded raise-ref     # the window script ran
PUNCHI_FLUID_WRITE_SAMPLE=1 cargo test -p punchi-fluid-export --test generate
cargo test -p punchi-fluid-catalog                              # lint and reviews
./scripts-dev/check.sh                                          # full gate
git --no-pager log --oneline -1                                 # one signed commit
```
