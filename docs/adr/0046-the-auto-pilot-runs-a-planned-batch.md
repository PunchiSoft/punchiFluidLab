# 0046 — The automatic pilot runs an approved plan under guardrails

- **Status:** accepted
- **Date:** 2026-11-24
- **Deciders:** project owner, implementation session
- **Context crates:** none (the delivery workflow)

## Context

The default delivery rule is that the project owner merges every pull request:
[AGENTS.md](../../AGENTS.md) and
[pull-request-workflow](../../.agents/skills/pull-request-workflow/SKILL.md) say the agent
opens the pull request, answers the review and stops. That default is correct: the merge is
the owner's decision, and no session permission changes it.

[ADR 0043](0043-the-agent-merges-under-guardrails.md) opened a time-bound pilot for the
ribbon plan, with the same guardrails. It proved that an agent can verify green CI, a clean
merge state and a checklist and still merge its own work without losing the repository's
discipline. The owner now wants the same discipline for any explicitly requested batch, not
only for that plan: write the plan, approve it, and let the agent deliver it task by task
without a human merge button at every step.

The risk is not a slow merge; it is an unplanned or unbounded run. An agent that starts
without a plan can invent scope, skip a phase or continue after a red gate. An agent that
keeps merging after a failure can spread a broken change across `main`. The rules below make
the plan the contract and the failure the stop condition.

## Decision

The **automatic pilot** is an explicit delivery mode for one approved
[phased-plan](../../.agents/skills/phased-plan/SKILL.md). It is not a permission granted by
the session, an environment variable or a token scope; it exists only when the owner asks
for it in words and a plan has already been approved.

1. **Trigger and stop.** The mode starts on `piloto automatico`, case-insensitive. It stops
   on `detener`, case-insensitive. Any other wording is a request for the default manual
   mode. The owner can pause or revoke the mode in any message.

2. **The plan gate.** The mode requires an approved phased plan with a phase board and at
   least two delivery phases; each phase names its branch, acceptance tests and checklist.
   The plan is in the owner's private notes or in `docs/` when the owner asked for a public
   one. Without that plan the agent answers exactly:

   `Se necesita planificación previa; no se procede.`

   It does not invent a plan, start a branch or merge anything. It may offer to write the
   plan and wait for approval.

3. **One task at a time.** For each phase, in plan order: return to updated `main`, create
   the phase branch, implement, run the gate, commit, push, open the pull request, verify
   every required check green and `mergeStateStatus: CLEAN`, squash-merge, verify
   `state: MERGED` with `mergedAt`, return to `main`, fast-forward, run the post-merge check,
   delete the branch locally and remotely, mark the phase on the board, write the report and
   continue. A phase that cannot merge alone is split wrong; the plan is corrected, not the
   rule.

4. **The guardrails of ADR 0043 remain.** The agent merges only its own pull requests, one at
   a time, with every required check green (`changes`, `check`, `window`, `minimum supported
   Rust version`), `MERGEABLE` and `CLEAN`, and the full gate run on the branch. It never
   approves on its own behalf, never pushes to `main`, never enables GitHub auto-merge and
   never runs `gh pr merge --auto`.

5. **Fix before stopping.** A red gate or red CI is fixed on the same branch and rechecked,
   up to **three attempts per failure**. A `BEHIND` pull request is rebased and rechecked. If
   the failure persists after three attempts, the merge is blocked, a conflict appears or the
   state is unknown, the mode stops and reports; it does not merge a red or unknown state and
   does not bypass the gate.

6. **The exceptions stay manual.** Changes to `.github/workflows`, secrets, dependency
   additions, the ABI removal, `AGENTS.md`, the ADRs or this delivery policy are never merged
   by the pilot; they stop for the owner. Owner-authored pull requests stay manual.

7. **The post-merge check.** After the merge the agent returns to `main`, fast-forwards it and
   runs `./scripts-dev/check-docs.sh` plus the CI result on `main`; when the phase changed
   code, it runs the full `./scripts-dev/check.sh`. A red post-merge check stops the run and
   is reported; the agent does not start the next phase on a broken `main`.

8. **The journal.** The run keeps a private continuity note,
   `docs-private/auto-pilot-<plan-slug>-journal.md`, with the plan path, approval, current
   phase, branch, pull request, last gate and CI result, merge commit, next step and stop
   reason. It is written after every step and read at the start. After a crash, a power cut or
   a session that stops answering, the agent reconstructs the state from `git` and `gh`,
   resumes from the last completed step without duplicating a commit, pull request or merge,
   and records the gap for the next bug report.

9. **`detener` asks before it continues.** At a safe boundary the agent stops, leaves the
   branch and pull request in their real state, records the exact point, and asks whether the
   owner wants to pause and continue later, stop because the plan changed (the plan returns to
   editing and approval before any resume), or abort. It never guesses the reason.

10. **The end is the plan's end.** When the last phase and the final evaluation and audit are
    complete, the mode stops. It does not start work that is not in the plan. ADR 0043 remains
    the historical record of the ribbon pilot; this ADR governs every explicitly requested
    automatic run and does not rewrite that record.

## Consequences

- The owner stops being the merge button during an approved batch; reading and rejecting the
  work stays theirs and can happen after the merge.
- The plan becomes the real contract: a vague plan produces a vague run, and the agent may
  not fill the gaps. The `phased-plan` gate and the private journal are the controls.
- A failure costs at most three fix attempts and then a stop; a broken `main` stops the run
  instead of feeding the next phase.
- The pilot is auditable: the journal, the phase board, the pull-request checklists and the
  merged commits are the record.
- [AGENTS.md](../../AGENTS.md),
  [pull-request-workflow](../../.agents/skills/pull-request-workflow/SKILL.md),
  [step-report](../../.agents/skills/step-report/SKILL.md) and
  [auto-pilot](../../.agents/skills/auto-pilot/SKILL.md) name this exception; otherwise the
  repository contradicts itself.
- The first automatic run is a practice, not a permanent right: the owner can pause it, and
  the default manual mode returns when the run ends.

## Alternatives considered

- **Keep the owner as the only merge button.** The safer default; it does not answer the
  owner's request for a fully automated, planned batch.
- **Let the session merge without a plan.** Faster to start and impossible to audit; a run
  without a contract is the failure this ADR exists to prevent.
- **Enable GitHub auto-merge.** It merges when checks pass without the final clean-state and
  one-at-a-time verification; the agent running the merge keeps the audit in one place.
- **One long branch with one pull request (`parity-run`).** Easier to review as a whole, but
  it loses the per-task commit, merge, check and cleanup cycle the owner asked for.
- **Generalize ADR 0043 in place.** It would rewrite an accepted record and erase the
  time-bound pilot's history; a new record keeps both.

## References

- [AGENTS.md](../../AGENTS.md) § Branches and delivery, updated by this ADR.
- [pull-request-workflow](../../.agents/skills/pull-request-workflow/SKILL.md).
- [auto-pilot](../../.agents/skills/auto-pilot/SKILL.md).
- [phased-plan](../../.agents/skills/phased-plan/SKILL.md) and
  [step-report](../../.agents/skills/step-report/SKILL.md).
- [ADR 0043](0043-the-agent-merges-under-guardrails.md), the historical pilot.
