# Work queue: what the parity run does, in order

Status: the authority for **order and scope** of the long branch `feat/parity-2` (run 2;
run 1 was delivered as #57), chosen by the owner on 2026-11-13 (delivery option **c**: one
long branch, one commit per capability, one pull request at the end). It does not replace [capabilities.md](capabilities.md), which says
what a capability *is* and how it is proven; this file says which one comes next and what
"done" means for the session doing the work.

## Rules of the run

1. **One commit per capability**, on `feat/parity-2`, with its test and its documentation in
   the same commit.
2. **The gate runs on the branch before each commit**: `./scripts-dev/check.sh` green. A red
   gate stops the run until it is fixed on the same branch.
3. **Nothing is merged by the agent.** The run ends with one pull request against `main`.
4. **No copying.** Behaviour is replicated from the standards and from the reference pages in
   this folder; no symbol, manual, example, name or file format from another program enters
   the repository (ADR 0009).
5. **A capability that needs a decision stops the run** and becomes a note in the report,
   never a guess.
6. **Rule of two attempts**: if a capability fails the gate twice for the same reason, the
   run leaves it and moves to the next one, recording why.

## The queue

The state is `ready`, `doing`, `done` or `blocked`. A `blocked` entry names what unblocks it.

| # | Capability | State | Acceptance |
|---|---|---|---|
| 1 | The ledger tells the truth; the ninth fault is the short circuit of the control rail | done | a new `short_circuit` finding with a test per side (L–N through a closed contact is reported; a live line with no return is not), and the stale ledger rows corrected |
| 2 | Relay: a coil that closes a contact | done | a relay file with a coil and a contact; the contact closes only while the coil is live; ABI test with a lamp or a valve behind it |
| 3 | Timers TON and TOF, bounded | done | a timer file with `kind = "ton"`/`"tof"` and a delay; the output changes after the declared time and never allocates per step; a test with a fixed step count |
| 4 | Indicator lamp | done | a one-terminal lamp lit while its net is live; the drawing shows lit/unlit as rays and the word beside it, never colour alone; ABI query and a drawing test |
| 5 | Buzzer with a continuous sound | done | an audible indicator with one terminal; it sounds while its net is live and stops when it is not; a generated `buzzer.wav`, the sample decoding in `--check-sounds`, and a session test that starts and stops it |
| 6 | Limit switch, driven by a cylinder at an end | done | a contact whose state follows a chosen cylinder reaching an end; the cylinder is chosen in the inspector and saved in the placement source; a test with a stroke that closes it |
| 7 | Pressure switch and vacuum switch | done | a contact that closes above a declared pressure and opens below it, read from a run; the vacuum switch does the same on the other side of atmosphere; tests with a rising and a falling line |
| 8 | Manual switches: selector, maintained, mushroom and a normally-closed contact | done | selector and maintained switches keep their state, the mushroom is a latching NC contact, and an NC contact file passes the catalog lint; ABI test with a lamp |
| 9 | Latching relay: set and reset coils | done | a file with two coils that holds its state after both are released; a test that sets, releases, resets |
| 10 | Counter relay | ready | counts rising edges up to a preset, resets on its reset coil; a test with the preset and one pulse more |
| 11 | Digital library: AND, OR and NOT as files | ready | three files sharing one behaviour variant (ADR); the boolean net computes their output; a truth-table test per gate |
| 12 | Digital: RS latch and edge detection | ready | a latch holds its output between pulses; a rising edge fires once per press; two tests |
| 13 | GRAFCET: steps, transitions and actions | ready | an ADR and a first slice: a sequence drawn from files, one step active, transitions advance it; a test that walks the sequence |
| 14 | Full symbol key in the printed legend | ready | every built-in symbol printed once with its name, bounded to the page, with an export test |
| 15 | Momentary press and release gesture | ready | a push button closes while held and opens on release; the ABI gains press/release for a contact, the test drives both edges, and the drawing follows |
| 16 | Multi-position selector | ready | a selector file with more than two positions drives one contact per position; the file names the connections, and a test turns it through every position |
| 17 | Vacuum source and dynamic vacuum test | ready | a vacuum generator file pulls a line below atmosphere; the vacuum switch closes under it and opens when the line returns; a test with pressure falling and rising |
| 18 | The 300 dpi review of every built-in symbol | blocked | unblocked by a screenshot pass at 300 dpi in both themes; needs the maintainer's reader |
| 19 | Hydraulic model and library | blocked | needs its own ADR and phase: fluid properties, incompressible network, symbols; it is not a slice |
| 20 | DC motor and generator | blocked | needs a mechanical domain and an electrical machine model, each with its own ADR; a spinning box without a load would not be a model |
| 21 | Voltmeter, ammeter and analogue pressure sensor | blocked | the boolean net carries liveness, not voltage or current; unblocked by a voltage/current model with its own ADR |
| 22 | Force sensor | blocked | needs a mechanical domain: `p·A` is not a force measurement, so the sensor waits with the force meter |

## Detail of the remaining entries

This is the work order a session follows without asking. Each entry names the layer it
touches, the decision it may need and the test that proves it. An entry that grows beyond
its slice is split, and the split is written here before the code.

### 10 — Counter relay

- **Schema**: `Behaviour::Counter { preset: u32 }`, a count-coil pair, a reset-coil pair and
  a contact pair. The preset is bounded (1–9999) and validated.
- **Model**: a rising edge on the count coil increments a run counter capped at the preset;
  the reset coil clears it; the contact closes at the preset. The edge is detected against
  the previous step, so a held button counts once. The state lives in the same cached model
  and the same bounded settle.
- **Catalog**: one file with the counter mark; the bar is the renderer's.
- **Evidence**: `a_counter_counts_to_its_preset_and_resets` (a button, N pulses, the contact
  at the preset and one pulse more), the symbol review, the catalog lint.
- **Decision**: none; IEC 61131-3 CTU is the reference.

### 11 — Digital library: AND, OR and NOT

- **Decision first**: ADR 0026, *a gate drives the net*. The current model has one kind of
  source, the rail. A gate is a second: its output is a seed when its inputs satisfy the
  truth table. The ADR fixes the rule for a driven output that is also wired to a rail, and
  says the check will report two drivers as a fault in a later slice.
- **Schema**: `Behaviour::Gate { kind: And | Or | Not }`, two input terminals (one for NOT)
  and one output terminal.
- **Model**: each settle pass adds the live outputs of the gates to the seeds, iterating with
  the relays/gates until stable; the bound is the number of switches plus gates.
- **Catalog**: three files with the IEC 60617 gate shapes; the output terminal is the only
  driver.
- **Evidence**: one truth-table test per gate over the ABI, the three symbol reviews, the
  catalog lint.

### 12 — Digital: RS latch and edge detection

- **Schema**: `Behaviour::Digital { kind: Rs | RisingEdge }`; the RS has set/reset inputs and
  one output, the edge has one input and one output.
- **Model**: the RS holds its output between pulses; the edge output is live for exactly one
  step after a rising input. Both use the latch map and the step counter already in place.
- **Catalog**: two files; the reviews record the pulse width of the edge.
- **Evidence**: `an_rs_latch_holds_between_pulses`, `a_rising_edge_fires_once_per_press`.

### 13 — GRAFCET: steps, transitions and actions

- **Decision first**: an ADR for the first slice. The smallest honest slice is a sequence of
  *step* files joined by *transition* files, with one active step and one action per step.
  A full GRAFCET editor is a phase of its own; the ADR says what the slice leaves out.
- **Schema**: `Behaviour::GrafcetStep`, `Behaviour::GrafcetTransition { condition }` and
  `Behaviour::GrafcetAction { … }` with the step/transition linking stored in the placement
  source or a small sequence table.
- **Model**: a run state with one active step; a transition fires when its condition is live
  and advances the active step; the action of the active step drives its output.
- **Catalog**: the step, transition and action shapes.
- **Evidence**: `a_sequence_walks_three_steps` (a pulse per transition, one active step at a
  time), the symbol reviews, the ADR.

### 14 — Full symbol key in the printed legend

- **Layer**: `punchi-fluid-export`.
- **Work**: every built-in definition printed once with its name, in a bounded column that
  takes room from the drawing; the existing title block and parts list stay.
- **Evidence**: an export test that finds every built-in name once and the column bounded to
  the page; a rendered A4 at 300 dpi.

### 15 — Momentary press and release

- **Layer**: `punchi-fluid-ffi` and the window.
- **Work**: `pf_session_press_contact` and `pf_session_release_contact` for a momentary
  contact; the mouse-down/mouse-up and keyboard press/release gestures call them; the
  existing `toggle_contact` stays for maintained parts.
- **Evidence**: an ABI test that closes while pressed and opens on release; the window
  `--selftest` step; no document change.

### 16 — Multi-position selector

- **Layer**: catalog, model and drawing.
- **Work**: a file with a common terminal and N position terminals, the same behaviour as a
  contact for each position, and a state per position. The click advances the position; the
  drawing shows the knob at the active one.
- **Evidence**: a test that turns through every position and closes the matching contact; the
  catalog lint; the review.

### 17 — Vacuum source and dynamic vacuum test

- **Layer**: catalog, solver and ABI.
- **Work**: a vacuum generator file with a negative supply pressure (or a dedicated
  `VacuumSupply` behaviour), the solver already handles a pressure below atmosphere; the
  vacuum switch then closes under it.
- **Evidence**: `a_vacuum_switch_closes_under_a_vacuum_source` (pressure falls below −0.4 bar
  and rises again), the symbol review, the catalog lint.

## The electrical palette as scope

The owner supplied a capture of the reference's electrical palette on 2026-11-13. It is used
as a **checklist of which components exist**, never as a source of geometry, names, files or
icons: every symbol here is drawn by hand against IEC 60617, and nothing is traced from that
image. Read that way, the palette adds the pressure and vacuum switches, the selector and
maintained switches, the mushroom button, the latching relay, the counter relay, the lamp and
buzzer, the DC motor and generator, and the voltmeter and ammeter. The queue above schedules
the ones whose behaviour this model can decide, and leaves blocked — with the reason — the
ones that need a domain we do not have yet.

The palette also shows a **0 V reference** beside the 24 V supply. Here the return is the `N`
terminal every supply already carries, so no separate file is queued until a circuit needs a
second reference point; inventing one now would duplicate the terminal that exists.

## The loop, without stopping

For every entry in order:

1. `git switch feat/parity-2`; `git status --short` must be clean.
2. Mark the entry `doing` in this file and in [parity-run.md](parity-run.md).
3. Implement in layer order: catalog file, model, drawing, ABI, window, document only if a
   format change is unavoidable (it needs a migration).
4. Write the evidence the entry names, and the documentation in the same commit: the
   capability ledger, the ADR when a decision is new, the catalog guide when a field is new,
   `CHANGELOG.md` in Spanish.
5. `./scripts-dev/check.sh`; green before the commit. A red gate is fixed on the branch.
6. Commit once with `-s`, push, and update this file and the continuity note in that same
   commit (amend the message if the hash is not known yet).
7. Take the next `ready` entry. Do not open the pull request yet.

**After a context reset** the repository is the memory: read
[parity-run.md](parity-run.md), this file and `git log --oneline main..HEAD`; the first
`ready` entry is the next step. Never rebuild from a summary of the conversation.

**Stopping is only allowed** when: an entry needs an owner decision (the three at the end of
the queue), the same gate fails twice for the same reason, or every remaining entry is
`blocked`. A context reset is not a stop: the note and the queue carry the state.

**Closing run 2**: full gate, final state in the note, push, then one pull request from
`feat/parity-2` to `main` with a section per capability (what was built, the test, what
remains). The owner merges; the agent never does.

## The report at the end

The pull request body carries, per capability: what was built, the test that proves it, and
what remains. An entry left `blocked` or `ready` is named there, never hidden.
