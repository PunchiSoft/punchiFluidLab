# The title block and the frame

This page records what the standards say about the sheet around the drawing, what this
project takes from them, and what is a company layout of our own. It is the reference for
the default template and for the editable title block that follows.

> **Status.** The centring marks are in the build. The title block described below — its
> 180 mm width, its fields and its two bands — is the target of this page and the next
> change; until it lands, the export still draws the older narrow block. The editor and the
> per-circuit storage follow that.

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
| Centring marks | Four, at the ends of the two symmetry axes of the trimmed sheet, symmetry tolerance 1 mm. **The form is free.** Recommended: continuous lines 0.7 mm wide, starting at the grid reference border and extending 10 mm beyond the frame |
| Grid reference system | Optional: 50 mm fields with letters and numerals, 3.5 mm high, 0.35 mm lines |
| Trimming marks | Optional: two overlapping 10 mm × 5 mm rectangles in the four corners |

The arrows in the owner's reference drawings are a **free form** of the centring mark; the
line is the recommended part. This project draws the line from the frame outward and a small
triangle at the paper edge, and records that the triangle is our form, not a standard cell.

### ISO 7200, the fields we use

The standard groups the fields into **identifying**, **descriptive** and **administrative**
data. It limits the title block to a minimum and says that fields such as scale, projection
symbol, general tolerance and surface texture may be presented outside the block when used.
It defines the field names, their content and their length; the exact table is in the
standard and is not copied here.

Our default company layout uses:

| Our label (interface) | ISO 7200 field | Kind | Notes |
|---|---|---|---|
| Empresa o curso | Legal owner | identifying | One text field; a logo image is future work |
| Título | Title | identifying | The circuit title by default |
| Nº de plano | Identification number | identifying | The file stem by default |
| Revisión | Revision index | identifying | `A` by default |
| Fecha | Date of issue | identifying | Supplied by the caller; the core has no clock |
| Hoja n/N | Sheet number | identifying | `n` is the circuit position, `N` the number of circuits |
| Diseñó | Creator | descriptive | Empty until filled |
| Revisó | Checker / technical reference | descriptive | Company field; empty until filled |
| Aprobó | Approval | descriptive | Empty until filled |
| Material | — | company | Product-specific; outside ISO 7200's scope |
| Escala | — (dynamic) | company | The print scale the page records |
| Unidad | — (dynamic) | company | `mm` by default; the geometry is millimetres |
| Proyección | Projection method | company | First angle by default; the symbol is future work |

The table is a **layout decision** of this project, made so a course can hand in a readable
sheet. It is not a claim that ISO 7200 mandates these cells in this order.

## Our geometry

- **Block width**: `min(180 mm, drawing space width)`, anchored bottom right. A4's drawing
  space is 180 mm wide, so the block spans it; on A3 and larger, the block is 180 mm at the
  right and the drawing keeps the space to its left. The width is our company layout; ISO
  5457 leaves it to ISO 7200 and the company.
- **Block height**: 32 mm, in two bands: responsibility, date, scale and unit above; owner,
  title, number, material, sheet and projection below. The height is our layout.
- **Frame**: 20 mm left and 10 mm elsewhere, 0.7 mm line, as ISO 5457.
- **Centring marks**: at the midpoints of the four sides, a 0.7 mm line from the frame
  outward, with a small triangle at the paper edge. ISO 5457 allows the form.
- **Parts list**: above the block, sharing its width, so the bottom band reads as one block
  of information.
- **Drawing area**: the content inside the frame, minus the block and the parts list.

## What is still open

- The editable title block: the fields above are read-only in the first slice; the editor,
  the document storage and format 8 follow.
- The grid reference system and the trimming marks are optional in ISO 5457 and are not
  drawn yet.
- The projection symbol of ISO 5456-2 is written as a text value for now; the glyph is a
  separate symbol review.
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
- ISO 5456-2, *Technical drawings — Projection methods — Part 2: Orthographic
  representations* (for the projection method field; the symbol is future work).

The standards are not redistributed here. The fields are summarised in our own words for the
implementation; the normative text remains the standard.
