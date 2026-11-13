# The parity run: continuity note

Status: **run 2 active**, branch `feat/parity-2`. Run 1 was delivered as
[#57](https://github.com/PunchiSoft/punchiFluidLab/pull/57) in `6def1d2`. This file is the
memory of the run. After a context
reset, a compaction or a new session, read this file first, then
[work-queue.md](work-queue.md), then `git log --oneline main..HEAD`.

It is written for the agent that continues the work, not for the public README. The
authority for what a capability *is* stays in [capabilities.md](capabilities.md); the queue
says what comes next; this file says where the run stands and what was decided.

## The authorization

The owner chose delivery option **c** on 2026-11-13: *one long branch, one commit per
capability, one pull request at the end*. That is the whole permission. It does **not**
include merging: the owner merges, the agent never does.

Run 2 was authorized the same way on 2026-11-13, after the owner merged run 1 (#57).

## Where things are

| Thing | File |
|---|---|
| Order, scope and acceptance of each capability | [work-queue.md](work-queue.md) |
| What a capability is, its state and its evidence | [capabilities.md](capabilities.md) |
| The control circuit: library and model | [adr/0017](adr/0017-the-electrical-circuit-is-a-second-domain.md), [adr/0018](adr/0018-the-control-circuit-is-a-boolean-net.md) |
| The sheet format | [reference/sheet-format.md](reference/sheet-format.md) |
| The loop and its gotchas | [.agents/skills/parity-run](../.agents/skills/parity-run/SKILL.md) |

## Progress log — run 1 (delivered as #57)

One line per commit, newest last. Each entry names the capability, the commit and the test
that proves it.

| # | Capability | Commit | Evidence |
|---|---|---|---|
| — | Queue, skill and this note | `a19850c` | `scripts-dev/check-docs.sh` |
| 1 | Ninth fault: the short circuit of the rail, and the ledger corrected | `feat(check): short circuit of the rail` | `short_circuit_tests::*`, the CLI reference and the reconciled ledger |
| 2 | Relay: a coil that closes a contact | `feat(catalog): a relay closes its contact from its coil` | `a_relay_names_two_coil_and_two_contact_terminals`, `a_relay_closes_its_contact_while_its_coil_is_live`, the symbol review |
| 3 | Timers TON and TOF, counted by the step | `feat(catalog): count TON and TOF timers with the step` | `a_timer_declares_a_bounded_delay_and_an_instant_relay_does_not`, `a_ton_contact_closes_after_its_delay`, `a_tof_contact_opens_after_its_delay` |
| 4 | Indicator lamp | `feat(catalog): let a lamp read the control line` | `a_lamp_lights_while_its_line_is_live`, `a_lamp_shows_its_state_as_rays_not_only_colour`, the symbol review |
| 5 | Buzzer with a continuous sound | `feat(audio): loop the buzzer while its line is live` | `a_buzzer_sounds_while_its_line_is_live`, `--check-sounds`, the symbol review |

## Progress log — run 2

| # | Capability | Commit | Evidence |
|---|---|---|---|
| 6 | Limit switch at a cylinder end (done in run 1) | #57 | `a_limit_switch_closes_at_the_end_of_its_cylinder`, the two symbol reviews |
| 7 | Pressure and vacuum switches | `feat(catalog): read a line with a pressure switch` | `a_pressure_switch_follows_its_line_up_and_down`, the two symbol reviews |
| 8 | Manual family: maintained, selector, mushroom, NC button | `feat(catalog): add the manual switch family` | `a_normally_closed_contact_opens_when_clicked`, the four symbol reviews |
| 9 | Latching relay: set and reset | `feat(catalog): hold a latching relay between two pulses` | `a_latching_relay_holds_until_the_reset_coil`, the symbol review |

## Decisions taken in this run

Recorded so a later session does not reopen them:

1. **The ninth check fault is the short circuit of the control rail**, not the reference's
   "labels that do not match". Our control circuit is wired: a coil and its contact are joined
   by a wire, and a label is a name drawn under the symbol. The reference couples them by name
   because its model needs to; ours does not, so the label fault is **not applicable** and the
   ninth fault is the one that exists under our rules: the live rail and its return in the same
   net through wires and closed contacts.
2. **A relay is a coil plus a contact on the same boolean net, in one file**: the coil is
   energised when exactly one of its two terminals is live, and the net settles in bounded
   passes so a relay can feed another or itself. No delay: a timer is a separate part.
3. **Timers follow IEC 61131-3**: `TON` (on-delay) and `TOF` (off-delay), implemented as a
   bounded count of steps, never a sleep, and tested with a fixed step count.
4. **Labels are drawn under the symbol** (already true) and never carry electrical meaning.
5. **The LED/lamp, the limit switch and the encoder are separate capabilities**, each with its
   own file and test, not one generic "output".
6. **The lamp and the buzzer are separate capabilities.** The lamp shipped as a visual state;
   a continuous sound needs the audio path and its own decoding test, so the queue carries
   the buzzer on its own rather than calling a silent lamp a buzzer.
7. **The reference is a guide to behaviour, not to appearance.** The owner supplied captures
   and a recording of the reference on 2026-11-13; the ribbon, the animations, the Rust
   workspace and the drawing are ours and stay. No symbol, icon, name or file from that
   program enters the repository: the palette is read as a checklist of components, and every
   symbol is drawn by hand against IEC 60617.

## Open questions for the owner

- **Hydraulics** is announced by the product and does not exist: it needs its own ADR and
  phase (fluid properties, incompressible network, hydraulic symbols). It is not a slice.
- **The 300 dpi review** needs a second reader's signature; the run cannot produce it.
- **Remembering the export panel's settings** per user is a small improvement, not yet
  scheduled.

## Next entry: 10, the counter relay

The latch is done; the next entry counts pulses.

- **Schema**: a `Behaviour::Counter { preset: u32 }` with a count coil, a reset coil and one
  contact; the count is a run artefact bounded by the preset.
- **Model**: a rising edge on the count coil increments (capped at the preset), the reset coil
  clears it, and the contact closes at the preset. The edge is detected against the previous
  step, so a held button counts once.
- **Catalog**: one file with the counter mark; the bar is the renderer's.
- **Evidence**: an ABI test with a button, N pulses, the contact at the preset and one pulse
  more; the symbol review and the lint.

## How to continue after a reset

1. `git status --short` must be clean; if not, the previous capability was interrupted: read
   `git diff` and finish it before anything else.
2. `git switch feat/parity-2` and `git --no-pager log --oneline main..HEAD` — one commit per
   capability, in queue order.
3. Open [work-queue.md](work-queue.md): take the first `ready` entry.
4. Follow the loop in the [parity-run skill](../.agents/skills/parity-run/SKILL.md): implement,
   test, document, gate, one commit, update the queue and this table.
5. Do not open a pull request until the queue is empty or only `blocked` entries remain.

## Gotchas that already cost time

- `scripts-dev/check.sh` runs the window when the build is configured: a QML icon that does not
  exist fails `--check-qml` even with Rust green.
- The architecture check fails a source file over 800 lines; move tests into `#[path]` modules.
- Clippy denies `unwrap`, `expect`, `panic!` and `too_many_lines` in production code; extract a
  helper rather than adding an `allow`.
- `check-docs.sh` requires every ADR and every document to be linked from its index.
- The changelog is Spanish, the rest is English; a doc-only pull request runs `check-docs.sh`
  and skips the window in CI.
