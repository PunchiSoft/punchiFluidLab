---
name: reference-parity
description: Study FluidSIM as the base product, record each capability before building it, replicate its observable behaviour, then improve it with a reason and a test. Use when starting a phase of the parity plan, planning a feature, or deciding what the product must do next.
license: GPL-3.0-or-later
---

# Reference parity: study first, replicate, then improve

The reference is *FluidSIM 4 Neumática*, and it is **the base product**. This project
exists to give the same workflow to people who cannot or will not buy it, so the base is
where a capability is learned and, at first, reproduced: a replacement that does less than
the base is not a replacement. Improving on the base is the point, but it comes **after**
the base is there, one recorded decision at a time.

The base is a source of **behaviour**, never of **content**: no symbol, circuit, sound,
bitmap, page or byte of it enters this repository.

**The gate: no feature work starts until the observable behaviour it replicates is written
down** in [docs/reference-parity.md](../../../docs/reference-parity.md), in our own words,
with the evidence that will prove it. A phase step without a study entry is not ready to
start, and a study entry without an observation is a wish.

## Two arcs: the base, then better than the base

| Arc | What it means | When it is allowed |
|---|---|---|
| **Replicate** | the same observable behaviour as the base, proven | first, always |
| **Improve** | the same capability, done better, with a reason and a test | only after the replicate arc is proven |

Replicating first is not deference, it is the only way to judge an improvement: a difference
you cannot explain against the base is a regression wearing a feature's name. A capability is
not done at parity either — it is done when the ledger says `base` or `better`, and every
capability we ship should end as `better` unless the honest answer is that the base already
does it well.

An improvement may not skip a function, may not break ISO 1219 conformance
([fluid-power-symbology](../fluid-power-symbology/SKILL.md)), may not make a claim the code
does not back, and may not be invisible: the ledger line says what differs and why.

Good improvements in this project are already the four rules at the end of
[docs/reference-parity.md](../../../docs/reference-parity.md): a keyboard path for every
action, a check that points at the fault instead of describing it, undo that never loses a
drag, and a theme that follows the desktop. Bad improvements are the ones that only look
different: a redecorated panel, a renamed button, a "modern" workflow nobody asked for.

## Coverage: the whole base is the target

[docs/reference-parity.md](../../../docs/reference-parity.md) keeps a **coverage ledger**:
one row per capability the base has, with one of five states — `none`, `partial`, `base`,
`better`, `not adopted`. Nothing stays blank, and `not adopted` carries the reason, because
the base's function is the default and dropping it is the exception that needs an argument.
The ledger is the answer to "what is missing to be a replacement", and it is read before any
new phase starts.

## Where the reference is

| Item | Path | What it gives |
|---|---|---|
| Reference root | `$PUNCHI_REFERENCE_DIR`, default `$HOME/Descargas/FLUID SIM NEUMATICO` | the installed program |
| Manual | `doc/fl_sim_p.pdf` — *FluidSIM 4 Neumática* | what a function is for, read only |
| Component library | `sym/{pneu,electr,Digital,misc,dde,grafcet}/*.ct` — 210 definitions | the inventory of functions to cover |
| Example circuits | `dida/*.ct` — 533 circuits | behaviour to reproduce as our own circuits |
| Circuit documents | `ct/*.ct` | the program's own files |

The directory is **read-only**: open, observe, never save into it, never copy out of it.
It is outside the repository, and no tracked file needs its path except this skill and
[docs/reference-parity.md](../../../docs/reference-parity.md); `git status` must stay clean
of anything that came from it.

## What may be read, and how

Set the path once in the shell you read with:

```bash
R=${PUNCHI_REFERENCE_DIR:-$HOME/Descargas/FLUID SIM NEUMATICO}
```

- **The manual**: read it for the meaning of a function and the order of a workflow.
  `pdfinfo "$R/doc/fl_sim_p.pdf"` and `pdftotext -f <page> -l <page> -layout … -` print
  pages to the terminal; never write extracted text to a file, an issue or a commit.
- **The library**: list it, count it, name it.
  ```bash
  for d in "$R"/sym/*/; do printf '%s %s\n' "$(basename "$d")" "$(find "$d" -iname '*.ct' | wc -l)"; done
  find "$R/dida" -iname '*.ct' | wc -l
  ```
  A file name says a function exists; the bytes are a proprietary format and are not a
  source. Never parse, convert or diff a `.ct`.
- **The program**: run it (a Windows build; the VM or Wine belongs to the owner's machine,
  not to this repository), or watch the owner demonstrate it, and describe what happens in
  a sentence a user would say. If it cannot be run, say so in the entry and cite the manual
  page and the example circuit name instead.

## Procedure

0. **Read the order first.** [docs/plan.md](../../../docs/plan.md) for the milestone,
   [docs/reference-parity.md](../../../docs/reference-parity.md) for the phase,
   [docs/next-steps.md](../../../docs/next-steps.md) for the step. If this skill and a
   document disagree, the document wins and this skill is the bug.
1. **Pick one capability, not a phase.** "Select a pipe and re-route it by dragging" is a
   capability; "the editor" is a phase. If its entry needs more than a screen of text, cut
   it smaller.
2. **Observe it in the reference.** One observable behaviour: what the user does, what they
   see, what the program does to the document or the model. Note where it was seen — screen,
   manual page, example circuit by name.
3. **Write the study entry** in `docs/reference-parity.md`, in English, in our own words:
   what the reference does, how it was observed, what we have, the gap, and the test that
   proves the gap closed. This is the gate: no entry, no code.
4. **Map the gap onto this workspace**, crate by crate with the table below. The map is part
   of the entry, because a capability that cannot say where it lives is not specified.
5. **Replicate in layer order**: catalog definition, model, drawing, interface, document
   format. A capability that skips a layer is either not replicated or faked.
6. **Prove it** at the level of evidence the claim needs
   ([rust-testing-and-verification](../rust-testing-and-verification/SKILL.md)); a visible
   capability also gets a light/dark capture. This is the moment its ledger row becomes
   `base`.
7. **Improve it, or say why not.** Write the decision in the entry: either "kept as the
   base, because …" or the rough edge, what we do instead, why it is cheap and honest, and
   the test that holds it. Then the ledger row becomes `better`. A capability left at
   `base` without a decision is unfinished, not finished.
8. **Close the phase** by updating the ledger rows and the parity table with their evidence,
   and by adding the Spanish changelog subsection saying what was replicated and what was
   improved.

## Where a replicated capability lands

| Layer | Crate or app | The test that proves it |
|---|---|---|
| A new or changed component | `assets/catalog/*.toml`, validated by `crates/punchi-fluid-catalog` | schema and rule test in the catalog crate |
| The document and undo | `crates/punchi-fluid-doc` | round-trip and undo test |
| Connections and the circuit check | `crates/punchi-fluid-doc`, `crates/punchi-fluid-check` | one test per fault, asserting coordinates |
| Physics | `crates/punchi-fluid-solver` | invariant or reference curve, tolerance stated |
| Drawing | `crates/punchi-fluid-draw` behind `Surface` | geometry test, then the same drawing on both devices |
| Export | `crates/punchi-fluid-export` | PDF and raster acceptance test |
| Interface | `crates/punchi-fluid-ffi` for the ABI, `apps/punchi-fluid-qt` for the window | headless ABI test plus a capture |

A capability that seems to need a new `behaviour` variant, a `match` on a component id or
hardcoded geometry is the signal to stop: a component is a file
([ADR 0002](../../../docs/adr/0002-components-are-files.md)), and a new variant needs an ADR.

## The entry shape

```markdown
### F1 — select a pipe and re-route it by dragging

- **Observed**: pressing a pipe selects it; dragging the selected pipe moves its bends and
  keeps both ends on their ports.
- **How observed**: <screen / manual chapter and page / example circuit name>, 2026-11-06.
- **We have**: selection and deletion; no drag.
- **Gap**: dragging a selected pipe, with the hit test against the drawn polyline and not
  the chord between ports.
- **Evidence**: an ABI test that drags a bend, asserts both endpoints and undoes it.
- **Then better**: snapping a bend to 45° and a keyboard nudge in whole millimetres,
  because the base's free angles read as sloppy on a sheet and cannot be reached without a
  mouse; the nudge test asserts both.
```

The five first fields are the gate to start. The "Then better" line is the gate to call it
finished: either the improvement, or "kept as the base, because …".

## Gotchas

- **The reference is a Windows program.** Its `.exe` and `.dll` files are not a source:
  never run, link, decompile or read them, and nothing taken from them counts as evidence
  of behaviour.
- **A screenshot of the reference is study material, not an asset.** Keep it in the session
  workspace or a pull-request comment; never in `assets/`, `docs/` or a commit.
- **Counts drift.** Every inventory line carries the date it was measured; re-measure before
  quoting a number, and never turn a count into a requirement by itself.
- **"The reference does X" is a claim, not a requirement**, until it is written as a sentence
  a user would say and paired with the test that fails today.
- **Parity is behaviour, not pixel-copying.** The ribbon, the palette and the theme follow
  [docs/ui-ux.md](../../../docs/ui-ux.md) and the desktop's colours; looking different from
  the reference is a decision, not a defect.
- **Do not let a study become a rewrite.** If a phase needs forty entries, the phase is
  wrong: cut it in the parity document before writing any code.
- **Improving is not redecorating.** If the difference cannot be stated as the base's rough
  edge and the test that holds ours, it is taste, and taste is not worth a divergence.
- **The base's function is the default; dropping it is the exception.** `not adopted` needs
  a reason about us — our licence, our scope, our content rules — never contempt for the
  base.
- **A study entry that names a reference file is not a specification.** The name says a
  function exists; only an observation says what it does.
- **Symbols are never traced** from the reference: draw them by hand against ISO 1219
  ([fluid-power-symbology](../fluid-power-symbology/SKILL.md)).

## Validation

- `git status` and `git ls-files` contain no file from the reference directory and no
  extracted manual text.
- The parity document holds an entry for the capability, with the five fields filled, before
  the first commit that implements it; the entry names a test that fails without the change
  and passes with it.
- The coverage ledger has a row for the capability with its state, and the state matches the
  entry: `base` only with the evidence, `better` only with the improvement and its test,
  `not adopted` only with the reason.
- `./scripts-dev/check.sh` is green, and a visible capability has its light/dark capture.
- The Spanish changelog subsection says what was replicated and what was improved.
- No capability is called finished at `base` without its "then better" decision — the
  improvement or the sentence that says the base is kept and why.
