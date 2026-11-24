# Engineering standards

How this codebase stays healthy while it grows into a drawing application. Everything
here is either enforced by a tool or written down so a review can point at it. A rule
that nobody can check is a wish, not a standard.

## Modularisation

### Crates, and what each one may not do

| Crate | Owns | Never contains | Milestone |
|---|---|---|---|
| `punchi-fluid-core` | components, ports, connections, typed units, paper and the legacy v1 model | numerics, files, widgets | M0 |
| `punchi-fluid-catalog` | definition files: schema, validation, loading, lint | physics, widgets | M0 |
| `punchi-fluid-solver` | lumped volumes, ISO 6358, integration, telemetry | editing, UI, file I/O | M0 |
| `punchi-fluid-file` | versioned document format, migrations, recent list | physics | M0 |
| `punchi-fluid-draw` | geometry in mm, `Surface` trait, symbol and sheet drawing | windows, I/O | M2 |
| `punchi-fluid-export` | page geometry, PNG, PDF and SVG backends | interaction | M2 |
| `punchi-fluid-ffi` | the retired C ABI of the Qt window; no product consumer | product logic | retired |
| `punchi-fluid-doc` | document, commands, undo, sheet | drawing, physics | M3 |
| `punchi-fluid-check` | design checks: the faults a circuit can have before it runs | physics, UI | M3 |
| `punchi-fluid-session` | sheet↔document, live model, recents, safety copy, templates | physics, widgets | M4 |
| `punchi-fluid-cli` | headless check, validate and simulate | UI | M0 |
| `punchi-fluid-window` | the egui window: ribbon, sidebar, sheet, status bar, start view, theme | physics, the document | M5 |

The window is **`apps/punchi-fluid`** (package `punchi-fluid-window`), a Rust binary in
the workspace built with egui (ADR 0035, ADR 0036). It owns the ribbon, the sidebar, the
sheet, the status bar, the start view and the theme adapter, and it links `doc`, `draw`,
`export`, `file` and `session` directly. It does not speak an ABI and does not depend on
the retired Qt app.

Dependency direction is acyclic and enforced by `scripts-dev/check-architecture.sh` plus
`cargo tree`: `core` ← `catalog`/`file` ← `solver`/`doc`/`session` ← `draw` ← `export` ←
`ffi` (retired); the window depends on `catalog`, `doc`, `draw`, `export`, `file` and
`session`. `cli` depends on `file`, `catalog`, `check`, `doc`, `draw` and `solver`. A new
edge needs an ADR.

### Extension points

Three, and no more. New behaviour arrives through one of them or not at all:

1. **A catalog file** — a new valve, cylinder, instrument or symbol. No code, no
   release. This is the point of the project.
2. **A `Surface` implementation** — a new drawing backend (the egui replay today, the
   PNG, PDF and SVG writers).
3. **A `behaviour` variant** — only when the physics is genuinely new, with an ADR and
   tests.

Anything else is a change to an existing module, with the review that implies.

### The C ABI retires

`punchi-fluid-ffi` and its `include/punchi_fluid.h` served the Qt Quick window. The Qt
window is gone and the Rust window links the crates directly, so the ABI has no product
consumer: only its own 96 tests and an untracked spike still reference it. ADR 0035 retires
it. The removal is a code change of its own: inventory those tests, port the unique ones to
`session`/the window, then delete the crate, the header, the `unsafe` exemption and the
`BIG_ALLOWED` entry. Until then it is dead weight, not an extension point.

## Memory

### Budget

| Item | Budget | How it is held |
|---|---|---|
| Steady-state RSS, empty document, 1080p | < 120 MB | measured at release, recorded in the changelog |
| Symbol geometry cache | ≤ 64 MB, LRU by (definition id, size bucket) | bound in code, test fills it past the limit |
| Undo history | ≤ 100 entries, ≤ 32 MB | `Arc<Document>` snapshots, oldest dropped |
| Telemetry ring | 10 Hz × 60 s × 32 B ≈ 19 KB | fixed capacity, no growth |
| Icon family | 55 compiled-in SVGs, no runtime cache | `include_bytes!` in `ribbon/icons.rs` |
| Start-view previews | rendered on demand, bounded by the four templates | no retained bitmap cache |
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

Method: `./scripts-dev/run.sh --release -- --trace-frames` writes the complete frame's
CPU duration as CSV on standard output. Redirect it to a file for analysis. Samples are
buffered in fixed batches of 256 and the final partial batch is flushed when the window
closes. Numbers are taken from that CSV, never estimated. Optimise after measuring, and
record before/after in the changelog. One optimisation per change, with the measurement
in the pull request.

## Security and privacy

### Assets

Circuit documents (the user's work), catalog files, exported files, and the integrity
of the machine. There is no account, no server and no telemetry, so there is nothing
to leak but local content — and nothing may leave the machine.

### Threats and controls

| Threat | Control |
|---|---|
| Malicious catalog or document file | unknown keys rejected, every number range-checked, a sheet read only from a regular file and through a bounded reader, file ≤ 8 MiB, ≤ 2 000 placements, ≤ 4 000 pipes, ≤ 64 circuits, no recursion in the format, no path from the file to the filesystem |
| Huge export exhausting memory | paper whitelist, DPI 72–600, one page, ≤ 80 Mpx, checked before allocating |
| Overwriting a file by accident | atomic write (temp + rename), the temporary created with `create_new` and never through a planted symlink, the document's permissions kept, flushed before the rename and removed on failure; refuse to overwrite without explicit confirmation, never write outside the chosen path |
| Metadata leaking the machine | no path, user name or host name in PDF metadata; title sanitised of control characters and bounded to 200 characters |
| Symbol files pulled from the network | none: symbols are local files, no download, no update check |
| Dependency compromise | `Cargo.lock` committed, `cargo-deny` (advisories, licences, sources) and `cargo-audit` in CI, dependabot PRs reviewed by a human, no build scripts from untrusted crates without review |
| Release tampering | signed tag, SHA-256 checksums published with the artefacts, reproducible build with `SOURCE_DATE_EPOCH` |
| Font or icon path tricks | read-only access, resolved under `XDG_DATA_DIRS`, no following of symlinks outside the icon roots |

Also: no `unsafe` in the product (workspace lint), no shell-outs, no `eval`, no plugin
loading of native code. The retired `punchi-fluid-ffi` crate is the only place that opts
out, and it has no product consumer; removing it removes the workspace's last `unsafe`.
The two fixed links in the About panel ask the desktop to open the repository or its
issue tracker **when the user clicks them**. The program runs no command of its own, never
takes a URL from a file and never makes a network request. Anything that changes this
list needs an ADR and a security review.

**Known gap:** the sheet parser still accepts four values that are only checked for being
finite, and the export's own atomic write uses `fs::write` on a predictable temporary
instead of the hardened writer in `punchi-fluid-file`. Both are recorded in the private
node register and must close with tests.

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
| Headless UI tests (`cargo test -p punchi-fluid-window`, egui_kittest) | hit areas, click routing, layout maths, the theme contrast | visual quality |
| Export tests (pixels, call lists) | margins, paper size, determinism | that the symbol is legible |
| `check.sh` | the gate above plus the CLI | the window, the theme, the packaging |
| Screenshot review per milestone | what the user actually sees | interaction over time |
| Symbol conformance review | the symbol matches the standard | that the physics is right |
| Reference curves and hand calculations | the physics | everything else |

A claim in a pull request names the level it rests on. "Works" without a level is not
a claim.

The skill catalog is data the model loads at runtime: a malformed frontmatter makes a skill
disappear without failing a build. `scripts-dev/check-skills.sh` parses every `SKILL.md`,
matches it with its catalog row and resolves its links; `check-docs.sh` runs it on the real
tree, and `test-check-skills.sh` proves the failure modes.

## GitHub practices

### Where the community files live

Two places, and the rule is what a person reads against what a machine finds.

| Place | What goes there | Why |
|---|---|---|
| Repository root | `README.md`, `README.es.md`, `LICENSE`, `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, `SUPPORT.md`, `CHANGELOG.md`, `AGENTS.md` | GitHub links them from the interface and a person looks for them here first. GitHub also accepts `.github/` and `docs/`, so moving them buys nothing: the community health score is the same and every relative link breaks |
| `.github/` | `ISSUE_TEMPLATE/`, `PULL_REQUEST_TEMPLATE.md`, `CODEOWNERS`, `dependabot.yml`, `FUNDING.yml`, `workflows/` | Configuration that GitHub reads from a fixed path and that nobody opens to read |

Three rules follow from that:

- **A new community file is added in three places**: this table, the documentation table of
  both READMEs, and `scripts-dev/check-docs.sh`.
- **A link to something the repository does not have is a defect, not an intention.** The
  issue form once offered a Discussions link on a repository with Discussions disabled;
  the check now fails on that.
- **The health score is not the goal.** It has been 100 % while the issue forms pointed at
  labels that did not exist. The goal is that someone who wants to report, ask or
  contribute finds the right door.

### The gate and a documentation-only pull request

`main` is protected, so every required check must report on every pull request. A
workflow skipped by path filtering leaves its checks in `Pending` and blocks the merge
(GitHub documents this), so `ci.yml` is never skipped by `paths` or `paths-ignore`.
Instead:

- a `changes` job classifies the diff with `scripts-dev/changed-paths.sh`; and
- `check` always runs: a documentation-only change runs `check-docs.sh`, a code change runs
  the full `check.sh`; and
- `window` and `minimum supported Rust version` are skipped for documentation-only changes.
  A job that runs and is skipped reports success, which the branch protection accepts.

A documentation-only change is an **allow-list**: the pages under `docs/`, the community
files, the captures and the authoring guides. Anything else is code. In particular, the
catalog TOML files, the sound samples and the symbol reviews are **compiled in or observed
by tests**, so they are code even when they are next to a Markdown file. An unknown path is
code, never documentation. A push to `main` and a manual run always run the full gate.

`scripts-dev/test-changed-paths.sh` exercises the classifier in `check.sh`, because a wrong
`docs` answer would skip the tests on a change they can observe.

### Delivery modes

Manual is the default: the owner merges. A code change starts only after the owner approved
its plan; a one-pull-request change uses a short plan, two or more use the formal
[phased-plan](../.agents/skills/phased-plan/SKILL.md). The hand-off carries the report table
of [step-report](../.agents/skills/step-report/SKILL.md).

The owner may ask for `piloto automatico`; the run follows
[auto-pilot](../.agents/skills/auto-pilot/SKILL.md) and
[ADR 0046](../docs/adr/0046-the-auto-pilot-runs-a-planned-batch.md). It needs an approved
phased plan with a board and at least two phases, and it stops on a red gate after three
attempts, a blocked or unknown merge, an exception, `detener` or the end of the plan. The
branch, gate and checklist rules are those of
[pull-request-workflow](../.agents/skills/pull-request-workflow/SKILL.md).

### Files that must exist

| File | Purpose |
|---|---|
| `README.md`, `README.es.md` | what it is, how to run it, where to read more |
| `CONTRIBUTING.md` | setup, gate, commit and review conventions |
| `SECURITY.md` | supported versions, private reporting, scope |
| `CODE_OF_CONDUCT.md` | adopted by reference (Contributor Covenant 2.1); scope and reporting here |
| `SUPPORT.md` | where to ask, and what this project does not promise |
| `LICENSE` | GPL-3.0-or-later |
| `CHANGELOG.md` | dated entries under `Unreleased` |
| `AGENTS.md` | rules for people and agents |
| `.github/ISSUE_TEMPLATE/*` | bug and feature forms, with a security pointer, using labels that exist |
| `.github/PULL_REQUEST_TEMPLATE.md` | the checklist a reviewer can hold the author to |
| `.github/CODEOWNERS` | who must review a change in each area |
| `.github/labels.txt` | the labels that exist, so a template cannot name one that does not |
| `.github/FUNDING.yml` | the optional donation link, and nothing that costs the user |
| `.github/dependabot.yml` | weekly cargo and Actions updates |
| `.github/workflows/ci.yml` | the full gate on a code change and on `main`; the documentation contracts on a docs-only pull request |
| `.github/workflows/audit.yml` | `cargo-deny` and `cargo-audit` on a schedule |
| `.github/workflows/release.yml` | tag → build, checksums, draft release |
| `.agents/skills/README.md` | skill catalog: procedures for Rust, GitHub and symbology |
| `deny.toml` | licence allow-list and advisory policy |
| `.editorconfig`, `clippy.toml` | formatting and lint configuration |

### Branching and review

- `main` is protected: pull request required, one review, CI green, linear history,
  no force push.
- The project owner is the only one who authorizes and merges a pull request into
  `main`; an agent opens it, answers the review and stops there.
- Short-lived branches named `feat/…`, `fix/…`, `docs/…`, `chore/…`.
- Conventional Commits (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`), English
  imperative subject under 72 characters, English body explaining the decision.
- Commits carry `Signed-off-by` (DCO). No merge commits on `main`.
- A pull request changes one thing and says which milestone it belongs to, which
  claims it makes and at which validation level.
- Large mechanical changes (formatting, renames) travel alone.

### Releases

1. `Cargo.toml` version, both READMEs, `AGENTS.md` and the changelog entry updated in
   one commit: `chore(release): 0.x.y`.
2. Signed tag `v0.x.y`.
3. `release.yml` builds, runs the gate again, produces the archive and checksums, and
   opens a draft release. **It still targets the retired Qt app and must be updated before a
   tag**; that is a named debt, not a working release path.
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

1. `./scripts-dev/check.sh` is green (or `./scripts-dev/check-docs.sh` for a
   documentation-only change) and no warning was silenced to get there.
2. Tests exist for the behaviour, and they fail if the behaviour is removed.
3. The claim is stated with its evidence: tests, measurement, screenshot or review.
4. Documentation updated in the same change: `CHANGELOG.md`, and the README, ADR or
   spec if the change touches behaviour, a decision or the interface.
5. No new dependency without a licence check (`cargo-deny`) and a reason in the pull
   request.
6. Memory and performance budgets still hold, or the change explains why they moved.
