# 0011 — The circuit check is a public command with a versioned contract

- **Status:** accepted; amended by [0026](0026-the-sheet-is-the-paper.md), which makes the
  measured area the document's sheet
- **Date:** 2026-11-07

## Context

The check of §4.10 already exists as a library (`punchi-fluid-check`) and is reachable from
the window, where the faults are drawn where they happen. It is not reachable from anywhere a
teacher can script: a course with forty student circuits is corrected by opening forty
windows, and nothing about the result is machine-readable.

That is also the one place where a free simulator can be clearly better than the reference it
replaces: the reference validates inside its own window and its document format is closed, so
no course can automate the correction of a submission.

## Decision

`punchi-fluid-cli` gains a `check` command over the **sheet** format, the one the editor
saves:

```
punchi-fluid-cli check <file> [--json] [--strict]
```

- Without `--json` it prints one line per fault, in the same order and with the same Spanish
  message the window shows.
- With `--json` it prints one object, `{"version": 1, "file": …, "count": …, "findings": […]}`,
  and nothing else on stdout. A file that cannot be read or parsed is reported in the same
  shape with an `error` field, so a caller never has to parse prose.
- Exit code `0` means the file is a sheet and was checked; `1` means it could not be read or
  parsed; `2` means the command line was wrong; `3` means faults were found **and** `--strict`
  was asked for. A caller that wants the list but not a failed job omits `--strict`.

The contract is public from the day it ships: its `version` moves when a field changes
meaning, and a test holds the shape. The ABI is unchanged — the window keeps calling the
library directly.

## Consequences

- `punchi-fluid-cli` gains three edges: `punchi-fluid-catalog` (the definitions the ports come
  from), `punchi-fluid-doc` (the document the check consumes) and `punchi-fluid-check` (the
  check itself). All three are downward in the existing graph and acyclic; the allowlist in
  `scripts-dev/check-architecture.sh` records them.
- The sheet-to-document mapping lives in the CLI for now, and the ABI keeps its own. Two
  mappings that must agree is a drift risk: the test `the_window_and_the_command_see_the_same_circuit`
  pins the one property that matters — the same file yields the same faults — and the mapping
  moves into `punchi-fluid-doc` when a third caller appears.
- The sheet area the check uses is the document's sheet: the drawing space inside the ISO
  5457 frame, so "outside the sheet" means the same in the window, the command and the
  printed page ([ADR 0026](0026-the-sheet-is-the-paper.md)).
- A future `--catalog <dir>` is not part of this decision: the command uses the built-in
  catalog, which is what a submission is expected to need.
