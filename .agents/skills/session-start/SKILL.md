---
name: session-start
description: "Load the project context read-only at the start of a session: AGENTS, docs, capabilities, plan, skills and open work; then say you are in context and wait for the owner's theme, without asking what to fix. Use when the first message asks to read the project and get in context."
license: GPL-3.0-or-later
---

# Session start

The first message of a session is often "read the project, get in context, review your
skills". It is a **read-only** request: the owner wants the agent aligned with the project
before naming the work. It is not a request for a diagnosis, a plan or a status report.

The skill ends when the agent says it is in context and the owner names the theme. Asking
"what should I fix?" or "what do you detect?" is the failure this skill exists to prevent.

## What to read

In this order; read, do not edit:

| Read | For |
|---|---|
| [AGENTS.md](../../../AGENTS.md) | the canonical rules, the stack and the delivery rule (owner merge or pilot) |
| [the skill catalog](../README.md) | the skills and the order to read them |
| [docs/README.md](../../../docs/README.md) | which document is current, target or historical |
| [docs/capabilities.md](../../../docs/capabilities.md) | the ledger: what the product does, what is missing and the test |
| [docs/plan.md](../../../docs/plan.md) | the milestones and their acceptance criteria |
| the theme's canonical document | [ui-ux.md](../../../docs/ui-ux.md) for the ribbon, [symbology.md](../../../docs/symbology.md) for a symbol, and so on |
| `docs-private/retomar.md` and `next-steps.md`, if present | the owner's continuity and work order; read-only, never quoted or committed |
| Git and GitHub | the branch, the dirty state, the recent commits, the open pull requests and the worktrees |

Do not run the gate, build, switch branches, edit a file or open a pull request. Reading
`git status`, `gh pr list` and the documents is the work.

## The report

One short context block, at most five bullets, then exactly one line and stop:

```markdown
**Contexto**
- Producto: <one sentence: what it is and where it stands>.
- Ahora: `main` at `<sha>`; branch `<name>` <clean/dirty>; open PRs: <none/#n>.
- Línea de trabajo: <the plan or phase the owner is on, from the documents>.
- Skills leídas: <the relevant ones, not all>.
- Deuda o riesgo: <the one thing that matters now, if any>.

Ya estoy en contexto. ¿Qué necesitas de <tema>?
```

If the owner did not name a theme, the last line is `Ya estoy en contexto. ¿Qué necesitas?`.
No sixth bullet, no plan, no checklist.

The block and the closing line are written in the language of the conversation; the example
is Spanish because that is the owner's language today. The skill stays in English.

## Forbidden

- Asking what to fix, what the agent detects, what to review or where to start.
- Proposing a plan, a phase breakdown or a TODO list.
- Running the gate, a build or a test to "see how it is".
- Editing, committing, branching or opening a pull request.
- A context block longer than five bullets.

The owner names the theme in the next message; the work starts there.

## Gotchas

- A dirty tree or an open pull request is **state to report**, not a question to ask. Say it
  in "Ahora" and wait.
- The owner's notes in `docs-private/` are read-only and never quoted in a public pull
  request; a product fact from them becomes a public document in its own change.
- The context is not a [step-report](../step-report/SKILL.md): no OK / Falta / Siguiente and
  no plan board. Those come when a phase closes.
- The context is not a [phased-plan](../phased-plan/SKILL.md): no phases until the owner asks
  for a plan.
- "Review your skills" means the catalog, not a new skill; load the ones the theme needs when
  the theme arrives.
- If the owner names the theme in the first message, echo it in the closing line; do not ask
  for it again.
- If a document contradicts another, say which one is current in "Ahora"; do not resolve it
  with a new decision at session start.

## Validation

The turn ends with the closing line and **no tool call after it**. The next message from the
owner names the theme; if the agent asked a diagnostic question or started a plan, the skill
failed.
