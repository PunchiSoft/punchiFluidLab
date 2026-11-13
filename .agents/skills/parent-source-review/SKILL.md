---
name: parent-source-review
description: Inspect the parent application's installed code and data read-only to derive behaviour, then reimplement it in Rust in our own model and format without copying bytes, geometry or files. Use when behaviour, timing, a symbol family or a workflow must be matched to the parent app, or when the running app alone does not answer the question.
license: GPL-3.0-or-later
---

# Parent source review

The owner ordered parity with the parent application. The running app answers *what it looks
like*; its installed code and data answer *how it decides*. Both are read-only references.
Neither is a source of bytes for this repository: behaviour is understood, written down in
our words, and implemented in Rust in our own document format and our own hand-drawn geometry
(ADR 0009).

This skill covers the code and data. The palette, the capture procedure and the symbol audit
are in [reference-parity](../reference-parity/SKILL.md); use both together.

## When the running app is not enough

Open the installed files only to answer a concrete question the running app cannot:

- Which ports does this valve position connect, and in which order?
- How long is this delay, how is it counted, and what resets it?
- What is the threshold or hysteresis of this switch?
- Which quantity drives an animation, and at what speed?
- Which component family and mark does a symbol belong to?
- How does a saved circuit represent a binding, a project or a unit?

A question without a test is not a reason to read code.

## Procedure

1. **Locate the installation, read-only.** The app runs through Proton; its folder lives in
   the prefix, typically `.../drive_c/Program Files/...`. Find it from the running process or
   the window's launch command. Do not modify, move or copy the folder.
2. **Inventory before reading.** List the symbol folder, the example/circuit folder and the
   data files. Record names and purposes in our own words; never paste file contents into the
   repository.
3. **Answer one question at a time.** Read the smallest file that answers it. Write a short
   note with: the question, what was observed, what it means for the model, and which Rust
   layer owns it (`catalog`, `solver`, `doc`, `draw`, `ffi`, `export`).
4. **Turn the observation into a rule, not a copy.** A valve position becomes a list of open
   paths in a catalog file; a timing becomes a step counter; a symbol becomes hand-written
   geometry. The rule must be testable without the parent app.
5. **Write the evidence.** A unit/invariant/reference test, an ABI test, or a drawing test; a
   symbol review when the geometry changed. The test never reads the parent's files.
6. **Record the decision.** If the rule changes the model, write the ADR. If it is only a
   fact (a threshold, a port order), put it in the catalog file and its guide.
7. **Gate and commit.** `./scripts-dev/check.sh` green, one signed commit, queue and
   continuity note updated.

## What must never happen

- Copying a file, a byte sequence, a symbol, a screenshot, a manual page or an example
  circuit from the parent into this repository.
- Reusing its file format, its identifiers or its visible names. The document format is ours
  and versioned; the catalog is ours and written by hand.
- Treating a decompiled or reconstructed fragment as a specification. If the rule cannot be
  stated in one sentence and proven by a test, it is not understood yet.
- Reading code to answer a question that the running app or the standards already answer;
  that wastes the owner's time and invites accidental copying.
- Stopping because the code is proprietary. If a question needs a file that is absent, write
  the blocker and the exact missing input; do not invent the behaviour.

## Mapping the observation to Rust

| Observation | Rust home | Evidence |
|---|---|---|
| Ports, roles, open paths, thresholds | `assets/catalog/*.toml` | catalog lint and validation test |
| Flow, pressure, timing, counting | `punchi-fluid-solver`, `punchi-fluid-ffi` | invariant/reference/step test |
| Positions, bindings, projects, units | `punchi-fluid-doc`, `punchi-fluid-file` | document and migration test |
| Symbol family and geometry | `punchi-fluid-draw`, `assets/symbols/review` | symbol lint, light/dark review |
| What the window shows and when | `punchi-fluid-ffi`, `punchi-fluid-qt` | ABI/selftest/QML check |

## Stopping

The owner's order is to continue until the behaviour and the symbology are correct. This
skill may not be used as a reason to pause. Stop only with a gated commit, or with a written
blocker that names the missing file, the question it would answer and the test that would
prove the answer. A summary of intentions is not progress.

## Gotchas

- Proton prefixes hide the installation under `drive_c`; the running process command line is
  the shortest path to it.
- Example circuits are copyrighted content: read one to understand a behaviour, never copy it
  or ship it.
- Installed data may be compiled into an archive; if it cannot be read as text, say so and
  use the running app plus the standard instead of guessing.
- The parent's labels often couple parts by name; our model couples them by wire, a source
  binding or a port. Translate the *effect*, not the mechanism.
- A parent symbol file is not a catalog schema; write our own fields and validate them.

## Validation

```bash
# The question, the observation and the Rust layer are written in the issue/ADRs/tests.
cargo test -p punchi-fluid-solver
cargo test -p punchi-fluid-ffi
cargo test -p punchi-fluid-catalog
./scripts-dev/check.sh
git --no-pager log --oneline -1
```
