# 0043 — The agent merges its own pull requests under guardrails

- **Status:** accepted
- **Date:** 2026-11-23
- **Deciders:** project owner, implementation session
- **Context crates:** none (the delivery workflow)

## Context

The delivery rule is that the project owner merges every pull request: [AGENTS.md](../../AGENTS.md)
and [pull-request-workflow](../../.agents/skills/pull-request-workflow/SKILL.md) say the agent
opens the pull request, answers the review and stops. That rule is right as a default: the
merge is the owner's decision, and no session permission changes it.

The owner has asked to test the other mode on the plan in progress. The ribbon repair has
three phases and a final audit left; each one is a branch, a gate and a pull request. Waiting
for a human at every merge turns a reviewed plan into a slow one, and the owner wants to see
whether an agent that verifies green CI, the clean merge state and the phase checklist can
merge its own work without the repository losing its discipline.

The pilot is a practice, not a permanent right: it is time-bound, revocable and audited.

## Decision

1. **Scope.** Only pull requests authored by the agent in this repository, only while the
   pilot is active. Owner-authored pull requests, releases, changes to `.github/workflows`,
   secrets, dependency additions, the ABI removal and any change to this ADR stay manual.
2. **Green and clean.** The agent merges only when every required check on the head commit is
   green (`changes`, `check`, `window`, `minimum supported Rust version`), the pull request is
   `MERGEABLE` with `mergeStateStatus: CLEAN`, and the full gate has run on the branch. A
   branch that is `BEHIND` is rebased and rechecked; a red, conflicting or unknown state stops
   the merge and the owner is told.
3. **One at a time.** One agent pull request open at a time. The merge is a squash, the branch
   is deleted locally and remotely afterwards, `main` is fast-forwarded, and the plan board
   marks the phase only after the merge.
4. **This record is manual.** The pull request that introduces this ADR is merged by the
   owner, like every change to the delivery policy; the pilot starts with the next agent pull
   request.
5. **Revocable and time-bound.** The owner may pause or revoke the pilot in any message; the
   agent stops merging the next time it would. The pilot ends when the ribbon plan's phases 7
   and 8 and the final audit are delivered, and the owner reviews it then.
6. **The audit is part of the delivery.** Each merged pull request records the checks that
   were green, the merge commit and its phase; the plan board and the step report carry the
   same state. A missing record is a process bug, not a detail.

## Consequences

- The owner stops being the merge button for these phases; reading and rejecting the work
  stays theirs and can happen after the merge.
- The agent gains the ability to break `main` with a change that passes the gate; the gate,
  the clean-state check, the one-at-a-time rule and the manual exceptions are the only
  controls. The pilot is limited to the ribbon plan for that reason.
- [AGENTS.md](../../AGENTS.md) and
  [pull-request-workflow](../../.agents/skills/pull-request-workflow/SKILL.md) must name this
  exception; otherwise the repository contradicts itself, and the same change updates both.
- The merge is a command the agent runs after verifying the checks; the agent does not enable
  GitHub auto-merge and never pushes to `main`.

## Alternatives considered

- **Keep the owner as the only merge button.** The default and the safer option; it does not
  answer the owner's request to test the pilot.
- **GitHub auto-merge enabled by the agent.** It would merge when the checks pass without a
  final read of the merge state; the agent verifying and running the merge keeps the
  clean-state gate and the audit in one place.
- **A bot account with merge rights.** The same power under a second identity and with no line
  in the plan; the session's own identity is easier to revoke.

## References

- [AGENTS.md](../../AGENTS.md) § Branches and delivery, updated by this ADR.
- [pull-request-workflow](../../.agents/skills/pull-request-workflow/SKILL.md), updated by
  this ADR.
- The ribbon plan in the owner's private notes.
