# A token or theme change

The token list is the table under
[Visual language](../../../../docs/ui-ux.md#visual-language). Its implementation is
[theme.cpp](../../../../apps/punchi-fluid-qt/src/theme.cpp), which builds `ThemeTokens` from
the desktop palette; [theme.h](../../../../apps/punchi-fluid-qt/src/theme.h) declares the
names. There is no second token file: a copied table is a second source of truth and drifts.

## Read first

- [Visual language](../../../../docs/ui-ux.md#visual-language) — what each token means and
  where it comes from.
- [Following the desktop](../../../../docs/ui-ux.md#following-the-desktop) — the palette and
  portal sources, read on a change, never per frame.
- [Accessibility](../../../../docs/ui-ux.md#accessibility) — contrast, reduced motion and the
  state rule.
- [theme.cpp](../../../../apps/punchi-fluid-qt/src/theme.cpp) — `tokensFrom`,
  `ensureContrast`, `measure` and `forcedPalette`. The only colour literals in the
  application live in the forced test palettes.

## Procedure

1. **Change the adapter, not a widget.** Add or rename the token in `ThemeTokens`, fill it
   in `tokensFrom`, and update the table in `ui-ux.md` in the same change so the document
   and the code cannot disagree.
2. **Keep the accent rule.** An accent handed by the desktop is shifted only until it
   reaches its criterion; it never becomes a new hue. The on-accent ink follows the rule in
   the document, not the accent's apparent brightness.
3. **Measure the pairings.** Add the pairing to the contrast measurement when the token
   carries meaning; the window's `--check-theme` must exit on the same contract.
4. **Keep motion a token.** The duration comes from the desktop's animation factor; a
   widget does not own a literal. Reduced motion keeps the state change and drops the
   easing.
5. **Do not add a second layer.** No token file, no palette copy, no CSS variable and no
   QML literal stand beside the adapter.

## Run

```bash
QT_QPA_PLATFORM=offscreen <app> --check-theme
QT_QPA_PLATFORM=offscreen <app> --check-layout
```

`--check-theme` fails when `contrast.passes()` is false; it prints the unit and the motion
values without asserting them and cannot see a colour or size literal in a widget. Those are
manual review items. `--check-layout` catches a token change that moved a region or the
ribbon. `./scripts-dev/check.sh` runs both when the window is configured.

## Evidence

- the contrast line for the changed pairing, from `--check-theme`;
- the light and dark capture when the token is visible in the shell;
- the manual note for the unit and the motion, because the command prints them;
- the updated table in `ui-ux.md`, in the same diff.

## Gotchas

- **`ensureContrast` is not a licence for a bad palette.** It shifts a colour to reach the
  criterion; it does not invent a hue or replace the desktop's choice.
- **A token that no surface uses is dead code**, and a token used by one widget is a
  widget rule in disguise. Both are review findings.
- **A colour literal outside `forcedPalette`** is the bug the token layer exists to prevent.
- **A theme change is not a symbol change.** The schematic's ink arrives as `PfInk`; the ISO
  1219 geometry and the review record are untouched.
