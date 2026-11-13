---
name: parity-run
description: Run a long, owner-authorized batch on one branch: pick the next capability from docs/work-queue.md, build it with its test and docs, gate, one signed commit per capability, and one pull request at the end. Use when a parity run is authorized or resumed, or when a fresh session must continue it after a context reset.
license: GPL-3.0-or-later
---

# Parity run

A *parity run* is a long batch of capabilities done without stopping to ask, under an
explicit authorization from the project owner. It exists because a review per slice costs the
owner more than one long branch, and because a session that loses its context must be able to
pick up the work from the repository alone.

The run does not invent the work: [docs/work-queue.md](../../../docs/work-queue.md) is the
queue and [docs/capabilities.md](../../../docs/capabilities.md) is the authority for what a
capability is and how it is proven. This skill is the loop.

## Before the first commit

1. **Confirm the authorization.** The run is option **c** in the owner's words: one long
   branch, one commit per capability, one pull request at the end. Without that sentence in
   the session, use [pull-request-workflow](../pull-request-workflow/SKILL.md) instead.
2. **Prove the base.** `git switch main && git merge --ff-only origin/main`, then
   `git switch -c feat/parity`. The branch is cut once and never rebased onto itself.
3. **Read the continuity note**: [docs/parity-run.md](../../../docs/parity-run.md) — branch,
   queue state, commits done, decisions taken, next step. After a reset it is the first file
   to open.

## The loop, one capability at a time

1. **Take the first `ready` entry** in the queue. Do not reorder by taste; if the order is
   wrong, say so in the report and continue with the next one.
2. **Mark it `doing`** in the queue and in the continuity note, with the date.
3. **Implement in layer order**: catalog file, model, drawing, ABI, window, document format.
   A capability that skips a layer is either not built or faked.
4. **Write the evidence the queue names** — the test that fails without the change — and the
   documentation in the same commit: the capability entry, the ADR when a decision changed,
   the catalog guide when a field was added, `CHANGELOG.md` in Spanish.
5. **Run the gate on the branch**: `./scripts-dev/check.sh`. Green before the commit; a red
   gate is fixed on the same branch, never bypassed.
6. **Commit once**, English conventional subject, body that says what and why, `-s` and the
   co-author trailer.
7. **Update the queue and the continuity note in that same commit**: state `done`, the test
   names, the next entry.
8. **Next capability.** Do not open a pull request yet.

## Stopping

- **A decision belongs to the owner**: leave the entry `blocked`, write the question in the
  continuity note, and continue with the next `ready` entry. Do not guess a teaching rule, a
  licence question or a product scope.
- **Two attempts** at the same capability with the same failing reason: leave it, record the
  reason, move on.
- **The queue is empty**, or only `blocked` entries remain: close the run.

## Closing the run

1. Run the full gate one last time.
2. Update `docs/parity-run.md` with the final state and the report material.
3. Push the branch and open **one** pull request against `main`, with the template filled and
   a section per capability: what was built, the test that proves it, what remains.
4. Hand it over. The owner merges; the agent never merges, enables auto-merge, approves or
   pushes to `main` — not even with option c.

## Gotchas

- **One commit per capability, and the docs travel inside it.** A later "docs for the run"
  commit makes the review impossible to bisect.
- **The gate includes the window.** `--check-qml`, `--theme`, `--icons` and `--selftest` run
  when the build is configured; a Rust-green branch with a broken ribbon is red.
- **Shared documents conflict first.** The queue, the ledger and the changelog are edited by
  every capability; keep the edits small and in the same commit.
- **A file over 800 lines fails the architecture check.** Move the tests out (`#[path]`
  modules) before the gate, not after.
- **The queue is not a wish list.** An entry without an acceptance test is not `ready`.
- **After a context reset**, the repository is the memory: the queue says what is done, the
  continuity note says where the run stands, and `git log` on the branch says what each
  commit built. Never rebuild from a summary of the conversation.

## Validation

```bash
git --no-pager log --oneline main..HEAD            # one commit per capability
git --no-pager status --short                      # clean between capabilities
./scripts-dev/check.sh                             # green on the branch
grep -c "| done |" docs/work-queue.md              # the queue agrees with the log
gh pr view --json baseRefName,headRefName,state    # one PR, base main, state OPEN
```
