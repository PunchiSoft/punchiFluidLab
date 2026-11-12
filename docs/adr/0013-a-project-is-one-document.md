# 0013 — A project is one document with several circuits

- **Status:** accepted
- **Date:** 2026-11-12
- **Context crates:** `punchi-fluid-file`, `punchi-fluid-doc`, `punchi-fluid-check`, `punchi-fluid-ffi`

## Context

Today a `.pfluid` file is one circuit. A course, a laboratory exercise or a machine often
needs several circuits that belong together: the power circuit, its control circuit and a
variant. Handing in one file per circuit loses the relationship, and a single undo history
cannot cover a change that moves a component from one circuit to another.

The reference behaviour groups circuits under a project and offers a selector between them.
Its ninth design check is a pipe that runs through a circuit it is not connected to. We keep
the **behaviour** — a project of circuits, a selector, a check for a pipe that does not
belong — but not the reference's container, which is a project file beside separate circuit
files. The owner chose one document with several circuits on 2026-11-12: one file is easier
to save, hand in, undo and check, and a student does not manage a folder of paths.

## Decision

A project is **one document** with one or more circuits:

- Format 4 adds `circuits`, keeps `placement` and `pipe` at project level, and adds a
  `circuit` id to every placement and pipe. `active` records which circuit was in view.
- A circuit has an id and a title. The project holds between 1 and 64 circuits, at most 2 000
  placements and 4 000 pipes in total; every id is unique in the project.
- **A circuit is a sheet.** The solver, the drawing and the export see only the active
  circuit; switching circuits is a view change, not an undo entry. The other circuits keep
  their own placements, pipes and per-instance values.
- The editor creates a placement or a pipe in the active circuit. A pipe whose two ends are
  in different circuits is refused by the editor and reported by the check as
  `PipeAcrossCircuits`.
- The ninth fault is adapted to this model. In the reference a pipe runs through a circuit
  it is not connected to; here a circuit is a sheet, so the same mistake is a pipe that joins
  two circuits. The check points at the pipe, and the drawing never shows a pipe that
  belongs to another sheet.
- A version 1, 2 or 3 sheet reads as a project with one circuit; saving writes version 4.
  The migration is tested with the exact text older releases wrote.

## Consequences

- `punchi-fluid-file` gains `StoredCircuit`, the project-level arrays and a v3 migration.
- `punchi-fluid-doc` gains a circuit identity on a placement and a pipe, and the commands
  that create one take the active circuit. The undo history stays one history for the whole
  document, so a circuit switch is not an edit.
- `punchi-fluid-check` gains `PipeAcrossCircuits`; the other faults run on the active sheet.
- The ABI gains the circuit list, the active circuit, and commands to select, add and rename
  a circuit. The window gains a selector.
- The export prints the active circuit and names it in the title block.
- The reference's geometric crossing (a pipe drawn across another circuit's symbols on one
  canvas) is **not** planned: circuits are pages, and the pipe-between-circuits fault covers
  the same teaching mistake with a simpler model.

## Alternatives considered

- **A project file beside circuit files**, which is what the reference does. More faithful to
  the reference container, but the owner chose one document; it also makes one undo history,
  one save and one check across the project harder.
- **Several circuits on one shared canvas.** It would allow the geometric version of the
  ninth fault, but it complicates editing, the solver, the draw and hit-testing for the first
  slice. A page is the unit a student already understands.
- **One `.pfluid` per circuit and a folder of documents.** It was the F4 row's earlier wish,
  but it leaves the relationship implicit and the project check impossible without reading a
  folder.

## References

- The reference behaviour, observed for the project selector and the ninth check. No manual,
  symbol, example or file format from it enters this repository (ADR 0009).
- [P1.8 in next-steps](../next-steps.md), the capability entry in [capabilities.md](../capabilities.md)
  and the migration tests in `punchi-fluid-file`.
