---
name: security-review
description: Review one change against the threat model and the attack surface it touches, with a per-surface record and hostile-input tests. Use when a change reads or writes a file, resolves a path, exports, handles the clipboard, autosave or recovery, opens an external link, or touches dependencies, workflows or release artefacts.
license: GPL-3.0-or-later
---

# Security review per change

The repository's threat model is `docs/engineering.md` §Security and the reporting policy
is `SECURITY.md`; neither is a procedure. This skill is the procedure: name the surface a
change touches, apply the control that already exists, prove it with a hostile case, and
record it. A change that widens a surface without a record is a review that did not
happen.

**Rule:** no network at runtime, no shell-out, no telemetry, no `unsafe`. The only
outbound action is a link the user clicks; anything else needs an ADR.

## 1. Name the surface

| Surface | Data from outside | Owner |
|---|---|---|
| Circuit document (`.pfluid`) | bytes, ids, numbers, paths | `punchi-fluid-file` |
| Catalog file | bytes, keys, numbers, ids | `punchi-fluid-catalog` |
| Recent list | paths, names | `punchi-fluid-file` |
| Autosave and recovery | a copy of the user's document | `punchi-fluid-doc`, ABI |
| Export (PDF, PNG, SVG) | paper, dpi, target path, title | `punchi-fluid-export` |
| Clipboard and paste | placements from another session | `punchi-fluid-doc` |
| Dialogs and paths | a chosen path, a name | the window |
| External links | the repository and issue URLs, clicked | the window |
| Dependencies, workflows, release | crates, actions, artefacts | repository |

## 2. Apply the control that exists

- **Parse defensively**: unknown keys rejected, numbers range-checked, no recursion, and
  the bounds from [untrusted-input-bounds](../untrusted-input-bounds/SKILL.md).
- **Write atomically**: temporary file in the same directory, then rename; never write
  outside the chosen path; refuse to overwrite without an explicit answer.
- **Keep metadata clean**: no path, user name or host in a PDF; titles sanitised of
  control characters and bounded.
- **Resolve paths under the allowed roots** (`XDG_DATA_DIRS` for icons), no symlink walk
  outside them, no path taken from a document.
- **Never build a command line**: no shell, no `QProcess`, no `eval`.
- **Links are explicit**: the browser opens only the two fixed URLs, on a click.
- **Dependencies**: `Cargo.lock` committed, `cargo-deny` and `cargo-audit` green, no build
  script from an unreviewed crate.
- **Workflows**: least privilege, pinned actions, no secret in a pull-request job
  ([github-actions-security](../github-actions-security/SKILL.md)).

## 3. Prove it with a hostile case

| Surface | The test that counts |
|---|---|
| Document | truncated file, absurd number, unknown key, path field pointing outside |
| Catalog | a rule broken on purpose, a file that must be skipped and reported |
| Recent list | too many entries, an over-long path, a file that no longer exists |
| Autosave | a copy from a crash is offered, never loaded on its own |
| Export | dpi outside 72–600, a page over the pixel budget, a target that exists |
| Paths | `..`, absolute path where a name is expected, a symlink |
| Links | no request at start; only the fixed URL on a click |

A case that only fails in a comment is not a control. Write the test at the level where
the decision is made, and assert the **message** and that nothing was written or changed.

## 4. Record it

For a change that touches a surface, the pull request says, in a short table:

| Surface | What enters | Control applied | Test |
|---|---|---|---|
| | | | |

If the change adds a surface, changes one row of the threat table, or relaxes a control,
it needs an ADR and a line in `docs/engineering.md`; the record is not optional.

## Recommendations while programming

- **Parse into a temporary and swap it in.** A refusal must leave the document, the file
  or the settings exactly as they were.
- **Validate before allocating.** A size read from a file is a claim; compare it before
  reserving or looping.
- **Never trust a path from a document.** A file may name a component, never a location.
- **Check the destination, not only the source.** Before writing, refuse a directory that
  is not writable, a name that is empty, a target that is the source.
- **Errors are sentences for the user and details for the log.** No stack trace, no path
  from the machine, in a message the user reads.
- **No secret in the repository, the settings or a test.** There are none to have today;
  keep it that way.
- **A new dependency is a security decision**, not a convenience
  ([rust-dependencies-and-supply-chain](../rust-dependencies-and-supply-chain/SKILL.md)).
- **A generated artefact carries its provenance**: licence, origin and checksum.
- **Fail closed.** If a control cannot run (no icon theme, no writable directory), say so
  and refuse the action; never fall back to something weaker silently.
- **Test the refusal, not only the success**: the file is untouched, the message names the
  reason, and the program keeps running.

## Traps

- A check in the UI that a second front end would not have.
- A temporary file left behind on failure; a rename across filesystems.
- Following a symlink out of the icon roots, or a relative path from a document.
- A title written into a PDF without sanitising control characters.
- A refused write that has already truncated the target.
- A test that asserts "does not crash" instead of "refuses and explains".
- A workflow with `pull_request_target` and a secret in scope.

## Validation

```bash
cargo test -p <crate that owns the surface>
./scripts-dev/check.sh
cargo deny check            # when a dependency moved
```

Then the manual pass: the hostile case above, run by hand when the test cannot reach it
(a dialog, a symlink, a missing file). The pull request carries the surface table and the
test name; without them the change is not reviewed.
