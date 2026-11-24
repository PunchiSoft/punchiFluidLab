---
name: auto-pilot
description: "Run an approved phased plan unattended: branch, implement, gate, commit, pull request, verify CI and CLEAN, squash-merge under ADR 0046, return to main, clean up and continue. Use when the owner says `piloto automatico` and a phased plan is approved."
license: GPL-3.0-or-later
---

# Automatic pilot

The automatic pilot is the delivery mode of an **approved** phased plan. It is not the
default and it is not a permission: the owner asks for it with `piloto automatico`
(case-insensitive), and a plan has already been approved. Without both, the agent stays in
the manual mode of [pull-request-workflow](../pull-request-workflow/SKILL.md).

The decision and its guardrails are [ADR 0046](../../../docs/adr/0046-the-auto-pilot-runs-a-planned-batch.md).
This skill is the procedure: the loop, the stopping rules, the journal and the report.

## Preconditions

Check all of these before the first branch. If one is missing, stop and say which one.

1. **The trigger.** The owner asked for `piloto automatico`, case-insensitive, and has not
   said `detener`. Any other wording is manual mode.
2. **The plan is approved.** An approved [phased-plan](../phased-plan/SKILL.md) exists, with
   Phase 0, a phase board and at least two delivery phases; every phase names its branch,
   acceptance tests and checklist. The approval is written in the plan
   (`Approved by owner: <date>`). A plan in the conversation is not enough for this mode.
3. **The ground is clean.** `main` equals `origin/main` at the baseline commit, the gate is
   green, the checkout is clean, and no other branch with unmerged work or open pull request
   competes with the plan. One phase at a time is the rule, not an optimisation.
4. **The next phase is inside the mode.** It does not touch `.github/workflows`, secrets, a
   dependency addition, the ABI removal, `AGENTS.md`, an ADR or this delivery policy. Those
   stay manual; stop and hand them to the owner.

If there is no approved plan, answer exactly:

`Se necesita planificación previa; no se procede.`

Then stop. It may offer to write the plan and wait; it never invents one, creates a branch or
merges anything.

## The journal

The run keeps a private continuity note:
`docs-private/auto-pilot-<plan-slug>-journal.md`. It is untracked and never in a pull
request. Write after every step:

| Field | Content |
|---|---|
| Plan | path and its approved date |
| Phase | number and name |
| Branch | the phase branch |
| Pull request | number, URL and its state |
| Gate | the last local gate result |
| CI | the last required check result |
| Merge | commit and `mergedAt` when it lands |
| Next step | the one action after this point |
| Stop | the reason and the exact boundary when it stops |

At the start, read the journal; after a crash, a power cut or a session that stopped
answering, rebuild the state from `git` and `gh`, resume from the last completed step and
record the gap. Do not duplicate a commit, a pull request or a merge.

## The loop

One phase at a time, in plan order:

1. **Return to `main`.** `git switch main`, `git merge --ff-only origin/main`, verify
   `git rev-list --left-right --count main...origin/main` is `0 0`.
2. **Branch.** Create the phase branch named in the plan from `main`.
3. **Implement the phase.** Work only on its scope. The plan's checklist is the contract;
   an unplanned item is added to the plan and approved first, or it waits.
4. **Run the gate on the branch.** `./scripts-dev/check.sh` (or `check-docs.sh` for a
   documentation-only phase). Red is fixed here, up to three attempts per failure.
5. **Commit and push.** One signed commit with an English conventional subject and the
   what/why body. Push the branch.
6. **Open the pull request.** Fill the template and the phase checklist from the plan, with
   the claim and its evidence. Do not ask the owner to merge.
7. **Wait for CI.** Every required check must be green: `changes`, `check`, `window` and
   `minimum supported Rust version`. Red is fixed on the same branch and rechecked, up to
   three attempts; a persistent failure stops the run.
8. **Verify the merge state.** `MERGEABLE` with `mergeStateStatus: CLEAN`. A `BEHIND` pull
   request is rebased and rechecked. A conflict, `BLOCKED` or unknown state stops the run;
   the pilot never merges it and never approves on its own behalf.
9. **Merge.** Squash-merge the agent's own pull request under ADR 0046. Verify
   `state: MERGED` with `mergedAt`; a `CLEAN` state alone is not a merge.
10. **Return and verify.** `main` fast-forwarded; run `./scripts-dev/check-docs.sh` and read
    the CI result on `main`. When the phase changed code, run the full
    `./scripts-dev/check.sh`. Red stops the run; the next phase does not start on a broken
    `main`.
11. **Clean.** Delete the merged branch locally and remotely, prune the worktree, mark the
    phase `[x]` on the board, write the report and update the journal.
12. **Next phase.** Repeat until the last delivery phase and the final evaluation and audit
    are complete; then stop.

## Guardrails

- **Merge only in this mode.** The agent squash-merges only its own pull requests, only under
  ADR 0046, only after every required check is green and the pull request is `CLEAN`, and only
  one at a time. It never approves its own pull request, never pushes to `main`, never enables
  GitHub auto-merge and never runs `gh pr merge --auto`.
- **Fix, then stop.** Up to three attempts per failure. After three, stop, leave the branch
  and pull request in their real state, and report. A red or unknown state is never merged.
- **The exceptions stay manual.** Workflows, secrets, dependencies, the ABI removal,
  `AGENTS.md`, an ADR and this policy stop for the owner.
- **One branch at a time.** Do not start the next phase before the previous merge, cleanup and
  post-merge check are complete.
- **The plan is the contract.** The pilot does not add scope, skip a phase or close a phase
  with an item it did not deliver.

## `detener`

Check for `detener` (case-insensitive) at every step boundary. When it appears, stop before
the next irreversible action, leave the branch and pull request in their current real state,
record the boundary in the journal, and ask which of these the owner wants:

- **pause and continue later** — keep the journal and the branch; resume from the journal;
- **stop because the plan changed** — return to the plan, edit it, get it approved again, and
  only then decide whether to resume;
- **abort** — stop the run, leave the open work for the owner and report what exists.

Never guess the reason and never delete work in response to `detener`.

## Report

After each phase, write the table of
[step-report](../step-report/SKILL.md) and keep the plan board:

| Ítem | Estado | Evidencia | Bloqueo |
|---|---|---|---|
| <planned item> | Hecho / En curso / Falta / Bloqueado | test, gate, CI check, PR, merge | <who unblocks it, or —> |

The table does not replace the plan board or OK / Falta / Siguiente. In this mode the report
is written without waiting for the owner; the owner reads it and can still say `detener`.

## Gotchas

- **A plan in the chat is not an approved phased plan.** Without the board and the two
  phases, the answer is the exact message, not a branch.
- **A green local gate is not green CI.** Run both; say which one ran.
- **`CLEAN` is not `MERGED`.** Read `state` and `mergedAt` after the merge.
- **A fix is not a plan change.** A failure is fixed inside the phase; new scope returns to
  the plan and needs approval.
- **The journal is not a substitute for git.** When they disagree, `git` and `gh` win and the
  journal is corrected.
- **`detener` is a stop, not a pause word in the middle of a merge.** At the merge boundary,
  finish the atomic merge, verify it, then stop and ask.
- **The pilot does not merge its own policy.** The PR that changes this skill, `AGENTS.md` or
  an ADR is manual.

## Validation

```bash
git status --short
git rev-list --left-right --count main...origin/main
gh pr view <n> --json state,mergedAt,mergeable,mergeStateStatus
gh pr checks <n>
./scripts-dev/check.sh          # or check-docs.sh for a documentation-only phase
```

The run is healthy when the journal names the same phase, branch, PR and merge state as
`git` and `gh`; every merged phase is `[x]` on the board with its merge commit; no branch or
pull request is left open after a completed phase; and the run stopped with a named reason.
