# supply.compressor_adjustable — conformance review

    status: complete
    date: 2026-11-24
    reviewer: Cristian Muñoz (owner), second reader
    function: compressor, 4 bar, with an adjustable delivery

## The claim

- **Function, in one sentence**: the compressor of `supply.compressor` whose delivery can be
  set.
- **Family**: supply (a machine).
- **Normative source**: ISO 1219-1:2006, 7.2.4, figure `X11400`, plus 9.4.3, rule `RF040`
  (the adjustability arrow on the diameter), read in the owner's copy and confirmed on the
  2012 basis in JIS B 0125-1:2020. Also `RF052`, `RF053`. The owner's 2026-11-25 correction
  removes the drive bar the previous record read into `X11400`; it belongs to another
  symbology, not the standard, and the removal is a correction, not a deviation.
- **Ports**: `1` intake, open to the atmosphere · `2` delivery, the pressure source.

## Reference and assumptions

- **Reference read**: the owner's capture (classified, not stored) and the standard's
  figures `X11400` and `X11300` (the variable-displacement pump, the standard's own example
  of the adjustability arrow).
- **Observed**: the capture draws the arrow across the circle, out of both sides, with a
  filled head at the upper end.
- **Required by the owner**: the same symbol as the compressor plus the adjustable delivery.
- **Stated by the standard**: the arrow that indicates adjustability sits on the **diameter**
  of the energy-conversion symbol and may be made longer (`RF040`); the head is filled.
- **Assumption**: the arrow's reach (1,51 r at the tail, 1,68 r at the point) comes from the
  capture; the standard fixes the line, not its length. The head is a filled polygon because
  the standard's arrow is solid.

## Measures

| Feature | Requirement | Drawing |
|---|---|---|
| Body | the compressor's, unchanged | circle (0,640; 0,450), r 0,280; triangle apex (0,640; 0,170) |
| Ports | all of them drawn | `2` (0,640; 0,060); `1` (0,640; 0,858) |
| Arrow | on the diameter, at 45°, out of both sides | (0,341; 0,749) → (0,913; 0,177) |
| Head | filled, at the upper end, outside the circle | tip (0,973; 0,118) |

## Evidence

- 300 dpi sheet: `supply.compressor_adjustable-300.png`; light and dark captures fitted.
- Symbol lint: passes.
- Drawing test: `the_adjustable_arrow_crosses_the_centre_at_forty_five_degrees` (distance to
  the centre, 45°, head beyond the circle and filled).
- Model and sheet: the same two tests as the compressor, through the same intake and delivery
  ports.

## Checklist

| Question | Answer |
|---|---|
| Is the body one equal square per position? | not a valve: the body is the circle |
| Is every closed port of every position marked? | not applicable: the intake is open to the atmosphere |
| Is every port the standard draws drawn? | yes: `1` and `2` |
| Do the numerals match the declared ports? | yes: `2` delivery, `1` intake |
| If it is a machine, is the drive bar on the standard's side? | no drive bar: X11400 prints none; the previous record read one from another symbology (owner's 2026-11-25 correction) |
| If it is adjustable, is the arrow on the diameter? | yes, and the head is filled |
| Is every numeral clear of the frame, the body and the lines? | the capture shows `2` beside the arrow's head, clear of it |
| Is the symbol legible at palette size and at 300 dpi? | yes, both captures |

## What is still open

- The arrow's reach is the capture's, not the standard's; the standard allows any length.
- The same intake and numeral questions as `supply.compressor`.

## Sign-off

- **Status**: `complete`
- **Reviewed by**: Cristian Muñoz (owner)
- **Date**: 2026-11-24
- **What changed since the last record**: nothing in the record changed; the
  second reader signed its measures and open items on 2026-11-24.

## Images

- `supply.compressor_adjustable-300.png`
- `supply.compressor_adjustable-light.png`, `-dark.png`

## Correction — 2026-11-25

The owner corrected the record: the drive bar the previous review read into `X11400` belongs
to another symbology and is removed from the definition and the captures. It is a
**correction**, not a deviation; the body is the circle, the outlined gas triangle and the two
ports.
