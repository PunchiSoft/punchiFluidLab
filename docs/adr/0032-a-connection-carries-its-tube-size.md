# 0032 — A connection carries its tube size

- **Status:** accepted
- **Date:** 2026-11-20
- **Deciders:** project owner, implementation session
- **Context crates:** `punchi-fluid-core`, `punchi-fluid-file`, `punchi-fluid-doc`,
  `punchi-fluid-draw`, `punchi-fluid-export`, `punchi-fluid-ffi`, `apps/punchi-fluid-qt`

## Context

A pneumatic hose is drawn at one fixed stroke and the parts list names only the components.
A student who is about to build the circuit cannot tell a 4 mm pilot line from a 16 mm main
run, and the printed page cannot be handed to a workshop as a cut list. The owner asked for
the tube diameter on the connection, a metric and an inch series that reaches 16 mm and more,
10 mm by default, a line that thickens with the tubing and a parts row with its length.

The value has to travel through the document, the file, the drawing, the parts list and the
interface, and it must not break the exact history comparison: `Connection` and `StoredPipe`
derive `Eq`/`Hash`, and the middle-segment offset already stores integer tenths for the same
reason. A floating-point diameter would either lose those derives or make NaN comparisons
non-reflexive.

The **physics is a separate node** (P-024b): conductance, volume, pressure drop, velocity and
the sizing warning need a reference and a tolerance. This decision stores and draws the size;
the solver keeps its current pipe model. The **electrical connection is deferred** at the
owner's request (P-026): it does not work as planned, so this change does not use, fix, hide
or test a wire as a tube.

ISO 1219 defines symbols, line classes and line widths, not nominal tube diameters. The
series below is the project's own practice list, not a normative ISO table.

## Decision

- **The size is integer hundredths of a millimetre** (`Option<u16>`), bounded `100..=5000`
  (1–50 mm). `None` means unspecified: an older sheet or a wire. A new pneumatic connection
  starts at **10 mm** (`1000`), a documented constant and not a page setting.
- **The metric and inch entries are separate.** The metric list is 4…50 mm; the inch list is
  the exact converted value rounded to the hundredth (`1/8"` is 318, not 300), through `1"`.
  A custom value is accepted inside the bounds and printed as a measurement with a decimal
  comma. The list is in `punchi-fluid-core::tube` and is enumerated to QML through the ABI;
  nothing copies it into QML.
- **The drawing uses a discrete line-width series**: 4–5 mm draws at **0.25 mm**, 6–8 mm at
  **0.35 mm**, 10 mm and above at **0.50 mm**; an unspecified old pipe keeps the historical
  **0.42 mm**. The cap is below the **0.7 mm** sheet frame, so a pipe never competes with the
  border. The class pattern (working solid, pilot dash, drain dash, electrical dotted) is
  unchanged, and an electrical wire keeps its old stroke.
- **The electrical wire is detected by the two `PortRole::Electrical` roles**, not by the
  stored class, because the sheet format cannot store `LineClass::Electrical` and the ABI
  saves it as `Working`. The parts list skips it, the width does not apply, and `connect`
  gives it `None`.
- **The parts list carries one row per `(od, class)`** with the **drawn route length** in
  whole millimetres: the segments of the route plus the port leads, rounded to the nearest
  millimetre. An unspecified pipe gets a `Tubo sin especificar` row, so the tubing is not
  silently omitted. The page states that the length is the drawn route, without bend
  allowance, fittings or waste, and the row cap reports what it cut.
- **Format 13** stores `tube_od_hundredths` as an optional, omitted-when-absent field. A
  format-12 sheet reads as `None` and keeps its 0.42 mm width; a newer file is still refused
  by the existing message. **P-021 (port closures) takes format 14.**
- **One undo** changes the size (`Change::Tubed`), and a split inherits the original pipe's
  size on all three connections instead of resetting to 10 mm.
- **The interface is one non-modal gallery** in the **Conducto** group, with the metric
  entries, the inch entries and **Sin especificar**; the current entry is marked with a check
  and a text value, never colour alone. The status bar reads `Tubo 10 mm` or
  `Sin especificar`, and an electrical wire shows no size. `Alt+D` opens the list. The ABI
  enumerates the series (`pf_tube_size_*`) and exposes
  `pf_session_pipe_is_electrical` so the status text can tell an unspecified pipe from a
  wire; `PF_ABI_VERSION` bumps to 11.
- **The width is a symbology change**, so it carries a manual review: the series and the
  frame separation are recorded in `docs/symbology.md`, with light/dark captures at 4, 10 and
  16 mm. The automated symbol lint does not cover pipe strokes; the manual review is the
  control.

## Consequences

- The document, the file and the parts list hold an exact integer; history equality and the
  undo comparison keep working, and the parts list keeps `Eq`.
- The drawing thickens with the tube while the frame stays the strongest line; a 4 mm and a
  6 mm line differ at 300 dpi, which a continuous map would not show.
- An old sheet opens unchanged and is reported as unspecified instead of being guessed as
  10 mm; the workshop sees that the tubing is unknown.
- The parts list is longer: components print first, tubes after them, and both counters say
  what the row cap dropped.
- The solver ignores the size until P-024b lands. The drawing says nothing about a physics
  it does not have.
- The electrical wire gains no tube, no row and no width. P-026 records the deferral.
- **Debt:** the tube length has no bend allowance, fittings or waste, and the size does not
  reach the model. P-024b pays the physics; the cut-list allowance would need its own
  decision.
- A new `StoredPipe` field means a migration test and a one-past-the-limit test for `0`,
  `99`, `5001`, `65000` and a string.

## Alternatives considered

- **Store the diameter as `f32`.** Loses `Eq`/`Hash` or makes history equality non-reflexive
  on NaN; the offset's integer tenths are the precedent.
- **One metric/inch table with nominal pairs.** Rejected: `1/4"` is 6.35 mm, not 6 mm; the
  entries are separate and the stored value is exact.
- **A continuous width map.** Rejected: it is not the 0.25/0.35/0.5 series, the 0.04 steps
  are invisible at 300 dpi, and the 0.5 cap keeps the frame separate.
- **A configurable 10 mm default in the page setup.** Two sources of truth for a new pipe;
  the gallery changes the pipe it is applied to.
- **Include the electrical wire as a tube.** Rejected by the owner: the electrical connection
  is deferred (P-026); it gets no tube, no row and no width.
- **Add connection copy/paste to inherit the size.** The app has no pipe copy/paste today;
  adding one is a product feature, not a tube-size change.
- **A static list in QML.** The core series and the QML list would drift; the ABI enumerates
  the series once.
- **A `f64` route length.** It would drop `Eq` from `PartsList`; an integer `length_mm` is
  exact for a cut list and covers 4 000 m.
- **A tube component in the catalog.** A tube is a connection property, not a symbol on the
  sheet.
- **Compute the conductance now.** P-024b needs a reference and a tolerance; a solver that
  guesses is worse than a stored size.

## References

- [reference/tube.md](../reference/tube.md): the series, the bounds, the width map and the
  parts row.
- [symbology.md](../symbology.md): the pipe width within the line-width and frame rules, and
  the manual review.
- [reference/sheet-format.md](../reference/sheet-format.md): format 13, the field and the
  migration.
- The owner's request of 2026-11-20 and the two Goldfish reviews of the specification
  (`specs/tube-size.md`).
- ADR 0026 (the sheet is the paper) for the frame the width has to stay below.
- P-024b (the physics) and P-026 (the electrical connection, deferred).
