---
name: capability-plan
description: Turn a teaching need into a specified capability before writing code: purpose, behaviour, evidence, then an improvement with a reason and a test. Use when planning a feature, starting a phase, or deciding what the program must do next.
license: GPL-3.0-or-later
---

# Capability plan: purpose first, evidence second, improvement last

This project exists so that a student with no licence, no budget and no connection can
draw a pneumatic or hydraulic circuit, run it and learn from what happens. Every
capability is judged against that sentence, not against what some other program does.

The authority for **what a capability is** is the standard; the authority for **why it
exists at all** is the teaching goal. No third-party product is named in this repository,
and no material from one is stored here.

## The gate

No feature work starts until the capability is written down in
[docs/capabilities.md](../../../docs/capabilities.md), in our own words: what a student
must be able to do, what they see, what changes in the document or the model, and the
evidence that will prove it. A step without an entry is not ready to start, and an entry
without a purpose is a wish.

## What settles what

| Question | Source |
|---|---|
| What does this symbol look like? | ISO 1219-1, in the diagram conventions of ISO 1219-2 and ISO 1219-3 |
| How much gas flows through this restriction? | ISO 6358, including the choked branch |
| What is this component called? | ISO 5598 vocabulary |
| Does this belong in the program at all? | the teaching goal: a student must be able to do it, understand it or check it |
| How good is our answer? | the test named in the entry, at the level the claim needs |

If a capability cannot be settled from those sources, it is not specified yet. Write the
open question in the entry instead of guessing.

## Two arcs: cover, then improve

1. **Cover.** The capability does what a teaching simulator is expected to do, proven by
   the evidence in its entry.
2. **Improve.** Then it gets one recorded decision: keep it as it is, with the reason, or
   improve it, with the rough edge named, what we do instead, why it is cheap and honest,
   and the test that holds it.

A difference you cannot explain is a regression wearing a feature's name. Improving is not
redecorating: if the change cannot be stated as "this is what a student could not do
before", it is taste, and taste does not justify a divergence.

The project's own improvements are: a keyboard path for every action, a check that points
at the fault instead of describing it, undo that never loses a drag, and a theme that
follows the desktop.

## The coverage ledger

[docs/capabilities.md](../../../docs/capabilities.md) holds one row per function a
teaching simulator is expected to have. Its state is one of five words, never blank:

| State | Means |
|---|---|
| `missing` | expected and absent; a gap with no work started |
| `partial` | started; the entry says exactly which part is missing |
| `covered` | the behaviour is there, proven by the evidence in its entry |
| `improved` | covered, plus a named improvement with its test |
| `not planned` | expected and deliberately out of scope; the reason is about **us** — licence, scope, content we do not own — never contempt for a user's need |

`not planned` is the only state that needs an argument. A function the category expects is
the default; dropping it is the exception.

## Where a capability lands

| Layer | Crate or app | The test that proves it |
|---|---|---|
| A new or changed component | `assets/catalog/*.toml`, validated by `crates/punchi-fluid-catalog` | schema and rule test in the catalog crate |
| The document and undo | `crates/punchi-fluid-doc` | round-trip and undo test |
| Connections and the design check | `crates/punchi-fluid-doc`, `crates/punchi-fluid-check` | one test per fault, asserting coordinates |
| Physics | `crates/punchi-fluid-solver` | invariant or reference curve, tolerance stated |
| Drawing | `crates/punchi-fluid-draw` behind `Surface` | geometry test, then the same drawing on both devices |
| Export | `crates/punchi-fluid-export` | PDF and raster acceptance test |
| Interface | `crates/punchi-fluid-ffi` for the ABI, `apps/punchi-fluid-qt` for the window | headless ABI test plus a capture |

A capability that seems to need a new `behaviour` variant, a `match` on a component id or
hardcoded geometry is the signal to stop: a component is a file
([ADR 0002](../../../docs/adr/0002-components-are-files.md)), and a new variant needs an ADR.

## Procedure

0. **Read the order first.** [docs/plan.md](../../../docs/plan.md) for the milestone,
   [docs/capabilities.md](../../../docs/capabilities.md) for the phase,
   [docs/next-steps.md](../../../docs/next-steps.md) for the step. If this skill and a
   document disagree, the document wins and this skill is the bug.
1. **Name the teaching need.** "A student must be able to re-route a pipe without deleting
   it" is a need; "improve the editor" is not.
2. **Pick one capability, not a phase.** "Select a pipe and re-route it by dragging" is a
   capability; "the editor" is a phase. If its entry needs more than a screen of text, cut
   it smaller.
3. **Specify the behaviour** in a sentence a user would say: what they do, what they see,
   what changes in the document or the model. Cite the standard that settles the details.
4. **Write the entry** in `docs/capabilities.md`, in English, in our own words: purpose,
   behaviour, what we have, the gap, and the test that proves the gap closed. This is the
   gate: no entry, no code.
5. **Map the gap onto this workspace**, crate by crate, with the table above. A capability
   that cannot say where it lives is not specified.
6. **Build in layer order**: catalog definition, model, drawing, interface, document
   format. A capability that skips a layer is either not built or faked.
7. **Prove it** at the level of evidence the claim needs
   ([rust-testing-and-verification](../rust-testing-and-verification/SKILL.md)); a visible
   capability also gets a light/dark capture. This is the moment its row becomes `covered`.
8. **Improve it, or say why not.** Write the decision in the entry, then the row becomes
   `improved`. A capability left at `covered` without a decision is unfinished.
9. **Close the phase** by updating the ledger with its evidence and adding the Spanish
   changelog subsection saying what was covered and what was improved.

## The entry shape

```markdown
### F1 — re-route a pipe by dragging its segment

- **Purpose**: a student corrects a route without deleting the pipe and losing the
  connection, which is the mistake a first schematic always needs to fix.
- **Behaviour**: pressing a pipe selects it; dragging the selected pipe moves its bends and
  keeps both ends on their ports.
- **We have**: selection and deletion; no drag.
- **Gap**: dragging a selected pipe, with the hit test against the drawn polyline and not
  the chord between ports.
- **Evidence**: an ABI test that drags a bend, asserts both endpoints and undoes it.
- **Then better**: snapping a bend to 45° and a keyboard nudge in whole millimetres,
  because free angles read as sloppy on a sheet and cannot be set without a mouse; the
  nudge test asserts both.
```

The first five fields are the gate to start. The "Then better" line is the gate to call it
finished: either the improvement, or "kept as it is, because …".

## Study material

A capability is specified from the standard, the teaching goal and the program's own
model. Nothing else is required, and the plan must never depend on a program the community
does not have.

- A local study copy may exist on the owner's machine for their own observation. It is
  addressed only through `$PUNCHI_REFERENCE_DIR`, which has **no default**: the repository
  does not know its name, its path or its contents.
- The repository never stores that path, the structure of that copy, its file names, its
  inventory or its counts. `git ls-files` and `git status` must stay empty of anything
  that came from it.
- Nothing is parsed, converted, diffed or copied from it, and no proprietary file format is
  read by this project.
- What an observation changes is **our decision**, recorded in our own words. A note that
  names a third-party file is not a specification.

## Gotchas

- **The plan cannot depend on private material.** If a capability can only be specified by
  opening something the community cannot open, it is not specified yet.
- **Counts drift.** Never quote an inventory number; a number is not a requirement.
- **"It does X" is a claim, not a requirement**, until it is written as a sentence a user
  would say and paired with the test that fails today.
- **Comparison is behaviour, not pixel-copying.** The ribbon, the palette and the theme
  follow [docs/ui-ux.md](../../../docs/ui-ux.md) and the desktop's colours; looking
  different is a decision, not a defect.
- **Do not let a study become a rewrite.** If a phase needs forty entries, the phase is
  wrong: cut it in the capability document before writing any code.
- **Improving is not redecorating.** If the difference cannot be stated as what a student
  could not do before, it is taste, and taste is not worth a divergence.
- **A function the category expects is the default; dropping it is the exception.**
- **Symbols are never traced** from another program, a screenshot or a manual: draw them by
  hand against ISO 1219 ([fluid-power-symbology](../fluid-power-symbology/SKILL.md)).
- **No names.** No third-party product, manual, vendor or path appears in a tracked file;
  `scripts-dev/check-docs.sh` fails if one comes back.

## Validation

- `git grep` finds no third-party product name and no path under `$HOME` in tracked files.
- The capability document holds an entry for the capability, with the fields filled, before
  the first commit that implements it; the entry names a test that fails without the change
  and passes with it.
- The ledger has a row for the capability, and its state matches the entry: `covered` only
  with the evidence, `improved` only with the improvement and its test, `not planned` only
  with a reason about us.
- `./scripts-dev/check.sh` is green, and a visible capability has its light/dark capture.
- The Spanish changelog subsection says what was covered and what was improved.
- No capability is called finished at `covered` without its "then better" decision.
