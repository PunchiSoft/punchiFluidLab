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
| 4 | Indicator lamp | ready | a one-terminal lamp that is lit while its net is live; the drawing shows lit/unlit as shape and text, never colour alone |
| 5 | Limit switch, driven by a cylinder at an end | ready | a contact whose state follows a chosen cylinder reaching an end; a test with a stroke that closes it |
| 6 | Digital library: AND, OR and NOT as files | ready | three files sharing one behaviour variant (ADR); the boolean net computes their output; a truth-table test per gate |
| 7 | Digital: RS latch and edge detection | ready | a latch holds its output between pulses; a rising edge fires once per press; two tests |
| 8 | GRAFCET: steps, transitions and actions | ready | an ADR and a first slice: a sequence drawn from files, one step active, transitions advance it; a test that walks the sequence |
| 9 | Full symbol key in the printed legend | ready | every built-in symbol printed once with its name, bounded to the page, with an export test |
| 10 | The 300 dpi review of every built-in symbol | blocked | unblocked by a screenshot pass at 300 dpi in both themes; needs the maintainer's reader |
| 11 | Hydraulic model and library | blocked | needs its own ADR and phase: fluid properties, incompressible network, symbols; it is not a slice |

## The report at the end

The pull request body carries, per capability: what was built, the test that proves it, and
what remains. An entry left `blocked` or `ready` is named there, never hidden.
