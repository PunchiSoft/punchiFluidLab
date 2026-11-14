# The `.pfluid` sheet format

Status: current, format 6. The implementation is
`crates/punchi-fluid-file/src/sheet.rs`; its tests are the authority when this page and the
code disagree.

A `.pfluid` file is a **project**: one or more circuits, each with its own placements and
pipes, plus the circuit that was in view. It is JSON, with a version number and no unknown
keys. It is **not** the old circuit format that `validate` and `simulate` read; see
[the CLI](cli.md) for that distinction.

## Versioning

| Field | Current | Read | Write |
|---|---|---|---|
| `format` | `12` | `1` to `12` | always `12` |

- A file with a **newer** `format` is refused with `sheet format N is newer than this build
  understands (12)`; it is never guessed.
- **Version 12** renamed the `model` of a block from the Spanish `escolar`/`tecnico` to
  `school`/`technical`. A format 11 file is read as the model it named and saved with the
  English name.
- **Version 11** moved the block positions to `top-right`, `bottom-right` and
  `bottom-full`, and added the free `remark`. `right` and the retired `left` read as
  bottom right; `full` reads as the bottom band.
- **Version 10** added the `model` and the per-field `visible` set of a block; an older
  block reads as the industrial model with every field.
- **Version 9** added the `project`, `equipment`, `scale` and `norms` of a block; an older
  block reads with an empty project and equipment, a `S/E` scale and no norms.
- **Version 8** added the `block` of a circuit: twelve text fields, revision `A`, scale
  `S/E`, and the circuit title as the default title. The fields, the models and their
  layout are in [title-block.md](title-block.md).
- **Version 7** added the `sheet`: the paper, its orientation and its margins. An older
  file reads as an unframed A4 that is exactly the area the older release drew on.
- **Version 6** added the `class` of a pipe: working, pilot or drain. A working line is
  not written, so an old sheet looks as it always did.
- **Version 5** added the `source` of a recorder: which placement it charts and which
  quantity. An older file reads with no recorder bound. The migration is step-wise, so a
  format 4 project keeps its circuits; a single step that reset them would merge every
  sheet into the first.
- **Version 4** added the circuit list, the `circuit` id on a placement and a pipe, and the
  `active` circuit. Versions 1–3 read as a project with **one circuit**, id `0`; saving
  writes the fields the reader filled in.
- Version 3 added the designation; version 2 added orientation; version 1 was the first
  sheet. A change to the shape bumps `SHEET_VERSION` and adds a migration test.

## Bounds

A project is untrusted input. Every collection and every number is checked before the
document is used.

| Bound | Value | Constant |
|---|---|---|
| File size | 8 MiB, checked before reading | `MAX_BYTES` |
| Circuits | 64 | `MAX_CIRCUITS` |
| Circuit title | 1 to 80 characters, trimmed, no control characters | `MAX_CIRCUIT_TITLE` |
| Placements | 2 000 in the project | `MAX_PLACEMENTS` |
| Pipes | 4 000 in the project | `MAX_PIPES` |
| Catalog or port id | 128 bytes; the catalog id is ASCII | `MAX_ID` |
| Symbol box side | greater than 0 and at most 2 000 mm | `MAX_SIDE_MM` |
| Designation | 1 to 40 characters, trimmed, no control characters | `MAX_DESIGNATION` |
| Pipe offset | at most 10 000 tenths of a millimetre (1 m) | — |

Unknown keys are rejected at every level. A duplicate circuit id, a duplicate placement id,
a pipe to a placement that is not in the project, or a pipe from a component to itself is an
error, not a repair. A placement or pipe that names a circuit not in the list is an error
too.

## Top level

```json
{
  "format": 4,
  "title": "Proyecto",
  "circuits": [
    { "id": 0, "title": "Potencia" },
    { "id": 1, "title": "Control" }
  ],
  "placement": [],
  "pipe": [],
  "active": 0
}
```

| Key | Type | Required | Meaning |
|---|---|---|---|
| `format` | integer | yes | Format version. |
| `title` | string | no, default `""` | Project name. |
| `circuits` | array of objects | yes in v4 | The circuits, in selector order. A v1–v3 file is migrated to one circuit. |
| `placement` | array of objects | no, default `[]` | Components of the whole project. |
| `pipe` | array of objects | no, default `[]` | Connections of the whole project. |
| `active` | integer | no, default `0` | Circuit that was in view; a hint, not an edit. |

## A circuit

```json
{ "id": 1, "title": "Control" }
```

| Key | Type | Meaning |
|---|---|---|
| `id` | integer | Unique in the project. A placement or a pipe names it. |
| `title` | string | Shown in the selector and the printed title block. |

A circuit is a **sheet**. The solver, the drawing and the export see the active circuit;
the other circuits keep their own placements and values. Switching circuit is a view
change, not an undo entry.

## A placement

```json
{
  "id": 2,
  "circuit": 0,
  "catalog": "valve.3_2.spring_return",
  "x": 94.0,
  "y": 93.6,
  "width": 33.8,
  "height": 33.8,
  "edit": { "conductance": 1.2e-9 },
  "quarter_turns": 1,
  "mirrored": false,
  "designation": "1V1"
}
```

| Key | Type | Required | Meaning |
|---|---|---|---|
| `id` | integer | yes | Identity a pipe uses in the project. Unique, non-zero. |
| `circuit` | integer | yes in v4 | Circuit this component belongs to. |
| `catalog` | string | yes | Catalog id of the definition, for example `supply.compressor`. |
| `x`, `y` | number | yes | Centre of the symbol box, millimetres. |
| `width`, `height` | number | yes | Symbol box, millimetres; finite, positive, at most 2 000. |
| `edit` | object | no | Per-instance overrides; absent means "use the definition". |
| `quarter_turns` | integer `0..=3` | no, default `0` | Clockwise quarter turns; omitted when zero. |
| `mirrored` | boolean | no, default `false` | Reflection on the symbol's own vertical axis; omitted when false. |
| `designation` | string | no | Name the user gave the component; omitted when it has none. |
| `source` | object | no | What a recorder charts; omitted when the component is not one or is not bound. |

The `edit` object, when present, may carry any of these fields. An absent field keeps the
value from the definition.

| Key | Unit | Rule |
|---|---|---|
| `pressure_bar` | bar, gauge | finite, zero or positive |
| `conductance` | m³/(s·Pa) | finite, positive |
| `stroke_mm` | mm | finite, positive |
| `bore_mm` | mm | finite, positive |
| `rod_mm` | mm | finite, zero or positive; smaller than `bore_mm` when both are present |
| `moving_mass_kg` | kg | finite, positive |

### The source of a recorder

```json
{ "placement": 2, "quantity": "pressure" }
```

| Key | Type | Meaning |
|---|---|---|
| `placement` | integer | Identity of the component being observed. It must exist, be in the same circuit and not be the recorder itself. |
| `quantity` | string | `pressure` (gauge bar at the first port), `flow` (normal litres per minute through a meter) or `position` (millimetres of a cylinder). |

The source is a **document setting**: it is saved, undone and checked with the placement.
The samples are not: a recorder starts empty when the document is opened, because the run
that produced them is gone. Whether the component can report the quantity is the catalog's
rule, refused by the ABI rather than written into the file.

## A pipe

```json
{
  "circuit": 0,
  "from": 1,
  "from_port": "1",
  "to": 2,
  "to_port": "1",
  "offset_tenths": 25,
  "class": "pilot"
}
```

| Key | Type | Required | Meaning |
|---|---|---|---|
| `circuit` | integer | yes in v4 | Circuit this pipe belongs to. |
| `from`, `to` | integer | yes | Placement ids in the project, different from each other. |
| `from_port`, `to_port` | string | yes | Port ids declared by the definition; non-empty, at most 128 bytes. |
| `offset_tenths` | integer | no, default `0` | Offset of the middle segment from the automatic route, in tenths of a millimetre. Bounded to ±10 000. |
| `class` | string | no, default `working` | `working`, `pilot` or `drain`; decides the dash pattern. A working line is omitted. |

The two ends **should** belong to the same circuit. The format does not refuse a pipe whose
ends are in different circuits, because the check has to see it: that is the ninth fault,
`PipeAcrossCircuits`. The editor never creates one; a hand-edited file can, and the loader
keeps it so the check can report it instead of hiding it.

The route is not stored point by point: it is the orthogonal route the editor computes
from the two ports, moved by `offset_tenths`.

## A complete example

```json
{
  "format": 4,
  "title": "Válvula 3/2 y control",
  "circuits": [
    { "id": 0, "title": "Potencia" },
    { "id": 1, "title": "Control" }
  ],
  "placement": [
    {
      "id": 1,
      "circuit": 0,
      "catalog": "supply.compressor",
      "x": 34.0,
      "y": 39.0,
      "width": 33.8,
      "height": 33.8
    },
    {
      "id": 2,
      "circuit": 0,
      "catalog": "valve.3_2.spring_return",
      "x": 94.0,
      "y": 93.6,
      "width": 33.8,
      "height": 33.8,
      "quarter_turns": 1,
      "designation": "1V1"
    },
    {
      "id": 3,
      "circuit": 0,
      "catalog": "cylinder.single_acting_20_100",
      "x": 144.0,
      "y": 33.8,
      "width": 74.4,
      "height": 33.8,
      "edit": { "stroke_mm": 80.0 },
      "designation": "1A1"
    },
    {
      "id": 4,
      "circuit": 1,
      "catalog": "junction.tee",
      "x": 40.0,
      "y": 40.0,
      "width": 20.0,
      "height": 20.0
    },
    {
      "id": 5,
      "circuit": 1,
      "catalog": "chart.recorder",
      "x": 90.0,
      "y": 100.0,
      "width": 23.8,
      "height": 23.8,
      "source": { "placement": 2, "quantity": "pressure" }
    }
  ],
  "pipe": [
    { "circuit": 0, "from": 1, "from_port": "1", "to": 2, "to_port": "1" },
    { "circuit": 0, "from": 2, "from_port": "2", "to": 3, "to_port": "1", "offset_tenths": 25 }
  ],
  "active": 0
}
```

## What the sheet does not store

The project is the circuits. These belong to the session or to the user's machine and are
deliberately absent:

- the zoom, the scroll position and the selection;
- the undo history;
- **the samples a recorder collected**, which belong to the run and not to the document;
- the edit/simulation mode and the simulated time;
- the catalog definitions themselves, which live in `assets/catalog/` and the user folder;
- the recent-document list and the safety copy, which live in `punchi-fluid-file::recent`
  and are written by the ABI.

The `active` index is a convenience hint: a project opens on the circuit that was in view,
but switching circuits is not an edit and is not undone.

## Reading and writing

- **Atomic write**: the writer validates the document, writes `<name>.pfluid.tmp` in the
  same folder and renames it over the target. A crash leaves the previous file intact.
- **Read**: the file size is checked before reading, then JSON parsing, then the migration,
  then validation. A malformed file is an error with the reason; it is not repaired into a
  different circuit.
- **The window and the ABI** save and open this format. The `punchi-fluid-cli check`
  command reads it. The legacy `validate` and `simulate` commands do **not**; they read the
  old circuit format.
