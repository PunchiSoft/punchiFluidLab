# Interface design

Status: **the three regions exist; parts of the spec are still open.** The sidebar tiles,
groups, search and footer, the panels that hide, the ribbon's depth and icon family, the
workspace frame and the adaptive shell are target, not current window; each section says
what is there today. The staged order of that work is
[interface-plan.md](interface-plan.md). The document stays the specification: a change to
a region starts here. The font item under *Still open* is the only open item of the theme
layer itself.

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
│ Archivo  Editar  Ver  Cajetín  Simular                 Acerca de  Licencia     │ tabs
├───────────────────────────────────────────────────────────────────────────────┤
│ ▤ Nuevo  ▤ Abrir │ ▤ Guardar │ ⤓ PNG PDF │ ▶ ⏭ ⟲ │ ⧉       0.25x  Estado    │ groups
├──────────────────────┬────────────────────────────────────────────────────────┤
│ 🔍 Buscar…           │                                                        │
│ ▾ Alimentación       │                                                        │
│   ⬤ Aire 4 bar       │                                                        │
│ ▾ Válvulas           │              hoja de simulación                        │
│   ⧉ 3/2 muelle       │        (rejilla, símbolos, tuberías, instrumentos)      │
│   ⧉ 5/2 palanca      │                                                        │
│ ▾ Actuadores         │                                                        │
│   ▭ Cilindro 20/100  │                                                        │
│ ▾ Instrumentos       │                                                        │
│   ◔ Manómetro        │                                                        │
├──────────────────────┴────────────────────────────────────────────────────────┤
│ t 2.506 s · p 4.00 bar · x 100.0 mm · v 0.00 m/s · 14 definiciones · en marcha │ status
└───────────────────────────────────────────────────────────────────────────────┘
```

The tab row is window chrome, not a fourth region: **Acerca de** and **Licencia** sit at
its right end as text, not as tabs, and they stay there when the ribbon folds. They open
a popup about the program, never a command of the circuit, so no command exists twice.

Fixed sizes come from the theme, never from literals: the tab row is one `row_height`,
the ribbon's icon and label sizes are tokens of their own, the sidebar
`18 × grid_unit` wide and resizable, the status bar one row. Minimum window 900 × 600.
**1366×768 is the support floor** (the entry and education laptop): there the ribbon is
**one row of the seven groups**, and a group that does not fit becomes a titled button
that opens its commands in a non-modal flyout instead of being clipped. The collapse
priority, from the least to the most used, is **Repartir, Alinear, Paleta, Historial,
Circuito, Componente, Conducto**; a group with no command — Paleta's note — is skipped
because it has nothing to fly out. The measured set per width is:

| Width | Collapsed groups |
|---|---|
| 1366 | Historial, Alinear, Repartir |
| 1440 and 1600 | Alinear, Repartir |
| 1920 | none |

At 1280 px wide and below the ribbon folds into a single **Menú** button (a ribbon needs
width to work) while the transport stays visible beside it, and at 125 % scaling a
1366-wide panel is ~1093 logical pixels, so it is the folded form there too; that is the
recorded case, not an exception. The ribbon height does not
change (121 px at the default unit), so the sheet below keeps its room and the A4 page is
not covered. **Today** the window applies that one breakpoint and keeps the catalog at
248 px and the right panels at 360 px; the three modes of *The window adapts* below are
target ([interface-plan.md](interface-plan.md) stage 6).

### 1. Ribbon (top)

The icons are the **desktop's own** everywhere except the ribbon: each action names a
freedesktop icon (`document-new`, `edit-undo`, `media-playback-start`), `QIcon::fromTheme`
finds it in the user's theme and rasterises it; a machine with no such icon gets the label
alone, as a native application does. The ribbon's commands use the project's own line
family instead, so one command looks the same on every desktop
([ADR 0028](adr/0028-the-ribbon-draws-its-own-icons.md)); the family falls back to the
desktop's symbol and then to the label. **Today** every command in the ribbon uses the project family — 46 names, from Nuevo and
Abrir to Alinear, Borrar, Iniciar and Menú — and the desktop icon remains only as the
fallback for a command that had no drawing. The rest of the window keeps the desktop's
own icons. See
[ADR 0004](adr/0004-icons-from-the-desktop.md).

The toolbar is a **ribbon**: a tab row, then the commands of the open tab in labelled
groups with a hair line between them. The simulation transport is the **Simular** tab,
selectable like Archivo, Editar and Ver.

**The ribbon has two planes.** The tab row sits on `surface_raised` and the body returns to
the window plane; a `line` closes the ribbon and a fainter one separates the groups. The
active tab takes the body's colour with an accent underline; a command takes a raised fill
on hover, the sunken one while pressed, and a visible accent ring when it holds the focus.
Depth comes from the palette, never from a new hue.

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
- **A list of circuits is not a ribbon command.** The strip at the foot of the sheet owns the
  list — its order, switching, closing and renaming — and the ribbon offers only the commands
  that act on the circuit in view. One button per circuit says the same thing twice, makes
  the group's width grow with the project (the adaptive breakpoints cannot measure it) and
  leaves the closing and the renaming in one place and the switching in two.
- **One file is a project; one sheet in it is a circuit.** A command, a dialog or a hint that
  acts on the file says *proyecto* — Nuevo proyecto, Abrir proyecto, Guardar proyecto,
  Recientes — and one that acts on a sheet says *circuito* — Nuevo circuito, Renombrar,
  Cerrar circuito. A dialog that says "abrir circuito" while the file holds several promises
  a file per circuit, which is not what the format does.
- A command that does not exist yet is **not on screen**. A tab whose work has not landed
  says so in one line. A grey button that teaches nothing is worse than a missing one.
- Every command has a shortcut and a hint that names it; the hint appears on hover after a
  pause and on keyboard focus, and the same string is the accessible description.
- **A group that does not fit collapses to its own title button** (P-028), not to an
  icon and not to a global menu: at 1366 the row stays unfolded with the seven names, and
  the collapsed group's **name plus a chevron** is the button. Its flyout is non-modal,
  opens with `Enter`/`Space`, closes on `Esc` or a click outside, takes the focus and
  carries the same commands as the expanded group, each with its label and accessible
  name. `Alt+D` still reaches the Conducto group's **Diámetro**, whether that group is
  expanded or in its flyout.
- At 1280 px and below the body folds away: the tabs stay, one **Menú** button opens
  the commands of the open tab (the Conducto group's **Diámetro** included), and the
  **transport stays in the tab row** — Iniciar/Pausa, Paso and Detener. A ribbon without
  width is a maze; the menu scrolls when the window cannot hold every command, so none is
  out of reach, but the three actions a lesson needs must not be behind a menu.
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
| **Archivo** | Nuevo, Abrir, Guardar, Guardar como, Exportar (PDF/PNG/SVG) and **Opciones…** (the export settings); the **Circuito** group with **Nuevo**, **Renombrar** and **Cerrar** for the circuit in view (the strip is the list); one **Configurar página** group in reference order: **Márgenes** (profile gallery), **Orientación** (Vertical/Apaisado dropdown), **Tamaño** (paper list with proportions, canonical and common names, plus **Personalizado…**) and a small page launcher; and **Recientes** as one dropdown (the last ten). Salir is **not in the window yet**; the export command opens a save dialog, and the panel keeps only the raster resolution and the parts list — [capabilities.md](capabilities.md) F5 says what remains |
| **Editar** | Deshacer, Rehacer, Copiar/Pegar, Duplicar, Borrar, Todo (`Ctrl+E`), Girar (`Alt+R`), Reflejar (`Alt+M`), Alinear (izquierda, derecha, arriba, abajo, **puertos**), Repartir (horizontal, vertical), Conducto (trabajo, pilotaje, drenaje, **Diámetro** `Alt+D`) |
| **Ver** | **Vista** (Aumentar, Disminuir, Tamaño original), **Paneles** (Catálogo, Propiedades, Ocultar al salir, Restablecer vista), the theme scheme and the contrast summary; `Ctrl+rueda` magnifies at the pointer, the wheel and the middle button pan, and the arrows pan from the keyboard |
| **Cajetín** | **Contenido** (one `Contenido: X` item opening the editor's Técnico/Escolar/Industrial/Sin cajetín combo), **Posición** (superior derecha, inferior derecha; portrait adds abajo centrado) and **Datos** (Editar datos…, Restablecer); each choice is one undo and the docked editor keeps the sheet visible |
| **Simular** | Iniciar (F9), Pausa (F8), Paso (F10), Hasta cambio (asynchronous; Pausa or Detener cancels it), Detener (F5), Restablecer (Ctrl+R), the speed 0,25× / 1× / 2× and the state (edición, en marcha, en pausa or hasta cambio; the time stays in the status bar); the six actions and the three speeds are large commands, like Nuevo, Abrir and Guardar |


The transport is the **Simular** tab. The shortcuts stay, so running the model still never
needs a pointer or a tab switch; the tab is where the commands are found. The speed chosen
is marked and the state is a word, never colour alone.

**The Cajetín tab and its docked editor edit one circuit while the sheet stays visible.**
The ribbon has five tabs — Archivo, Editar, Ver, **Cajetín**, Simular.
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
are shorter. Labels elide rather than clip and the value fits inside its cell. The live
layout protects the **block's own rectangle** — what it prints on — and not a band: a drop or
a move is clamped only inside its horizontal range, so the rest of the frame, including the
lower left beside the block, is drawing space. `Sin cajetín` frees the frame, and a block
enabled over an existing drawing is a fault the check reports where it happens
([ADR 0030](adr/0030-the-block-protects-its-own-box.md)).

**The properties panel edits one instance, and its first field is its name.** The
designation is text, not a number: it is the field that ends up under the symbol and in the
parts list, so the form sends it verbatim and the core trims and bounds it.

**The window opens on the circuit, not on an empty sheet.** The starting composition is
the source bottom left, the service unit beside it, the valve at the middle right and the
cylinder top right, so the air reads left to right along the bottom and rises to the
actuator. The floating captions of the old example are gone from the editable sheet: a
component carries its designation, and the values live in the inspector and the status bar.

**The sheet and the print are configured in two places.** Paper, orientation and margins
are the **document's sheet**, edited in the **Configurar página** panel opened from
Archivo; the panel owns the margin gallery, the four numeric margins, the custom size and
the grid. A custom size is saved with the document, not kept for one print. The page panel
applies every change at once and each change is one undo, so it has no Aceptar. The export
panel keeps only the raster resolution and the parts list. The export dialog then asks for
the file, and a setting that leaves no room for the drawing is refused with a message
instead of printing an empty page.

**Conducto acts on the selected hose.** A pilot line and a drain line differ by their
pattern before anyone reads a label, so the class is a property of the pipe, the drawing
takes the dashes from it and the file keeps it. The three class commands are disabled until
a hose is selected. The electrical class is not offered as a pipe property: a wire takes it
from its two terminals, and that wire is not a tube.

**The tube is part of the hose, not of the page.** **Conducto ▸ Diámetro** opens a gallery
with **Métrica** (4, 5, 6, 8, 10, 12, 14, 16, 20, 25, 32, 40 and 50 mm), **Pulgada**
(1/8", 5/32", 3/16", 1/4", 5/16", 3/8", 1/2", 5/8", 3/4" and 1") and **Sin especificar**;
the metric and inch entries are separate, so 10 mm is never 3/8". The entry in force is
marked by the value, a check and a border, never colour alone. Choosing one applies the size
to the selected pipe as **one undo**, and `Alt+D` opens the gallery from any tab (bringing
**Editar** forward). A new pneumatic pipe starts at **10 mm**, a documented constant rather
than a page setting; an older sheet reads **Sin especificar**; an electrical wire has no
diameter and shows no reading. The status bar prints `Tubo 10 mm` or
`Tubo Sin especificar`. The line thickens through the 0.25 / 0.35 / 0.5 mm series, capped
below the sheet frame, and the parts list names the tube with its drawn length
([reference/tube.md](reference/tube.md), [symbology.md](symbology.md#pipe-widths-the-tube)).

**The port menu: a closure belongs to the port.** A blind plug or a silencer is fitted
to a port, not dropped on the sheet as a component with its own hose. **A repeated press
on a port** (a double click) opens a non-modal menu anchored at that port; **`Alt+C`** is
the keyboard path and opens it on the selected component's first closable port. The menu
names the port in the display form (`escape 3`), lists the placement's closable ports so
one menu serves the whole component, and offers **Sin cierre**, **Tapón** and — only on an
exhaust — **Silenciador**. The current choice is marked with a check **and** text, and
choosing another is one undo. `Esc` or a click outside closes it.

- **A hose wins.** While the port carries a pipe, **Tapón** and **Silenciador** are
  disabled and the menu says the hose must be removed first: nothing is deleted silently.
  The canvas does not start a pipe on a closed port either, and the document refuses the
  connection with its own message.
- **Mechanical and electrical ports are excluded.** They have no closure, no menu and no
  parts row; the electrical connection waits for P-026, and the menu hides the section
  rather than showing a choice that does nothing.
- **A single press still starts a pipe.** The first press arms the pipe; a release that
  ends where it started is a no-op, not a self-connection error. The repeated press is
  handled before the pipe branch, so the second click never begins a second gesture.
- **The Propiedades panel only reads the closures.** For the selected component it shows
  a **Cierres de puerto** block with one line per port, `escape 3 · silenciador`; the
  editing stays in the menu, and the panel changes when the menu applies a choice without
  a second inspection call. The keyboard path is `Alt+C`, disabled while a text field or
  the start view owns the keyboard.

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
| **Detener** (F5) | leaves simulation and returns to editing; during **Hasta cambio** it cancels the fast-forward immediately |
| **Hasta cambio** | advances one frame per event-loop turn until the first piston end or valve change; Pausa and Detener cancel it, and the window never stops answering the pointer |
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

**Today** the sidebar is the palette: the count, the search, the collapsible categories,
two-column tiles and the footer, with a width the user can drag and the session remembers.
The panel that lists the reasons for each skipped file is still target.

- Groups are the `category` field (`supply`, `valve`, `actuator`, `throttle`,
  `accessory`, `sensor`), collapsible, in a stable order, state remembered per user; the
  label is translated in the session, so the QML never learns the numeric enum.
- The sidebar is **a palette of tiles**, two per row: the symbol, the name on two lines
  and the `id` beneath. The header carries the title, the number of entries and the search
  field; the footer names the folder in use and the number of skipped files.
- The thumbnail of each entry is **the symbol of its own definition**, drawn by the
  same renderer as the canvas. No separate icon set can drift from the diagram.
- **A closure is not a palette entry.** `plug.blind` and `silencer.pneumatic` are closure
  definitions: the menu fits one to a port and the file stores the choice per port, so the
  palette hides them instead of offering a box with its own hose. They stay in the ABI
  catalog, so an old sheet that already places one still opens and simulates; only the
  palette row is gone, and the catalog index the row would have kept is untouched.
  The filter is in the session (`readCatalog`), not in QML, so the row index and the search
  count cannot drift ([ADR 0033](adr/0033-a-port-closure-belongs-to-the-port.md)).
- Search filters by `name`, `id` and a few synonyms; `Ctrl+F` focuses it; typing
  selects the first match and `Enter` starts placing it.
- The footer names the folder the core reads and the skipped files; a panel that lists the
  reasons for each skipped file is still target.
- The sidebar is `18 × grid_unit` wide by default and resizable between its minimum and a
  third of the window; the width is remembered as view state, not saved in the document.
- **A pipe is drawn port to port**: press on a port (the pointer turns into a crosshair
  over one), drag, release on another port. The pipe being drawn is a dashed line in the
  accent colour, so the gesture is never invisible; a release that does not land on a port
  cancels it. Pipes are routed as an orthogonal run with one jog, drawn in the line ink.
- **A junction is the one place where the edge is ambiguous**: its ports share the centre,
  so the route takes the edge from the component or pipe the hose faces — a gauge above a
  tee leaves through its bottom port in a straight vertical stub instead of a jog.
- A connection is refused, never silently dropped, when it would join a port to itself,
  repeat an existing pipe, or name a component that is not on the sheet. The refusal sets
  the message the status bar shows.
- **Dragging a tile owns the pointer until the release.** The palette sits in a scroll area, and a scroll area that takes the gesture past its threshold cancels the carry: the preview disappears halfway when the hand drifts diagonally. The tile prevents that steal, so the gesture survives any path over the panel, the docks and the sheet; the palette is scrolled with the wheel or its bar, never by dragging on a symbol. Releasing outside the sheet still cancels, and the two-step and tolerance work is recorded as the next slice in [capabilities.md](capabilities.md).
- **Right-click or `Shift+F10` on a tile opens its menu**, with **Añadir a la hoja**: the entry is
  placed at the centre of the sheet through the same path as the search, as one undo. The menu
  is offered while editing, names the entry it will add for assistive technology, and the right
  button never starts a carry.
- **Drag from the sidebar to the sheet** creates a component, snapped to the grid step (5 mm by default).
  The component being carried is drawn with its own symbol while it moves, so the drop
  is not a leap of faith. A component dropped before the first frame is kept: the
  starting circuit is added around it, never in its place.
- **A dropped component takes the same box as the templates**: the canonical 20 mm symbol
  square, and the actuator the wider barrel box (`CYLINDER_RATIO`, 2.2, so 44 × 20 mm).
  The symbol is read by shape, so how it arrived never changes its size, and a circuit
  built by dragging reads at the size the start page shows.
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

### 2.1 Panels: view state, not document state

The catalog on the left and the right dock (the properties of one component and the title
block's editor) are **panels**, and a panel is chrome: it can be hidden, widened or
narrowed, and none of that changes the circuit. The page setup and the export editor are
**invoked editors** for now: they open centered, they close, and moving them to the right
dock is a later decision (stage 6 of [interface-plan.md](interface-plan.md)).

- A panel has a **header** with its title and its hide control, and a **rail** at the edge
  while it is closed, one unit wide and never below 24 px, so the control stays a target.
  A hidden panel always has a handle and never becomes a mode the user cannot find.
- The hide control is a **project glyph** of two rectangles with the side that folds
  marked. The desktop icon themes have no panel glyph, and a label alone is not a control.
- Hiding and showing is **animated** with the theme's `motionMs`; with reduced motion it is
  instant. The content is clipped, and the sheet takes the width in the same gesture.
- **Auto-hide is off by default.** When the user enables it in **Ver**, a dwell of about
  350 ms on the rail opens the panel and about 800 ms without the pointer closes it; a
  return cancels the timer. It never hides while a catalog drag is in progress, while the
  keyboard focus is inside the panel, or while a dialog is open.
- **Every panel action has a keyboard path**: `Ctrl+B` for the catalog and `Ctrl+J` for the
  right dock, besides the commands in **Ver**. Each control exposes its name and its state
  to the accessibility layer, and the state is a word, not a colour.
- **Width and visibility are remembered** across sessions. **Ver ▸ Restablecer vista**
  returns the panels, the zoom and the pan to their defaults. The state is never saved in
  the document and never enters the undo history.

Both panels implement this today: the catalog and the right dock fold to their rail,
auto-hide is opt-in, the state is remembered and **Restablecer vista** puts it back. The
adaptive modes that move a panel to a drawer are stage 6 of
[interface-plan.md](interface-plan.md).

### 2.2 The window adapts

The shell chooses one of three modes from the room it has, and the sheet keeps a minimum
before a panel is allowed to cover it:

| Mode | When | Ribbon | Catalog | Right dock |
|---|---|---|---|---|
| **Ancha** | the sheet, the catalog and the dock fit at their preferred widths | full | docked | docked |
| **Media** | the preferred widths no longer fit but the minimums do | full | docked, narrower | docked, narrower |
| **Compacta** | the minimums do not fit (about 1036 px and below at the default font) | tabs and **Menú**; optionally minimized | rail, opened as a drawer | rail, opened as a drawer |

- The minimums are content measurements, not magic numbers: the sheet keeps 480 px, the
  catalog 220 px and the right dock 300 px. At the default font the wide mode starts at
  **1200 px** and the medium one at **1036 px**; a larger font moves both, because the panel
  widths follow the spacing unit.
- **The ribbon can be minimized** by the user, with the control at the right end of the tab
  row: the tab row stays and the open tab's groups appear as a panel over the drawing area
  (never over the panels' headers), closing after a command or `Esc`. The state is
  remembered with the panels. Below the collapse threshold the ribbon already folds to
  **Menú** instead.
- **The first window is screen-aware** (P-028): with no remembered size it opens at
  **1440×900** on a 1080p screen. On a smaller screen it takes the largest size that fits
  the available area minus a two-unit margin, never below 900 × 600, and it is centred in
  the area. A 1366×768 panel therefore opens about **1330×732**, above the 1280 fold;
  1280×800 opens below it, and at 125 % the logical area shrinks and the folded form
  applies.
- **The window remembers its size and place**, and only when the place still lands on a
  screen: a laptop undocked from a second monitor must not reopen off-screen. A remembered
  size is **clamped to the current screen** with the same margin and minimum, not only to
  the minimum, so a window saved on a 1920 monitor reopens inside a 1366 laptop. A
  maximized window keeps the size it had before. The minimum is 900 × 600.
- **The mode boundaries follow the spacing unit**, which follows the desktop font, so a
  larger font moves them. The ribbon's own text and heights are still literals; turning
  them into tokens is stage 7.
- **The mode is a consequence of the limits, not a list of numbers.** Each region declares
  a minimum and a preferred width; wide, medium and compact follow from what fits, so a new
  panel does not need a new breakpoint.
- **The breakpoint is measured on the room the shell has**, in device-independent pixels,
  never on a resolution remembered from a screenshot: HiDPI changes the pixel count without
  changing the window.
- **The sheet's minimum wins.** When a panel's preferred width and the sheet's floor
  disagree, the panel narrows to its minimum and then becomes a rail or a drawer; the
  drawing is the region that never gives way.
- **Layouts or anchors, never both in the same item.** A region placed by a layout must not
  set `x`, `y`, `width` or `height`, and an anchored region must not declare `Layout.*`:
  mixing them produces geometry conflicts and binding cycles.
- **Today** the stage is built: `layoutMode` derives the three modes from the content
  minimums, the sheet never drops below 480 px, the compact window folds both panels to
  their 24 px rails and opens them as drawers, the ribbon folds to its tab row, the window
  opens at the screen-aware size and remembers its size and place. `--check-layout` proves
  it at seven widths, with the drawer open, with the ribbon folded and with the geometry
  restored, at 1.25×, 1.5× and 2× density and with a 1.5× font
  ([interface-plan.md](interface-plan.md) stage 6a). `--selftest` proves the first-window
  sizes and the restored clamp with synthetic screen areas, so it does not depend on the
  machine's own screen.

### 2.3 The start view: Inicio

**With no project open, the drawing region holds the start view**, not an empty sheet: it is
not a fourth region and not a ribbon tab, because the ribbon holds commands and the start view
holds the decision of what to work on. **Inicio is a page, not a document**: while it is open
the ribbon and the status bar are hidden instead of greyed, and the page fills the window from
the top edge to the bottom, including the 1366×768 support floor. The view closes when a
project opens ([ADR 0031](adr/0031-the-program-opens-on-a-start-view.md)).

**The left region carries the start navigation** while the start view is shown: the catalog's
column holds a rail with **Iniciar** — **Inicio**, **Nuevo proyecto**, **Abrir…**, each with its
symbol and the `Ctrl+N`/`Ctrl+O` hint — and **Aprender** — **Acerca de** and **Licencia** — instead
of a second sidebar. **Atajos** and **ISO 1219** need a panel and content of their own and stay
out until a node owns them. The right dock is hidden, since with no project it has no inspector
and no block. The commands that need a document (`Guardar`, `Exportar`) are not shown greyed:
they appear with the project. `Inicio` is reachable from **Archivo ▸ Inicio**, from the
**strip's home control** at the foot of the sheet, and from `Ctrl+Home`; with a project open the
view covers the sheet and **Volver a la hoja** returns to it, and with none it is the only view.
`Esc` returns as well, and the page takes it before anything behind it. The rail is the same
region, the same width and the same tokens as the catalog; returning to the sheet returns the
catalog with its remembered width.

**The page is read in planes.** The rail and the page sit on the window, and the page is one
plane: **Empezar**, **Plantillas** and **Recientes** are section labels, and each choice is a
card of its own, as in the suite's other start pages — no box wraps a whole group. The
default way in, **Nuevo proyecto**, is tinted with `accentSoft` and bordered with the accent;
its text stays in the ink, so a hover cannot repaint a fill under a light-on-dark word.
**Abrir…** is the quieter secondary card. The rail keeps the catalog's width and tokens,
gives each row its symbol and its shortcut, and marks the current view with a filled accent
row whose text and icon are the measured `onAccent`.

**Today** the program opens on Inicio in a normal run (the automated window checks keep the sheet), and the page closes with **Volver a la hoja**, with **Esc** or with **Nuevo**. Inicio takes the whole window: the ribbon and the status bar are not shown, the catalog and the right dock are hidden, and the rail takes their place; the window title names the program, never an untitled document. It offers, in reading order:

- **Nuevo proyecto** and **Abrir…**, followed by **Plantillas** and **Recientes**. The page
  lists at most `PF_RECENT_PAGE_LIMIT` (five) of the ten remembered, newest first, with the
  name, the folder, the **modified** relative time (its absolute value in the tooltip) and the
  **real size** read from the file's metadata — the file is never opened or parsed. When more
  than five are remembered, a line says the rest are in **Archivo ▸ Recientes**, whose group
  already lists all ten. A document whose file has gone stays in the list in muted ink, marked
  **no encontrado**, cannot be opened, and leaves it only through its **quitar** control, which
  is a button with an accessible name and does not touch the document, the history or the
  modified mark.
- **An empty Recientes** says the next step instead of showing a blank table: *Aún no hay
  proyectos recientes. Crea un proyecto nuevo o abre un archivo; aparecerá aquí.*
- **Plantillas**, **three** circuits offered and authored for this project: a 3/2 valve and
  single-acting cylinder (the worked example), a 5/2 valve and double-acting cylinder, and air
  preparation ahead of a valve. The fourth compiled template — an electrical pushbutton driving
  a solenoid valve — stays in the ABI for **P-026**, but its card is not offered while that
  connection is deferred. Each card is a keyboard-focusable button; choosing one copies its
  circuit into a clean project, while **Nuevo proyecto** remains empty.
- **Aprender**: **Acerca de** and **Licencia**. The shortcuts and what “conforme a ISO 1219”
  means need their own content and are not invented in this pass.

The example cards **divide the page's width between them**, with one unit between cards and
a minimum of 10 × `unit` before the grid starts a second row. Each card's height follows its
width: the plate is the content width over the raster's 512:288 aspect, and the card is the
plate over 0.65, so the plate is 65 % of it and the proportion holds at every breakpoint. At
the default 18 px unit and a 1280 px window a card is about 342 × 278 px with a plate of
about 321 × 181 px. Each card is a picture and its label: the picture fills the plate width
and keeps its aspect, with a transparent band for the 1.2 % difference between the 512 × 288
raster and the authored 180 × 100 mm area; the plate is `panel` on a `surfaceRaised` card
(`surfaceSunken` while pressed, `accent` border on hover and focus), and the name is
`textLabel` with the detail on one elided line below it, both with the picture's left margin,
plus a tooltip that names the circuit. The default way in (**Nuevo proyecto**) carries the
accent tint and border, and its text is ink, never a light word over a fill that hover
repaints.
Each thumbnail calls the same Rust drawing and Qt replay path as the sheet, then keeps one
512 × 288 RGBA image until the theme changes or the card is destroyed. The three offered cards
bound that cache at 1.69 MiB (2.25 MiB if the fourth returns); no image is fetched. The
start-view icons follow **one raster rule with two sizes**: `Math.round(unit * 1.1)` on the rail
and `Math.round(unit * 1.8)` on the two «Empezar» cards, each with `sourceSize` at that size so
the provider rasterises it and `fillMode: Image.PreserveAspectFit`; the tone is always a theme
token. Every
`image://theme` URL carries `&v=<session.themeRevision>`: a palette change increments the
revision and QML re-requests the icons, so the old tint never stays on screen. Template cards
and recent rows expose their title/name and description to assistive technology. Inicio is
reachable from **Archivo ▸ Inicio**; the domain chip, the recent thumbnail and the opened time
are deferred to P-034b, with a preview-by-path bound and a `recent.json` version change.

### 3. The sheet

**The sheet is a raised card on a sunken workspace.** The canvas fills the drawing area
inside **half a `unit`** of margin over `surface_sunken`; the paper is filled with `panel`, edged
with `line_strong` and carries the `shadow` token offset once behind it. The drawing's own
background follows the paper, so a symbol that erases against the page matches. Every
floating editor is one `Card` with the same radius, edge and shadow; the shadow is drawn,
never blurred per frame, and none of it changes the zoom, the pan, the check or the print.

**An empty sheet says what to do.** With no components it carries a short invitation — a
first line *Arrastra símbolos a la hoja para empezar* and a second one naming the catalog and
the start view — instead of a sentence about the state. It is text, not an illustration, and
it disappears with the first component.

**The circuits sit at the foot of the sheet.** A strip inside the drawing region — not a
fourth region — shows one tab per circuit with its icon and its name, the active one marked by
a background and an accent rule at the seam, never by colour alone. **Inicio sits at the left
of the strip**, before the scroll: a compact icon control with a tooltip and an accessible
name, separated by a rule from the project. It is the way back, not a circuit, so it never
enters the list, the file or the undo history; `Ctrl+Home` from the sheet is the same command,
and both close Inicio with **Volver a la hoja**. A click switches the
circuit — view state, no undo entry — and `Ctrl+Tab` walks them, scrolling the strip so the
active tab is in view; `F2` or a double click renames **in the tab**, with `Enter` to save and
`Esc` to cancel, which is a document edit and one undo like any other. The circuit tabs and
the plus are keyboard reachable and named; the strip scrolls under the fixed home.

**Closing a tab removes the circuit**, with everything it holds, as one undoable edit. The
close control is on **every tab except the last** — a project without circuits is not a
project — and it sits at the right of the name with its own hit area and an accessible name.
A circuit with components asks first, naming it, and offers Cancelar; an empty one closes at
once. The circuit in view hands over to its right neighbour, or to the left one when it was
the last of the strip. `+` sits **right after the last tab**, as a browser does, scrolls with
them and disables at the project limit of **32**, with the limit in its help; the strip
scrolls horizontally instead of growing into a second row.

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
  circuit leaves the catalog alone. A read-only **Cierres de puerto** block lists each port
  with a closure, `escape 3 · silenciador`; the edit itself is the port menu. Name, summary,
  actuation/spring and the remaining cylinder parameters are still read-only; the ledger in
  [capabilities.md](capabilities.md) names them.

- Infinite scrollable surface with a configurable grid. The **Configurar página** panel
  offers 5 (the default), 10, 20 or 25 mm,
  with a heavier line every five steps. The step is a **view setting** shared by the ruling
  and the snap, so the lines never lie about where a component will land; it is never
  printed and never enters the undo history. The grid is a *guide*, not a background
  pattern to be proud of.
- **The sheet is the paper.** A new document opens on **A4 landscape** with the ISO 5457
  frame (20 mm on the filing edge, 10 mm on the other three); the **Configurar página**
  panel returns to A4 portrait with the same frame. The **Disposición** dropdown in the
  Archivo group offers A0–A5, Letter, Legal, Oficio and a custom size; the panel adds the
  orientation, the margin profiles, the four numeric margins and the grid. The paper, the
  turn and the margins are
  the document's, saved in format 7 and drawn by the canvas, measured by the check and
  printed. An older file opens as an unframed A4 so its drawing does not move. The common
  names (`Letter (Carta)`, `Legal (Oficio)`, `Oficio chileno (Folio)`) are a display alias
  only; the file keeps `letter`, `legal` and `oficio`. A sheet per circuit is not there yet
  ([ADR 0026](adr/0026-the-sheet-is-the-paper.md)).
- **The frame carries the four centring marks** at the midpoints of the sheet, a 0.7 mm
  continuous line outward with no arrow, and the **reference zones** of ISO 5457 / NCh 13.Of93:
  four sides from A3 upwards (50 mm fields, letters `A`, `B`, … down the left and right bands
  and numerals `1`, `2`, … across the top and bottom, skipping `I` and `O`) and the
  standard's two-sided form on A4 (numerals on the top band, letters on the right band);
  3.5 mm characters and 0.35 mm rules, all inside the margin bands. A5 and the non-ISO
  sheets stay clean. [reference/title-block.md](reference/title-block.md) records the sheet and
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
- **Selection is the symbol's own lines in the accent colour, not a box around it**: the
  reference marks the component, and an extra rectangle is a second drawing that says
  nothing the symbol does not. The inspector names what is selected, so the state is never
  colour alone. Hover only changes the cursor and shows a tooltip (never moves anything: a
  hover that changes the document is a known UX trap).
- Pipes are orthogonal; the route is stored as a list of waypoints so it survives a
  move. Ports snap with a magnet radius derived from the grid step.
- **A hose straightens when its component is dragged near the other end.** Two ports on the
  same axis make a straight run only when their other coordinate matches, and the grid
  cannot reach every offset (the starting circuit has 8.02 mm). Within half a grid step plus 1 mm (3.5 mm on the default 5 mm grid) the dragged port
  pulls onto the axis, off the grid, and `Alt+A` / **Alinear ▸ Puertos** does the same for the
  selected hose. A hose between perpendicular edges is refused, not guessed.
- Pressurised lines change colour with pressure; the state is never colour-only,
  because the valve symbol also shows its spool position and the status bar prints
  the number.
- Every long operation (> 100 ms) shows progress in the status bar, never a modal.

### 3.1 The view strip

The commands used while reading the sheet — moving, zooming, fitting, hiding a guide and
starting the model — are a **strip that floats over the right edge of the drawing area**,
not a fourth column and not a floating window: it takes no width from the paper, it does
not fight the right dock, and it leaves with the room it needs.

| Control | Behaviour |
|---|---|
| **Mover** | A visible mode: the pointer becomes a hand, the button stays marked, and `Esc` or the same button ends it. The wheel, the middle button and the arrows keep working in either mode, and entering the simulation leaves the mode, because a click must actuate there. |
| **Zoom** | One button that opens a short panel: `−`, `+`, **Zoom a una zona** (drag the area; one use, `Esc` cancels) and **Anterior** (back to the view before the last zoom or fit). |
| **Ajustar a la hoja** | Fits the **whole sheet**, margins included: the paper is what the reader asked to see, edges and all. It is also the **double click of the middle button**, which is where a schematic reader looks for it; the single press still pans. Because 100 % means the same view, the panel no longer offers both. |
| **Mostrar cajetín** | One toggle over what is drawn on screen, and nothing else: the block is view state, like the zoom, with no undo entry, no modified mark and no file field. |
| **Iniciar / Pausa** | One button, the same command as `F9`/`F8`; neutral until the model runs, and the rest of the transport stays in **Simular**. |

- **No literal in the strip.** Its width, height, icon size and row height come from the
  spacing unit and the font; its colours from the theme tokens; the double-click interval
  from the desktop's `QStyleHints`, so a slower double click is the one the system already
  taught the user.
- **Nothing is duplicated.** Every button calls the same action as the ribbon and the
  keyboard path; the strip is a second place to reach the command, never a second command.
- **One toggle, not three.** The grid already has its toggle in **Ver** and in
  Configurar página, and the paper's edge is always drawn: it is where the sheet ends and
  the print carries it.
- **The border is normative, not decoration.** The paper's edge carries four centring
  marks, and ISO 5457 §4.3 fixes them as continuous lines of 0.7 mm, 10 mm beyond the
  frame, without an arrow; no view toggle removes them.
- **It folds.** The user can fold it to one button, the state is remembered as view state,
  and below the compact breakpoint it folds on its own. Reduced motion makes the fold
  instant.
- **No minimap.** A second drawing of a single sheet is not a map, and it costs a frame
  budget to say what the zoom readout and **Ajustar a la hoja** already say.
- **Today** the strip exists: it floats over the right edge of the drawing with **Ajustar a
  la hoja**, a **zoom panel** (−, +, vista anterior), **Mostrar cajetín**, the single
  **Iniciar/Pausa**, **Mover** (a visible hand mode) and the fold control, and it is
  remembered as view state. The middle button pans and its **double click fits the sheet**,
  one step of view history is kept, the **zoom window** is one drag from the panel, and
  `Ctrl+0` returns to 1:1. All of stage 6c is built.

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
| `line`, `line_strong` | `text_muted` blended towards the surface and measured; a line that carries meaning reaches 3:1 |
| `shadow` | the ink at low alpha, drawn once behind the paper, never a per-frame blur |
| `text`, `text_muted`, `text_disabled` | `[Colors:Window] ForegroundNormal/Inactive` |
| `accent`, `accent_hover` | `[Colors:Selection] BackgroundNormal` (49,91,239 here) |
| `on_accent` | `[Colors:Selection] HighlightedText`, or the better of black and white when it cannot reach 4.5:1 on the accent; the ink of a filled accent control, such as the active rail row |
| `danger`, `warning`, `ok` | `[Colors:Window] ForegroundNegative/Neutral/Positive` |
| `work_line` | derived from `ok` by pressure ratio, not a new hue |
| flow arrow on a line | same ink as the line; it carries the direction, never the state |
| `grid_minor`, `grid_major` | `text_muted` at 22 % and 34 % opacity |
| spacing | `grid_unit` (18 px in Breeze) and its multiples, with `unit` for the panel margins |
| typography | `[General] font` family and point size; `textDisplay` is the page title of a view that is not a document, about 1.5 × the spacing unit and one display step above the label |
| motion | `motionMs`, read from `[KDE] AnimationDurationFactor`; `0` disables every animation and keeps the state change ([Motion](#motion)) |
| icons | the ribbon's own line family, one ink; the rest of the window the desktop theme ([ADR 0028](adr/0028-the-ribbon-draws-its-own-icons.md)) |

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
5. Icons: `QIcon::fromTheme`, which reads the same Breeze theme, for the whole window
   except the ribbon, whose commands use the project's line family
   ([ADR 0028](adr/0028-the-ribbon-draws-its-own-icons.md)). Icons are only for actions;
   components are shown by their own symbol, drawn by `punchi-fluid-draw`.
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
- Keyboard: every action reachable without a pointer. `Tab` cycles regions, arrows pan a
  magnified sheet while it holds the focus, `Del` deletes, `Ctrl+Z`/`Ctrl+Shift+Z` undo and
  redo, `F9`/`F8`/`F5` start, pause and stop, and `Space` holds the valve while it is down.
  The document keys follow the desktop's tabs: **`Ctrl+N` a new project, `Ctrl+T` a new
  circuit, `Ctrl+W` closes the circuit in view, `Ctrl+Q` quits, `Ctrl+Tab` and
  `Ctrl+Shift+Tab` walk the circuits, `F2` renames the one in view.** `Ctrl+W` closes a
  sheet, never the window: a pointer cannot lose a project by closing what it is looking at.
  While a circuit is being renamed or a close waits for an answer, `Ctrl+T` and `Ctrl+W` stay
  out of the way: the dialog owns the keyboard until it closes.
- State never communicated by colour alone.
- **Panels**: the hide control and the edge rail are reachable with `Tab`, expose their
  name and their state, and hiding a panel never leaves the focus inside it. Auto-hide is
  off by default, and a panel that is hidden still has its handle and its shortcut.
- Reduced motion: `[KDE] AnimationDurationFactor = 0` disables every animation; the
  piston still shows its position, it just does not ease, and a panel opens or closes
  without the width animation.
- Targets ≥ 24 × 24 px; the canvas is exempt because it is a drawing surface, and it
  has keyboard alternatives for everything it does.

### Motion

Motion shows a change of place or state that the eye would otherwise lose. It is never
decoration and never a delay. The reference app decides *what* moves; this contract decides
*how*, and `motionMs` decides *how long*.

**What moves.** Only the chrome: a panel that folds, opens or yields its width, the rail
beside it, a dialog and a popover. The sheet has its own motion, which belongs to the model
([Animation on the sheet](#animation-on-the-sheet)).

**What never moves.** Selection, a drag, the pan, the zoom, the routing of a hose, the
deletion of a component and every command the keyboard issues to the document. They are
used hundreds of times a day, and a tween between the pointer and the result is a delay the
reader pays on every operation. Type is never tweened: its size, weight and spacing change
in a frame.

**One duration, one curve per intent.** `motionMs` is the duration of every chrome
transition; it comes from the desktop ([Visual language](#visual-language)) and no widget
owns a second one.

| Intent | Curve | Used by |
|---|---|---|
| Entering, leaving, folding | decelerating: fast at the start, settling at the end (`cubic-out`) | a panel, the rail, a dialog, a popover |
| Moving while it stays on screen | accelerates and then brakes (`cubic-in-out`) | a control that changes place |
| Progress or constant motion | linear | a progress indicator, a hold |
| Hover and press | the same `motionMs`, no overshoot | a control's colour |

A spring, a bounce or an overshoot is not used anywhere: a simulator does not move what the
model did not move, and a bounce is the interface imitating physics it does not have. An
exit is not faster than its entrance: the desktop's factor is the only knob, and a second
duration for leaving would be a second token by another name.

**Paired elements share one transition.** A panel and its rail, a dialog and the band
behind it: if they move as one thing they take the same curve and the same `motionMs`, or
the pair visibly comes apart.

**Interruption replaces, never restarts.** A transition that is reversed or retargeted
starts from the value it holds at that moment, not from zero, so a reader who changes their
mind sees one movement and no jump back. The result under the pointer is never held back
while a transition runs.

**Reduced motion removes the tween and keeps the change.** `[KDE] AnimationDurationFactor
= 0` makes `motionMs` zero: every state change happens in the frame it was commanded, no
transition runs and no frame is requested for one. The piston still shows its position, the
panel is already folded, and the focus moves where it would have moved. Nothing is delayed
and nothing is left half-drawn; the sheet's static arrowhead is the same rule
([ADR 0034](adr/0034-the-sheet-animates-the-models-flow.md)).

**Motion is view state.** It never enters the document, the file, the undo history or the
modified mark. A fold, a dialog and a hover change nothing that can be saved, and reopening
the document does not replay them.

**The frame budget does not move.** A transition is event-driven: it requests the frames it
needs and none after it settles, so the transport's clock stays the only continuous one. An
animated width or height lays the window out on every frame, and a panel width reflows the
sheet; the cost is measured at 200 components before it ships, and a transition that misses
the budget is shortened or moved to the end of the gesture. The theme check prints
`motionMs`; it does not prove that a widget used it, so the reduced-motion branch is a test
in the adapter.

### Animation on the sheet

Being in simulation mode *is* the animation; a stopped sheet is still, and a paused sheet
keeps the frame it was on. The reference's signature is the circuit in motion, and the sheet
replicates it from the model, never from a timer that guesses. The chrome's transitions are
the [Motion](#motion) contract; this section is only the sheet's.

| What moves | Source | Rule |
|---|---|---|
| Piston and rod | `extension` | Already; the position is the model's |
| Valve spool | valve position | Already; the active square is the model's |
| Contact blade, relay bar, lamp rays | run state | Already; the shape changes, never colour alone |
| **Flow in a hose** | **signed mass flow of that hose** | A **stream of solid arrowheads** spaced along the route, each on a short stroke; pace from the hose, direction from the sign; none at zero |
| Instrument reading | the quantity | The number and the needle update with the sample |

- **Direction is the model's.** A pressure difference at a dead end is not flow; the stream
  stays still unless the solver reports flow through that hose.
- **The head is the standard's, the stroke is ours.** ISO 1219-1 marks direction with a solid
  arrowhead, and it is the same primitive the symbols and the fixed example already draw
  (`draw_flow_arrow`); the stroke is what says the air moves, drawn a little thicker than the
  hose or it would vanish into the line. Neither is traced from another program.
- **A stream, never one mark.** One mark has to restart at the source and that restart is a
  visible jump; a stream lets an arrowhead enter while another leaves, so the hose is never
  empty while the air moves. The marks are an exact lattice, so they never bunch where the
  travel wraps.
- **The air moves with the flow.** The stream's speed follows `|flow|` between a floor and a
  cap — a trickle drifts, a strong current runs — so the sheet's ordering is the model's; a
  hose long enough takes longer to cross. The numbers are a readability scale, not the
  physical air speed.
- **The mark is the hose's colour**, the pressure blend the line already has, never the
  accent: on this sheet the accent means selection, and a selected hose must stay
  distinguishable from a hose with air in it.
- **The 16 ms tick.** The transport's existing 16 ms timer advances the phase while the
  model runs and the desktop allows motion. There is no second timer: start clears the
  phase, pause freezes it, stop and restart clear it, and a single step adds the step's own
  seconds, not a frame's.
- **Reduced motion.** `[KDE] AnimationDurationFactor = 0` replaces the stream with one
  static arrowhead in the middle of the route, in the flow direction, with no stroke: the
  direction stays readable and the phase does not advance.
- **The printed page keeps the direction.** The export passes the model's flow with a zero
  phase and `motion = false`, so a page carries one arrowhead per hose that holds air and
  no pulse. A document with no live model has no reading, and the page draws no mark.
- **No allocation per frame.** The route is already computed; the phase and the flow are
  scalars, and each mark is one short polyline and one triangle on the same surface, in a
  fixed buffer.
- **Performance.** The animation must not move the frame budget; the stream is at most eight
  short segments and eight polygons on a route the drawing already builds. A suspected
  frame-time change is measured at 200 components, not guessed.

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
- **No auto-hide by default.** A panel that hides itself during a demonstration is a
  setting the user turns on, never a surprise.
- **No hidden panel without a handle.** If a panel can hide, its rail and its command stay
  visible and reachable.
- **No separate icon set for components.** The symbol is the icon. The ribbon's own line
  family is chrome and never stands for a component.
- **No invented colour for depth.** The planes, the lines and the shadow come from the
  desktop palette; a ribbon with a brand colour of its own is not this project.
- **No colour without a number.** Every colour-coded value also has its value.
- **No hidden mode.** Edit and simulation look different and are named in the status
  bar.

## Still open

- The font family from `[General] font` is read and stored, not loaded; text size is.
- Whether the page setup and the export editor move from the center to the right dock is
  decided after the adaptive modes are measured
  ([interface-plan.md](interface-plan.md) stage 6).
- The **fit and the zoom** are under review: the owner wants the sheet fitted rather than the
  frame, and the zoom and the middle double click checked against the two CAD references.
  Nothing changes until that note arrives ([interface-plan.md](interface-plan.md#open-decisions-owner-2026-11-15)).
- **The block's reserve is decided**: `Sin cajetín` frees the frame, and enabling a block
  over an existing drawing is a fault the check reports
  ([ADR 0030](adr/0030-the-block-protects-its-own-box.md)); the footprint replaces the band
  with its capability.
- UI strings move to the Qt catalogs in M6; the two built-in languages already exist as
  `name` and `name_es` in the catalog ([ADR 0027](adr/0027-translation-catalogs-are-qts.md)).

Breeze symbolic icons are no longer open: `QIcon::fromTheme` finds them in the user's
theme and the theme adapter tints and caches them, and a machine without that theme shows
the label alone, as a native application does. `--check-icons` is the evidence.
