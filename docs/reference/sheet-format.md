# The `.pfluid` sheet format

Status: current, format 3. The implementation is
`crates/punchi-fluid-file/src/sheet.rs`; its tests are the authority when this page and the
code disagree.

A `.pfluid` file is the circuit the editor saves: the placements, their pipes, the
orientation and the name of each component, and the per-instance values that override the
catalog. It is JSON, with a version number and no unknown keys. It is **not** the same
document as the legacy circuit format that `validate` and `simulate` read; see
[the CLI](cli.md) for that distinction.

## Versioning

| Field | Current | Read | Write |
|---|---|---|---|
| `format` | `3` | `1` to `3` | always `3` |

- A file with a **newer** `format` is refused with `sheet format N is newer than this build
  understands (3)`; it is never guessed.
- An **older** file is read with the default of every field it does not have: version 1 had
  no orientation, version 2 had no designation, and both read as an unturned, unnamed
  component. Reading upgrades the in-memory document to format 3, so saving it writes the
  fields the reader filled in.
- A change to the shape bumps `SHEET_VERSION` and adds a migration test. A new field needs a
  documented default for every older file.

## Bounds

A sheet is untrusted input. Every collection and every number is checked before the
document is used.

| Bound | Value | Constant |
|---|---|---|
| File size | 8 MiB, checked before reading | `MAX_BYTES` |
| Placements | 2 000 | `MAX_PLACEMENTS` |
| Pipes | 4 000 | `MAX_PIPES` |
| Catalog or port id | 128 bytes; the catalog id is ASCII | `MAX_ID` |
| Symbol box side | greater than 0 and at most 2 000 mm | `MAX_SIDE_MM` |
| Designation | 1 to 40 characters, trimmed, no control characters | `MAX_DESIGNATION` |
| Pipe offset | at most 10 000 tenths of a millimetre (1 m) | — |

Unknown keys are rejected at every level. A duplicate placement id, a pipe to a placement
that is not on the sheet, or a pipe from a component to itself is an error, not a repair.

## Top level

```json
{
  "format": 3,
  "title": "Circuito",
  "placement": [],
  "pipe": []
}
```

| Key | Type | Required | Meaning |
|---|---|---|---|
| `format` | integer | yes | Format version. |
| `title` | string | no, default `""` | Name shown in the window. It is stored but not editable in the interface yet. |
| `placement` | array of objects | no, default `[]` | Components, in placement order. |
| `pipe` | array of objects | no, default `[]` | Connections, in drawing order. |

## A placement

```json
{
  "id": 2,
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
| `id` | integer | yes | Identity a pipe uses. Unique, non-zero. |
| `catalog` | string | yes | Catalog id of the definition, for example `supply.compressor`. |
| `x`, `y` | number | yes | Centre of the symbol box, millimetres. |
| `width`, `height` | number | yes | Symbol box, millimetres; finite, positive, at most 2 000. |
| `edit` | object | no | Per-instance overrides; absent means "use the definition". |
| `quarter_turns` | integer `0..=3` | no, default `0` | Clockwise quarter turns; omitted when zero. |
| `mirrored` | boolean | no, default `false` | Reflection on the symbol's own vertical axis; omitted when false. |
| `designation` | string | no | Name the user gave the component; omitted when it has none. |

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

## A pipe

```json
{
  "from": 1,
  "from_port": "1",
  "to": 2,
  "to_port": "1",
  "offset_tenths": 25
}
```

| Key | Type | Required | Meaning |
|---|---|---|---|
| `from`, `to` | integer | yes | Placement ids on the sheet, different from each other. |
| `from_port`, `to_port` | string | yes | Port ids declared by the definition; non-empty, at most 128 bytes. |
| `offset_tenths` | integer | no, default `0` | Offset of the middle segment from the automatic route, in tenths of a millimetre. Bounded to ±10 000. |

The route is not stored point by point: it is the orthogonal route the editor computes
from the two ports, moved by `offset_tenths`. That is what lets a dragged segment survive a
move of either component without storing a second geometry.

## A complete example

```json
{
  "format": 3,
  "title": "Válvula 3/2 y cilindro",
  "placement": [
    {
      "id": 1,
      "catalog": "supply.compressor",
      "x": 34.0,
      "y": 39.0,
      "width": 33.8,
      "height": 33.8
    },
    {
      "id": 2,
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
      "catalog": "cylinder.single_acting_20_100",
      "x": 144.0,
      "y": 33.8,
      "width": 74.4,
      "height": 33.8,
      "edit": { "stroke_mm": 80.0 },
      "designation": "1A1"
    }
  ],
  "pipe": [
    { "from": 1, "from_port": "1", "to": 2, "to_port": "1" },
    { "from": 2, "from_port": "2", "to": 3, "to_port": "1", "offset_tenths": 25 }
  ]
}
```

## What the sheet does not store

The document is only the circuit. These belong to the session or to the user's machine and
are deliberately absent:

- the zoom, the scroll position and the selection;
- the undo history;
- the edit/simulation mode and the simulated time;
- the catalog definitions themselves, which live in `assets/catalog/` and the user folder;
- the recent-document list and the safety copy, which live in `punchi-fluid-file::recent`
  and are written by the ABI.

A sheet that opens on another machine therefore opens with the same circuit and the same
values, and with whatever the local catalog and view settings are.

## Reading and writing

- **Atomic write**: the writer validates the document, writes `<name>.pfluid.tmp` in the
  same folder and renames it over the target. A crash leaves the previous file intact.
- **Read**: the file size is checked before reading, then JSON parsing, then validation.
  A malformed file is an error with the reason; it is not repaired into a different
  circuit.
- **The window and the ABI** save and open this format. The `punchi-fluid-cli check`
  command reads it. The legacy `validate` and `simulate` commands do **not**; they read the
  old circuit format.
