# Export a drawing

Status: current. This is the how-to: take the sheet that is on screen and write it as a PDF
for a report or a PNG for a slide.

## Before you export

1. **Detener** the simulation, so the sheet is in editing.
2. **Editar ▸ Comprobar** and fix what it reports. A component outside the sheet is still
   exported: the fit includes it, so it will be on the page, perhaps smaller than the rest.
3. Save the sheet if you want the export and the file to match.

What leaves is the document in the session — the same placements, pipes, designations and
per-instance values that are on screen — not a fixed example.

## Choose the paper

The **Hoja** group in **Archivo** lists the nine named papers and the landscape toggle. The
paper in use is marked. The list comes from the export crate, so the window cannot offer a
size the writer does not know.

| Paper | `--paper` index | Size (portrait) |
|---|---|---|
| A5 | `0` | 148 × 210 mm |
| A4 | `1` | 210 × 297 mm |
| A3 | `2` | 297 × 420 mm |
| A2 | `3` | 420 × 594 mm |
| A1 | `4` | 594 × 841 mm |
| A0 | `5` | 841 × 1189 mm |
| Letter | `6` | 215.9 × 279.4 mm |
| Legal | `7` | 215.9 × 355.6 mm |
| Oficio | `8` | 216 × 330 mm |

**Apaisado** swaps width and height; the drawing is the same. The export starts on the paper
this machine prints on (Oficio where the system says Chile, A4 elsewhere), which is a
starting point and not a decision.

The custom size exists in the export API but has no field in the window yet.

## Write the file

**Archivo ▸ Exportar** opens a save dialog:

- a name ending in `.pdf` writes a vector page;
- a name ending in `.png` writes a raster at 300 dpi.

The page carries the frame, the title block, a legend of line classes and port numerals, and
— while any symbol review is open — the note `symbols not yet verified against ISO 1219-1`.
Two exports of the same circuit produce the same bytes.

**Archivo ▸ Hoja ▸ Márgenes, tamaño y lista…** opens the panel that decides what the page
carries: a named paper or a custom size in millimetres, the four margins, the raster
resolution and whether the parts list is printed. The panel never touches the document; it
only says how the sheet leaves.

**Archivo ▸ Exportar** offers three files: PDF for printing, PNG for a slide and **SVG** for
a drawing that stays vector. The SVG is the same page — frame, block, parts list and circuit
— measured in millimetres, its text readable and its lines editable, and two exports of the
same sheet are byte for byte the same file.

The sheet also carries a **parts list** in a column above the title block: one row per
definition with how many of it, its catalog name and the designations of the group. The
column takes its width from the drawing, so a long list shrinks the diagram instead of
overlapping it.

The title block takes the **name of the file** you write, the active circuit's name in the
project field, and the date of the day the window runs. A drawing larger than the area the
frame leaves free is scaled down to fit; the scale is recorded in the sheet field of the
title block (`1:1`, `1:2`, …). A smaller drawing keeps its size and is centred. The page
prints the **active circuit**; switch circuits before exporting to print another one.

## Without the window

The same command is available for a script or a job:

```bash
apps/punchi-fluid-qt/build/punchi-fluid --export plano.pdf
apps/punchi-fluid-qt/build/punchi-fluid --export plano.png --paper 8 --landscape --dpi 300
```

- `--paper N` uses the index in the table above; an index outside the list is refused.
- `--landscape` rotates the page.
- `--dpi N` applies to PNG only, from 72 to 600; PDF is vector and ignores it.
- The file extension decides the format, exactly as in the dialog.

The gate uses this path to write a PDF and an Oficio PNG and to check both headers.

## What this build does not do

These are recorded in [capabilities.md](../capabilities.md) and in
[ADR 0012](../adr/0012-the-paper-a-drawing-is-printed-on.md); they are not hidden:

- **There is no panel for margins, custom size or the PNG resolution**, and the last paper
  choice is not remembered per user.
- **There is no parts list** in the title block yet.
- **The scale is a fit, not a chosen ratio.** The export reduces a drawing that does not
  fit and records the resulting scale; it does not let the user pick a drawing scale.
- **The title block has no editable project or revision field** in the window yet; the
  block carries the file name, the date, `A` and the paper.

## If the page is wrong

- Blank or missing a component: return to [troubleshooting](troubleshooting.md); a component
  with a catalog id that is not installed is kept in the file but not drawn.
- The wrong paper: the Hoja group marks the current choice; choose again before **Exportar**.
- The drawing is very small: it was fitted to the page; the title block records the scale.
  Choose a smaller paper or move the components closer together.
- `Ctrl+Shift+E` does not open the dialog: the shortcut needs the window to have focus. The
  **Archivo ▸ Exportar** command always works.
- A PNG that is too small: the resolution is fixed at 300 dpi in this build; export a larger
  paper, use `--dpi` on the command line, or wait for the resolution field.
