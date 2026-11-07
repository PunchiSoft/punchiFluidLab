---
name: rust-testing-and-verification
description: 'Test levels and proof for this workspace: unit, invariant, reference-curve, determinism, allocation and headless UI tests. Use when adding or changing behaviour, fixing a failing test, or deciding what evidence a claim needs.'
license: GPL-3.0-or-later
---

# Testing and verification

A model is not done because it compiles, and a window is not done because it opens.
Every claim in a pull request names the level at which it was proven; this skill maps
a claim to the cheapest test that can falsify it.

## Procedure

1. **State the claim in one sentence.** "The piston never leaves the barrel at either
   end of the stroke", "the exported page keeps its margin", "the solver allocates
   nothing after the first step".
2. **Pick the lowest level that can fail** from the table below. A higher level is
   slower, flakier and less precise about what broke.
3. **Write the test first, with its reference in a comment.** For a physical number,
   write the source of the expected value and the tolerance: "hand calculation from
   ISO 6358, §…, 1 % tolerance". A number with no source is a snapshot of a bug.
4. **Assert the invariant, not the implementation.** Prefer "mass is conserved" and
   "the piston stays inside the barrel" to "this field equals 0.42".
5. **Make it deterministic.** No wall clock, no network, no unseeded randomness, no
   display or GPU dependency in the headless suite.
6. **Run the targeted suite while iterating**, then the whole gate:
   `cargo test -p <crate>` → `./scripts-dev/check.sh`.
7. **Report the count and the level.** The changelog records "N tests, two new", not
   "tests pass".

## Claim → level

| Claim | Level | Where it lives |
|---|---|---|
| A pure function or a parser rule | unit test in the module | `#[cfg(test)] mod tests` |
| A catalog rule (schema, lint) | one test per rule | `crates/punchi-fluid-catalog/src/lint.rs` |
| A format or migration | integration test | `crates/punchi-fluid-file/` (module tests today; `tests/` when a migration needs end-to-end) |
| A physical invariant or reference curve | solver unit test with a stated reference and tolerance | `crates/punchi-fluid-solver/src/pneumatic.rs` |
| A hit area, layout or click | headless UI test | `apps/punchi-fluid-qt` (`--selftest`, offscreen) |
| Exported geometry, margins, determinism | pixel or call-list test | `crates/punchi-fluid-export/tests/` |
| Every built-in has a symbol review | coverage test | `crates/punchi-fluid-catalog/tests/review.rs` |
| Contrast and no colour literals | theme test | `apps/punchi-fluid-qt` (`--check-theme`) |
| The CLI contract end to end | smoke test | `scripts-dev/check.sh` |

## Rules

- **A behavioural test is protected.** Never delete, `#[ignore]`, weaken an assertion or
  narrow an input set to make a change pass. If the test is wrong, fix it in its own
  commit and say why in the message.
- **A failing test is the finding.** Do not relax a tolerance before you know whether
  the model or the expectation is wrong.
- **Golden files change on purpose.** `PUNCHI_FLUID_WRITE_SAMPLE=1 cargo test -p
  punchi-fluid-export --test generate` rewrites the review images; the diff is reviewed
  and attached to the pull request, never regenerated silently.
- **Tests stay readable.** A sentence-case name states the behaviour; the body has one
  reason to fail.
- **`unwrap`/`expect` are allowed in tests** (`clippy.toml` sets
  `allow-unwrap-in-tests`), but a panic in a production path is still a defect.
- **Do not test private layout constants** unless the constant is the contract. Test the
  geometry through the public surface the renderer uses.

## Gotchas

- `cargo test --workspace` does not run `#[ignore]`d tests; an ignored failure is a
  hidden failure.
- A test that needs a window, a GPU or a display belongs outside the headless gate; the
  CI machine has none of them.
- `assert_eq!` on `f64` is wrong for anything that was integrated; use an explicit
  tolerance and justify its size.
- Timing tests are flaky by construction. Assert a bound derived from physics ("stroke
  time lies between the two flow bounds"), not a wall-clock duration.
- A passing test suite on a debug build proves logic, not performance; budgets are
  measured in release, see [rust-performance-and-memory](../rust-performance-and-memory/SKILL.md).
- Coverage is not a target. A line executed by no meaningful assertion is not tested.

## Validation

```bash
cargo test -p <crate>              # the focused loop
cargo test --workspace             # everything, headless
./scripts-dev/check.sh             # fmt, clippy -D warnings, tests, CLI smoke
RUST_BACKTRACE=1 cargo test -p <crate> -- --nocapture   # when a failure needs detail
```

The full gate is the merge criterion. The count it prints goes in the changelog entry,
with the new tests named by the behaviour they protect.

## Sources

The level model mirrors the "claim and evidence" section of
[CONTRIBUTING.md](../../../CONTRIBUTING.md) and
[.github/PULL_REQUEST_TEMPLATE.md](../../../.github/PULL_REQUEST_TEMPLATE.md). The
physics-test rules follow the repository's own contract in
[docs/engineering.md](../../../docs/engineering.md).
