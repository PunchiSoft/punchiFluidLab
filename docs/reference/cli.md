# The command line

Status: current. The implementation is `crates/punchi-fluid-cli/src/main.rs`; the check's
JSON contract is [ADR 0011](../adr/0011-the-check-is-a-public-command.md).

`punchi-fluid-cli` runs without a window. It is the door a course, a script or CI uses to
check a student's circuit, and the gate uses it as a smoke test.

```text
punchi-fluid-cli <validate|simulate|check> <file> [options]
```

## Two formats, and which command reads which

This is the one thing to know before using the command:

| Command | Reads | For |
|---|---|---|
| `check` | the **sheet** format, `.pfluid`, versions 1–3 | the circuit the editor saves |
| `validate` | the **legacy circuit** format, version 1 | the first vertical slice and the old CLI examples |
| `simulate` | the **legacy circuit** format, version 1 | the old fixed model |

The editor and the window save the sheet format; a file a student hands in is checked with
`check`. `validate` and `simulate` do **not** read a `.pfluid` sheet and will report
`unsupported format version 3`. They are kept for the old format and are not the path a
teacher should use.

The two formats are described in [the sheet format](sheet-format.md) and in the legacy
`Document` type in `crates/punchi-fluid-file/src/lib.rs`.

## Commands

### `check <file> [--json] [--strict]`

Reads a sheet, builds the same document the window would, runs the design check against the
built-in catalog and the editor's logical sheet (200 × 130 mm), and prints the findings.

```bash
punchi-fluid-cli check circuito.pfluid
punchi-fluid-cli check circuito.pfluid --json
punchi-fluid-cli check circuito.pfluid --strict
```

- Without `--json` it prints `ok: sin faltas`, or one line per finding plus the count.
- `--json` prints one object, described below.
- `--strict` changes only the exit code: without it, a sheet with findings still exits `0`;
  with it, it exits `3`. It never changes what is printed.
- The check uses the **built-in catalog**. A user definition with the same id is checked
  against the built-in one; a user definition the build does not have is not seen at all.
  `--catalog` is a later phase.

The six findings this build knows, in the order the report uses:

| `kind` | Message (Spanish) |
|---|---|
| `outside_sheet` | un componente se sale de la hoja |
| `overlapping_components` | dos componentes se solapan |
| `pipe_through_component` | un tubo cruza un componente que no es su extremo |
| `overlapping_pipes` | dos tubos se superponen |
| `open_connection` | una conexión neumática queda abierta |
| `duplicate_designation` | dos cilindros llevan la misma designación |

Three faults the reference lists are not implemented yet: a pipe whose ends do not fit, a
label that does not match, and a pipe through an unrelated circuit. The ledger in
[capabilities.md](../capabilities.md) says why each one waits.

### `validate <file>`

Reads the legacy circuit format and prints `ok: N components` when it is valid. It does not
run the design check and does not read a sheet.

### `simulate <file>`

Reads the legacy circuit format, steps the old model by 1 ms and prints `ok: N volumes`. It
is a smoke test of the old solver, not the network the editor builds.

## Exit codes

| Code | `check` | `validate` / `simulate` |
|---|---|---|
| `0` | the file was read and checked (even with findings, unless `--strict`) | the file was read (and simulated) |
| `1` | the file could not be read, parsed or validated | the file could not be read, parsed or simulated |
| `2` | the command line is wrong | the command line is wrong |
| `3` | findings were reported and `--strict` was asked for | — |

A wrong command line prints a usage line on stderr. A missing file is `1`, not `2`.

## The JSON object

`check --json` prints exactly one object:

```json
{
  "version": 1,
  "file": "circuito.pfluid",
  "count": 2,
  "findings": [
    {
      "kind": "outside_sheet",
      "message": "un componente se sale de la hoja",
      "x": 250.000,
      "y": 60.000,
      "component": 0,
      "pipe": null
    },
    {
      "kind": "open_connection",
      "message": "una conexión neumática queda abierta",
      "x": 145.000,
      "y": 97.000,
      "component": 2,
      "pipe": null
    }
  ]
}
```

- `version` is the version of the JSON contract, independent of the sheet format. A
  consumer that reads it must refuse a version it does not know.
- `file` is the path as it was given on the command line.
- `count` is the length of `findings`.
- `x` and `y` are millimetres on the editor's logical sheet, with three decimals.
- `component` is the zero-based index into `placement`, or `null`; `pipe` is the zero-based
  index into `pipe`, or `null`.
- The order is stable: by `kind` in the table above, then by document index, so two runs on
  the same file produce the same array.

A file that cannot be read or parsed with `--json` still prints one object, with an empty
`findings` array and an `error` string, and exits `1`:

```json
{
  "version": 1,
  "file": "roto.pfluid",
  "count": 0,
  "findings": [],
  "error": "invalid sheet: ..."
}
```

## What the command does not do

- It does not open a window, read a desktop theme or need a display.
- It does not write or convert a sheet.
- It does not load the user catalog yet.
- It does not run the network solver for a sheet; `simulate` is the legacy path.
- It does not print a human-readable report with colours or a summary file; a pipeline uses
  `--json`.
