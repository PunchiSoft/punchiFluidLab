# Project instructions: Punchi Fluid Lab

This file is canonical for this repository. Skills live in `.agents/skills/` and
supplement it; user instructions take precedence. The catalog and the reading order are
in [.agents/skills/README.md](.agents/skills/README.md).

**Resuming work with no context:** start at [docs/README.md](docs/README.md), the index that
says which document is current, target or historical, then
[docs/capabilities.md](docs/capabilities.md) for the product and the test that proves each
capability.

The owner keeps the continuity note, the work order, the node register and the other working
notes in a local, untracked `docs-private/`, written in Spanish and versioned in their own
repository. They are not part of this repository, and a public document never links to them.

**Before a capability: [docs/capabilities.md](docs/capabilities.md)** is the plan — what a
student must be able to do, how it behaves, and the test that proves it. The authority is
the standard and the teaching goal, never another product; the procedure is
[.agents/skills/capability-plan](.agents/skills/capability-plan/SKILL.md).

Where the rest lives: [docs/README.md](docs/README.md) is the index and says which document
is current, target or historical. In short, [docs/architecture.md](docs/architecture.md)
(the system map), [docs/ui-ux.md](docs/ui-ux.md) (interface),
[docs/symbology.md](docs/symbology.md) (ISO 1219 conformance),
[docs/plan.md](docs/plan.md) (milestones and acceptance criteria),
[docs/engineering.md](docs/engineering.md) (memory, security, GitHub practices),
[docs/adr/README.md](docs/adr/README.md) (the decision index) and
[assets/catalog/README.md](assets/catalog/README.md) (authoring a component).

## Scope and product

A desktop simulator of pneumatic and hydraulic circuits: draw a schematic, connect
ports, run the model, watch the actuators move. Free and open, built so that a student with
no licence and no budget can learn the subject. **No asset, symbol set, manual, example
circuit or file format from any other program enters this repository.**

Status: `0.1.0`, unreleased, **under construction** — advanced in the maintainer's free
time, one capability at a time, with no promised date. A capability that is `missing` or
`partial` in [docs/capabilities.md](docs/capabilities.md) is not there yet, and nothing may
pretend otherwise. There is an editor, a network solver, an export and the first slice of the electrical
library with its boolean control model; the export crate can build a parts list but the
window does not print one yet. There is no hydraulic model yet; the rest of the electrical,
digital and GRAFCET libraries are missing, and the 300 dpi symbol review is still open.

## Language

English is the source language for code, identifiers, comments, logs, tests, scripts,
commit subjects and bodies, pull-request titles and bodies, documentation, skills and the
README. The exceptions are:

- **user-facing interface strings**, which are keys in the translation catalogs under
  `i18n/`: English is the source and the default, Spanish a selectable translation
  ([ADR 0045](docs/adr/0045-the-interface-speaks-english-through-embedded-toml-catalogs.md)).
  The Spanish literals that remain are migration debt, not a second source;
- **`CHANGELOG.md`**, in Spanish by project decision;
- **`README.es.md`**, the Spanish copy of the README.

Conversation may be in Spanish. A Spanish UI label may be quoted inside an English comment
(`Edición ▸ Girar`, `Mayús`), but the sentence around it is English. Internal diagnostics
and test names are English. No Spanish prose in identifiers, comments,
logs or tests.

## Request modes and the plan gate

- **Plan before code.** A request that changes code — a feature, a fix, a failing test, a
  refactor — starts with an implementation plan approved by the owner. A question that only
  asks for information does not need one; if it turns into a change, the plan comes first.
  - One pull request: a short plan in the conversation that names what changes, the files, the
    evidence and the risk, approved before editing.
  - Two or more pull requests: the formal
    [phased-plan](.agents/skills/phased-plan/SKILL.md), with Phase 0, a phase board and a
    checklist per phase, approved before Phase 1.
  - No code, stash, commit or push before the owner approves the plan.
- **Manual mode is the default.** The owner merges. The hand-off carries the report table of
  [step-report](.agents/skills/step-report/SKILL.md) and asks for the merge.
- **Automatic pilot.** The owner may ask for `piloto automatico` (case-insensitive). It runs
  only an approved `phased-plan` with a board and at least two phases; without one the agent
  answers exactly: `Se necesita planificación previa; no se procede.` and waits. It follows
  [auto-pilot](.agents/skills/auto-pilot/SKILL.md) and
  [ADR 0046](docs/adr/0046-the-auto-pilot-runs-a-planned-batch.md): one phase at a time, the
  gate before the commit, every required CI check green and `mergeStateStatus: CLEAN` before
  the squash merge, up to three fix attempts, the post-merge check, the cleanup and the next
  phase. The agent merges only under that mode and its guardrails; it never approves on its
  own behalf, pushes to `main` or enables GitHub auto-merge. Changes to workflows, secrets,
  dependencies, the ABI removal, this file, an ADR or this policy stay manual. The owner stops
  it with `detener` (case-insensitive); the run records its state in the private journal.
- **The report.** In manual mode, between pull requests, the hand-off includes the table
  `Ítem | Estado | Evidencia | Bloqueo` plus the plan board and OK / Falta / Siguiente. In
  automatic mode the same table is written after each merged phase; it does not replace the
  plan board.

## Stack

Rust, stable toolchain pinned in `rust-toolchain.toml`. See
[docs/adr/0001-language-and-layout.md](docs/adr/0001-language-and-layout.md) for the
decision and its consequences; changing it needs a new ADR, not a silent edit.

## Workspace layout

| Crate | Owns | Never contains |
|---|---|---|
| `crates/punchi-fluid-core` | components, ports, connections, units, paper and the sheet frame | numerics, files, widgets |
| `crates/punchi-fluid-i18n` | the language, the embedded TOML catalogs and the key lookup | physics, widgets, files, UI |
| `crates/punchi-fluid-catalog` | component definition files: schema, validation, loading | physics, widgets |
| `crates/punchi-fluid-solver` | lumped-parameter model, orifice flow, integration | editing, UI |
| `crates/punchi-fluid-file` | versioned documents: the sheet format and its migrations | physics |
| `crates/punchi-fluid-draw` | the single drawing of a circuit, in millimetres, behind a `Surface` trait | windows, files, fonts, solving |
| `crates/punchi-fluid-doc` | the edited document: placements, commands, bounded undo | physics, drawing, files |
| `crates/punchi-fluid-check` | design faults a circuit can have before it is run | physics, drawing, UI |
| `crates/punchi-fluid-export` | standard sheets, title block, the PNG, PDF and SVG writers | physics, widgets |
| `crates/punchi-fluid-ffi` | the retired C ABI of the Qt window; no product consumer (ADR 0035; removal is a debt) | product logic |
| `crates/punchi-fluid-session` | the document's life between the file and the window: the sheet↔document conversion, the live model of a circuit and the editor state as it moves | physics, widgets |
| `crates/punchi-fluid-cli` | headless check, validate and simulate | UI |
| `apps/punchi-fluid` | the Rust window: ribbon, sidebar, sheet, status bar, start view and theme adapter, with egui (ADR 0035, ADR 0036) | physics, the document |

Dependency direction: `core` is the base; `i18n` depends on no crate of ours and is the
translation base; `catalog` and `file` depend on `core` and on no other crate of ours; `solver` depends on `core` and `catalog`; `draw` on `catalog`,
`core` and `solver`; `doc` on `core` and `draw` (the sheet is a domain value the document
stores); `check` on `catalog`, `doc` and `draw` (it reads
port geometry from `draw`, because that is where a millimetre lives); `export` on
`core`, `catalog`, `draw` and `solver` (the paper and the frame are domain values, so the
page reads them from `core`); `session` on `core`, `catalog`, `check`, `doc`, `draw`, `file` and `solver` (the sheet format, the live document, the design check and the model that runs it meet there; the physics stays in the solver);
`ffi` retires with the Qt window and has no product consumer; `cli` on `core`, `catalog`,
`check`, `doc`, `draw`, `file` and `solver`, because it runs the same check and the same
model without a window. Nothing in Rust depends on a window. The Rust window in
`apps/punchi-fluid` depends on `core`, `doc`, `draw`, `export`, `session`, `catalog`, `file`
and `i18n` — the solver reaches it through the session's model — with no ABI ([ADR 0035](docs/adr/0035-the-window-is-rust.md),
[ADR 0036](docs/adr/0036-the-toolkit-is-egui.md)). A new dependency direction needs an ADR;
the `Surface` layer that made `draw` and `export` possible is
[ADR 0003](docs/adr/0003-one-drawing-two-devices.md). `scripts-dev/check-docs.sh` fails if
this table and `Cargo.toml` disagree.

Non-code directories: `assets/catalog/` (the component definitions and their guide),
`assets/symbols/` (ISO 1219-1 reference), `docs/` (ADRs and the record), `i18n/`
(translations), `packaging/` (AppImage, Flatpak, Windows), `scripts-dev/` (developer
tooling, never a runtime dependency).

## Code rules

- **Units are typed.** Pressures, temperatures and volumes go through
  `punchi-fluid-core::units`; a bare `f64` crossing a crate boundary for a physical
  quantity is a bug. Store SI internally, accept the unit the user thinks in.
- **No `unwrap`, `expect` or `panic!` outside tests.** The workspace lints deny
  them. Recoverable failures are `thiserror` enums; a value that cannot be trusted
  falls back to a documented default instead of aborting.
- **No `unsafe`.** Forbidden by the workspace lints. The retired `punchi-fluid-ffi` crate is
  the one exception, it has no product consumer, and removing it is a named debt.
- **The domain is total.** A component kind knows its ports; a circuit validates
  connections; the file format never produces an invalid domain object silently.
- **Files are versioned.** Any change to the document shape bumps `FORMAT_VERSION`
  and adds a migration test.
- Comments explain a decision or a trap, never restate the line, and are in English.
- Every source file starts with the SPDX headers used in this repository.

## Component catalog rules

- **A component is a file, never a Rust type.** No `match` on a valve id, no
  `Valve3_2` struct, no hardcoded geometry. See
  [docs/adr/0002-components-are-files.md](docs/adr/0002-components-are-files.md).
- Behaviour is expressed with **port roles and open paths**, so an unknown valve is a
  new file and not a new code path. A new `behaviour` variant is the only exception,
  and it needs an ADR.
- A malformed file is **skipped and reported**, never fatal: the simulator must start
  with whatever is valid.
- Built-in definitions live in `assets/catalog/` and are compiled in with
  `include_str!`; the same text is what a user copies to override one.
- The catalog is validated once, in `punchi-fluid-catalog`, with a test per rule. Do
  not duplicate those checks in the solver or the UI.
- New field: document it in `assets/catalog/README.md` in the same change.

## Symbology rules

- Symbols follow **ISO 1219-1:2012**, in the diagram conventions of ISO 1219-2:2012
  and ISO 1219-3:2016. The contract, the guarantees and the review checklist are in
  [docs/symbology.md](docs/symbology.md).
- **The owner's reference is classified; another program's source and assets may not be
  stored.** A normative standard figure, or an engineering drawing with the owner's
  dimensions, may be read for geometry and measures. A capture of another application or a
  photograph identifies the component, its function and its arrangement; it is not traced or
  copied. The reconstruction is written as our TOML; the reference image is not stored, and
  the review records what was observed, what the owner required, what the standard states and
  what is an assumption ([ADR 0040](docs/adr/0040-the-owner-reference-authorizes-the-reconstruction.md)).
  Another program's source code or assets never enter the repository, and `check-docs.sh`
  keeps failing on those names.
- **The symbols are normative, not ours.** They are the standard's, one form in every
  application: no variant or transformation of a symbol of ours, and no change based on
  another program. An owner-provided reference may give the intended form and the measures
  (ADR 0040). The sheet's connection points are the diagram's rendering, not a symbol
  variant; the library panel shows the symbol without them.
- A symbol change needs a **conformance review**: the automated `symbol lint` plus the
  manual checklist, with light/dark screenshots attached to the pull request.
- Normative details are not negotiable for taste: box proportions, arrow direction,
  port numerals, blocking marks, spring and actuator ends, line classes and junction
  dots all come from the standard.
- When a normative question is open, the code says so and the export labels the
  document as unverified. A plausible-looking diagram that is not conformant is the
  worst outcome this project can ship.

## Physics rules

- Pneumatics is a **network of lumped volumes joined by restrictions**, not CFD.
- Orifice flow follows ISO 6358 (`punchi-fluid-solver::orifice`), including the
  choked branch: above the critical ratio the flow stops growing.
- A model is not "done" because it compiles: it needs invariants (mass
  conservation, pressures never negative) and a reference curve or a hand
  calculation. State the reference and the tolerance in the test.
- The solver never touches the UI and never reads files; it receives validated
  definitions and reads numbers from them, so a conductance is a value in a file.
- A definition-driven model is still a model: invariants and reference tests use the
  numbers from the definition, never a copy of them.

## Security and privacy

- Circuit files are untrusted input: parse defensively, bound sizes, never index
  with unvalidated data.
  Every limit has a number, a message and a one-past-it test in the crate that owns
  the data ([.agents/skills/untrusted-input-bounds](.agents/skills/untrusted-input-bounds/SKILL.md)).
- No telemetry, no network at runtime, no shell-outs.
- `assets/` may only contain files whose licence and origin are recorded; a reference
  provided by the owner may guide the reconstruction, but no reference image, figure or text
  enters `assets/`.
- Catalog files are untrusted input too: they are parsed with unknown keys rejected,
  every number is range-checked, and no file can make the loader allocate or loop
  beyond a bound.
- A change that touches a file, a path, an export, the clipboard, autosave or a link is
  reviewed against the threat table in [docs/engineering.md](docs/engineering.md) with
  [.agents/skills/security-review](.agents/skills/security-review/SKILL.md): the surface,
  the control, the hostile case and the record.

## Interface rules

- The interface is specified in [docs/ui-ux.md](docs/ui-ux.md). Three regions: ribbon,
  sidebar that mirrors the catalog, sheet. Adding a fourth needs an ADR.
- **Colour, size and font come from the theme adapter**
  (`apps/punchi-fluid/src/theme.rs`), never from a literal in a widget. The theme's own
  tests measure the contrast and fail below the criterion.
- **The desktop is the default appearance**: the window reads `$XDG_CONFIG_HOME/kdeglobals`
  once at start for the palette, accent, font and motion, and a machine without it gets the
  light fallback; no file is read per frame. The appearance setting may choose a built-in
  theme instead ([ADR 0042](docs/adr/0042-the-appearance-is-a-setting.md)), and every theme
  is a `Palette` seed through the same adapter — no colour literal in a widget.
- Components are shown by **their own symbol**, never by a separate icon set.
- State is never colour-only, hover never changes the document, and there is no modal
  on the normal path.
- Accessibility is part of the definition of done: keyboard reachable, contrast
  measured, names exposed to the accessibility layer, reduced motion honoured.

## Memory and performance rules

- Budgets and their verification are in [docs/engineering.md](docs/engineering.md).
- No allocation in the solver step or the per-frame draw loop; reuse buffers.
- Every cache has a bound and an eviction rule; a cache without one is a leak. The procedure is
  [.agents/skills/untrusted-input-bounds](.agents/skills/untrusted-input-bounds/SKILL.md).
- `Arc` for shared immutables, never `Rc<RefCell<…>>`, no global mutable state.
- A widget and its state have one owner, and images, models and caches do not grow while
  the window is idle.
- A change that moves a budget says so in the pull request, with the measurement.

## Validation

```bash
./scripts-dev/check.sh
```

Builds run with two cargo jobs and every test binary under a memory ceiling derived from
this machine, because a single runaway allocation has taken the whole session down before.
The profile is [.cargo/config.toml](.cargo/config.toml) and
[scripts-dev/limits.sh](scripts-dev/limits.sh); the procedure for diagnosing a kill is
[.agents/skills/machine-resource-budget](.agents/skills/machine-resource-budget/SKILL.md).

It runs `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
`scripts-dev/test.sh` (which covers the workspace), the documentation drift check
`scripts-dev/check-docs.sh`, the architecture contract `scripts-dev/check-architecture.sh`
and a CLI smoke test over a temporary circuit file.

CI runs that full gate for a code change. A **documentation-only** pull request runs
`scripts-dev/check-docs.sh` instead: the index and links, the ADR metadata, the crate list
against `Cargo.toml` and the issue labels. The workflow is never skipped by path filtering,
because a skipped workflow leaves required checks pending on protected `main`;
`scripts-dev/changed-paths.sh` classifies the diff and is tested in the gate. The full gate
still runs on `main` and on a release.

The window's tests run headlessly through `cargo test -p punchi-fluid-window`
(`egui_kittest`), and CI has a `window` job that builds the binary and runs that suite. The
retired `--check-qml`, `--check-theme`, `--check-icons`, `--check-layout` and `--selftest`
steps belonged to the Qt app and are not part of this product.

During a session, `./scripts-dev/test.sh` runs the same tests and
`./scripts-dev/run.sh` opens the window; both compile only when a source changed and
accept `--no-build` to execute the previous build directly. `./scripts-dev/dev.sh` is the
live loop: it watches the sources, rebuilds what changed and relaunches the window; the
menu's first entry and `menu.sh --dev` start it.
A green run proves the crates and the format; it does not prove the window draws
correctly or that a model matches reality.

Behavioral tests are protected: never delete, disable or weaken one to make a change
pass.

## Branches and delivery

- **Nothing is edited on `main`.** Every change is a branch created from updated `main`
  (`feat/`, `fix/`, `docs/` or `chore/`), and it lands as a pull request. `main` is
  protected: a pull request, the `check`, `window` and `minimum supported Rust version`
  jobs and linear history are required, so a direct push is rejected.
- Prove the base from Git, never from a branch name:
  `git rev-list --left-right --count main...HEAD` is `0 <n>` and
  `git log --oneline origin/main..HEAD` lists exactly the commits being delivered.
- One branch per change, one thing per commit, squash-merged by the owner or by the agent in
  an approved automatic run ([ADR 0046](docs/adr/0046-the-auto-pilot-runs-a-planned-batch.md)),
  and the branch deleted afterwards. A sub-branch created from another feature branch is an
  exception and needs a stated reason.
- **One active branch at a time.** While a branch holds unmerged work, including a pull
  request still open, do not start a parallel branch for an unrelated change: finish it,
  merge it, or set it aside with explicit owner authorization. Parallel branches drift and
  produce structural conflicts.
- **Working notes stay private.** The owner's continuity note, work order, node register and
  parity queue live in the ignored `docs-private/`; they are never committed, never reviewed
  and never open a pull request. A decision or a fact the product needs becomes a document
  in `docs/` in its own change.
- **Ask before committing.** Before staging or pushing, ask the owner to confirm that the
  change matches the request and the expected result, and wait for the answer. A request to
  commit does not replace that confirmation unless the exact change was already reviewed. In
  an approved automatic run the approved plan is that confirmation; outside it, the question
  stands.
- **Verify the merge.** After opening or updating the pull request, read its `state` and
  `mergedAt`. In manual mode, while it is not merged, tell the owner and ask them to merge it
  before starting or continuing further work. After the merge, return to updated `main`,
  fast-forward it, and ask whether to delete the merged branch locally and on the remote. In
  an approved automatic run, the run verifies, merges, fast-forwards and deletes according to
  [auto-pilot](.agents/skills/auto-pilot/SKILL.md) without asking between phases.
- **The project owner merges by default.** The exception is the approved automatic run of
  [ADR 0046](docs/adr/0046-the-auto-pilot-runs-a-planned-batch.md); the historical ribbon
  pilot of [ADR 0043](docs/adr/0043-the-agent-merges-under-guardrails.md) records a past run,
  not a standing permission. Under ADR 0046 the agent
  squash-merges its own pull requests only in `piloto automatico`, one at a time, after every
  required check is green, the pull request is `MERGEABLE` with `mergeStateStatus: CLEAN`, and
  the full gate ran on the branch; a red, conflicting or unknown state stops the run. The
  agent never approves on its own behalf, never pushes to `main` and never enables GitHub
  auto-merge, however many permissions the session was given. Outside the approved automatic
  run, an instruction to merge is answered with the pull request URL.
- The gate runs on the branch being delivered, and the changelog and the docs travel in
  the same pull request.
- The procedure, the gotchas and the settings are
  [.agents/skills/pull-request-workflow](.agents/skills/pull-request-workflow/SKILL.md).

## Versioning and changelog

`CHANGELOG.md` is in Spanish. Work accumulates under `## Unreleased` in dated `###
<hito> — <YYYY-MM-DD>` subsections; the declared version changes only when a release
is prepared, updating `Cargo.toml` (`workspace.package.version`), both READMEs and
this file. Documentation, tests and tooling alone never justify a bump.

## Adding a skill

Create `.agents/skills/<name>/SKILL.md` in English: `name` equal to the folder in
kebab-case, `license: GPL-3.0-or-later` in the frontmatter, a description under 300
characters with a `Use when` clause, a procedure, gotchas and a validation section.
Register it in `.agents/skills/README.md`.
