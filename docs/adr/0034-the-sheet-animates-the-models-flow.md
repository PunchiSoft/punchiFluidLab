# 0034 — The sheet animates the model's flow, and the arrowhead says which way

- **Status:** proposed
- **Date:** 2026-11-21
- **Deciders:** project owner, implementation session
- **Context crates:** `punchi-fluid-solver`, `punchi-fluid-draw`, `punchi-fluid-export`,
  `punchi-fluid-ffi`, `apps/punchi-fluid-qt`

## Context

The circuit in motion is the behaviour a student reads first: a piston that moves while the
air that pushes it is invisible teaches the wrong model. The interactive sheet colours a
hose by its pressure, but it draws no flow at all. The fixed example in
`punchi-fluid-draw::schematic` draws a solid arrowhead from the network's averaged supply
and working flows, so the claim that a hose shows its direction is only true of the example.

The solver already computes the signed flow of every edge on each substep and discards it;
only the supply, working and part totals are exposed. The drawing has no per-hose flow to
read.

Two constraints decide the shape of the answer. The screen, the check and the printed page
come from one drawing, so the mark belongs in `punchi-fluid-draw` and not in the window. A
printed page cannot animate, so the direction has to survive without movement: that is what
an arrowhead is for, and it is why a travelling stroke on its own cannot be the whole mark.

The owner asked on 2026-11-21 to see the arrowhead. It is not borrowed from another
program: ISO 1219-1 shows flow direction with a solid arrowhead, this repository already
draws it in `symbol::draw_flow_arrow` for the symbols and for the example's hoses, and the
new mark reuses that primitive. What is ours is the stroke that carries it.

## Decision

- **Only the model's signed flow produces a mark.** The solver stores the flow of each
  document connection, indexed by the connection's position in the document's list, before
  the circuit and drawing filters, and it stores the **net flow of the frame** — the mean
  over the call's substeps — not the sign of one 2 us substep. A sealed line chatters
  between equal and opposite flows while it settles, and the single-substep reading made the
  sheet draw air running backwards on a line that was still. A connection the model cannot represent keeps its
  `None` slot; a modelled hose with no air reports `Some(0.0)`. The vector is borrowed on
  the frame path. A timer, a pressure difference, a pressure colour or a “running” flag may
  never draw a mark: a dead end with a pressure difference stays still.
- **The mark is a stream of solid arrowheads.** One arrowhead alone cannot work: it has to
  leave the hose and be born again at the source, and that restart is a visible jump. The
  drawing spaces several marks along the route, each a short stroke in the hose's own colour
  — a little thicker than the hose, so it reads against the line — with a solid arrowhead on
  its leading end, the same one the symbols and the example draw. An arrowhead enters as
  another leaves, so no frame is empty while the air moves, and the spacing is an exact
  lattice so two marks can never bunch together where the travel wraps. The head is never
  optional: a mark without it says nothing on a page that cannot travel.
- **The mark takes the hose's own colour**, the pressure blend the line already carries,
  and never the accent: on this sheet the accent means selection, and a selected hose must
  stay distinguishable from a hose with air in it.
- **The phase is a scalar owned by the front end.** `FluidSession` keeps `m_phaseS` and a
  `motion` flag on the theme's `motionMs`; `pf_session_build_frame` carries both. Start
  clears the phase, the transport's 16 ms tick advances it while running, pause freezes it,
  stop and restart clear it, and one step adds the step's own seconds. The solver, the ABI
  and the drawing run no clock. It is view state: the document, the file format, the undo
  history and the modified mark never see it.
- **The air moves with the flow, between a floor and a cap.** The stream's speed follows
  `|flow|` — a trickle drifts at no less than six millimetres per second, a strong current
  runs at no more than forty-five — so the sheet's ordering is the model's and a hose long
  enough takes longer to cross. The scale itself is a readability constant, not the physical
  air speed, and it is a debt. A sealed line chatters while it settles, which is why the
  solver publishes the frame's net flow and not one substep's sign.
- **Reduced motion and paper freeze one arrowhead.** With `AnimationDurationFactor = 0`, and
  on the printed or exported page, each hose that carries air shows a single static
  arrowhead at the middle of its route, in the direction of its flow, with no stream: the
  direction is information, the travel is motion. The page receives the model's flow with a
  zero phase and `motion = false`; with no live model it draws no mark, because no value
  claims a flow.
- **No allocation per frame.** A stream is at most eight short runs of points on fixed-size
  stack buffers and eight triangles; the phase and the flow are scalars. A route too short to
  hold a mark draws nothing rather than a smudge, and an electrical wire never draws one.
- **The ABI frame is 13.** `pf_session_build_frame` carries `phase_s` and `motion` and
  refuses a `motion` value that is not `0` or `1`; `PF_ABI_VERSION` is 13. `Pipe` carries
  `flow_l_min`, `phase_s` and `animate`.
- **No copy rule.** The stroke is hand-written from the model and the route; the arrowhead is
  our own primitive drawn to ISO 1219-1. No chevron, screenshot, trace or geometry from
  another program enters the drawing.

## Consequences

- A hose now shows what the air does, not only how much pressure it holds; a dead end and a
  closed valve stay still because the model says they carry nothing.
- The printed page keeps the direction the model reported, so a PDF, a PNG and an SVG read
  like the sheet; a page exported before the model exists carries no mark and invents
  nothing.
- `symbology.md`'s “flow direction” row is true for the document's sheet and not only for
  the example, and its evidence names both tests.
- The window has one clock. `--selftest` reads the read-only `phase` property and checks
  start, pause, stop, restart and a dead-end frame.
- **Debt:** the pace (a traversal in about three simulated seconds, hurried by at most a
  quarter, between 3 and 24 mm/s) is a readability constant, not the physical air speed. A
  later node may scale it with the tube's section when P-024b gives the tube its physics.
- **Debt:** the fixed example in `punchi-fluid-draw::schematic` keeps its own arrow routine
  and its averaged supply and working flows; merging the two call sites is a follow-up and
  is not part of this decision.

## Alternatives considered

- **A travelling stroke with no arrowhead.** Rejected: on a still page the stroke is a bar
  with no direction, and the requirement is a direction mark.
- **A single travelling mark.** Rejected on evidence: the owner's recording of 2026-11-21
  caught the one mark jumping from the sink back to the source and leaving the hose empty in
  between, which reads as flicker. The stream removes both the jump and the empty frame.
- **Animate from the pressure difference.** Rejected: a dead end has a pressure difference
  and no flow, and a closed valve would keep moving. The solver's signed flow is the only
  source that cannot lie.
- **Let the drawing own a clock.** Rejected: the export would need time, the screen and the
  page would disagree at the instant of printing, and the drawing crate would hold state
  the document does not describe.
- **Animate in QML or in the window's paint path.** Rejected: the printed page and the
  `--selftest` frame come from the ABI; a second animation layer would be a second drawing.
- **A `Vec` of points built per frame.** Rejected by the frame budget: the route is already
  computed and the stream fits in fixed buffers.

## References

- [work-queue.md](../work-queue.md) §10a: the owner's order and the acceptance criteria.
- [ui-ux.md](../ui-ux.md) §“Animation on the sheet”: the behaviour, the reduced-motion rule,
  the printed page and the no-allocation rule.
- [symbology.md](../symbology.md): the arrowhead and the “flow direction” requirement.
- [ADR 0003](0003-one-drawing-two-devices.md): one drawing, two devices, which is why the
  mark lives in `punchi-fluid-draw` and not in the window.
- [engineering.md](../engineering.md): the frame budget the stack buffer protects.
- The owner's order of 2026-11-13; the arrowhead kept on 2026-11-21.
