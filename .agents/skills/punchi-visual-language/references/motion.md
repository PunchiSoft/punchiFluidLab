# A motion change

The contract is [Motion](../../../../docs/ui-ux.md#motion): one duration, one curve per
intent, the list of what never moves, reduced motion and the frame budget. This reference is
the procedure between a request and that contract; a duration, a curve or a delay written
here would be a second source of truth.

The sheet's animation is not this procedure. It belongs to the model and has its own rules
in [Animation on the sheet](../../../../docs/ui-ux.md#animation-on-the-sheet).

## Read first

- [Motion](../../../../docs/ui-ux.md#motion) — the intent table, the refusals and the
  reduced-motion branch.
- [Accessibility](../../../../docs/ui-ux.md#accessibility) — reduced motion drops the
  easing and keeps the state.
- [Visual language](../../../../docs/ui-ux.md#visual-language) — `motionMs` is a token
  beside the colour and the font, not a per-widget value.
- [theme.rs](../../../../apps/punchi-fluid/src/theme.rs) — the adapter that turns
  `motionMs` into the toolkit's clock. A widget asks the theme; it never names a duration.
- [Animation on the sheet](../../../../docs/ui-ux.md#animation-on-the-sheet) — the model's
  motion, to know what not to touch.

## Procedure

1. **Name the intent.** Entering or leaving, moving while it stays on screen, progress, or
   no motion at all. The contract's table gives the curve; an intent that is not there is a
   question for the owner, not a new literal.
2. **Ask the frequency question before the curve.** A control the reader uses every minute
   — zoom, select, drag, delete, a keyboard command — does not animate. Motion is for the
   occasional change of place, and the sheet is never animated for effect.
3. **Read the token.** The duration comes from `motionMs`. When it is zero, jump to the
   final state in the same frame; never start a transition with a zero duration and never
   skip the state change itself.
4. **Keep the pair together.** If a panel and its rail, or a dialog and its band, move as
   one thing, they take the same duration and the same curve from the same call site.
5. **Replace, do not restart.** A reversed or retargeted transition starts from the value it
   holds now. Interpolate the current value; do not reset the clock to zero.
6. **Stop when it settles.** The transition requests the frames it needs and none after it
   ends. Do not leave a timer repainting an idle window, and do not add a clock beside the
   transport's.
7. **Measure if it reflows.** An animated width or height lays the window out on every
   frame, and a panel width reflows the sheet. Measure the frame at 200 components and
   record the number in the pull request; shorten the transition or move the reflow to the
   end if it misses the budget.
8. **Test the zero and capture the rest.** A test proves that `motionMs == 0` produces the
   final state with no tween; a capture in both themes proves the end state on screen.

## Run

```bash
./scripts-dev/check-docs.sh        # a documentation-only change
./scripts-dev/check.sh             # a change that touches code
```

`--check-theme` prints `motionMs` and asserts the contrast only: it cannot prove that a
widget used the token or honoured the zero. Those are the test and the manual capture.

## Evidence

- the capture at rest, light and dark, when the state is visible;
- the name of the test that proves the reduced-motion branch;
- the measurement note when the change reflows, with the component count;
- the diff of [Motion](../../../../docs/ui-ux.md#motion) when the intent table changed.

## Gotchas

- **The sheet is not chrome.** A symbol, a hose or a piston that eases for effect is a
  fidelity bug; the position and the flow are the model's.
- **A literal is a bug.** A duration, a curve name, a delay or a spring constant in a widget
  is a second token; the theme owns the clock.
- **`--check-theme` prints motion; it does not assert it.** A green run does not prove the
  zero branch, the curve or the frame budget.
- **No spring, no bounce, no overshoot.** The interface does not imitate physics this
  product does not simulate.
- **No faster exit.** One desktop factor is the contract; an exit that outruns its entrance
  is a second duration by another name. If it is proposed, it is a contract change.
- **Interruption is not a restart.** A transition that begins again from zero jumps back and
  is visible; use the current value.
- **A transition that never stops is a battery bug.** Chrome motion is event-driven; the
  only continuous clock is the model's transport.
- **Motion is view state.** It never reaches the document, the file, the undo history or the
  modified mark, and reopening a document does not replay it.
- **Reduced motion is not “no feedback”.** The state still changes, the focus still moves
  and the direction still reads; only the tween is gone.
