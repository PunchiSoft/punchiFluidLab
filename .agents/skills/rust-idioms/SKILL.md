---
name: rust-idioms
description: 'Idiomatic Rust for this workspace: ownership, typed errors, units and lints under a pinned MSRV. Use when writing, reviewing, refactoring or debugging Rust code, designing a public API, or fixing clippy and rustfmt findings.'
license: GPL-3.0-or-later
---

# Rust idioms in Punchi Fluid Lab

[AGENTS.md](../../../AGENTS.md) states the rules; this skill is the working procedure
for the language half of them. The workspace is edition 2024, MSRV 1.85, `unsafe` is
forbidden and `unwrap`/`expect`/`panic!` are denied outside tests.

## Procedure

1. **Place the change first.** Read the crate's row in AGENTS.md and
   [docs/engineering.md](../../../docs/engineering.md); a type in the wrong crate is a
   refactor, not a detail.
2. **Name the quantity.** If a value is a pressure, temperature, volume, length or
   time, look for its type in `punchi-fluid-core::units` before writing a field.
   A bare `f64` for a physical quantity is a bug, even inside one crate.
3. **Make the invalid state unrepresentable.** Prefer an enum over a bool flag, a
   newtype over a validated `String`/`f64`, and a fallible constructor (`new` returning
   `Result`) over a struct users can populate wrongly. Validate once at the boundary.
4. **Model failure as data.** Every recoverable failure is a variant of a `thiserror`
   enum in the crate that owns the operation. Callers match or propagate; they do not
   print and continue.
5. **Borrow first, share second.** Try a named `let`, a shorter block, `mem::take` or
   splitting the struct before cloning. Reach for `Rc`/`Arc` only when shared ownership
   is the design, and for `RefCell`/`Mutex` only when interior mutability is.
6. **Write the public surface deliberately.** Default to `pub(crate)`. A public item
   needs a doc comment, a `# Errors` section when fallible, and a `# Panics` section
   only when it really can panic (it normally cannot here).
7. **Run the gate before handing back.** `./scripts-dev/check.sh` is the definition of
   done; see Validation below for the fast loop.

## Patterns this workspace uses

```rust
#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    #[error("unknown key `{key}` in {file}")]
    UnknownKey { file: String, key: String },
    #[error("port `{port}` is outside the symbol box")]
    PortOutsideBox { port: String },
}
```

- Return `Option<T>` only when absence is a normal outcome: a lookup that can miss.
  When absence means the caller made a mistake, return a named error.
- Validate numbers at the boundary (`is_finite`, range) and keep the interior total.
  `orifice::mass_flow` returning `None` for an out-of-range input is the house style.
- Use `#[expect(clippy::name, reason = "...")]` on the narrowest item when a lint is
  genuinely wrong. Never add a blanket `#![allow(...)]`.
- Derive `Debug` on public types: `missing_debug_implementations` is a workspace warning.
- Linelike data that crosses the file format is `serde` with explicit field names;
  unknown keys are rejected by the loader, so do not add `#[serde(default)]` to hide a
  typo.

## Gotchas

- **MSRV 1.85 is a promise, not a suggestion.** `let` chains stabilized in 1.88 and are
  not available here. So are APIs newer than 1.85, even if the local `rustc` accepts
  them. `clippy.toml` sets `msrv = "1.85"`; keep it and `Cargo.toml` in sync.
- **Do not clone to silence the borrow checker.** A clone added for that reason is
  usually a lost write later. See step 5.
- **`expect` is denied too**, not only `unwrap`. In tests both are allowed through
  `clippy.toml`; in production, propagate or fall back to a documented default.
- **No `#![deny(warnings)]` in source.** A new toolchain would break the build; CI runs
  clippy with `-D warnings` instead.
- **Edition 2024 changed temporary scopes and `impl Trait` capture.** If a change
  suddenly needs a `let` binding to keep a value alive, that is the edition rule, not a
  compiler bug.
- **`f64` equality** is only acceptable for constants and exact state copies. Compare
  measured values with an explicit tolerance and say where the tolerance comes from.
- **A doc comment is not decoration.** `cargo doc --workspace --no-deps` catches broken
  intra-doc links, and the `# Errors` section is the contract a caller reads.

## Related skills

- Crate boundaries: [rust-workspace-architecture](../rust-workspace-architecture/SKILL.md)
- Proving the behaviour: [rust-testing-and-verification](../rust-testing-and-verification/SKILL.md)
- New crates and features: [rust-dependencies-and-supply-chain](../rust-dependencies-and-supply-chain/SKILL.md)

## Validation

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
```

`./scripts-dev/check.sh` runs the first three plus the CLI smoke test. When a change can
plausibly use a newer API, prove the MSRV too:

```bash
cargo +1.85.0 build --workspace --locked
```

A green local `rustc` 1.98 run does not prove the 1.85 promise; the `msrv` CI job does.

## Sources

Distilled from the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/),
the [Google Rust Style Guide](https://google.github.io/styleguide/rustguide.html),
[Effective Rust](https://www.lurklurk.org/effective-rust/), the
[Rust Design Patterns](https://rust-unofficial.github.io/patterns/) catalogue and the
[Microsoft Pragmatic Rust Guidelines](https://microsoft.github.io/rust-guidelines/),
filtered through this repository's own rules. No text from those guides is vendored
here.
