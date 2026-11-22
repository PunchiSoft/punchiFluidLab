# 0040 — The owner's reference authorizes the reconstruction

- **Status:** proposed
- **Date:** 2026-11-22
- **Deciders:** project owner, implementation session
- **Context crates:** `punchi-fluid-catalog`, `punchi-fluid-draw`

## Context

[ADR 0009](0009-identity-and-standards.md) forbids material from another program and
requires symbols drawn by hand to conform. The symbology rules went further and forbade
tracing a symbol from the standard itself, because the standard's figures and text are
copyrighted and the repository records our engineering contract, not its pages.

The project owner is the authority for the intended symbol: its reference appearance, its
functional interpretation and the engineering requirements they supply. The owner's
references are not limited to the standard — they include photographs, hand-drawn sketches,
screenshots from any software, technical manuals, normative figures, existing diagrams and
annotated images. Refusing all of them blocks the reconstruction the owner wants, while the
rule's real purpose — keeping another application's implementation and proprietary material
out of the repository — is a separate matter.

## Decision

1. **The owner may provide any visual reference**: photographs, hand-drawn sketches,
   screenshots from any software, technical manuals, normative standard figures, existing
   hydraulic or pneumatic diagrams and annotated images.
2. **The references may be analysed** to identify geometry, proportions, positions,
   connections and graphical relationships when the owner authorizes it.
3. **The agent may extract geometric information** from an owner-provided reference and
   construct an independent TOML definition with PunchiFluidLab's own primitives and
   renderer.
4. **The record distinguishes five things**: the observed geometry, the owner-provided
   functional requirements, the normative requirements, the engineering interpretations and
   the assumptions that still need confirmation. A normative requirement is never presented
   as verified without adequate evidence.
5. **The owner's functional instructions take precedence** over unsupported model
   assumptions. When the reference and the declared normative behaviour disagree, the agent
   identifies the discrepancy and asks the owner how to proceed.
6. **The purpose is the intended symbol**, not the reproduction of another application's
   internal implementation, proprietary assets or source code.
7. **All resulting geometry is represented in PunchiFluidLab's TOML schema.**
8. **The reference image is not stored.** The repository keeps the TOML, the geometry
   fixture, our own captures and the review; no image, crop or path of the reference is
   committed.
9. **Another program's source code and assets remain out.** The reconstruction does not copy
   them, and `scripts-dev/check-docs.sh` keeps failing on the forbidden names.

## Consequences

- `AGENTS.md`, [docs/symbology.md](../symbology.md) and the `fluid-power-symbology` skill
  distinguish an owner-provided reference (analysed, with the five categories recorded) from
  another application's source and assets (forbidden).
- The `punchi-symbol-reconstruction` skill and its scripts read an owner-provided reference,
  propose the TOML, render it with `symbol-sheet`, and compare it with the reference for a
  person to review.
- The review records the provenance: what was observed, what the owner required, what the
  standard states, what is our interpretation and what remains an assumption.
- The owner accepts this reading as the project's provenance rule.
- **Debt:** a reader of the TOML alone cannot tell which measure came from the owner's
  reference, the standard or an interpretation; the review's table carries that record.

## Alternatives considered

- **Keep the blanket ban.** Rejected: it blocks the owner's workflow while the rule's real
  purpose — no another program's source or assets — is covered by point 9.
- **Allow only the standard's figures.** Rejected: too narrow for the owner's references
  (photographs, sketches, screenshots, manuals, diagrams).
- **Store the reference next to the symbol.** Rejected: the repository keeps our TOML and
  our captures, not the reference image.

## References

- [ADR 0009](0009-identity-and-standards.md), the rule this amends.
- [ADR 0002](0002-components-are-files.md): the geometry stays in the file.
- ISO 1219-1:2012, ISO 1219-2:2012 and ISO 1219-3:2016.
- The owner's decision of 2026-11-22: the references are provided by the owner and the
  reconstruction is written in our TOML.
