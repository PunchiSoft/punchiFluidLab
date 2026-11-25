# Review record: `valve.shuttle`

    status: complete
    date: 2026-11-25
    reviewer: Cristian Muñoz (owner)
    function: shuttle valve (OR); the outlet follows the higher-pressure inlet

## The claim

- **Function, in one sentence**: with pressure on either inlet, the ball is pushed off
  that inlet's seat and onto the other, so the outlet follows the higher of the two.
- **Family**: valve — a flat pressure element (OR), not a position valve.
- **Normative source**: **ISO 1219-1:2006**, clause **6.1.5.5** and **7.1.5.5**,
  registration number **`X10740`** (the 2006 copy is the edition read on this machine).
  Confirmed against **JIS B 0125-1:2020** (the 2012 basis), clause **6.1.5.5** and
  **7.1.5.5**, same `X10740`. Modules named by the standard for the figure: `101V16`
  (functional unit for a dual pressure valve), `2162V1` (seat of a non-return valve),
  `2163V1` (moving part of a non-return valve), `501V2` (connection of two fluid lines
  within a symbol), `401V1` (line) and `401V2` (port).
- **Ports**: `1a` supply, printed `1` (left) · `1b` supply, printed `1` (right) · `2`
  working (top). All three are drawn, with stubs and the small open connection circle.

## Reference and assumptions

- **Reference read**: the normative figure `X10740` in ISO 1219-1:2006 (6.1.5.5 and
  7.1.5.5) and the same figure in JIS B 0125-1:2020, rendered and measured. No page,
  scan or extract is stored in the repository; the figure was read for form and
  proportions only. The owner's circuit of 2026-11-22 identified the component and the
  two-signal case, and it is not stored or traced either.
- **Observed**: a horizontal rectangle with the two inlets on opposite sides at
  mid-height and the outlet on the top edge at the centre; a junction dot where the
  outlet branch meets the through path; two opposed V seats of **equal size** on the
  through path, with a ball drawn on the left seat at rest.
- **Required by the owner**: the two inlets must both read `1` while staying distinct
  ids (`1a`, `1b`), the outlet reads `2`, and the ball keeps the three positions of
  ADR 0041 (left, centre, right) as the live state.
- **Stated by the standard**: the figure's body measures 8M × 4M and the ball 1M across
  (M = 2 mm, so 16 × 8 mm and a 1 mm radius, read from the figure); the junction dot is a
  filled dot on the through path at the body centre. The figure prints no port numerals:
  it is a symbol, not a circuit. The numerals here follow `RF053` (1 inlet, 2 outlet).
- **Owner's correction, 2026-11-25**: the reference draws the two seats as **equal V**s; the
  previous record's "1M either side" reading and the later small/large reading were wrong.
  The two `1` numerals are **above** their stubs (`numeral_side = "above"`), not one above
  and one below. The ball's tangent centres follow the new seats.
- **Assumption**: the moving ball's centre and right positions (ADR 0041) are the
  simulator's state display and have no printed dimension in the standard; they are
  placed symmetrically about the body centre. The 20 mm project box maps the figure's
  8M × 4M body to 0.10 … 0.90 × 0.30 … 0.70, leaving 1M stubs at both sides.

## Measures

The symbol box is **25 × 17 mm**: the 16 × 8 mm body and the 4.5 mm margins the numerals
need outside it (the owner's general rule: a numeral never sits on the symbol). Features are
given in millimetres of that box; the figure's own proportions are in M, with M = 2 mm.

| Feature | Requirement (figure) | Drawing (mm) |
|---|---|---|
| Box | 25 × 17 mm, the body plus the numeral margins | `box = [25, 17]` |
| Body | one rectangle 8M × 4M | x 4.50 … 20.50, y 4.50 … 12.50 (16.00 × 8.00) |
| Frame | the body's edge, the numerals' boundary | `frame = [0.18, 0.2647, 0.82, 0.7353]` |
| Inlet ports | on opposite sides, at mid-height | `1a` (0.00, 8.50), `1b` (25.00, 8.50) |
| Outlet port | top edge, at the body centre | `2` (12.50, 0.00) |
| Inlet stubs | from the box edge to the body | 0.00 … 4.50 and 20.50 … 25.00 |
| Outlet stem | from the top edge to the through path | x 12.50, y 0.00 … 8.50 (4.50 outside the body) |
| Through path | **continuous** from inlet to inlet, the ball and the seats drawn on top | y 8.50, x 0.00 … 25.00, one segment |
| Ball | 1M diameter, seated on the left and **tangent** to the seat's arms, unfilled | centre (8.31, 8.50), radius 1.00; the circumference touches the left seat at its perimeter |
| Junction dot | filled, on the through path at the centre | (12.50, 8.50), radius 0.30 |
| Left seat | V opening towards the ball, the reference's equal arms | vertex (6.89, 8.50), tips (9.09, 6.32) and (9.09, 10.68) |
| Right seat | the same V mirrored about the centre | vertex (18.60, 8.50), tips (16.40, 6.32) and (16.40, 10.68) |
| Moving ball | left, centre and right seats, each tangent to its seat | x 8.31, 12.50 and 17.18 at y 8.50 |
| Numerals | one per port, **outside the body**, clear of the body and of each other; the two `1`s on the same side by the owner's reference | `1` outside the left of the body, `1` outside the right, `2` above the outlet (`numeral_side = "above"` for the two `1`s) |
| Line class | every line and the seat polylines solid working | `working` on every primitive |

## Evidence

- Symbol lint: passes (`every_builtin_definition_passes_the_lint`).
- Geometry fixture: `assets/symbols/fixtures/valve.shuttle.txt` regenerated and read; the
  through path is one segment, the ball is at the tangent centre and unfilled.
- The owner's 2026-11-24 correction is covered by three tests:
  `the_shuttle_through_line_is_continuous`,
  `the_shuttle_ball_touches_its_seat_without_crossing_it` and
  `the_shuttle_ball_is_not_filled`.
- The three states: `the_moving_ball_takes_the_left_centre_and_right_seats` still passes.
- 300 dpi sheet, print ink:
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- valve.shuttle assets/symbols/review/valve.shuttle-300.png`
- Light and dark captures, fitted to the drawing:
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- valve.shuttle assets/symbols/review/valve.shuttle-light.png --ink light --fit`
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- valve.shuttle assets/symbols/review/valve.shuttle-dark.png --ink dark --fit`

## Checklist

| Question | Answer |
|---|---|
| Is the body one equal square per position? | Not a position valve: it is a flat pressure element (ADR 0039), so the rule does not apply; the figure's single 8M × 4M body is drawn. |
| Is every closed port of every position marked? | No position and no closed port: the two seats are the checks, not blocking marks. |
| Is every port the standard draws drawn? | Yes: two inlets and one outlet (`RF052`), all three with stubs and connection circles. |
| Do the numerals match the declared ports? | Yes: `1a`/`1b` print `1`, `2` prints `2` (`RF053`); the ids stay distinct for the document and the solver. |
| If it is a machine, is the drive bar on the standard's side? | Not a machine. |
| If it is adjustable, is the arrow on the diameter or through the element, and filled? | Not adjustable. |
| Is every numeral clear of the envelope/frame, the body and the primitive lines? | Yes: the lint's envelope rule passes and the captures show the numerals clear of the body and of each other. |
| Does the arrow point at the far end of the connection? | No flow arrow on a flat pressure symbol: the seats and the dot carry the meaning. |
| Are spring and actuator on the declared sides? | No spring and no actuator. |
| Is the symbol legible at palette size? | Yes: `valve.shuttle-light.png`, `valve.shuttle-dark.png`. |
| Is the through line continuous from inlet to inlet? | Yes: one segment, with the ball and seats on top. |
| Does the ball touch its seat at the perimeter instead of crossing it? | Yes: its centre is the tangent distance from the vertex, proven by `the_shuttle_ball_touches_its_seat_without_crossing_it`. |
| Is the ball free of a fill that would cut the line? | Yes: the circle is unfilled (`the_shuttle_ball_is_not_filled`). |
| Is the symbol legible at 300 dpi? | Yes: `valve.shuttle-300.png`. |

## Open items

- The standard prints no port numerals; the two `1`s and the `2` are the project's
  `RF053` designations, and the record says so instead of claiming the figure prints them.
- The moving ball's centre and right seats are the state display of ADR 0041, not
  dimensions printed by `X10740`; the figure draws the ball on one seat only. The tangent
  centres are derived from the seat's own arms, not printed by the figure.
- The owner's 2026-11-24 correction is visual: the line, the tangency and the unfilled
  ball; the solver and ADR 0041 are unchanged.
- **Owner's 2026-11-25 correction**: the left seat is the **small V** and the right the
  **large V** of the reference; the two `1` numerals are **above** their stubs
  (`numeral_side = "above"`). The ball tangent centres follow the new seats.
- The two seat polylines are the reference's: the two Vs are equal and open towards the ball. The figure prints no dimensions; the sizes are read from
  the owner's reference and recorded.

## Images

- `valve.shuttle-light.png` (light theme, fitted)
- `valve.shuttle-dark.png` (dark theme, fitted)
- `valve.shuttle-300.png` (300 dpi, print ink)

## Sign-off

- **Status**: `complete`
- **Reviewed by**: Cristian Muñoz (owner)
- **Date**: 2026-11-25
- **What changed since the last record**: the owner's 2026-11-24 correction. The through
  line is one continuous segment, the ball's centre is at the tangent distance from each
  seat so the circumference touches the arms without crossing them, and the ball is
  unfilled so the line is not cut. The previous record described the line as interrupted
  by the ball; this record replaces it and waits for the owner's signature.
