# The frame and the control-diagram title block

This page records what the standards say about the sheet around the drawing, what this
project takes from them, and what is a company layout of our own. It is the reference for
the default template and for the editable title block that follows.

> **A schematic is not a part drawing.** The owner's reference image is a title block for a
> **mechanical part**: it carries scale, material and units because a part is manufactured
> to size. A pneumatic schematic is a **control/automation document** — pneumatics is part
> of robotics — so its block follows the convention of **electrical diagrams** (IEC 61082-1)
> with the ISO 7200 data fields. In particular, a schematic is **not to scale**: the symbols
> are standard sizes, not a reduced copy of the machine. Scale, material, unit and
> projection symbol are deliberately absent from this block.
>
> **Status.** The centring marks are in the build and the older block no longer prints a
> scale. The block described below — its 180 mm width, its fields and its two bands — is the
> target of the next change; the editor and the per-circuit storage follow that.

## Which block this is

| Document | Convention | Fields that matter |
|---|---|---|
| Mechanical part drawing | ISO 7200 on an ISO 5457 sheet; ISO 5456-2 for the projection | title, number, revision, date, scale, material, unit, projection, owner |
| **Pneumatic / electrical schematic** | **IEC 61082-1** (documents used in electrotechnology) on an ISO 5457 sheet, with ISO 7200 data fields | title, document number, revision, date, sheet n/N, owner, prepared / checked / approved |

This is the difference the owner pointed out. The sheet frame, the borders and the centring
marks are still ISO 5457; the *content* of the block is the control-diagram one.

## The two standards

- **ISO 5457:1999** — *Technical product documentation — Sizes and layout of drawing
  sheets*. It fixes the paper sizes, the frame, the borders, the centring marks and where
  the title block sits. It does **not** fix the cells inside the title block: it refers to
  ISO 7200.
- **ISO 7200:2004** — *Technical product documentation — Data fields in title blocks and
  document headers*. It defines the data fields, their names, their content and their
  length. It does not prescribe one universal grid either; the layout is the company's.

So there is a standard for the **fields** and for the **place** of the block, not a single
mandatory drawing of the block. A program that claims otherwise is showing one company's
layout, not the standard.

### ISO 5457, the points that matter here

| Point | Requirement |
|---|---|
| Borders | 20 mm on the left edge, including the frame (filing margin); 10 mm on the other three |
| Frame | Continuous line, 0.7 mm wide, around the drawing space |
| Title block position | A0–A3: bottom right of the drawing space. A4: the shorter, lower part. The drawing reads in the direction of the block |
| Centring marks | Four, at the ends of the two symmetry axes of the trimmed sheet, symmetry tolerance 1 mm. **The form is free.** Recommended: continuous lines 0.7 mm wide, starting at the grid reference border and extending 10 mm beyond the frame. This project draws the line from the frame across the whole border to the paper edge: the form is free, and it keeps the 20 mm filing margin from looking like a gap next to the other three sides |
| Grid reference system | Optional: 50 mm fields with letters and numerals, 3.5 mm high, 0.35 mm lines |
| Trimming marks | Optional: two overlapping 10 mm × 5 mm rectangles in the four corners |

The arrows in the owner's reference drawings are a **free form** of the centring mark; the
line is the recommended part. This project draws the line from the frame outward and a small
triangle at the paper edge, and records that the triangle is our form, not a standard cell.

### ISO 7200 and IEC 61082-1, the fields we use

ISO 7200 groups the fields into **identifying**, **descriptive** and **administrative** data.
It limits the title block to a minimum and says that fields such as scale, projection symbol,
general tolerance and surface texture may be presented outside the block when used. IEC
61082-1 is the convention for documents used in electrotechnology; it uses the same kind of
data fields for a diagram, and a diagram is not to scale.

Our default **control-diagram** block uses:

| Our label (interface) | ISO 7200 field | Kind | Notes |
|---|---|---|---|
| Empresa o curso | Legal owner | identifying | One text field; a logo image is future work |
| Título | Title | identifying | The circuit title by default |
| Nº de documento | Identification number | identifying | The file stem by default |
| Revisión | Revision index | identifying | `A` by default |
| Fecha | Date of issue | identifying | Supplied by the caller; the core has no clock |
| Hoja n/N | Sheet number | identifying | `n` is the circuit position, `N` the number of circuits |
| Diseñó | Creator | descriptive | Empty until filled |
| Revisó | Checker / technical reference | descriptive | Company field; empty until filled |
| Aprobó | Approval | descriptive | Empty until filled |

The mechanical fields are **not** part of this block: no scale, no material, no unit and no
projection symbol. The export still fits a large drawing onto the page so it is not cropped,
but that is a print fit, not a drawing scale, and it is not written in the block.

The table is a **layout decision** of this project, made so a course can hand in a readable
schematic. It is not a claim that ISO 7200 or IEC 61082-1 mandates these cells in this order.

## Our geometry

- **Block width**: `min(180 mm, drawing space width)`, anchored bottom right. A4's drawing
  space is 180 mm wide, so the block spans it; on A3 and larger, the block is 180 mm at the
  right and the drawing keeps the space to its left. The width is our company layout; ISO
  5457 leaves it to ISO 7200 and the company.
- **Block height**: 32 mm, in two bands: prepared, checked, approved, date, revision and
  sheet above; owner, title and document number below. The height is our layout.
- **Frame**: 20 mm left and 10 mm elsewhere, 0.7 mm line, as ISO 5457.
- **Centring marks**: at the midpoints of the four sides, a 0.7 mm line from the frame
  across the border to the paper edge, with a small triangle at the edge. ISO 5457 allows
  the form.
- **Parts list**: above the block, sharing its width, so the bottom band reads as one block
  of information.
- **Drawing area**: the content inside the frame, minus the block and the parts list.
- **No scale, no material, no unit, no projection symbol.** The drawing is a diagram, not a
  part; adding those fields would promise a scale the schematic does not have.

### The scale: ISO 5455

There is a standard for scales: **ISO 5455** recommends 1:1, 1:2, 1:5, 1:10, 1:20, 1:50,
1:100, 1:200, 1:500 and 1:1000, with 2:1, 5:1 and 10:1 for enlargements. It applies to a
drawing that represents a real object at a ratio.

A **schematic is not one of those drawings**. IEC 61082-1 and ISO 1219-2 describe a diagram
whose symbols have standard shapes and readable sizes; it is not a reduced photograph of the
machine. That is why the block has no scale field. If a circuit is too large for A4, the
standard answer is a **larger ISO sheet** — A3 or A2, chosen in the ribbon — or smaller
symbols in the definition, not a scale written in the block. The export may fit the drawing
to the drawing space for reproduction; that fit is a print convenience, not a drawing scale,
and it is not recorded.

## What is still open

- The editable title block: the fields above are read-only in the first slice; the editor,
  the document storage and format 8 follow.
- The grid reference system and the trimming marks are optional in ISO 5457 and are not
  drawn yet.
- A logo image: the owner chose text only for the first slice; an image needs its own format,
  validation and renderer.
- The exact character lengths of ISO 7200 are not enforced yet; the editor will bound each
  field and the block will elide what does not fit.
- A4 landscape is allowed by this program even though ISO 5457:1999 permits only vertical
  A4; Amendment 1:2010 opens both. The sheet setting keeps both.

## Sources

- ISO 7200:2004, *Technical product documentation — Data fields in title blocks and document
  headers*, [ISO catalogue](https://www.iso.org/standard/35446.html).
- ISO 5457:1999, *Technical product documentation — Sizes and layout of drawing sheets*.
- ISO 5457:1999/Amd 1:2010, orientation of A4.
- IEC 61082-1:2014, *Preparation of documents used in electrotechnology — Part 1: Rules*,
  for the control/electrical diagram convention and the absence of a drawing scale.
- ISO 5455:1979, *Technical drawings — Scales*, for the recommended scales that a schematic
  does not use.
- ISO 1219-2:2012, *Fluid power systems and components — Graphical symbols and circuit
  diagrams — Part 2: Circuit diagrams*, for the pneumatic diagram itself.

The standards are not redistributed here. The fields are summarised in our own words for the
implementation; the normative text remains the standard.
