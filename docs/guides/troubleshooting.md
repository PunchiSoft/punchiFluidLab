# Troubleshooting

Status: current. Find the symptom, then the cause and the shortest way out. When the answer
is "not implemented yet", the page says so instead of inventing a workaround.

## The window does not behave as I expect

### It opens with a circuit I did not draw

The window starts with a small example — a compressor, a service unit, a 3/2 valve and a
single-acting cylinder, with their pipes — so there is something to run on the first launch.
**Archivo ▸ Nuevo** starts that example again; it is not an empty sheet.

To clear it and draw your own, press `Ctrl+E` (**Todo**) and then `Supr`. Components you drop
afterwards are kept; the example is not added back.

### A component will not drop on the sheet

- The drag has to end **inside** the sheet. A drop on the sidebar or outside the frame is
  discarded without a message; drag again inside the frame.
- The sidebar only starts a drag in editing. Press **Detener** (`F5`) first; in simulation a
  press on a valve actuates it and does not place anything.
- The drop snaps to the 10 mm grid, so the symbol may move a few millimetres from the
  pointer.

### A pipe is refused

A connection is refused, with the reason in the status bar, when it would join a port to
itself, repeat an existing pipe, or name a component that is not on the sheet. Press on the
port itself, not on the line beside it, and release on the other port.

### A valve does not answer a click

- In **editing** a click selects the valve and opens its properties; it does not actuate it.
  Press **Iniciar** (`F9`) or **Paso** (`F10`) first.
- A spring-return valve is held: keep the pointer down, or hold `Espacio`. Release and the
  spring returns it.
- A valve without a spring latches: one press changes the position and it stays there.

### The cylinder does not move

1. Check the supply pressure in the properties of the compressor; the default is 6 bar.
2. Check the valve position: a spring-return valve is in its rest position until you hold
   it.
3. Run **Editar ▸ Comprobar**. An open working port, a pipe through a component or an
   overlapping pipe is reported there.
4. Confirm the pipe reaches the cylinder's working port, not its mechanical or exhaust port.

### The window is slow, or the GPU stays busy while nothing moves

The simulation timer stops when the model settles; an idle circuit does not keep computing.
If the desktop still shows the GPU busy, the compositor may be presenting frames for the
window at the panel's refresh rate, which is not the solver. The machine profile and the
measurement notes are in [engineering.md](../engineering.md).

## The check reports a fault

**Editar ▸ Comprobar** lists the faults and marks them on the sheet. The six this build
knows:

| Fault | What to do |
|---|---|
| a component is outside the sheet | drag it inside the frame |
| two components overlap | move one; the check uses the symbol boxes |
| a pipe crosses a component that is not its end | re-route the middle segment or move the component |
| two pipes overlap | re-route one of them |
| a pneumatic connection is open | connect the port or remove the component; an exhaust port may stay open |
| two cylinders have the same designation | give one a different name |

Three faults from the reference are not implemented: a pipe whose ends do not fit, a label
that does not match, and a pipe through an unrelated circuit. The last two need the
electrical library's encoder and a project of circuits, respectively.

## Files

### A catalog file is ignored

The catalog loads from `assets/catalog/` and from
`~/.local/share/punchi-fluid-lab/catalog/` (or `$PUNCHI_FLUID_CATALOG_DIR`). A file is
skipped, never fatal, when:

- `schema` is not 2 or 3, or a key is unknown (a typo is an error, not a default);
- `id` or `name` is empty, or a port id is empty or outside the symbol box;
- a number is outside its range, or a cylinder rod is not smaller than the bore;
- the symbol has a primitive outside the box, a line without a class, a zero-length line or
  a zero-radius circle.

**The window does not show the skipped file or its reason yet.** The catalog API records
both, but there is no panel for them. Until there is, check the file against the rules in
[assets/catalog/README.md](../../assets/catalog/README.md), fix the key, and restart the
window: there is no Reload button, the catalog is read at start.

A user file with the same `id` as a built-in one replaces it; `disabled = true` hides the
built-in one. The command line `check` uses the **built-in** catalog, so a user definition
is not seen there; the window is the place to try it.

### A sheet will not open

The window shows the reason in the status bar. `punchi-fluid-cli check <file>` prints the
same reason and exits `1`; the file is not repaired or partially loaded. A file written by a
newer build is refused rather than guessed; the format and its versions are in
[the sheet format](../reference/sheet-format.md).

### A component disappeared after opening a sheet

The sheet stores a catalog id. If that definition is not installed (or is `disabled`), the
placement is kept in the file but the symbol is not drawn, and the check does not report it
yet. This is a known gap. Install the definition or change the location with the catalog
folder described above.

### The title has a `•`

The sheet has work that is not in the file. **Archivo ▸ Guardar** clears it. Closing, Nuevo
or Abrir asks once before discarding it. A safety copy is written every thirty seconds and
offered on the next start; it is never loaded on its own.

### Where are the recent list and the safety copy?

In the application data folder. On Linux that is
`~/.local/share/PunchiSoft/Punchi Fluid Lab/`: `recent.json` for the last ten files and
`autosave.pfluid` for the safety copy. The catalog is separate, under
`~/.local/share/punchi-fluid-lab/catalog/`.

## Export

### The page is blank or a component is missing

Run **Editar ▸ Comprobar** first. A component outside the sheet is still exported, but it can
fall outside the printed page. The drawing is centred, not scaled; a circuit larger than the
area the frame leaves free can be cropped. Choose a larger paper or move the circuit inside
the sheet.

### The date in the title block is wrong

It is fixed at 2026-11-07 in this build; the date is not read from the system. This is a
recorded debt, not a setting.

### The export has no shortcut

The button hint says `Ctrl+E`, but that key marks everything on the sheet. Export has no
working shortcut yet; use **Archivo ▸ Exportar**. The wrong hint is a known bug with its own
fix.

### The PNG is too small

The raster is written at 300 dpi and the resolution field is not in the window yet. Export a
larger paper, or use `--dpi` on the command line (72–600) until the panel grows the field.

## The interface

### There is no search box

The sidebar is a flat list of the catalog and has no search yet. Scroll it. This is recorded
in [capabilities.md](../capabilities.md), not a hidden setting.

### The icons are missing or the font looks different

Icons come from the desktop's icon theme; a machine without the icon shows the label alone,
as a native application does. The palette follows the desktop and `--theme light|dark` pins
it for a screenshot. The font **family** from `kdeglobals` is read but not loaded yet; the
size and the palette are.

### The ribbon is a menu

Below 1100 px the ribbon folds its body into one **Menú** button plus the transport. That is
the designed behaviour, not a broken window; widen it to get the groups back.
