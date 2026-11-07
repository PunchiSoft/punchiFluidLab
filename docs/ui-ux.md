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
(`document-new`, `edit-undo`, `media-playback-start`), `QIcon::fromTheme` finds it in the
user's theme and rasterises it; a machine with no such icon gets the label alone, as a
native application does. See [ADR 0004](adr/0004-icons-from-the-desktop.md).

The toolbar is a **ribbon**: a tab row, then the commands of the open tab in labelled
groups with a hair line between them, and the transport pinned to the right of the tabs
so running the model never needs a tab switch.

- A **large** command draws an icon over its label, the way a native ribbon draws the
  primary actions; a **small** command draws the icon beside the label. Group names sit
  underneath, which is what makes the row readable instead of a guessing game.
- **No command appears twice.** The transport owns start, pause, step and reset and is
  always visible; a tab does not repeat them. The Simular tab carries what the transport
  does not — the mode and the speed. There is no separate "animate" action: being in
  simulation mode *is* the animation, and its commands are the transport's.
- A command that does not exist yet is **not on screen**. A tab whose work has not landed
  says so in one line. A grey button that teaches nothing is worse than a missing one.
- Every command has a shortcut and a hint that names it; the hint appears on hover after a
  pause and on keyboard focus, and the same string is the accessible description.
- Below 1100 px the body folds away: the tabs and the transport stay, and one **Menú**
  button opens the commands of the open tab. A ribbon without width is a maze.

The list of commands lives in one table in `RibbonBar.qml`; the expanded groups and the
collapsed menu are generated from it, so they cannot drift apart.

| Tab | Contents |
|---|---|
| **Archivo** | Nuevo, Abrir, Guardar, Guardar como, Exportar PNG…, Exportar PDF…, Salir |
| **Editar** | Deshacer, Rehacer, Copiar/Pegar, Duplicar, Borrar, Girar (`Alt+R`), Reflejar (`Alt+M`) |
| **Ver** | Rejilla, Imán a rejilla, Imán a puertos, Zoom (−, %, +), Ajustar a ventana, Tema (sistema/claro/oscuro), Panel lateral, Barra de estado |


The transport (`▶ ⏸ ⏭ ⏹ ⟲` — start, pause, step, stop, reset, with the speed beside them)
is always visible to the right of the tabs: running the model is the most frequent action
and must never need a tab switch.

**The transport changes the mode; there is no mode button.** This is how the reference
behaves, and the reason the tabs are three and not four:

| Command | What it does to the mode |
|---|---|
| **Iniciar** (F9) | enters simulation and starts |
| **Pausa** (F8) | stays in simulation and stops; from editing it *enters* simulation without starting, so settings can be looked at first |
| **Paso** (F10) | one frame, then paused |
| **Detener** (F5) | leaves simulation and returns to editing |
| **Restablecer** (Ctrl+R) | back to `t = 0` |

A separate "Editar" button would be a second way to say what **Detener** already says, and
it would do nothing on its own — which is exactly how it read when it existed.

- **A step is one frame**: the same number of simulated seconds that playing would have
  advanced, so step-by-step and continuous play cannot diverge.
- **A step pauses the transport**, as a simulator's step button does. Stepping while
  running would make the two modes race each other.
- Start and pause are each available only in the state where they mean something, and the
  hint says why when they are not ("ya está en marcha", "ya está en pausa"). The transport
  also shows the state and the simulated time, so the answer to "is it running" is never
  colour alone.

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
- **A pipe is drawn port to port**: press on a port (the pointer turns into a crosshair
  over one), drag, release on another port. The pipe being drawn is a dashed line in the
  accent colour, so the gesture is never invisible; a release that does not land on a port
  cancels it. Pipes are routed as an orthogonal run with one jog, drawn in the line ink.
- A connection is refused, never silently dropped, when it would join a port to itself,
  repeat an existing pipe, or name a component that is not on the sheet. The refusal sets
  the message the status bar shows.
- **Drag from the sidebar to the sheet** creates a component, snapped to the 10 mm grid.
  The component being carried is drawn with its own symbol while it moves, so the drop
  is not a leap of faith. A component dropped before the first frame is kept: the
  starting circuit is added around it, never in its place.
- The document is the sheet: a click selects, dragging moves (one undo entry per drag,
  not per pixel), `Supr` removes, `Ctrl+Z` and `Ctrl+Shift+Z` walk the history, which is
  bounded at 100 entries. The starting circuit is not an edit and cannot be undone.
- A placed component that is not one of the three the model simulates is drawn as its
  symbol and nothing else: it has no physics until connections exist (M4). The interface
  must not pretend otherwise.
- A footer line shows the folder in use and the number of skipped files, with a link
  to the Catalog panel: a broken file must be discoverable without opening a menu.

Instruments (manometer, flow meter, force meter) are catalog entries like the rest;
their behaviour is a `behaviour` variant, not a special case in the sidebar.

### 3. The sheet

**Two modes, and the click means something different in each.** This is the split a
schematic simulator lives by, and the one the reference tool uses:

| Mode | A click on the valve | The model |
|---|---|---|
| **Editar** | opens the properties of the component | frozen; the transport is unavailable and says why |
| **Simular** | actuates the valve, and holds it while the button is down | runs when the transport runs |

- A spring-return valve is **held**: press actuates it, release lets the spring return the
  spool. A valve without a spring **latches** on press. The front end asks the model which
  it is instead of guessing, because guessing makes a lever valve stay actuated after a
  click.
- The pointer changes over a valve and a dashed ring marks the hit area, in the accent
  colour. Every valve in the circuit answers, not only the first one: the click names the
  placement, and the model actuates that valve. An affordance nobody can see is not an
  affordance.
- The properties panel shows **what the definition file says** — name, id, category,
  ports and positions — and edits the numbers the model uses: supply pressure, valve or
  throttle conductance, and a cylinder's stroke, bore, rod and moving mass. Apply is one
  undo entry and the model is rebuilt from the document, so a larger cylinder in one
  circuit leaves the catalog alone. Name, summary, actuation/spring and the remaining
  cylinder parameters are still read-only; the ledger in
  [reference-parity.md](reference-parity.md) names them.

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
…". No message that requires a click to disappear. It ends on the right with the sound
toggle, which is a command like the others: a label, the desktop's own icon and `Ctrl+M`.

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

A single semantic layer, `apps/punchi-fluid-qt/src/theme.cpp`, converts the palette into
`ThemeTokens` once per theme change and pushes them into the drawing as `PfInk`. Widgets
ask the session for `accent` or `muted`, never for a literal. The rule is in `AGENTS.md`:
a hardcoded colour or pixel size in a widget is a bug, and the only literals in the
application are the two forced palettes used to test light and dark.

A colour that does not reach its criterion against the surface is shifted just enough to
reach it (`ensureContrast`), because a desktop may hand the application an accent that
disappears on its own window colour.

### Following the desktop

1. Colour scheme and palette: the Qt platform plugin has already merged
   `$XDG_CONFIG_HOME/kdeglobals` and the `org.freedesktop.appearance color-scheme`
   portal into `QApplication::palette()`. The front end reads that, on start and on
   `ApplicationPaletteChange`, never per frame.
2. Accent: `QPalette::Highlight`, shifted by `ensureContrast` when the desktop picks one
   that cannot be seen on its own window colour.
3. Secondary greys: `QPalette::WindowText` blended towards `QPalette::Window` by 30 %
   and 55 %, then measured; the palette names neither.
4. Font: `[General] font` reaches Qt through the platform plugin; the drawing asks only
   for a cap height, so a font change needs no layout change.
5. Icons: `QIcon::fromTheme`, which reads the same Breeze theme. Icons are only for
   actions; components are shown by their own symbol, drawn by `punchi-fluid-draw`.
6. HiDPI: Qt scales logical pixels; the drawing is in millimetres and the replay applies
   one scale, so a scale change needs no layout change.
7. Non-KDE sessions (GNOME, tiling WMs, no `kdeglobals`): the platform plugin supplies
   its own palette and the same code path applies. The application must look deliberate,
   not broken.

### Accessibility

- Qt exposes roles and names through `QAccessible`/AT-SPI; a `QML_ELEMENT` with a
  `Q_PROPERTY` is reachable, and a custom item needs `accessibleRole` and
  `accessibleName` set explicitly.
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

### Sound

Sound is feedback, not information. Two events make a noise and nothing else does:

| Event | Sample | Why that one |
|---|---|---|
| the spool changes position | `valve.wav` | a real valve is one syllable, and a spring-return one makes it twice — down and back |
| the transport starts | `air.wav` | the volume is being charged; the bed falls away as the model reaches equilibrium |

Rules:

- **Never the only signal.** Every event that sounds also moves the sheet and a readout,
  so a silenced window behaves and reads exactly the same. Silence is a supported mode,
  not a degraded one.
- **The sound follows the model, not the click.** A press the definition refuses to move
  is silent, because nothing happened.
- **Nothing plays before the user touches it.** The window opens already running and
  makes no noise on start; a simulator that greets with a hiss is a surprise, not feedback.
- The toggle is in the status bar, reachable with `Tab` and `Ctrl+M`, and its label says
  the state — `Sonido` or `Silencio` — instead of a colour change.
- The samples are generated by `scripts-dev/make-sounds.py` and compiled in; a WAV of the
  same name in `$XDG_DATA_HOME/punchi-fluid-lab/sounds/` replaces them. See
  [assets/sounds/README.md](../../assets/sounds/README.md).

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
