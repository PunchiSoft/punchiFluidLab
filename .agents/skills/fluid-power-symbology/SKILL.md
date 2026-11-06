---
name: fluid-power-symbology
description: ISO 1219 conformance workflow for pneumatic and hydraulic symbols in the catalog. Use when drawing or changing a symbol, adding a valve or actuator, running the symbol lint, or preparing a conformance review.
license: GPL-3.0-or-later
---

# Fluid-power symbology (ISO 1219)

A schematic that is almost right is worse than one that is obviously a sketch: it
misleads a reader who trusts it. This skill is the procedure for changing a symbol
without breaking that trust.

**Hard rule: never trace or copy a symbol** from another program, a screenshot, a
manual, a datasheet or the standard itself. Geometry is written by hand to conform,
and the pull request states which requirement was checked. The normative text is
copyrighted; this repository records *our* engineering contract, not the standard.

## Standards in force

| Standard | Edition | Governs here |
|---|---|---|
| ISO 1219-1 | 2012 | graphical symbols, conventional use and data processing |
| ISO 1219-2 | 2012 | circuit diagrams |
| ISO 1219-3 | 2016 | symbol modules and connected symbols |
| ISO 6358 | — | pneumatic flow through a restriction (solver, not drawing) |
| Port identification | to confirm | ISO 11727 (pneumatic) / ISO 9461 (hydraulic); see Open questions |

The full contract, the guarantees and the open items are in
[docs/symbology.md](../../../docs/symbology.md). Read it before this skill; if the two
disagree, the document wins and this skill is the bug.

## Vocabulary that changes the drawing

- **Function**: what the component does (3/2 directional valve, single-acting
  cylinder, pressure relief valve).
- **Position**: one state of a valve. A 3/2 valve has two positions, a 5/3 valve three.
- **Way**: a distinct port path. "3/2" means three ports, two positions.
- **Rest position**: the state with no actuation, held by the spring. Position *N* in
  a file is the *N*-th square **counted from the spring end**.
- **Port role**: `supply`, `working`, `exhaust`, `mechanical` in the catalog. The
  solver classifies a connection by roles, not by numerals.
- **Line class**: `working`, `pilot`, `drain`, `electrical`. Each has its own pattern;
  a definition declares the class, it never picks a dash pattern.
- **Pneumatic vs hydraulic**: same ISO 1219-1 symbol language, different circuit
  conventions (see the table below). This project simulates pneumatics today; a
  hydraulic symbol may be drawn, but its physics needs a model before it can be claimed.

## Procedure: changing or adding a symbol

1. **Name the function and the class.** Write the sentence the symbol must let a
   reader say: "pressed, 1 connects to 2; released, 2 connects to 3." If you cannot
   write it, the symbol is not understood yet.
2. **Collect the normative requirements** for that class from
   [docs/symbology.md](../../../docs/symbology.md) and the checklists below. Read the
   standard itself when you have access; do not copy its figures.
3. **Check the existing definition** in `assets/catalog/`. Copying the closest file to
   the user catalog and changing its numbers is the supported path
   ([assets/catalog/README.md](../../../assets/catalog/README.md)).
4. **Author the file, not the drawing.** Schema 2 declares parts:
   `body` (split into one equal square per position), `[[port]]` with `id`, `role`,
   `position`, `[spring]`, `[actuator]`, `[[behaviour.positions]]` with `connection`
   and `blocked`, and `[[symbol]]` primitives with `line_class`. The renderer draws the
   stubs, numerals, flow arrows and blocking marks — do not hand-draw them.
5. **Run the lint.** `cargo test -p punchi-fluid-catalog` runs the eight lint rules and
   their tests. A file that breaks a rule is skipped and reported, never fatal.
6. **Look at it at both sizes.** `PUNCHI_FLUID_PREVIEW=1 ./scripts-dev/run.sh` opens the
   catalog sheet; check palette size (24–30 px) and export size (300 dpi), light and
   dark.
7. **Write the review** in `assets/symbols/review/<id>.md`: status, date, reviewer,
   provenance of the geometry, automated-check table, visual-check table, what is
   still open. `pending-300dpi` keeps the export's "not yet verified" note; only
   evidence changes it.
8. **Open the pull request with the evidence**: the lint result, the two screenshots,
   the review file and the checklist item that was checked. A symbol change without a
   conformance review is not reviewable.

## Valve checklist

- One equal square per position, sharing edges, counted from the spring end. The
  rest position is the square against the spring.
- Actuator at one end, return spring at the opposite end, both outside the body.
  `spring_return = true` without `[spring]` is a defect; a spring at the actuator end
  is a defect.
- Ports drawn once, at the rest square: one set of numerals, stubs straight to the
  body edge. Numerals come from the port table and are identical in every square.
- Inside each square, only that position's flow paths: a straight line from the port
  it leaves to the port it reaches, arrowhead at the destination. A path never crosses
  into the neighbouring square.
- A port the position closes carries its blocking mark **inside that square**; each
  state must be readable on its own.
- Paths are declared as connections and classified by port roles. Never author an
  arrow by hand: the arrow direction is the connection.
- A 3-position valve shows the centre position too, with its own paths or blocking
  marks; no empty square.

## Actuator checklist

- Single-acting cylinder: barrel, piston, rod **and return spring at the rod side**,
  between piston and rod-side wall. The spring compresses with the stroke: same wire,
  fewer pitch, slightly larger amplitude.
- Double-acting cylinder: barrel, piston, rod, **no spring**, two working ports, one at
  each end, drawn above the barrel because the rod leaves on the centreline.
- The barrel is the widest rectangle the definition declares; a longer cylinder is a
  different file, not a code change. The piston never leaves the barrel; only the rod
  reaches outside the body.
- Moving parts come from the simulation state; fixed parts from the file. `kinematics =
  "linear_actuator"` declares that the renderer draws piston, rod and (only if
  `[spring]` exists) the spring.

## Circuit checklist (Parts 2 and 3)

- Orthogonal lines; a line never passes behind a symbol; a connection joins ports, not
  arbitrary points on a body.
- A filled dot where three or more lines meet; no dot where lines cross.
- Components grouped and numbered; the sheet carries an identification block (title,
  project, date, sheet, revision).
- The drawing sits inside the sheet frame and content area, the same geometry the PNG
  and PDF export uses.

## Pneumatic and hydraulic differences

| Concern | Pneumatic | Hydraulic |
|---|---|---|
| Supply | compressor / air supply symbol | pump symbol, reservoir and return line |
| Exhaust | exhaust port, often a silencer | return to tank; drain line to tank |
| Line classes | working, pilot, drain, electrical | pressure, return, drain, pilot |
| Energy storage | receiver | accumulator and pressure relief valve |
| Medium model | compressible, ISO 6358 | incompressible, different model (not implemented) |
| Port numerals | 1 pressure, 2/4 working, 3/5 exhaust, 12/14 pilot | same convention; confirm current port standard |

Do not put a pneumatic exhaust or a silencer in a hydraulic circuit, or a tank return
in a pneumatic one. The symbol set is shared; the circuit conventions are not.

## What the renderer guarantees (do not re-check by hand)

1. Valve squares are equal and aligned; a file cannot make them ragged.
2. Arrowheads point along the flow and a blocked port always shows its mark.
3. Port numerals come from the port table, so the file cannot disagree with the drawing.
4. No meaning depends on colour alone: the active square is a shape/ink change and
   values also appear as numbers.
5. A symbol below the legibility floor is replaced by a simplified marker.

These are enforced in `punchi-fluid-draw` and `punchi-fluid-catalog`; a bug there is a
code bug with a test, not a per-file workaround.

## Gotchas

- **Position index is counted from the spring end**, not from the left. A spring on the
  right reverses the reading; the test `every_position_shuts_its_blocked_port_inside_its_own_square`
  is the guard.
- **Do not add a second set of port numerals per square.** The numerals belong to the
  rest square and are read with the valve.
- **A flow path that touches the square border is a defect**, even if it looks better
  centred; it makes two states unreadable.
- **`line_class` is mandatory** on every line and polyline. `dashed = true` is for a
  part that is dashed by nature (a spring), never a substitute for a pilot line.
- **Primitives live in the `0..=1` box**, shared with port positions. No zero-length
  line, zero radius or stroke below the floor.
- **Legibility is part of conformance.** A symbol that cannot be read at palette size
  fails review even with perfect geometry.
- **The review status is evidence, not paperwork.** `pending-300dpi` is why the export
  still labels the document unverified; flipping it without a reviewed image is a lie.
- **No colour-only state.** Spool position must be visible as shape/ink, and every
  colour-coded value must also be a number.
- **No third-party asset**, including a "reference" PNG in the repository. If the
  provenance is not ours or hand-drawn, it does not enter.

## Open questions — do not invent the answer

- **Port-identification edition**: confirm the current ISO catalogue edition for the
  pneumatic and hydraulic port/control-mechanism identification standards before citing
  them in a review. The numeral convention used here (1 pressure, 2/4 working, 3/5
  exhaust, 12/14 pilot) is consistent with it, but the edition is not recorded.
- **300 dpi raster**: each built-in review is `pending-300dpi` until a reviewed image
  exists; the PDF is vector, but the evidence is not there yet.
- **Second reader**: the reviews are an automated pass; a second person's signature is
  open.
- **Junction dots and title block**: they need the pipe network (M4) and the sheet layout
  (M2); do not fake them in a symbol file.

When a normative question is open, the code says so and the export labels the document
unverified. A plausible-looking but non-conformant diagram is the worst outcome this
project can ship.

## Validation

```bash
cargo test -p punchi-fluid-catalog      # schema, lint rules, review coverage
cargo test -p punchi-fluid-draw         # geometry and position tests
./scripts-dev/check.sh                  # the whole gate

PUNCHI_FLUID_PREVIEW=1 ./scripts-dev/run.sh
PUNCHI_FLUID_WRITE_SAMPLE=1 cargo test -p punchi-fluid-export --test generate
```

Then the manual part: read the rendered symbol at 24–30 px in light and dark, at 300
dpi, against the checklist for its class, and record the result in the review file. The
pull request carries the screenshots and names the requirement checked.

## Sources

- ISO 1219-1:2012, ISO 1219-2:2012, ISO 1219-3:2016 — read the standard; do not
  reproduce it.
- ISO 6358 — pneumatic flow (solver side, see [solver-validation](../solver-validation/SKILL.md)).
- [docs/symbology.md](../../../docs/symbology.md) — the repository's normative contract.
- [assets/catalog/README.md](../../../assets/catalog/README.md) — schema 2 and the lint.
