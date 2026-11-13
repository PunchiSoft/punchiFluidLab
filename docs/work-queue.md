# Work queue: what the parity run does, in order

Status: the authority for **order and scope** of the long branch `feat/parity`, chosen by the
owner on 2026-11-13 (delivery option **c**: one long branch, one commit per capability, one
pull request at the end). It does not replace [capabilities.md](capabilities.md), which says
what a capability *is* and how it is proven; this file says which one comes next and what
"done" means for the session doing the work.

## Rules of the run

1. **One commit per capability**, on `feat/parity`, with its test and its documentation in
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
| 2 | Relay: a coil that closes a contact | ready | a relay file with a coil and a contact; the contact closes only while the coil is live; ABI test with a lamp or a valve behind it |
| 3 | Timers TON and TOF, bounded | ready | a timer file with `kind = "ton"`/`"tof"` and a delay; the output changes after the declared time and never allocates per step; a test with a fixed step count |
| 4 | Indicator lamp and buzzer | ready | a one-terminal lamp lit while its net is live, and an acoustic indicator that sounds while it is; the drawing shows lit/unlit as shape and text, never colour alone |
| 5 | Limit switch, driven by a cylinder at an end | ready | a contact whose state follows a chosen cylinder reaching an end; a test with a stroke that closes it |
| 6 | Pressure switch and vacuum switch | ready | a contact that closes above a declared pressure and opens below it, read from a run; the vacuum switch does the same on the other side of atmosphere; tests with a rising and a falling line |
| 7 | Manual switches: selector, maintained, mushroom and a normally-closed contact | ready | selector and maintained switches keep their state, the mushroom is a latching NC contact, and an NC contact file passes the catalog lint; ABI test with a lamp |
| 8 | Latching relay: set and reset coils | ready | a file with two coils that holds its state after both are released; a test that sets, releases, resets |
| 9 | Counter relay | ready | counts rising edges up to a preset, resets on its reset coil; a test with the preset and one pulse more |
| 10 | Digital library: AND, OR and NOT as files | ready | three files sharing one behaviour variant (ADR); the boolean net computes their output; a truth-table test per gate |
| 11 | Digital: RS latch and edge detection | ready | a latch holds its output between pulses; a rising edge fires once per press; two tests |
| 12 | GRAFCET: steps, transitions and actions | ready | an ADR and a first slice: a sequence drawn from files, one step active, transitions advance it; a test that walks the sequence |
| 13 | Full symbol key in the printed legend | ready | every built-in symbol printed once with its name, bounded to the page, with an export test |
| 14 | The 300 dpi review of every built-in symbol | blocked | unblocked by a screenshot pass at 300 dpi in both themes; needs the maintainer's reader |
| 15 | Hydraulic model and library | blocked | needs its own ADR and phase: fluid properties, incompressible network, symbols; it is not a slice |
| 16 | DC motor and generator | blocked | needs a mechanical domain and an electrical machine model, each with its own ADR; a spinning box without a load would not be a model |
| 17 | Voltmeter, ammeter and analogue pressure sensor | blocked | the boolean net carries liveness, not voltage or current; unblocked by a voltage/current model with its own ADR |
| 18 | Force sensor | blocked | needs a mechanical domain: `p·A` is not a force measurement, so the sensor waits with the force meter |

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

## The report at the end

The pull request body carries, per capability: what was built, the test that proves it, and
what remains. An entry left `blocked` or `ready` is named there, never hidden.
