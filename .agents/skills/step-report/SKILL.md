---
name: step-report
description: "Close a work step with the ordered work table and a short checklist for the owner — what is done and proven, what is still missing and who unblocks it, and the one next step. Use when a branch, commit, pull request or phase reaches a hand-off point, or when the owner asks for a status."
license: GPL-3.0-or-later
---

# Step report

A step is finished when it has a gate result, an owner's decision or a merged pull
request, not when the code compiles. The owner reads the report to decide what to
merge, not to reconstruct the session.

A step report has two layers. The **work table** lists every planned item with its status,
its evidence and who unblocks it; the short sections **OK**, **Falta** and **Siguiente**
summarise it. A step that is a phase of a [phased-plan](../phased-plan/SKILL.md) also carries
the **Plan** board: the board answers what is missing from the complete plan, the table
orders the phase's work, and the three sections answer what is missing from the phase.

The table is mandatory in the manual hand-off; the automatic pilot writes the same table
after every merged phase and keeps the board, so the owner can read the run without
reconstructing it.

This is the reporting half of
[pull-request-workflow](../pull-request-workflow/SKILL.md). The route (branch, gate,
pull request, merge) stays there; this skill only says how the hand-off is written.

## Procedure

1. **Name the step and its scope.** One line: the branch, the pull request or the phase,
   and what it was supposed to deliver.
2. **If it is a phase, start with the plan board.** Copy the board from the plan: every
   phase and the final evaluation and audit appear, `[x]` only for a merged phase (or an
   approved Phase 0), `[ ]` for an open pull request or a pending phase. Name in one line
   what remains. The board is the answer to "what is missing from the complete plan"; do
   not reconstruct it from memory. If the plan has no board, add it to the plan first; the
   report copies it, never invents it.
3. **Fill the work table from the phase checklist.** One row per planned item, in the
   plan's order: `Hecho`, `En curso`, `Falta` or `Bloqueado`; the evidence (test, gate, CI
   check, PR, merge commit); and who unblocks a missing or blocked item (`—` when nobody).
   A row without evidence is not `Hecho`.
4. **Prove every OK item from the tool, not from memory.** Re-read `gh pr view`,
   `gh pr checks`, `git status` and the gate output. Each bullet names its evidence: the
   test, the gate exit, the check name, the merge state.
5. **List what is missing and who unblocks it.** A pending owner merge, a decision, a
   screenshot that could not be taken headlessly, a deferred theme. Never hide a gap in
   the OK list to make the step look finished. The board carries the plan-level gaps; the
   table carries the item-level ones; this section carries the phase-level ones.
6. **Name exactly one next step.** What it is, what it depends on, and whether it waits
   for the merge.
7. **Write it in the language of the conversation** (Spanish today). The skill and the
   template stay in English.

## Template

```markdown
## Estado — <step>
**Plan — phase <n> of <N>**
- [x] <phase 1> — <name> — merged #<n>
- [x] <phase 2> — <name> — merged #<n>
- [ ] <phase 3> — <name> — PR #<n>, checks green, waiting for merge
- [ ] <phase 4> — <name> — pending
- [ ] final — evaluation and audit — pending
**Remaining in the plan:** <phases still `[ ]`, including the final audit>
**Trabajo**
| Ítem | Estado | Evidencia | Bloqueo |
|---|---|---|---|
| <planned item> | Hecho / En curso / Falta / Bloqueado | <test / gate / CI / PR / merge> | <who unblocks it, or —> |
**OK**
- <what is done> — <evidence: test / gate / check / merge state>
**Falta**
- <what is missing> — <who unblocks it>
**Siguiente**
- <one step> — <what unblocks it>
```

Omit the `Plan` block for a step that is not a phase; keep the work table. The board is
copied from the plan file, not invented in the report; it is a checklist, not prose, and it
names the final audit even when no phase has reached it. The table is ordered by the plan,
not by the order the work happened; an unplanned item waits for the plan or is added to it
first.

## Gotchas

- A `CLEAN` pull request is not merged. Only `state: MERGED` with a `mergedAt` closes
  the step, and only then the phase goes `[x]` on the board.
- **A phase report that names only the next phase hides the rest of the plan.** The board
  lists every phase and the final audit, and the `Remaining in the plan` line says it in
  one sentence; the `Siguiente` is one step, not the plan.
- **The board is copied from the plan and updated in the same hand-off.** A board that
  disagrees with the plan is the bug; fix both in the same pass.
- **The board is the plan; the table is the phase; OK, Falta and Siguiente are the summary.**
  Do not merge them into one list: the board says which phases remain, the table orders the
  phase's items, and the sections say what the phase still owes.
- **A manual hand-off without the work table is incomplete.** The owner asked for the
  ordered table; an OK list alone hides which planned item is missing.
- **A `Hecho` row without evidence is a claim, not a report.** Name the test, the gate, the
  CI check, the pull request or the merge commit.
- A green local gate does not prove CI, and a green CI does not prove the window draws.
  Say which one ran.
- `git branch -d` refuses a squash-merged branch. Prove it has no unique content with
  `git diff <merge-commit> <branch>` before deleting with `-D`.
- A step with nothing visible can still be finished (an ADR, a settings foundation);
  say what it unblocks instead of inventing a screenshot.
- Do not list files as progress. Name behaviour, evidence and the next decision.
- In manual mode the report ends with the pull request URL and the request to merge; in an
  approved automatic run the agent merges under ADR 0046 and the report records the merge.
  The agent never approves its own pull request in either mode.

## Validation

```bash
git --no-pager status --short
git --no-pager log --oneline origin/main..HEAD
gh pr view <n> --json state,mergedAt,mergeStateStatus
gh pr checks <n>
./scripts-dev/check.sh   # check-docs.sh for a documentation-only change
```

The report names the branch and the pull request, every OK item has a command or a
link, every missing item names who unblocks it, and there is exactly one next step. When
the step is a phase, the board names every phase and the final audit, and no `[x]` lacks a
merged pull request (or an approved Phase 0).
