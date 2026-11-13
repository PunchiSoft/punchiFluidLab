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

### The air line

The source and the parts that condition the air are a set, and a reader recognises them by
shape alone. Every one of them is a **square turned 45°** or an upright square, drawn from the
requirement and never traced, and every connection point is the small open circle of the
reference set so that a hose is visibly meant to arrive there.

| Symbol | Shape, and what carries the meaning |
|---|---|
| Compressed-air **network** | The bare **outline triangle** on a stem, **apex up at the connection**: the air arrives from the plant, the machine that makes it is drawn elsewhere, and the point is where the hose attaches. A triangle standing on its base would read as a source delivering downwards, away from its own line |
| **Compressor** | The same triangle inside a **circle**, apex up, its delivery leaving the top through that stem and ending in the connection circle |
| **Filter** | A square turned 45° with the flow crossing it from vertex to vertex; the line stops at each vertex and is not drawn through the inside |
| **Filter with manual drain** | The same diamond, plus the element as the **dashed line on the vertical diagonal** and the **drain stem** leaving the bottom vertex. Without the stem it is a filter, not a separator with a drain |
| **Oil lubricator** | The same diamond and **no dashed element**: nothing is filtered here, oil is added. The absent element is the difference |
| **Service unit, full** | A **dashed frame** around the three functions, in the order the air meets them: the filter diamond, the regulator as an **upright square with the two facing arrows** inside and its setting **spring on top**, and the regulator's **dial** as a gauge on a stem that taps the line after it. The spring carries a **diagonal arrow**, which is what says the setting is adjustable |
| **Service unit, simplified** | One body with the **dashed element line** and the **dial** inside it, and the **drain leaving the bottom** as an arrow. A plain rectangle is not this symbol |

Rules that hold for all of them:

- **The box is square.** The drawing is laid out as a horizontal band inside a square symbol
  box, so the normalised coordinates map to the same number of millimetres on both axes. A
  diamond drawn in a box that is wider than it is tall comes out stretched, and a stretched
  symbol is not the symbol. The cylinder is the one exception, because its barrel is a
  rectangle and a rectangle stays a rectangle.
- **A square stays a square.** The regulator's body is drawn as an upright square of equal
  sides, and the filter and the lubricator as squares turned 45°: four equal sides, diagonals
  meeting at the centre. The proportions are the shape's, not an accident of the box.
- **The line stops at the symbol.** A working line is not drawn through the inside of a
  diamond or a square: what is inside those shapes is their own symbol, and a line across it
  reads as two overlapping drawings.
- **The air reaches the circuit through the conditioning parts**, in the order source, unit,
  valve — never the unit after the valve, because what leaves the unit is the line pressure the
  rest of the circuit is designed around.
- **A two-port part conducts.** A filter, a lubricator, a restrictor or a service unit is part
  of the circuit, so the model has an edge between its ports with the conductance its file
  declares. A part that is drawn but not modelled is a dead end with a symbol on it.

### Actuator glyphs

Each actuation is a glyph drawn outside the body at the declared end, at a size
proportional to the body. The shape is the only thing that tells two actuations apart, so
these are not interchangeable decorations:

| Actuation | Glyph |
|---|---|
| Spring return | A zigzag of four peaks against the body face, drawn in the same ink and weight as the other functional lines |
| Manual, general | A bar across the end of the stem, square ends |
| Lever | The bar of the general actuation, the arm, and the **pivot circle** at its far end |
| Lever with detent | The lever plus the notch that keeps the spool where it is left |
| Push button | The stem ending in a bar with **rounded** ends: the rounding is what makes it a button and not a manual lever |
| Mushroom button | A stem with a **dome** a palm presses |
| Pedal | The lever's foot with the slanting plate a foot stands on |
| Piston (tappet) | A **dome sitting on the body**, with no stem and nothing to grip |
| Roller | A circle on a short arm with its **axle** marked at the centre |
| Roller, one direction | The roller plus the tick that says it only picks the cam up one way |
| Pneumatic pilot | A **box** on the end face with an **open** triangle inside, pointing away from the body |
| Hydraulic pilot | The same box with the triangle **filled**: open for air, filled for oil |
| Solenoid | A box with a diagonal through it, one box per winding |
| Motor | A circle with an `M` drawn as lines, so it scales with the symbol and not with a font |

A pilot without its box cannot be told from an arrow marking a flow, and a lever without
its pivot cannot be told from a plain manual actuation. When a definition file names an
actuation, this table is what the renderer must draw, and a change to a glyph is a change
to the standard's reading of it.

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
  **two ports, one drilled in each end cap**. Both leave their cap **downwards, at the same
  depth**, so the pair reads at a glance as one cylinder driven from both ends. The two ways
  this is not drawn: a port tucked into the corner where an end wall meets the barrel's edge
  reads as part of the outline, and a port laid over the rod on the centreline reads as the
  rod. Neither is used.
- **Every cylinder port is drilled in a cap.** A single-acting cylinder has the one port in
  the cap end and none at the rod end, because that absence is what makes it single-acting;
  a double-acting cylinder has one in each cap. A port anywhere else is a different machine.
- **The numeral of a port on a horizontal edge is written beside its own stub**, on the
  side where no other port is written, in the band between the body edge and the end of the
  stub. Three places are wrong and each is wrong for its own reason: between two neighbouring
  stubs it reads as one port with two names; past the end of the stub it sits where the pipe
  arrives and its halo cuts the pipe in two; over the body it belongs to no port at all.
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

### Instruments

An instrument is drawn **as the thing it is**, on the line or on a stem, and never as a
text label the drawing happens to put beside a component: a label does not connect, is not
checked and reads nothing.

- A **pressure gauge** is a **circle on a stem** that taps the line, with a needle and a
  hub, and the stem ends at the connection. The dial is the symbol.
- A **flow meter** is a **circle the line passes through**: one stub on each side and an
  **arrow inside the circle**, from port 1 to port 2, which is the direction its reading
  counts as positive. The line does not continue through the circle; the circle is the
  instrument.
- A **recorder** is a **frame with graduations on two axes**, with no ports and no stubs:
  it observes one quantity of another component, and the trace inside the frame is the
  model's. The file draws the paper, never a curve: an empty chart that looks like data is
  worse than an empty frame.
- The **value is a reading, not a primitive**: it is drawn from the model beside or inside
  the symbol, in the unit the definition declares, so one shape serves a 0–10 bar gauge, a
  0–100 l/min meter and a 60 s chart, and no file stores a number the simulation owns.
- The requirement for these two shapes is written here from ISO 1219-1's function-first
  reading of a measuring instrument; no published figure was traced. Each instrument's
  review stays `pending` until a second reader checks it
  ([review](../assets/symbols/review/)).

### Terminations

- A **blind plug** is a **stub that ends in the blocking bar**, the same mark a valve draws
  for a closed way: a reader learns one shape for "closed" and finds it everywhere. It has
  one port, on the side the hose arrives, and no stub on the far side because nothing
  leaves.
- A **silencer** is a **cone that widens away from its port**, with two relief lines across
  its mouth: the air spreads as it leaves, and the relief lines are what tell it apart from
  a plain nozzle. It sits on an **exhaust** port and nowhere else. Its body is a solid
  working line like any component's; the class describes lines, not the things fitted to
  them.
- **Neither symbol claims a property the model does not have.** The plug is the closed
  port, not a valve; the silencer's file states a conductance and the review states that
  the noise is not modelled.

### The electrical control circuit

The control circuit is a second domain in the same document, and its symbols follow the
general convention for electrical diagrams; the geometry is ours, hand-drawn, and no figure
from another program or from the standard was traced.

- A **supply** is the pair of plates of a d.c. source: the **long line positive**, the short
  one negative, with a stem to each terminal and the polarity drawn as lines, never typed.
- A **contact** is two terminals with a **visible gap**, the movable bar and the push rod
  with its cap. The bar is drawn by the renderer: above the gap when the contact is open,
  across it when the circuit is closed, so the sheet shows the state the run reports. A
  normally closed contact is another file, not another drawing.
- A **solenoid valve** is the familiar valve body with the windings at the end opposite the
  spring and a **coil terminal** beside them; the terminal is where the control circuit
  arrives, and no valve position opens or closes it.
- **A wire is dotted.** The electrical class says what the line is, so a hose between two
  electrical terminals is drawn dotted without anyone choosing a class, and the class is
  what the reader sees before any label.
- **The body of a symbol is solid**; only the wires carry the class. A symbol drawn entirely
  in dots is a symbol that looks like a wire.

### Circuit diagrams (Part 2/3)

- Components are grouped and numbered, and the diagram carries an identification
  block: title, project, date, sheet, revision.
- Lines are orthogonal; a line never passes behind a symbol; a connection is drawn
  between ports, never between arbitrary points on the body.
- **A hose turns as few times as its two ports require.** Leaving a side and entering a top
  is one bend; two ports on parallel edges keep the Z, which is the shortest orthogonal
  route between them and the one a user can drag. The old router always drew the Z, and a
  connection that did not need a stair read like one.
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
6. **Every port carries the small open circle** of a connection, drawn by the renderer and
   filled with the paper colour so the last millimetre of a hose goes behind it. A
   definition cannot forget it, and a beginner always sees where a hose attaches.

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
