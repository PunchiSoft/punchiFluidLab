# 0009 — The project stands on its mission and the standards

- **Status:** accepted
- **Date:** 2026-11-07
- **Supersedes:** the naming and "base product" rules of [0001](0001-language-and-layout.md)
  and [0005](0005-qt-quick-over-a-c-abi.md)
- **Context crates:** all of them

## Context

The documentation described the product in terms of another program: it was named in
`AGENTS.md`, in both READMEs, in the capability plan and in the skill that planned
capabilities, and the planning procedure depended on a local copy of that program's manual
and component library — including the path of that copy, an inventory of its files and
verbatim sentences from the manual, in a public repository.

Two problems, and they are different. The first is identity: the project presented itself
as a replacement for something else, when what it is for is a student with no licence, no
budget and no connection. The second is legal: quoting a manual and inventorying a library
that this project does not own is not something a free project should carry, and a folder
name does not change what the material is.

## Decision

1. **The program is described by its mission and by the standard**, never by another
   product. No third-party product name, vendor, manual title, file name, inventory or
   count appears in a tracked file.
2. **The authority for behaviour** is ISO 1219-1 (symbols), ISO 1219-2 and ISO 1219-3
   (diagram conventions), ISO 6358 (flow through a restriction), ISO 5598 (vocabulary), and
   the teaching goal: a student must be able to do it, understand it or check it.
3. **A capability enters the plan when a student needs it**, ordered by teaching value. The
   coverage states are `missing`, `partial`, `covered`, `improved` and `not planned`.
4. **The content rule is general and permanent:** no symbol, icon, bitmap, sound, example
   circuit, manual page, documentation text or file format from any other program enters
   this repository. Symbols are drawn by hand to conform and reviewed.
5. **A study copy, if one exists, lives outside the repository.** It is addressed only
   through `$PUNCHI_REFERENCE_DIR`, which has no default; the repository never records its
   name, its structure, its files or its counts, never quotes it, and never needs it to
   write the plan.

## Consequences

- `docs/capabilities.md` and `.agents/skills/capability-plan/` replace the parity document
  and its skill. Both are written from the standards and from our own teaching goals, and
  the decision log describes behaviour in our own words.
- ADR 0001 and ADR 0005 keep their decisions and lose the name; the amendment is recorded
  in both and here, so the record stays honest about what changed.
- `scripts-dev/check-docs.sh` fails when a forbidden term returns to a tracked file, so this
  is enforced rather than remembered.
- Planning no longer depends on any program being installed. That is the point: the
  community this project is for does not have it either.
- A capability that can only be specified by opening something the community cannot open is
  not specified yet.

## What would reverse this

Nothing about naming: the content rule is a licence boundary, not a preference. A future
change of product identity would need its own ADR and would still not license copying
assets, symbols, manual text or file formats.
