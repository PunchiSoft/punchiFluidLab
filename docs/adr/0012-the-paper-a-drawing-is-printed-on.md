# 0012 — The paper a drawing is printed on

- **Status:** accepted in part; the "paper is a preference, not a document property"
  consequence is superseded by [0026](0026-the-sheet-is-the-paper.md), and the sizes and
  the rest of the rules remain
- **Date:** 2026-11-07

## Context

A circuit leaves the program as a PDF or a PNG, and the first question a drawing office asks
about a PDF is which sheet it is. Today `punchi-fluid-export::page::PaperSize` knows A4 and A3,
the raster and PDF tests measure the page from the file header, and the window has no export
command at all. The READMEs claimed "paper from A5 to A2 plus Letter and Legal", which was a
wish written as a fact.

The sizes are not interchangeable. A DIN A sheet has a fixed aspect ratio and halves cleanly,
which is what lets a drawing be reprinted at another size; US Letter and US Legal do not; and
**Oficio/Folio is 216 × 330 mm**, which is neither: longer than A4, shorter than Legal, and
0.1 mm wider than Legal — close enough to be mistaken for it, and not the same paper. An office in Chile stocks Oficio, an office in Germany stocks A4, and a program that
offers only one of them decides for the user.

## Decision

The paper set the export offers is:

| Family | Sizes | Notes |
|---|---|---|
| ISO 216 / ISO 5457 | A4, A3, A2, A1, A0 | The ISO world. Frame and margins follow ISO 5457: a 10 mm margin, 20 mm on the binding edge. |
| North American | Letter (8.5 × 11 in), Legal (8.5 × 14 in) | Written in inches and printed in millimetres once, so the drawing never depends on a locale. |
| Chilean | Oficio/Folio, 216 × 330 mm | Neither A4 nor Legal: longer than A4, shorter than Legal, and 0.1 mm wider than Legal. Named `Oficio` in the interface, sized in millimetres. |
| Custom | width × height in millimetres, bounded | For the sheet that is not on the list. |

Rules that hold for every size:

- **The drawing is scaled to fit**, never cropped: the sheet's logical area keeps its
  proportions and the scale is recorded.
- **The chosen size is written into the file** (PDF page box, PNG metadata and the title
  block), so a printed page can be identified after the fact.
- **Orientation is a property of the sheet, not of the drawing**: landscape and portrait are
  the same content on a rotated page.
- **A size outside the bounds is refused before allocating**, as the raster path already does.

## Consequences

- `PaperSize` grows from two variants to the set above, each with its millimetre size, and the
  existing tests that read the size back from the file header extend to it.
- The interface's export panel lists families, not a flat list of fifteen entries, and the
  last choice is remembered per user — paper is a preference, not a document property.
- Oficio exists because the project is developed in Chile; naming it is not an exotic extra,
  it is the paper the reader has in the printer.
- The window command that reaches the export is still missing (F5): this decision defines the
  target, and the phase that adds the command implements it.
