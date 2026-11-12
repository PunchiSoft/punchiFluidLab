# Architecture decision records

One file per decision, numbered in creation order. The contract, the format and the rules
for editing an accepted record are in [template.md](template.md); this page is the index
and the status at a glance.

An ADR records **why** the project is shaped the way it is. It is not a tutorial and it is
not a second copy of the code: when the code and an ADR disagree, the ADR is either
outdated or the code has a bug, and one of them is fixed in the same change.

## Index

| ADR | Decision | Status | Date |
|---|---|---|---|
| [0001](0001-language-and-layout.md) | Rust and a crate-per-layer workspace | superseded by [0005](0005-qt-quick-over-a-c-abi.md); naming amended by [0009](0009-identity-and-standards.md) | 2026-11-06 |
| [0002](0002-components-are-files.md) | Components are definition files, not Rust types | accepted | 2026-11-06 |
| [0003](0003-one-drawing-two-devices.md) | One drawing behind a `Surface` trait | accepted | 2026-11-06 |
| [0004](0004-icons-from-the-desktop.md) | Icons come from the desktop, never from the repository | superseded by [0005](0005-qt-quick-over-a-c-abi.md) | 2026-11-06 |
| [0005](0005-qt-quick-over-a-c-abi.md) | The sheet is Qt Quick over a C ABI, the model stays Rust | accepted; supersedes 0001 and 0004 | 2026-11-06 |
| [0006](0006-a-junction-is-a-component.md) | A T junction is a component, not a document node | accepted | 2026-11-06 |
| [0007](0007-the-sheet-format-in-the-abi.md) | The saved sheet is a file format of `punchi-fluid-file`, written by the ABI | accepted | 2026-11-06 |
| [0008](0008-a-gauge-is-a-behaviour.md) | A pressure gauge is a behaviour, not a draw-time decoration | accepted | 2026-11-06 |
| [0009](0009-identity-and-standards.md) | The project stands on its mission and the standards | accepted; amends the naming of 0001 and 0005 | 2026-11-07 |
| [0010](0010-the-check-reads-geometry-from-the-drawing.md) | The design check reads port geometry from the drawing crate | accepted | 2026-11-07 |
| [0011](0011-the-check-is-a-public-command.md) | The circuit check is a public command with a versioned contract | accepted | 2026-11-07 |
| [0012](0012-the-paper-a-drawing-is-printed-on.md) | The paper a drawing is printed on | accepted | 2026-11-07 |
| [0013](0013-a-project-is-one-document.md) | A project is one document with several circuits | accepted | 2026-11-12 |
| [0014](0014-a-flow-meter-is-a-restriction-that-reports.md) | A flow meter is an inline restriction that reports its flow | accepted | 2026-11-12 |
| [0015](0015-a-recorder-samples-a-named-quantity.md) | A recorder is a component that samples a named quantity | accepted | 2026-11-12 |
| [0016](0016-a-termination-is-a-component.md) | A termination is a component: a plug closes, a silencer vents | accepted | 2026-11-12 |

## When a decision needs an ADR

A change needs an ADR when it does any of these:

- adds a crate, a dependency edge between our crates, or a new public layer;
- adds a `behaviour` variant to the catalog schema, the one exception to "a component is a
  file";
- changes the file format in a way a previous version cannot read;
- changes a rule in `AGENTS.md`, the standards contract or the memory budget;
- chooses a library or a format the project will have to live with.

It does **not** need one for a new catalog file, a new test, a bug fix that restores the
documented behaviour, or a refactor inside one crate with no public change.

## Superseding a decision

A new ADR names the old one in its `Status` and in its `Supersedes` line. The old file is
not rewritten; it gains one line in its status that points to the new record. The index
above is the only place where the chain is read in order.
