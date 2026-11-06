# Engineering standards

How this codebase stays healthy while it grows into a drawing application. Everything
here is either enforced by a tool or written down so a review can point at it. A rule
that nobody can check is a wish, not a standard.

## Modularisation

### Crates, and what each one may not do

| Crate | Owns | Never contains | Milestone |
|---|---|---|---|
| `punchi-fluid-core` | components, ports, connections, typed units | numerics, files, widgets | M0 |
| `punchi-fluid-catalog` | definition files: schema, validation, loading | physics, widgets | M0 |
| `punchi-fluid-solver` | lumped volumes, ISO 6358, integration, telemetry | editing, UI, file I/O | M0 |
| `punchi-fluid-file` | versioned document format, migrations | physics | M0 |
| `punchi-fluid-draw` | geometry in mm, `Surface` trait, symbol and sheet drawing | windows, I/O | M2 |
| `punchi-fluid-export` | page geometry, PNG and PDF backends | interaction | M2 |
| `punchi-fluid-theme` | semantic tokens, `kdeglobals`, colour-scheme portal | widgets | M1 |
| `punchi-fluid-doc` | document, commands, undo, autosave | drawing, physics | M3 |
| `punchi-fluid-ui` | widgets, ribbon, sidebar, canvas | file I/O, physics | M0 |
| `punchi-fluid-desktop` | window, wiring, menus, accessibility | primitives | M0 |
| `punchi-fluid-cli` | headless validate and simulate | UI | M0 |

Dependency direction is acyclic and enforced by review plus `cargo tree`: `core` ←
`catalog`/`file` ← `solver`/`doc` ← `draw` ← `ui` ← `desktop`. `export` depends on
`draw` and nothing above it. A new edge needs an ADR.

### Extension points

Three, and no more. New behaviour arrives through one of them or not at all:

1. **A catalog file** — a new valve, cylinder, instrument or symbol. No code, no
   release. This is the point of the project.
2. **A `Surface` implementation** — a new drawing backend (egui today, a PDF writer,
   a future QML or SVG front end).
3. **A `behaviour` variant** — only when the physics is genuinely new, with an ADR and
   tests.

Anything else is a change to an existing module, with the review that implies.

## Memory

### Budget

| Item | Budget | How it is held |
|---|---|---|
| Steady-state RSS, empty document, 1080p | < 120 MB | measured at release, recorded in the changelog |
| Symbol geometry cache | ≤ 64 MB, LRU by (definition id, size bucket) | bound in code, test fills it past the limit |
| Undo history | ≤ 100 entries, ≤ 32 MB | `Arc<Document>` snapshots, oldest dropped |
| Telemetry ring | 10 Hz × 60 s × 32 B ≈ 19 KB | fixed capacity, no growth |
| Icon cache | ≤ 200 entries per (size, tint) | LRU |
| Export bitmap | ≤ 80 Mpx (≈ 320 MB), DPI 72–600, one page | refused before allocation |

### Rules

- **No allocation in a hot path**: the solver step and the per-frame draw loop reuse
  buffers (`clear` + `extend`), never grow a `Vec` per call. The simulation state is a
  fixed set of fields.
- **Shared immutables are `Arc`**, never `Rc<RefCell<…>>`; a cycle is a leak, and a
  cell that must be mutated in a frame is a design smell.
- **Every cache has a bound and an eviction rule.** A cache without a bound is a leak
  with a nicer name.
- **No global mutable state.** Theme, catalog and document are owned by the
  application and passed down.
- **`Drop` releases the expensive thing first**: textures and pixmaps before
  bookkeeping.

### Verification

A test binary with a counting global allocator records live bytes and allocation
count after warm-up and after N frames. It asserts:

- live bytes do not grow beyond ±1 % across 5 000 UI frames with no input;
- the simulation loop performs **zero** allocations after the first step;
- filling every cache past its limit keeps total live bytes under the budget.

This catches the class of leak this project is most exposed to: caches keyed by
something that changes every frame, like a zoom level or a layout height in floating
point.

## Performance

| Target | Measurement |
|---|---|
| 60 fps with 200 components | frame CPU time < 6 ms, p95, recorded per milestone |
| Solver step, 200 components | < 4 ms |
| Cold start to first frame | < 400 ms |
| Catalog reload | < 50 ms |
| Export A4 at 300 dpi | < 1.5 s |

Method: a `--trace-frames` flag writes per-frame timings to a CSV; numbers are taken
from that file, never estimated. Optimise after measuring, and record before/after in
the changelog. One optimisation per change, with the measurement in the pull request.

## Security and privacy

### Assets

Circuit documents (the user's work), catalog files, exported files, and the integrity
of the machine. There is no account, no server and no telemetry, so there is nothing
to leak but local content — and nothing may leave the machine.

### Threats and controls

| Threat | Control |
|---|---|
| Malicious catalog or document file | unknown keys rejected, every number range-checked, file ≤ 1 MiB, ≤ 512 nodes, no recursion in the format, no path from the file to the filesystem |
| Huge export exhausting memory | paper whitelist, DPI 72–600, one page, ≤ 80 Mpx, checked before allocating |
| Overwriting a file by accident | atomic write (temp + rename), refuse to overwrite without explicit confirmation, never write outside the chosen path |
| Metadata leaking the machine | no path, user name or host name in PDF metadata; title sanitised of control characters and bounded to 200 characters |
| Symbol files pulled from the network | none: symbols are local files, no download, no update check |
| Dependency compromise | `Cargo.lock` committed, `cargo-deny` (advisories, licences, sources) and `cargo-audit` in CI, dependabot PRs reviewed by a human, no build scripts from untrusted crates without review |
| Release tampering | signed tag, SHA-256 checksums published with the artefacts, reproducible build with `SOURCE_DATE_EPOCH` |
| Font or icon path tricks | read-only access, resolved under `XDG_DATA_DIRS`, no following of symlinks outside the icon roots |

Also: no `unsafe` (workspace lint), no shell-outs, no `eval`, no plugin loading of
native code. Anything that changes this list needs an ADR and a security review.

### Failure behaviour

- A file that cannot be read is reported where it was opened, with the reason.
- A document that references a missing component opens read-only with a placeholder
  and a banner; it never silently drops data.
- The solver that cannot converge stops, says why and keeps the last good state; it
  never returns NaN to the UI.
- Panics are denied outside tests. A panic that escapes is a bug with a test to write,
  not a policy.

## Validation

| Level | Proves | Does not prove |
|---|---|---|
| `cargo test --workspace` | domain, physics, format, catalog, geometry | that anything looks right |
| Headless UI tests (egui `Context::run_ui`) | hit areas, click routing, layout maths | visual quality |
| Export tests (pixels, call lists) | margins, paper size, determinism | that the symbol is legible |
| `check.sh` | the gate above plus the CLI | the window, the theme, the packaging |
| Screenshot review per milestone | what the user actually sees | interaction over time |
| Symbol conformance review | the symbol matches the standard | that the physics is right |
| Reference curves and hand calculations | the physics | everything else |

A claim in a pull request names the level it rests on. "Works" without a level is not
a claim.

## GitHub practices

### Files that must exist

| File | Purpose |
|---|---|
| `README.md`, `README.es.md` | what it is, how to run it, where to read more |
| `CONTRIBUTING.md` | setup, gate, commit and review conventions |
| `SECURITY.md` | supported versions, private reporting, scope |
| `CODE_OF_CONDUCT.md` | expected behaviour and how to report |
| `LICENSE` | GPL-3.0-or-later |
| `CHANGELOG.md` | dated entries under `Unreleased` |
| `AGENTS.md` | rules for people and agents |
| `.github/ISSUE_TEMPLATE/*` | bug and feature forms, with a security pointer |
| `.github/PULL_REQUEST_TEMPLATE.md` | the checklist a reviewer can hold the author to |
| `.github/CODEOWNERS` | who must review a change in each area |
| `.github/dependabot.yml` | weekly cargo and Actions updates |
| `.github/workflows/ci.yml` | the gate on every push and pull request |
| `.github/workflows/audit.yml` | `cargo-deny` and `cargo-audit` on a schedule |
| `.github/workflows/release.yml` | tag → build, checksums, draft release |
| `.agents/skills/README.md` | skill catalog: procedures for Rust, GitHub and symbology |
| `deny.toml` | licence allow-list and advisory policy |
| `.editorconfig`, `rustfmt.toml`, `clippy.toml` | formatting and lint configuration |

### Branching and review

- `main` is protected: pull request required, one review, CI green, linear history,
  no force push.
- Short-lived branches named `feat/…`, `fix/…`, `docs/…`, `chore/…`.
- Conventional Commits (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`),
  imperative subject under 72 characters, body explaining the decision.
- Commits carry `Signed-off-by` (DCO). No merge commits on `main`.
- A pull request changes one thing and says which milestone it belongs to, which
  claims it makes and at which validation level.
- Large mechanical changes (formatting, renames) travel alone.

### Releases

1. `Cargo.toml` version, both READMEs, `AGENTS.md` and the changelog entry updated in
   one commit: `chore(release): 0.x.y`.
2. Signed tag `v0.x.y`.
3. `release.yml` builds, runs the gate again, produces the archive, AppImage and
   checksums, and opens a draft release.
4. The draft lists the changelog section, the known limitations and the validation
   level reached (for example "symbols verified against ISO 1219-1" only when true).
5. After publishing, `Unreleased` is emptied and the next milestone starts.

### Labels and milestones

- Type: `bug`, `feature`, `docs`, `security`, `performance`, `a11y`, `i18n`, `packaging`.
- Area: `area:catalog`, `area:solver`, `area:drawing`, `area:export`, `area:theme`,
  `area:editor`, `area:ci`.
- Triage: `needs-reproduction`, `needs-decision`, `good first issue`, `help wanted`.
- Milestones mirror [plan.md](plan.md): `M1 shell`, `M1.5 symbology`, `M2 export`, …
  An issue without a milestone is not scheduled; a milestone without acceptance
  criteria is not a milestone.

## Definition of done

A change is done when all of these are true:

1. `./scripts-dev/check.sh` is green and no warning was silenced to get there.
2. Tests exist for the behaviour, and they fail if the behaviour is removed.
3. The claim is stated with its evidence: tests, measurement, screenshot or review.
4. Documentation updated in the same change: `CHANGELOG.md`, and the README, ADR or
   spec if the change touches behaviour, a decision or the interface.
5. No new dependency without a licence check (`cargo-deny`) and a reason in the pull
   request.
6. Memory and performance budgets still hold, or the change explains why they moved.
