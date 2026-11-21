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
`in progress`, `done`, or `blocked` with the reason written. A node the owner sets aside is
`deferred` with the decision and the node that replaces it; it is not in the current queue.

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
| P-012 | The theme icon cache is unbounded | `done` | [icons.cpp](../apps/punchi-fluid-qt/src/icons.cpp) — bounded `QCache` merged as #145: cap 73, working set 58, max group 21, LRU and palette key |
| P-013 | The icon cache can show the previous theme's tint because QML never re-requests it | `done` | [StartView.qml](../apps/punchi-fluid-qt/StartView.qml) and [session.cpp](../apps/punchi-fluid-qt/src/session.cpp) — `&v=<themeRevision>` in every `image://theme` URL, merged as #145 |
| P-018 | A port designation touches or covers the symbol | `in progress` | P-018a: `envelope`, `draw_numeral`, the ninth lint rule, the clearance test and `preparation.service_unit`; P-018b follows on `feat/port-numeral-clearance-rest` |
| P-020 | The reference zones name the rows and columns backwards | `in progress` | `fix/reference-zone-axes`: letters down the vertical bands, numerals across the horizontal, `I`/`O` skipped, and the standard's two-sided form on A4; export, canvas, tests, docs and captures |
| P-021a | A plug or a silencer is dropped on the sheet instead of fitted to the port | `in progress` | [P-021a](#p-021a--a-port-closure-belongs-to-the-port) on `feat/port-closures`: the per-port map, the menu, the drawing, the parts row, format 14 and ABI 12 |
| P-021b | The silencer + regulator chain cannot be drawn | `planned` | [P-021b](#p-021b--the-flow-regulator-and-the-silencer-chain): the flow regulator, the chain and the full menu, with the next free format |
| P-022 | The branch of a tee bends instead of staying straight | `in progress` | `fix/tee-gauge-route`: a junction port takes its edge from the component or pipe it faces, the check passes the same hint, and the selftest tee probe and the captures prove the straight branch |
| P-023 | The air the model moves is invisible on the sheet | `in progress` | [P-023](#p-023--the-sheet-animates-the-air-with-a-stream-of-solid-arrowheads) on `feat/air-flow-animation`: the solver's per-hose signed flow, the stream of arrowheads, the session phase, the reduced-motion and printed arrowhead, and the captures |
| P-024a | A pneumatic connection does not carry its tube size | `in progress` | [P-024a](#p-024a--a-pneumatic-connection-carries-its-tube-size) on `feat/tube-size`: format 13, the width series, the parts rows, the Conducto gallery; the physics is P-024b |
| P-025 | The supply starts at 6 bar instead of the industry's 4 bar | `in progress` | [P-025](#p-025--the-supply-starts-at-4-bar) on `chore/supply-four-bar`: the three built-in supplies, the templates, the example, the tests and the docs; an explicit per-placement pressure still wins |
| P-026 | The electrical connection does not work as planned | `deferred` | [P-026](#p-026--the-electrical-connection-does-not-work-as-planned): the owner's decision of 2026-11-20; not in the current queue and not touched by P-024a |
| P-027 | The ribbon's horizontal groups overflow at 1280/1440 | `in progress` | [P-027](#p-027--the-ribbons-horizontal-groups-overflow-at-12801440) on `fix/ribbon-collapse-1280`: the fold at 1280, the reachable Diámetro, the layout check and the 1440 known limit |
| P-028 | The ribbon's group row must fit 1366×768 unfolded | `in progress` | [P-028](#p-028--the-ribbons-group-row-fits-1366768) on `fix/ribbon-1366`: one row with per-group collapse, the measured set, the layout matrix, the flyout, the screen-aware first window and the captures |
| P-029 | `canvasitem.cpp` grew past the input-path budget with the port menu | `reported` | [P-029](#p-029--canvasitemcpp-grew-past-the-input-path-budget): the double click and the release no-op put it at 812 lines; split the input path instead of keeping the `BIG_ALLOWED` exemption |
| P-030 | The documentation diagrams are ASCII instead of editorial SVG | `planned` | [P-030](#p-030--the-documentation-diagrams-become-editorial-svg): the `diagram-design` fork with a Punchi profile, accessible SVG and a recorded origin; the ISO 1219 symbols and the real captures are untouched |
| P-031 | Nothing guards English titles and file names outside the documented exceptions | `planned` | [P-031](#p-031--english-and-translation-are-guarded): `check-language.sh`, the exception list in `docs/style.md` and a fixture; no runtime, UI or documentation content change |
| P-032 | Visual work has no single route to its documents, commands and evidence | `in progress` | [P-032](#p-032--a-punchi-visual-language-skill) on `docs/visual-language-skill`: the skill and its five references, the catalog row, the skill and anchor checks with their fixture, and the changelog; the diagrams and the ISO symbology are untouched |
| P-033 | The icon family's inventory and gaps are not documented | `planned` | [P-033](#p-033--the-simulator-owns-its-icon-family): the inventory in `docs/reference/icons.md`, the shared-shape gaps and the small-size proof; after P-032; the ISO 1219 symbols and the diagrams are untouched |
| P-034a | Inicio does not look like the program nor tell the truth about what it offers | `in progress` | [P-034](#p-034--inicio-tells-the-truth): three offered templates, five honest recent rows, missing/`quitar`, the empty state, `Volver a la hoja`, `Esc`, and the page filling 1366×768; on `feat/start-view-visual` |
| P-034b | The start view has no domain chip, recent thumbnails or opened time | `planned` | [P-034](#p-034--inicio-tells-the-truth): the template domain model, a preview-by-path entry with a bound and a one-past test, and a `recent.json` version change with its migration; after P-034a |

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

### P-023 — The sheet animates the air with a stream of solid arrowheads

- **Problem**: the circuit moves but the air that moves it is invisible: the interactive
  sheet colours a hose by pressure and draws no flow at all, the solver computes the signed
  flow of every edge and discards it, and only the fixed example draws an arrowhead. The
  owner wants the sheet to show it, and asked on 2026-11-21 that the **solid arrowhead** be
  kept on the travelling mark: a stroke alone says nothing on a still page.
- **Evidence**: the owner's request of 2026-11-20 (a travelling mark, not a row of chevrons)
  and of 2026-11-21 (“quiero ver la punta de la flecha”); the approved spec
  `specs/air-flow-animation.md`; the captures
  `specs/evidence/20261121-p023-phase-1.png`, `-phase-2.png` and `-reduced-motion.png`, which
  still show the stroke before the head and are to be repeated with it; the solver, draw,
  ABI and `--selftest` tests named below.
- **Touches**: `crates/punchi-fluid-solver/src/network.rs` (the `pipe` edge index, the
  document-indexed `pipe_flows` store and borrowed accessor, `Network::build(...,
  pipe_count)`, the P-021a terminations and the tests),
  `crates/punchi-fluid-draw/src/placed.rs` (the `Pipe` fields, the pulse and its head, the
  `Vec`-free segment), `crates/punchi-fluid-draw/src/placed_flow_tests.rs` (the pulse and
  head tests), `crates/punchi-fluid-draw/src/symbol.rs` (`draw_flow_arrow`, unchanged and
  reused), `crates/punchi-fluid-draw/src/placed_electrical_tests.rs`,
  `placed_tube_tests.rs`, `placed_tests.rs`, `schematic.rs` and
  `crates/punchi-fluid-check/src/tests.rs` (the `Pipe` literals and the `Network::build`
  count), `crates/punchi-fluid-ffi/src/lib.rs` (the enumerated `rebuild_model`,
  `SceneState`/`scene`, `SceneState::printed`, the frame/export/template/`bounds_placed`/
  `parts_list` callers and the scene tests), `crates/punchi-fluid-ffi/include/punchi_fluid.h`
  (`phase_s`/`motion`, `PF_ABI_VERSION` 13) and `tests/abi.rs`,
  `apps/punchi-fluid-qt/src/session.cpp`/`.h` (the phase and motion state, the read-only
  `phase` property, start/pause/stop/step), `Main.qml` (the existing 16 ms tick),
  `canvasitem.cpp`, `src/main.cpp` (the `animation:` selftest steps and the frame calls),
  `docs/adr/0034-the-sheet-animates-the-models-flow.md`, `docs/adr/README.md`,
  `docs/ui-ux.md`, `docs/work-queue.md` §10a, `docs/capabilities.md`, `docs/symbology.md`,
  this register and `CHANGELOG.md`. It changes no document field, format version, diagram,
  symbol, ISO 1219 geometry, sheet, zoom or export composition.
- **Options**:
  1. Chevrons spaced along the route, as work-queue 10a was first written. **Pro**: no head
     to draw. **Con**: several marks per hose cost more per frame and give no reading a
     single pulse with a head does not give.
  2. Guess the flow from the pressure difference. **Pro**: no solver change. **Con**: a dead
     end with pressure has a difference and no flow, so the mark would lie; a closed valve
     would keep moving.
  3. A per-frame particle system. **Pro**: a richer motion. **Con**: allocations and
     unbounded state on the frame path.
  4. Expose the solver's signed flow, carry it as a value with a phase, and overlay a short
     stroke **carrying the existing solid arrowhead** on the route the drawing already
     builds. **Pro**: the model is the only source, the head is ISO 1219-1 and our own
     primitive, the phase is a scalar, and the page can freeze the one head that keeps the
     direction. **Con**: an ABI 13 frame change and a `Pipe` field set to update at every
     literal.
- **Decision**: option 4, approved by the owner on 2026-11-20 and confirmed with the arrowhead
  on 2026-11-21. The flow vector is indexed by the **document connection list**, enumerated
  before the circuit and drawing filters, so a skipped hose keeps its own `None` slot
  (`Some(0.0)` is a modelled hose with no flow). The C++ `FluidSession` owns the phase: start
  clears it, the existing 16 ms tick advances it, pause freezes it, a step adds the step's
  seconds, and stop or restart clears it; the solver, the ABI and the drawing run no clock.
  `draw_pipe` keeps the line class and overlays a **stream** of 4 mm strokes **in the hose's
  own colour** — never the accent, which means selection — drawn 1.7× the hose's width so
  they read against the line, each with `symbol::draw_flow_arrow` on its leading end, from
  `from` for a positive flow and `to` for a negative one. The marks are an exact lattice
  (two to eight per hose, 14 mm apart where it fits), so an arrowhead enters while another
  leaves and no frame is empty. The stream's speed follows `|flow|` between 6 and 45 mm/s, so
  a trickle drifts and a strong current runs; the scale is readability, not the physical air
  speed, and the ordering is the model's. The solver publishes the **frame's net flow**, not the last
  substep's: the owner's second recording caught the arrows reversing at 500 Hz on a sealed
  line, and `a_dead_end_line_does_not_report_a_reversing_flow` fixes it. Zero flow, a route
  shorter than a mark plus its clearance, and an electrical pipe draw nothing. Reduced motion
  draws one static arrowhead at the middle of the route without advancing the phase, and
  `SceneState::printed` passes the model's flows with phase zero and no motion, so the page
  keeps one frozen arrowhead per hose with air. **ADR 0034** records it; **ABI 13** adds the
  frame signature; no document or format change.
- **Stages**: one branch, `feat/air-flow-animation` — (1) the solver's document-indexed flow
  and its three tests (**green**); (2) the `Pipe` fields, the pulse, the solid head and the
  draw tests (**green**); (3) the ABI 13 frame, the session phase and motion, the frozen
  arrowhead on the printed page, the selftest steps and the captures (**in progress**);
  (4) `docs/capabilities.md`, this node, `CHANGELOG.md` and the gate.
- **Done when**: the solver tests (`a_pipe_reports_the_flow_through_it`,
  `a_zero_flow_pipe_reports_zero_after_a_step`, `a_skipped_pipe_reports_no_flow`), the draw
  tests (`the_stream_never_leaves_the_hose_empty`,
  `the_marks_are_ordered_inside_the_hose_and_do_not_overlap`,
  `the_stream_moves_with_the_phase`, `the_sign_reverses_the_stream`,
  `a_still_hose_draws_nothing`, `reduced_motion_keeps_one_arrowhead_in_the_middle`,
  `a_short_route_draws_nothing`,
  `the_pace_comes_from_the_hose_and_the_flow_only_hurries_it`,
  `every_mark_draws_a_thick_stroke_with_its_head`,
  `a_drawn_mark_keeps_the_line_class_and_the_numerals`,
  `a_page_without_flow_draws_no_mark`) and
  the ABI tests (`the_frame_accepts_a_phase_and_a_motion_flag_and_refuses_any_other_flag`,
  `a_printed_page_carries_the_flow_and_freezes_it`) are green; the `animation:` selftest steps
  pass; `--check-qml`, `--check-theme` and `./scripts-dev/check.sh` exit 0; the captures show
  the head at two phases, the frozen head under reduced motion and the head on a printed
  page; no allocation per frame is confirmed by review; and the owner confirms the diff.
- **State**: `in progress` on `feat/air-flow-animation`; `done` after the owner merges.

### P-036 — The transport disappears when the ribbon folds

- **Problem**: at 1280 px and below the ribbon folds to its tab row and the transport —
  Iniciar, Pausa, Paso, Detener — lived only inside the **Menú**. The owner ran the window at
  that width, had a circuit on the sheet, and could not start the simulation; `ui-ux.md`
  says the transport is **always visible**.
- **Evidence**: the owner's report of 2026-11-21 («hay circuito pero Iniciar no responde»),
  the 1200×800 capture before the change (no transport in the row) and the one after it
  (Menú · Iniciar · Paso · Detener).
- **Touches**: `apps/punchi-fluid-qt/RibbonBar.qml` (the folded row and the three buttons),
  `apps/punchi-fluid-qt/src/main.cpp` (`--check-layout` asserts the cluster at 1280),
  `docs/ui-ux.md` (the fold paragraph) and `CHANGELOG.md`.
- **Options**: 1. put the transport in the menu (today). **Con**: two clicks and a scroll for
  the action a lesson uses most. 2. a second, floating transport bar. **Con**: a second
  transport that can disagree with the ribbon. 3. three compact buttons in the folded tab
  row, play and pause sharing one control. **Pro**: always visible, one definition, no new
  region. **Con**: the row loses ~150 px at 1280.
- **Decision**: option 3. The folded row carries play/pause (one button whose label, icon
  and action follow the session), Paso and Detener; the menu keeps the rest, Restablecer and
  «Hasta cambio» included.
- **Done when**: `--check-layout` reports `transport visible` at 1280 and fails when the
  cluster is missing or hidden; the 1200×800 capture shows the three buttons; the selftest is
  green; the owner confirms the diff.
- **State**: in progress on `feat/collapsed-transport`; `done` after the owner merges.

### P-024a — A pneumatic connection carries its tube size

- **Problem**: a hose was drawn at one fixed stroke and the parts list had no tube rows
  and no length, so a drawing could not say which tubing a circuit needs.
- **Evidence**: the owner's request of 2026-11-20 (“the tube diameter is not specified when
  connecting; the line should thicken with the tubing; metric and inch sizes, 10 mm by
  default; 16 mm and more exist”); the approved spec `specs/tube-size.md`; the format-13
  captures `p024a-tube-4-10-16-{light,dark,300}.png` in the EGM evidence folder.
- **Touches**: `crates/punchi-fluid-core/src/tube.rs` (the series and the bounds),
  `crates/punchi-fluid-file/src/sheet.rs` (format 13 and the migration),
  `crates/punchi-fluid-doc/src/lib.rs` (the connection value, `Change::Tubed`, the splice),
  `crates/punchi-fluid-draw/src/placed.rs` (`tube_width_mm`, `route_length_mm`),
  `crates/punchi-fluid-export/src/parts.rs` (`TubeRow`), `src/page.rs`, `src/sheet.rs`,
  `crates/punchi-fluid-ffi` (the tube ABI and the series enumeration),
  `apps/punchi-fluid-qt` (`TubeSizeControl.qml`, the Conducto group, the status bar,
  `Alt+D`), `docs/adr/0032-a-connection-carries-its-tube-size.md`,
  `docs/reference/tube.md`, `docs/symbology.md`, `docs/ui-ux.md`, `docs/capabilities.md`,
  this register and `CHANGELOG.md`.
- **Options**:
  1. Store the diameter as `f32`. **Con**: `Connection` and `StoredPipe` derive `Eq`/`Hash`
     for the undo history, and a float breaks that; the offset already uses integer tenths.
  2. One metric/inch table with nominal pairs. **Con**: 1/4" is 6.35 mm, not 6 mm; the
     entries must stay separate and the stored value exact.
  3. A continuous width map. **Con**: 0.04 mm steps are invisible at 300 dpi and the frame
     stops being a different class.
  4. Integer hundredths, a series of three widths capped below the frame, and separate
     metric and inch entries. **Pro**: exact equality, visible steps, the frame stays apart.
- **Decision**: option 4, approved by the owner on 2026-11-20 (“procede”): pneumatic only,
  the electrical connection deferred, 10 mm, integer hundredths, the width series, the parts
  rows and format 13 (P-021 takes 14). ADR 0032 records it; the physics is P-024b and the
  solver is untouched.
- **Stages**: one branch, `feat/tube-size` — (1) the core series; (2) the format 13 and its
  bounds; (3) the document value, the default, the seeds and the splice; (4) the stroke and
  the route length; (5) the parts rows and the page; (6) the ABI and the series enumeration;
  (7) the Qt gallery, the status bar and `Alt+D`; (8) the docs, the conformance review and
  the captures, then the gate.
- **Done when**: the P-024a tests named in `docs/capabilities.md` are green (the series, the
  10 mm default, the old sheet, the bounds, the width series, the electrical exception, the
  patterns, the splice, the parts rows, the route length, one undo, the ABI); `--selftest`
  exits 0 with the `tube: …` steps and `--check-theme`, `--check-layout` and `--check-qml`
  pass; `./scripts-dev/check.sh` is green; the 4/10/16 mm light/dark and 300 dpi captures
  show the widths and the frame separation; the owner confirms the diff.
- **State**: `in progress` on `feat/tube-size`; `done` after the owner merges.

### P-021a — A port closure belongs to the port

- **Problem**: a blind plug or a silencer was a separate placement with its own port and a
  hose to the valve, so the drawing showed the cone away from the exhaust and the parts
  list counted a free component and a tube that the circuit does not use.
- **Evidence**: the owner's report of 2026-11-20 and the split of 2026-11-21; the approved
  spec `specs/port-closures.md`; the `--selftest` steps `close: …`; the 300 dpi page with
  `Silenciador · 1V1 · escape 3`; the light/dark captures
  `p021a-port-closures-{light,dark}.png` and their zoomed pair.
- **Touches**: `crates/punchi-fluid-doc/src/lib.rs` (`port_closures`,
  `Change::Closured`, `set_port_closure`, the pipe refusal), `punchi-fluid-catalog`
  (`MAX_PORTS`), `punchi-fluid-file` (format 14, `StoredPlacement.port_closures`,
  `validate_pipes`), `punchi-fluid-solver` (`PortTermination`, the exhaust skip),
  `punchi-fluid-draw` (`placed/closure.rs`, `Placed.closures`, the numeral suppression),
  `punchi-fluid-check` (`BadPortClosure`), `punchi-fluid-export` (`ClosureRow`, the page),
  `punchi-fluid-ffi` (the port/closure ABI, `PF_ABI_VERSION = 12`),
  `apps/punchi-fluid-qt` (`PortClosureMenu.qml`, the `SchematicItem` signal and ordering,
  `Alt+C`, the palette filter, the panel block), `docs/adr/0033-…`, the reference pages,
  `docs/ui-ux.md`, `docs/symbology.md`, `docs/capabilities.md`, this register and
  `CHANGELOG.md`.
- **Options**: keep the placement and add an attach gesture (rejected: the hose and the
  free-component count remain); a child placement with a parent id (rejected: a second
  relation and hit test for ports only); a `Behaviour::PortClosure` variant (rejected: it
  duplicates the physics and blocks the chain); the per-port map (chosen).
- **Decision**: option 4, the per-port closure map, approved by the owner with the P-021a /
  P-021b split on 2026-11-21. **ADR 0033** records it and amends ADR 0016 without erasing
  the two behaviours. The electrical port is excluded with **P-026**; the chain is
  **P-021b**.
- **Stages**: one branch, `feat/port-closures` — (1) the document map, the change and the
  pipe refusal; (2) the catalog `MAX_PORTS` and format 14 with its bounds; (3) the solver
  termination; (4) the drawing and the numeral rule; (5) the check; (6) the parts rows and
  the page; (7) the ABI 12; (8) the Qt menu, the ordering, the release no-op, `Alt+C`, the
  panel and the palette filter; (9) the docs, the evidence and the gate.
- **Done when**: the P-021a tests named in `docs/capabilities.md` and the `close:` selftest
  probes are green, `--check-qml`, `--check-theme`, `--check-layout` and `--selftest` exit
  0, `./scripts-dev/check.sh` is green, the 300 dpi page shows the closure rows and the
  light/dark captures show the closure at the port with the numeral clear, and the owner
  confirms the diff.
- **State**: `in progress` on `feat/port-closures`; `done` after the owner merges.

### P-021b — The flow regulator and the silencer chain

- **Problem**: an exhaust often carries a **flow regulator and a silencer in series**, and
  a single closure id cannot express the chain or its setting. The full port menu that
  edits both is the second half the owner split from P-021a.
- **Evidence**: the owner's split of 2026-11-21; the alternatives in ADR 0033; the
  `throttle.adjustable` definition still missing.
- **Touches**: `punchi-fluid-catalog` (the `throttle.adjustable` file and its review),
  `punchi-fluid-doc`/`punchi-fluid-file` (the chain shape and the next free format, with an
  explicit migration from the single closure id to a one-element chain), the solver
  termination for the regulator, `punchi-fluid-draw` (the chain at the port),
  `punchi-fluid-export` (the row for the chain), `punchi-fluid-ffi`, the Qt menu, the ADR
  that follows 0033 and the docs.
- **Options**: reuse the single id and add the regulator as a second key (rejected: two
  independent closures cannot say "in series" or order the parts); a chain value with the
  regulator's setting (chosen by the split); a new behaviour (rejected: the physics is the
  existing throttle plus the existing silencer).
- **Decision**: P-021b owns the chain and the flow regulator. It takes the **next free
  format at its branch** and publishes the migration from the single id to a one-element
  chain; it does not promise 15 here if the shape changes. The single-id map of P-021a is
  the honest cut, not a chain in disguise.
- **Done when**: the regulator file and its review exist, the chain is drawn, stored and
  listed with the same numeral and parts rules, the migrated format reads an old sheet with
  one closure, and the owner approves its own specification.
- **State**: `planned`; not started.

### P-029 — `canvasitem.cpp` grew past the input-path budget

- **Problem**: the port menu added the repeated press, the pipe-start refusal and the
  release no-op to `canvasitem.cpp`, and the file reached **812 lines**, past the
  800-line workspace budget. It is listed in `BIG_ALLOWED` with a comment that the split
  is a follow-up, not a permanent exemption.
- **Evidence**: `scripts-dev/check-architecture.sh` `BIG_ALLOWED` and the 812-line
  measurement on `feat/port-closures`; the owner's decision of 2026-11-21.
- **Touches**: `apps/punchi-fluid-qt/src/canvasitem.cpp` (the editing press path, the
  double-click count, the port menu signal and the release handling), its header, the
  `SchematicItem` selftest fixture in `src/main.cpp`, and the `BIG_ALLOWED` line.
- **Options**: leave the exemption (rejected as a permanent answer: the input path keeps
  growing); split the pointer/gesture helpers into a small C++ unit (chosen); move the
  whole item to a `QQuickItem` subclass with a private implementation (rejected for now:
  it touches the QML registration and the tests).
- **Decision**: split the **input path** — the press/release/double-click decision, the
  port hit and the pipe start — into a helper compiled beside the item, keeping the QML
  type and the behaviour unchanged. The `BIG_ALLOWED` entry is removed with the split.
- **Done when**: `canvasitem.cpp` is under 800 lines, `BIG_ALLOWED` no longer names it,
  the `--selftest` editing and `close:` probes stay green, and `./scripts-dev/check.sh`
  passes.
- **State**: `reported`; not started.

### P-026 — The electrical connection does not work as planned

- **Problem**: the electrical wire — its class, its stroke and its drawn route — does not
  behave as the design intended, so it cannot be relied on as a tube or as a measured line.
- **Evidence**: the owner's decision of 2026-11-20, recorded in `specs/tube-size.md`; the
  P-024a implementation deliberately leaves the wire's class, stroke and drawing untouched.
- **Touches**: none yet; it is not in the current queue. The node exists so a later session
  finds the decision instead of reopening it as a missing tube.
- **Options**: (not opened) the owner reopens the node with its own spec, review and tests
  when the electrical connection is taken up again.
- **Decision**: **deferred** by the owner on 2026-11-20. It stays at the margin until a
  future effort; P-024a does not use, fix, hide or test it, and a wire has no tube size, no
  width series and no parts row.
- **Done when**: a future node reopens it with an approved spec.
- **State**: `deferred`; not in the current queue.

### P-027 — The ribbon's horizontal groups overflow at 1280/1440

- **Problem**: the **Editar** groups in a row measure about 1875 px (Historial 144,
  Circuito 95, Componente 506, Alinear 403, Conducto 337, Repartir 188, Paleta 202) while
  the content is ~1274 px at 1280×800 and ~1434 px at 1440×900. The compact breakpoint was
  1240, so at 1280 the ribbon unfolded and the Conducto group started past the right edge;
  P-024a's **Diámetro** was among the controls the user could not reach.
- **Evidence**: `editar-light.png` at 1280 already cuts the group, so the overflow
  pre-dates P-024a; the spec `specs/ribbon-collapse-1280.md`; after the fix the
  `--check-layout` lines for the 1440 reach and the 1280 folded menu, the `--selftest`
  `ribbon: 1280 …` and `ribbon: 1440 …` steps, and the 1280/1440 light/dark captures in the
  EGM evidence folder.
- **Touches**: `apps/punchi-fluid-qt/RibbonBar.qml` (the breakpoint, the folded menu's
  Diámetro row, the measured object names), `apps/punchi-fluid-qt/src/main.cpp`
  (`--check-layout` and the `--selftest` step), `docs/ui-ux.md`, `docs/next-steps.md`, this
  register and `CHANGELOG.md`.
- **Options**:
  1. Collapse at 1100. **Pro**: none for this case. **Con**: 1280 is the common small
     laptop width and the fold would come after the clipping starts.
  2. Horizontal scroll or a “more” menu. **Con**: a new overflow model and a second place
     to find every command; the owner rejected both.
  3. Wrap the groups onto a second row. **Con**: the ribbon's height changes and takes room
     from the sheet.
  4. Collapse at 1280 and record the 1440 overflow as a later node. **Pro**: one breakpoint
     where the fold is needed; no second mechanism. **Con**: the unfolded row still does not
     fit at 1440.
- **Decision**: option 4, chosen by the owner on 2026-11-21. At ≤1280 the tab row and the
  **Menú** are the ribbon; the menu gains one **Diámetro: …** row so the Conducto group,
  P-024a's control included, is reachable instead of clipped. `Alt+D` and the status bar do
  not change. At >1280 the current unfolded row stays; its overflow at 1440 is this node's
  known limit and the next node's question.
- **Stages**: one branch, `fix/ribbon-collapse-1280` — the breakpoint, the folded menu row,
  the layout check's 1440 known limit and 1280 folded measurement, the selftest steps, the
  four documents, the captures and the gate.
- **Done when**: `--check-layout` is 0 failures at 1280×800, 1440×900, `QT_SCALE_FACTOR`
  1.25/1.5/2 and `--font-scale 1.5`; the 1280 folded form and the 1440 known limit are
  printed; the `--selftest` 1280/1440 steps pass; `--check-qml` is 0 and `--check-theme`
  green; the light/dark captures at 1280/1440 show the folded Diámetro and the sheet;
  `./scripts-dev/check.sh` is green; the owner confirms the diff.
- **State**: `in progress` on `fix/ribbon-collapse-1280`; `done` after the owner merges.

### P-028 — The ribbon's group row fits 1366×768

- **Problem**: P-027 made the ribbon fold at 1280 but left the unfolded row measuring about
  1875 px. At 1366×768 — the support floor, the entry and education laptop — the content
  is about 1360 px, so Conducto, Repartir and Paleta were clipped past the right edge and
  the Conducto group's **Diámetro** could not be reached. The owner's requirement is that
  the ribbon stays visible and **unfolded** at 1366, not replaced by the reduced menu.
- **Evidence**: the owner's requirement and his choice of the group-collapse pattern after
  using it in Inventor; the P-027 measurements; the spec `specs/ribbon-1366.md`. After the
  fix, `--check-layout` prints the one-row ribbon at 1280 folded, 1366, 1440, 1600 and
  1920 with seven groups and the measured collapse set; `--selftest` proves the 1366 row,
  the `Enter` flyout, the 1920 expansion and the 1280 fold; the window probe proves
  1920×1080 → 1440×900, 1366×768 → 1330×732, 1280×800 → 1244×764 and a restored 1920×1080
  clamped to 1366×768; the captures are `p028-ribbon-1366x768-*`, `…-1440x900-*` and
  `…-1920x1080-*`, light and dark, plus the 1366 flyout.
- **Touches**: `apps/punchi-fluid-qt/RibbonBar.qml` (the one-row layout and the collapse
  priority), `apps/punchi-fluid-qt/RibbonGroup.qml` (the collapsed titled button and the
  flyout from the same content), `apps/punchi-fluid-qt/Main.qml` (the first window's
  screen-aware size and the restored clamp), `apps/punchi-fluid-qt/src/main.cpp`
  (`--check-layout` matrix, the `--selftest` steps, the `--ribbon-flyout` capture hook),
  `docs/ui-ux.md`, `docs/next-steps.md`, this register and `CHANGELOG.md`.
- **Options**:
  1. Two rows of groups. **Con**: rejected by the owner; it is the least common pattern and
     takes ~82 px from the sheet.
  2. Icon-only controls. **Con**: rejected; the labels are what a student reads.
  3. One row with per-group collapse. **Pro**: the seven names stay visible, the height
     stays 121 px, every command keeps a labelled home. **Chosen** by the owner.
- **Decision**: when the row exceeds the content, collapse greedily in the order
  **Repartir, Alinear, Paleta, Historial, Circuito, Componente, Conducto**. A group with no
  command — Paleta's note — is skipped because it has nothing to fly out. The measured set
  is **1366 → Historial, Alinear, Repartir**; **1440/1600 → Alinear, Repartir**;
  **1920 → none**. The collapsed group is its name plus a chevron; its non-modal flyout
  opens with `Enter`/`Space`, closes on `Esc` or a click outside, and carries the same
  commands with their accessible names. The 1440 known-limit exemption is removed. The
  **first window is screen-aware**: 1440×900 on 1080p, the largest size that fits minus a
  two-unit margin on a smaller screen, never below 900×600 and centred; a remembered size is
  clamped to the current screen, so a 1920 window reopens inside a 1366 laptop.
- **Stages**: one branch, `fix/ribbon-1366` — the layout and the group modes, the check
  matrix, the selftest steps and the window probe, the four documents, the captures and the
  gate.
- **Done when**: `--check-layout` is 0 failures at 1280 folded and at 1366/1440/1600/1920
  unfolded, at `QT_SCALE_FACTOR` 1.25/1.5/2 and `--font-scale 1.5`; the `--selftest`
  `ribbon: 1366 …`, `… Enter …`, `… 1920 …` and `… 1280 …` steps and the `window: …` size
  steps pass; `--check-qml` is 0 and `--check-theme` green; the light/dark captures show the
  one row at the three widths and a flyout at 1366; `./scripts-dev/check.sh` is green; the
  owner confirms the diff.
- **State**: `in progress` on `fix/ribbon-1366`; `done` after the owner merges.

### P-030 — The documentation diagrams become editorial SVG

- **Problem**: the documentation explains the system with ASCII diagrams, starting with
  “The shape” in `docs/architecture.md`. They are hard to read, render differently in every
  viewer and cannot carry the project's visual language.
- **Evidence**: `specs/PLAN.md` 3b; the ASCII blocks in `docs/architecture.md` and the
  documents that repeat them.
- **Touches**: `docs/` only, plus the generated SVG files and their origin/licence record.
  It does not touch `crates/punchi-fluid-draw`, `assets/symbols`, the schematic, the
  exported page or the real captures in the README.
- **Decision**: replace the blocks with editorial SVG generated from the
  `diagram-design` fork (MIT) under a Punchi profile: the project's tokens, the desktop
  fonts, the project grid and no shadows. Each figure carries `role="img"` with `title` and
  `desc`, reaches the project's contrast contract, and records its origin and licence. The
  prose remains the source of truth and the SVG is regenerated from it; the text is not
  replaced by a picture.
- **Stages**: one documentation branch per document group, starting with
  `docs/architecture.md`; the text and the figure travel together.
- **Done when**: the named diagrams have an accessible SVG, the origin and licence are
  recorded, `./scripts-dev/check-docs.sh` is green, and the ISO 1219 symbols, the review
  record and the drawing engine are untouched.
- **State**: `planned`; after P-021a and P-028, and independent of P-031 and P-032.

### P-031 — English and translation are guarded

- **Problem**: the English audit of 2026-11-21 found no Spanish titles or file names
  outside the documented exceptions, but nothing in the gate keeps it that way. A new
  Spanish heading or file name would land silently.
- **Evidence**: `specs/reports/language-audit-2026-11-21.md`; the exceptions in `AGENTS.md`
  (`docs/retomar.md`, `CHANGELOG.md`, `README.es.md` and the user-facing interface strings).
- **Touches**: a new `scripts-dev/check-language.sh` and its fixture, the exception list in
  `docs/style.md`, and the gate in `scripts-dev/check.sh`. It does not touch code, the UI or
  the current documentation.
- **Decision**: add the guard, not a rewrite. The script fails on a Spanish title or file
  name outside the documented list and prints the file and line; the list lives in
  `docs/style.md`; the fixture proves a failure and a pass.
- **Stages**: one tooling branch: script, fixture, style note and the gate hook.
- **Done when**: the gate runs the guard, the fixture proves both answers, the real tree
  passes and the exceptions are documented in one place.
- **State**: `planned`; the audit is done, the guard is not.

### P-032 — A Punchi visual-language skill

- **Problem**: the project has the canonical visual documents — `docs/ui-ux.md`, the theme
  adapter, ADR 0028, the ribbon node and the layout skill — but no single route from a
  visual request to the document to read, the command to run and the evidence to attach.
  The risk is a second copy of the values in a skill or a chat.
- **Invariants**: **the documentation diagrams (P-030) and the ISO 1219 symbology and its
  review record (P-018b, the 300 dpi review) are untouched.** This node does not edit
  `crates/punchi-fluid-draw`, `assets/symbols`, the schematic, the exported page,
  `docs/symbology.md` or the review record; it owns chrome-and-documentation procedure.
- **Evidence**: `specs/visual-language.md`; the skill, its five references and the extended
  `check-docs.sh` on `docs/visual-language-skill`; the fixture driver.
- **Touches**: `.agents/skills/punchi-visual-language/` (the skill and five references),
  `.agents/skills/README.md` (the catalog row), `scripts-dev/check-docs.sh` (skill links,
  rows, names, anchors and the `ROOT`-bounded resolve), the new
  `scripts-dev/test-check-docs.sh`, `scripts-dev/check.sh` (the hook), this register and
  `CHANGELOG.md`.
- **Decision**: the skill is procedure and links only. It routes by task (a control, a
  token, an icon, the ribbon, a layout and a review) to the canonical home, the command and
  the evidence, and it carries no literal width, size, contrast, motion or icon geometry.
  The layout row routes to `qt-adaptive-layout`; the icon row routes to ADR 0028 and P-033's
  inventory until `docs/reference/icons.md` lands. The documentation check validates
  relative links, the catalog row, the frontmatter `name` and the `#fragment` against the
  destination's headings, and the fixture proves the failures on a temporary tree. The
  path-resolution security review is recorded in the pull request and in the skill's
  `review.md`.
- **Stages**: one documentation-and-tooling branch, `docs/visual-language-skill`: the skill,
  the catalog row, the check extension, the fixture and the hook, the nodes and the
  changelog. No runtime file.
- **Done when**: `./scripts-dev/test-check-docs.sh`, `./scripts-dev/check-docs.sh` and
  `./scripts-dev/check.sh` exit 0; the fixture proves a broken link, a missing row, a wrong
  anchor and a path escape; the skill's links and anchors resolve; the diagrams, the ISO
  symbols and the exported page are untouched; the owner confirms the diff.
- **State**: `in progress` on `docs/visual-language-skill`; `done` after the owner merges.

### P-033 — The simulator owns its icon family

- **Problem**: the ribbon's line family exists in `actionicons.cpp`, but its inventory is
  not a document and some commands share a drawing that loses the meaning: the duct classes
  share `draw-line`, the speeds share `speedometer`, `check` uses a generic tick,
  `align-ports` is not a port and `tube-diameter` has no drawing. The code and the docs also
  disagree on the count.
- **Evidence**: `specs/icon-family.md`; the 2026-11-21 inventory of `actionicons.cpp`, the
  command table and `--check-icons`.
- **Touches**: `apps/punchi-fluid-qt/src/actionicons.cpp`, the ribbon command table, a new
  `docs/reference/icons.md`, the icon checks and the small-size/dark evidence. It does not
  touch `crates/punchi-fluid-draw`, `assets/symbols`, the ISO 1219 geometry, the schematic,
  the exported page or the review record.
- **Decision**: keep the project-owned family of ADR 0028 and document it. Tabler, Lucide or
  Fluent are inspiration, never a source; an external set would need an ADR and a licence
  record. The inventory covers the real drawings, the render sizes the window uses and the
  gaps, with the shared-shape fixes first, the diameter next and the polish last; the proof
  reads the small render in both schemes and at a higher density.
- **Stages**: the inventory document, then the priority fixes, each on its own branch; the
  guide is P-032.
- **Done when**: `docs/reference/icons.md` matches `actionicons.cpp`, `--check-icons` has no
  gap, the changed drawings read at the small size in light and dark, and the ISO 1219
  symbols and the diagrams are untouched.
- **State**: `planned`; after P-032 and after the owner approves the P0 set.

### P-034 — Inicio tells the truth

- **Problem**: Inicio is the first screen and it does not look like the program nor say what
  it really offers: four template cards include an electrical circuit the simulator cannot
  connect yet, the recent rows say *no disponible* and a path instead of the honest
  **no encontrado**, modified time and real size, and the page stopped one hidden ribbon above
  the 1366×768 floor, so the recents were clipped.
- **Evidence**: the 1366×768 captures before and after; `--check-layout` start-view cases;
  `--selftest` `start:` steps; `specs/evidence/20261121-p034a-*.png`.
- **Touches**: `apps/punchi-fluid-qt/StartView.qml` (two start cards, three template cards,
  five recent rows and the empty state), `Main.qml` (the rail, `Volver a la hoja`, `Esc`, and
  the region measured with the visible bars only), `src/session.cpp`/`.h` (modified, size,
  available from `QFileInfo`; no new ABI), `src/main.cpp` (the adapted template checks, the
  `--selftest` steps and the layout cases), `docs/ui-ux.md` §2.3, ADR 0031's amendment, this
  register and `CHANGELOG.md`. It does not touch the ABI, the diagrams, the ISO 1219 symbols,
  the sheet, the zoom, the active fill or `.pfl`.
- **Decision**: three cards are offered and the compiled fourth stays for P-026; the page is
  a Flickable that fills the window; recents are five rows with modified/size and a
  `quitar` that never deletes; `Esc` leaves the page; the domain chip, the thumbnails and the
  opened time wait for the data in P-034b.
- **Stages**: `feat/start-view-visual` (P-034a) delivers the page, the recents, the checks,
  the captures and the docs; P-034b follows with the domain model, the preview-by-path entry
  and the recent format change.
- **Done when**: the 1366 captures fill the window and show the recent rows, a missing row
  says **no encontrado** with `quitar`, the empty state names the next step, `--check-qml`,
  `--check-theme`, `--check-layout` (1280/1366/1440/1600), `--selftest` and
  `./scripts-dev/check.sh` exit 0, and the owner confirms the diff.
- **State**: P-034a `in progress` on `feat/start-view-visual`, `done` after the owner merges;
  P-034b `planned`.

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
