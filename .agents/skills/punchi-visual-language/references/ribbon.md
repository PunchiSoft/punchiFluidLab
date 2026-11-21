# The ribbon

The ribbon is specified in [The ribbon](../../../../docs/ui-ux.md#1-ribbon-top) and its row
has a rule recorded from P-028. This reference routes to that rule; it does not restate the
groups, the fold order, the widths or the floor.

## Read first

- [The ribbon](../../../../docs/ui-ux.md#1-ribbon-top) — the two planes, the group names,
  large and small commands, the Simular tab and the no-duplicate rule.
- [The window adapts](../../../../docs/ui-ux.md#22-the-window-adapts) — the mode, the
  fold and the screen-aware first window.
- [ADR 0028](../../../../docs/adr/0028-the-ribbon-draws-its-own-icons.md) — the commands'
  icon family.
- **P-028** — the node that made the group row fit the support floor with per-group
  collapse. Its measurements are in P-028's spec in the EGM workspace; the rule it produced
  is in `ui-ux.md`.

## Procedure

1. **Place the command by meaning.** A command belongs to the tab that owns it and appears
   once; a second copy is a bug, not a convenience. The Simular tab owns the transport and
   the mode.
2. **Give it its label and its icon.** A large command draws icon over label; a small one
   draws the icon beside it. The icon is the family's and the label is the student's first
   reading ([an icon](icons.md)).
3. **Let the row collapse by the rule.** The priority, the measured set and the floor are
   in `ui-ux.md` and P-028. Do not add a second breakpoint, a horizontal scrollbar or a
   “more” menu in a command change.
4. **Keep the flyout the same content.** A collapsed group's flyout carries the same
   commands, labels and accessible names; it opens from the keyboard, closes on `Esc` or a
   click outside, and never becomes a modal.
5. **Keep the height and the sheet's room.** The ribbon's height and the sheet's floor are
   measurements in `ui-ux.md` and in the check; a group change must not take room from the
   drawing silently.
6. **Prove the matrix.** `--check-layout` measures the widths and the group modes;
   `--selftest` walks the fold, the flyout and the screen-aware first window.

## Run

```bash
QT_QPA_PLATFORM=offscreen <app> --check-layout
QT_QPA_PLATFORM=offscreen <app> --selftest
```

`--check-layout` fails on a group outside the content, a wrong mode set or a broken row;
`--selftest` fails on the interaction steps it names. Both need a configured Qt build, and
`check.sh` runs them with the scales and the larger font it already declares.

## Evidence

- the width captures that show the row and the collapsed groups, in both schemes;
- the flyout capture when a group is collapsed;
- the `--check-layout` lines for the widths the change affects;
- the `--selftest` steps for the fold, the flyout and the first window.

## Gotchas

- **The fold order, the floor and the measured set live in `ui-ux.md` and P-028.** A number
  copied here would drift the first time the rule moves.
- **A command outside the ribbon still needs an accessible name and a keyboard path**, but
  it is not a reason to widen the ribbon.
- **A collapsed group is not a smaller group.** It is a titled button with the same content
  in its flyout; a squeezed copy is the failure the collapse exists to avoid.
- **The first window is screen-aware.** A remembered size is clamped to the current screen;
  a change that only fits one monitor is incomplete.
- **The ribbon's icons are not the desktop's.** The rest of the window may keep the theme's
  icons; the ribbon's command language is the project's.
