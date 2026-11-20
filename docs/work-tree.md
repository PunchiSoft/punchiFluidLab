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
| P-001 | The start-view icons read as smudges | `planned` | [start-view-corrections §1](start-view-corrections.md) |
| P-002 | Card text runs into its icon | `planned` | [§2](start-view-corrections.md) |
| P-003 | Template pictures are tiny and pale | `planned` | [§3](start-view-corrections.md) |
| P-004 | Recent rows overlap | `planned` | [§4](start-view-corrections.md) |
| P-005 | No way back to Inicio with a project open | `planned` | [§5](start-view-corrections.md) |
| P-006 | Recents have no page bound and do not mark a missing file | `planned` | [§6](start-view-corrections.md) |
| P-007 | Closing the last project does not return to Inicio | `planned` | [§7](start-view-corrections.md) |
| P-008 | The rail is uneven and its focus is not the catalogue's | `planned` | [§8](start-view-corrections.md) |
| P-009 | `electrical.relay.latching`: A2 and R1 collide at 300 dpi | `reported` | [review record](../assets/symbols/review/electrical.relay.latching.md) |
| P-010 | The shuttle and the two-pressure valve are missing (OR/AND) | `reported` | [capabilities F2](capabilities.md) and ADR 0029 |

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
