---
name: punchi-visual-language
description: Route a visual change in the Punchi window to its canonical document, its command and its evidence: controls, tokens, icons, the ribbon, layout and review. Use when adding or changing a control, a theme token, an icon, the ribbon, a window layout, or when reviewing the visual language.
license: GPL-3.0-or-later
---

# Punchi visual language: route, prove, record

The visual language is specified once, in [docs/ui-ux.md](../../../docs/ui-ux.md); its
colours, sizes and motion are implemented once, in the theme adapter. This skill is the
**procedure** between a request and that contract. It adds no value of its own: a width, a
size, a contrast, a motion or an icon geometry written here would be a second source of
truth, and a second source is a bug.

The diagrams of the documentation belong to **P-030**, and the schematic's symbols belong
to the ISO 1219 contract in [docs/symbology.md](../../../docs/symbology.md). Neither is a
visual-language change: this skill does not touch `crates/punchi-fluid-draw`,
`assets/symbols`, the schematic, the exported page or the symbology review record.

## The rule

**Read the canonical home, run the command, attach the evidence.** A task that cannot name
its document has not been understood yet; ask before inventing a rule. The paper contract is
[docs/ui-ux.md](../../../docs/ui-ux.md); the window's responsive procedure is
[qt-adaptive-layout](../qt-adaptive-layout/SKILL.md), which this skill routes to and does
not repeat.

The diagrams and the ISO symbols are outside the scope even when a change looks “only
visual”. A symbol change goes through
[fluid-power-symbology](../fluid-power-symbology/SKILL.md) and its conformance review; a
documentation diagram is P-030's.

## The link base

A skill file is deeper than it looks, and the documentation check resolves links from the
file that contains them, never from the repository root:

- from this file, [docs/ui-ux.md](../../../docs/ui-ux.md);
- from a reference under `references/`, the same document is written
  `[docs/ui-ux.md](../../../../docs/ui-ux.md)`; the code span is the illustration, the
  references carry the working link.

Write the path from the file that contains the link, then run
`./scripts-dev/check-docs.sh`: it resolves the target and validates the `#fragment` against
the headings of the destination document. A heading rename that is not carried into the
link fails the check.

## Routing by task

| Task | Read (canonical) | Run | Evidence |
|---|---|---|---|
| A new control | [Visual language](../../../docs/ui-ux.md#visual-language); [The window adapts](../../../docs/ui-ux.md#22-the-window-adapts) | `--check-theme`, `--check-qml`, `--check-layout`, `--selftest` | light/dark capture; the manual checklist |
| A token or theme change | [Visual language](../../../docs/ui-ux.md#visual-language); [theme.cpp](../../../apps/punchi-fluid/src/theme.rs) (the names) | `--check-theme`, `--check-layout` | the contrast line; the manual unit, motion and no-literal checks |
| A motion change | [Motion](../../../docs/ui-ux.md#motion); [theme.rs](../../../apps/punchi-fluid/src/theme.rs) (the clock; the Qt adapter retires with ADR 0035) | `--check-theme` (prints `motionMs`); the frame measurement at 200 components | the capture at rest in both themes; the reduced-motion test; the measurement note |
| A new icon | [ADR 0028](../../../docs/adr/0028-the-ribbon-draws-its-own-icons.md); [actionicons.cpp](../../../apps/punchi-fluid/src/ribbon/icons.rs); P-033 | `--check-icons`; `--icon-sheet <file> --theme light` and `--icon-sheet <file> --theme dark` | the rendered sheet; the small-size reading |
| A ribbon change | [The ribbon](../../../docs/ui-ux.md#1-ribbon-top); the P-028 rule it records | `--check-layout`, `--selftest` | the width captures |
| A layout change | [The window adapts](../../../docs/ui-ux.md#22-the-window-adapts); [qt-adaptive-layout](../qt-adaptive-layout/SKILL.md) | `--check-layout`, `--check-qml` | the size captures |
| A review | the task's row | `./scripts-dev/check.sh` | the checklist marked and the evidence linked |

The window checks need a configured Qt build. `./scripts-dev/check.sh` skips the UI step
when `apps/punchi-fluid-qt/build` was never configured, so a green gate alone does not
prove the window: run the checks from the table on a machine where the window is built.
`--icon-sheet` is a manual render, not a gate check: `--icon-sheet <file> --theme light` or
`--icon-sheet <file> --theme dark` writes a sheet for the reviewer to read, and its exit
status proves nothing.

## The checklist

**Machine-checked** — the command and what it actually fails on:

- **contrast**: `--check-theme` exits when `contrast.passes()` is false. That is all it
  asserts: the unit and the motion are **printed, not asserted**, and it cannot see a colour
  literal in a widget. Those three are manual review items; a green command does not prove
  them.
- **QML and bindings**: `--check-qml` fails when a component does not load or a binding is
  broken.
- **layout and the ribbon**: `--check-layout` fails on the measured widths, the modes and
  the ribbon's group row.
- **the gesture and panel path**: `--selftest` fails on the interaction steps it names.
- **icon coverage and use**: `--check-icons` fails on a command with no drawing, a broken
  fallback or a use outside the family.
- **documentation links, the skill row and its anchors**: `./scripts-dev/check-docs.sh`
  fails on a relative link that leaves the tree or does not resolve, a skill folder without
  its catalog row, a `name` that differs from the folder, and an `#fragment` that no
  heading of the target document produces.

**Manual, with evidence** — record the capture or the measurement:

- no widget holds a literal colour, size, radius or motion value; the token is used and the
  accent follows the rule in [Visual language](../../../docs/ui-ux.md#visual-language);
- unit and motion are the theme's, not a literal; `--check-theme` only asserts the contrast;
- the motion follows the intent table in [Motion](../../../docs/ui-ux.md#motion), and a
  document command still changes in the frame it was issued;
- every state exists (normal, hover, pressed, disabled, checked) and none is colour-only;
- the control is keyboard reachable and exposes an accessible name;
- the icon reads at the small render size and stands for one concept;
- the chrome uses the spacing unit and the drawing uses millimetres;
- nothing is copied from another product.

The references carry the detail for each row:

- [a control](references/controls.md);
- [a token or theme change](references/tokens.md);
- [a motion change](references/motion.md);
- [an icon](references/icons.md);
- [the ribbon](references/ribbon.md);
- [the review](references/review.md).

## Gotchas

- **A value copied into this skill is a bug.** Widths, sizes, contrast, motion and icon
  geometry live in `docs/ui-ux.md`, `theme.cpp`/`theme.h`, ADR 0028 or P-033's inventory.
  Link them; never restate them.
- **A heading rename breaks links silently without the check.** `check-docs.sh` validates
  the fragment now; carry the rename into every link in the same change.
- **`--check-theme` asserts contrast only.** It prints the unit and the motion values; a
  green run does not prove them.
- **The token is the duration; the curve is the intent table's.** A new curve, a spring, a
  bounce or a faster exit is a contract change in `docs/ui-ux.md`, never a literal in a
  widget.
- **A configured build is not the same as a checked window.** Without
  `apps/punchi-fluid-qt/build`, `check.sh` skips the window and says so on stderr.
- **`--icon-sheet` is not a gate.** It is a manual render used to look at the family; the
  gate's icon check is `--check-icons`.
- **The token names are declared in `theme.h` and built in `theme.cpp`.** A widget that
  names a colour or a pixel is a bug; only the forced test palettes may hold literals.
- **The layout is `qt-adaptive-layout`'s.** This skill does not restate breakpoints, minimum
  widths or the fold rules; it routes to that procedure.
- **P-033's inventory has not landed as a repository document yet.** Until
  `docs/reference/icons.md` exists, the icon family's inventory is P-033's spec in the EGM
  workspace; this skill links to ADR 0028 and `actionicons.cpp` and says the inventory is
  P-033's.
- **P-030 owns the documentation diagrams; this skill owns the procedure.** A diagram
  change, an ISO symbol change or a review-record change is not a visual-language change.

## Validation

```bash
./scripts-dev/test-check-docs.sh
./scripts-dev/check-docs.sh
./scripts-dev/check.sh
```

The first builds a fixture and proves the three failures the extension exists for (a broken
relative link, a missing catalog row and a wrong anchor) plus the path escapes (`../`,
an absolute path and a symlink) without touching the real tree; the second validates the
real tree, including this skill's links and anchors; the third
is the full gate when the change touches more than documentation. A skill-only change is
documentation: `check-docs.sh` and its fixture are the proof, and no window screenshot is
required. If the change also touches an application file, run the table's commands and keep
the capture with the pull request.
