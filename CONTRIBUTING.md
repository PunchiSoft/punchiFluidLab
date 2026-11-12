# Contributing

Thanks for reading the rules before writing the code. They are short.

**This project is under construction and maintained in someone's free time.** It is
advanced on the free days left by a full-time job, so an issue may wait days and a review
may take a week. That is the pace, not a forgotten pull request: everything is answered,
nothing is merged without the gate. Contributions that arrive with the gate green and the
evidence the rules ask for are the ones that fit that pace; a good report or a
reproduction is as welcome as a patch.

## Set up

```bash
git clone https://github.com/PunchiSoft/punchiFluidLab
cd punchi-fluid-lab
./scripts-dev/check.sh          # full gate: fmt, docs, clippy, tests, CLI smoke test
./scripts-dev/check-docs.sh     # documentation contracts only (index, links, ADRs, labels)
./scripts-dev/test.sh           # tests only; add --no-build to skip Cargo entirely
./scripts-dev/run.sh            # open the window; add --no-build to run the last build
cmake -S apps/punchi-fluid-qt -B apps/punchi-fluid-qt/build -G Ninja   # once, for the window
```

Rust stable, version pinned in `rust-toolchain.toml`, plus Qt 6.5 or newer with Qt Quick,
CMake and Ninja for the window. The Rust core is linked into the Qt binary as a static
library, so the result is one file with no runtime search path to get wrong.

## The gate is not negotiable

Every **code** pull request must leave `./scripts-dev/check.sh` green without silencing
warnings. A **documentation-only** pull request runs the documentation contracts
(`./scripts-dev/check-docs.sh`): the index and links, the ADR metadata, the crate list
against `Cargo.toml` and the issue labels. CI classifies the diff with
`scripts-dev/changed-paths.sh`; it never skips the workflow, because a path-filtered
workflow would leave the required checks pending on protected `main`. The full gate still
runs on `main` and on a release.

A behavioural test may never be deleted, disabled or weakened to make a change pass: if a
test is wrong, fix the test in its own commit and say why.

## What to read first

| If you touch | Read |
|---|---|
| anything | [AGENTS.md](AGENTS.md) |
| a valve, cylinder or symbol file, or the catalog loader | [assets/catalog/README.md](assets/catalog/README.md) and [docs/symbology.md](docs/symbology.md) |
| the window, ribbon, sidebar, colours | [docs/ui-ux.md](docs/ui-ux.md) |
| physics or the solver | `AGENTS.md` → Physics rules |
| memory, dependencies, CI, releases | [docs/engineering.md](docs/engineering.md) |
| a decision that changes a rule | write an ADR in [docs/adr/](docs/adr/) first |

## Symbols follow the standard

A symbol change is not a styling change. It needs the automated `symbol lint` plus the
conformance review from [docs/symbology.md](docs/symbology.md), with light and dark
screenshots attached to the pull request. Symbols are drawn by hand to conform to
ISO 1219-1; nothing is traced from another program, a screenshot or the standard
itself.

## Branches

Never work on `main`: update it, then create one branch per change —
`feat/`, `fix/`, `docs/` or `chore/` followed by a short `kebab-case` description — and
open a pull request from it. `main` is protected, so a direct push is rejected; the
project owner squash-merges the pull request and deletes the branch. Verify the base
rather than assuming it:

```bash
git fetch origin && git switch main && git merge --ff-only origin/main
git switch -c feat/<descriptor>
git rev-list --left-right --count main...HEAD   # 0 <n>
```

## Commits

- Conventional Commits: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`.
- Imperative subject under 72 characters; the body explains the decision, not the
  diff.
- One thing per commit, one thing per pull request. Large mechanical changes
  (formatting, renames) travel alone.
- Sign off your commits (`git commit -s`): this project uses the DCO, not a CLA.

## Pull requests

Fill the template. A reviewer will ask three things: which milestone this belongs to,
what claim you are making, and at which validation level it holds — unit test,
headless UI test, screenshot, measurement, or manual review. "It works" without one of
those is not a description.

Keep pull requests reviewable: under ~400 changed lines outside generated files. If it
must be bigger, explain why in the description. The branch carries the gate result, the
changelog entry and the documentation of what it changed; a follow-up pull request to
"document it later" is not accepted. The full procedure is
[.agents/skills/pull-request-workflow](.agents/skills/pull-request-workflow/SKILL.md).

The project owner is the only one who authorizes and merges a pull request into `main`.
A contributor leaves it open with a green gate and a review answered; an agent does the
same, whatever permissions its session holds, and never merges or enables auto-merge.

## Language

Code, comments, commit messages, documentation and issue text in English. Channel
conversation can be in Spanish. `CHANGELOG.md` is in Spanish, as a project decision.

## Reporting problems

- Bug or feature: open an issue with the form.
- Security: [SECURITY.md](SECURITY.md), never a public issue.
- Behaviour: [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
- Anything else, and what to expect: [SUPPORT.md](SUPPORT.md).
