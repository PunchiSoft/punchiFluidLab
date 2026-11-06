---
name: github-collaboration
description: Commit, pull request, review, issue and release practice for this repository. Use when committing, opening or reviewing a PR, triaging an issue, cutting a release, or updating the changelog.
license: GPL-3.0-or-later
---

# GitHub collaboration

[CONTRIBUTING.md](../../../CONTRIBUTING.md) and the pull request template are the
contract. This skill is the order in which to do it and the places where reviews
actually fail.

## Commits

- **Conventional Commits**: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `perf:`,
  `build:`, `ci:`, `chore:`, `revert:`. Scope is optional but useful:
  `fix(solver): keep the cylinder inside its stroke`.
- Imperative subject under 72 characters; the body explains the decision, not the diff.
- One thing per commit. Formatting, renames and dependency updates travel alone.
- **Sign off** (`git commit -s`, DCO). When an agent co-authors, add the
  `Co-authored-by:` trailer.
- Reference the issue in the pull request description (`Closes #123`), not as a
  substitute for the explanation.

```bash
git status --short
git diff --cached
git commit -s -m "fix(catalog): reject a port outside the symbol box" -m "The loader accepted..."
```

## Pull requests

1. **Fill every section of the template**: what changes, milestone, claim and evidence,
   checklist, known limitations. Reviewers read "known limitations" first.
2. **Name the evidence level**: unit test, headless UI test, pixel/call-list test,
   measurement, screenshot, symbol conformance review. "It works" is not a level.
3. **Keep it reviewable**: under ~400 changed lines outside generated files. A larger
   change explains why in the description and lands in reviewable commits.
4. **Run the gate before pushing**: `./scripts-dev/check.sh`. A red CI run costs a
   review round.
5. **Update what the change touches**: `CHANGELOG.md` under `Unreleased`, the README or
   ADR if behaviour or a decision changed, `assets/catalog/README.md` for a new field.
6. **Screenshots for anything visual**, light and dark. For a symbol, the conformance
   review from [docs/symbology.md](../../../docs/symbology.md) is mandatory.
7. **New dependency**: licence checked, reason in the description.

## Review

- Block only on correctness, security, a broken contract, or a missing proof; style is
  rustfmt/clippy's job.
- Ask for the failed test first. A behavioural test may not be deleted or weakened to
  make a change pass; a wrong test is fixed in its own commit.
- Check the budget table when the change touches the solver, the draw loop, a cache or
  the export.
- Check the layer: a file-I/O call in `punchi-fluid-ui` or a physics formula in
  `punchi-fluid-core` is a design finding even when tests pass.

## Issues

- Bug or feature: the issue forms under `.github/ISSUE_TEMPLATE/`. Include the version
  or commit, the platform, a minimal circuit file and what you observed.
- Security: [SECURITY.md](../../../SECURITY.md) and GitHub private advisories. Never a
  public issue, never a real document with private work attached.
- A behaviour problem: [CODE_OF_CONDUCT.md](../../../CODE_OF_CONDUCT.md).

## Releases

1. Everything under `## Unreleased` in [CHANGELOG.md](../../../CHANGELOG.md) is already
   in dated `### <hito> — YYYY-MM-DD` subsections, in Spanish. Do not rewrite history
   to make a release look tidy.
2. Bump `workspace.package.version` in `Cargo.toml`, both READMEs and `AGENTS.md` in one
   change. Documentation, tests and tooling alone never justify a bump.
3. Update `Cargo.lock` with the version change.
4. Tag a signed commit `vX.Y.Z`; the release workflow runs only from a `v*` tag.
5. Verify the published artefacts: checksums, `SOURCE_DATE_EPOCH` reproducibility, and
   the notes matching the changelog.

## Repository settings to keep

These live in GitHub, not in the repository; check them when onboarding or after an
incident:

- `main` protected: pull request required, `check` and `minimum supported Rust version`
  required, CODEOWNERS review, conversation resolution, linear history, no force push,
  no deletion.
- Dependabot alerts and security updates enabled; Actions default token read-only.
- Secret scanning and push protection enabled; private vulnerability reporting enabled.
- Release tags protected; release environment with required reviewers.
- Immutable releases enabled when available, so a published artefact cannot be swapped.
- OpenSSF Scorecard or an equivalent supply-chain check reviewed periodically.

## Gotchas

- **Do not merge with admin overrides** to beat a flaky job; fix or re-run the job.
- **Do not rewrite `main`.** Force-push is for your own branch before review only.
- **Do not add a commit trailer or a licence header by hand** if a template or tool
  already does it; the diff should show the decision, not boilerplate churn.
- **The changelog is Spanish** by project decision even though code and documentation
  are English. Channel conversation can be Spanish.
- **A generated file is not a source of truth.** `Cargo.lock`, sample images and review
  artefacts are regenerated, then reviewed as a diff.

## Validation

```bash
./scripts-dev/check.sh
gh pr checks
gh pr view --json title,body,state,reviewDecision
gh release view vX.Y.Z
```

For branch protection, the API is the evidence:

```bash
gh api repos/{owner}/{repo}/branches/main/protection
```

## Sources

- [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/)
- [DCO 1.1](https://developercertificate.org/)
- [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/) and
  [Semantic Versioning 2.0.0](https://semver.org/)
- [GitHub docs: about protected branches](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches)
- [OpenSSF Scorecard](https://github.com/ossf/scorecard)
