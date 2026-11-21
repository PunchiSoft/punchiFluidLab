# An icon

The ribbon's icons are the project's own line family: vector geometry in the application,
one ink, drawn by hand from the command's meaning. The rest of the window may use the
desktop's themed icon. A component is never shown by an action icon; it is drawn by
`punchi-fluid-draw` from its ISO 1219 definition.

## Read first

- [ADR 0028](../../../../docs/adr/0028-the-ribbon-draws-its-own-icons.md) — the family, its
  grid and stroke, the resolution order and the “nothing copied” rule.
- [actionicons.cpp](../../../../apps/punchi-fluid-qt/src/actionicons.cpp) — the existing
  drawings and the command table they serve.
- **P-033** — the family's own node and inventory. Its spec is
  `specs/icon-family.md` in the EGM workspace until `docs/reference/icons.md` lands in this
  repository; this skill does not own the inventory and does not restate it.
- [The ribbon](../../../../docs/ui-ux.md#1-ribbon-top) — where the family is used.

## Procedure

1. **Name the action and one concept.** The drawing says what the command does; a second
   meaning, a second object or a letter is another icon, not a busier one.
2. **Draw it in the family.** Follow ADR 0028's grid, stroke, caps and ink. The values are
   the ADR's; this skill does not repeat them. The drawing is geometry, not a traced
   screenshot and not an SVG from another set.
3. **Add it to the table and the fallback.** A command with a drawing uses it first; the
   desktop's symbolic icon is the second answer; the label alone is last. A missing drawing
   is a visible fallback, never a blank button.
4. **Check coverage and use.** `--check-icons` reports project, fallback and missing per
   name; it fails on a command the family does not cover and on a use outside the family.
5. **Render the sheet and read it small.** Use `--icon-sheet` in both themes and look at the
   smallest size the window renders; the icon is one concept at that size or it is redrawn.
6. **Leave the schematic alone.** The icon family never draws a valve, a cylinder, an
   instrument or a port; those are ISO 1219 symbols and their review is P-018b's and the
   pending symbology review's.

## Run

```bash
QT_QPA_PLATFORM=offscreen <app> --check-icons
QT_QPA_PLATFORM=offscreen <app> --icon-sheet <file> --theme light
QT_QPA_PLATFORM=offscreen <app> --icon-sheet <file> --theme dark
```

`--check-icons` is the gate's coverage check. `--icon-sheet <file> --theme light|dark` is a
**manual** render, not a gate check: it takes an output path and a theme, writes a sheet and
does not fail on a poor reading. Its exit status proves nothing. The build must be
configured; `check.sh` skips the window when it is not.

## Evidence

- the rendered sheet for both themes, linked from the pull request;
- the small-size reading: the name of the icon and the concept it still communicates;
- the `--check-icons` line for the new name and its fallback;
- a note that no asset, set or screenshot from another product was used.

## Gotchas

- **A second icon set is a supply-chain and licence decision**, not a drawing session:
  Tabler, Lucide or Fluent may be inspiration, never a source.
- **A component is not an action.** A `valve` or a `cylinder` drawn in the ribbon would
  break the symbol contract; the component keeps its own ISO 1219 symbol.
- **The inventory must match the code.** P-033 owns the list and the gaps; until
  `docs/reference/icons.md` exists, quote P-033's spec rather than inventing a count here.
- **A high-density render is not the small-size proof.** `--check-icons` and the sheet are
  read at the sizes the window uses; a larger render can hide a shape that does not read at
  the small one.
