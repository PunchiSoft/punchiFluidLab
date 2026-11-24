# Review record: `silencer.pneumatic`

    status: complete
    date: 2026-11-24
    reviewer: Cristian Muñoz (owner), second reader
    function: vents an exhaust to atmosphere through a restriction

## The claim

- **Function, in one sentence**: an exhaust fitted with this restriction breathes to
  atmosphere through a fixed conductance, so the actuator it serves moves more slowly
  than one venting through an open port; the noise itself is not modelled.
- **Family**: accessory — a termination (ADR 0016).
- **Normative source**: **ISO 1219-1:2006**, clause **8.7.37**, registration number
  **`2033V1`** (silencer). Confirmed against **JIS B 0125-1:2020**, clause **8.7.37**,
  same `2033V1`. **The standard draws a rectangle 4M × 2M with three relief lines at 1M
  spacing** (two from one edge and one from the other, each 1M long), at the end of a
  pipe whose axis is dotted. **Our contract draws a cone that widens away from its port
  with two relief lines** (ADR 0016, `docs/symbology.md`); it is not the figure's form.
  The difference is recorded here and in `docs/symbology.md` and is an open item, not a
  claimed conformance.
- **Ports**: `1` exhaust — the only port, and it is only ever fitted on an exhaust
  (catalogue rule: a silencer needs exactly one exhaust port).

## Reference and assumptions

- **Reference read**: the normative figure `2033V1` in ISO 1219-1:2006 (8.7.37) and the
  same figure in JIS B 0125-1:2020 (8.7.37), rendered and measured. No page, scan or
  extract is stored in the repository; the figure was read to record what it draws and to
  measure its box. The project's cone is the contract's form, not a trace of the figure.
- **Observed**: the standard's box is 8 × 4 mm at M = 2 mm (4M × 2M), with three relief
  lines 2 mm (1M) long at 2 mm (1M) spacing: the first and third hang from the top edge,
  the second rises from the bottom edge. The pipe arrives on the left with a dotted axis.
- **Required by the owner**: a cone that widens away from its port with two relief lines,
  fitted on an exhaust (the approved plan's form).
- **Stated by the standard**: the silencer is `2033V1`, a rectangle with relief lines.
  The standard does not draw a cone under any registration number read for this pass.
- **Assumption**: the contract's cone keeps the project's form until the owner rules on
  the difference. Its throat is at the port and its mouth is the far end, so the taper
  widens in the flow direction and never reads as a nozzle that speeds the air up. The
  two relief lines are placed inside the widening cone, each spanning the cone's section.

## Measures

The closure is drawn in a 10 mm square box (the project's accessory box), with its own port
on the box's edge facing the host. The standard's comparable measures are in M, M = 2 mm.

| Feature | Requirement | Drawing in the 10 mm box (mm) |
|---|---|---|
| Port | one, exhaust, on the side the host arrives | `1` at (0.00, 5.00) |
| Stem | the port's own stub to the throat | 0.00 … 2.80 along y = 5.00 |
| Cone | throat at the port, mouth away from it, widening in the flow direction | apex (2.80, 5.00); mouth x = 7.80, y 2.20 … 7.80 (5.60 tall) |
| Relief lines | two, across the widening cone, contained by its walls | x = 5.40, y 3.60 … 6.40 (2.80 long); x = 6.80, y 2.80 … 7.20 (4.40 long) |
| Line class | the body is a component, drawn solid working | `working` on every primitive |
| Acoustic model | none: the file states a conductance only | `conductance = 4.0e-10` |

The standard's `2033V1` measures, for comparison only: a rectangle 8 × 4 mm (4M × 2M),
three relief lines 2 mm (1M) long at 2 mm (1M) spacing, the first and third from the top
edge and the middle from the bottom edge.

## Evidence

- Symbol lint: passes (`every_builtin_definition_passes_the_lint`).
- Geometry fixture: `assets/symbols/fixtures/silencer.pneumatic.txt` regenerated and read;
  the cone's apex moved to the port side and its mouth to the far end (the drawing used to
  narrow away from the port), and the two relief lines now stop on the cone's walls
  instead of crossing past them.
- The closure at the port: `a_silencer_owns_the_mouth_of_the_jet` and
  `the_closure_is_drawn_at_the_port_and_does_not_cover_the_numeral` pass; the closure is
  drawn after the port stub and has no ring of its own, and the host's numeral stays.
- 300 dpi sheet, print ink:
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- silencer.pneumatic assets/symbols/review/silencer.pneumatic-300.png`
- Light and dark captures, fitted to the drawing:
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- silencer.pneumatic assets/symbols/review/silencer.pneumatic-light.png --ink light --fit`
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- silencer.pneumatic assets/symbols/review/silencer.pneumatic-dark.png --ink dark --fit`

## Checklist

| Question | Answer |
|---|---|
| Is the body one equal square per position? | No body and no position: it is an accessory, not a position valve. |
| Is every closed port of every position marked? | Not a position valve; it is a restriction, not a closed port. |
| Is every port the standard draws drawn? | Yes: one exhaust port (`RF052`), the only port the standard draws. |
| Do the numerals match the declared ports? | The port is `1`; when fitted as a closure its numeral is suppressed and the host's stays. |
| If it is a machine, is the drive bar on the standard's side? | Not a machine. |
| If it is adjustable, is the arrow on the diameter or through the element, and filled? | Not adjustable. |
| Is every numeral clear of the envelope/frame, the body and the primitive lines? | The host numeral is checked by the draw test; the closure's own numeral is not printed. |
| Does the arrow point at the far end of the connection? | No arrow: the cone and its relief lines carry the meaning. |
| Are spring and actuator on the declared sides? | No spring and no actuator. |
| Is the symbol legible at palette size? | Yes: `silencer.pneumatic-light.png`, `silencer.pneumatic-dark.png`. |
| Is the symbol legible at 300 dpi? | Yes: `silencer.pneumatic-300.png`. |

## Open items

- **The `2033V1` difference**: the standard draws a rectangle with three relief lines; the
  contract (ADR 0016) draws a cone with two. The record does not claim the cone is the
  standard's figure. Settling it needs the owner to choose between the standard's box and
  the contract's cone; changing the form would also change ADR 0016 and this record.
- The acoustic part is not modelled: a conductance is the whole of the physics.
- The cone's relief lines are two; the standard's box carries three. The difference is
  recorded instead of adding a third line to make the cone look like the box.

## Images

- `silencer.pneumatic-light.png` (light theme, fitted)
- `silencer.pneumatic-dark.png` (dark theme, fitted)
- `silencer.pneumatic-300.png` (300 dpi, print ink)

## Sign-off

- **Status**: `complete`
- **Reviewed by**: Cristian Muñoz (owner)
- **Date**: 2026-11-24
- **What changed since the last record**: the cone was turned around so it widens away
  from the port, its two relief lines now end on the cone's walls, and the record states
  the 2033V1 box-versus-cone difference, the clause and the measures.
