# Review record: `valve.two_pressure`

    status: pending-300dpi
    date: 2026-11-24
    reviewer: automated pass, pending a person's sign-off
    function: two-pressure valve (AND); the outlet follows the lower inlet while both carry pressure

## The claim

- **Function, in one sentence**: an output appears at `2` only while both inlets carry
  pressure, and it follows the weaker of the two.
- **Family**: valve — a flat pressure element (AND), not a position valve.
- **Normative source**: **ISO 1219-1:2006**, clause **7.1.3.6**, registration number
  **`X10620`** (the 2006 copy is the edition read on this machine). Confirmed against
  **JIS B 0125-1:2020** (the 2012 basis), where the same figure sits at clause
  **7.1.3.5** — note the clause number differs between the editions; the registration
  number `X10620` does not. Modules named by the standard: `101V16` (functional unit for
  a dual pressure valve), `F040V1`, `401V1` (line) and `401V2` (port).
- **Ports**: `1a` supply, printed `1` (left) · `1b` supply, printed `1` (right) · `2`
  working (top). All three are drawn, with stubs and the small open connection circle.

## Reference and assumptions

- **Reference read**: the normative figure `X10620` in ISO 1219-1:2006 (7.1.3.6) and the
  same figure in JIS B 0125-1:2020 (7.1.3.5), rendered and measured. No page, scan or
  extract is stored in the repository; the figure was read for form and proportions only.
  The owner's circuit identified the component and its two-inlet arrangement.
- **Observed**: a horizontal rectangle 8M × 4M; the two inlets on opposite sides at
  mid-height; the outlet on the top edge at the centre; two short baffles hanging from
  the top edge either side of the outlet; a free double piston between the inlets, two
  vertical faces joined by a rod drawn as two parallel lines. No line runs through the
  inside of the body from one inlet to the other.
- **Required by the owner**: the two inlets both read `1` while staying distinct ids
  (`1a`, `1b`), and the outlet reads `2`.
- **Stated by the standard**: the figure's body is 8M × 4M (16 × 8 mm at M = 2 mm); each
  baffle is about 1M long and sits 2M either side of the centre; the piston faces are
  2M tall and 3M either side of the centre; the rod is a 0.5M bar between them. The
  figure prints no port numerals; the numerals here follow `RF053`.
- **Assumption**: the double piston is drawn static (ADR 0041 excludes it from the
  moving-part decision), and the figure's undimensioned depths are read from the drawing
  and recorded as derived: baffles 1M, piston faces 2M, rod 0.5M.

## Measures

The symbol box is 20 mm square (1 normalized unit = 20 mm). Features are given in
millimetres of that box; the figure's proportions are in M, with M = 2 mm.

| Feature | Requirement (figure) | Drawing (mm) |
|---|---|---|
| Body | one rectangle 8M × 4M | x 2.00 … 18.00, y 6.00 … 14.00 (16.00 × 8.00) |
| Inlet ports | on opposite sides, at mid-height | `1a` (0.00, 10.00), `1b` (20.00, 10.00) |
| Outlet port | top edge, at the body centre | `2` (10.00, 0.00) |
| Inlet stubs | 1M each side, stopping at the body edge | 0.00 … 2.00 and 18.00 … 20.00 |
| Outlet stem | from the top edge up to the port, no line inside the body | x 10.00, y 0.00 … 6.00 |
| Baffles | two, 1M long, hanging from the top edge 2M either side of the centre | x 6.00 and 14.00, y 6.00 … 8.00 |
| Piston faces | two vertical faces, 2M tall, 3M either side of the centre | x 4.00 and 16.00, y 8.00 … 12.00 |
| Rod | one bar 0.5M thick between the faces, drawn as two parallel lines | y 9.50 and 10.50, x 4.00 … 16.00 |
| Through path in the body | none: the figure draws no line between the inlets | none drawn |
| Numerals | one per port, clear of the body and of each other | `1` above the left stub, `1` below the right stub, `2` beside the outlet |
| Line class | every line solid working | `working` on every primitive |

## Evidence

- Symbol lint: passes (`every_builtin_definition_passes_the_lint`).
- Geometry fixture: `assets/symbols/fixtures/valve.two_pressure.txt` regenerated and read;
  the body moved from 10 × 8 mm to 16 × 8 mm in the 20 mm box (32 × 16 mm in the 40 mm
  fixture box), the through line and the line from the outlet to the piston are gone, and
  the two baffles, the two piston faces and the two rod lines appear.
- Solver: `a_two_pressure_valve_passes_the_lower_inlet`,
  `a_two_pressure_valve_vents_when_one_inlet_is_at_atmosphere` and the mass tests still
  pass; the drawing change does not touch the behaviour.
- 300 dpi sheet, print ink:
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- valve.two_pressure assets/symbols/review/valve.two_pressure-300.png`
- Light and dark captures, fitted to the drawing:
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- valve.two_pressure assets/symbols/review/valve.two_pressure-light.png --ink light --fit`
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- valve.two_pressure assets/symbols/review/valve.two_pressure-dark.png --ink dark --fit`

## Checklist

| Question | Answer |
|---|---|
| Is the body one equal square per position? | Not a position valve: it is a flat pressure element (ADR 0039); the figure's single 8M × 4M body is drawn. |
| Is every closed port of every position marked? | No position and no closed port. |
| Is every port the standard draws drawn? | Yes: two inlets and one outlet (`RF052`), all three with stubs and connection circles. |
| Do the numerals match the declared ports? | Yes: `1a`/`1b` print `1`, `2` prints `2` (`RF053`); the ids stay distinct. |
| If it is a machine, is the drive bar on the standard's side? | Not a machine. |
| If it is adjustable, is the arrow on the diameter or through the element, and filled? | Not adjustable. |
| Is every numeral clear of the envelope/frame, the body and the primitive lines? | Yes: the lint's envelope rule passes and the captures show the numerals clear. |
| Does the arrow point at the far end of the connection? | No flow arrow on this flat symbol: the double piston carries the meaning. |
| Are spring and actuator on the declared sides? | No spring and no actuator. |
| Is the symbol legible at palette size? | Yes: `valve.two_pressure-light.png`, `valve.two_pressure-dark.png`. |
| Is the symbol legible at 300 dpi? | Yes: `valve.two_pressure-300.png`; the second reader's signature is still owed. |

## Open items

- The second reader's signature is owed (the record is `pending-300dpi`).
- **The clause number differs between the editions**: ISO 1219-1:2006 prints the figure at
  7.1.3.6 and JIS B 0125-1:2020 at 7.1.3.5. The `X10620` registration number is the same;
  the review cites both instead of choosing one.
- The standard prints no port numerals; the two `1`s and the `2` are the project's
  `RF053` designations.
- The double piston is static in the drawing (ADR 0041 excludes it from the moving-part
  decision); the model still decides by pressure.

## Images

- `valve.two_pressure-light.png` (light theme, fitted)
- `valve.two_pressure-dark.png` (dark theme, fitted)
- `valve.two_pressure-300.png` (300 dpi, print ink)

## Sign-off

- **Status**: `pending-300dpi`
- **Reviewed by**: automated pass, pending a person's sign-off
- **Date**: 2026-11-24
- **What changed since the last record**: the body follows the figure's 8M × 4M (it was
  10 × 8 mm), the through line and the line from the outlet to the piston are removed,
  and the two baffles, two piston faces and the two-line rod were added from the figure.
