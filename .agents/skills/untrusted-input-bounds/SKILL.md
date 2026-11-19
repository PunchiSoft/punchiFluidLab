---
name: untrusted-input-bounds
description: Bound every untrusted input and every collection or cache that can grow, in the crate that owns the data, with a message and a one-past-the-limit test. Use when writing a parser, a list, a queue, a cache, an undo history or any loop over data from a file or the UI.
license: GPL-3.0-or-later
---

# Untrusted input and resource bounds

A parser without a bound is a crash waiting for the wrong file, and a list without a
bound is a memory leak waiting for a patient user. Security here is a **number, a place
and a test**, not a review habit.

**Rule:** everything that comes from outside — a circuit file, a catalog file, a path, a
name, a recent list, a paste, an imported document — and everything that grows — a queue,
a cache, a history, a log, a set of thumbnails — has a limit in the crate that owns the
data, a message the reader can act on, and a test that pushes past it.

## Names to use in a review

| Term | Meaning |
|---|---|
| Resource limit / bound | the maximum the program accepts |
| Untrusted input | bytes the program did not produce |
| Resource exhaustion | OWASP API4:2023 *Unrestricted Resource Consumption* |
| CWE-400 | Uncontrolled Resource Consumption |
| CWE-770 | Allocation of Resources Without Limits or Throttling |
| CWE-789 | Memory Allocation with Excessive Size Value |

## Checklist before writing the code

1. **File**: the maximum number of bytes accepted, checked **before** parsing, not after.
2. **String**: the maximum length, and what happens past it. Reject by default.
3. **Collection**: the maximum number of elements (circuits, recents, pipes, fields).
4. **Loop**: bounded by a value already validated, never by a value read in the loop.
5. **Allocation**: reserve with the checked size, never with the size the input declares.
6. **Cache**: a top and an eviction rule; a cache without one is a leak.
7. **History**: the maximum depth of undo, and what the oldest step costs.
8. **Time and wait**: a loop that can spin waits with a bound, not for ever.
9. **UI**: the interface reads the limit and shows it; it never owns it.
10. **Test**: the value, the value plus one, the message, and that the refusal changes
    nothing.

## Reject or truncate

- **Reject** when truncating changes the meaning: a path, an identifier, a number, a
  coordinate, a connection. A truncated path opens the wrong file.
- **Truncate** only when the meaning survives: a label for display, an elided line, a
  log message. Say so in the code, and test the truncation.
- A refusal is an error with a sentence; it is never a silent fallback to a default
  that the user cannot see.

## Where the limit lives

In the crate that owns the data (`punchi-fluid-core`, `-file`, `-catalog`, `-doc`), as a
`pub const` with a doc comment, used by both the writer and the reader. The ABI and the
window read it through a getter when they must show it ("10 recientes", "máximo 32
circuitos"). A limit that exists only in QML is not a limit: a second front end would
not have it.

## The table this repository keeps

| Limit | Value | Owner | Evidence |
|---|---|---|---|
| Recent projects | 10 | `punchi-fluid-file` | fifteen remembered, ten kept |
| Recent path | 4096 bytes | `punchi-fluid-file` | a longer path is refused |
| Recent list file | 64 KB | `punchi-fluid-file` | a larger file is refused |
| Circuits per project | 32 | `punchi-fluid-core` | refused opening and adding |
| Undo depth | the document's | `punchi-fluid-doc` | bounded steps |
| Catalog keys and numbers | per field | `punchi-fluid-catalog` | one test per rule |
| Icon and thumbnail caches | to be declared | `apps/punchi-fluid-qt` | owed |

When a limit moves, the table and the changelog move with it.

## Traps

- A bound in the UI only, or a bound in the reader but not in the writer.
- `usize` from the file used to reserve before it is compared with the maximum.
- A loop over `count` read from the input, where `count` was never checked.
- A cache keyed by path or id with no top, growing with every document opened.
- Truncating an identifier and then matching on the truncated value.
- A test that only checks the happy path: the bound has no test until one past it fails.
- A refusal that leaves the document half-edited: parse into a temporary and swap it in.

## Validation

```bash
cargo test -p <crate that owns the data>   # the one-past-the-limit test
./scripts-dev/check.sh                     # the whole gate
```

A change that adds a limit says so in the pull request, with the number, the crate and
the test name. A change that removes one needs a recorded decision.
