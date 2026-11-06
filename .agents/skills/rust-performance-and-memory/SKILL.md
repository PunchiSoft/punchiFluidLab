---
name: rust-performance-and-memory
description: Memory and frame-time budgets, allocation-free hot paths and bounded caches. Use when writing the solver step or draw loop, adding a cache, profiling, or when a change may move a budget.
license: GPL-3.0-or-later
---

# Performance and memory

The budgets are in [docs/engineering.md](../../../docs/engineering.md). They are
promises with measurements behind them, not aspirations. A change that moves one says
so in the pull request, with the before and after numbers.

## Budgets

| Item | Budget | How it is held |
|---|---|---|
| Steady-state RSS, empty document, 1080p | < 120 MB | measured at release |
| Symbol geometry cache | ≤ 64 MB, LRU by (definition id, size bucket) | bound in code, test fills past it |
| Undo history | ≤ 100 entries, ≤ 32 MB | `Arc<Document>` snapshots, oldest dropped |
| Telemetry ring | 10 Hz × 60 s × 32 B ≈ 19 KB | fixed capacity |
| Icon cache | ≤ 200 entries per (size, tint) | LRU |
| Export bitmap | ≤ 80 Mpx, DPI 72–600, one page | refused before allocation |

Targets: 60 fps at 200 components with p95 frame CPU < 6 ms; solver step < 4 ms;
cold start < 400 ms; catalog reload < 50 ms; A4 at 300 dpi < 1.5 s.

## Procedure

1. **Measure before changing.** Run the release build, reproduce the scenario, record
   the number and the method. A performance claim without a measurement is a guess.
2. **Find the allocation or the copy**, not the line count. In the solver, look for a
   `Vec` built per step, a `String` from a port id, or a `format!` in a loop. In the
   draw loop, look for a per-frame `Vec`, a font layout cache miss, or a clone of
   geometry.
3. **Reuse the buffer**: `clear()` then `extend_from_slice`, or a fixed-capacity
   structure. The simulation state is a fixed set of fields.
4. **Bound every cache** with a capacity and an eviction rule (LRU, or drop the oldest).
   Key it by something stable: definition id plus a size bucket, never a floating-point
   zoom level.
5. **Optimise one thing** behind a measurement. Then re-measure with the same scenario
   and record both numbers in the pull request and in `CHANGELOG.md`.
6. **Check the budget test** still passes; a cache that is correct but unbounded is a
   leak with better naming.

## Rules

- **No allocation in the solver step or the per-frame draw loop.** This is testable with
  a counting global allocator: zero allocations after warm-up.
- **`Arc` for shared immutables**, never `Rc<RefCell<…>>`. A cell mutated during a frame
  is a design smell, not an optimisation.
- **No global mutable state.** Theme, catalog and document are owned by the application
  and passed down.
- **`Drop` releases the expensive thing first**: textures and pixmaps before
  bookkeeping.
- **Every cache has a bound and an eviction rule.** A cache without one is a leak.

## Gotchas

- **The instrument may not exist yet.** `docs/engineering.md` specifies a
  `--trace-frames` flag; grep for it before quoting output. If it is not wired, wiring
  it is part of the work — do not invent a number.
- **Debug builds lie.** `cargo run` without `--release` can be 10–50× slower; never
  quote a debug measurement as a budget result.
- **Caches keyed by a float change every frame.** Zoom level, layout height and time are
  the classic leaks; bucket them or do not cache them.
- **`clone()` on an `Arc` is cheap; cloning the contents is not.** Check what is inside
  before celebrating.
- **`HashMap` rehashing** is an allocation in the middle of a frame; reserve once or use
  a sorted `Vec` for small maps.
- **Do not optimise a path no test covers.** If the scenario cannot be reproduced, there
  is nothing to improve.
- **`perf`, `heaptrack` and `criterion` are not in the repository.** Use them if they
  are installed, and state the tool and version in the measurement.

## Validation

```bash
cargo build --release --workspace
cargo test --workspace
./scripts-dev/check.sh
```

The allocation test is the machine check; the budget table is the human one. Both must
agree, and the changelog entry records the scenario, the tool and the numbers.
