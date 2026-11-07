# Symbology: the normative contract

The diagrams this program draws must be readable as fluid-power diagrams by anyone
who reads them on paper. That is a hard requirement, not a stylistic preference: a
schematic that is almost right is worse than one that is obviously a sketch.

## The standards that apply

| Standard | Edition in force | What it governs here |
|---|---|---|
| ISO 1219-1 | 2012 | Graphical symbols for conventional use and data-processing applications |
| ISO 1219-2 | 2012 | Circuit diagrams |
| ISO 1219-3 | 2016 | Symbol modules and connected symbols in circuit diagrams |
| ISO 6358 | — | Flow through a pneumatic restriction: sonic conductance and critical ratio |

### National adoptions

A norm is what a country applies, not what an international committee published. Where the
standard has been adopted nationally, the adoption is the text a reader here is expected to
follow, and it is the one to cite in a review:

| Country | Adoption | Status in this repository |
|---|---|---|
| Chile | **NCh-ISO 1219**, the INN adoption of ISO 1219 (symbols, circuit diagrams and symbol modules) | **not verified from this machine**: neither the number nor the in-force year has been confirmed against the INN catalogue, so it is named here as the adoption to check, not quoted as fact |

A symbol is **not conformant** until the adoption that applies to its reader has been
confirmed and written in its review file. Until then the review says `pending`, and the
export labels the document as unverified — which is what
[assets/symbols/README.md](../assets/symbols/README.md) already promises.

Confirmed against the ISO catalogue on 2026-11-06: 1219-1:2012, 1219-2:2012 and
1219-3:2016 are the current editions. Port numerals follow the fluid-power
port-identification convention used with those symbols (1 pressure, 2 and 4 working,
3 and 5 exhaust, 12 and 14 pilot). **Before the conformance review in M1.5, confirm
the current edition of the port-identification standard in the ISO catalogue**; it
was not verifiable from this machine.

The standard text is copyrighted and is not reproduced here, in the repository or in
the application. What follows is our own engineering contract: the requirements we
implement and the checks we run, expressed in our own words. Reading the standard
itself is the responsibility of whoever adds or reviews a symbol.

## How a symbol is built

### Directional control valves

- **One square per position**, equal size, sharing edges, side by side. Position *N*
  of the definition file is the *N*-th square **counted from the spring end**: the
  square against the spring is the one the spool takes when nothing actuates it, so it
  is the rest square. Two positions, two squares; three positions, three squares.
- **The actuator and the spring sit at opposite ends**, actuator at one end and return
  spring at the other, outside the body. A valve with `spring_return = true` and no
  spring symbol is a defect, and so is a spring drawn at the actuator end. Both are drawn
  **at a size proportional to the body** (a tenth of the body side was the defect that made
  them unreadable at palette size), and both start clear of the body edge.
- **A lever is an arm and a grip**, not a stem with a blob: the arm leaves the end face at
  an angle and ends in a bar **across** that arm, which is the part a hand pushes. A stem
  that turns into a hook reads as a stray line and leaves the reader unable to tell a
  manual lever from a detent or a pedal.
- **The ports are drawn once**, at the square of the rest position: one set of
  numerals, stubs attached to the body at fixed x. Port positions are read inside the
  symbol box and then brought to the body edge, so a stub is straight and vertical (or
  horizontal) whatever the body margins are.
- **Inside each square**, the flow path of that position, drawn **entirely inside that
  square**: a straight line from the port it leaves to the port it reaches, with the
  arrowhead at the port the flow goes to. A path never crosses into the neighbouring
  square, because that is what lets a reader look at one square and read one state. The
  two squares end up mirrored, which is the signature of a real two-position symbol.
- A port that a position closes carries the blocking mark **inside that square**: the
  port's line runs on past the body edge into the square and ends in a bar across it,
  the T of a shut-off. A mark outside the body would belong to no square; drawn inside,
  each half of the valve states its own position and the two can be compared.
- **Ports are identified with numerals** placed outside the body next to their
  stub, at the position of the port in the definition. The numerals are the same in
  every square, which is what makes the valve readable when the spool moves.
- Line classes are not interchangeable: working lines solid, pilot lines dashed,
  drain lines dashed and distinct, electrical connections dotted. The renderer draws
  each class with its own pattern; a definition declares which class a line belongs
  to instead of picking a random dash pattern.

### Actuators and the rest

- A single-acting cylinder shows the barrel, the piston, the rod **and its return
  spring** at the rod side, between the piston and the rod-side wall. The spring **spans
  the chamber it fills**, from the piston to that wall: it is the return element of that
  chamber and not a decoration in one corner of it. It is drawn thin around the rod and
  **compresses with the stroke**: same wire, fewer pitch, slightly larger amplitude as the
  coils bunch. A coil that no longer fits at its nominal pitch is
  drawn with **fewer turns** rather than with more lines stacked on the same millimetre,
  because a band of turns drawn over each other is a filled rectangle and not a spring;
  the floor is two turns, below which the shape stops reading as a coil. The coil is drawn
  in the **same ink and weight as the other functional lines**: ISO 1219 has line classes,
  not faded lines, and a grey part is the first one to disappear in a photocopy.
- The piston **crosses the internal length the definition declares**: the chamber is the
  stroke plus the dead length, so at full extension the piston stops with only the dead
  length left against the rod-side wall. That residual space is where the return spring
  is compressed, and it is not a fraction chosen by the renderer: a cylinder that only
  used part of its chamber would read as a cylinder that had not finished its stroke.
- The piston **meets both walls of the bore** at every position of the stroke. A line that
  stops short of the walls reads as a bar floating inside the barrel; the piston is the
  seal between the two chambers and is drawn as such.
- A double-acting cylinder shows the same barrel, piston and rod and **no spring**, with
  **two ports, one at each end**: the cap-side port on the centreline at the cap, and the
  rod-side port **in the rod-end wall, above the rod**, because the rod occupies the
  centreline at that end. A port tucked into the corner where the rod-end wall meets the
  barrel's top edge reads as part of the outline and not as a connection, so it is not
  used.
- The moving parts come from the definition, not from a hardcoded layout: the barrel is
  the widest rectangle the file declares, so a longer cylinder is a different file. The
  piston never leaves the barrel at either end of the stroke, and the rod is the only
  part that reaches outside the body. The rod is one piece of constant length: it shows
  outside the barrel at rest and its tip reaches the edge of the symbol box at full
  stroke, so it is never swallowed by the barrel nor drawn over its neighbour.
- A supply shows the source symbol; a restrictor shows the restriction; a silencer
  shows its own symbol. When in doubt, the shape comes from Part 1, not from a
  neighbouring product's marketing.
- Junction dots: a filled dot where three or more lines meet; no dot where lines
  merely cross. A crossing that looks like a junction is a defect.

### Circuit diagrams (Part 2/3)

- Components are grouped and numbered, and the diagram carries an identification
  block: title, project, date, sheet, revision.
- Lines are orthogonal; a line never passes behind a symbol; a connection is drawn
  between ports, never between arbitrary points on the body.
- The sheet has a frame and margins, and the drawing sits inside the content area —
  the same rule the PNG and PDF export uses.

## What the renderer guarantees

Independent of what a definition file says:

1. Squares of a valve are equal and aligned; a definition cannot make them ragged.
2. Arrowheads point along the flow, and a blocked port always shows its mark.
3. Port numerals are drawn from the `port` table, so the file cannot disagree with
   the drawing.
4. No meaning depends on colour alone: the spool position is a shape change, and every
   colour-coded value also appears as a number.
5. A symbol never renders below a legibility floor; below it the canvas draws a
   simplified marker instead of an illegible smudge.

## Conformance review

Automated (`symbol lint`, part of the catalog validation, milestone M1.5):

- number of squares equals the number of positions;
- every position has at least one path or at least one blocking mark;
- every port has a numeral, unique, and consistent with its role;
- `spring_return = true` implies a spring symbol is declared;
- actuator declared for `actuation = toggle` or `momentary`;
- no primitive outside the `0..=1` box, no zero-length line, no zero radius;
- line class declared for every polyline;
- minimum legibility: stroke and gaps above the floor at the smallest palette size.

Manual, once per symbol, signed off in the pull request:

- the five checks above plus a side-by-side against the Part 1 symbol for that
  function, done by reading the standard — not by comparing with another program;
- the symbol is legible at 24 px (palette) and at 300 dpi (PDF export);
- a screenshot of the symbol in light and dark themes is attached to the PR.

## Current state, honestly

The structural half of the contract is **implemented**, the visual half is **reviewed at
palette and canvas size**, and the 300 dpi half is **open** until the export exists.

| Requirement | State | Evidence |
|---|---|---|
| One equal square per position, impossible to make ragged | **enforced** | the renderer splits the declared `body`; the lint checks the width |
| Blocking mark on every closed port of a position | **enforced** | drawn from `blocked`, never authored |
| Port numerals from the `port` table | **enforced** | `parts::draw_port_numeral`; the file has no shape to disagree with |
| Flow path arrows along the connection | **enforced** | the path is routed from `connection`, the arrow points at its far end |
| Spring and actuator as declared parts | **enforced** | `[spring]` / `[actuator]`, required by the lint |
| Line classes with their own dash pattern | **enforced** | `line_class` on every line and polyline |
| Flow direction shown on a line that carries air | **enforced** | solid arrowhead on the line, straight from the sign of the flow |
| Junction dots at three-way junctions | **not drawn** | needs pipes (M4) |
| Title block on the sheet itself | **not drawn** | M2 |
| Legible at palette size (24–30 px) | **reviewed** | screenshots in `assets/symbols/review/` |
| Legible at 300 dpi | **open** | needs the export pipeline (M2) |
| Second reader's signature | **open** | the reviews are an automated pass |

Each built-in has a review file in [assets/symbols/review/](../assets/symbols/review/)
with `status: pending-300dpi`. The reviews are compiled into the binary
(`punchi-fluid-catalog::reviews`), and the PDF export asks them whether it may drop the
note “symbols not yet verified against ISO 1219-1” from its title block: it may not while
any review is not `complete`. A test asserts the compiled reviews cover every built-in,
and another asserts the exported file carries the note while they are open.

The PDF is vector, so the drawing is exact at any print resolution; what is still
missing is a reviewed 300 dpi raster per symbol as evidence.

One symbol is weaker than the others and says so in its own review: the fixed
restrictor's throat was drawn without a copy of the standard at hand. A
conformant-looking document that is not conformant is the one outcome this project
cannot afford, so the file states it instead of hiding it.
