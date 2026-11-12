# Documentation style

Rules for the documents in `docs/`, the READMEs and the guides. They exist so that a reader
finds the same shape in every file and a claim can be checked instead of believed. The
canonical rules for the code and the repository are [AGENTS.md](../AGENTS.md); this page
is only about writing.

## Language

- **English is canonical** for the repository: code identifiers, comments, logs, tests,
  scripts, commit subjects and bodies, pull-request titles and bodies, and `docs/`. The
  deliberate exceptions are [retomar.md](retomar.md) (the owner's continuity note in
  Spanish) and [CHANGELOG.md](../CHANGELOG.md) (Spanish by project decision).
- [README.es.md](../README.es.md) is the Spanish copy of the README and must say the same
  thing; when one changes, the other changes in the same pull request.
- The interface is Spanish today. A document may name a command in Spanish and gloss it in
  English, for example **Iniciar** (start). The same applies to a comment that quotes a UI
  label: the label is Spanish, the sentence is English. The i18n plan is
  [i18n/README.md](../i18n/README.md).
- `--selftest` output, internal diagnostics and test names are English. A Spanish UI label
  is quoted, never translated into a Spanish sentence.
- No document names another simulator, copies its manual, its symbols or its examples
  (ADR 0009). When the behaviour of another program is the reference, describe the
  behaviour, never the product.

## Shape of a document

- One `H1` at the top, and section levels in order. A file does not start with `##`.
- A **status line** under the title when the document is not simply current:
  `Status: current`, `Status: target`, `Status: historical` or `Status: internal`.
  A target document says what exists today and what is still a target; a historical
  document is dated and is not edited to look current.
- A short opening paragraph that says what the document is for and who should read it.
- Tables for decisions, fields and comparisons; numbered lists for procedures; prose for
  reasoning. A table with one row is a sentence.
- Dates in ISO 8601 (`2026-11-12`). Units with their symbol and a space: `6 bar`, `100 mm`,
  `1.0e-9 m³/(s·Pa)`.
- No emoji as decoration in `docs/` or the ADRs. The README may use the construction sign
  as a status marker; the changelog and the interface may use a symbol when it is the
  thing itself (`•` in the title, a transport glyph).

## Claims and evidence

- A claim names its evidence level: unit test, invariant or reference curve, ABI test,
  headless window test, measurement, screenshot or manual review. "Works" without a level
  is not a claim; see
  [rust-testing-and-verification](../.agents/skills/rust-testing-and-verification/SKILL.md).
- A number that can rot (test count, line count, crate size) is either derived by a script
  in the same change or left out. Prefer "the gate is green" to "37 tests".
- A gap is written as a gap. A program that says what it cannot do is trusted; one that
  describes an export panel it does not have is not.
- When a normative question is open, the document says so and names the phase that closes
  it. It does not guess a standard clause.

## Links and paths

- Relative links, from the file that contains them. `docs/ui-ux.md` links to
  `../assets/sounds/README.md`, not to an absolute path.
- Link text names the destination ("the paper ADR", not "here").
- A link to a file the repository does not have is a defect, not an intention; the
  documentation check fails on it.
- A document that moves updates every link in the same change. A redirect stub is not used:
  the repository is small enough to fix the links.

## Naming

- Files in `kebab-case.md`, except the ADRs, which are `NNNN-kebab-case.md`.
- A guide is named after what the reader does (`first-circuit.md`, `export-a-drawing.md`),
  not after the feature (`export.md` is allowed only while there is one export page).
- A reference is named after the thing it describes (`sheet-format.md`, `catalog.md`).
- A decision is named after the decision, in the imperative or the noun phrase the ADR
  title uses.

## What not to duplicate

The project pays context for every duplicate that drifts. Put the rule in one place:

| Question | Home |
|---|---|
| What may a person or an agent do? | [AGENTS.md](../AGENTS.md) |
| What does the program have to do, and what proves it? | [capabilities.md](capabilities.md) |
| What is built next, in which order? | [next-steps.md](next-steps.md) |
| Why is the system shaped this way? | [architecture.md](architecture.md), [adr/](adr/README.md) |
| What does the interface look like? | [ui-ux.md](ui-ux.md) |
| What does ISO 1219 require? | [symbology.md](symbology.md) |
| What is the budget, the threat model and the GitHub practice? | [engineering.md](engineering.md) |
| How do I add or change a component? | [assets/catalog/README.md](../assets/catalog/README.md) |

A guide may link to these and summarise one sentence; it does not restate the table.

## Review checklist

Before a documentation pull request is opened:

- [ ] The language is English, except the continuity note, the changelog, the Spanish
      README and the user-facing UI strings.
- [ ] One `H1`, ordered sections, a status line when the document is not simply current.
- [ ] Every claim names its evidence or is removed.
- [ ] Every relative link resolves; no link to a file that does not exist.
- [ ] Dates, units and numbers are in the project format.
- [ ] The changed document is listed in [README.md](README.md) if it is new.
- [ ] `CHANGELOG.md` has the entry under `Unreleased`.
- [ ] `./scripts-dev/check-docs.sh` is green, and the documentation architecture check
      (`docs/README.md`, ADR index, links) is green.
