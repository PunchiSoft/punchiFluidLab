# Your first circuit

Status: current. This is the tutorial: draw a small pneumatic circuit, connect it, run it,
check it and save it. It takes about ten minutes. The interface is Spanish, so the commands
are named in Spanish with the English meaning beside them.

If you only want to watch something move, open the window and press **Iniciar**; the example
that comes with the program is already a complete circuit. The rest of this page builds one
from the catalog.

## 1. Open the window

```bash
./scripts-dev/run.sh
```

The window has three regions: the ribbon on top, the catalog on the left and the sheet in
the middle. The status bar at the bottom shows pressure, stroke, velocity and simulated
time.

The window opens with a small example: a compressor, a service unit, a 3/2 valve and a
single-acting cylinder, with the pipes between them. That example is a starting point, not
an empty document, and **Archivo ▸ Nuevo** starts it again. To clear it and draw your own,
press `Ctrl+E` (**Todo**) and then `Supr`.

## 2. Place three components

The left sidebar **is** the catalog: every row is a component and its thumbnail is the
symbol the sheet will draw. Scroll it and drag a row onto the sheet.

Place these three, left to right:

- **Compresor, 6 bar** — the supply.
- **Válvula 3/2, retorno por muelle** — the valve.
- **Cilindro de simple efecto, 20/100** — the actuator.

The drop snaps to the 10 mm grid. If the release is outside the sheet, nothing is placed;
drag the row again inside the frame.

A component you do not need can be selected with a click and removed with `Supr`. `Ctrl+Z`
undoes it.

## 3. Connect the ports

Every symbol has numbered ports: the small stubs at its edge. To make a pipe:

1. Press on the port where the air leaves.
2. Drag to the port where it enters.
3. Release.

Connect the supply's **1** to the valve's **1**, and the valve's **2** to the cylinder's
**1**. The valve's **3** is its exhaust; leave it open. The pipe is drawn as an orthogonal
run; a middle segment can be dragged later to re-route it without losing the connection.

A connection is refused, never silently dropped, when it would join a port to itself, repeat
an existing pipe, or name a component that is not on the sheet. The status bar says which.

## 4. Name the cylinder (optional)

A name is what makes a schematic readable, and it is what the parts list will use.

1. Make sure the transport is stopped: the sheet is in editing, not simulation.
2. Click the cylinder. The properties panel opens on the right.
3. The first field is **Designación**. Type `1A1` and press **Aplicar**.

The name is drawn under the symbol and saved with the sheet. The other fields in the panel
edit the values of this one instance — pressure, conductance, stroke, bore, rod and moving
mass — and **Aplicar** is one undo entry, not one per keystroke.

## 5. Run it

Press **Iniciar** (`F9`). The model starts and the transport stays visible.

- A spring-return valve is **held**: press the valve symbol with the pointer down, or hold
  `Espacio`, and the spool stays actuated; release and the spring returns it.
- A valve without a spring latches: one press changes the position and it stays there.
- The cylinder moves. The status bar shows the pressure, the stroke and the velocity.

The rest of the transport:

| Command | What it does |
|---|---|
| **Pausa** (`F8`) | stops the clock but stays in simulation |
| **Paso** (`F10`) | advances one frame and pauses |
| **Hasta cambio** | runs until a piston reaches an end or a valve changes position |
| **Detener** (`F5`) | returns to editing |
| **Restablecer** (`Ctrl+R`) | goes back to `t = 0` |

The speed selector beside the transport changes how much simulated time one frame carries.
The default is slowed down on purpose: a 20 mm cylinder with this valve finishes its stroke
in about 0.13 s of simulated time, which is too fast to watch at real speed.

In simulation, a click on a valve actuates it. In editing, the same click selects it and
opens its properties. The mode is the transport's, not a separate button.

## 6. Check it

Press **Detener** to return to editing, then **Editar ▸ Comprobar**. The check looks at the
sheet before you run it and marks each fault where it happens; the panel lists them.

The six faults this build knows are: a component outside the sheet, two components on top of
each other, a pipe through a component, two pipes on top of each other, a pneumatic port
with nothing attached, and two cylinders with the same designation. Three more faults from
the reference are not implemented yet; the ledger in
[capabilities.md](../capabilities.md) says which and why.

## 7. Save it

**Archivo ▸ Guardar** (`Ctrl+S`) asks for a file the first time and writes a `.pfluid`
sheet. **Guardar como** (`Ctrl+Shift+S`) chooses another file; **Abrir** (`Ctrl+O`) reads
one back.

The title shows `•` while the sheet has work that is not in the file. Nuevo, Abrir and
closing the window ask before discarding it. A safety copy is written every thirty seconds
while the work is unsaved and is **offered** on the next start; it is never loaded on its
own. The last ten files are in **Archivo ▸ Recientes**.

## 8. Move, copy and turn

- Drag a component to move it; the drop snaps to the grid.
- `Shift`+click adds or removes one component from the selection; dragging on empty sheet
  draws a rubber band; `Ctrl+E` marks everything.
- `Ctrl+C`, `Ctrl+V` and `Ctrl+D` copy, paste and duplicate. `Shift`+drag copies the
  component under the pointer.
- `Alt+R` turns a quarter clockwise, `Alt+M` reflects it on its own axis. `Ctrl` and a
  double click turns anticlockwise.
- `Supr` removes the selection; `Ctrl+Z` and `Ctrl+Shift+Z` walk the history.
- With more than one component marked, **Editar ▸ Alinear** lines the edges up on the
  component already furthest out.

## 9. Several circuits in one project

A `.pfluid` file is a **project**: it can hold several circuits, each a sheet of its own.
The **Circuito** group in **Archivo** lists them and marks the one on screen; **Nuevo
circuito** adds an empty sheet and opens it, and **Renombrar** changes its name. The solver,
the drawing and the export work on the active circuit, and the other circuits keep their
components, pipes and values.

A pipe whose ends belong to different circuits is the ninth check fault; the editor never
creates one, and saving the project keeps the circuits together.

## 10. Where to go next

- [Export the drawing](export-a-drawing.md) to PDF or PNG.
- [Troubleshooting](troubleshooting.md) when a gesture does nothing.
- [The sheet format](../reference/sheet-format.md) and [the CLI](../reference/cli.md) for
  what the saved file contains and how to check it without the window.
