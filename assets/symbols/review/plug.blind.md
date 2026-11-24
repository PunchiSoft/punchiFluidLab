# Review record: `plug.blind`

    status: complete
    date: 2026-11-24
    reviewer: Cristian Muñoz (owner), second reader
    function: caps a port so the line ends there on purpose

## The claim

- **Function, in one sentence**: a plug fitted on a port closes it, so nothing leaves the
  circuit there; it is the closed port, not a valve.
- **Family**: accessory — a termination (ADR 0016).
- **Normative source**: **ISO 1219-1:2006**, clause **8.2.9**, registration number
  **`2172V1`** (closed path or port: a stub 1M long closed by a bar 1M across), and
  clause **8.2.10**, registration number **`F038V1`** (plug inside a fluid line: a square
  plug drawn in the line). Confirmed against **JIS B 0125-1:2020**, same clauses 8.2.9
  and 8.2.10, same registration numbers. **The two figures differ**: the port closure is
  drawn with `2172V1`'s bar; `F038V1`'s square is a plug *inside* a line and is not drawn
  here. The difference is stated, not hidden.
- **Ports**: `1` working — the port the plug caps. It is the only port, because nothing
  leaves on the far side.

## Reference and assumptions

- **Reference read**: the normative figures `2172V1` (8.2.9) and `F038V1` (8.2.10) in
  ISO 1219-1:2006 and in JIS B 0125-1:2020, rendered and measured. No page, scan or
  extract is stored in the repository. The F038V1 square was read to classify it and is
  not traced.
- **Observed**: `2172V1` draws a pipe or stub of length 1M whose mouth is closed by a bar
  across it, the bar's length also 1M; `F038V1` draws a square of side 1M on the line,
  which is the plug seen inside a fluid line.
- **Required by the owner**: a stem that ends in the blocking bar, one port, fitted on
  the port it closes.
- **Stated by the standard**: the bar across the mouth is the standard's mark for a
  closed path or port (`2172V1`); the bar's stroke is the symbol's own weight, and the
  figure prints no port numeral (a symbol, not a circuit).
- **Assumption**: the blocking bar is drawn at the same weight as the valve's blocked-way
  mark (1.4 × the base stroke), because it is the same mark and a reader learns one shape
  for "closed". The closure's own numeral is suppressed at the port, so the host's numeral
  stays the only number there.

## Measures

The closure is drawn in a 10 mm square box (the project's accessory box), with its own port
on the box's edge facing the host. The same definition drawn as a standalone component in
the 20 mm box keeps the same proportions.

| Feature | Requirement | Drawing in the 10 mm box (mm) |
|---|---|---|
| Port | one, on the side the hose arrives | `1` at (0.00, 5.00) |
| Stem | 1M long from the port to the bar | 0.00 … 6.20 along y = 5.00 |
| Blocking bar | 1M across the end, the closure mark | x = 6.20, y 1.90 … 8.10 (6.20 long) |
| Bar weight | the symbol's closure weight, as the valve's blocked way | 1.4 × the base stroke |
| Far side | no stub: the bar ends the line | nothing past the bar |
| Own numeral | suppressed when fitted on a port | no `1` printed beside the host's port |
| Host numeral | visible and clear of the closure box | kept by the host, outside the 10 mm box |

The figure's printed dimensions are 1M and 1M: the stub and the bar are the same length,
and the bar is as wide as the closed pipe. The drawing keeps that 1 : 1 relation; the
absolute size is the closure box's, as the standard allows a symbol to be scaled.

## Evidence

- Symbol lint: passes (`every_builtin_definition_passes_the_lint`).
- Geometry fixture: `assets/symbols/fixtures/plug.blind.txt` regenerated and read; the bar
  grew from 5.6 mm to 6.2 mm in the 10 mm scale (22.4 mm to 24.8 mm in the 40 mm fixture
  box) so that it equals the stem, and the stem and port are unchanged.
- The closure at the port: `the_closure_is_drawn_at_the_port_and_does_not_cover_the_numeral`,
  `the_closure_keeps_its_own_numeral_silent_and_the_host_numeral_visible` and
  `a_turned_host_takes_its_closure_to_the_turned_edge` pass unchanged; `draw_closures` is
  called after the body and its port stubs, so the closure is drawn on top of the stub,
  and it draws no selection ring of its own.
- 300 dpi sheet, print ink:
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- plug.blind assets/symbols/review/plug.blind-300.png`
- Light and dark captures, fitted to the drawing:
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- plug.blind assets/symbols/review/plug.blind-light.png --ink light --fit`
  `cargo run -p punchi-fluid-export --bin symbol-sheet -- plug.blind assets/symbols/review/plug.blind-dark.png --ink dark --fit`

## Checklist

| Question | Answer |
|---|---|
| Is the body one equal square per position? | No body and no position: it is an accessory, not a position valve. |
| Is every closed port of every position marked? | Not a position valve; its single port is the closed one and it is drawn ended by the bar. |
| Is every port the standard draws drawn? | Yes: one port, on the side the hose arrives (`RF052`; the far side has nothing to draw). |
| Do the numerals match the declared ports? | The port is `1`; when fitted, its numeral is suppressed and the host's stays visible. |
| If it is a machine, is the drive bar on the standard's side? | Not a machine. |
| If it is adjustable, is the arrow on the diameter or through the element, and filled? | Not adjustable. |
| Is every numeral clear of the envelope/frame, the body and the primitive lines? | The host numeral is checked by the draw test; the closure's own numeral is not printed. |
| Does the arrow point at the far end of the connection? | No arrow: a plug closes, it does not direct flow. |
| Are spring and actuator on the declared sides? | No spring and no actuator. |
| Is the symbol legible at palette size? | Yes: `plug.blind-light.png`, `plug.blind-dark.png`. |
| Is the symbol legible at 300 dpi? | Yes: `plug.blind-300.png`. |

## Open items

- **The F038V1 difference**: the standard's `F038V1` is a square plug inside a fluid
  line; a closure fitted on a port uses `2172V1`'s bar. The two are different figures and
  the record states which one the definition draws. If the owner wants the in-line square
  as well, it is a separate definition, not a variant of this one.
- The bar's absolute size is the 10 mm closure box's; the symbol-sheet evidence draws the
  same definition in its fixed 40 mm box, so the capture shows the same proportions, not
  the fitted size.
- The plug's own numeral is suppressed only when it is fitted as a closure; as a
  standalone placement the renderer prints `1`.

## Images

- `plug.blind-light.png` (light theme, fitted)
- `plug.blind-dark.png` (dark theme, fitted)
- `plug.blind-300.png` (300 dpi, print ink)

## Sign-off

- **Status**: `complete`
- **Reviewed by**: Cristian Muñoz (owner)
- **Date**: 2026-11-24
- **What changed since the last record**: the record was rewritten with the 2172V1 and
  F038V1 clauses and their difference, the measures in millimetres of the closure box,
  and the bar lengthened to the 1M : 1M relation the figure prints.
