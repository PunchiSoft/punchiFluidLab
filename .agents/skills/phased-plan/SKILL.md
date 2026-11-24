---
name: phased-plan
description: "Structure a requested plan as Phase 0 (baseline, scope, specification), delivery phases 1..N and a final evaluation and process audit, with a delivery checklist in every pull request. Use when the owner asks for a plan, a phase breakdown, or a multi-pull-request run."
license: GPL-3.0-or-later
---

# Phased plan

When the owner asks for a plan, the answer is a sequence of phases, not a list of files and
not a promise of dates. Each phase is one branch, one pull request and one checklist; the
last phase evaluates the result and audits how it was delivered. The shape is fixed so a
session that loses the conversation can resume from the plan alone.

A plan is work, not product: it lives in the owner's private notes. The capability entries,
the ADRs and the pull requests are what the repository keeps.

## When a plan is required

- **A code change needs a plan.** A feature, a fix, a failing test or a refactor starts with
  an implementation plan the owner approves before the first branch. A question that only
  asks for information does not need one; if it turns into a change, the plan comes first.
- **One pull request:** a short plan in the conversation, naming what changes, the files,
  the evidence and the risk, is enough. It still needs the owner's `proceede`.
- **Two or more pull requests:** this formal shape, with Phase 0, a phase board and a
  checklist per phase. The owner approves it before Phase 1.
- **The automatic pilot:** [auto-pilot](../auto-pilot/SKILL.md) runs only an approved plan
  with a board and at least two delivery phases. Without it, the answer is
  `Se necesita planificación previa; no se procede.`; the agent may offer to write the plan
  and waits.

The approval is written in the plan (`Approved by owner: <date>`). An unapproved plan is a
proposal, not a delivery contract.

## The shape

| Phase | Question it answers | Ends with |
|---|---|---|
| **0 — preparation** | what exists, what is in scope, and what will prove it? (0A baseline, 0B scope, 0C specification) | a proven baseline, a closed scope, the phase checklists |
| **1..N — delivery** | one deliverable each | a merged pull request and its checked list |
| **final — evaluation and audit** | did it work, and was it delivered well? | a verdict, the carried items, the next plan |

Phase 0 is preparation and changes no product code. 0A, 0B and 0C are separate because each
answers a different question: a plan that skips one of them starts coding from a guess.

A plan with one delivery phase is a step, not a plan: say so and use the step route; the
plan shape starts when the work needs at least two merges. A plan whose phases cannot each
merge alone is split wrong; the number of phases is the work's, never a promise.

## The skeleton

The plan is a document, written before the first branch. In the owner's private notes it is
in Spanish (`Fase`, `Hecho`, `Falta`); the marks are the ones below.

```markdown
# Plan — <subject> — <YYYY-MM-DD>

**Base:** `main` at `<sha>`, gate `<result>`.
**Scope:** <one sentence a student would recognise>.
**Out of scope:** <what this plan does not do, and why>.
**Approved by owner:** <YYYY-MM-DD, or "pending">.

## Phase board
- [ ] 0 — preparation (0A–0C) — in progress
- [ ] 1 — <name> — pending
- [ ] 2 — <name> — pending
- [ ] final — evaluation and audit — pending

## Phase 0 — Preparation (no product change)

### 0A — Baseline
- [ ] `main` equals `origin/main` at `<sha>`; gate `<result>`
- [ ] inventory: the capability rows, crates and behaviour this plan touches
- [ ] open pull requests and active branches: <...>

### 0B — Scope and authority
- [ ] in: <...>; out: <...>
- [ ] every open question and who settles it: <standard / doc / ADR / owner>

### 0C — Specification and evidence
- [ ] capability rows: <IDs in docs/capabilities.md>
- [ ] ADR: <number or "none">
- [ ] acceptance tests of each phase: <names>
- [ ] the branch and the checklist of each phase: written below

## Phase 1 — <name>
**Branch:** `<prefix>/<descriptor>` · **Acceptance:** <tests> · **Writes:** <crates>
- [ ] <outcome> — <evidence>
- [ ] <outcome> — <evidence>

## Phase N — <name>
...

## Final phase — Evaluation and process audit
- [ ] every acceptance criterion verified against the merged artefact
- [ ] capability rows, documents and changelog closed
- [ ] process audit written
- [ ] verdict, carried items and the next plan
```

## Phase 0

### 0A — baseline

Prove the ground before promising anything: `main` equals `origin/main` at a named commit,
the gate is green, the working tree is known, the open pull requests and active branches are
listed, and the capability rows the plan will touch are read in
[docs/capabilities.md](../../../docs/capabilities.md). No edit.

### 0B — scope and authority

Write what is in and what is out, in sentences a student would recognise. For every open
question, name who settles it: the standard, [docs/capabilities.md](../../../docs/capabilities.md),
an ADR, or the owner. A question without an authority is not ready for 0C.

### 0C — specification and evidence

This is the gate: no code before it is written.

- the capability rows the plan covers, with their state and the test that proves each;
- an ADR when a decision changes the architecture, the file format or a dependency edge;
- the acceptance test of every phase, named, at the level the claim needs
  ([rust-testing-and-verification](../rust-testing-and-verification/SKILL.md));
- the branch, the pull request and the checklist of every phase, written before the work.

These public documents are written here and travel with the phase that proves them — the
ledger row and the changelog entry in the same pull request as the code — unless the owner
asks for the specification as its own documentation change.

## A delivery phase

Each phase is one coherent slice that can merge alone, in this order: domain, model,
drawing, interface, document format. It never pretends to deliver a layer it skipped.

| Field | Content |
|---|---|
| Name | one behaviour, not a file list |
| Branch | `feat/<descriptor>`, `fix/<descriptor>`, `docs/<descriptor>` from updated `main` |
| Deliverable | what the student can do, or what the product now proves |
| Acceptance | the tests and captures that fail without it |
| Evidence | gate result, CI check, capture, measurement |
| Checklist | the one written in 0C, marked during delivery |
| Exit | merged by the owner, or by the agent under an approved automatic run (ADR 0046), with its checklist and evidence |

A phase is not finished because the code compiles or the pull request is open: it is
finished when the owner merges it, or the agent merges it under an approved automatic run of
[auto-pilot](../auto-pilot/SKILL.md) and ADR 0046
([pull-request-workflow](../pull-request-workflow/SKILL.md)).
The next phase starts from updated `main`, unless the owner explicitly authorises parallel
work; in that case the branch is created from `main` and the plan says so.

After the merge, mark the phase `[x]` on the board and carry the board into the
[step-report](../step-report/SKILL.md). The board is the plan's state; the phase checklist
is the phase's state. A report with one but not the other is half a report.

## The delivery checklist

Every pull request that delivers a phase carries the phase's checklist in its body, in
English beside the template's own checklist: after the claim and its evidence, before the
generic checklist. The owner's copy in the private plan keeps the same marks in Spanish.

```markdown
## Phase checklist — Phase 2: <name>

- [x] <planned outcome> — <evidence: test name, gate line, PR #, capture>
- [x] <planned outcome> — <evidence>
- [ ] <planned outcome> — missing: <what>; unblocks: <owner decision / phase 3>

### Missing from this phase
- <item> — <why it is not there> — <who unblocks it>

### Next
- <one next step> — <what it waits for>
```

Rules:

- **Written in 0C, marked during delivery.** A checklist written after the work is a report
  and hides the work that was skipped.
- **`[x]` is the check; `[ ]` is the missing item.** GitHub renders the first as a check, and
  the second stays visible with what is missing and who unblocks it.
- **Never delete an unchecked item.** Carry it to the next phase and say which; an item that
  is dropped says why, in the plan and in the pull request.
- **`[x]` is a claim with evidence.** Re-read the test, the gate and the pull request before
  marking it; memory is not evidence.
- **The list is the phase's, not the session's.** An item that was not planned is added to
  the plan first, with its reason, or it waits.
- **The report is the summary, not the list.** At hand-off,
  [step-report](../step-report/SKILL.md) gives OK / Falta / Siguiente from these boxes; the
  checklist is the evidence behind each line.

## The final phase

The last phase is never "everything done". It evaluates, and then it audits.

**Evaluation** — from the artefacts, not from memory:

- every phase's acceptance criteria, checked against the merged pull request, the test that
  exists, the capture, the measurement or the job in CI;
- the capability rows updated to their real state, `covered` only with the test that proves
  it;
- the changelog and the documents that travelled with the code;
- the phase board closed: every phase `[x]` or carried with a named reason;
- the debts that remain, named with their owner and the next plan that carries them.

**Process audit** — how the work was delivered:

| Question | Evidence |
|---|---|
| Was every phase based on updated `main`? | `git rev-list --left-right --count` per branch |
| One branch, one pull request, one thing per commit? | the pull request list and the log |
| Did the gate run on each branch? | the gate's output or the CI job |
| Did the owner merge, never the agent? | `state: MERGED` and `mergedAt` |
| Were the checklists honest? | every `[x]` has its evidence; every carry-over is named |
| Did the scope move? | the plan against what was delivered, with the reason |
| Did a private note leak? | no `docs-private/` path in the repository |
| Are the claims supported? | each claim names its test, gate or capture |

The audit is not a victory lap: it looks for drift between the plan, the code and the
evidence, and it writes what it finds. A finding that changes a product rule becomes a
public document in its own change; the audit itself stays in the owner's notes.

## Procedure

1. **Read the state.** `docs/README.md`, `docs/capabilities.md`, `docs/plan.md`, the owner's
   private work order, and Git: `main`, `origin/main`, open pull requests and branches.
2. **Write Phase 0**, 0A to 0C, before naming a single delivery phase.
3. **Split the work** into phases that each merge alone; name the branch of each.
4. **Write the checklists** of every phase now, with the evidence each item will need.
5. **Get the plan approved** by the owner before Phase 1 and write the approval in the
   plan. The plan is a document, not code; a phase that starts before 0C is a guess with a
   branch. Without the approval the automatic pilot does not start.
6. **Deliver each phase** through [pull-request-workflow](../pull-request-workflow/SKILL.md),
   with its checklist in the body, mark its row on the board, and hand off with
   [step-report](../step-report/SKILL.md); the board is what the owner reads to know what
   remains.
7. **Close with the final phase**: evaluation, audit, verdict and the carried items.

## Where the plan lives

- The plan and its audit are the owner's working notes: `docs-private/plan-<slug>-<YYYY-MM-DD>.md`,
  in Spanish, untracked and never in a pull request. The repository does not link to them.
- The capability rows, the ADR, the code, the tests and the pull-request checklists are the
  public half; they are what a future session reads.
- If the owner asks for a public plan, it is a document under `docs/` with its index row, and
  the same shape applies.
- If a plan produces a decision the product must keep, that decision becomes a public
  document in its own change, not a copy of the plan.

## Adapted to this repository

| Need | Use |
|---|---|
| The gate | `./scripts-dev/check.sh`; a documentation-only change runs `./scripts-dev/check-docs.sh` |
| The route | [pull-request-workflow](../pull-request-workflow/SKILL.md) |
| A capability | [capability-plan](../capability-plan/SKILL.md) and [docs/capabilities.md](../../../docs/capabilities.md) |
| The hand-off | [step-report](../step-report/SKILL.md) |
| The window's tests | `cargo test -p punchi-fluid-window`, the CI `window` job |
| The owner's notes | `docs-private/`, ignored; never linked from the repository |

The plan does not repeat those documents: it names the phase, the branch and the evidence, and
links the rest.

## Gotchas

- **No 0C, no code.** A capability without its row and its test is a wish; the plan cannot
  make it ready.
- **A phase that cannot merge alone is split wrong.** Cut by behaviour, not by layer or file.
- **A checklist is not retrofitted.** If it was written after the work, it is a report and it
  hides what was skipped; say so and write the next plan's checklists first.
- **Dates are not phases.** The project promises no date; a plan says what will be proven,
  not when.
- **Parallel phases are an exception.** Only with explicit owner authorization, always from
  `main`, and the plan records it; a branch stacked on another phase is not a phase.
- **The plan is not the ledger.** Link `docs/capabilities.md`; do not copy its states into the
  plan.
- **A plan without a board is a list of phases.** The board is what the owner reads to know
  what remains; the phase checklist is the phase, not the plan.
- **A plan without the owner's approval is a proposal.** The automatic pilot refuses it, and
  the manual mode does not start the first branch from it.
- **A `[x]` on the board without a merged pull request (or an approved Phase 0) is a false
  claim.** The board is updated after the merge, not after the push.
- **The final audit reads the repository, not the plan.** Git, CI and the merged pull
  requests are the evidence; the plan is the claim under audit.
- **A checked list with a silent gap is worse than an honest one.** A missing item that is
  deleted from the list looks delivered.

## Validation

```bash
git --no-pager rev-list --left-right --count main...HEAD
git --no-pager log --oneline origin/main..HEAD
gh pr view <n> --json state,mergedAt,mergeStateStatus
./scripts-dev/check.sh          # or check-docs.sh for a documentation-only change
```

The plan has 0A, 0B, 0C, at least two delivery phases (or says it is a step) and the final
phase; every phase names its branch, its acceptance and its checklist; every `[x]` names its
evidence; every carried item names its phase; the audit names each phase's merged pull
request and its gate.
