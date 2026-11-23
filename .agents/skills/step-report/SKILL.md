---
name: step-report
description: Close a work step with a short checklist for the owner — what is done and proven, what is still missing and who unblocks it, and the one next step. Use when a branch, commit, pull request or phase reaches a hand-off point, or when the owner asks for a status.
license: GPL-3.0-or-later
---

# Step report

A step is finished when it has a gate result, an owner's decision or a merged pull
request, not when the code compiles. The owner reads the report to decide what to
merge, not to reconstruct the session. Three sections, no prose: **OK**, **Falta**,
**Siguiente**.

This is the reporting half of
[pull-request-workflow](../pull-request-workflow/SKILL.md). The route (branch, gate,
pull request, merge) stays there; this skill only says how the hand-off is written.

## Procedure

1. **Name the step and its scope.** One line: the branch, the pull request or the phase,
   and what it was supposed to deliver.
2. **Prove every OK item from the tool, not from memory.** Re-read `gh pr view`,
   `gh pr checks`, `git status` and the gate output. Each bullet names its evidence: the
   test, the gate exit, the check name, the merge state.
3. **List what is missing and who unblocks it.** A pending owner merge, a decision, a
   screenshot that could not be taken headlessly, a deferred theme. Never hide a gap in
   the OK list to make the step look finished.
4. **Name exactly one next step.** What it is, what it depends on, and whether it waits
   for the merge.
5. **Write it in the language of the conversation** (Spanish today). The skill and the
   template stay in English.

## Template

```markdown
## Estado — <step>
**OK**
- <what is done> — <evidence: test / gate / check / merge state>
**Falta**
- <what is missing> — <who unblocks it>
**Siguiente**
- <one step> — <what unblocks it>
```

## Gotchas

- A `CLEAN` pull request is not merged. Only `state: MERGED` with a `mergedAt` closes
  the step.
- A green local gate does not prove CI, and a green CI does not prove the window draws.
  Say which one ran.
- `git branch -d` refuses a squash-merged branch. Prove it has no unique content with
  `git diff <merge-commit> <branch>` before deleting with `-D`.
- A step with nothing visible can still be finished (an ADR, a settings foundation);
  say what it unblocks instead of inventing a screenshot.
- Do not list files as progress. Name behaviour, evidence and the next decision.
- The owner merges. The report ends with the pull request URL and the request to merge,
  never with the agent merging or approving on the owner's behalf.

## Validation

```bash
git --no-pager status --short
git --no-pager log --oneline origin/main..HEAD
gh pr view <n> --json state,mergedAt,mergeStateStatus
gh pr checks <n>
./scripts-dev/check.sh   # check-docs.sh for a documentation-only change
```

The report names the branch and the pull request, every OK item has a command or a
link, every missing item names who unblocks it, and there is exactly one next step.
