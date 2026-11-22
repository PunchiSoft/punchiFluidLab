# 0040 — The standard's symbols may be reconstructed from its references

- **Status:** proposed
- **Date:** 2026-11-22
- **Deciders:** project owner, implementation session
- **Context crates:** `punchi-fluid-catalog`, `punchi-fluid-draw`

## Context

[ADR 0009](0009-identity-and-standards.md) forbids material from another program and
requires symbols drawn by hand to conform. The symbology rules went one step further and
forbade tracing a symbol **from the standard itself**, because the standard's figures and
text are copyrighted and the repository records our engineering contract, not its pages.

The symbols are normative: ISO 1219-1 defines one form for each, identical in every
application. The owner's references are captures of **those symbols**, not screenshots of
another program, and the reconstruction of one from its reference is the task the new
`punchi-symbol-reconstruction` skill must solve. Reading a symbol's measures from the
standard's own form is not copying another program; it is reading the object the project
implements. Keeping the two cases under one prohibition blocks the workflow without
protecting anything the rule was written for.

## Decision

1. **A standard reference may be used.** A capture of a symbol as ISO 1219-1 defines it may
   identify the form and be read for its measures: positions, radii, angles, port sides and
   numerals. The reconstruction is written as our TOML.
2. **The rule against another program stays.** No capture, asset, icon, manual, example,
   text or file format from another program enters the repository or drives the
   reconstruction. `scripts-dev/check-docs.sh` keeps failing on those names.
3. **The reference image is never stored.** The repository keeps the TOML, the geometry
   fixture, our own captures and the review; no image, crop or path of the reference is
   committed.
4. **The review records the source.** For each reconstructed symbol it names the standard,
   the edition and the clause read, the measures taken from the reference, and the fact that
   the reference is the standard's symbol.
5. **The image tooling is developer tooling.** It is deterministic where possible, it writes
   a candidate TOML or a comparison for a person to review, and it never declares a symbol
   conformant on its own.
6. **The copyright boundary is the text and the figures of the standard.** They are not
   reproduced; the symbol forms are normative and used as intended. The owner accepts this
   reading as the project's provenance rule.

## Consequences

- `AGENTS.md`, [docs/symbology.md](../symbology.md) and the `fluid-power-symbology` skill
  distinguish a reference of another program (forbidden) from a reference of the standard
  (allowed, with the measures recorded).
- The new `punchi-symbol-reconstruction` skill and its scripts read a reference, propose the
  TOML, render it with `symbol-sheet`, and compare it with the reference for a person to
  sign.
- The no-third-party-asset check and the review requirement are unchanged.
- **Debt:** a reader of the repository cannot tell from the TOML alone whether a measure came
  from the standard's clause or from its drawing; the review's table carries that record.

## Alternatives considered

- **Keep the ban on the standard too.** Rejected: it blocks reconstructing the object the
  project implements, while the rule's purpose — no other program's material — is covered by
  point 2.
- **Allow any screenshot as a reference.** Rejected: that is the third-party material ADR
  0009 exists to keep out.
- **Store the reference next to the symbol.** Rejected: it would put the standard's figure in
  the repository, which is the copyright boundary this record keeps.

## References

- [ADR 0009](0009-identity-and-standards.md), the rule this amends.
- [ADR 0002](0002-components-are-files.md): the geometry stays in the file.
- ISO 1219-1:2012, ISO 1219-2:2012 and ISO 1219-3:2016.
- The owner's decision of 2026-11-22: the references are captures of the standard's symbols.
