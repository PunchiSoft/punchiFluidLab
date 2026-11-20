# Inicio: correction list

Every known defect of the start view, one entry each, with the file and symbol to touch,
the acceptance test and the state. The owner reports; the icons, the card layout, the
recent rows and the missing way back are the first four.

Status words: `open`, `in progress`, `done`, `blocked`.

Reference for composition: the sibling CAD's start page, read as behaviour and
proportions (`crates/ui-egui/src/home.rs` and the code that draws it, under
`/home/cristian/Desarrollo/solidcraft`). No code, text or asset is copied: the measures
are translated to this project's tokens first ([ui-ux.md](ui-ux.md) §2.3).

## 1. Icons look bad

- **Symptom**: the icons on the start cards and the rail are small, soft or badly scaled;
  the card's icon reads as a smudge at any window size.
- **Where**: `apps/punchi-fluid-qt/StartView.qml`, every `Image` whose source is
  `image://theme/<name>?tone=…` (the rail rows, the two start cards, the About and
  Licence rows); the provider lives in `apps/punchi-fluid-qt/src/theme.cpp` and its QML
  image provider.
- **Fix**: one size rule for every start-view icon (`Math.round(session.unit * 1.1)`),
  `sourceSize` set to the same value so the provider rasterises at the right density,
  `fillMode: Image.PreserveAspectRatio`, and the tone taken from the theme (`accent`,
  `muted`, `onaccent`), never a literal. Check the provider caches by name, size and
  scheme ([qt-lifetimes](../.agents/skills/qt-lifetimes/SKILL.md)).
- **Test**: capture at 1440 and 1280 in both schemes; `--check-icons` stays at 0 missing.
- **State**: in progress.
- **After #143**: the tones come from the theme and the provider keys by name, size and
  scheme, but one size rule, `sourceSize` and `fillMode` remain, since two sizes (0.9× and
  1.8× `unit`) still upscale from the provider's 22 px fallback.

## 2. The start card's text runs into its icon

- **Symptom**: in the two cards of «Empezar», the title starts over the icon column and
  the card's left edge, and the description is closer to the icon than to its title.
- **Where**: `StartView.qml`, the `Empezar` row and its two `Rectangle` cards (the
  `Column` inside each card and the `Image` before it).
- **Fix**: a fixed icon column (`unit × 2`), a gap (`unit / 2`), and the text column
  anchored **after** it; both cards share the same internal margins, and the description
  is a second line of the same column, not a separate one.
- **Test**: capture; no text overlaps a border or an icon at 1280 and 1440.
- **State**: in progress.
- **After #143**: the icon column, the `unit / 2` gap and the text column after it were
  delivered, but the column is 1.8× `unit` instead of the fixed 2× `unit`.

## 3. Template thumbnails are tiny and washed out

- **Symptom**: the four template cards are large and the drawing inside is a small pale
  sketch in the top half; at a glance the card looks empty.
- **Where**: `StartView.qml`, the `Plantillas` grid and the thumbnail part of the card
  (`templateCard`, around the comment «the thumbnail is the picture of the…»); the
  picture itself comes from `FluidSession`'s thumbnail source.
- **Fix**: the picture takes the **card width minus its padding** and 60–70 % of its
  height, keeps its aspect ratio, and is drawn with the printing ink on the panel, not
  muted. The title and the one-line description sit below it with the same left margin as
  the picture. Cards keep one proportion at every breakpoint.
- **Test**: capture at both widths; each of the four pictures is legible without zoom.
- **State**: in progress.
- **After #143**: the picture follows the card's content width, but it still takes about
  51 % of the height instead of 60–70 %, and the 180×100 mm raster is stretched to the item
  rect, so the aspect is not preserved.

## 4. Recent rows overlap

- **Symptom**: the name and the path of a recent project are printed over each other, two
  entries share one row's height, and «quitar» is not on the row's centre.
- **Where**: `StartView.qml`, `Recientes`: the delegate `recentRow`, its inner `Column`
  and the `forgetButton`.
- **Fix**: one row per project, height from `unit`; the text column takes the row width
  minus the button and its margins; the name and the path are two lines with
  `elide`/`NoWrap`; the path keeps the «no disponible · …» form and both lines use the
  theme's sizes. The button is vertically centred and never overlaps the text.
- **Test**: a session with two recents, one of them missing, shows two rows; a capture
  proves no overlap; a test checks the number of rows and the label.
- **State**: done.
- **After #143**: one row per entry at 1.6× `unit` with the name and the path stacked, the
  «no disponible · …» form kept and the button centred; nothing remains.

## 5. No way back to Inicio from a project

- **Symptom**: after **Nuevo** (or opening a project) the start page cannot be reached
  again.
- **Where**: `apps/punchi-fluid-qt/CircuitTabs.qml` (the strip's row, before the
  `Repeater` of circuits) and `apps/punchi-fluid-qt/Main.qml` (the handler that calls
  `openStartView`); the ribbon command already exists (`RibbonBar.qml`, `id: "home"`).
- **Fix**: a **fixed Inicio tab** at the left of the strip, outside the scroll and never
  removed or renamed, with its own accessible name and `Ctrl+Home`; the ribbon command
  stays as the second path. It is not a circuit: it never enters the list, the file or the
  undo history.
- **Test**: `--selftest`: with a project open the Inicio tab exists, opens the page, and
  **Volver** returns to the sheet; the tab is not counted as a circuit.
- **State**: open.

## 6. Recents: the page's own bound and the missing file

- **Symptom**: the page renders the whole list the core hands it and does not say when a
  file is gone; the core's bound is the only one.
- **Where**: `StartView.qml` `recentPage`; `crates/punchi-fluid-file/src/recent.rs`
  (`MAX_RECENT`, `MAX_PATH`, `MAX_BYTES`); `pf_session_forget_recent` in the ABI.
- **Fix**: the page declares its own top (the same ten), shows a missing path greyed and
  named, offers **quitar** (one call, one change to the list, never to the document), and
  scrolls if the top ever grows. The bound is a constant, not a literal in QML.
- **Test**: `untrusted-input-bounds`'s one-past-the-limit test in the page (eleven
  entries, ten shown); a missing file is greyed and removable.
- **State**: in progress.
- **After #143**: a missing path is greyed, named and removable, but the page still renders
  the ABI's five-entry `PF_RECENT_PAGE_LIMIT` slice with no list scroll; it must declare its
  own constant of ten, the whole `MAX_RECENT` list (owner decision, 2026-11-20).

## 7. Closing the last project should return to Inicio

- **Symptom**: closing the only circuit leaves an empty sheet instead of the start page.
- **Where**: `Main.qml` `requestActiveCircuitRemoval` / `requestCircuitRemoval` and the
  window's `startViewOpen`.
- **Fix**: when the last circuit closes and the project has no unsaved work, open Inicio;
  with unsaved work, the existing confirmation decides first. A project is never left
  without circuits.
- **Test**: `--selftest`: close the only circuit, Inicio is shown; with changes, the
  confirmation appears and cancelling keeps the sheet.
- **State**: open.

## 8. Rail polish

- **Symptom**: uneven item heights, a separator that does not reach the content, hover and
  focus that differ from the catalogue's.
- **Where**: `StartView.qml`, the rail and its repeaters.
- **Fix**: one height per item, one separator, hover from `surfaceRaised`, a visible focus
  ring from the accent only on keyboard focus, names for the accessibility layer, and the
  rail's width from `session.unit`.
- **Test**: `Tab` walks the rail; a keyboard-only run reaches Nuevo, Abrir, About and
  Licence; capture in both schemes.
- **State**: in progress.
- **After #143**: hover comes from `surfaceRaised` and most items are 1.9× `unit`, but
  «Volver» is 1.6× `unit`, the separator sits inside the margins, and the rail has no focus
  ring, keyboard reach or `Accessible.name`.

## Order

1, 2, 3 and 4 are one visual pass (the reported defects). 5 and 7 are one behaviour pass.
6 and 8 close the page. Each pass: one branch, captures light/dark at 1440 and 1280, and
the `--selftest` steps named above.
