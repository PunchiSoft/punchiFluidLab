# Contributing

Thanks for reading the rules before writing the code. They are short.

## Set up

```bash
git clone https://github.com/PunchiSoft/punchi-fluid-lab
cd punchi-fluid-lab
./scripts-dev/check.sh          # fmt, clippy -D warnings, tests, CLI smoke test
./scripts-dev/test.sh           # tests only; add --no-build to skip Cargo entirely
./scripts-dev/run.sh            # open the window; add --no-build to run the last build
cargo run -p punchi-fluid-desktop
```

Rust stable, version pinned in `rust-toolchain.toml`. Nothing else is needed to
build; the window uses `glow` on Wayland and X11.

## The gate is not negotiable

Every pull request must leave `./scripts-dev/check.sh` green without silencing
warnings. A behavioural test may never be deleted, disabled or weakened to make a
change pass: if a test is wrong, fix the test in its own commit and say why.

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
must be bigger, explain why in the description.

## Language

Code, comments, commit messages, documentation and issue text in English. Channel
conversation can be in Spanish. `CHANGELOG.md` is in Spanish, as a project decision.

## Reporting problems

- Bug or feature: open an issue with the form.
- Security: [SECURITY.md](SECURITY.md), never a public issue.
- Behaviour: [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
