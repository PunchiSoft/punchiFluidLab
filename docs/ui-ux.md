# Interface design

Status: **the three regions exist; parts of the spec are still open.** The sidebar groups,
search and footer, the full export panel and the missing status readings are target, not
current window; each region says what is there today. The document stays the
specification: a change to a region starts here. The font item under *Still open* is the
only open item of the theme layer itself.

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
│ Archivo  Editar  Ver  Disposición  Cajetín  Simular    Acerca de  Licencia     │ tabs
├───────────────────────────────────────────────────────────────────────────────┤
│ ▤ Nuevo  ▤ Abrir │ ▤ Guardar │ ⤓ PNG PDF │ ▶ ⏭ ⟲ │ ⧉       0.25x  Estado    │ groups
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
│ t 2.506 s · p 4.99 bar · x 100.0 mm · v 0.00 m/s · 14 definiciones · en marcha │ status
└───────────────────────────────────────────────────────────────────────────────┘
```

The tab row is window chrome, not a fourth region: **Acerca de** and **Licencia** sit at
its right end as text, not as tabs, and they stay there when the ribbon folds. They open
a popup about the program, never a command of the circuit, so no command exists twice.

Fixed sizes come from the theme, never from literals: the tab row is one `row_height`,
the ribbon's icon and label sizes are tokens of their own, the sidebar
`18 × grid_unit` wide and resizable, the status bar one row. Minimum window 900 × 600; below 1100 px wide the ribbon collapses into a
single menu button (a ribbon needs width to work).

### 1. Ribbon (top)

The icons are the **desktop's own**: each action names a freedesktop icon
(`document-new`, `edit-undo`, `media-playback-start`), `QIcon::fromTheme` finds it in the
user's theme and rasterises it; a machine with no such icon gets the label alone, as a
native application does. See [ADR 0004](adr/0004-icons-from-the-desktop.md).

The toolbar is a **ribbon**: a tab row, then the commands of the open tab in labelled
groups with a hair line between them. The simulation transport is the **Simular** tab,
selectable like Archivo, Editar and Ver.

- A **large** command draws an icon over its label, the way a native ribbon draws the
  primary actions; a **small** command draws the icon beside the label. Group names sit
  underneath, which is what makes the row readable instead of a guessing game.
- The **Simular** commands are **large commands**, icon over label and 62 px tall, the
  same presentation as Nuevo, Abrir and Guardar: the six transport actions and the three
  speeds are what the hand goes to while a model runs, so they are not small menu rows. The
  speed in force is marked on its button.
- **No command appears twice.** The **Simular** tab owns start, pause, step, hasta cambio,
  stop, reset, the speed and the mode; no other tab repeats them. There is no separate
  "animate" action: being in simulation mode *is* the animation, and its commands are the
  transport's.
- A command that does not exist yet is **not on screen**. A tab whose work has not landed
  says so in one line. A grey button that teaches nothing is worse than a missing one.
- Every command has a shortcut and a hint that names it; the hint appears on hover after a
  pause and on keyboard focus, and the same string is the accessible description.
- Below 1100 px the body folds away: the tabs stay, and one **Menú** button opens the
  commands of the open tab. A ribbon without width is a maze.
- **The two help links are text at the right end of the tab row**, with the same weight
  as the tabs: **Acerca de** opens a popup with the name, the core's version, "versión en
  desarrollo, sin publicar", a one-line description, the repository and the issue tracker,
  the copyright with the full licence and the author. **Licencia** opens a popup with the
  GPL summary and the complete text, which is compiled into the binary. The repository and
  issue links open the desktop's browser on an explicit click; the program itself never
  fetches anything. Both popups close with **Cerrar**, `Esc` or a click outside, and
  opening one never changes the document.

The list of commands lives in one table in `RibbonBar.qml`; the expanded groups and the
collapsed menu are generated from it, so they cannot drift apart. The two help links are
a second, short table beside it, because they are not commands of a tab.

| Tab | Contents |
|---|---|
| **Archivo** | Nuevo, Abrir, Guardar, Guardar como, Exportar (PDF/PNG/SVG) and **Opciones…** (the export settings); the **Circuito** group with the circuit selector, Nuevo circuito and Renombrar; the **Configurar página** group with one large button that opens the detailed sheet editor; and Recientes (the last ten). Salir is **not in the window yet**; the export command opens a save dialog, and the panel keeps only the raster resolution and the parts list — [capabilities.md](capabilities.md) F5 says what remains |
| **Editar** | Deshacer, Rehacer, Copiar/Pegar, Duplicar, Borrar, Todo (`Ctrl+E`), Girar (`Alt+R`), Reflejar (`Alt+M`), Alinear (izquierda, derecha, arriba, abajo, **puertos**), Repartir (horizontal, vertical), Conducto (trabajo, pilotaje, drenaje) |
| **Ver** | Aumentar, Disminuir, Tamaño original, the theme scheme and the contrast summary; `Ctrl+rueda` magnifies at the pointer, the wheel and the middle button pan, and the arrows pan from the keyboard |
| **Disposición** | **Página** with the **Tamaño** combo (the paper's proportions, its normalised name and the common name in parentheses, plus **Personalizado…**), and **Orientación** (Vertical, Apaisado); **Márgenes** with the gallery **Normal (ISO 5457 / NCh 13.Of93)**, **Estrecho**, **Amplio** and **Márgenes personalizados…**, each with a preview of the frame and its four values; **Cuadrícula** with Visible/Oculta and the step (5, 10, 20 or 25 mm, a heavier line every five steps); **Configurar página** opens the detailed editor. The size and margin lists mark what is in force, and each document choice is one undo; the grid is view state, never printed and never undone |
| **Cajetín** | **Contenido** (one `Contenido: X` item opening the editor's Técnico/Escolar/Industrial/Sin cajetín combo), **Posición** (superior derecha, inferior derecha; portrait adds abajo centrado) and **Datos** (Editar datos…, Restablecer); each choice is one undo and the docked editor keeps the sheet visible |
| **Simular** | Iniciar (F9), Pausa (F8), Paso (F10), Hasta cambio, Detener (F5), Restablecer (Ctrl+R), the speed 0,25× / 1× / 2× and the state (edición, en marcha or en pausa, with `t`); the six actions and the three speeds are large commands, like Nuevo, Abrir and Guardar |


The transport is the **Simular** tab. The shortcuts stay, so running the model still never
needs a pointer or a tab switch; the tab is where the commands are found. The speed chosen
is marked and the state is a word, never colour alone.

**The Cajetín tab and its docked editor edit one circuit while the sheet stays visible.**
The ribbon has six tabs — Archivo, Editar, Ver, **Disposición**, **Cajetín**, Simular.
**Cajetín** groups
**Contenido** (Escolar, Técnico, Industrial, Sin cajetín; the current one marked),
**Posición** (top right, bottom right; portrait adds the full bottom band; marked, disabled
under Sin cajetín) and **Datos** (Editar datos…; Restablecer when the block is
personalized). Choosing a model or position is one undo, and choosing content after Sin
cajetín brings the block back in the standard position in the same undo.

**Editar datos…** opens a **docked panel on the right**, not a modal: the sheet's column
shrinks and the live sheet remains visible. Its labelled **content combo** (Técnico,
Escolar, Industrial, Sin cajetín), the labelled **position combo** with its miniature
delegates, and the field rows follow the same content model. The position options follow
the sheet orientation; the content options are always the four. The **Nota:** box is
multi-line, keeps its text while the block is hidden and is not printed yet. Only the model's fields are shown, label and input on
one row, in print order, with the derived sheet `n/N` read-only. **Personalizar** reveals a
show/hide switch per field; hiding keeps the value, collapses its cell and marks the block
**Personalizado**, and **Restablecer modelo** restores the model's set. There is no Save:
every change is one undo and marks the document modified. There is **no material, unit or
projection**; the scale starts at **`S/E`** and stays editable. The industrial block is
180 × 65 mm on A3 and larger and 170 × 45 mm on A4 and A5; the school and technical models
are shorter. Labels elide rather than clip, the value fits inside its cell, and the live
layout reserves the band and clamps a component dropped or moved under it.

**The properties panel edits one instance, and its first field is its name.** The
designation is text, not a number: it is the field that ends up under the symbol and in the
parts list, so the form sends it verbatim and the core trims and bounds it.

**The window opens on the circuit, not on an empty sheet.** The starting composition is
the source bottom left, the service unit beside it, the valve at the middle right and the
cylinder top right, so the air reads left to right along the bottom and rises to the
actuator. The floating captions of the old example are gone from the editable sheet: a
component carries its designation, and the values live in the inspector and the status bar.

**The sheet and the print are configured in two places.** Paper, orientation and margins
are the **document's sheet**, edited in the **Disposición** tab and in **Configurar
página**; a custom size is saved with the document, not kept for one print. The page panel
applies every change at once and each change is one undo, so it has no Aceptar. The export
panel keeps only the raster resolution and the parts list. The export dialog then asks for
the file, and a setting that leaves no room for the drawing is refused with a message
instead of printing an empty page.

**Conducto acts on the selected hose.** A pilot line and a drain line differ by their
pattern before anyone reads a label, so the class is a property of the pipe, the drawing
takes the dashes from it and the file keeps it. The three commands are disabled until a
hose is selected, and the electrical class is not offered until the electrical library
exists.

**Repartir measures gaps between symbol boxes, not centres.** A wide cylinder and a narrow
valve look evenly spaced on centres and are not; the command follows what a reader sees and
keeps the outermost symbols where they are, so the sheet keeps its extent. Three or more
symbols are needed, and the result is one undo.

**A recorder picks its source from the panel.** When the inspected component is one, the
panel lists the quantities the circuit can report — pressure at a port, flow through a
meter, position of a cylinder — with the bound one marked, and one click binds or unbinds
it. The list comes from the core, so the panel cannot offer a source the model cannot
read, and the choice is one undoable edit. State is never colour-only: the marked row also
carries the word "en pantalla".

**One confirmation, and only when it can lose work.** The sheet offers a safety copy when
one is waiting from a previous run, and stops once before Nuevo, Abrir or closing discards
unsaved work. Both are panels beside the sheet, not modal windows, and neither appears on
the normal path: an unmodified circuit closes and opens without a question. The window
title carries `•` while there is work that is not in the file — a character, not a tint,
because a title bar has no other channel.

**The transport changes the mode; there is no mode button.** This is how the reference
behaves, and the reason the commands live in the **Simular** tab and not in a separate
"mode" button:

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

**Today** the sidebar is a flat list of that catalog, each row drawn with its own symbol;
a click selects and a drag places. The collapsible category groups, the search box and
the footer with the catalog folder and the skipped-file count below are the target and are
not in the window yet.

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
- **Drag from the sidebar to the sheet** creates a component, snapped to the grid step (10 mm by default).
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
  [capabilities.md](capabilities.md) names them.

- Infinite scrollable surface with a configurable grid: 5, 10 (the default), 20 or 25 mm,
  with a heavier line every five steps. The step is a **view setting** shared by the ruling
  and the snap, so the lines never lie about where a component will land; it is never
  printed and never enters the undo history. The grid is a *guide*, not a background
  pattern to be proud of.
- **The sheet is the paper.** A new document opens on **A4 landscape** with the ISO 5457
  frame (20 mm on the filing edge, 10 mm on the other three); **Configurar página** returns
  to A4 portrait with the same frame. The **Disposición** tab offers A0–A5, Letter, Legal,
  Oficio and a custom size, the orientation and the margin presets, and **Configurar
  página** edits the four margins in millimetres. The paper, the turn and the margins are
  the document's, saved in format 7 and drawn by the canvas, measured by the check and
  printed. An older file opens as an unframed A4 so its drawing does not move. The common
  names (`Letter (Carta)`, `Legal (Oficio)`, `Oficio chileno (Folio)`) are a display alias
  only; the file keeps `letter`, `legal` and `oficio`. A sheet per circuit is not there yet
  ([ADR 0026](adr/0026-the-sheet-is-the-paper.md)).
- **The frame carries the four centring marks** at the midpoints of the sheet, a 0.7 mm
  line outward and a small arrow, and the **reference zones** of ISO 5457 / NCh 13.Of93
  from A3 upwards: 50 mm fields, numerals down the left and right bands, letters across the
  top and bottom, 3.5 mm characters and 0.35 mm rules, all inside the margin bands. A4
  stays clean. [reference/title-block.md](reference/title-block.md) records the sheet and
  the **control-diagram** block (industrial 180 × 65 mm on A3, 170 × 45 mm on A4; school and
  technical shorter; IEC 61082-1 with ISO 7200 fields, scale `S/E`).
  A schematic is not to scale, so the block has no material, unit or projection.
- **A hose keeps clear of the symbols it joins and the ones in between.** The automatic
  route leaves and enters along the port edge and goes round the endpoint boxes with the
  largest clearance that fits (up to 2 mm). A third component on the way is added to the
  search and the route recomputed, at most three passes and six extras, so the normal route
  pays nothing extra. The check measures exactly that route — port edges and the dragged
  offset included — and reports a hose that crosses any component, its own ends included;
  only the short port lead is ignored.
- **Zoom and pan are view state.** `Ctrl` and the wheel magnify about the pointer in the
  buttons' quarter steps, between 0.4× and 6×; a plain wheel pans and `Shift` swaps the
  axis; the middle button drags the sheet; the arrow keys pan from the keyboard, and
  `Ctrl+0` returns to 1:1 and drops the pan. Above 1:1 the sheet can move until only a
  sliver remains in view, so a zoom at the pointer never throws it away; at 1:1 it stays
  centred. The status bar prints the magnification and the paper's name and size. None of
  it is a document change, so none of it enters the undo history.
- Two modes, visibly different:
  - **Edición** (default): select, move, connect, delete. Grid snapping on.
  - **Simulación**: the model runs; clicking a valve actuates it, dragging a
    throttle changes it. Components cannot be moved or deleted, so a stray click
    during a demonstration never destroys work. `Esc` returns to edit mode.
- Selection is drawn with the accent colour plus a 2 px outline; hover only changes
  the cursor and shows a tooltip (never moves anything: a hover that changes the
  document is a known UX trap).
- Pipes are orthogonal; the route is stored as a list of waypoints so it survives a
  move. Ports snap with a magnet radius derived from the grid step.
- **A hose straightens when its component is dragged near the other end.** Two ports on the
  same axis make a straight run only when their other coordinate matches, and the grid
  cannot reach every offset (the starting circuit has 8.02 mm). Within half a grid step plus 1 mm (6 mm on the default 10 mm grid) the dragged port
  pulls onto the axis, off the grid, and `Alt+A` / **Alinear ▸ Puertos** does the same for the
  selected hose. A hose between perpendicular edges is refused, not guessed.
- Pressurised lines change colour with pressure; the state is never colour-only,
  because the valve symbol also shows its spool position and the status bar prints
  the number.
- Every long operation (> 100 ms) shows progress in the status bar, never a modal.

### 4. Status bar

Time, pressure, position, velocity, air consumed, definition count, mode, and the
last message. It is the place for "saved", "3 files skipped", "export written to
…". No message that requires a click to disappear. It ends on the right with the sound
toggle, which is a command like the others: a label, the desktop's own icon and `Ctrl+M`.

**Today** the bar shows pressure, stroke, velocity, simulated time, the magnification, the
paper's name and size, one hint or error line, and the sound toggle. The air consumed, the
definition count and the mode text are the target, not in the bar yet.

## Visual language

| Token | Source |
|---|---|
| `surface`, `surface_alt`, `surface_raised` | `[Colors:Window]` / `[Colors:View]` / `[Colors:Button]` |
| `text`, `text_muted`, `text_disabled` | `[Colors:Window] ForegroundNormal/Inactive` |
| `accent`, `accent_hover` | `[Colors:Selection] BackgroundNormal` (49,91,239 here) |
| `danger`, `warning`, `ok` | `[Colors:Window] ForegroundNegative/Neutral/Positive` |
| `work_line` | derived from `ok` by pressure ratio, not a new hue |
| flow arrow on a line | same ink as the line; it carries the direction, never the state |
| `grid_minor`, `grid_major` | `text_muted` at 22 % and 34 % opacity |
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
  pan a magnified sheet while the sheet holds the focus, `Del` deletes,
  `Ctrl+Z`/`Ctrl+Y` undo and redo, `Space` starts and stops the simulation, `F1` opens
  shortcuts.
- State never communicated by colour alone.
- Reduced motion: `[KDE] AnimationDurationFactor = 0` disables every animation; the
  piston still shows its position, it just does not ease.
- Targets ≥ 24 × 24 px; the canvas is exempt because it is a drawing surface, and it
  has keyboard alternatives for everything it does.

### Animation on the sheet

Being in simulation mode *is* the animation; a stopped sheet is still, and a paused sheet
keeps the frame it was on. The reference's signature is the circuit in motion, and the sheet
replicates it from the model, never from a timer that guesses.

| What moves | Source | Rule |
|---|---|---|
| Piston and rod | `extension` | Already; the position is the model's |
| Valve spool | valve position | Already; the active square is the model's |
| Contact blade, relay bar, lamp rays | run state | Already; the shape changes, never colour alone |
| **Flow in a hose** | **signed mass flow of that hose** | Travelling chevrons along the route; speed from `|flow|`, direction from the sign; none at zero |
| Instrument reading | the quantity | The number and the needle update with the sample |

- **Direction is the model's.** A pressure difference at a dead end is not flow; the
  chevrons stay still unless the solver reports flow through that hose.
- **Start, pause, reset.** Starting begins the phase at zero; pausing freezes it; reset
  clears it. A single step advances the phase by one step, not by a frame.
- **Reduced motion.** `[KDE] AnimationDurationFactor = 0` replaces the travelling chevrons
  with one static direction arrow; pressure colours and moving parts remain readable.
- **No allocation per frame.** The route is already computed; the phase and the flow are
  scalars, and the chevrons are line segments on the same surface.
- **Performance.** The animation must not move the frame budget; if a long route costs more
  than the static draw, the chevrons are capped per hose, not skipped silently.

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
  [assets/sounds/README.md](../assets/sounds/README.md).

### Language

UI strings in Qt catalogs (`i18n/punchi-fluid_es.ts`, compiled to `.qm`; English is the
source language and needs no file), component names in the catalog (`name`, `name_es`).
The window translates the key the core returns; the core never carries a sentence
([ADR 0027](adr/0027-translation-catalogs-are-qts.md)). Spanish is the first translation
because it is the project's own. No string concatenation and no `(s)` plural: a
translator must be able to reorder the sentence.

## What this design refuses

- **No modal for the normal path.** Export, save and settings are panels.
- **No hover that changes state.** It was a real bug in the sibling project: cards
  slid under the pointer and selected themselves in a loop.
- **No separate icon set for components.** The symbol is the icon.
- **No colour without a number.** Every colour-coded value also has its value.
- **No hidden mode.** Edit and simulation look different and are named in the status
  bar.

## Still open

- The font family from `[General] font` is read and stored, not loaded; text size is.
- UI strings move to the Qt catalogs in M6; the two built-in languages already exist as
  `name` and `name_es` in the catalog ([ADR 0027](adr/0027-translation-catalogs-are-qts.md)).

Breeze symbolic icons are no longer open: `QIcon::fromTheme` finds them in the user's
theme and the theme adapter tints and caches them, and a machine without that theme shows
the label alone, as a native application does. `--check-icons` is the evidence.
