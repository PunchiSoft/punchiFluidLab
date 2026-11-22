# Action icons

Status: current. The inventory of the window's own action-icon family, the drawings in
`apps/punchi-fluid-qt/src/actionicons.cpp`, and the coverage the `--check-icons` run
asserts. The authority is the code; this page names what the code draws and why.

Since [ADR 0035](../adr/0035-the-window-is-rust.md) the same drawings are also files under
[`assets/icons/actions`](../../assets/icons/actions): one SVG per id in the table below, in
the box, stroke and ink that record fixes. `scripts-dev/check-icons.sh` validates every
file and asserts that the table and the files name the same ids; the 16, 24 and 32 px
renders in `assets/icons/review/` are the review evidence. The Qt window still draws the
C++ family until the Rust window loads the files, so the two must stay in step.

## The count

The family has **55 drawings**. The number counts **drawings**, the ids returned by
`actionicons::names()`, not commands: one drawing can serve several commands, and some
commands are generated from the same table. The table below has one row per drawing, and
`--icon-sheet` prints the same number; if the two disagree, the code is right and this page
is stale.

The ribbon table, the view strip, the start view and the collapsed menu are the callers.
The ribbon's command icons are checked in both directions by `--check-icons`: every icon
name a command asks for resolves to a project drawing, and the table entries that carry no
icon are exactly the documented allowlist below.

## Family rules

The box, stroke, caps and joins, ink, states, resolution order and coverage rule are
[ADR 0028](../adr/0028-the-ribbon-draws-its-own-icons.md)'s. This page does not restate
those values; it records the drawings that follow them.

- A drawing is named for the action, in the existing freedesktop-style kebab-case where
  that name already existed.
- The shape is the same in every state; the state changes the tone, never the silhouette.
  A current choice can add the accent tone, not a second drawing.
- A filled dot or triangle is allowed only when it is the metaphor (a junction, play,
  stop); it is the same ink, never a second colour.
- A component is never an action icon: the palette and the sheet keep the catalog's ISO
  1219 symbol.
- The reference sets are inspiration only. Tabler is a reference for the breadth of tool
  and option glyphs, Lucide for legibility at the small render, and Fluent for the toolbar
  family and its states. No path, asset, palette or name is taken from them; the geometry
  is written here from the command's meaning.

## Inventory

One row per drawing, alphabetical by id. **Size** is the logical render size in the
window, the theme unit's large and small button values (16 and 24 at the default unit);
`--icon-sheet` also renders 32 for review, and the small row is the acceptance size.
**State** is `static` (one shape, the tone carries normal, muted, accent or disabled) or
`current` (a choice that adds the accent tone while it is the current value).

| id | command | metaphor | group | size | state |
|---|---|---|---|---|---|
| `align-horizontal-left` | Izquierda | two boxes against a left edge line | Editar ▸ Alinear | 16, 24 | static |
| `align-horizontal-right` | Derecha | two boxes against a right edge line | Editar ▸ Alinear | 16, 24 | static |
| `align-ports` | Puertos | two port stubs and two dots on one axis | Editar ▸ Alinear | 16, 24 | static |
| `align-vertical-bottom` | Abajo | two boxes against a bottom edge line | Editar ▸ Alinear | 16, 24 | static |
| `align-vertical-top` | Arriba | two boxes against a top edge line | Editar ▸ Alinear | 16, 24 | static |
| `check` | Comprobar | circuit outline with a check mark | Editar ▸ Circuito | 16, 24 | static |
| `distribute-horizontal-equal` | Horizontal | three boxes with equal gaps across | Editar ▸ Repartir | 16, 24 | static |
| `distribute-vertical-equal` | Vertical | three boxes with equal gaps down | Editar ▸ Repartir | 16, 24 | static |
| `document-export` | Exportar | page with an out arrow | Archivo ▸ Proyecto | 16, 24 | static |
| `document-new` | Nuevo | page with a plus | Archivo ▸ Proyecto; Inicio | 16, 24 | static |
| `document-open` | Abrir | open folder | Archivo ▸ Proyecto; Inicio | 16, 24 | static |
| `document-open-recent` | Recientes | clock | Archivo ▸ Recientes | 16, 24 | static |
| `document-properties` | Opciones…; Propiedades; Personalizado…; Márgenes personalizados…; Configurar página… | page with two slider rows | Archivo ▸ Proyecto, Configurar página; Ver ▸ Paneles; tira de vista | 16, 24 | current |
| `document-save` | Guardar | page with a down arrow | Archivo ▸ Proyecto | 16, 24 | static |
| `document-save-as` | Guardar como | page with a down arrow and a plus | Archivo ▸ Proyecto | 16, 24 | static |
| `edit-copy` | Copiar | two overlapping pages | Editar ▸ Componente | 16, 24 | static |
| `edit-delete` | Borrar | trash can | Editar ▸ Componente | 16, 24 | static |
| `edit-duplicate` | Duplicar | two pages with a plus | Editar ▸ Componente | 16, 24 | static |
| `edit-paste` | Pegar | clipboard | Editar ▸ Componente | 16, 24 | static |
| `edit-redo` | Rehacer | curved arrow to the right | Editar ▸ Historial | 16, 24 | static |
| `edit-rename` | Renombrar | page with a pencil | Archivo ▸ Circuito | 16, 24 | static |
| `edit-reset` | Restablecer vista | house outline | Ver ▸ Paneles | 16, 24 | static |
| `edit-select-all` | Todo | dashed selection box | Editar ▸ Componente | 16, 24 | static |
| `edit-undo` | Deshacer; Restablecer; Vista anterior | curved arrow to the left | Editar ▸ Historial; Cajetín ▸ Datos; tira de vista | 16, 24 | static |
| `go-bottom` | Inferior derecha; Abajo centrado; Ocultar la tira de vista | down arrow to a line | Cajetín ▸ Posición; tira de vista | 16, 24 | current |
| `go-home` | Inicio | house | Archivo ▸ Inicio | 16, 24 | static |
| `go-top` | Superior derecha; Mostrar la tira de vista | up arrow from a line | Cajetín ▸ Posición; tira de vista | 16, 24 | current |
| `hand` | Mover | palm outline | tira de vista | 16, 24 | current |
| `list-add` | Nuevo | text lines with a plus | Archivo ▸ Circuito | 16, 24 | static |
| `media-playback-pause` | Pausa | two filled bars | Simular ▸ Transporte; tira de vista | 16, 24 | static |
| `media-playback-start` | Iniciar | filled triangle | Simular ▸ Transporte; tira de vista | 16, 24 | static |
| `media-playback-stop` | Detener | filled rounded square | Simular ▸ Transporte | 16, 24 | static |
| `media-seek-forward` | Paso | triangle against a bar | Simular ▸ Transporte | 16, 24 | static |
| `media-skip-forward` | Hasta cambio | two triangles against a bar | Simular ▸ Transporte | 16, 24 | static |
| `object-flip-horizontal` | Reflejar | two triangles across a vertical axis | Editar ▸ Componente | 16, 24 | static |
| `object-rotate-right` | Girar; Apaisado | circular arrow | Editar ▸ Componente; Archivo ▸ Configurar página | 16, 24 | current |
| `open-menu-symbolic` | menú de la cinta | three lines | cinta | 16, 24 | static |
| `pipe-drain` | Drenaje | pipe with the long-dash class pattern and a flow chevron | Editar ▸ Conducto | 16, 24 | static |
| `pipe-pilot` | Pilotaje | pipe with the short-dash class pattern and a flow chevron | Editar ▸ Conducto | 16, 24 | static |
| `pipe-working` | Trabajo | solid pipe with a flow chevron | Editar ▸ Conducto | 16, 24 | static |
| `preferences-desktop-theme` | Esquema | half-filled circle | Ver ▸ Tema | 16, 24 | static |
| `speed-0.25` | 0,25× | gauge arc with one tick | Simular ▸ Velocidad | 16, 24 | current |
| `speed-1` | 1× | gauge arc with two ticks | Simular ▸ Velocidad | 16, 24 | current |
| `speed-2` | 2× | gauge arc with four ticks | Simular ▸ Velocidad | 16, 24 | current |
| `tube-diameter` | Diámetro | tube section with a diameter dimension | Editar ▸ Conducto | 16, 24 | static |
| `view-fit` | Ajustar al marco; Zoom a una zona | four corners around a space | tira de vista | 16, 24 | static |
| `view-list-details` | Catálogo | rows with bullets | Ver ▸ Paneles | 16, 24 | current |
| `view-refresh` | Restablecer | circular arrow | Simular ▸ Transporte | 16, 24 | static |
| `view-restore` | Ocultar al salir | panel with a back arrow | Ver ▸ Paneles | 16, 24 | current |
| `window-close` | Cerrar | x | Archivo ▸ Circuito | 16, 24 | static |
| `x-office-address-book` | Editar datos…; Acerca de | card with a person | Cajetín ▸ Datos; Inicio | 16, 24 | static |
| `x-office-document` | Contenido; Vertical; page and margin choices; Licencia | page with text lines | Cajetín ▸ Contenido; Archivo ▸ Configurar página; Inicio | 16, 24 | current |
| `zoom-in` | Aumentar; Zoom | magnifier with a plus | Ver ▸ Vista; tira de vista | 16, 24 | static |
| `zoom-original` | Tamaño original | magnifier with a centre dot | Ver ▸ Vista | 16, 24 | static |
| `zoom-out` | Disminuir | magnifier with a minus | Ver ▸ Vista; tira de vista | 16, 24 | static |

## Entries without an icon

`--check-icons` walks the ribbon table and prints every entry that carries no icon. The
list is a contract, documented here and asserted in `apps/punchi-fluid-qt/src/main.cpp`;
a new entry fails the check until it is drawn or added to this list with a reason.

- **Custom groups**: `page-setup` (Configurar página) and `recent` (Recientes). The group
  header is a control, not a button; the commands inside it carry icons.
- **Note-only groups**: Paleta, Contraste and Estado. They carry text, not a command.
- **Help links**: `about` (Acerca de) and `license` (Licencia). They are window chrome at
  the right of the tab row, not commands of the circuit.

## Duplicates deferred

These commands still share a drawing. Each is recorded here instead of being left
undocumented; splitting one needs its own metaphor and its own 16 px review.

- **`document-properties`, three meanings.** Export options, the right-dock toggle and
  the page-setup launcher (custom paper, custom margins, the page panel) still share the
  same page-with-sliders drawing. They are three different things; giving each its own
  drawing is the first follow-up, after the P0 set.
- **`edit-undo` as Restablecer.** The title block's Restablecer is a reset, not an undo,
  but it still borrows the undo arrow. Its own reset metaphor is deferred; the label says
  what it does until then.
- **Two houses and two resets.** `go-home` and `edit-reset` both draw a house, because
  Inicio and Restablecer vista both return to a known place. The simulation Restablecer
  uses `view-refresh`, so a house and a refresh arrow can both read as "reset". A distinct
  home and a distinct reset are deferred; the group and the label disambiguate today.
- **`go-bottom` for two positions.** Inferior derecha and Abajo centrado both point down
  at a line. The centred placement needs a full-width bar or a centre mark; deferred.
- **Portrait `x-office-document`.** Vertical orientation, the paper choices and the margin
  presets share the page with text lines. The orientation pair needs a portrait/landscape
  proportion; deferred.

The three pipe classes and the three speeds no longer share a drawing: each has its own
pattern or tick count, reviewed at the small size. The old `draw-line`, `speedometer`,
`dialog-ok-apply` and `align-horizontal-center` drawings were retired with them and no
longer appear in `actionicons::names()`.

## Checking the family

- `QT_QPA_PLATFORM=offscreen apps/punchi-fluid-qt/build/punchi-fluid --check-icons`:
  coverage in both directions, the cache bound and the palette key.
- `--icon-sheet <file> --theme light|dark --scale 2`: the manual review sheet at 16, 24 and
  32 px, and its 2× raster. This is local evidence, not a gate check.
- The window checks need a configured Qt build; CI does not configure it. The Rust tests
  are the CI-enforced part.
