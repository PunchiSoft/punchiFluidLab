# Interface design

Status: **the three regions exist; parts of the spec are still open.** The sidebar tiles,
groups, search and footer, the panels that hide, the ribbon's depth and icon family, the
workspace frame and the adaptive shell are target, not current window; each section says
what is there today. The staged order of that work is kept in the owner's working notes. The
document stays the specification: a change to
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
`18 × grid_unit` wide and resizable, the status bar one row. Minimum window 900 × 600,
unless the room a screen has is smaller: then the minimum is the room (§2.2).
**1366×768 is the reference screen** (the entry and education laptop; the window opens
maximized there, filling its work area); there the ribbon is
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
target (the owner's staged plan, stage 6).

### 1. Ribbon (top)

The ribbon's commands use the **project's own line family**, embedded from `assets/icons`
and tinted by the theme ([ADR 0028](adr/0028-the-ribbon-draws-its-own-icons.md)), so one
command looks the same on every desktop; a command without a drawing falls back to its
label. The rest of the window keeps the desktop's own icons
([ADR 0004](adr/0004-icons-from-the-desktop.md)).

**The table carries a command, not a label.** `ribbon::Command` names each action the window
handles; a control with no handler is painted disabled and cannot dispatch, and the window's
dispatch is exhaustive, so a command without a handler does not compile. Two controls that
read alike — the project's **Nuevo** and the circuit's **Nuevo circuito** — are different
commands.

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
  that act on the circuit in view. Today the strip shows one tab per circuit, the home
  control, the × on every tab but the last, the + after the last tab, and switches with a
  click, `Enter`/`Space` or `Ctrl+Tab`/`Ctrl+Shift+Tab` as a view change; `F2` or a double
  click opens the inline editor in the tab (`Enter` commits as one undo, `Escape` cancels).
  The canvas reserves its band. One button per circuit says the same thing twice, makes
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

The list of commands is one table in `apps/punchi-fluid/src/ribbon/model.rs`; the expanded
groups, the collapsed groups and their flyouts are generated from it, so they cannot drift
apart. The shortcuts and their hints come from one shortcut table
(`apps/punchi-fluid/src/shortcuts.rs`) whose rows name the command they dispatch, so a hint
cannot announce a key no handler answers. The two help links are drawn in the tab row beside the tabs, because they are not
commands of a tab. **Today** the file commands, the sheet commands (**Márgenes**,
**Orientación**, **Tamaño**), the title block commands (**Contenido**, **Posición**,
**Restablecer**), **Circuito ▸ Nuevo circuito**, the edit commands (including **Conducto**'s
class and **Diámetro**, **Repartir**, **Todo**), the view commands, the transport with its
**Restablecer** and speeds, **Inicio**, the live **Recientes** list, **Configurar página** and
the theme dispatch; the rest is painted disabled until its phase of the ribbon plan.

**Configurar página** (the group's launcher) opens a panel with the paper — the named sizes
or custom millimetres — the orientation and the margins, by preset or in millimetres.
**Editar datos…** opens the title block editor: the twelve stored fields, a **Mostrar**
switch per field, the content model that sets its default field set and the position on the
sheet. **Comprobar** runs the design check on the circuit in view and opens the faults
panel: each fault says what is wrong and where, selecting it marks its component or pipe, and
an empty list says the circuit passes. **Circuito ▸ Renombrar** and **Cerrar** open a dialog: the first renames the active
circuit, the second confirms removing it with its components and pipes, both as one undo;
**Cerrar** is disabled with a single circuit and the last one is refused. **Opciones…** sets
the export format the name defaults to, the PNG resolution and whether the drawing carries a
parts list. In every panel **Aplicar** is one undo, **Cerrar** or `Esc` discards the draft, and
an impossible change is refused with a notice instead of applied. The grid the reference had is
not in the Rust window yet; it is its own capability, not a control that pretends.

| Tab | Contents |
|---|---|
| **Archivo** | Nuevo, Abrir, Guardar, Guardar como, Exportar (PDF/PNG/SVG) and **Opciones…** (the export settings); the **Circuito** group with **Nuevo**, **Renombrar** and **Cerrar** for the circuit in view (the strip is the list); one **Configurar página** group in reference order: **Márgenes** (profile gallery), **Orientación** (Vertical/Apaisado dropdown), **Tamaño** (paper list with proportions, canonical and common names, plus **Personalizado…**) and a small page launcher; and **Recientes** as one dropdown (the last ten). Salir is **not in the window yet**; the export command opens a save dialog, and the panel keeps only the raster resolution and the parts list — [capabilities.md](capabilities.md) F5 says what remains |
| **Editar** | Deshacer, Rehacer, Copiar/Pegar, Duplicar, Borrar, Todo (`Ctrl+A`), Girar (`Alt+R`), Reflejar (`Alt+M`), Alinear (izquierda, derecha, arriba, abajo, **puertos**), Repartir (horizontal, vertical), **Comprobar** (el panel de faltas), Conducto (trabajo, pilotaje, drenaje, **Diámetro** `Alt+D`) |
| **Ver** | **Vista** (Aumentar, Disminuir, Tamaño original, Vista anterior, the grid toggle and the view-only title-block toggle), **Paneles** (Catálogo, Propiedades, Ocultar al salir, Restablecer vista), the theme scheme and the contrast summary; `Ctrl+rueda` magnifies at the pointer, the wheel and the middle button pan, the middle double click fits the frame, and the arrows pan from the keyboard |
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
two-column tiles (a click chooses one; its menu or the keyboard adds it at the centre) and the
footer, with a width the user can drag and the session remembers; the drag from the palette is
the next slice.
The panel that lists the reasons for each skipped file is still target.

- Groups are the `category` field (`supply`, `valve`, `actuator`, `throttle`,
  `accessory`, `sensor`), collapsible, in a stable order, state remembered per user; the
  label is translated in the session, so the widget never learns the numeric enum.
- The sidebar is **a palette of tiles**, two per row: the symbol, the principal name on up
  to two lines and the variant on its own line, with the full name and the `id` in the
  tooltip and in the accessible name. A name wraps at a space and the variant elides; no
  text in the palette widens the panel past its column. The header carries the title, the
  number of entries and the search field; the title, the count and a category's name elide in
  place. The footer names the folder in use and the number of skipped files. A search with no
  match says so in the panel instead of leaving it blank.
- The thumbnail of each entry is **the symbol of its own definition**, drawn by the
  same renderer as the canvas. No separate icon set can drift from the diagram.
- **A closure is not a palette entry.** `plug.blind` and `silencer.pneumatic` are closure
  definitions: the menu fits one to a port and the file stores the choice per port, so the
  palette hides them instead of offering a box with its own hose. They stay in the ABI
  catalog, so an old sheet that already places one still opens and simulates; only the
  palette row is gone, and the catalog index the row would have kept is untouched.
  The filter is in the session, not in the widget, so the row index and the search count
  cannot drift ([ADR 0033](adr/0033-a-port-closure-belongs-to-the-port.md)).
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
- **Dragging a tile owns the pointer until the release.** The palette sits in a scroll area, and a scroll area that takes the gesture past its threshold cancels the carry: the preview disappears halfway when the hand drifts diagonally. The tile prevents that steal, so the gesture survives any path over the panel, the docks and the sheet; the palette is scrolled with the wheel or its bar, never by dragging on a symbol. The release places at the pointer snapped to the grid's step; a release outside the sheet and `Escape` cancel. The drop tolerance outside the sheet and the two-step placement remain the next slice. The tile prevents that steal, so the gesture survives any path over the panel, the docks and the sheet; the palette is scrolled with the wheel or its bar, never by dragging on a symbol. Releasing outside the sheet still cancels, and the two-step and tolerance work is recorded as the next slice in [capabilities.md](capabilities.md).
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
dock is a later decision (stage 6 of the owner's staged plan).

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

Both panels implement the fold, the memory and the reset now: the catalog and the right dock
fold to their rail, **Restablecer vista** puts both back along with the zoom and the pan, and
`Ctrl+B` and `Ctrl+J` are their keyboard paths. Width and visibility are view state
remembered across sessions in `XDG_DATA_HOME/punchi-fluid/settings.state` beside the recent
list: a defensive `key=value` file of its own, bounded in bytes and lines, written atomically,
ignoring unknown keys and refusing an unknown version, which never enters the document, the
undo history or the modified mark. A width above the maximum of the layout mode the window
opens in is clamped, and a closed panel starts as its rail. **Ver ▸ Ocultar al salir** is
auto-hide's toggle, and its label does not change: a dwell of about 350 ms on a closed
panel's rail opens it and about 800 ms without the pointer closes it, a return cancels the
pending close, and the timers never fire while a pointer drag is in progress, while the
keyboard focus is inside the panel or while a dialog is open. The flag is remembered too, and
**Restablecer vista** returns both panels to their defaults and persists them. What is still
missing is the persisted ribbon-minimized state: the minimized ribbon is not built in this
window yet (§2.2), so there is no state to remember, and the title block still opens as an
invoked editor instead of living in the right dock. The `cargo test -p punchi-fluid-window`
suite proves the file's round trip and its truncated, out-of-range and unknown-key cases, the
clamp to the opening mode, the restore, the settle write, the reset and both timers,
headless.

### 2.2 The window adapts

The shell chooses one of three modes from the room it has, and the sheet keeps a minimum
before a panel is allowed to cover it:

| Mode | When | Ribbon | Catalog | Right dock |
|---|---|---|---|---|
| **Ancha** | the sheet, the catalog and the dock fit at their preferred widths | full | docked | docked |
| **Media** | the preferred widths no longer fit but the minimums do | full | docked, narrower | docked, narrower |
| **Compacta** | the minimums do not fit (below 1000 px at the default font in the Rust shell) | the groups that fit, collapsing as they do today | rail, opened as a drawer | rail, opened as a drawer |

- The minimums are content measurements, not magic numbers: the sheet keeps 480 px, the
  catalog 220 px and the right dock 300 px. The Rust shell derives the modes from those
  measurements and the panels' preferred widths (240 and 360): wide from **1080 px**,
  medium from **1000 px**, compact below that at the default font. The retired Qt shell added
  its own chrome and so had larger figures; a larger font moves both, because the panel
  widths follow the spacing unit.
- **The ribbon can be minimized** by the user, with the control at the right end of the tab
  row: the tab row stays and the open tab's groups appear as a panel over the drawing area
  (never over the panels' headers), closing after a command or `Esc`. The state is
  remembered with the panels. Below the collapse threshold the ribbon already folds to
  **Menú** instead.
- **The first window is screen-aware** (P-028, revised 2026-11-23): with no remembered size
  it opens at **1440×900** on a 1080p screen. The toolkit reports the monitor, not the work
  area, so the shell subtracts a **two-unit margin** and an **80-point chrome reserve** (the
  title bar plus the taskbar or panel) and sizes the window inside what is left; the window
  is centred in that room, not in the monitor. When the room is shorter than 900 points the
  window opens **maximized**, so the compositor fits the real work area instead of the shell
  guessing it: 1366×768, 1536×864 (1080p at 125 %), 1280×720 (1080p at 150 %), 1600×900 and
  1280×800 open maximized with a normal size that fits (1366×768 keeps **1334×656** for the
  day it is unmaximized); 1920×1080 and 1920×1200 open at 1440×900 normal. If the room is
  below 900×600 the minimum follows the room (1366×768 at 125 % has 502 points of height), so
  no window state hides the status bar under the taskbar. The 1366-wide panel keeps the
  unfolded ribbon; at 125 % its ~1093 logical points fold it, as before.
- **The window remembers its size and place**, and only when the place still lands on the
  room: a laptop undocked from a second monitor must not reopen off-screen. A remembered
  size is **clamped to the current room** with the same margin, reserve and minimum, not only
  to the minimum, so a window saved on a 1920 monitor reopens inside a 1366 laptop. A
  maximized window keeps the size it had before. The design minimum is 900 × 600.
- **A returning window is created with its remembered size**, so it does not resize in front
  of the reader; the place is applied on the first frame, which keeps the monitor the window
  opened on as the anchor of the fallback. When that monitor reports again and the window no
  longer lands on it (an extension monitor was unplugged), the shell centres and clamps it
  back into the room; a window moved to a monitor of another size is left alone, and a
  secondary of the same size cannot be told apart without the platform's topology.
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
- **Today** the Rust shell builds the modes and the geometry. `LayoutMode` derives **Ancha**,
  **Media** and **Compacta** from the room the window has and the declared measurements, in
  device-independent points: the sheet keeps 480 px, the catalog 220 px and the right dock
  300 px, and the preferred widths are 240 px and 360 px, so **Ancha** begins at 1080 points
  and **Media** at 1000 (the widths are still the interface constants, not spacing-unit
  multiples; the ribbon's text and heights are literals too). **Ancha** docks both panels at
  the width the reader gave them, **Media** at their minimums, and **Compacta** folds both to
  their rails and keeps the sheet its floor. In **Compacta** `Ctrl+B` and `Ctrl+J` open the
  panel as a drawer over the drawing area, never pushing it: a modal overlay that closes on
  `Esc`, on a click outside and on its hide control, one panel at a time. **Restablecer
  vista** also dismisses an open drawer. The ribbon keeps the collapsing it had: each group
  folds to its own title button by available width; the **Menú** compact ribbon and the
  minimized ribbon are not built in this slice. The window opens at 1440×900 when the room
  has it, otherwise maximized at the largest normal size that fits the room; it remembers
  its size, its place and whether it was maximized in
  `XDG_DATA_HOME/punchi-fluid/window.state` beside the recent list, and restores them only
  when the place still lands on the room, clamped to the current room with the same margin,
  reserve and minimum; a maximized window keeps the size it had before. A returning window
  is created with the remembered size and the place lands on the first frame; a window left
  on an unplugged extension monitor comes back to the room it opened on. The
  `cargo test -p punchi-fluid-window` suite proves the three derivations from widths, the
  first size against the LATAM screen matrix (1024×768 to 4K, including 1080p at 125 and
  150 %), the clamp and the off-screen rejection, the returning size, the reanchor, the state
  file's round trip and its defensive reads, and the compact rails, the drawer and its
  dismissal, headless;
  `--check-layout` and `--selftest` were the Qt window's checks and are not ported to this
  one, so no automated check opens the real desktop window. What is left:
  the **Menú** compact ribbon and the minimized ribbon with its groups as a panel over the
  drawing area (and with it, a ribbon state to remember), the ribbon's text and heights from
  the theme tokens (stage 7).

### 2.3 The start view: Inicio

**With no project open, the drawing region holds the start view**, not an empty sheet: it is
not a fourth region and not a ribbon tab, because the ribbon holds commands and the start view
holds the decision of what to work on. **Inicio is a page, not a document**: while it is open
the ribbon and the status bar are hidden instead of greyed, and the page fills the window from
the top edge to the bottom, including the 1366×768 support floor. The view closes when a
project opens ([ADR 0031](adr/0031-the-program-opens-on-a-start-view.md)).

**The left region carries the start navigation** while the start view is shown: the catalog's
column holds a rail with **Inicio** chosen, **Nuevo proyecto** and **Abrir…**, a separator,
**Configuración** with its **gear**, which opens the window of §2.4, and at the foot
**Acerca de** and **Licencia**, instead of a second sidebar. The rail carries no brand and no
group headings; the shortcuts live in the tooltips, not in the row, and the chosen row is a
light pill named *selected* to the accessibility layer, never colour alone. With a document
open, **Volver a la hoja** sits above **Inicio**; at launch it is absent. **Atajos** and
**ISO 1219** need a panel and content of their own and stay out until a node owns them. The right dock is hidden, since with no project it has no inspector
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
gives each row its symbol, marks the current view with a light pill and exposes it as
selected.

**Today** the program opens on Inicio in a normal run (the automated window checks keep the sheet), and the page closes with **Volver a la hoja**, with **Esc** or with **Nuevo**. Inicio takes the whole window: the ribbon and the status bar are not shown, the catalog and the right dock are hidden, and the rail takes their place with **Inicio**, **Nuevo proyecto**, **Abrir…**, **Configuración** and, at the foot, **Acerca de** and **Licencia**; the window title names the program, never an untitled document. It offers, in reading order:

- **Nuevo proyecto** and **Abrir…**, followed by **Plantillas** and **Recientes**. The page
  lists at most `PF_RECENT_PAGE_LIMIT` (five) of the ten remembered, newest first, with the
  name, the folder and the **size** read from the file's metadata — the file is never opened
  or parsed; the modified relative time waits for P-034b. When more
  than five are remembered, a line says the rest are in **Archivo ▸ Recientes**, whose group
  already lists all ten. A document whose file has gone stays in the list in muted ink, marked
  **no encontrado**, cannot be opened, and leaves it only through its **quitar** control, which
  is a button with an accessible name and does not touch the document, the history or the
  modified mark.
- **An empty Recientes** says the next step instead of showing a blank table: *Aún no hay
  proyectos recientes. Crea un proyecto nuevo o abre un archivo; aparecerá aquí.*
- **Plantillas**, **four** circuits offered and authored for this project: a 3/2 valve and
  single-acting cylinder (the worked example), a 5/2 valve and double-acting cylinder, air
  preparation ahead of a valve, and an electrical pushbutton driving a solenoid valve. The
  fourth is offered with the rest; its electrical connection remains **P-026**. Each card is a
  keyboard-focusable button; choosing one copies its circuit into a clean project, while
  **Nuevo proyecto** remains empty.
- **Aprender**: **Acerca de** and **Licencia**. The shortcuts and what “conforme a ISO 1219”
  means need their own content and are not invented in this pass.

The example cards **divide the page's width between them**, with one unit between cards and
a minimum of 10 × `unit` before the grid starts a second row. Each card's height follows its
width: the plate is the content width over the 16:9 aspect, and the card is the plate over
0.65, so the plate is 65 % of it and the proportion holds at every breakpoint. At the default
18 px unit and a 1280 px window a card is about 342 × 278 px with a plate of about
321 × 181 px. Each card is a picture and its label: the picture fills the plate width and
keeps its aspect; the plate is `panel` on a `surfaceRaised` card
(`surfaceSunken` while pressed, `accent` border on hover and focus), and the name is
`textLabel` with the detail on one elided line below it, both with the picture's left margin,
plus a tooltip that names the circuit. The default way in (**Nuevo proyecto**) carries the
accent tint and border, and its text is ink, never a light word over a fill that hover
repaints.
Each thumbnail is the template's own document drawn by the same preview path as the sheet,
fitted into the card's plate with the theme ink; no 512 × 288 raster and no provider cache is
kept, so a theme change cannot leave a stale tint on screen. The rail and the cards take
their sizes and tones from the theme tokens. Template cards and recent rows expose their
title/name and description to assistive technology. Inicio is reachable from **Archivo ▸
Inicio**; the domain chip, the recent thumbnail and the opened time are deferred to P-034b,
with a preview-by-path bound and a `recent.json` version change.

### 2.4 Configuración

**The window is one surface, not a fourth region.** It opens from **Ver ▸ Tema**, `Ctrl+,`
and the Inicio rail, over the sheet or the start page, and closes with `Esc` or **Cerrar**;
the focus returns to the command that opened it, and nothing it does enters the document, the
history or the modified mark. It is a dialog the user opens on purpose, the exception to "no
modal on the normal path".

**Its shape is a nav, a page and a footer.** A 200 px rail on `panel` lists the pages it can
show; the page scrolls; the footer carries **Restaurar página** and **Cerrar**. Preferred
size 880 × 600, minimum 760 × 520, so it fits the 1366 × 768 reference and the 900 × 600
window floor. The nav is keyboard reachable, and the page has a heading and an accessible
name.

**The pages exist only when their settings do.** **Apariencia** and **Paneles** are the
first two; **Catálogo** (the folder in use and the skipped files with their reason) and
**Atajos** (the command table, read-only) wait for their content. A category with nothing
behind it is not shown.

**Apariencia** has three modes: **Escritorio** (default; the desktop palette, which already
switches with light and dark), **Sistema** (the theme assigned to the light or dark scheme)
and **Un tema** (one fixed theme). Under **Sistema** the page shows two slots, **Día** and
**Noche**, and choosing a card fills the aimed slot; under **Un tema** a card applies
directly; under **Escritorio** a card offers to switch to **Un tema**. The gallery is a grid
of 160 × 98 cards, each a mini window drawn with that theme's tokens (ribbon, panel, canvas,
paper, accent dot), its name, **Sistema** or **Integrado**, and **En uso** marked with a
check, never by colour alone. Opening the page reveals the theme in use; arrows move, `Enter`
applies; **Restaurar apariencia** returns to `Escritorio`. The theme applies live; there is no
Apply/Cancel. The three modes are segments of the page; under **Sistema** the two slots show
the theme each scheme uses, and a card fills the slot being edited.

**Paneles** holds **Ocultar al salir** and **Restablecer vista**. The **Ver** menu keeps the
quick toggles as shortcuts; Configuración is the same setting with its context, not a second
owner.

**What it refuses**: a search field with two pages, an account or profile page, importing,
exporting or following a theme file, and an empty category.

**A theme is a `Palette` seed.** The page never writes a colour: `Theme::for_appearance`
builds the palette and `tokens_from` derives the tokens, so `paper`, `line`, `ink_dim` and
the contrast rules are the same as the visual language below. A theme that misses a criterion
is shifted until it reaches it; the card shows the measured result.

### 3. The sheet

**The sheet is a raised card on a sunken workspace.** The canvas fills the drawing area
inside **half a `unit`** of margin over `surface_sunken`; the paper is filled with `paper`,
edged with `line_strong` and carries the `shadow` token offset once behind it. The drawing's
own background follows the paper, so a symbol that erases against the page matches. The
drawing is written for the page, black ink on white; the window's `Surface` maps that palette
to the theme's `ink`, `paper`, `ink_dim` and `ink_muted`, so the same recording is legible on
both devices. Every floating editor is one `Card` with the same radius, edge and shadow; the
shadow is drawn, never blurred per frame, and none of it changes the zoom, the pan, the check
or the print.

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
| **Editing (stopped or paused)** | selects and edits the component; a valve never actuates | frozen; a valve held when Play pauses is released |
| **Play running** | a primary-button hold actuates the valve; release returns a spring valve | advances |

- A valve accepts actuation input only while Play is running. A spring-return valve is
  **held**: press actuates it, release or pause lets the spring return the spool. A valve
  without a spring **latches** on press. The front end asks the model which it is instead
  of guessing, because guessing makes a lever valve stay actuated after a click.
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

**A placed symbol carries its own commands.** A right click on the sheet selects the symbol
under the pointer and opens the reference's menu over it: **Mover**, **Girar**, **Espejo
horizontal**, **Espejo vertical**, **Configurar**, **Propiedades**, **Descripción del
componente** and **Eliminar**. The empty sheet and a hose open nothing and keep what was
selected. `Shift+F10` opens the same menu for the selection from the keyboard, and `Esc` or a
click outside closes it. While it is open the view stays fixed, so the arrow keys belong to the menu.
**Mover** arms the gesture instead of requiring a drag: the selection follows the pointer,
the arrows nudge it 1 mm (`Shift` 10 mm), a click or `Enter` drops it as one undo and `Esc`
cancels. Opening the menu is view state, never a document edit; the right button
never presses a valve. In editing mode, a double click on a two-position 3/2 or 5/2
directional valve opens its editor. **Configurar** in the context menu, reachable with
`Shift+F10`, opens the same editor. It has a designation field, left and right catalog
actuation panels, a graphical choice of initial position and a preview from the shared
renderer. The preview shows the selected paths and follows horizontal and vertical
reflection. **Apply** commits the draft as one undo. **Cancel** or Escape discards it.
Outside these valve families, **Configurar** still opens the placeholder.
**Descripción del componente** opens a
read-only card for a catalog id with a picture: the normalized 800 × 450 image and the
definition's English name, summary and id, with **Cerrar**; a component without a card keeps
the mockup note. The card is catalog metadata, never an editor, and it does not touch the
document or the format. **Espejo vertical** flips the selection top to bottom as one undo;
the ribbon button and a direct shortcut are not there yet, so the menu is its keyboard path.

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
  - **Edición** (default, including a stopped or paused model): select, move, connect,
    delete. Grid snapping on. A valve click only selects it, and a drag moves it.
  - **Simulación**: the model runs; pressing a valve actuates it while held. Pausing
    releases a held spring valve. Components cannot be moved or deleted during a run.
    `Esc` stops the simulation and returns to editing.
- **Selection is the symbol's own lines in the accent colour, not a box around it**: the
  reference marks the component, and an extra rectangle is a second drawing that says
  nothing the symbol does not. The inspector names what is selected, so the state is never
  colour alone. Hover only changes the cursor and shows a tooltip (never moves anything: a
  hover that changes the document is a known UX trap).
- Pipes are orthogonal; the route is stored as a list of waypoints so it survives a
  move. Ports snap with a magnet radius derived from the grid step. In Edición, a primary
  drag from one port to another shows the route in the accent and creates the connection on
  release; `Esc` or a release away from a port cancels it. A new pneumatic hose starts at
  10 mm, and an electrical wire has no tube size.
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
not fight the right dock, and it leaves with the room it needs. The page is fitted and
centred **left of the strip's channel**: the drawing area keeps a gutter on the right where
the strip floats, so the strip never covers the paper, and the drawing clips against the
canvas, never the window, so a zoom above the fit cannot paint over the ribbon or the docks.

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
- **Today** the strip exists: it floats over the right edge of the drawing, inside the channel
  the drawing area reserves for it, so the page is fitted left of the strip and the strip takes
  no width from the paper. It carries **Mover**, **Ajustar al marco**, **Zoom** with its panel
  (Disminuir, Aumentar, Zoom a una zona, Vista anterior), the view-only **Cajetín**, the single
  **Iniciar/Pausa** and the **fold**, sized from `unit` and named for the accessibility layer;
  it is remembered as view state and the compact mode starts folded. The window also fits the
  frame on `Ctrl+0` and on the middle button's double click, keeps one step of view history,
  has the zoom window and the hand mode in the ribbon, and bounds the pan to the paper (the
  margins may lie beyond the canvas and are clipped there). The view grid shows under the
  drawing with its step in 5, 10, 20 and 25 mm, toggled from Ver and set in Configurar página;
  it is a view guide and never printed.

### 4. Status bar

Time, pressure, position, velocity, air consumed, definition count, mode, and the
last message. It is the place for "saved", "3 files skipped", "export written to
…". No message that requires a click to disappear. It ends on the right with the sound
toggle, which is a command like the others: a label, the desktop's own icon and `Ctrl+M`.

**Today** the bar shows pressure, stroke, velocity, simulated time, the magnification, the
paper's name and size, one hint or error line, and the sound toggle. The air consumed, the
definition count and the mode text are the target, not in the bar yet.

## Visual language

The unsaved changes confirmation, the valve editor, the rename and close circuit dialogs,
the page setup and export options forms, the settings window, and the file dialog share a
dialog frame. The heading and visible close button sit on a raised band, and a matching
footer aligns the actions to the right. The body scrolls when it exceeds the available
height. Buttons have a minimum height of 32 logical points. The theme owns a fixed light and
dark dialog palette, so the approved tones do not follow the desktop scheme. The rename
dialog starts with the name field focused and keeps itself open on an empty name; the close
circuit confirmation names the circuit and focuses Cancel, never the action that removes it.
The valve editor offers the actuation and return variants the catalog holds; the variant is one undo, the preview and the model follow it, and it survives saving. The configuration forms group related fields in cards and keep their Apply/Cancel moment:
page setup applies the sheet as one undo and discards its draft on Cancel, while its grid
stays view state; export options commit on Apply and discard the draft on Cancel. The
settings window applies its appearance and language live, groups its pages in cards, acts on
the visible page with Reset page and closes with Done; below 560 useful points its nav
stacks over the page. The file dialog keeps the folder, its rows and the destination in the
body and the main action and Cancel in the footer; the overwrite question is a bordered card
that names the file, and naming another destination asks again. The close button cancels the
pending action or closes a live settings window, including when the unsaved changes question
is shown over the valve editor.

| Token | Source |
|---|---|
| `surface`, `surface_alt`, `surface_raised` | `[Colors:Window]` / `[Colors:View]` / `[Colors:Button]` |
| `paper` | the sheet: `[Colors:View]`, lifted towards the ink when the desktop makes it equal to the window (Breeze dark: `Window` and `View` are both 36,36,36), so the card reads over `surface_sunken` |
| `line`, `line_strong` | `text_muted` blended towards the surface and measured; a line that carries meaning reaches 3:1 |
| `shadow` | the ink at low alpha, drawn once behind the paper, never a per-frame blur |
| dialog palette | the approved light and dark dialog tones, a fixed pair in `theme.rs` rather than a derivation from the desktop |
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

**Text roles and control measures.** The adapter derives one scale from the desktop font, so
a larger font moves every step: `text_page` is the title of a view that is not a document,
`text_section` a group label, `text_name` a component or row, `text_action` a command,
`text_value` a reading and `text_secondary` a path or an explanation. The same adapter owns
`row_height`, `tile_band`, `target_min` — never below 24 points — and `focus_width`; a
widget asks for the role or the measure and never sets a pixel of its own.

A single semantic layer, `apps/punchi-fluid/src/theme.rs`, reads the desktop's
`kdeglobals` once at start, derives the tokens and applies them to egui. Widgets ask the
tokens for `accent` or `muted`, never for a literal. The rule is in `AGENTS.md`: a
hardcoded colour or pixel size in a widget is a bug, and the only fixed palettes in the
application are the fallback, the built-in appearances, the approved dialog pair and the
forced palettes used to test light and dark.

A colour that does not reach its criterion against the surface is shifted just enough to
reach it (`ensureContrast`), because a desktop may hand the application an accent that
disappears on its own window colour.

**An appearance is a palette, not a second language.** `Escritorio` builds the `Palette` from
`kdeglobals`; a built-in theme supplies the same seven seed colours, and the table above still
derives every token from them. No widget learns a theme id; it asks the tokens
([ADR 0042](adr/0042-the-appearance-is-a-setting.md)).

### Following the desktop

1. Colour scheme and palette: the window reads `$XDG_CONFIG_HOME/kdeglobals` once at
   start and derives the tokens from it. A machine without that file gets the light
   fallback, and the window never reads the file per frame.
2. Accent: the desktop's highlight colour, shifted by `ensure_contrast` when it cannot be
   seen on its own surface.
3. Secondary greys: the text colour blended towards the window colour by 30 % and 55 %,
   then measured; the palette names neither.
4. Font: the family and point size come from `[General] font`; `fontdb` loads the family
   and the text styles are derived from the size.
5. Icons: the window's own SVG family from `assets/icons/actions`, compiled in; components
   are shown by their own symbol, drawn by `punchi-fluid-draw`
   ([ADR 0028](adr/0028-the-ribbon-draws-its-own-icons.md)).
6. HiDPI: egui's points scale with the viewport; the drawing is in millimetres and one
   scale is applied, so a scale change needs no layout change.
7. Non-KDE sessions (GNOME, tiling WMs, no `kdeglobals`): the light fallback is the same
   code path. The application must look deliberate, not broken.

### Apariencia

**The desktop is the default, not the only source.** `Escritorio` is what a new install shows
and what a theme returns to; it reads `kdeglobals` and the portal. The other two modes are
opt-in and live in `settings.state` (`appearance=desktop|system|<theme>`; with `system`,
`theme_day` and `theme_night`), never in the document. An unknown value reads as `desktop`,
and the settings file keeps its version because the keys are optional
([ADR 0042](adr/0042-the-appearance-is-a-setting.md)).

**A built-in theme is a `Palette`, not a new token set.** Its seven seed colours are compiled
in Rust; `tokens_from` derives `surface_sunken`, `paper`, `line*`, `ink_dim`, `shadow` and
the rest with the same rules and the same tests. A seed that does not reach a criterion is
shifted by `ensure_contrast`, so the shipped look is the measured one. The themes arrive one
per change, each with its contrast row and its screenshots; **Sistema** is offered once a
light and a dark theme both exist. **Today** the window offers **Escritorio**, **Sistema** (CAD claro by day, Pizarra by night)
and **Un tema** with CAD claro, Pizarra, Nórdico, Azul profundo or Grafito.

### Accessibility

- `eframe` exposes the widget tree through accesskit; the frame node's accessible name is
  a debt recorded in ADR 0036, and every custom control must set its label explicitly.
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
keeps its model state after any held spring valve is released. The reference's signature is the circuit in motion, and the sheet
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

**The language is a setting, and its first slice exists.** [ADR 0045](adr/0045-the-interface-speaks-english-through-embedded-toml-catalogs.md)
supersedes the retired Qt plan of [ADR 0027](adr/0027-translation-catalogs-are-qts.md):
`punchi-fluid-i18n` reads the embedded `i18n/en.toml` and `i18n/es.toml`, English is the
source and the visible default, and **Configuración ▸ Apariencia** offers the language without
a restart. The whole Rust window is on the catalog: the ribbon, the catalog, the inspector, Configuración,
the check panel and its finding sentences, the start page, the status bar and its notices, the
lifecycle panels, the file, circuit, page-setup, title-block and export-options dialogs, the
sheet menu, the description card and the language selector. The technical error causes from
the crates stay in English inside the translated frame (a diagnostic, not UI copy); the
exporter labels are the remaining debt, and the CLI is a fixed English contract. The
description card reads `name_es`/`summary_es` when the definition has them. Keys live in the catalogs, the exporter receives its labels
from the caller, the CLI keeps a fixed English contract, and no sentence is concatenated. Until
a surface is migrated its Spanish literal is debt ([i18n/README.md](../i18n/README.md)).

## What this design refuses

- **No modal for the normal path.** Export and save are panels; **Configuración** is a window the user opens, and closing it returns to the sheet.
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
- **No theme file.** A theme is a compiled `Palette`; a file would be an untrusted input
  with its own parser and bounds ([ADR 0042](adr/0042-the-appearance-is-a-setting.md)).
- **No colour without a number.** Every colour-coded value also has its value.
- **No hidden mode.** Edit and simulation look different and are named in the status
  bar.

## Still open

- The font family from `[General] font` is loaded by the window (`fontdb`); the theme's
  size and contrast rules are tested.
- Whether the page setup and the export editor move from the center to the right dock is
  decided after the adaptive modes are measured
  (the owner's staged plan, stage 6).
- The **fit and the zoom** are under review: the owner wants the sheet fitted rather than the
  frame, and the zoom and the middle double click checked against the two CAD references.
  Nothing changes until that note arrives.
- **The block's reserve is decided**: `Sin cajetín` frees the frame, and enabling a block
  over an existing drawing is a fault the check reports
  ([ADR 0030](adr/0030-the-block-protects-its-own-box.md)); the footprint replaces the band
  with its capability.
- UI strings are still Spanish literals; the English-first catalog mechanism has no ADR
  yet. The two languages of the catalog data already exist as `name` and `name_es`.

Breeze symbolic icons are no longer open: the window draws its own SVG action family,
compiled in from `assets/icons/actions`, and components are shown by their own symbol. The
coverage guard is a debt (see [reference/icons.md](reference/icons.md)).
