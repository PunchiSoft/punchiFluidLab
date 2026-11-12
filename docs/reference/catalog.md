# The component catalog at a glance

Status: current, schema 3. The canonical authoring guide, with a field-by-field example and
the lint rules, is [assets/catalog/README.md](../../assets/catalog/README.md). This page is
the map: where the files live, what a definition is made of, and what the loader refuses.

## Where a definition lives

| Origin | Folder | Wins on a conflict |
|---|---|---|
| Built-in | `assets/catalog/*.toml`, compiled into the binary with `include_str!` | loses |
| User | `~/.local/share/punchi-fluid-lab/catalog/`, or `$PUNCHI_FLUID_CATALOG_DIR` | wins by `id` |

A user file with the same `id` replaces the built-in one; `disabled = true` hides it,
including a built-in file. A malformed file is **skipped and reported**, never fatal: the
program starts with whatever is valid. The skipped files and their reasons are available
from the catalog API; the window does not show them yet.

## What a definition declares

| Field | Meaning |
|---|---|
| `schema` | File format, `3` today; `2` still reads. `1` and a future version are refused. |
| `id` | Stable `family.variant` in `snake_case`, for example `valve.3_2.spring_return`. |
| `name`, `name_es` | Name in English and the Spanish name the interface shows. |
| `category` | `supply`, `valve`, `actuator`, `throttle`, `accessory` or `sensor`. |
| `summary` | One line, optional. |
| `disabled` | Hides the definition, including a built-in one with the same id. |
| `port` | Connection points, each with an `id`, a `role` and a position inside the symbol box. |
| `symbol` | Primitives drawn in every position: line, polyline, polygon, rect, circle, arc, text. |
| `body` | For a directional valve, the rectangle split into one equal square per position. |
| `spring` / `return` | The return part: a spring, air, fluid or a detent. `spring` is the short form of `return kind = "spring"`; declaring both is an error. |
| `actuator` | The actuation part: lever, push button, pedal, solenoid, and so on. |
| `behaviour` | What the solver does with it: `supply`, `valve`, `cylinder`, `gauge`, `flow_meter`, `throttle` or `junction`. |
| `kinematics` | Moving parts the renderer animates, for example a piston. |

A port has a **role**, not a number that means something: `supply`, `working`, `exhaust` or
`mechanical`. A valve position declares its open paths by port id, and the solver classifies
them by role. That is why a new valve is a new file and not a new branch in a `match`.

The `behaviour` is the one closed enum, and a new variant is the exception that needs an ADR
(ADR 0002, ADR 0008). Everything a file can express with the existing behaviours is a file,
not code.

## What the loader checks

- `schema` is 2 or 3; an unknown key is an error, not a default (a typo in a key is caught).
- `id` is not empty, `name` is not empty, every port id is not empty and every port sits
  inside the symbol box (`0..=1`).
- Every number is finite and inside its range; a supply pressure and a throttle conductance
  are positive; a cylinder bore and stroke are positive; a rod is zero or positive and
  smaller than the bore.
- A valve declares a body with one equal square per position, a path or a blocking mark per
  position, a numeral per port and a declared spring or actuator where the behaviour needs
  one.
- Every line declares its line class; no primitive is outside the box; no line is
  zero-length and no circle has zero radius.
- The symbol is legible at palette size.

The lint has one test per rule in `punchi-fluid-catalog`, and a built-in definition that
fails it is a build failure, not a runtime surprise. The review of each symbol, with its
light and dark captures, lives in
[assets/symbols/README.md](../../assets/symbols/README.md).

## What a definition never contains

- A path in Rust, a widget, a colour of the interface or a font.
- A number that only makes sense in one document: per-instance pressure, stroke, bore, rod
  and mass are overrides stored in the sheet, not in the catalog.
- A second copy of the standard's geometry: the drawing is written by hand against ISO 1219
  and reviewed; a definition is our own text, not a trace of another program.
