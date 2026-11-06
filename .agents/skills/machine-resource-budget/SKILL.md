---
name: machine-resource-budget
description: 'Building, testing and diagnosing this project on a 15 GiB laptop: the cargo job profile, the per-binary memory ceiling and runaway-allocation diagnosis. Use when a build or test run kills the session, the desktop slows down, a loop may allocate without a bound, or an idle window keeps redrawing.'
license: GPL-3.0-or-later
---

# Fitting the work into the machine

The development laptop is small enough that a single careless loop takes the whole
session down. This skill is the budget, the tooling that enforces it, and the procedure
for finding the culprit when something slips through.

## The machine

Measured on 2026-11-06 from `/proc/meminfo`, `nproc` and `kscreen-doctor`:

| Item | Value | Consequence |
|---|---|---|
| Threads | 8 | two rustc jobs, not eight |
| RAM | 15 GiB | a quarter (3.8 GiB) is the ceiling for one test binary |
| Swap | 8 GiB zram | slow, and it does not save a runaway allocation |
| cgroup memory limit | none | the kernel kills the biggest process, often the editor |
| Panel | 1920×1080 @ 144 Hz | an idle redraw costs 144 composites per second |

The last row is the one people misread: `nvidia-smi` showing 78–100 % utilization with
**7 W and 47 °C at base clocks** is the compositor presenting frames, not compute. Judge
GPU load by power draw and clock, never by the utilization percentage alone.

## The budget

| Setting | Where | Value |
|---|---|---|
| `[build] jobs` | `.cargo/config.toml` | 2 |
| `profile.dev.debug` | `.cargo/config.toml` | `1` — line tables only |
| `PUNCHI_BUILD_JOBS` | `scripts-dev/limits.sh` | `nproc / 4`, clamped to 4 |
| `PUNCHI_TEST_MEMORY_KB` | `scripts-dev/limits.sh` | a quarter of RAM, floor 1 GiB |

`scripts-dev/limits.sh` derives both from the machine and exports `CARGO_BUILD_JOBS`, so
a bigger machine gets a bigger budget without editing anything. `test.sh` runs every test
binary through `run_capped`, and `check.sh` calls `test.sh` rather than `cargo test` so
the gate cannot skip the ceiling.

## Procedure: the normal loop

```bash
./scripts-dev/test.sh                  # every binary, capped, one at a time
./scripts-dev/test.sh -p punchi-fluid-solver
./scripts-dev/test.sh valve             # filter by name
```

`test.sh` builds with `--no-run`, records the binaries and then executes them one by one.
The ordering is deliberate: one binary at a time means one process at a time can be the
biggest, which makes an out-of-memory kill unambiguous.

## Procedure: the session died

1. **Confirm it was the kernel, not a panic.**
   ```bash
   journalctl -k -n 50 --no-pager | grep -i "oom\|killed process"
   ```
   The line names the process. `anon-rss` is what it actually allocated; a figure in the
   gigabytes means one process ran away and the rest of the log is noise.
2. **Find the binary.** The name in the `oom-kill` line is the test binary: `punchi_fluid_dr`
   is `punchi-fluid-draw`. The crate tells you which module to read.
3. **Bisect inside the binary, not the workspace.**
   ```bash
   B=$(ls target/debug/deps/punchi_fluid_draw-* | head -1)
   ( ulimit -v 2000000; "$B" --test-threads=1 )
   ```
   Serial execution prints the name of the test that dies; parallel execution does not,
   because the output of the killed test is lost.
4. **Read the failing test's code path for a loop without progress.** The 2026-11-06 crash
   was `dashes()` in `punchi-fluid-draw`: a dash phase that landed on a boundary made the
   step round to zero in `f32`, so `travelled` never advanced and the loop appended a point
   per iteration.
5. **Fix the invariant, then add the bound.** Both, not either: the progress floor makes it
   correct, and the iteration budget makes a future defect cost a wrong drawing instead of
   the machine.

## Rules this repository follows because of that crash

- **Every loop over geometry carries a budget** derived from its input length, and a step
  that cannot advance the position is not a loop iteration.
- **Never allocate in proportion to input without a ceiling.** The catalog is untrusted
  input; so is a circuit file.
- **A window redraws when something changed, not on a timer.** `FluidSession::settled`
  stops the sheet loop after half a second of stillness; measured, an idle window went
  from 238 paints in four seconds to 1.
- **Fixed-capacity buffers in hot paths**, per
  [rust-performance-and-memory](../rust-performance-and-memory/SKILL.md).

## Gotchas

- **`ulimit -v` caps the compiler too.** Never wrap `cargo build` in it: `rustc` and the
  linker legitimately want more virtual memory than a test binary. Cap the test binaries
  and limit the compiler with `CARGO_BUILD_JOBS` instead.
- **zram is not RAM.** It compresses into memory; a runaway allocation consumes the
  original pages first.
- **A green `cargo test --workspace` is not evidence the suite is small.** It is one
  process per binary with the default thread count; measure with `test.sh`.
- **Do not raise `jobs` to "compile faster" here.** The gain is small and the cost is the
  session.
- **The Qt application is a separate build.** `cmake --build apps/punchi-fluid-qt/build`,
  with that directory ignored. Its link step needs the Rust static library, which is why
  the CMake target depends on `cargo build --release -p punchi-fluid-ffi`.

## Validation

```bash
bash -n scripts-dev/limits.sh && bash -n scripts-dev/test.sh && bash -n scripts-dev/check.sh
bash -c 'source scripts-dev/limits.sh; echo "$PUNCHI_BUILD_JOBS $PUNCHI_TEST_MEMORY_KB"'
./scripts-dev/test.sh                                        # every binary, all green
apps/punchi-fluid-qt/build/punchi-fluid --report-paints 4     # a settled window: 1 paint
journalctl -k -n 20 --no-pager | grep -ci oom                 # 0 after a full run
```

The paint count is the evidence for the redraw rule; an empty `oom` count after a complete
run is the evidence for the budget.
