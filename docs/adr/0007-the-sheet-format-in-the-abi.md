# 0007 — The saved sheet is a file format of `punchi-fluid-file`, written by the ABI

- Status: accepted
- Date: 2026-11-06
- Deciders: project owner, implementation session

## Context

The editor works on `punchi-fluid-doc::Document`: placements, pipes with their routes,
per-instance values and an undo history. Until now the only file format in the workspace
was `punchi-fluid-file`'s legacy circuit format, which describes a different, smaller
domain (`core` components by kind). The window had no way to save its work.

Two shapes were possible:

1. put the editor format in `punchi-fluid-doc` and let the ABI serialise the document;
2. keep formats in `punchi-fluid-file`, with neutral stored types, and let the ABI map
   between them.

## Decision

`punchi-fluid-file::sheet` owns the saved schematic: a bounded, versioned JSON document
with placements (catalog id, centre, box, per-instance values) and pipes (placement ids,
ports, route offset). `punchi-fluid-ffi` depends on `punchi-fluid-file` and maps the
document to and from it; the format crate does not depend on `doc`, so a change in the
editor's in-memory model does not drag physics or drawing into the file layer.

`punchi-fluid-file` already depended on `core` and `serde`; the new edge is
`ffi -> file`, in the direction the ABI already takes for `catalog`, `doc`, `draw` and
`solver`. No crate depends on `ffi`.

## Consequences

- The format is testable headlessly (round trip, unknown keys, bounds, atomic write)
  without a window.
- The legacy `Document`/`Circuit` format stays where it is; the CLI keeps using it. The
  two formats are separate on purpose: the editor's sheet carries geometry and instances
  the legacy circuit does not.
- The ABI gains `pf_session_save`, `pf_session_open` and `pf_session_new_document`, and
  the window gains Archivo ▸ Nuevo/Abrir/Guardar/Guardar como.
- A future migration of the sheet format is a change in `punchi-fluid-file` plus a
  migration test, following the standing rule. Version 2 added the orientation of a
  placement, the quarter turns and the mirror, as two fields that an unturned symbol
  omits; a version 1 file reads with no orientation and is written back as version 2.
