# Tube sizes

Status: current. The series is implemented in `crates/punchi-fluid-core/src/tube.rs`, the
drawing map in `crates/punchi-fluid-draw/src/placed.rs` (`tube_width_mm`,
`route_length_mm`), and the parts row in `crates/punchi-fluid-export/src/parts.rs`. The
tests are the authority when this page and the code disagree.

A tube is a property of a **pneumatic connection**, not a component: it is stored on the
pipe, drawn as the pipe's stroke, listed in the parts list and saved with the sheet. The
series is this project's own **nominal practice list**, not a normative ISO table — ISO 1219
defines symbols, line classes and line widths, not tube outside diameters.

## The value

| Property | Value |
|---|---|
| Stored form | `Option<u16>`, nominal outside diameter in **hundredths of a millimetre** |
| Bounds | `100..=5000` (1–50 mm); `MAX_TUBE_HUNDREDTHS = 5000` |
| Unspecified | `None`: an older sheet or an electrical wire |
| New pneumatic connection | `Some(1000)` — **10 mm**, `DEFAULT_TUBE_HUNDREDTHS` |
| Format | optional `tube_od_hundredths` in sheet format **13** |

Integer hundredths keep `Eq`/`Hash` on the document and the file, so the undo history
compares two pipes exactly; a floating-point diameter would break that. The default is a
documented constant, not a page setting: **Editar ▸ Conducto** changes the selected pipe,
never a global default.

## The metric series

| Label | Hundredths |
|---|---|
| 4 mm | 400 |
| 5 mm | 500 |
| 6 mm | 600 |
| 8 mm | 800 |
| 10 mm | 1000 |
| 12 mm | 1200 |
| 14 mm | 1400 |
| 16 mm | 1600 |
| 20 mm | 2000 |
| 25 mm | 2500 |
| 32 mm | 3200 |
| 40 mm | 4000 |
| 50 mm | 5000 |

## The inch series

The stored value is the exact imperial size converted to millimetres and rounded to the
hundredth; the label is the entry's name. Metric and inch entries are **separate**: 10 mm is
not 3/8", and 6 mm is not 1/4".

| Label | Exact mm | Hundredths |
|---|---|---|
| 1/8" | 3.175 | 318 |
| 5/32" | 3.96875 | 397 |
| 3/16" | 4.7625 | 476 |
| 1/4" | 6.35 | 635 |
| 5/16" | 7.9375 | 794 |
| 3/8" | 9.525 | 953 |
| 1/2" | 12.7 | 1270 |
| 5/8" | 15.875 | 1588 |
| 3/4" | 19.05 | 1905 |
| 1" | 25.4 | 2540 |

A value inside the bounds that is not on either list is accepted and printed as a
measurement with the decimal comma the interface uses: `10,50 mm`. A named entry keeps its
name: `10 mm`, `3/8"`.

## The drawn width

The width is a **series**, not a continuous map, so a 4 mm and a 6 mm line differ at 300 dpi
and the sheet frame stays a different class of line.

| Tube | Width |
|---|---|
| `None` (an older pipe) | 0.42 mm |
| 4–5 mm | 0.25 mm |
| 6–8 mm | 0.35 mm |
| 10 mm and more | 0.50 mm |
| electrical wire | the class stroke, unchanged |

The largest pipe width is **0.50 mm**, below the **0.7 mm** ISO 5457 frame. The class pattern
does not change: working lines stay solid, pilot lines dashed, drain lines dashed and
electrical wires dotted. Port stubs belong to the symbol and do not thicken; the selection
ring keeps the base line width.

## The parts row

The printed parts list carries one row per `(od, class)` **after** the component rows:

```text
Tubo 10 mm · trabajo · 3,4 m
Tubo sin especificar · drenaje · 4,0 m
```

- The length is the **drawn route** in whole millimetres, including the port leads, summed
  in `f64` and rounded to the nearest millimetre.
- Below one decimetre the row reads `< 0,1 m`.
- An unspecified pipe gets its own row: the technician sees that the tubing exists and that
  its size is unknown.
- An electrical wire gets no row, because it is not a tube.
- The length is a **drawn route**, with no bend allowance, fittings or waste. It is not a
  cut list for a bender; it is the tubing the drawing shows.
- The `MAX_ROWS` cap counts components and tubes together, components first; the omitted
  counters say which kind was cut.

## Where it lives

| Layer | What it does |
|---|---|
| `punchi-fluid-core::tube` | The series, the labels, the bounds, the default |
| `punchi-fluid-file` | `StoredPipe::tube_od_hundredths`, format 13, the migration and the bounds |
| `punchi-fluid-doc` | `Connection::tube_od_hundredths`, `connect(from, to, tube)`, `seed_pipe_tube`, `Change::Tubed`, splice inheritance |
| `punchi-fluid-draw` | `Pipe::tube_od_hundredths`, `tube_width_mm`, `route_length_mm`, the electrical skip |
| `punchi-fluid-export` | `TubeRow`, `PartsList::group`, the sheet row and its height |
| `punchi-fluid-ffi` | The getter/setter, `pf_session_pipe_is_electrical`, the series enumeration, the parts data |
| `apps/punchi-fluid-qt` | `TubeSizeControl.qml`, the Conducto group, the status bar, `Alt+D` |

## Not in this series

The physics of the tube — conductance, volume, pressure drop, velocity and the sizing
warning — is **P-024b**: it needs a reference curve and a tolerance, and the solver keeps its
current pipe model until then. Fittings, tube material, wall thickness, pressure rating,
bend radius and colour coding are out of scope. The **electrical connection** is deferred at
the owner's request (P-026) and has no tube.

## Evidence

- `crates/punchi-fluid-core/src/tube.rs` tests the series and the bounds.
- `crates/punchi-fluid-file/src/sheet_tests.rs` tests the format-13 round trip and the
  format-12 read.
- `crates/punchi-fluid-draw/src/placed.rs` tests the width series and the electrical stroke.
- `crates/punchi-fluid-export/src/parts.rs` tests the rows, the length text and the cap.
- `crates/punchi-fluid-ffi/tests/abi.rs` tests the getter/setter, the refusal and the series.
- `--selftest` steps `tube: …`; `docs/symbology.md` records the manual width review.
