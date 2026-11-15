# Interface plan: the window, stage by stage

Status: **target** — this is the order and the acceptance of the interface work. The
window of `main` is the starting point, not the result: every stage below says what exists
today, what it closes and how that is proven. The capability definitions are in
[capabilities.md](capabilities.md); the contract of the interface is
[ui-ux.md](ui-ux.md).

This plan exists because the sheet is the protagonist and the chrome around it has grown
without a shape: five colour tokens, one breakpoint, a flat catalog, a ribbon that is a
single plane and panels that never leave. A student on a laptop loses drawing room to a
catalog that is always there, and a student on another desktop sees a different icon for
the same command. None of that is a teaching feature, and all of it is in the way of one.

## Decisions taken by the owner

| Date | Decision | Why |
|---|---|---|
| 2026-11-14 | The order is tokens → panels → ribbon → catalog → workspace → adaptive shell → consistency | each stage uses the previous one and leaves a working window |
| 2026-11-14 | Auto-hide is **off by default**; it is enabled in **Ver** and remembered | a panel that hides itself during a demonstration is a surprise, not a feature |
| 2026-11-14 | The hide control is a **project glyph** of two rectangles | the desktop themes have no panel glyph, and a label alone is not a control |
| 2026-11-14 | Panel visibility and width are **remembered**, never saved in the document | it is view state, like zoom and pan |
| 2026-11-14 | The ribbon has **one project family of line icons**, large and small | the same command must look the same on every desktop |
| 2026-11-14 | The adaptive shell is **its own stage**, not part of the panel stage | it changes the layout architecture, not only the chrome |
| 2026-11-15 | The view controls are a **strip floating over the right edge of the drawing area** | it takes no width from the paper and does not fight the right dock; a fourth column was refused |
| 2026-11-15 | **No minimap**; **Ajustar al marco** and the zoom readout take its place | a second drawing of a single sheet is not a map, and it spends a frame budget |
| 2026-11-15 | Fitting means the **frame inside the margins**, on the button and on a **double click of the middle button**; the single press still pans | the drawing space is what the reader came for, and the middle click is where the gesture is looked for |
| 2026-11-15 | **One play/pause button** in the strip, the rest of the transport in **Simular** | the access is worth it during a demonstration; a second transport is not |
| 2026-11-15 | The strip's sizes come from **`unit` and the font**, and the double-click interval from the desktop | a 44 px strip is a screenshot, not a design |
| 2026-11-15 | The adaptive shell is built with **content minimums and one owning layout** | a breakpoint per panel does not survive the next panel |
| 2026-11-15 | **One show toggle, the block**; the grid keeps its own and the border is always drawn | a second grid toggle duplicates **Ver**, and the paper's edge is not optional |
| 2026-11-15 | The **centring marks** follow ISO 5457 §4.3: 10 mm lines beyond the frame, no arrow | the form is normative; "the form is free" was our own note |

## Stages

One stage is one branch and one pull request. A stage that grows past roughly 400 changed
lines outside generated files is split before it starts, never during review. The gate is
`./scripts-dev/check.sh` on the branch that will be merged.

| Stage | Branch | What it delivers | Evidence | Split when |
|---|---|---|---|---|
| 0 | `docs/interface-plan` | this plan, the capability entries, the `ui-ux.md` contract and ADR 0028 | `check-docs.sh` | — |
| 1 | `feat/interface-tokens` | theme tokens, motion token, contrast measurements | `--check-theme`, `--check-qml` | — |
| 2a | `feat/panel-dock` | `DockPanel`, the catalog header toggle, the edge rail and the width animation | `--selftest`, light/dark captures | the right dock is 2a-ii |
| 2a-ii | `feat/right-dock` | the inspector and the title-block editor share the right dock | `--selftest`, captures | the sections need tabs |
| 2b | `feat/panel-autohide` | opt-in auto-hide, hysteresis, remembered state, **Restablecer vista** | `--selftest`, captures | — |
| 3 | `feat/ribbon-depth` | planes, tab and command states, galleries on the same tokens | `--check-theme`, `--selftest`, captures | the galleries move alone |
| 3b | `feat/ribbon-icons-a` | the family's first eight icons, drawn as geometry | `--check-icons`, captures at 16/24/32 | — |
| 3c | `feat/ribbon-icons-b` | the remaining commands of the table | `--check-icons` | — |
| 4a | `feat/catalog-header` | title, count, search, collapsible categories, footer | `--selftest`, captures | — |
| 4b | `feat/catalog-tiles` | two-column tiles, selection, drag, keyboard, resizable width | `--selftest`, captures | — |
| 5 | `feat/workspace-frame` | sunken workspace, paper elevation, one `Card` for the floating editors | captures, contrast, zoom tests | — |
| 6a | `feat/adaptive-shell` | three modes from content minimums, sheet floor, compact rails and drawer, `--check-layout` | `--check-layout`, captures | the rest moves to 6a-ii |
| 6a-ii | `feat/adaptive-rest` | minimized ribbon, remembered size and place, 900 × 600, scale and large-font runs | `--check-layout`, captures | — |
| 6b | `feat/ribbon-minimized` | minimized ribbon, remembered geometry | `--selftest`, captures | — |
| 6c | `feat/view-strip` | the floating view strip, fit to the frame, view history, the three toggles | `--selftest`, captures | the zoom-window gesture if it grows |
| 7 | `feat/interface-consistency` | literals to tokens, focus, tooltips, status bar, reduced motion | the full gate | — |

## Stage detail

### 0 — The specification

Writes the capability rows and entries, the panel and adaptive sections of
[ui-ux.md](ui-ux.md), and [ADR 0028](adr/0028-the-ribbon-draws-its-own-icons.md). No code.
**Done when** `check-docs.sh` is green and the owner has read the contract.

### 1 — Theme tokens and motion

Today the interface exposes five tokens (`background`, `panel`, `foreground`, `muted`,
`accent`) and the widgets fill the gaps with literals: radii of 4, 6 and 8, widths of 248,
360 and 680, a margin of 12. Depth cannot be added without a plane that is not the window
and a line that is not the panel.

- **Tokens**: `surfaceRaised`, `surfaceSunken`, `line`, `lineStrong`, `accentSoft`,
  `shadow` and `unit`, derived from `QPalette` by `theme.cpp` like the existing ones.
- **Motion**: `motionMs`, read once from `[KDE] AnimationDurationFactor`; `0` means no
  animation, and the behaviour still happens.
- **Measurement**: `--check-theme` measures text on the raised surface at 4.5:1 and lines
  at 3:1 against their surface, in light and dark.
- **Does not change**: the drawing ink (`PfInk`), the file format and the document.

**Files**: `src/theme.{h,cpp}`, `src/session.{h,cpp}`, `src/main.cpp`.
**Done when** the gate is green and the light/dark captures show the same window with no
literal colour added.

### 2a — The panel dock

The catalog is 248 px that never leaves; the properties are a floating card; the title
block is a 360 px column; the page setup is a 680 px card. There is no control to hide or
show any of them.

- **`DockPanel.qml`**: a header with a title and a hide control, a content slot, a width,
  and a rail of one unit (24 px minimum, so the control stays a target) when closed. One
  component, used by the catalog on the left and, in 2a-ii, by the right dock.
- **Right dock (2a-ii)**: the inspector and the title-block editor share one column, with
  sections or tabs; the sheet keeps its room. The page setup and the export editor stay
  centered; moving them is a stage 6 decision, not this one.
- **Animation**: width only, `motionMs`, clipped content; reduced motion is instant.
- **Keyboard**: `Ctrl+B` toggles the catalog, `Ctrl+J` the right dock; every control has an
  accessible name and a state word, not colour alone.
- **Not a document change**: no undo entry, no modified mark, no file field.

**Evidence**: `--selftest` steps `panels: the catalog hides and returns`,
`panels: the dock hides and returns`, `panels: hiding is not a document change`; captures
open and closed in both schemes.
**Done when** the sheet gains the width of a closed panel and the gate is green.

**Later (owner, 2026-11-14)**: keep a sliver of the panel visible when it folds instead of
the rail alone, and redraw the control with the icon family of stage 3b. Neither is part of
2a; both are recorded here so a later review does not lose them.

### 2b — Auto-hide and memory

- **Opt-in**: a **Ver ▸ Ocultar al salir** command, off on first start.
- **Hysteresis**: open after a dwell of about 350 ms on the edge rail; close after about
  800 ms without the pointer; leaving and returning cancels the timer.
- **Never during**: a catalog drag, keyboard focus inside the panel, an open dialog or a
  running fast-forward that owns the pointer.
- **Memory**: `QSettings` keys for visibility, width and the auto-hide flag.
- **Restablecer vista**: one command returns panels, zoom and pan to their defaults, so
  remembered state is never a trap.
- **Motion**: the same `motionMs`; reduced motion changes the state without easing.

**Evidence**: `--selftest` steps `panels: the edge opens it after the dwell`,
`panels: focus keeps it open`, `panels: a drag keeps it open`, `panels: the state is
remembered`, `panels: reset restores the default`; captures of the rail.

### 3 — The ribbon with planes

The ribbon paints the tab row and the body on the same colour as the window, separates
groups with a 1 px panel line, and has no pressed state: the only difference between a
command and its neighbour is the glyph.

- The tab row sits on `surfaceRaised`; the body sits on `background`; a `line` closes the
  ribbon and the groups use a fainter one.
- The active tab carries the accent underline and the raised fill; hover is a step lighter,
  pressed a step darker, focus a visible ring.
- The page-setup galleries (`MarginGallery`, `PaperCombo`, `OrientationControl`,
  `GridControl`) use the same card, border and state tokens.
- No invented colour: depth is plane plus line, both derived from the palette.

**Evidence**: `--check-theme`, `--check-qml`, `--selftest`, and captures of every tab in
both schemes, including a disabled and a focused command.

### 3b and 3c — The icon family

Today the ribbon asks the desktop for a `-symbolic` icon and tints it. Breeze is a fine
family but its stroke and corners are not the ribbon's, and another desktop answers with
another family, so the command language changes with the session. ADR 0028 records the
decision: the ribbon draws its own.

- **Grid and ink**: a 24-unit box, 2-unit stroke, round caps and joins, one ink from the
  theme; a filled dot is allowed for a plus or a dot, never a second colour.
- **Resolution order**: project family → desktop `-symbolic` → label alone.
- **Coverage**: every `icon` name in the ribbon table has a drawing; a command with no
  drawing fails `--check-icons` instead of showing a blank.
- **Bound**: the family covers ribbon commands and panel controls, never a component; a
  component is still shown by its own symbol.
- **HiDPI**: rasterise at the requested size times the device pixel ratio, and put the
  ratio in the cache key; a capture at 2× is the evidence.
- **Split**: 3b draws the eight primary commands first and is reviewed before the rest;
  3c completes the table.

**Evidence**: `--check-icons` reports every name as resolved by the project, by the theme
or missing; captures at 16, 24 and 32 in both schemes.

### 4a and 4b — The catalog is a palette

The sidebar is a flat list of 60 px rows with a 12 px title that does not act as a header. It is the
palette, so it has to be searchable and ordered.

- **Header**: **Catálogo**, the entry count, a search field and the hide control.
- **Search**: filters by `name`, `id` and a few synonyms; `Ctrl+F` focuses it; `Enter`
  starts placing the first match.
- **Groups**: the catalog's own categories, collapsible, in a stable order, state
  remembered; the label is translated in `session.cpp`, so the QML does not learn the
  domain enum.
- **Tiles**: two columns, the symbol drawn by the same renderer as the canvas, the name on
  two lines and the `id` beneath; selection is the accent bar plus the word, never colour
  alone; drag and keyboard both place.
- **Footer**: the catalog folder and the skipped-file count, with a link to the panel.
- **Width**: the spec's `18 × grid_unit` as the default, resizable with a handle, bounded.

**Evidence**: `--selftest` steps for the filter count, a collapsed group and a placed tile;
captures at two widths, in both schemes.
**Done when** a component is found by typing three letters and placed without the pointer.

### 5 — The workspace and its cards

The sheet is a rectangle with a 12 px margin on the window colour; every floating editor
has its own radius and width, and none of them reads as being *above* the sheet.

- **Workspace**: `surfaceSunken` behind the sheet, a margin from `unit`, the paper drawn as
  a raised card with a `line` edge and the `shadow` token.
- **`Card.qml`**: one component for the inspector, the page setup, the export, the check and
  the rename panel, with the same radius, border, elevation and title row.
- **No effect on**: zoom, pan, the check or the print. The shadow is drawn, not blurred per
  frame, so the frame budget does not move.

**Evidence**: captures at 1:1 and zoomed, in both schemes; `--check-theme` for the sheet
edge against the workspace; the existing zoom tests still green.

### 6a and 6b — The adaptive shell

At 900 px the ribbon folded, but the catalog and the right dock kept their 324 and 360 px:
the sheet had 180 px and the drawing was unusable. The shell now chooses its mode from the
room it has.

**Landed in 6a.**

- **Modes** from content minimums rather than magic numbers: the sheet keeps 480, the
  catalog 220 and the right dock 300, plus the two canvas margins. `layoutMode` is one
  expression in `Main.qml`, so a new panel does not add a breakpoint. At the default font
  the wide mode starts at 1200 px and the medium one at 1036 px.
- **Medium** narrows both panels towards their minimums and gives the rest to the sheet; at
  1050 px the catalog takes 220 and the dock 314, which is exactly the sheet's floor.
- **Compact** folds both panels to their 24 px rails and gives the sheet the whole width.
  A panel opens as a drawer over the sheet, with the card's edge and shadow, and starts
  folded so a small window does not open with the drawing covered.
- **The sheet's floor holds** in every mode: 480 px at 1050, 1100 and 1200, and more above.
- **`--check-layout`** resizes to 900, 1000, 1050, 1100, 1200, 1280 and 1920 and asserts the
  mode, the floor, that the sheet spans the window in compact and that the two panels never
  overlap; then it opens the drawer and checks it took no column. The boundaries come from
  the live minimums, so a larger font moves them instead of failing the check.
- **A defect found on the way**: a fresh profile opened the catalog at its 220 px minimum,
  because the stored-width fallback was clamped before the theme settled the spacing unit.
  The binding to `18 × unit` now survives when there is no stored preference.

**Landed in 6a-ii.**

- The **minimized ribbon**: a control at the right of the tab row folds it to the tabs, the
  open tab's groups open as a panel over the drawing area only, and a command or `Esc`
  closes it. The state is remembered with the panels, never in the document.
- The **remembered size and place**, guarded so a stored place that no longer lands on a
  screen is dropped, and a maximized window keeps the size it had before.
- The minimum is now **900 × 600** in the code, as the contract said.
- The check runs at **1.25×, 1.5× and 2×** density and with a **1.5× font**, and the
  boundaries move with the unit instead of failing.

**Left for stage 7.** The ribbon's own text sizes and heights are still literals: the
ribbon does not grow with the desktop font, and the panel heights (34, 62, 84) are not
tokens. That pass belongs to the consistency stage, which the capability row records.

**How it is built.** The regions are anchored, not stacked by a `Row`: in compact the
workspace spans the window and the panel floats above it. One expression decides the mode
from the minimums, and no region mixes anchors with `Layout.*`. Thresholds are read in
logical pixels, never from a resolution, and the sheet's minimum outranks a panel's
preferred width. The procedure and the traps are
[qt-adaptive-layout](../.agents/skills/qt-adaptive-layout/SKILL.md).

**Evidence**: `--check-layout` (seven widths plus the drawer, 0 failures), `--check-qml`,
`--selftest`, captures of compact folded, compact with the drawer, medium and wide.

### 6c — The view strip

The reader who is drawing looks at the sheet, not at the ribbon; zooming, fitting and
starting the model should cost no trip to the top of the window. The strip lives **over the
right edge of the drawing area** and takes no width from the paper.

- **What exists today**: `Ctrl` + wheel and the ribbon's three zoom commands, the wheel, the
  middle button and the arrows to pan, `Ctrl+0` for 1:1, the grid toggle, and the block's
  **Sin cajetín** preset — which is a document change, not a view toggle.
- **Fit to the frame**: the zoom is the largest that fits the frame inside the margins, and
  the pan centres it. `Ctrl+0` keeps meaning the paper at 1:1, so the two are different
  readings of the sheet and neither replaces the other. The same fit answers the **double
  click of the middle button**; the single press keeps panning.
- **View history**: one step back, so a zoom at the pointer or a fit can be undone without
  hunting for the previous magnification. It is view state like the rest.
- **The toggle**: one, **Mostrar cajetín**, drawn or not, with no undo entry and no
  modified mark, so it needs a view flag in the session and not a change to the document.
  The grid keeps its toggle in **Ver** and Configurar página, and the paper's edge is always
  drawn.
- **The marks are the standard's**: four continuous lines of 0.7 mm, 10 mm beyond the frame,
  without an arrowhead, on screen and in the print.
- **Transport**: one play/pause button, the same command as `F9`/`F8`, neutral until there
  is a model to run.
- **Sizes**: strip width, row height and icon size from `unit` and the font; the double
  click interval from `QStyleHints`; colours from the theme. No minimap.

**Open question for the owner**: whether hiding the block releases its band to the drawing
or keeps it reserved ([ui-ux.md](ui-ux.md) §3.1).

**Evidence**: `--selftest` steps for the fit, the double click, the view history, each
toggle and the play button; captures in both schemes and at the compact breakpoint; the
keyboard path for every control; `--check-qml`; the zoom self-tests still green.


### 7 — Consistency

One pass over the window: every radius, margin and duration comes from a token; every
interactive item has hover, pressed, focus and disabled; every tooltip names the shortcut;
the status bar groups its readings; reduced motion is honoured everywhere; the docs say
what the window does. The gate, the captures and the changelog close the plan.

## Rules that hold in every stage

- **View state is not document state**: panels, zoom, pan, grid and the ribbon's minimized
  state never enter the file, the undo history or the modified mark.
- **No literal in a widget**: a colour, a radius, a duration or a spacing that is not a
  token is a bug, the same rule [ui-ux.md](ui-ux.md) already states.
- **No third-party material**: the icons are drawn here, the geometry is ours, and no
  product, manual or asset of another program enters the repository
  ([ADR 0009](adr/0009-identity-and-standards.md)).
- **Keyboard and accessibility are part of done**: a control that can only be clicked is
  not finished; contrast is measured, not judged by eye.
- **Reduced motion changes the easing, never the behaviour**.
- **A visible change carries light and dark captures** and, when it moves a budget, the
  measurement.

## Not in this plan

- **Moving the page setup or the export editor into the right dock**: the panel just landed
  centered, and it keeps its own layout; stage 6 decides after the adaptive modes are
  measured.
- **A minimap**: a second drawing of a single sheet is not a map, and it spends a frame
  budget to say what the zoom readout and the fit already say.
- **A navigation wheel, an orbit cube or a cinematic camera**: there is no 3D model to
  orbit, and a radial menu hides the commands it offers.
- **A fourth region**: the window keeps three; a document tab strip below the sheet would
  need an ADR.
- **A brand palette**: the colours stay the desktop's; depth comes from planes and lines.
- **A 3D view or a view cube**: there is no 3D model to navigate.
- **A per-document panel layout**: the file stays about the circuit.

## Risks and conflicts

| Risk | Mitigation |
|---|---|
| `session.*` and `RibbonBar.qml` change often | branch after the previous pull request is merged, rebase, and keep the transport behaviour intact |
| Animating the panel width repaints the sheet | measure during the animation; shorten it or move the reflow to the end if it moves the frame budget |
| Remembered state surprises a student | auto-hide is opt-in and **Restablecer vista** always exists |
| The icon family drifts from the command table | `--check-icons` fails on a missing name |
| The right dock grows into a second window | one column, sections, and the sheet keeps its minimum |
| The page setup pack is fresh | it is not restructured until stage 6, with the owner's decision |

## How to resume

1. Read this page, [capabilities.md](capabilities.md) under F0 and [ui-ux.md](ui-ux.md).
2. Prove the base from Git: `git rev-list --left-right --count main...HEAD` is `0 0` and
   `git log --oneline origin/main..HEAD` is empty before branching.
3. Pick the first stage whose row is not delivered and open its branch; one stage, one
   pull request.
4. Run `./scripts-dev/check.sh` on the branch, attach the captures the stage names, and
   leave the merge to the owner.
