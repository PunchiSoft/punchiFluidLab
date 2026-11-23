# Export a drawing

Status: current. This is the how-to: take the sheet that is on screen and write it as a PDF
for a report, a PNG for a slide or an SVG for a drawing that stays vector.

## Before you export

1. **Detener** the simulation, so the sheet is in editing mode.
2. Save the sheet if you want the file and the export to match. An export is not a save: it
   does not change the document's path, its modified mark or its safety copy.

What leaves is the document in the session — the same placements, pipes, designations and
per-instance values that are on screen.

## Choose the paper

The **Configurar página** group in **Archivo** carries the document's page:

- **Márgenes** opens the gallery of margin presets;
- **Orientación** switches between vertical and landscape;
- **Tamaño** lists A4, A3, A2 and a custom size.

The paper, its orientation and the margins belong to the document and are saved with it, so
a reopened sheet prints on the page it was drawn on. Every page change is one undo.

The custom size exists in the window; the nine named papers of the export API are not all
offered by the menu yet.

## Write the file

**Archivo ▸ Exportar** opens a save dialog. The name's extension decides the writer:

- `.pdf` writes a vector page;
- `.png` writes a raster at **300 dpi**;
- `.svg` writes the same page as text, measured in millimetres.

The page carries the frame and the title block, and while any symbol review is open it also
carries the note `symbols not yet verified against ISO 1219-1`. Two exports of the same
circuit produce the same bytes.

## What this build does not do

These are product gaps, not hidden behaviour:

- **There is no parts list on the exported page from the window.** The export crate can build
  one and its tests cover it, but the window passes `parts: None`.
- **There is no panel for margins, custom size, PNG resolution or the parts-list toggle.**
  The menu above is the whole page setup, and the PNG resolution is fixed at 300 dpi.
- **There is no headless export command in the Rust binary yet.** The export crate and its
  renderers are tested; the old `--export` command belonged to the retired Qt app.
- **There is no chosen drawing scale.** A drawing larger than the framed area is fitted to
  it; the user does not pick a ratio, and the block shows the document's own scale field,
  not a computed fit ratio.

## If the page is wrong

- Blank or missing a component: a component with a catalog id that is not installed is kept
  in the file but not drawn; return to [troubleshooting](troubleshooting.md).
- The wrong paper: **Configurar página ▸ Tamaño** marks the current choice; choose again
  before **Exportar**.
- The drawing is very small: it was fitted to the page. Choose a smaller paper or move the
  components closer together.
- A PNG is too small: the resolution is fixed at 300 dpi in this build; export a larger paper.
