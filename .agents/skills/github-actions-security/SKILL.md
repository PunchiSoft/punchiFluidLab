---
name: github-actions-security
description: Threat model and hardening checklist for .github/workflows. Use when writing or reviewing a workflow, touching permissions, secrets, caching, third-party actions, or a release pipeline.
license: GPL-3.0-or-later
---

# GitHub Actions security

Workflows are YAML that becomes shell with a token attached. The dangerous parts are
the trigger, the token scope, the interpolation and the actions you did not write.

## Procedure

Review or author a workflow in this order:

1. **Triggers.** Prefer `pull_request` over `pull_request_target`. A `pull_request_target`
   job runs with the base repository's token and secrets and checks out the base branch;
   if it also checks out the pull request head and runs it, the fork owns your token.
   `workflow_run` and `issue_comment` carry the same escalation risk.
2. **Permissions.** Start from `permissions: contents: read`; grant at job level, not
   workflow level, and only what the job uses. `write-all` is a defect. A release job
   needs `contents: write` to create a release; the build job does not.
3. **Interpolation.** `${{ ... }}` is expanded into the script before the shell parses
   it. Never put attacker-influenced text (issue title, PR body, branch name, commit
   message, input) inside `run:`. Pass it through `env:` and quote the variable:

   ```yaml
   - env:
       TITLE: ${{ github.event.issue.title }}
     run: printf '%s\n' "$TITLE"
   ```

4. **Action references.** Pin third-party actions to a full commit SHA with the version
   in a comment; a tag is mutable. First-party actions are not automatically safe
   either. `uses: owner/action@v4` is a moving target.
5. **Checkout and credentials.** Set `persist-credentials: false` unless a later step
   needs the token to push, and prefer an explicit `token:` there. Do not check out
   untrusted code in a job that holds secrets.
6. **Inputs and environment files.** `workflow_dispatch` inputs are strings from a
   human; validate them. Never write untrusted content to `$GITHUB_ENV`,
   `$GITHUB_OUTPUT` or `$GITHUB_PATH` (newline injection).
7. **Caches and artefacts.** A cache restored from a fork branch can contain executable
   build output. Do not cache `target/` between trust boundaries without a key that
   includes the toolchain and lockfile; treat downloaded artefacts as untrusted.
8. **Secrets and OIDC.** No long-lived cloud keys in repository secrets when OIDC is
   available. Never echo a secret, and never pass one to a `pull_request_target` job
   that runs fork code.
9. **Release integrity.** Build from a protected tag, use `--locked`, set
   `SOURCE_DATE_EPOCH`, publish checksums, and attach provenance (`actions/attest-build-
   provenance`) when available. Protect the tag; a tag can be moved.
10. **Bounds.** Set `timeout-minutes`, use `concurrency` with `cancel-in-progress` for
    CI, and never give a public repository a self-hosted runner that runs fork code.

## State of this repository

Audit these facts when the skill is used; they are intentionally not a claim that the
workflows are perfect:

- `.github/workflows/ci.yml`: `permissions: contents: read`, `persist-credentials:
  false`, `checkout` pinned by SHA. `dtolnay/rust-toolchain@stable` and
  `Swatinem/rust-cache@v2` are mutable tags — residual risk to pin.
- `.github/workflows/audit.yml`: read-only, `cargo install cargo-deny --locked` without
  a pinned version — the installed version changes over time.
- `.github/workflows/release.yml`: workflow-level `contents: write` applies to the build
  job too; scope it to `publish`. `upload-artifact@v4` and `download-artifact@v4` are
  major tags. No provenance attestation yet; the draft release uses `--verify-tag`.

Treat the list as a review prompt, not as a to-do you may change without a pull request.

## Gotchas

- **`pull_request_target` plus a checkout of `github.event.pull_request.head.sha`** is
  the classic remote-code-execution pattern. It looks like a normal test job.
- **A tag is not a commit.** Pinning `@v4` protects against nothing if the tag moves;
  pin the SHA and keep the human-readable version in a comment.
- **First-party actions can be compromised too.** Pin them when the cost is low.
- **`${{ }}` inside `run:` is the injection**, even inside double quotes in the YAML;
  the quotes belong to the script, not to the expansion.
- **A fork PR runs with a read-only token**, which is why `pull_request` is the safe
  trigger; the moment you add `pull_request_target` for a label or a comment, you have
  changed the trust model.
- **Cache keys are shared within a branch scope**; a poisoned cache can survive a
  force-push. Do not cache executable output across trust boundaries.
- **`workflow_dispatch` is not a secret boundary.** Anyone with write access can run it
  with repository secrets; do not let an input choose a path or a command.
- **`permissions:` at workflow level is inherited by every job.** A missing job-level
  block is a silent grant.
- **Artifacts are not signed.** `download-artifact` gives you bytes, not provenance.

## Validation

```bash
# Where interpolation could reach a shell
grep -n '\${{' .github/workflows/*.yml

# If the tools are installed; they are not part of the runtime dependencies
actionlint .github/workflows/*.yml
zizmor .github/workflows

gh workflow list
gh run list --limit 10
```

Then walk the ten steps above against each file and record the findings in the pull
request. `actionlint` and `zizmor` catch syntax and some patterns; the trust-boundary
review is still human.

## Sources

- [GitHub Docs: security hardening for GitHub Actions](https://docs.github.com/en/actions/security-for-github-actions/security-guides/security-hardening-for-github-actions)
- [GitHub Docs: `pull_request_target` and fork security](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#pull_request_target)
- [OpenSSF Scorecard](https://github.com/ossf/scorecard) and
  [SLSA](https://slsa.dev/) for the supply-chain half
- The `github-actions-hardening` skill in
  [github/awesome-copilot](https://github.com/github/awesome-copilot) covers the same
  threat model in more depth; this file is the repository-specific version.
