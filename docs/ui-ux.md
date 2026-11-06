# Interface design

Status: **implemented in M1**, except the two items listed under *Still open* at the
end. The document stays the specification: a change to a region starts here.

Verified against this machine on 2026-11-06: Plasma 6.7.5 on Wayland, `kdeglobals`
with `[Colors:Window] BackgroundNormal=36,36,36` and
`[Colors:Selection] BackgroundNormal=49,91,239`, portal
`org.freedesktop.appearance color-scheme` answering `1` (prefer dark), and Breeze
symbolic icons under `/usr/share/icons/breeze/actions/22/`.

## The idea in one paragraph

Pneumatic diagrams are drawn quickly and read at a glance: the tool should feel like
a drawing program with an instrument panel, not like a CAD package. Top ribbon for
what applies to the whole document, left sidebar that *is* the component catalog,
and everything else is the sheet where the circuit lives and reacts. The sheet is
the protagonist: no dialog in front of it unless the user opened it.

## Regions

```
┌───────────────────────────────────────────────────────────────────────────────┐
│ Archivo   Editar   Ver   Simular   Ayuda                                       │ menu bar
├───────────────────────────────────────────────────────────────────────────────┤
│ ▤ Nuevo  ▤ Abrir │ ▤ Guardar │ ⤓ PNG PDF │ ▶ ⏭ ⟲ │ ⧉      0.25x  Simulación │ toolbar
├──────────────────────┬────────────────────────────────────────────────────────┤
│ 🔍 Buscar…           │                                                        │
│ ▾ Alimentación       │                                                        │
│   ⬤ Aire 6 bar       │                                                        │
│ ▾ Válvulas           │              hoja de simulación                        │
│   ⧉ 3/2 muelle       │        (rejilla, símbolos, tuberías, instrumentos)      │
│   ⧉ 5/2 palanca      │                                                        │
│ ▾ Actuadores         │                                                        │
│   ▭ Cilindro 20/100  │                                                        │
│ ▾ Instrumentos       │                                                        │
│   ◔ Manómetro        │                                                        │
├──────────────────────┴────────────────────────────────────────────────────────┤
│ t 2.506 s · p 4.99 bar · x 100.0 mm · v 0.00 m/s · 4 definiciones · en marcha  │ status
└───────────────────────────────────────────────────────────────────────────────┘
```

The menu bar is window chrome, not a fourth region: it carries the same `Action`
values the toolbar does, so no command exists twice. Its order follows the desktop
(File, Edit, View, Simulate, Help) and it ends with the help menu, whose last entry
opens the about window.

Fixed sizes come from the theme, never from literals: the menu bar is one `row_height`,
the toolbar is `1.4 × row_height` with its icon and label sizes as tokens of their own,
the sidebar
`18 × grid_unit` wide and resizable, the status bar one row. Minimum window 900 × 600; below 1100 px wide the ribbon collapses into a
single menu button plus the simulation transport (a ribbon needs width to work).

### 1. Ribbon (top)

The icons are the **desktop's own**: each action names a freedesktop icon
(`document-new`, `edit-undo`, `media-playback-start`), `punchi-fluid-icons` finds it in the
user's theme and rasterises it; a machine with no such icon gets the label alone, as a
native application does. See [ADR 0004](adr/0004-icons-from-the-desktop.md).

The toolbar is the native shape, as Kate draws it: one row of flat buttons that measure
their own content — never a fixed width per label, which is what lets the text run over
the next button — the icon and the label for the frequent commands (`New`, `Open`, `Save`, `Save as`), the icon
alone with a tooltip for the rest, a thin rule between groups. It is a **selection**:
the menus keep every command, so the toolbar never has to grow into a ribbon. The list
lives in one table in `ribbon.rs` and a test walks it. Every action also has a keyboard
shortcut and a menu entry, so nothing is mouse-only.

| Tab | Contents |
|---|---|
| **Archivo** | Nuevo, Abrir, Guardar, Guardar como, Exportar PNG…, Exportar PDF…, Salir |
| **Editar** | Deshacer, Rehacer, Cortar/Copiar/Pegar, Duplicar, Borrar, Seleccionar todo, Girar 90° |
| **Ver** | Rejilla, Imán a rejilla, Imán a puertos, Zoom (−, %, +), Ajustar a ventana, Tema (sistema/claro/oscuro), Panel lateral, Barra de estado |
| **Simular** | Reproducir, Pausa, Paso, Reiniciar, Velocidad (0,05×–1×), Modo edición/simulación |

The transport (`▶ ⏸ ⟲`) is always visible to the right of the tabs: running the model
is the most frequent action and must never need a tab switch.

Rules:

- Icon + short text for the primary actions, icon only for the rest, with tooltip
  that includes the shortcut.
- Destructive actions (Borrar, Reiniciar con el documento modificado) ask once, not
  twice; they are undoable where the domain allows it.
- No action is disabled without a tooltip explaining why.

### 2. Sidebar: the catalog, not a list of buttons

The sidebar is generated from the catalog, so it is exactly as complete as
`~/.local/share/punchi-fluid-lab/catalog/` plus the built-in files. Adding a valve
file adds an entry; deleting the file removes it; `disabled = true` hides it.

- Groups are the `category` field (`supply`, `valve`, `actuator`, `throttle`,
  `accessory`, `sensor`), collapsible, state remembered per user.
- The thumbnail of each entry is **the symbol of its own definition**, drawn by the
  same renderer as the canvas. No separate icon set can drift from the diagram.
- Search filters by `name`, `id` and a few synonyms; `Ctrl+F` focuses it; typing
  selects the first match and `Enter` starts placing it.
- Drag from the sidebar to the sheet creates a component; double click places it
  centred in the visible area.
- A footer line shows the folder in use and the number of skipped files, with a link
  to the Catalog panel: a broken file must be discoverable without opening a menu.

Instruments (manometer, flow meter, force meter) are catalog entries like the rest;
their behaviour is a `behaviour` variant, not a special case in the sidebar.

### 3. The sheet

- Infinite scrollable surface with a light grid (10 mm) and a heavier one (50 mm).
  The grid is a *guide*, not a background pattern to be proud of.
- Two modes, visibly different:
  - **Edición** (default): select, move, connect, delete. Grid snapping on.
  - **Simulación**: the model runs; clicking a valve actuates it, dragging a
    throttle changes it. Components cannot be moved or deleted, so a stray click
    during a demonstration never destroys work. `Esc` returns to edit mode.
- Selection is drawn with the accent colour plus a 2 px outline; hover only changes
  the cursor and shows a tooltip (never moves anything: a hover that changes the
  document is a known UX trap).
- Pipes are orthogonal; the route is stored as a list of waypoints so it survives a
  move. Ports snap with a magnet radius derived from `grid_unit`.
- Pressurised lines change colour with pressure; the state is never colour-only,
  because the valve symbol also shows its spool position and the status bar prints
  the number.
- Every long operation (> 100 ms) shows progress in the status bar, never a modal.

### 4. Status bar

Time, pressure, position, velocity, air consumed, definition count, mode, and the
last message. It is the place for "saved", "3 files skipped", "export written to
…". No message that requires a click to disappear.

## Visual language

| Token | Source |
|---|---|
| `surface`, `surface_alt`, `surface_raised` | `[Colors:Window]` / `[Colors:View]` / `[Colors:Button]` |
| `text`, `text_muted`, `text_disabled` | `[Colors:Window] ForegroundNormal/Inactive` |
| `accent`, `accent_hover` | `[Colors:Selection] BackgroundNormal` (49,91,239 here) |
| `danger`, `warning`, `ok` | `[Colors:Window] ForegroundNegative/Neutral/Positive` |
| `work_line` | derived from `ok` by pressure ratio, not a new hue |
| flow arrow on a line | same ink as the line; it carries the direction, never the state |
| `grid_minor`, `grid_major` | `text_muted` at 8 % and 16 % opacity |
| spacing | `grid_unit` (18 px in Breeze) and its multiples |
| typography | `[General] font` family and point size; one scale ×1, ×1.15, ×1.3 |

A single semantic layer converts these tokens into `egui::Visuals` + `Style` once per
theme change. Widgets ask for `colour(Token::Accent)`, never for a literal. The rule
is in `AGENTS.md`: a hardcoded colour or pixel size in a widget is a bug.

### Following the desktop

1. Colour scheme: `org.freedesktop.portal.Settings` →
   `org.freedesktop.appearance color-scheme` (`0` no preference, `1` dark, `2` light),
   then `SettingChanged` for live switching. Verified available on this machine.
2. Palette: `$XDG_CONFIG_HOME/kdeglobals` (`[Colors:*]`), respecting
   `LookAndFeelPackage`. Read on start and on theme change, never per frame.
3. Accent: `[Colors:Selection] BackgroundNormal`, falling back to the scheme default.
4. Font: `[General] font=<family>,<point size>,…`; fall back to the default font when
   the family is missing.
5. Icons: prefer `-symbolic` SVGs from `breeze` (found under
   `/usr/share/icons/breeze/actions/22/`), tinted with `text` and cached per
   (name, size, tint). Rasterised through `egui_extras` with the `svg` feature. Icons
   are only for actions; components use their own symbol.
6. HiDPI: `pixels_per_point` from the window; sizes are expressed in points, so a
   scale change needs no layout change.
7. Non-KDE sessions (GNOME, tiling WMs, no `kdeglobals`): portal first, then
   `Visuals::dark()`/`light()`. The application must look deliberate, not broken.

### Accessibility

- `accesskit` is a feature of `eframe` 0.37; enable it in `punchi-fluid-desktop` so
  the ribbon, sidebar and status bar expose roles and names.
- Contrast: text ≥ 4.5:1, lines and icons ≥ 3:1 against their surface, measured in a
  test, not by eye.
- Keyboard: every action reachable without a pointer; `Tab` cycles regions, arrows
  move the selection on the grid, `Del` deletes, `Ctrl+Z`/`Ctrl+Y` undo and redo,
  `Space` starts and stops the simulation, `F1` opens shortcuts.
- State never communicated by colour alone.
- Reduced motion: `[KDE] AnimationDurationFactor = 0` disables every animation; the
  piston still shows its position, it just does not ease.
- Targets ≥ 24 × 24 px; the canvas is exempt because it is a drawing surface, and it
  has keyboard alternatives for everything it does.

### Language

UI strings in Fluent files (`i18n/en.ftl`, `i18n/es.ftl`), component names in the
catalog (`name`, `name_es`). English is the source language; Spanish is the first
translation because it is the project's own. No string concatenation to build a
sentence: a translator must be able to reorder it.

## What this design refuses

- **No modal for the normal path.** Export, save and settings are panels.
- **No hover that changes state.** It was a real bug in the sibling project: cards
  slid under the pointer and selected themselves in a loop.
- **No separate icon set for components.** The symbol is the icon.
- **No colour without a number.** Every colour-coded value also has its value.
- **No hidden mode.** Edit and simulation look different and are named in the status
  bar.

## Still open

- Breeze symbolic icons: action buttons use text today. The theme layer already owns
  the tint and the cache contract, but nothing loads `.svg` yet.
- The font family from `[General] font` is read and stored, not loaded; text size is.
- UI strings become Fluent messages in M6; the two built-in languages already exist as
  `name` and `name_es` in the catalog.
