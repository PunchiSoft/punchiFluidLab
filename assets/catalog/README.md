# Component catalog

Every symbol the simulator knows — valves, cylinders, supplies, restrictors — is one
TOML file in a catalog folder. Adding a component is adding a file. Changing its
behaviour or its drawing is editing that file. Removing it is deleting the file.

Nothing in the Rust code names a specific valve: it reads the files, validates them
and simulates whatever it finds.

## Where the files live

| Priority | Folder | Origin |
|---|---|---|
| Low | `assets/catalog/` (compiled into the binary with `include_str!`) | `built-in` |
| High | `$XDG_DATA_HOME/punchi-fluid-lab/catalog/`, i.e. `~/.local/share/punchi-fluid-lab/catalog/` | `user` |

The application loads the built-in set first and then every `.toml` file in the user
folder, in file-name order. A user file with the same `id` **replaces** the built-in
one; that is the intended way to tune a component without forking the project.

`PUNCHI_FLUID_CATALOG_DIR` points the application at a different user folder, which
is handy while editing.

The **Catalog** panel of the window lists every loaded definition with its origin,
shows the folder in use, offers a *Reload* button and prints the reason for every
file it had to skip.

## Adding a component

1. Copy the closest existing file in this folder to
   `~/.local/share/punchi-fluid-lab/catalog/`.
2. Change its `id`, `name` and numbers.
3. Press *Reload* in the window, or restart the application.

A file that does not parse, or that describes something impossible, is skipped: the
rest of the catalog keeps working and the reason appears in the panel. It never
stops the application from starting.

## Removing a component

- Your own file: delete it.
- A built-in definition: create a file in the user folder with the same `id` and
  `disabled = true`. It disappears from the catalog and from the palette.

## File format, schema 3

Schema 2 replaced hand-drawn valve boxes with declared parts: a body split into equal
squares, a blocking list per position, a declared spring and actuator, and a line class
on every line. Schema 3 added the declared `[return]` part and the whole set of actuation
kinds. This build reads schema 2 and 3: a file that still says 2 loads, because `[spring]`
is the short form of `[return] kind = "spring"`. Schema 1 and anything newer than 3 are
rejected with the version in the message.

```toml
schema = 3                          # required, must be 2 or 3
id = "valve.3_2.spring_return"      # required, lowercase letters, digits, . _ -
name = "3/2 valve, spring return"   # required, shown in the interface
name_es = "Válvula 3/2, retorno por muelle"   # optional Spanish name
category = "valve"                  # supply | valve | actuator | throttle | accessory | sensor
summary = "One line for the palette"          # optional
disabled = false                    # optional, hides this id from the catalog
kinematics = "linear_actuator"      # optional: none (default) | linear_actuator
body = [0.06, 0.28, 0.88, 0.44]     # valves: x, y, width, height of the body
```

### Declared parts

A valve body is not a list of rectangles. `body` is split by the renderer into **one
equal square per position**, counted from the spring end — the square against the spring
is the rest position — so two squares can never come out unequal or misaligned. The spring and the actuator are declared the
same way: what and where, never how.

```toml
[return]                 # required when spring_return = true
kind = "spring"          # spring | pneumatic | hydraulic | detent
side = "right"           # left | right | top | bottom; away from the actuator

[actuator]               # required when actuation is toggle or momentary
kind = "lever"           # see the actuation list below
side = "left"            # the end opposite the return, as the standard draws it
```

`[spring]` with a single `side` still works and means `[return] kind = "spring"`: it is
the spelling every file written before schema 3 uses. Declaring both is an error, because
it is one part with two meanings.

`kind = "detent"` is the absence of a return: the spool stays where it is left, so a file
that says `spring_return = true` cannot declare it.

### Actuation kinds

Each one is a function, not a drawing: the renderer owns the geometry, and a valve pushed
by a roller is a new file with `kind = "roller"`, never a new branch in the code.

| `kind` | Spanish name | Group |
|---|---|---|
| `lever` | Palanca | manual |
| `lever_detent` | Palanca con enclavamiento | manual |
| `push_button` | Botón | manual |
| `mushroom_button` | Botón tipo hongo | manual |
| `pedal` | Pedal | manual |
| `plunger` | Émbolo | mechanical |
| `roller` | Rodillo | mechanical |
| `roller_one_way` | Rodillo escamoteable | mechanical |
| `pilot` | Pilotaje neumático | by fluid pressure |
| `pilot_hydraulic` | Pilotaje hidráulico | by fluid pressure |
| `solenoid` | Electroimán, una bobina | electrical |
| `solenoid_double` | Electroimán, dos bobinas | electrical |
| `solenoid_servo` | Electroimán con servopilotaje | electrical |
| `motor` | Motor eléctrico | electrical |

The reference sheet in the application (`View → Symbol sheet`) draws all of them, and the
four returns, with the same renderer as the valves, so what the sheet shows is what a
file gets.

The renderer draws the stubs from the port positions to the body edge, the port
numerals outside, the flow path of every open connection with its arrow, and the
blocking mark of every port the active position closes. None of that is authored, so
none of it can disagree with the simulation.

### Ports

One `[[port]]` table per connection point. `role` is what the simulation uses:
`supply`, `working`, `exhaust` or `mechanical`. `position` is inside the symbol box,
`0..=1`, and it is where the canvas attaches a pipe.

```toml
[[port]]
id = "1"
role = "supply"
position = [0.32, 1.0]
```

### Behaviour

Exactly one `[behaviour]` table, tagged with `type`.

```toml
[behaviour]
type = "supply"
pressure_bar = 6.0
```

```toml
[behaviour]
type = "throttle"
conductance = 5.0e-10        # sonic conductance C, m³ / (s · Pa)
```

```toml
[behaviour]
type = "gauge"
min_bar = 0.0                # lowest pressure on the dial, gauge bar
max_bar = 10.0               # highest pressure on the dial, gauge bar
```

```toml
[behaviour]
type = "flow_meter"
conductance = 2.0e-9         # restriction the meter adds, m³ / (s · Pa)
nominal_l_min = 100.0        # full scale of the reading, normal litres per minute
```

The meter sits **in** the line: two working ports, one arriving and one leaving. The
solver models it as the restriction its `conductance` declares and keeps the gas that
went through it on the last substep, so the reading is a measurement of this circuit and
not of the definition. Positive means gas from port 1 to port 2. The number is drawn in
normal litres per minute at 0 °C and 1 013.25 hPa — the normal litre ISO 6358 uses for the
conductance, so the number and the restriction share one reference; the file declares the
scale, never the number. See
[ADR 0014](../docs/adr/0014-a-flow-meter-is-a-restriction-that-reports.md).

```toml
[behaviour]
type = "cylinder"
bore_mm = 20.0
rod_mm = 0.0                 # rod diameter; 0 when the return chamber has the full bore
stroke_mm = 100.0
dead_length_mm = 5.0         # residual volume expressed as a length
moving_mass_kg = 0.5
spring_rate = 250.0          # N / m
spring_preload = 5.0         # N
damping = 2.0                # N · s / m
friction = 1.5               # N
```

```toml
[behaviour]
type = "valve"
actuation = "toggle"         # none | toggle | momentary
spring_return = true         # releasing returns to default_position
default_position = 0

[[behaviour.positions]]
name = "rest"                # identifier and English name
name_es = "reposo"           # optional; the interface speaks Spanish and shows this

[[behaviour.positions.connection]]   # one open path per position
from = "2"
to = "3"
conductance = 1.0e-9

[[behaviour.positions]]
name = "actuated"

[[behaviour.positions.connection]]
from = "1"
to = "2"
conductance = 1.0e-9
```

```toml
[behaviour]
type = "recorder"
span_s = 60.0                # window of the chart, 1 to 600 seconds
rate_hz = 10.0               # samples per second, 1 to 50 (product bounded)
```

A recorder is the one component with **no ports**: it observes a quantity of another
component instead of joining the circuit, and the placement stores which one
(`source` in the sheet format). Its window and rate bound the samples the session keeps;
the curve is the model's, never a primitive of the file
([ADR 0015](../docs/adr/0015-a-recorder-samples-a-named-quantity.md)).

```toml
[behaviour]
type = "junction"            # no fields: it adds nothing to the network
```

A junction is where hoses meet. It has no volume, no restriction and no force, so the
solver treats all of its ports as one point — which is why a junction file puts them on
top of each other. Three ports on one point is the honest model of a node, and the
filled dot of ISO 1219-1 is its whole symbol; the reason it is a component at all is
[ADR 0006](../docs/adr/0006-a-junction-is-a-component.md). The circuit check does not
report its ports as open connections, because an unused port of a node is an artefact
of modelling a node as a component. A part that shares air between ports *and holds a
volume* — a manifold block — is not a junction: that one needs its own behaviour.

A connection is classified by the **roles** of its two ports, not by their numbers:
`supply`↔`working` pressurises the working line, `working`↔`exhaust` vents it. Any
other pair is rejected when the circuit is built, with the port pair named in the
error. That is what makes a 5/2 valve a file and not a code change.

### Symbol

Drawing primitives in the same `0..=1` box as the ports:

```toml
[[symbol]]
kind = "line"                # line | polyline | polygon | rect | circle | arc
points = [0.20, 0.0, 0.20, 0.22]
line_class = "working"       # required on line and polyline
filled = false               # optional, fills polygon / rect / circle
dashed = false               # optional, for a part that is dashed by nature
stroke = 1.0                 # optional width multiplier, floor 0.5
```

`line_class` is `working`, `pilot`, `drain` or `electrical`: the renderer picks the
dash pattern, so a pilot line can never be drawn as a working line by accident.

`points` is a flat list, interpreted by `kind`:

| `kind` | Values |
|---|---|
| `line` | `x1, y1, x2, y2` |
| `polyline` | `x1, y1, x2, y2, …` (at least two points) |
| `polygon` | same as `polyline`, at least three points, closed |
| `rect` | `x, y, width, height` |
| `circle` | `cx, cy, radius` |
| `arc` | `cx, cy, radius, start_degrees, end_degrees` |

`[[symbol]]` is drawn in every position and may not lie inside a valve body: the body
belongs to the renderer. For a valve, `[[behaviour.positions.symbol]]` is an optional
annotation drawn on top of the position; the flow paths themselves come from the
`connection` table, so the arrow direction is never authored by hand.

### Normative symbols

A definition is not free-form drawing. It must reproduce the symbol that
**ISO 1219-1:2012** defines for that function, in the diagram conventions of
ISO 1219-2:2012 and ISO 1219-3:2016. The requirements, the renderer's guarantees and
the review checklist are in [docs/symbology.md](../../docs/symbology.md); the short
version:

- one equal square per valve position, position *N* in the *N*-th square counted from
  the spring end, so the rest position is the square against the spring;
- flow paths as straight arrows pointing along the flow, closed ports marked with the
  T of a shut-off drawn inside their own square;
- every port identified with its numeral, identical in every square;
- the return spring at the spring end and the actuator at the actuator end, declared
  as parts and not as loose lines;
- working, pilot and drain lines drawn in their own line class;
- a filled dot where three or more lines join, no dot where lines cross.

`symbol lint` enforces the structural half of those rules when the catalog loads: eight
rules, one test each, in `crates/punchi-fluid-catalog/src/lint.rs`. A file that breaks
one is skipped and reported like any other malformed definition.

The visual half is reviewed once per symbol and recorded in
[assets/symbols/review/](../symbols/review/): the review file states `status:` as
`complete` or `pending-300dpi`, and a test fails when a built-in has no review. Every
built-in is reviewed at palette and canvas size and still `pending-300dpi`, which is why
the export keeps labelling the document as unverified.

**Convention:** the active position's square is filled and the other positions' paths
are dimmed, so the spool position never depends on colour alone. `PUNCHI_FLUID_PREVIEW=1
./scripts-dev/run.sh` opens the catalog sheet the reviews are based on: every definition
drawn large in the current theme.

A component that moves declares `kinematics = "linear_actuator"`: the file holds the
fixed parts and the renderer draws the piston, the rod and, **only if the file declares
`[spring]`**, the return spring from the simulation state. The barrel is the widest
rectangle the file declares, so the animation follows the geometry instead of a layout
hidden in the code: `cylinder.single_acting_20_100` has a spring, `cylinder.double_acting_32_200`
has two working ports and no spring. The piston crosses the whole internal length the
file declares — `stroke_mm` plus `dead_length_mm` — and stops with only the dead length
against the rod-side wall, which is the room the compressed spring occupies; the rod is
one piece of constant length, visible outside the barrel at rest and reaching the edge of
the symbol box at full stroke.

## Validation

The loader rejects a file when:

- `schema` is not `1`, or a key is unknown (a typo is an error, not a default);
- `id` is empty or uses characters other than lowercase letters, digits, `.`, `_`,
  `-`;
- `name` is empty, or the component has no ports;
- two ports share `id`, or a `position` falls outside `0..=1`;
- a primitive has the wrong number of values or a non-positive radius or stroke;
- a valve has fewer than two positions, a `default_position` that does not exist, a
  connection to an unknown port or to itself, or a non-positive conductance;
- a cylinder has a non-positive bore, stroke or mass.

## Testing a change

```bash
cargo test -p punchi-fluid-catalog     # format, precedence, disabled, typo
./scripts-dev/run.sh                   # watch it in the window
```

The built-in definitions are covered by a test that fails if any of them stops being
valid, because they ship inside the binary.
