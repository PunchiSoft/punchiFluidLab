---
name: rust-dependencies-and-supply-chain
description: Adding and auditing Cargo dependencies under the licence allowlist, MSRV and deny rules. Use when a crate is proposed, a dependabot PR lands, Cargo.toml changes, or a build fails on the pinned toolchain.
license: GPL-3.0-or-later
---

# Dependencies and supply chain

A dependency is code shipped to every user under this project's GPL-3.0-or-later
licence. [deny.toml](../../../deny.toml) is the machine-checkable part of the policy;
this skill is the review that comes before it.

## Procedure: adding a crate

1. **Justify it.** Can `std`, an existing dependency, or twenty lines of owned code do
   the job? A new crate needs a reason in the pull request, not just a working build.
2. **Check the licence** against the allowlist in `deny.toml` and against
   GPL-3.0-or-later compatibility. A licence not on the list is a decision for the
   maintainer, not an edit to the list.
3. **Check the crate's own constraints**: `rust-version` ≤ 1.85, `unsafe` policy,
   build scripts, native/system libraries, feature flags. A crate that needs OpenSSL is
   banned outright (`openssl-sys`).
4. **Minimise the surface.** Add it to `[workspace.dependencies]` with
   `default-features = false` and the smallest feature set that works. Crates refer to
   it with `{ workspace = true }` or a path; versions are never wildcards.
5. **Check the graph.** `cargo tree -d` for duplicate major versions, `cargo tree -i
   <crate>` for who pulls it in, and `cargo deny check` for advisories, licences,
   bans and sources.
6. **Commit `Cargo.lock`** in the same change. This repository ships a lockfile; an
   application without one is not reproducible.
7. **Build the pinned toolchain** and the release profile:
   `cargo build --workspace --locked` and `cargo +1.85.0 build --workspace --locked`.
8. **Record the decision.** Pull request: crate, version, purpose, licence, why the
   existing set could not do it. A dependency update follows the same path, with
   `chore(deps)` in the commit.

## Rules

- **No git dependencies, no unknown registries, no wildcards.** `deny.toml` denies all
  three. A patch section needs an ADR-level reason.
- **Build scripts are reviewed like code.** A `build.rs` runs at build time with the
  developer's permissions; it is not a detail of a dependency.
- **Features are additive.** Never enable a feature that changes behaviour for other
  users of the same crate; this is an application, but the rule keeps the graph sane.
- **Dependabot PRs are read before merging.** Groups are configured for `egui`,
  serialization and dev dependencies; a green CI run is not a review.
- **MSRV is part of the supply chain.** A dependency update that raises the floor
  breaks the `msrv` job and is a project decision, not a routine bump.

## Gotchas

- **`cargo deny check` needs the tool.** Install it with `cargo install cargo-deny
  --locked` (the audit workflow does the same) or report that the check is missing; do
  not silently skip it.
- **`cargo audit` needs the advisory database** and network access. The audit workflow
  runs it in CI; a local offline run may be stale.
- **Transitive licences count.** The allowlist applies to the whole graph, including
  dev-dependencies, because `all-features = true` in `deny.toml`.
- **`Cargo.lock` is not hand-edited.** Change the manifest, then let Cargo update it.
- **A `cargo update` is a change.** It moves many versions at once; keep it in its own
  pull request and read the diff.
- **Feature unification** can turn on a heavy feature through an unrelated dependency;
  check with `cargo tree -e features`.

## Validation

```bash
cargo deny check
cargo audit
cargo tree -d
cargo build --workspace --locked
cargo build --workspace --release --locked
cargo +1.85.0 build --workspace --locked
./scripts-dev/check.sh
```

`cargo deny check` is the gate; the `msrv` workflow job is the promise. If a tool is not
available locally, say so and rely on CI rather than claiming the check passed.

## Sources

- [Cargo Book: workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
  and [dependency resolution](https://doc.rust-lang.org/cargo/reference/resolver.html)
- [`cargo-deny` book](https://embarkstudios.github.io/cargo-deny/)
- [RustSec advisory database](https://rustsec.org/)
- [GitHub Actions hardening](../github-actions-security/SKILL.md) for the CI half of
  the same threat model
