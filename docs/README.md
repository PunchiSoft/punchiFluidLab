# Project record

`docs/` holds the decisions, the specification and the development record. It is not
part of the application binary and is not shipped in a package.

**Resuming work without context? Start with [retomar.md](retomar.md)** (Spanish): state
of the project, what was done step by step, how to verify it, decisions taken, open
questions and the exact prompt to continue in a new window.

**Writing code? Start with [next-steps.md](next-steps.md):** where the project stands,
what is built next, which files it touches and which tests prove it.

Reading order for someone new: [next-steps.md](next-steps.md) →
[ui-ux.md](ui-ux.md) → [symbology.md](symbology.md) → [plan.md](plan.md) →
[engineering.md](engineering.md), with the ADRs as reference.

| Document | Purpose |
|---|---|
| [retomar.md](retomar.md) | **Continuity note (Spanish)**: everything needed to resume with no context |
| [next-steps.md](next-steps.md) | **The work order**: current state, next steps, files, tests, open questions |
| [adr/0001-language-and-layout.md](adr/0001-language-and-layout.md) | Why Rust, and why the workspace is split into crates |
| [adr/0002-components-are-files.md](adr/0002-components-are-files.md) | Why valves and symbols are definition files instead of Rust types |
| [ui-ux.md](ui-ux.md) | The interface: regions, theming, accessibility, what it refuses |
| [symbology.md](symbology.md) | ISO 1219 conformance: requirements, renderer guarantees, review checklist |
| [plan.md](plan.md) | Milestones with acceptance criteria and evidence |
| [engineering.md](engineering.md) | Modularisation, memory budget, security model, GitHub practices |
| [../assets/catalog/README.md](../assets/catalog/README.md) | How to add, change or delete a component |
| [../AGENTS.md](../AGENTS.md) | Rules for people and agents working in this repository |
| [../CHANGELOG.md](../CHANGELOG.md) | Changes by date, under `Unreleased` |

Architecture decisions are recorded as ADRs: one file per decision, numbered,
never edited after acceptance except to mark it superseded.
