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
| P-005 | No way back to Inicio with a project open | `planned` | [§5](start-view-corrections.md) |
| P-006 | Recents have no page bound and do not mark a missing file | `in progress` | [§6](start-view-corrections.md) |
| P-007 | Closing the last project does not return to Inicio | `planned` | [§7](start-view-corrections.md) |
| P-008 | The rail is uneven and its focus is not the catalogue's | `in progress` | [§8](start-view-corrections.md) |
| P-009 | `electrical.relay.latching`: A2 and R1 collide at 300 dpi | `reported` | [review record](../assets/symbols/review/electrical.relay.latching.md) |
| P-010 | The shuttle and the two-pressure valve are missing (OR/AND) | `reported` | [capabilities F2](capabilities.md) and ADR 0029 |
| P-012 | The theme icon cache is unbounded | `in progress` | [icons.cpp](../apps/punchi-fluid-qt/src/icons.cpp) — bounded `QCache` on `feat/start-view-visual-pass-2`: cap 73, working set 58, max group 21, LRU and palette key |
| P-013 | The icon cache can show the previous theme's tint because QML never re-requests it | `in progress` | [StartView.qml](../apps/punchi-fluid-qt/StartView.qml) and [session.cpp](../apps/punchi-fluid-qt/src/session.cpp) — `&v=<themeRevision>` in every `image://theme` URL |

## A node, filled

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
