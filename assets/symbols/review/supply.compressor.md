# supply.compressor — conformance review

    status: complete
    date: 2026-11-24
    reviewer: Cristian Muñoz (owner), second reader
    function: compressor, 4 bar: the machine that makes the compressed air

## The claim

- **Function, in one sentence**: a machine driven from outside that takes air at
  atmospheric pressure and delivers it at a set gauge pressure.
- **Family**: supply (a machine, not a part).
- **Normative source**: ISO 1219-1:2006, 7.2.4, figure `X11400`, read in the owner's copy
  and confirmed on the 2012 basis in JIS B 0125-1:2020, 7.2.4. Rules: `RF052` (every port of
  a symbol is drawn), `RF053` (`1` inlet, `2` outlet). The owner's 2026-11-25 correction: the previous record read a drive bar into `X11400`; the bar belongs to another symbology, not the standard, and is removed. It is a correction, not a deviation.
- **Ports**: `1` intake, open to the atmosphere · `2` delivery, the pressure source.

## Reference and assumptions

- **Reference read**: the owner's capture of the reference application (classified: it
  identifies the component and its arrangement; it is not traced and not stored) and the
  standard's figure.
- **Observed**: the capture draws the circle, the outlined triangle at the top and the two
  connections; it does not draw a drive bar. The previous record added one from another
  symbology; the owner's 2026-11-25 correction removed it.
- **Required by the owner**: ports `1` (intake) and `2` (delivery), the standard's form.
- **Stated by the standard**: `X11400` consists of the circle, the outlined gas triangle with
  its point at the delivery and the two connections; `RF052` asks for every port, so the
  intake is drawn even when the circuit takes its air from the room. The standard prints no
  drive bar.
- **Assumption**: the intake is declared with the `exhaust` role, the schema's "port open to
  the atmosphere". The model keeps no check valve, so a hose fitted to the intake would vent
  the circuit; the review records it instead of pretending the machine is one-way.

## Measures

| Feature | Requirement | Drawing |
|---|---|---|
| Circle | the machine's body | centre (0,640; 0,450), r 0,280 of the 20 mm box |
| Triangle | outline (a gas), equilateral, its point on the delivery | apex (0,640; 0,170); base (0,544; 0,334)–(0,736; 0,334) |
| Ports | all of them drawn | `2` at (0,640; 0,060); `1` at (0,640; 0,858) |
| Stems | straight from the port to the body | delivery 0,060 → 0,170; intake 0,730 → 0,858 |
| Numerals | from the port table | `2` above the delivery, `1` below the intake |

## Evidence

- 300 dpi sheet, print ink: `supply.compressor-300.png`.
- Light and dark captures, fitted: `supply.compressor-light.png`, `-dark.png`.
- Symbol lint: passes.
- Drawing tests: `the_compressor_triangle_leaves_from_the_top_of_its_circle`,
  `the_compressor_has_no_drive_mark`.
- Model: `the_supply_reservoir_goes_on_its_supply_port` proves the reservoir lands on the
  delivery, not on whichever port a map yields first.
- Sheet: `a_format_fourteen_sheet_moves_the_compressor_hose_to_its_discharge` keeps an old
  hose on the delivery.

## Checklist

| Question | Answer |
|---|---|
| Is the body one equal square per position? | not a valve: the body is the circle |
| Is every closed port of every position marked? | not applicable: the intake is open to the atmosphere |
| Is every port the standard draws drawn? | yes: `1` and `2` |
| Do the numerals match the declared ports? | yes: `2` delivery, `1` intake |
| If it is a machine, is the drive bar on the standard's side? | no drive bar: X11400 prints none; the previous record read one from another symbology (owner's 2026-11-25 correction) |
| If it is adjustable, is the arrow on the diameter? | not this file: see the adjustable compressor |
| Is every numeral clear of the frame, the body and the lines? | yes, at the 300 dpi capture |
| Is the symbol legible at palette size and at 300 dpi? | yes, both captures |

## What is still open

- The 2012 text is read through the Japanese adoption (MOD), not the ISO text itself.
- The intake's `exhaust` role is a mapping that must be read with the review: the role says
  "open to the atmosphere", not "air leaves here".
- The numeral's room above or to the left (`RF053`) is an open question of the contract.

## Sign-off

- **Status**: `complete`
- **Reviewed by**: Cristian Muñoz (owner)
- **Date**: 2026-11-24
- **What changed since the last record**: nothing in the record changed; the
  second reader signed its measures and open items on 2026-11-24.

## Images

- `supply.compressor-300.png`
- `supply.compressor-light.png`, `supply.compressor-dark.png`

## Correction — 2026-11-25

The owner corrected the record: the line the previous review called a **shaft bar** is not
part of `X11400`; it is a visual mark that belongs to another symbology. It is removed from
the definition and the captures. This is a **correction**, not a deviation: the symbol is the
circle, the outlined gas triangle and the two connections.
