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

### What has been read

The rules below were read in a copy of **ISO 1219-1:2006** and confirmed against
**JIS B 0125-1:2020**, the Japanese adoption prepared on ISO 1219-1:2012 with Amendment
1:2016. The registration numbers (`X11400`, `RF052`…) name a symbol or a rule across
editions; a clause number is the one of the edition read, and a review records which
edition it read.

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

## The symbols are not ours

The symbols and the circuit conventions are the standard's: **one form, the same in every
application.** This project implements them; it does not own them and may not adapt them.

- The authority is ISO 1219-1/-2/-3 (and the national adoption named above) and, in our own
  words, this contract. A reference application, a screenshot, a capture or a manual can
  only say **which symbol exists**; it never supplies a proportion, a shape, an added mark
  or a correction.
- **No transformation of a symbol of ours**: no simplified variant, no added mark, no
  orientation or proportion chosen to match another program or a capture. A drawing that
  differs from the standard is a defect of the drawing, not a variant of the symbol. The
  library panel and the sheet are two views: **the panel shows the symbol itself, without
  the diagram's connection points; the sheet draws them because that is where a hose
  connects**, and those marks are not a symbol variant. Neither view is changed to imitate
  the other.
- The standard is never modified to fit the application, a capture or a previous drawing. A
  correction changes our definition, the renderer and the review record, with the clause and
  the measures.

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
- **The sides and offsets are the figure's.** In a 3/2 or 5/2, the working ports (`2` and
  `4`) sit on the top edge, the supply (`1`) and the exhausts (`3` and `5`) on the bottom.
  In a 3/2 the supply and its working port share the same offset inside the square, so the
  actuated path is straight and the rest path is the diagonal to the exhaust at the other
  quarter (`X10280`, `X10270`). In a 5/2 the two working ports sit at the outer quarters,
  the supply in the middle, and each exhaust directly under its own working port, so each
  position reads as one straight supply path and one straight exhaust path (`X10400`). A
  port order that is not the figure's is a different symbol.
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
  drain lines dashed and distinct, electrical connections dotted, and an assembly boundary a
  **chain line** (a long dash, a gap, a dot and a gap). The renderer draws each class with
  its own pattern; a definition declares which class a line belongs to instead of picking a
  random dash pattern.

### Ports and their designations

- **Every port of a symbol is drawn** (`RF052`): a machine shows its intake and its delivery,
  a valve shows its ways, and a part shows its terminals. A connection the symbol omits is a
  connection the reader cannot make.
- **The numeral says what the port is**: `1` is the inlet, `2` and `4` the outlets, `3` and
  `5` the exhausts, `12` and `14` the pilot ports (`RF053`, which refers the pneumatic
  numbering to ISO 11727 and the hydraulic letters — `P`, `T`, `A`, `B`, `X`, `Y` — to
  ISO 9461).
- **The designation has its own room**, above or to the left of the port (`RF053`). The
  renderer places it beside its stub on the clear side; whether the standard asks for that
  position or only for the room is an open question of this contract, recorded here until
  the owner settles it.
- **A designation outside a frame sits in the margin, beside its port.** When the ports are
  drawn outside the unit's chain-line frame (the service unit, after the owner's 2026-11-24
  decision), the numeral goes outside the frame too: `1` above the left stub and `2` below
  the right one, each inside the symbol box and clear of the frame, the stubs and the drawn
  parts. A numeral back inside the frame is the defect the owner reported on the reference
  capture.

### The air line

The source and the parts that condition the air are a set, and a reader recognises them by
shape alone. Every one of them is a **square turned 45°** or an upright square, drawn from the
requirement and never copied from another program, and every connection point is the small open circle of the
reference set so that a hose is visibly meant to arrive there.

| Symbol | Shape, and what carries the meaning |
|---|---|
| Compressed-air **network** | The bare **outline triangle** on a stem, **apex up at the connection**: the air arrives from the plant, the machine that makes it is drawn elsewhere, and the point is where the hose attaches. A triangle standing on its base would read as a source delivering downwards, away from its own line |
| **Compressor** | The same triangle inside a **circle**, apex up, the point touching the delivery where the stem leaves the top; the **shaft bar** at the drive side, a thick bar at mid-height, because a machine shows its drive; the delivery above (port 2) and the intake below (port 1) |
| **Adjustable compressor** | The compressor plus the **adjustability arrow** on the diameter, at 45°, with a filled head at the upper end; the standard allows a longer arrow |
| **Filter** | A square turned 45° with the flow crossing it from vertex to vertex; the line stops at each vertex and is not drawn through the inside |
| **Filter with manual drain** | The same diamond, plus the element as the **dashed line on the vertical diagonal**, a **solid chord across the lower part** (the bowl the condensate collects in) and the **drain stem** leaving the bottom vertex. The element stops at the chord because the bowl owns the space below it. Without the chord and the stem it is a filter, not a separator with a drain |
| **Oil lubricator** | The same diamond, a **plain line from the top vertex toward the centre** and **no dashed element**: nothing is filtered here, oil is added. The line has no arrowhead, because a head would read as a flow direction; the absent element and the line are the difference |
| **Service unit, full** | A **chain-line frame** around the functions, in the order the air meets them: the filter diamond with its element down to the bowl chord, the chord and the drain, the regulator as an **upright square with the flow through it and the two facing arrows**, its **setting spring on top** with the **diagonal arrow** that says the setting is adjustable, its **vent** as an open triangle on the left and its **sensing line** from the outlet back to its underside, the **dial** as a gauge on a stem that taps the line after the regulator, and the **lubricator diamond** at the right. The ports sit **outside the frame** on the working line, and their numerals outside it, beside each port |
| **Service unit, simplified** | One wide body with the **dashed element line at the left**, the **dial** inside it, the **purge arrow in the bottom band** and the **short mark on the top edge at the right**. This compact form carries the automatic drain by the owner's reference; the generic simplified figure of `X12160` does not print it, and the review says so. A plain rectangle is not this symbol |
| **Filter regulator** | The filter with manual drain and the pressure-relieving regulator in the same **chain-line frame**, with the regulator's vent, setting spring, diagonal arrow, vent and dashed feedback, and the gauge on its stem after it; **no lubricator**. `X12150` prints the regulator without the gauge and `X12160` prints the gauge; the file composes the two and the review says so |
| **Filter with automatic drain** | The same diamond as the manual-drain filter, with the element down to the **bowl chord**, the chord and the **drain stem**; the drain empties the bowl by itself. The owner's reference draws the same body, without a separate manual actuation; the plan records the automatic-drain mark as `X12200`/`F065V1`, and the review states that the figure itself was not re-read in this pass |

Rules that hold for all of them:

- **The box is the file's, and it follows the function.** The drawing is laid out inside a
  symbol box declared by the definition (`box = [width, height]`), and the default is the
  **20 mm square**. A square box is the rule for a valve, a filter or a supply; a cylinder
  declares **44 × 20 mm**, because its barrel is a rectangle, and the two service units
  declare **46 × 25 mm** and **40 × 20 mm**, the wide proportions of `X12160` that a square
  box compressed. A cylinder's barrel fills 18.48 × 8.40 mm of its 44 × 20 mm box (2.2 : 1;
  the figure's barrel is 2.23 : 1) and the rest of the box is the rod's travel. The box is a
  project size and not a fraction of the sheet: A3 does not make a valve bigger and a crowded
  A4 does not make it smaller. The worked example composes with
  `min(0.26 · height, 0.17 · width, 20 mm)`, so a thumbnail shrinks below the rule and a full
  sheet stops at it ([`schematic::layout`](../crates/punchi-fluid-draw/src/schematic.rs)).
  A box that is not the declared one stretches the drawing: a 3/2 valve placed in a 1.2 : 1
  box came out wide, which is a drawing that is not the symbol. The templates and the
  worked example are checked for the declared box, and the capture
  `assets/screenshots/symbol-size-light.png` records the 20 mm grid, where a valve box is
  two cells across. The service unit's band used to be compressed into the square; the
  review now records the 46 × 25 mm measures instead of claiming the proportions are the
  figure's.
- **A square stays a square.** The regulator's body is drawn as an upright square of equal
  sides, and the filter and the lubricator as squares turned 45°: four equal sides, diagonals
  meeting at the centre. The proportions are the shape's, not an accident of the box.
- **The line stops at the symbol.** A working line is not drawn through the inside of a
  diamond: what is inside it is the element or the oil mark, and a line across it reads as two
  overlapping drawings. A square is different only when it is a valve body and the line is its
  flow path, as the regulator's square is; a line through any other closed shape is a defect.
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
| Spring return | A zigzag against the body face, its ends on the actuation axis, drawn in the same ink and weight as the other functional lines (`2002V1`; the standard fixes the form, not a number of turns) |
| Manual, general | A bar across the end of the stem, square ends |
| Lever | The **pivot circle at the body end**, the arm leaving it at an angle, and the bar of the general actuation as the grip at the far end (`688V1`; the roller lever `X10270` puts its roller where a plain lever puts the grip) |
| Lever with detent | The lever plus the V notch that keeps the spool where it is left (`655V1`, and the manual override of `X10290`) |
| Push button | The stem ending in a **bracket**: a bar across the end with two short prongs back toward the body (`682V1`, and the 2/2 valve `X10210`) |
| Mushroom button | A stem with a **dome** a palm presses |
| Pedal | The lever's pivot circle and arm, with the slanting plate a foot stands on at the far end (`689V1`) |
| Piston (tappet) | A **dome sitting on the body**, with no stem and nothing to grip |
| Roller | A circle on a short arm with its **axle** marked at the centre (`712V1`) |
| Roller, one direction | The roller plus the tick that says it only picks the cam up one way (`X10060`) |
| Pneumatic pilot | A **box** on the end face with an **open** triangle inside, its apex **at the body**: the pressure pushes the valving element (`F044V1`, and the complete valves `X10410`, `X10460`) |
| Hydraulic pilot | The same box with the triangle **filled**: open for air, filled for oil (`F043V1`) |
| Solenoid | A box with a diagonal through it, one box per winding (`212V1`) |
| Motor | A circle with an `M` drawn as lines, so it scales with the symbol and not with a font (`F019V1`) |

A pilot without its box cannot be told from an arrow marking a flow, a lever without
its pivot from a plain manual actuation, and a push button without its bracket from a bar.
When a definition file names an actuation, this table is what the renderer must draw, and a
change to a glyph is a change to the standard's reading of it.

### What is adjustable

An adjustable setting is shown by an **arrow**, and where it goes is not a matter of taste:

- on a **machine** (a pump, a compressor), the arrow sits on the **diameter** of the circle
  and may be longer (`RF040`); it carries a filled head;
- on an **adjustable element** (the regulator's spring, a throttle), the arrow goes
  **through the element** (`RF048`), which is the arrow the service unit's spring carries.

An arrow that does not cross the element it adjusts reads as a flow direction.

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
  length left against the rod-side cap's inner face. That residual space is where the return
  spring is compressed, and it is not a fraction chosen by the renderer: a cylinder that only
  used part of its chamber would read as a cylinder that had not finished its stroke.
- The piston **meets both walls of the bore** at every position of the stroke. A line that
  stops short of the walls reads as a bar floating inside the barrel; the piston is the
  seal between the two chambers and is drawn as such.
- A double-acting cylinder shows the same barrel, piston and rod and **no spring**, with
  **two ports, one drilled in each end cap**. Both leave their cap **downwards, at the same
  depth** — in the 44 × 20 mm box each sits 0.92 mm from its end wall and its stub runs
  5.8 mm below the barrel — and inside the cap, not in the chamber the piston crosses, so
  the pair reads at a glance as one cylinder driven from both ends. The two ways this is
  not drawn: a port drawn on the corner itself merges with the outline, and a port laid
  over the rod on the centreline reads as the rod. Neither is used.
- **Every cylinder port is drilled in a cap.** The cap is the end region between the end
  wall and the cap's inner face, which in the 44 × 20 mm box is 2.03 mm deep (a tenth of
  the barrel's length, as the figure draws it); the chamber the piston crosses starts at
  that face. A port drilled inside the chamber is swept by the piston: a single-acting
  cylinder would pressurise the spring chamber at rest and a double-acting one would lose
  the connection when its piston passed over it. A single-acting cylinder has its **working**
  port in the cap end and none at the rod end, because that absence of a working line is
  what makes it single-acting; a double-acting cylinder has one working port in each cap. A
  port anywhere else is a different machine.
- **The figure's spring-chamber connection is a separate matter.** `X11440` is registered as
  a single-acting cylinder whose spring chamber carries a connection, and prints a second
  stub in that chamber, at the rod end. The catalog's single-acting cylinder declares the
  working port only: the model gives it one working chamber, and the schema has no breather
  role, so the second connection is recorded against `RF052` in its review instead of being
  forced into a `working` or `exhaust` role that would change what the part is.
- **The cap's inner face is an inset, not a second rectangle.** The figure draws it as a
  line at the cap end; our barrel is the one rectangle the file declares and the cap is the
  renderer's 11 % inset, so the line is not drawn separately. The port is inside the cap,
  which is what the function needs; the missing line is an open item of the cylinder
  reviews, not a variant of the symbol.
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

### Flat pressure elements (OR and AND)

A shuttle and a two-pressure valve are **flat symbols**: they are a pressure function, not
a spool with positions, so they declare no `body` and the square-per-position rule does not
apply (ADR 0039). Both were read in `X10740` (shuttle, OR) and `X10620` (two-pressure,
AND) — ISO 1219-1:2006 at 6.1.5.5/7.1.5.5 and 7.1.3.6, confirmed against JIS B 0125-1:2020
at 6.1.5.5/7.1.5.5 and **7.1.3.5** (the clause number differs between the editions; the
registration number does not).

- **The shuttle (`X10740`) is one rectangle with two opposed check seats and one ball
  between them**, the junction dot on the through path and the outlet branching from it at
  the centre. Both inlets are on opposite sides at mid-height; the ball is pushed off the
  inlet that carries the higher pressure and onto the other, so the outlet follows the
  higher one. The figure's body is **8M × 4M** and the ball **1M** across (M = 2 mm, so
  16 × 8 mm and a 1 mm radius). The ball's three positions are the live state of
  ADR 0041; the seats and the dot are the figure's static form.
- **The two-pressure valve (`X10620`) is one rectangle with two baffles hanging from the
  top edge either side of the outlet and a free double piston between the inlets.** The
  rod is drawn as two parallel lines between the two faces. The figure draws **no through
  line inside the body**: the inlets stop at the body edge. Its body is **8M × 4M**, the
  baffles **1M** long at **2M** either side of the centre, the piston faces **2M** tall at
  **3M** either side, and the rod **0.5M** thick.
- **Both inlets print `1` and keep distinct ids** (`1a`, `1b`), and the outlet prints `2`
  (`RF053`; ADR 0039). The figure is a symbol and prints no numerals: the designations are
  the circuit convention's, so the drawing adds them and the ids stay what a pipe and the
  solver address.
- Neither symbol carries a spring or an actuator, and neither has a flow arrow: the check
  seats, the junction dot and the double piston are what state the function.

### Terminations

- A **blind plug** is a **stub that ends in the blocking bar**, the same mark a valve draws
  for a closed way: a reader learns one shape for "closed" and finds it everywhere. The mark
  is the standard's **`2172V1` (8.2.9), closed path or port**: a stub 1M long closed by a bar
  1M across. It has one port, on the side the hose arrives, and no stub on the far side
  because nothing leaves. The standard's other plug figure, **`F038V1` (8.2.10), plug inside
  a fluid line**, is a square plug drawn *in* the line; a closure fitted on the port itself
  uses `2172V1`'s bar, and the review records that these are two different figures instead
  of mixing them.
- A **silencer** is, in this contract, a **cone that widens away from its port**, with two
  relief lines across the widening cone: the air spreads as it leaves, and the relief lines
  are what tell it apart from a plain nozzle. **The standard does not draw that form**:
  `2033V1` (8.7.37) draws a rectangle **4M × 2M** with **three** relief lines at 1M spacing,
  two from one edge and one from the other. The difference is recorded in the review and
  kept as an open item for the owner; the contract's cone is not presented as the figure's
  geometry. The silencer sits on an **exhaust** port and nowhere else. Its body is a solid
  working line like any component's; the class describes lines, not the things fitted to
  them.
- **Neither symbol claims a property the model does not have.** The plug is the closed
  port, not a valve; the silencer's file states a conductance and the review states that
  the noise is not modelled.
- **A closure is drawn at the port it belongs to**, outside the host body and along the
  port's outward edge, with the closure definition's own hand-drawn symbol. The closure is
  a small accessory box (10 mm) whose own port faces the host, so the plug's stem (ending in
  the bar) or the silencer's cone (widening to its mouth) starts at the connection and grows
  away from the body; it grows the
  drawn footprint but is not a separate selectable item and has no selection ring of its
  own. The closure's **own port numeral is suppressed** — the plug or the cone must not
  read as a labelled component — while the **host port's numeral stays visible** and is
  kept clear of the closure box. The closure is drawn after the body, on top of the port
  stub, never underneath it.
- **Manual review, once per closure-at-a-port change.** The symbol lint does not inspect
  this composite: the reviewer checks in light and dark, and on the 300 dpi page, that the
  closure sits outside the body on the port's edge, that the host numeral is legible and
  not covered, and that the closure's own `1` is absent. The captures are the ones named
  in the P-021a evidence; the two source symbols keep their own review records.

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

### Pipe widths (the tube)

A pneumatic connection carries a nominal outside diameter, and the drawing shows it as the
pipe's **stroke**. The width is a **series**, not a continuous map, so two tube sizes that a
reader must tell apart differ by a visible step at 300 dpi, and the ISO 5457 frame stays a
different class of line:

| Tube | Width |
|---|---|
| unspecified (an older sheet) | 0.42 mm |
| 4–5 mm | 0.25 mm |
| 6–8 mm | 0.35 mm |
| 10 mm and more | 0.50 mm |
| an electrical wire | the class stroke, unchanged |

The largest pipe width is **0.50 mm**, kept below the **0.7 mm** frame on purpose: a pipe that
reaches the frame's weight competes with the sheet's own border. The series is the project's
nominal practice list; ISO 1219 does not define tube outside diameters ([the series and the
parts rows](reference/tube.md)). The **class pattern is unchanged** — working solid, pilot
dashed, drain dashed — and the width changes only the stroke. The port stubs belong to the
symbol and do not thicken; the selection ring keeps the base `LINE` weight. An electrical
wire is not a tube: the renderer checks the electrical case before the width, so the wire
keeps its dotted class stroke and never takes a pipe width. The electrical connection itself
is deferred (P-026, the owner's node register).

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
7. **A closure fitted to a port belongs to that port.** It is drawn outside the host body,
   on the port's outward edge, after the body and its port stub so it lies on top of the
   stub; the host port numeral is kept and stays clear of the closure, the closure's own
   numeral is suppressed, and the closure draws no selection ring of its own. The closure
   is part of the host's drawing, not a placement with its own box in the document.

## Conformance review

Automated (`symbol lint`, part of the catalog validation, milestone M1.5):

- number of squares equals the number of positions;
- every position has at least one path or at least one blocking mark;
- every port has a numeral, unique, and consistent with its role;
- `spring_return = true` implies a spring symbol is declared;
- actuator declared for `actuation = toggle` or `momentary`;
- no primitive outside the `0..=1` box, no zero-length line, no zero radius;
- line class declared for every polyline;
- a declared envelope lies inside `0..=1`, contains the body, every port and the frame
  primitive; a missing envelope is allowed, a malformed one is not;
- a port numeral is placed clear of the resolved outer boundary — the envelope, else the
  body, else the symbol box — inside or outside but never on it, and clear of a primitive
  line; the draw test proves `preparation.service_unit` today and P-018b extends it to the
  rest of the inventory;
- minimum legibility: stroke and gaps above the floor at the smallest palette size.

Manual, once per symbol, signed off in the pull request:

- the five checks above plus a side-by-side against the Part 1 symbol for that
  function, done by reading the standard — not by comparing with another program;
- every port the symbol has is drawn (`RF052`) and carries the numeral `RF053` gives it
  (`1` inlet, `2`/`4` outlets, `3`/`5` exhausts, `12`/`14` pilots);
- a machine shows its drive bar on the standard's side (`RF038`), and an adjustable one its
  arrow on the diameter (`RF040`) or through the element (`RF048`);
- the record names the edition read and the registration number of the figure;
- every port designation sits clear of the outer boundary, the body and the primitive
  lines, inside or outside but never on a line, in both themes;
- the symbol is legible at 24 px (palette) and at 300 dpi (PDF export);
- a screenshot of the symbol in light and dark themes is attached to the PR.

Manual, once per pipe-width or line-class change — the `symbol lint` does not inspect
connection strokes, so this list is the control:

- a 4, 6, 10 and 16 mm pipe draws at 0.25, 0.35 and 0.50 mm (the first two sizes share one
  step when they fall in the same band), measured on the 300 dpi page within one device
  pixel;
- the widest pipe stays visibly below the 0.7 mm frame on the same page;
- the working, pilot and drain patterns survive every width;
- an electrical wire keeps its dotted class stroke and carries no tube width;
- the port stubs keep the symbol's own weight and the selection ring keeps `LINE`;
- the light and dark captures of the 4/10/16 mm sheet are attached to the PR.

## Correcting what is wrong comes first

**Owner's decision, 2026-11-15.** The next pass over the symbols is a **correction**, not an
extension: pneumatic geometry goes first, and no other library is added until what exists is
conformant. The order and the stop are recorded in the owner's working notes. A symbol is
not corrected by
comparing it with a drawing from another program: the requirement is read, written in words,
drawn by hand, and checked with the lint and the manual list.

## Current state, honestly

The structural half of the contract is **implemented** and the visual half is **reviewed at
palette and canvas size**. Every built-in has its 300 dpi capture; the **core's** records
were signed by the second reader on 2026-11-24, and the symbols outside the core still wait
for that signature. The 2026-11-24 correction of the two service units changed their
drawing, so **their two reviews are `pending-300dpi` again** until the owner signs the new
captures: the core is not complete while they are open, and the export keeps its unverified
note. The **filter-regulator** and the **filter with automatic drain** added on
2026-11-24 are new conditioning forms and are pending as well. The signatures record the ISO 1219-1:2006 reading confirmed against JIS B 0125-1:2020;
the Chilean adoption (**NCh-ISO 1219**) is still unconfirmed from this machine, as the
national-adoptions section says.

| Requirement | State | Evidence |
|---|---|---|
| One equal square per position, impossible to make ragged | **enforced** | the renderer splits the declared `body`; the lint checks the width |
| Blocking mark on every closed port of a position | **enforced** | drawn from `blocked`, never authored |
| Port numerals from the `port` table | **enforced** | `parts::draw_port_numeral`; the file has no shape to disagree with |
| Port designation per `RF053` | **partial** | the numerals come from the port table; whether the designation belongs above/left of the port is open (see “Ports and their designations”) |
| A port designation clear of the symbol | **enforced for the two service units; the general rule is in the drawing** | `the_service_unit_numerals_stay_clear_of_the_frame_and_the_drawing` and `the_simplified_service_unit_numerals_sit_outside_its_body`; P-018b extends it to the rest |
| Flow path arrows along the connection | **enforced** | the path is routed from `connection`, the arrow points at its far end |
| Spring and actuator as declared parts | **enforced** | `[spring]` / `[actuator]`, required by the lint |
| Line classes with their own dash pattern | **enforced** | `line_class` on every line and polyline; the boundary class is the chain line of `a_chain_line_draws_a_dash_a_gap_a_dot_and_a_gap`, and its four SVG values are in `a_chain_line_writes_its_four_dash_values` |
| Pipe stroke from the tube size, capped below the frame | **enforced, reviewed by hand** | `tube_width_mm` and `the_line_width_is_the_series_width_below_the_frame`; the manual checklist and the 4/10/16 mm captures attached to the pull request |
| Flow direction shown on a line that carries air | **enforced** | solid arrowhead on the line, straight from the sign of the flow |
| Junction dots at three-way junctions | **drawn** | a tee is a component and its filled dot is its symbol ([ADR 0006](adr/0006-a-junction-is-a-component.md)); the solver treats its ports as one point |
| Title block on the sheet itself | **drawn** | the content block is on the canvas and in the export ([capabilities](capabilities.md), [ADR 0030](adr/0030-the-block-protects-its-own-box.md)) |
| Legible at palette size (24–30 px) | **reviewed** | screenshots in `assets/symbols/review/` |
| Legible at 300 dpi | **done for the core except the two service units, pending their new captures; open outside it** | the core's 300 dpi sheets are in the review folder, signed 2026-11-24; the two service units were redrawn on 2026-11-24 and await the owner's signature |
| Second reader's signature | **the core was signed; the two service units are pending again; the rest is open** | Cristian Muñoz (owner), second reader, signed the core on 2026-11-24; the 2026-11-24 correction returns `preparation.service_unit` and `preparation.service_unit_simple` to `pending-300dpi` |

Each built-in has a review file in [assets/symbols/review/](../assets/symbols/review/).
The core's files read `status: complete`, signed by the second reader; the files outside the
core keep `status: pending-300dpi`. The reviews are compiled into the binary
(`punchi-fluid-catalog::reviews`), and the PDF export asks them whether it may drop the
note “symbols not yet verified against ISO 1219-1” from its title block: it may not while
any review is not `complete`, which is why the note stays while a non-core review is open.
A test asserts the compiled reviews cover every built-in, another asserts the exported file
carries the note while any review is open, and the catalog's review logic has a case for the
one condition that drops it — every review `complete`.

The three captures of every built-in are regenerated with
`cargo run -p punchi-fluid-export --bin symbol-sheet -- --all assets/symbols/review`, which
writes `<id>-300.png`, `<id>-light.png` and `<id>-dark.png` from the same drawing and the
same options the per-symbol command documents; the per-symbol form stays for one review at a
time.

The PDF is vector, so the drawing is exact at any print resolution. The 300 dpi raster that
serves as evidence exists for every built-in; for the core it is reviewed and signed, for
the rest it is captured but not yet signed.

One symbol is weaker than the others and says so in its own review: the fixed
restrictor's throat was drawn without a copy of the standard at hand. A
conformant-looking document that is not conformant is the one outcome this project
cannot afford, so the file states it instead of hiding it.
