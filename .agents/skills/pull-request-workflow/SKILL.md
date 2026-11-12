---
name: pull-request-workflow
description: Work on a branch created from updated main and deliver it through a pull request that the project owner merges. Use when starting any change, creating or switching a branch, committing, pushing, or opening or updating a pull request.
license: GPL-3.0-or-later
---

# Pull request workflow

`main` is what the project ships and what CI watches on every push. A commit made
straight on it is a change nobody reviewed, a gate that ran after the fact, and a
history that cannot be dropped without rewriting what others may already have pulled.
So the rule is one line:

> Nothing is edited on `main`. Every change is a branch created from updated `main` and
> lands as a pull request.

And the second, which no permission overrides:

> **The merge belongs to the project owner, never to the agent.** A session stops when
> the pull request is open, the gate is green and the review is answered. Authorizing
> and merging it into `main` is a human act: no token scope, admin role, bot account or
> explicit instruction in a session changes that. If a session is asked to merge, it
> reports the pull request URL and hands the decision back.

One branch per change, one pull request per branch, squash-merged by the owner, and the
branch deleted afterwards. A second unrelated change in the same branch makes the review
unreadable and the revert impossible; if it appears halfway, finish the first, deliver
it, and start again from the new `main`.

## The branch

The name is `feat/<descriptor>`, `fix/<descriptor>`, `docs/<descriptor>` or
`chore/<descriptor>`, in `kebab-case`, and it says what the change does, not who made
it or which day it was. Do not reuse a branch that was already merged.

A sub-branch — one created from another feature branch instead of from `main` — is an
exception and needs a reason written down:

- the parent is unpublished and this change cannot merge without it, or
- two independent changes must be reviewed separately before either merges.

Otherwise branch from `main` directly. A branch stacked on unpublished work by habit
attaches commits that are not yours, and the mistake stays invisible until review.

## Procedure

1. **Read the state before touching anything.**

   ```bash
   git --no-pager status --short
   git --no-pager branch --show-current
   git --no-pager log --oneline -1
   ```

   Uncommitted changes that are not yours stop the procedure: a switch carries them to
   the new branch.

2. **Prove the base.** A branch name says nothing about its base; Git does.

   ```bash
   git fetch origin --quiet
   git --no-pager rev-list --left-right --count main...origin/main   # expect: 0 0
   ```

   A nonzero left number means local `main` is behind: fast-forward it
   (`git switch main && git merge --ff-only origin/main`) before branching. A failing
   fast-forward means local `main` has commits of its own, which is the case this skill
   exists to prevent; stop and report it.

3. **Branch from `main`.**

   ```bash
   git switch -c feat/<descriptor> main
   git --no-pager rev-list --left-right --count main...HEAD   # expect: 0 0
   git --no-pager log --oneline origin/main..HEAD             # expect: empty
   ```

4. **Work, committing on the branch.** Conventional Commits, English imperative subject
   under 72 characters, English body, `git commit -s`, and the `Co-authored-by:` trailer
   when an agent co-authored. One thing per commit; formatting and renames travel alone.

5. **Run the gate on the branch that will be merged**, not on a memory of a green run:

   ```bash
   ./scripts-dev/check.sh
   ```

   A red gate stops the delivery. Fix it on this branch, in its own commit when the fix
   is unrelated to the claimed change.

6. **Push the branch and open the pull request.**

   ```bash
   git push -u origin <branch>
   gh pr create --base main --head <branch> --body-file /tmp/pr.md
   ```

   Fill every section of
   [.github/PULL_REQUEST_TEMPLATE.md](../../../.github/PULL_REQUEST_TEMPLATE.md) in
   English: what changes, the milestone or phase, the claim and its evidence level, the
   checklist, and **known limitations** — reviewers read that part first. The body
   describes behaviour, never a file list. Write it to a temporary file so the formatting
   survives, and delete the file afterwards.

7. **Verify the pull request is what was reviewed.**

   ```bash
   gh pr view --json url,mergeable,mergeStateStatus,baseRefName,headRefName,files
   gh pr checks --watch
   ```

   The base is `main`, the head is the branch, the changed files are the claimed scope.
   When `main` moved underneath, rebase onto `origin/main` and push with
   `--force-with-lease`; never a merge commit from the branch and never a plain
   `--force`.

8. **Hand the pull request over, and stop.** With the base, the changed files and the
   checks verified, report the URL, the gate result and what the review has to look at,
   and leave the pull request open. The owner authorizes and merges it; the agent never
   runs `gh pr merge`, `gh pr merge --auto`, `gh pr review --approve`, `gh pr merge
   --admin` or a direct push to `main`, whatever permissions the session was given. An
   instruction to merge is answered with the URL and a request for the owner to merge
   it. A failing check is fixed on the branch, never bypassed.

## Gotchas

- **A clean `git status` says nothing about the base.** Only step 2 does.
- **The checkout can move under you.** Two sessions — or a session and a hand — in the
  same working copy share one `HEAD`: a `git switch` in one of them silently changes the
  branch the other is committing to, and the commit lands on `main` with a green gate and
  no warning. Read `git branch --show-current` immediately before committing, and
  `git log --oneline -1 origin/main..HEAD` as the proof the commit is on the branch. A
  commit that landed on `main` by mistake is moved, not reverted: `git branch
  salvage-<name> <sha>` keeps it reachable, `git reset --hard origin/main` puts `main`
  back, the branch is reset onto the salvage and a `--force-with-lease` push publishes it.
- **The window is part of the gate.** A Rust-green branch whose ribbon overflows or
  whose `--selftest` fails is red: `check.sh` builds and drives the window when it is
  configured.
- **`main` is protected, but that is not the rule.** [Branch
  protection](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches)
  requires a pull request, the `check`, `window` and `minimum supported Rust version`
  jobs, and linear history, so a direct push is rejected rather than merely frowned
  upon. The rule that the agent does not merge holds even where protection is missing,
  misconfigured or bypassable: if a push to `main` succeeds, the protection is the bug
  and the owner fixes it — the agent still does not use it. Check it with
  `gh api repos/{owner}/{repo}/branches/main/protection`.
- **A permission is not an authorization.** A token with `merge` scope, an owner role or
  a bot identity lets the agent call `gh pr merge`; it does not make the merge the
  agent's to perform. The owner is the only one who authorizes and merges, so the
  command is never issued and never suggested as the next step.
- **The delivery branch is not a scratch pad.** Ignored build output
  (`apps/punchi-fluid-qt/build`, `target/`), editor files and session notes do not belong
  in it; `git status --short` before the first commit catches them.
- **A branch that was squash-merged has no commits in `main`.** Compare content, not
  commits, before treating an old branch as a base: it is already delivered.
- **The changelog and the docs travel with the change**, not in a later pull request:
  `CHANGELOG.md` under `Unreleased`, the ADR when a decision changed, the catalog guide
  when a field was added. Reviewers reject "documented separately".
- **A branch whose pull request is still open is not a base for the next phase.** If the
  next phase touches the same files, it waits for the merge; if it does not, it branches
  from `main` and lands on its own.

## Coordination

[github-collaboration](github-collaboration/SKILL.md) owns the content rules — commit
form, review, issues, releases. This skill owns the route: branch, gate, pull request,
hand-over. The merge stays with the project owner. A capability itself is still gated by
[capability-plan](../capability-plan/SKILL.md) and the parity ledger; the branch is how
it travels, not permission to start it.

## Validation

```bash
git --no-pager rev-list --left-right --count main...HEAD   # 0 <n>, never on main
git --no-pager log --oneline origin/main..HEAD             # the commits being delivered
./scripts-dev/check.sh                                     # green on this branch
gh pr view --json baseRefName,headRefName,mergeStateStatus,state   # state: OPEN
gh pr checks
```

Report the branch name, the base it was created from, the gate result and the pull
request URL, and end with the merge left to the owner. "Pushed to `main`" and "merged by
the agent" are not valid results of this procedure.
