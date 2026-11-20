# The work tree

The order this project works in when a session can lose its context: every problem becomes
a **node** with an identity, the options that could solve it, the parts of the code they
touch, and the evidence that closes it. Nothing is implemented until its node is planned
and approved; anything found on the way becomes a new node instead of a detour.

The nearest names for this are an **issue tree** (the problem broken into nodes), a
**work breakdown structure** (each node broken into stages) and the **PDCA loop**
(plan, do, check, act). This file is the register; the stages are ordinary branches and
pull requests.

## The loop

1. **The owner reports a problem** in conversation, however small or visual.
2. **It is written as a node** in this file the same turn: identity, symptom, where it is
   seen, what it touches, options with pros and cons, and an owner question if a decision
   is not ours.
3. **The owner says "this too"**: another node, same treatment. The tree grows; nothing is
   lost between messages.
4. **The plan is proposed** as an ordered set of stages, with the nodes it closes and the
   order it takes them in. The owner approves the order, not each line.
5. **Work starts on the first approved stage**: one branch, one node (or a small group
   that shares a file), the gate, the evidence, the pull request.
6. **New problems found while working** are added as nodes with a link to where they were
   seen, and are scheduled, not patched in silence.
7. **At the end of a session** the open nodes keep their state and the next one is named,
   so a fresh session reads this file and continues.

A node is `reported` until it has a plan, `planned` until the owner approves the order,
`in progress`, `done`, or `blocked` with the reason written.

## What a node holds

| Field | Content |
|---|---|
| **Id** | `P-###`, never reused |
| **Problem** | one sentence, what the reader sees |
| **Evidence** | capture, test name, file and line, or the steps to reproduce |
| **Touches** | files and symbols, so a session without context knows where to start |
| **Options** | two or three, each with pros, cons, cost and what it touches |
| **Decision** | which option, and why; a link to the ADR when it changes a rule |
| **Stages** | the smallest sequence that can be delivered and proved separately |
| **Done when** | the test, capture or measurement that closes it |
| **State** | `reported` / `planned` / `in progress` / `done` / `blocked` |

## Rules

- **No code before the node is planned.** A one-line typo may skip the ceremony; a visual
  or behavioural change may not.
- **One branch per node or per stage**, never a branch that mixes two problems.
- **The tree is the queue, not the contract.** A rule still lives in `AGENTS.md` and
  `docs/`; a capability in `capabilities.md`; a decision in an ADR. The node links to
  them, it does not replace them.
- **A node that grows past its plan is split**, not stretched.
- **The owner's words are quoted** when the symptom is visual ("se ve horrible", "los
  iconos se ven malísimos"): the quote is the acceptance criterion, the node the
  measurement of it.
- **A session starts by reading this file** and the node it is about; it ends by writing
  the state and the next node.

## Register

| Id | Problem | State | Detail |
|---|---|---|---|
| P-001 | The start-view icons read as smudges | `in progress` | [start-view-corrections §1](start-view-corrections.md) |
| P-002 | Card text runs into its icon | `in progress` | [§2](start-view-corrections.md) |
| P-003 | Template pictures are tiny and pale | `in progress` | [§3](start-view-corrections.md) |
| P-004 | Recent rows overlap | `done` | [§4](start-view-corrections.md) |
| P-005 | No way back to Inicio with a project open | `done` | [§5](start-view-corrections.md) — home control in the strip, `Ctrl+Home`, `Volver` for an empty project; merged as #146 |
| P-006 | Recents have no page bound and do not mark a missing file | `in progress` | [§6](start-view-corrections.md) |
| P-007 | Closing the last project does not return to Inicio | `planned` | [§7](start-view-corrections.md) |
| P-008 | The rail is uneven and its focus is not the catalogue's | `in progress` | [§8](start-view-corrections.md) |
| P-009 | `electrical.relay.latching`: A2 and R1 collide at 300 dpi | `reported` | [review record](../assets/symbols/review/electrical.relay.latching.md) |
| P-010 | The shuttle and the two-pressure valve are missing (OR/AND) | `reported` | [capabilities F2](capabilities.md) and ADR 0029 |
| P-012 | The theme icon cache is unbounded | `in progress` | [icons.cpp](../apps/punchi-fluid-qt/src/icons.cpp) — bounded `QCache` on `feat/start-view-visual-pass-2`: cap 73, working set 58, max group 21, LRU and palette key |
| P-013 | The icon cache can show the previous theme's tint because QML never re-requests it | `in progress` | [StartView.qml](../apps/punchi-fluid-qt/StartView.qml) and [session.cpp](../apps/punchi-fluid-qt/src/session.cpp) — `&v=<themeRevision>` in every `image://theme` URL |
| P-018 | A port designation touches or covers the symbol | `in progress` | P-018a: `envelope`, `draw_numeral`, the ninth lint rule, the clearance test and `preparation.service_unit`; P-018b follows on `feat/port-numeral-clearance-rest` |
| P-020 | The reference zones name the rows and columns backwards | `in progress` | `fix/reference-zone-axes`: letters down the vertical bands, numerals across the horizontal, `I`/`O` skipped, and the standard's two-sided form on A4; export, canvas, tests, docs and captures |
| P-022 | The branch of a tee bends instead of staying straight | `in progress` | `fix/tee-gauge-route`: a junction port takes its edge from the component or pipe it faces, the check passes the same hint, and the selftest tee probe and the captures prove the straight branch |
| P-025 | The supply starts at 6 bar instead of the industry's 4 bar | `in progress` | [P-025](#p-025--the-supply-starts-at-4-bar) on `chore/supply-four-bar`: the three built-in supplies, the templates, the example, the tests and the docs; an explicit per-placement pressure still wins |

## A node, filled

### P-018 — A port designation touches or covers the symbol

- **Problem**: the service unit's numerals 1 and 2 were centred on the dashed frame, so
  half of each glyph sat on the line and its halo erased the dash underneath.
- **Evidence**: `preparation.service_unit-{light,dark,300}.png`; before the fix the
  numerals straddled the frame.
- **Touches**: `crates/punchi-fluid-catalog/src/schema.rs` (`envelope`),
  `crates/punchi-fluid-catalog/src/lint.rs` (the ninth rule),
  `crates/punchi-fluid-draw/src/symbol.rs` (`draw_numeral`, the test),
  `assets/catalog/preparation.service_unit.toml`, `docs/symbology.md`, the review template
  and record.
- **Decision**: a declared `envelope` is the symbol's functional outer boundary; the
  renderer places each numeral on the clear side of it, inside when the outside is too
  tight. No glyph extents are invented: the test uses the anchor plus the cap.
- **Stages**:
  - **P-018a** `in progress` on `feat/port-numeral-clearance`: the envelope, the ninth
    lint rule, `draw_numeral`, the general test helper, the docs and
    `preparation.service_unit`.
  - **P-018b** after P-018a merges, on `feat/port-numeral-clearance-rest`: the latching
    relay, the 0.0/1.0 class, the gauge/designation overlap, the valves and the cylinders.
- **Done when**: the service-unit test passes, the 300 dpi and light/dark captures show
  the numerals clear, and the gate is green; P-018b extends the assertions to the rest.
- **State**: `in progress` (P-018a); P-018b not started.

### P-020 — The reference zones name the rows and columns backwards

- **Problem**: on A3 and larger the grid put **numerals in the vertical bands and letters in
  the horizontal ones**, so a note such as B3 named a different field than the standard; the
  default A4 sheet carried no grid at all.
- **Evidence**: the owner's A3 observation on 2026-11-20; the A3 canvas capture with `A`–`H`
  across and `1`–`6` down; the positional tests below and the A3/A4 captures in the EGM
  evidence folder (`p020-a3-landscape-{light,dark}.png`, `p020-a3-portrait-…`,
  `p020-a4-landscape-…`, `p020-a4-portrait-…`, `p020-a0-landscape-light.png`,
  `p020-custom-450-symmetric-light.png`).
- **Touches**: `crates/punchi-fluid-export/src/reference_zones.rs` (`draw_reference_zones`,
  `row_label`, the tests), `crates/punchi-fluid-export/src/sheet.rs` (the frame and legend
  around them), `crates/punchi-fluid-export/src/page.rs`
  (`has_reference_zones` / `uses_large_block`), `crates/punchi-fluid-export/src/block_layout.rs`,
  `crates/punchi-fluid-export/tests/png.rs`, `apps/punchi-fluid-qt/src/sheetframe.{h,cpp}`,
  `apps/punchi-fluid-qt/src/main.cpp`, `docs/reference/title-block.md`,
  `docs/designs/title-block-and-sheet.md`, `docs/capabilities.md`, `docs/ui-ux.md`,
  `docs/next-steps.md`, `assets/screenshots/README.md`, `CHANGELOG.md`.
- **Options**:
  1. Keep the swapped convention as a documented deviation. **Pro**: no code. **Con**: a grid
     read by notes is wrong, which is worse than no grid; the owner asked for the standard.
  2. Fix the export only. **Pro**: the printed page is right. **Con**: the canvas would print
     one grid and show another, the divergence ADR 0003 exists to prevent.
  3. Fix both devices with four sides on every sheet. **Pro**: one rule. **Con**: four sides
     on A4 is the noise the original design rejected, and the standard asks for two.
  4. Fix both devices, four sides from A3 upwards, the standard's **two-sided** form on A4.
     **Pro**: the standard's form and a referenceable default sheet. **Con**: A4's block gate
     has to split so its fitted 170 × 45 mm block does not grow.
- **Decision**: option 4, chosen by the owner on 2026-11-20: letters on the vertical bands
  read top to bottom, numerals on the horizontal ones left to right, `I` and `O` skipped, and
  A4 with numerals on the top band and letters on the right band only. The geometry stays
  duplicated between Rust and C++ on purpose, with no new dependency edge; `--selftest`
  compares the label sequence. The A0 17-vs-16 field count stays an open question and is not
  changed here.
- **Stages**: one branch, `fix/reference-zone-axes` — the export, the canvas, the positional
  and clearance tests, the documents and the captures, then the gate.
- **Done when**: `cargo test -p punchi-fluid-export` and `./scripts-dev/check.sh` are green,
  the `--selftest` zone steps pass, the A3/A4 captures and 300 dpi exports show the corrected
  grid, and the owner confirms the diff.
- **State**: `in progress` on `fix/reference-zone-axes`; `done` after the owner merges.

### P-022 — The branch of a tee bends instead of staying straight

- **Problem**: a gauge above a tee connected with a horizontal stub and then a vertical one:
  the dot sat at the meeting point but the T was bent. Every junction port is at the centre,
  so `port_edge`'s first minimum was `Left` for all three and the branch left horizontally.
- **Evidence**: the owner's vertical case reported on 2026-11-20; before the fix the branch
  routed down, sideways and down again. After the fix the selftest prints
  `tee probe branch=2 left=0 right=1 old=-1`; `p022-gauge-on-tee-{light,dark}.png` and the
  300 dpi page show the straight stub; the five tests named below pin the edge, the branch,
  the two halves, an ordinary port and the check/draw parity.
- **Touches**: `crates/punchi-fluid-draw/src/placed/edge.rs` (`EDGE_TIE_MM`,
  `port_edge_towards`, `turned_edge`; split from `placed.rs` to stay under the 800-line
  architecture budget), `crates/punchi-fluid-draw/src/placed.rs` (the re-export and
  `draw_pipe`), `crates/punchi-fluid-draw/src/placed_routing_tests.rs`,
  `crates/punchi-fluid-check/src/lib.rs` (`pipe_ends`) and its `tests.rs`,
  `crates/punchi-fluid-ffi/src/lib.rs` (`pipe_route`),
  `apps/punchi-fluid-qt/src/main.cpp` (the selftest fixture and the `pipe_at` probes), the
  ADR, capabilities, UI, work-tree, next-steps and changelog entries.
- **Options**:
  1. Keep the bend and document it. **Pro**: no code. **Con**: the owner's example is a valid
     T, and a bent stub is a drawing error, not a style.
  2. Give each junction port an explicit edge in the catalog. **Pro**: one place names the
     intent. **Con**: a schema and catalog change for one ambiguous case, and a tee on a
     vertical line or a gauge below would still be wrong.
  3. Resolve the tie from the other endpoint. **Pro**: no schema change; an ordinary port and
     a corner keep exactly the `port_edge` result, and the check can pass the same hint.
     **Con**: the gesture helpers keep the bare edge (recorded below).
  4. Add a direct-path fast path at the top of `route_between_boxes_avoiding`. **Con**: it
     touches every pipe in the project, and the first review could not demonstrate the
     detour it was meant to remove.
- **Decision**: option 3, on 2026-11-20. `port_edge_towards` applies the hint only when all
  four nearest-edge distances tie within `EDGE_TIE_MM` (the exact-centre junction); otherwise
  it returns the `port_edge` result, including its first-minimum corner order. A missing or
  coincident hint falls back to the port id in the placement's local frame. `draw_pipe`,
  `pipe_route` and `pipe_ends` pass the hint; `pipe_is_elbow`, `magnet_centre` and
  `straighten_pipe` keep the bare edge on purpose, so the drag/straighten refusal is
  unchanged and is a future node. No document, catalog, solver or format change, and no new
  ADR: this restores the intended T.
- **Stages**: one branch, `fix/tee-gauge-route` — (1) `port_edge_towards` and its unit tests
  with no call-site change; (2) the three call sites, the route/parity fixture and the
  crossing assertion; (3) the selftest fixture, the documents and the captures, then the gate.
- **Done when**: `port_edge_towards_resolves_the_centre_from_the_hint`,
  `an_ordinary_port_keeps_its_edge`, `a_gauge_above_a_tee_gets_a_straight_vertical_stub`,
  `the_two_halves_of_a_spliced_pipe_stay_straight` and
  `the_check_route_matches_the_drawn_route_on_a_tee` are green with the existing junction and
  splice tests; `--selftest` exits 0 with `branch=2 left=0 right=1 old=-1`; `--check-layout`,
  `--check-qml`, `--check-theme` and `--check-icons` pass; `./scripts-dev/check.sh` is green;
  the light/dark and 300 dpi captures show the straight branch; the owner confirms the diff.
- **State**: `in progress` on `fix/tee-gauge-route`; `done` after the owner merges.

### P-025 — The supply starts at 4 bar

- **Problem**: the built-in supplies declared 6 bar, so a new circuit was designed around a
  pressure the owner does not want as the workshop default; 4 bar with 10 mm tubing is the
  industry case he teaches.
- **Evidence**: the owner's request of 2026-11-20; the three catalog files declared
  `pressure_bar = 6.0`, the four templates place `supply.compressor`, and the worked example
  ran at 6 bar.
- **Touches**: `assets/catalog/supply.compressor.toml`,
  `assets/catalog/supply.compressor_adjustable.toml`, `assets/catalog/supply.network.toml`,
  the names they expose, `assets/catalog/flow.meter.toml` (its conductance reference),
  `crates/punchi-fluid-catalog/src/catalog_tests.rs`,
  `crates/punchi-fluid-solver/src/network.rs` (the two pressure references),
  `crates/punchi-fluid-ffi/tests/abi.rs` (the new tests, the templates, the example and the
  gauge), `assets/catalog/README.md`, `docs/capabilities.md`, `docs/reference/catalog.md`
  if it names the default, `docs/guides/`, the glossary, the interface mock, this register
  and `CHANGELOG.md`.
- **Options**:
  1. Change the catalog default only. **Pro**: one field per supply; a per-placement edit
     already exists. **Con**: an old document with no edit changes behaviour, which has to be
     stated and tested.
  2. Store 4 bar in every placement. **Con**: a format change and a migration for a value
     that is already a catalog default.
  3. Add a per-document supply pressure. **Con**: a new document field outside this node's
     scope.
- **Decision**: option 1, on 2026-11-20. The pressure is a catalog value; the existing
  per-placement edit still wins. The third supply (`supply.compressor`) is the solver's
  `DEFAULT_SUPPLY` and the one every template places, so it changes with the two the spec
  names; leaving it at 6 bar would leave the product default at 6. A document with an
  explicit `pressure_bar = 6.0` keeps it; one with no edit reads 4 bar, because the value
  was never stored, and needs the edit to stay at 6. No format change and no new field.
- **Stages**: one branch, `chore/supply-four-bar` — the three catalog files, the names, the
  flow-meter reference, the tests, the docs and the changelog, then the gate.
- **Done when**: `a_new_supply_is_four_bar`,
  `an_explicit_six_bar_edit_is_kept_when_the_file_is_reopened`,
  `an_old_supply_without_an_edit_reads_the_new_default`, the template/example tests, the
  gauge test and the solver pressure references are green; `./scripts-dev/check.sh` passes;
  the owner confirms the diff.
- **State**: `in progress` on `chore/supply-four-bar`; `done` after the owner merges.

### P-011 — Example: the window title shows an empty document name

- **Problem**: on Inicio the window still names an untitled document.
- **Evidence**: capture `assets/screenshots/start-view-…`; the title reads «Sin título».
- **Touches**: `apps/punchi-fluid-qt/Main.qml` (`title:` binding) and the start view state.
- **Options**:
  1. Title from the program name while Inicio is open. **Pro**: one binding. **Con**: none.
  2. Hide the window title. **Con**: the window manager owns it.
- **Decision**: option 1; no ADR, it is a UI detail.
- **Stages**: the binding, a capture, one `--selftest` check.
- **Done when**: the capture shows the program name and the check passes.
- **State**: `done` (delivered with the start view).

## Reconciliation after #143 (2026-11-20)

Read against `015ec77`: the files were checked, not the branch name. P-009, P-010 and
P-011 are untouched by it.

- **P-001** `in progress`: the tones come from the theme and the provider keys by name,
  size and scheme; one size rule, `sourceSize` and `fillMode` remain, so two sizes still
  upscale from the provider's 22 px fallback.
- **P-002** `in progress`: the icon column, the `unit / 2` gap and the text column after
  it were delivered; the column is 1.8× `unit`, not the fixed 2× `unit`.
- **P-003** `in progress`: the picture follows the card's content width; it still takes
  about 51 % of the card height and the 180×100 mm raster is stretched to the item rect,
  so the aspect is not preserved.
- **P-004** `done`: one row per entry at 1.6× `unit`, name and path stacked, the
  «no disponible · …» form kept and the button centred; nothing remains.
- **P-006** `in progress`: a missing path is greyed, named and removable; the page
  declares no bound of its own, renders the ABI's five-entry page and the list does not
  scroll.
- **P-008** `in progress`: hover comes from `surfaceRaised` and most items are 1.9×
  `unit`; «Volver» is 1.6× `unit`, the separator sits inside the margins, and the rail has
  no focus ring, keyboard reach or `Accessible.name`.
- **P-012** `reported`: the provider's `QHash`, keyed by name, tone, pixels and DPR,
  inserts without a cap or an eviction rule.

Order: **A** this reconciliation, docs only, `check-docs.sh`; **B** P-001 and P-012 with
P-002 and P-003, the visual pass; **C** P-005 and P-007, behaviour; **D** P-006 and P-008,
closing the page. B, C and D each take their own branch, gate and pull request.

## Implementation in progress (2026-11-20)

`feat/start-view-visual-pass-2`, from `1afef86`; P-001, P-002, P-003, P-012 and P-013 are
implemented, and the register moves them to `done` after the merge. Evidence: `--check-icons`
is 0 missing at 1×, 1.25×, 1.5× and 2× (cache total 58, max group 21, LRU and palette probes
ok); `--check-layout` is ok at 1280 and 1440 and at 1.25×, 1.5× and font 1.5×; `--selftest`
theme steps ok; `--report-paints 2` is one paint; `--check-qml` is 0; four captures at 1280
and 1440 in both schemes.
