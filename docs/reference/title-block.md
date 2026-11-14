# The sheet: frame, margins, reference zones and the title block

This page records what the standards and the two Chilean guides supplied by the owner say
about the sheet around the drawing, what this project takes from them, and what is a
company layout of our own. It is the reference for the default template and for the editable
title block.

> **A schematic is not a part drawing.** A part is manufactured to size, so its block carries
> material, units and a projection. A pneumatic or electrical schematic is a **control
> document**: its symbols are standard sizes and it is not a reduced photograph of the
> machine. The block therefore carries the data fields of `NCh 14.Of93` (`ISO 7200`), the
> sheet follows `NCh 13.Of93` (`ISO 5457`), and the scale starts at **`S/E`** ("sin escala")
> because the diagram has no ratio. Material, unit and projection symbol stay absent.
>
> **Status.** The build draws the sheet, the four centring marks, the optional reference
> zones and the control block on the canvas and in the export: **180 × 65 mm industrial on
> A3 and larger**, the guide's template, and **170 × 45 mm industrial on A4 and A5**, fitted
> so the block is not too large and leaves 10 mm at the left of A4's portrait drawing space.
> The school and technical models are shorter. Each circuit stores **twelve editable
> fields**, a **content model** and its **per-field visibility** (format 12), and edits them
> from the **Cajetín** ribbon tab and its docked editor, one undo per change. The sheet
> number `n/N` is derived from the project and is not stored.

## The two guides and the standards

The owner supplied two Chilean technical guides, summarised here in our own words and never
redistributed: one on **cajetines** for hydraulic and pneumatic drawings, and one on
**marcos, márgenes y formatos**. They separate the format of the sheet from the content of
the diagram, which is the distinction this page keeps.

| Reference | Subject | Use on the sheet |
|---|---|---|
| `NCh 13.Of93` / `ISO 5457` | Sizes and layout of drawing sheets | Paper sizes, margins, frame, centring marks and reference zones |
| `NCh 14.Of93` / `ISO 7200` | Data fields in title blocks | Organisation of the identification and administrative fields |
| `NCh 15` / `ISO 3098` | Technical lettering | Heights and proportions of the characters |
| `NCh 1193` / `ISO 128` | Technical drawing representation | Line types and principles |
| `NCh 1471` / `ISO 5455` | Scales | The scale field and the `S/E` indication |
| `NCh 1768/1.Of94` / `ISO 1219-1` | Fluid-power symbols | The diagram itself |
| `IEC 61082-1` | Electrotechnical documentation | The control/electrical convention of the block |

The guides warn that a national designation does not automatically adopt every edition of
the related ISO standard, and that the edition required by the contract or the company must
be confirmed in the INN catalogue. This page is therefore a design record, not a claim of
certification.

## The sheet: margins, frame and centring marks

`NCh 13.Of93` / `ISO 5457` fixes the drawing space and what surrounds it:

| Element | This project |
|---|---|
| Filing edge | 20 mm on the left, including the frame |
| Other margins | 10 mm on the top, right and bottom |
| Frame | Continuous line, 0.7 mm, around the drawing space |
| Centring marks | Four, at the midpoints of the symmetry axes: a 0.7 mm line from the frame across the border to the paper edge, with a small triangle at the edge. The form is free; the line is the recommended part |
| Reference zones | Optional in ISO 5457; drawn from A3 upwards (see below) |

The A4 sheet therefore has a drawing space of 180 × 277 mm in portrait and 267 × 190 mm in
landscape; A3 has 267 × 400 mm in portrait and 400 × 267 mm in landscape, and A0 811 ×
1169 mm. The export and the canvas measure the same millimetres.

On A4 the fitted block is 170 mm wide inside the 180 mm drawing space, so it leaves 10 mm at
the left when the sheet is portrait. The guide is explicit that 170 mm is a reference for
the **identification zone** inside the block, not the mandatory outer width of every block;
the 180 × 65 mm proposal is an A3 template.

## The reference zones

The guide recommends the reference grid for large sheets and multi-sheet sets: it lets a
note, a valve or a terminal be located as **B3** instead of described by position. This build
draws it:

- **From A3 upwards** (`A3`, `A2`, `A1`, `A0`, and a custom sheet whose short side reaches
  297 mm). **A4 and the two American sheets keep a clean sheet**, because a one-page sketch
  does not need a grid it will never use.
- The drawing space is divided into fields of **at most 50 mm**.
- **Numerals** `1`, `2`, … run down the left and right bands, one per row.
- **Letters** `A`, `B`, … run across the top and bottom bands, one per column.
- Characters are **3.5 mm** high and the division rules are **0.35 mm**.
- Both the rules and the characters stay **inside the margin bands**: the drawing space
  itself receives no extra ink. The centring marks remain drawn over them.

The export and the canvas implement the same rule, and two export tests pin it: A3 carries
the letters and numerals, A4 carries none, and no reference rule enters the drawing space.

## The title block

The block is a company layout built on the guide's recommended template:

- **Outer size**: the industrial model is **180 mm wide × 65 mm tall on A3 and larger** and
  **170 mm wide × 45 mm tall on A4 and A5** (and on the other sheets below A3: Letter,
  Legal and Oficio), fitted to the sheet. The guide says the 180 × 65 mm proposal is for an
  A3 template and must be adjusted to the available space; on A4 the fitted block leaves
  10 mm at the left of the 180 mm drawing space when the sheet is portrait. Anchored at the
  **bottom right**. The program opens A4 landscape, where the same 170 mm block sits at the
  right of the 267 mm drawing space and the drawing keeps the rest.
- **Content models**: **Escolar** is 170 × 35 (A4) / 180 × 40 (A3), **Técnico** 170 × 40 /
  180 × 50 and **Industrial** 170 × 45 / 180 × 65. A hidden field collapses its cell; a row
  with no visible field disappears, so a personalized block is shorter. On a sheet that
  cannot hold the block both sides shrink to what the content area has.
- **Outer border**: 0.7 mm. Inner rules: 0.35 mm.
- **Positions**: **top right**, **bottom right** (the standard) and **bottom, centred and
  edge to edge**; the content combo's **Sin cajetín** hides it. A portrait sheet offers all
  three positions, a landscape sheet the two corners. The standard width is 180 mm on A3
  and 170 mm on A4. The fields are kept when it is hidden.
- **Nota**: a free multi-line note stored with the block and edited in the docked panel. It
  is **not printed yet**; the opposite-corner placement (top left when the block is bottom
  right, and the other way round) is the next change.
- **Parts list**: above the block, sharing its width, so the bottom of the sheet reads as
  one band of information.
- **Drawing area**: the content inside the frame, minus the block and the parts list. The
  live session reserves the same band before it seeds the example and before it reports the
  check's drawing space; a component dropped or moved under the block is clamped above it,
  so the canvas never hides a symbol behind the block. The reference zones stay in the
  margin bands and do not reduce this area.

### The five rows

The heights below are the industrial A3 template's 65 mm. On A4 every row is multiplied by
45/65, so the same five rows fit the 45 mm block; the cell widths scale the same way from
180 to 170 mm. The school and technical models use their own shorter rows. Text keeps its
ISO 3098 height, and a value drops one step (for example 5 → 4.5 mm) when the scaled row
cannot hold it; it never crosses the bottom rule.

| Row | Height (A3) | Cells | Content |
|---|---|---|---|
| 1 | 14 mm | 80 + 60 + 40 mm | Empresa / propietario · Proyecto / instalación · Equipo / sistema |
| 2 | 15 mm | 180 mm | Título |
| 3 | 12 mm | 70 + 20 + 25 + 35 + 30 mm | Nº de plano · Revisión · Hoja · Fecha · Escala |
| 4 | 12 mm | 60 + 60 + 60 mm | Dibujado · Revisado · Aprobado |
| 5 | 12 mm | 180 mm | Normas aplicadas |

### The fields

| Interface label | Stored field | Default | Notes |
|---|---|---|---|
| Empresa / propietario | `owner` | empty | Company or organisation responsible for the document; a logo image is future work |
| Proyecto / instalación | `project` | empty | Project, plant, line or area |
| Equipo / sistema | `equipment` | empty | Machine, power unit or pneumatic circuit |
| Título | `title` | the circuit title | The document title |
| Nº de plano | `number` | the file stem when printed | Unique, controlled document code |
| Revisión | `revision` | `A` | Revision index and issue state |
| Fecha | `date` | supplied by the caller | The core has no clock |
| Dibujado | `prepared` | empty | Who prepared the drawing |
| Revisado | `checked` | empty | Who checked it |
| Aprobado | `approved` | empty | Who approved it |
| Escala | `scale` | **`S/E`** | `S/E` means "sin escala"; the field stays editable for a company note |
| Normas aplicadas | `norms` | empty | Representation and symbology standards actually used |
| Hoja | derived | `1/1` | `n` is the circuit position, `N` the number of circuits; never stored |

The twelve stored fields travel per circuit in format 12, with the content model and the
visible field set. An older format 9 block reads as the industrial model with every field
shown; a format 8 block also gains the empty project and equipment, the `S/E` scale and no
norms.

### Content models and Personalizar

| Model | Default fields | Size on A4 / A3 |
|---|---|---|
| Escolar | owner (course), title, prepared (author), date; sheet derived | 170 × 35 / 180 × 40 mm |
| Técnico | the school fields plus number, revision, checked, approved and norms | 170 × 40 / 180 × 50 mm |
| Industrial | all twelve, including project, equipment and scale `S/E` | 170 × 45 / 180 × 65 mm |
| Sin cajetín | hides the block; every value is kept | — |

The model is chosen from a labelled **combo box** in the docked editor (Técnico, Escolar,
Industrial, Sin cajetín); the Cajetín ribbon tab shows the current model as one
`Contenido: X` item. **Personalizar** reveals a show/hide switch per field. Hiding keeps the
value, collapses the cell and marks the block **Personalizado**; **Restablecer modelo**
restores the model's field set. Selecting a model after **Sin cajetín** brings the block back
at the standard position in the same undo.

### Text heights

`NCh 15` / `ISO 3098` gives the heights; this block uses three of them:

| Element | Height |
|---|---|
| Title and company values | **5 mm** |
| Other field values | **3.5 mm** |
| Field labels | **2.5 mm** |

Labels are muted ink, values are black, and every value is elided to its cell width so a
long document number cannot run into its neighbour.

### The scale: S/E

`NCh 1471` / `ISO 5455` recommends 1:1, 1:2, 1:5, 1:10, 1:20, 1:50, 1:100, 1:200, 1:500 and
1:1000, with 2:1, 5:1 and 10:1 for enlargements. Those scales apply to a drawing that
represents a real object at a ratio. A **schematic is not one of those drawings**: its
symbols have standard shapes and readable sizes, so it is not a reduced photograph of the
machine. The guide lists a scale field for the template and accepts the indication **`S/E`**
when the drawing is not to scale, which is this project's default. If a circuit is too large
for A4, the standard answer is a **larger ISO sheet** — A3, A2 — not a false scale. The
export may fit the drawing to the drawing space for reproduction; that fit is a print
convenience, not a drawing scale, and it is not recorded.

## What is still open

- The trimming marks of ISO 5457 are optional and are not drawn yet. The reference zones
  are.
- A logo image: the owner chose text only for this slice; an image needs its own format,
  validation and renderer.
- The exact character lengths of the standards are not enforced yet; the editor bounds each
  field at 120 characters and the block elides what does not fit.
- ISO 5457:1999 permits only vertical A4; Amendment 1:2010 opens both. **The program now
  opens on A4 landscape by default**, and `A4 normal` in the ribbon returns to portrait; the
  sheet setting keeps both.

## Sources

- `NCh 13.Of93` (`ISO 5457:1980`), *Dibujos técnicos — Formatos y elementos gráficos de las
  hojas de dibujo*: sizes, margins, frame, centring marks and reference zones.
- `NCh 14.Of93` (`ISO 7200:1984`), *Dibujos técnicos — Cuadro de rotulación*: the data fields
  and the 170 mm identification zone inside the block.
- `NCh 15`, *Dibujos técnicos — Escritura técnica* (`ISO 3098`): character heights.
- `NCh 1471.Of93` (`ISO 5455:1979`), *Dibujos técnicos — Escalas*: the scales and the `S/E`
  indication.
- `NCh 1768/1.Of94` (`ISO 1219-1`), *Símbolos gráficos de potencia fluida*.
- `ISO 5457:1999` and `ISO 5457:1999/Amd 1:2010`, *Technical product documentation — Sizes
  and layout of drawing sheets*, [ISO catalogue](https://www.iso.org/standard/35446.html).
- `ISO 7200:2004`, *Technical product documentation — Data fields in title blocks and
  document headers*, [ISO catalogue](https://www.iso.org/standard/35446.html).
- `IEC 61082-1:2014`, *Preparation of documents used in electrotechnology — Part 1: Rules*,
  for the control/electrical diagram convention.
- `ISO 1219-2:2012`, *Fluid power systems and components — Graphical symbols and circuit
  diagrams — Part 2: Circuit diagrams*.

The standards and the two guides are not redistributed here. Their requirements are
summarised in our own words for the implementation; the normative text remains the standard.
