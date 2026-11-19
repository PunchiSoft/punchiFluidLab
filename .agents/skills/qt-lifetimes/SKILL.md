---
name: qt-lifetimes
description: Keep Qt and QML objects alive exactly as long as they are used, with one owner, connections that end and images and models that do not grow. Use when adding a QObject, a QML item, a connection, a model, a cached image or a per-frame grab, or when a window grows in memory while idle.
license: GPL-3.0-or-later
---

# Qt and QML lifetimes

Rust's checks stop at the ABI. On the other side, an object lives or dies because of its
**parent**, a connection lives as long as its **context**, and an image lives until the
**cache** lets it go. Most leaks in this project would be born there, not in Rust.

## One owner per object

- Every `QObject` has a parent, or is owned by a `std::unique_ptr`/member and nothing else.
  A `new` without parent and without owner is a leak, even if it looks like it works.
- A C++ object handed to QML is owned by its parent; never `delete` it from C++ while QML
  may still hold it. Use `deleteLater()` when it must go.
- A QML item created with `Qt.createComponent`/`createObject` has an explicit parent and is
  destroyed with it; a `Component.onDestruction` is for state, not for ownership.
- The session object outlives every view: nothing in a delegate stores it past its
  destruction, and nothing keeps a reference to a destroyed item.

## Connections end with their context

- `connect(sender, signal, context, lambda)` with a **context object**; the connection dies
  with the context. A lambda that captures `this` without a context is an use-after-free
  waiting to happen.
- Connect **once**. Connecting inside `refresh()`, `paint()` or a binding multiplies the
  calls and keeps the sender alive: that is a signal leak with a performance cost.
- Disconnect what was connected to an object that outlives the receiver, or give it a
  context that does not.
- A `QTimer` is owned by its receiver; a timer started in a temporary is a timer that fires
  into freed memory.

## QML bindings and models

- A binding runs many times: create nothing inside it. No `new`, no object construction, no
  `Qt.createQmlObject` per evaluation.
- A `Repeater` over a list that grows without a bound builds items without a bound: the list
  needs its limit (see [untrusted-input-bounds](../untrusted-input-bounds/SKILL.md)).
- A `ListView` carries `cacheBuffer` and delegates that are cheap; a delegate that loads an
  image at full size keeps every visited row in memory.
- `Image` sets `sourceSize` for a large file: a 4000 px photo drawn at 24 px is 24 px of
  screen and megabytes of memory.
- A model updated item by item emits a change per item; use the batch signal when the whole
  list changes.
- View state that belongs to the window (widths, folds) is not document state and is not a
  binding loop: a property written from a handler that writes it again is a loop that never
  settles.

## Images, grabs and caches

- `grabWindow`/`grabToImage` allocate a full frame: a capture or a test, never the paint
  loop. `--report-paints` proves an idle window does not repaint; it also proves nothing
  grabs per frame.
- Every cache (icons, thumbnails, pixmaps) has a **top and an eviction rule**, per
  `AGENTS.md`; the icon cache is keyed by name, size and colour scheme, never by a growing
  path list.
- A `QPixmap` held in a static map keeps its device resources after the window closes;
  clear on theme change or bound the map.
- A texture or image loaded for a document is released when the document closes.

## Tests that catch it

- **Idle**: the window at rest does not repaint and does not allocate per frame
  (`--report-paints`, a counting allocator in a Rust test).
- **Repeat**: open and close a document, or switch circuits, N times; memory and object
  count do not grow. `QObject::dumpObjectTree` or a counter of live children makes it
  visible.
- **Destroy order**: closing the window while a job is running, a dialog open or a timer
  armed does not crash; the test drives that order.
- **ASan/LSan** on a debug build for the window when a leak is suspected; a manual
  `heaptrack` run for the real number, not for the guess.

## Traps

- A parentless `QObject` stored in a `QHash` and never removed.
- A connection made in a loop over sessions or circuits.
- A lambda capturing `this` connected to a long-lived sender without a context.
- A `Repeater` over `session.recent` or a catalog with no bound.
- An `Image` without `sourceSize` for a file the user chose.
- Grabbing the window for a thumbnail on every `documentChanged`.
- A cache keyed by file path that never evicts.
- A test that only opens and closes once.

## Validation

```bash
QT_QPA_PLATFORM=offscreen ./apps/punchi-fluid-qt/build/punchi-fluid --report-paints
./scripts-dev/check.sh
```

Then the manual pass: open and close five times, watch the resident memory; close during a
simulation with a dialog open; switch theme with a document open. The pull request says
which case was run and what the number was.
