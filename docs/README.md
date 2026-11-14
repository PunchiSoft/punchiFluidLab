# Project documentation

This is the index. `docs/` holds the decisions, the specification, the record and the
guides; it is not part of the application binary and is not shipped in a package. Every
document is reachable from here: the top-level pages are linked below, and the ADRs from
their own [index](adr/README.md). A new document is added here in the same change that
creates it.

The rules for writing and linking are in [style.md](style.md). The shape of the system and
the reason for each crate are in [architecture.md](architecture.md). The rules that bind
people and agents are in [AGENTS.md](../AGENTS.md), which is canonical.

## Start where you are

| If you… | Read |
|---|---|
| want to draw your first circuit | [guides/first-circuit.md](guides/first-circuit.md) → [guides/export-a-drawing.md](guides/export-a-drawing.md) |
| are opening the repository for the first time | [README](../README.md) → [architecture.md](architecture.md) → [CONTRIBUTING](../CONTRIBUTING.md) |
| are resuming work with no context | [retomar.md](retomar.md) (Spanish) → [next-steps.md](next-steps.md) |
| are continuing or resuming a parity run | [parity-run.md](parity-run.md) → [work-queue.md](work-queue.md) → `git log --oneline main..HEAD` |
| are about to write code | [next-steps.md](next-steps.md) → [capabilities.md](capabilities.md) → the relevant [skill](../.agents/skills/README.md) |
| are planning a capability | [capabilities.md](capabilities.md) and [capability-plan](../.agents/skills/capability-plan/SKILL.md) |
| are changing a symbol | [symbology.md](symbology.md) and [fluid-power-symbology](../.agents/skills/fluid-power-symbology/SKILL.md) |
| are changing the interface | [ui-ux.md](ui-ux.md) |
| are changing the sheet frame or the title block | [reference/title-block.md](reference/title-block.md) and [ADR 0026](adr/0026-the-sheet-is-the-paper.md) |
| are reviewing a change | [capabilities.md](capabilities.md), the phase in [next-steps.md](next-steps.md), and the ADR it touches |
| want to know why the system is shaped this way | [architecture.md](architecture.md) and the [ADR index](adr/README.md) |

## Kinds of document

The taxonomy is the usual four kinds of technical documentation, plus the material that
belongs to running the project. The folder is flat on purpose: the number of documents is
small and a deep tree makes a link harder to write, not easier to find.

| Kind | What it answers | In this repository |
|---|---|---|
| Tutorial | Teach by doing, for a reader who has never used the program | [guides/first-circuit.md](guides/first-circuit.md) |
| How-to | Solve one task | [guides/export-a-drawing.md](guides/export-a-drawing.md), [guides/troubleshooting.md](guides/troubleshooting.md) |
| Reference | Describe a format, a command, a field or a unit | [reference/sheet-format.md](reference/sheet-format.md), [reference/cli.md](reference/cli.md), [reference/catalog.md](reference/catalog.md), [reference/title-block.md](reference/title-block.md), [glossary.md](glossary.md), [assets/catalog/README.md](../assets/catalog/README.md) |
| Explanation | Explain why and how the system works | [architecture.md](architecture.md), [ui-ux.md](ui-ux.md), [symbology.md](symbology.md), [engineering.md](engineering.md) |
| Process and record | Say what is built, in what order, and what happened | [next-steps.md](next-steps.md), [capabilities.md](capabilities.md), [plan.md](plan.md), [retomar.md](retomar.md) |
| Decision | Record one choice and its consequences | [adr/](adr/README.md) |
| Design | A specification written and reviewed before the code (EGM) | [designs/](designs/README.md) |

## Status

A document that is not simply current carries a status line under its title:

| Status | Meaning |
|---|---|
| `current` | It describes the code as it is today; a disagreement is a bug in one of them. |
| `target` | It is the specification; it says which parts exist today and which are still a target. |
| `historical` | It is a dated record. It is not edited to look current; a new decision is a new document. |
| `internal` | It is for the maintainer, not the product: the continuity note is the only one. |

## The documents

| Document | Kind | Status | What it is |
|---|---|---|---|
| [designs/title-block-and-sheet.md](designs/title-block-and-sheet.md) | design | current | The EGM design for the sheet, the A4 landscape default, the title block and its positions, and the reference zones |
| [retomar.md](retomar.md) | process | internal | The owner's continuity note, in Spanish: state, history, verification, decisions and the prompt to continue |
| [parity-run.md](parity-run.md) | process | internal | The parity run's continuity note: branch, progress log, decisions and how to continue after a reset |
| [work-queue.md](work-queue.md) | process | current | The parity run's queue: one entry per capability, its state and its acceptance test |
| [next-steps.md](next-steps.md) | process | current | The work order: where the project stands, the next phase, the files and the tests |
| [capabilities.md](capabilities.md) | process | current | The coverage ledger: what a student must be able to do, its state and its evidence |
| [plan.md](plan.md) | process | current | The milestones and their acceptance criteria, with the status of each |
| [architecture.md](architecture.md) | explanation | current | The system map: crates, data flow, extension points, debts |
| [ui-ux.md](ui-ux.md) | explanation | target | The interface: regions, theme, accessibility, what is still open |
| [symbology.md](symbology.md) | explanation | current | The ISO 1219 contract, the renderer guarantees and the review checklist |
| [engineering.md](engineering.md) | explanation | current | Modularisation, memory budget, security model and GitHub practice |
| [reference/sheet-format.md](reference/sheet-format.md) | reference | current | The `.pfluid` format: fields, bounds, versions and migrations |
| [reference/cli.md](reference/cli.md) | reference | current | `check`, `validate` and `simulate`: formats, exit codes and the JSON contract |
| [reference/catalog.md](reference/catalog.md) | reference | current | The catalog at a glance, with the authoring guide as the canonical reference |
| [guides/first-circuit.md](guides/first-circuit.md) | tutorial | current | Draw, connect, run, check and save a first circuit |
| [guides/export-a-drawing.md](guides/export-a-drawing.md) | how-to | current | Choose the paper and write the sheet as PDF or PNG, with the known limits |
| [guides/troubleshooting.md](guides/troubleshooting.md) | how-to | current | The symptom, the cause and the way out |
| [glossary.md](glossary.md) | reference | current | The vocabulary of the project, in English with the Spanish interface word |
| [style.md](style.md) | process | current | How a document is written, linked, dated and proven |
| [adr/README.md](adr/README.md) | decision | current | The index of architecture decisions, with status and date |
| [adr/template.md](adr/template.md) | decision | current | The shape of a new decision record |
| [../assets/catalog/README.md](../assets/catalog/README.md) | reference | current | How to add, change or delete a component |
| [../assets/symbols/README.md](../assets/symbols/README.md) | reference | current | The symbol set, its provenance and its reviews |
| [../i18n/README.md](../i18n/README.md) | process | target | The translation plan |
| [../packaging/README.md](../packaging/README.md) | process | target | The packaging plans |
| [../AGENTS.md](../AGENTS.md) | process | current | The canonical rules for people and agents |
| [../CHANGELOG.md](../CHANGELOG.md) | process | current | Changes by date, under `Unreleased`, in Spanish |
| [../CONTRIBUTING.md](../CONTRIBUTING.md) | process | current | Setup, gate, commits and review |
| [../SECURITY.md](../SECURITY.md) | process | current | Supported versions, private reporting and scope |
| [../SUPPORT.md](../SUPPORT.md) | process | current | Where to ask, and what the project does not promise |
| [../CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md) | process | current | The code of conduct, adopted by reference |

## Reading order for a new contributor

1. [../README.md](../README.md) — what the program is and how to build it.
2. [architecture.md](architecture.md) — the map, so a file means something.
3. [../CONTRIBUTING.md](../CONTRIBUTING.md) — the environment and the gate.
4. [../AGENTS.md](../AGENTS.md) — the rules that are not negotiable.
5. The [skill catalog](../.agents/skills/README.md) — the procedure for the task at hand.
6. [next-steps.md](next-steps.md) and [capabilities.md](capabilities.md) — what is next and
   what proves it.

## Rules for this folder

- English is canonical, except [retomar.md](retomar.md) and [../CHANGELOG.md](../CHANGELOG.md).
- One `H1` per file, ordered sections, relative links, ISO dates, units with their symbol.
- A claim names its evidence or is removed; a number that can rot is derived or left out.
- A new document is listed in the table above in the same change.
- An accepted ADR is not rewritten; a new decision supersedes it (see
  [adr/template.md](adr/template.md)).
- `./scripts-dev/check-docs.sh` checks the workspace documents against `Cargo.toml`, the
  community files, the labels, the index coverage, every relative link and the ADR
  numbering, status and date. It runs in `./scripts-dev/check.sh`.
