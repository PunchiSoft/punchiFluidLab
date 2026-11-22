# ADR 0028 — The ribbon draws its own action icons

- **Status:** accepted
- **Date:** 2026-11-14
- **Deciders:** project owner, implementation session
- **Context crates:** `apps/punchi-fluid-qt`
- **Current inventory:** [reference/icons.md](../reference/icons.md) — the drawings, their commands and the deferred duplicates

## Context

[ADR 0004](0004-icons-from-the-desktop.md) chose the desktop's own icons over a shipped set,
and [ADR 0005](0005-qt-quick-over-a-c-abi.md) kept that rule when the toolkit moved: the
window asks `QIcon::fromTheme` for a freedesktop name, prefers the `-symbolic` variant and
tints it with the theme's ink.

That is honest and needs no assets, but it makes the ribbon's command language depend on
the session. Two machines with two icon themes draw the same command with a different
stroke, different corners and a different optical weight; a theme with no answer falls back
to the label alone. The ribbon is the one surface where the same action must be recognised
before its label is read, and it is also the surface a student compares between a
classroom machine and their own. A family drawn here can have one stroke, one grid and one
silhouette on every desktop.

The rule that no asset of another program enters the repository still holds: the choice is
between the desktop's icons and drawings made by this project, never a third-party set. A
component continues to be shown by its own ISO 1219 symbol; an action icon is chrome and
never stands for a component.

## Decision

The ribbon's actions use a **project-owned line family**, drawn by this project and
rendered by the application as vector geometry. The rules that follow:

- **Grid and ink.** A 24-unit box, a 2-unit stroke, round caps and joins, and one ink taken
  from the theme. A small filled dot is allowed where a plus or a point is the meaning; a
  second colour is not.
- **Resolution order.** The project family first, the desktop's `-symbolic` icon second,
  the label alone last. A missing drawing is a visible fallback, never a blank button.
- **Coverage.** Every `icon` name in the ribbon's command table has a drawing; a command
  with none fails `--check-icons`. The family covers ribbon commands, the collapsed menu
  and the panel controls. The status bar, the dialogs and any control outside the ribbon
  may keep the desktop's icon, because they are not the ribbon's language.
- **No asset files.** The drawings are geometry in the application, not SVG files loaded at
  run time: the runtime image plugins are not guaranteed on every machine, and a missing
  plugin would turn every icon into a blank. The geometry is data, not a picture, and it is
  rasterised at the requested size times the device pixel ratio.
- **No component icons.** A component is still drawn by `punchi-fluid-draw` from its
  definition; the action family never describes a valve, a cylinder or an instrument.
- **Nothing copied.** The drawings are made here, from the command's meaning, and are not
  traced from another program, a screenshot or a manual.

## Consequences

- The ribbon looks the same on every desktop, in both schemes, and its icons can be
  measured: stroke, bounds and coverage are testable without a screenshot diff.
- The project owns roughly one drawing per command; adding a command means drawing its
  icon, which is the cost that keeps the table complete. The work is split into a primary
  review set and the rest.
- The ribbon deliberately stops following the desktop's icon theme. That is a divergence
  from ADR 0004's reasoning, and this record is the reason: the command language is worth
  more than the theme's variety on the one surface that is always visible.
- `--check-icons` changes from "what the theme answered" to a coverage report: project,
  theme fallback or missing, per name.
- The rendering path must account for HiDPI and cache by pixel ratio; a 2× capture is part
  of the evidence.
- The symbols of the schematic are untouched; the conformance review keeps its own rules.

## Alternatives considered

- **Keep the desktop's symbolic icons and polish the rasterisation.** Cheapest, but the
  stroke, the corners and the optical weight stay the theme's, and a session with another
  theme draws another ribbon. It does not close the gap this record exists for.
- **Draw only the large commands.** Less work, but the same action would look one way in
  the ribbon and another in the collapsed menu, which is generated from the same table.
  Two languages in one control is worse than one family to maintain.
- **Adopt a freely licensed icon set.** It would still need a licence record, a style
  audit and a new supply chain, and its silhouette would have to be redrawn to match the
  ribbon. Drawing the family here is smaller and keeps the origin unambiguous.
- **Ship SVG files.** Easier to edit with a drawing tool, but it adds a runtime image
  plugin dependency and a second format for something the renderer can express as
  geometry.

## References

- [ADR 0004](0004-icons-from-the-desktop.md), superseded by ADR 0005, whose icon rule this
  record amends for the ribbon.
- [ADR 0009](0009-identity-and-standards.md): behaviour may be studied; material is never
  taken.
- [ui-ux.md](../ui-ux.md), the interface contract, and
  the owner's staged plan (private), stages 3b and 3c.
