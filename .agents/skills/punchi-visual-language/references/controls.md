# A control

A control is chrome: it edits, commands or shows state, and it belongs to the visual
language. It is not a component of the schematic, which is drawn by `punchi-fluid-draw`
from its definition and never by this path.

## Read first

- [Visual language](../../../../docs/ui-ux.md#visual-language) — the tokens, the states and
  the rule that a widget names no colour.
- [The window adapts](../../../../docs/ui-ux.md#22-the-window-adapts) — how the regions
  and the control's room change with the window.
- [Accessibility](../../../../docs/ui-ux.md#accessibility) — names, roles, keyboard, targets
  and reduced motion.
- [theme.cpp](../../../../apps/punchi-fluid/src/theme.rs) and
  [theme.h](../../../../apps/punchi-fluid/src/theme.rs) — the token names the control may
  ask for. There is no other palette.

## Procedure

1. **Choose the token, not the value.** Name the semantic role the control needs (surface,
   ink, line, accent, danger, spacing, type or motion) and take it from the theme. A literal
   in a widget is a bug; if no token fits, the token layer is the change, not the widget.
2. **Build every state.** Normal, hover, pressed, disabled and checked when the control has
   one. The state must survive without colour: a shape, an outline, a mark or a label
   carries it too.
3. **Use the theme's unit and motion.** Size the control from the spacing unit and the
   desktop font, and take the transition from the motion token. The desktop's reduced
   motion setting disables the animation and keeps the state change.
4. **Make it reachable.** Give it a focus ring, a keyboard path and an accessible name and
   role. Tab must reach it; a pointer-only control is unfinished.
5. **Follow the shell.** The control uses the region's mode from
   [qt-adaptive-layout](../../qt-adaptive-layout/SKILL.md); it does not invent a width, a
   breakpoint or a second layout. Layouts and anchors are not mixed in one item.
6. **Prove it.** Run the checks below, capture the result in both schemes and walk the
   manual checklist in [the review](review.md).

## Run

```bash
QT_QPA_PLATFORM=offscreen <app> --check-theme
QT_QPA_PLATFORM=offscreen <app> --check-qml
QT_QPA_PLATFORM=offscreen <app> --check-layout
QT_QPA_PLATFORM=offscreen <app> --selftest
```

The application lives under `apps/punchi-fluid-qt/build` and the build must be configured;
`./scripts-dev/check.sh` skips the window when it is not. `--check-theme` exits on the
contrast contract; the unit and motion lines are printed for the reader, not asserted, and
the no-literal-colour rule is a manual review item it cannot see.

## Evidence

- a light and a dark capture of the control in its states, or the headless probe when the
  check already measures it;
- the contrast line, when the control introduces a pairing;
- the manual checklist marked, with the states, the keyboard path and the accessible name
  named explicitly.

## Gotchas

- **A state only in colour fails the rule**, even when the contrast passes.
- **A control that copies the desktop's icon** is fine only outside the ribbon; the ribbon's
  family is [ADR 0028](../../../../docs/adr/0028-the-ribbon-draws-its-own-icons.md)'s.
- **A panel control is view state**, never document state; it does not enter undo.
- **A new control is not a reason for a new token, a new palette or a new breakpoint.**
