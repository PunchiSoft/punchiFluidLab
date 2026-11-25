---
name: token-efficiency
description: "Cut the agent's token spend without losing precision, safety or evidence: result first, indexes before files, grouped tool calls, local edits and a gate that is never skipped. Use when writing a response, planning what to load, or deciding what evidence a claim needs."
license: GPL-3.0-or-later
---

# Token efficiency

Every token the agent spends on restating, re-reading or re-running is a token not spent on
the work. This skill is the cross-cutting budget: it makes the same change, the same proof
and the same hand-off cost less context, not less rigour.

It changes **how much** is said and loaded, never **what** is required. It cannot remove a
test, the gate, a safety check, a document, an ADR, the changelog or the pull-request flow.
Those are the change; brevity is only the packaging.

It applies to all agent work: the conversation, the context it loads and the tool calls it
makes. Sections D–F apply when the work touches code, verification or a delivery.

## A — Communication

- **Result first.** The answer, the path, the command or the decision opens the turn; the
  reasoning follows only if it changes a decision.
- **No filler.** No greeting, no "let me", no "as I mentioned", no recap of the request. Do
  not repeat what an earlier message established; reference it.
- **Do not narrate each action.** Say what changed and what proves it, not the sequence of
  clicks that got there.
- **One recommendation.** Offer alternatives only for a decision that belongs to the owner:
  scope, architecture, a security posture, a test strategy or a naming that outlives the
  change. Where one option is clearly right, state it and go.
- **Exact names.** Paths, identifiers, commands, versions and numbers are quoted verbatim; a
  paraphrase that must be re-derived costs more than it saves.
- A short task still gets its report: result, files, checks, what is pending. That compact
  form is the exception of section F, not a licence to omit evidence.

## B — Context

- **Index and sections before whole files.** Read `docs/README.md`, a catalogue, a table of
  contents or a heading range first; open the full file only when the answer is not in the
  navigable part.
- **Load the skills the theme needs, not the catalogue.** [session-start](../session-start/SKILL.md)
  already says "the catalog, not all"; the same rule governs the rest of the session.
- **Do not re-read what did not change.** A file, a test result or a `git status` read
  earlier stays valid until something edits it. When in doubt, re-read only the part the
  change touches.
- **Reload the source before the summary.** An index or a summary orients; the evidence
  comes from the file, the command or the diff itself.
- **Read once, use many times.** When a document drives several decisions, extract what it
  decides in one pass and keep the decisions, not the whole text.

## C — Tools

- **Group independent queries.** One message with several reads, greps or commands beats
  five sequential turns that each wait for the last.
- **No duplicate calls.** Before a search or a read, check whether the answer is already in
  the conversation or in the last tool output.
- **Search precisely.** `grep` with a pattern and an `include` filter, `glob` with a real
  extension, a `read` with an offset and a limit — not a directory dump to be filtered by
  eye.
- **Filter the output, not the finding.** Trim a noisy log to the failing lines, but never
  hide an error, a warning or an incomplete run behind a "looks green" summary.

## D — Code

- **Local edits.** Change the function, the field, the row: not the file around it. No mass
  rewrite, no re-formatting of untouched lines, no comment restating the old behaviour.
- **No speculative abstraction.** One caller gets a function; two get a shared one; three
  are the point to design, not a reason to invent a trait earlier.
- **The required documents are in scope.** A new catalog field, a document-shape change, a
  decision or a symbol brings its README entry, its `FORMAT_VERSION` bump, its ADR and its
  changelog line in the same change. They are the change, not extra tokens.
- **Reuse what the skill already says.** [rust-idioms](../rust-idioms/SKILL.md),
  [rust-workspace-architecture](../rust-workspace-architecture/SKILL.md) and the rest own
  the how; this skill only forbids paying twice for it.

## E — Verification

- **The gate is not optional and not shortened.** `./scripts-dev/check.sh` for code,
  `scripts-dev/check-docs.sh` for a documentation change; a green run is cited, not assumed.
- **Reuse a result only while it is still valid.** A gate, a test or a build may be quoted
  again when the files it covers and the commit did not change; after an edit, it is stale.
- **Never claim without evidence.** Name the test, the command, the exit code or the link.
  "Should work" is not a verification, and it is not cheaper: it costs a failed check later.
- **One failing check is not fixed by a weaker test.** The fix is on the branch; deleting or
  loosening a behavioural test is a red line, not an optimisation.

## F — Delivery

- **A small task** — one commit, one document, one symbol: result, changed files,
  verification and pending items, in that order. That is the whole report.
- **A phase, a batch or a pilot** uses the systems that already exist: the table and the
  board of [step-report](../step-report/SKILL.md), the phases of
  [phased-plan](../phased-plan/SKILL.md) and the loop of [auto-pilot](../auto-pilot/SKILL.md).
  This skill must not grow a second template beside them.
- **The route is [pull-request-workflow](../pull-request-workflow/SKILL.md)'s.** Branch,
  gate, pull request, hand-off; a short report is not permission to skip the route.

## Precedence

When brevity and a rule disagree, the rule wins. In order: [AGENTS.md](../../../AGENTS.md),
the project rules, the approved plan, the report table, the tests, the security review, the
symbology conformance, the required documents and the pull-request flow. This skill never
overrides them, and it never shortens authored evidence — it shortens the prose around it.

## Punchi restrictions

- No `unwrap`, `expect`, `panic!` or `unsafe` outside tests, however compact the fix.
- Typed units, versioned files and the migration test are not optimised away.
- A symbol change keeps its conformance review and its screenshots.
- The gate, the review record and the changelog travel with the change.
- Another program's source or assets never enter the repository, whatever the token saving.

## Gotchas

- **Brevity that hides a failure is a lie, not an optimisation.** Report the red check, the
  skipped test and the open question first.
- **A summary is not the source.** Do not answer from an index, a doc comment or a previous
  summary when the file can be read; cite the source.
- **Reusing a stale green is the expensive shortcut.** A gate run before the last edit
  proves the edit did not exist yet.
- **Skipping a skill is not a saving.** The skill that would have prevented the wrong
  assumption costs one read; the assumption costs the change and the reviewer's turn.
- **Grouping has a limit.** Independent reads in one turn, yes; a batch so large that its
  output is truncated or misread is more expensive than two small ones.
- **The gate can be the token sink.** Do not run it after every edit; run it once the
  change is complete, read the first failure and fix that, not all flags at once.
- **Long argument in the chat is not evidence.** The proof lives in the diff, the test and
  the check output; the chat carries the result.
- **A compact report is not a shorter welcome.** The owner reads OK / Falta / Siguiente or
  the table, not a re-narration of the session.

## Validation

```bash
./scripts-dev/check-docs.sh     # documentation-only change, includes check-skills
./scripts-dev/check.sh          # code change: fmt, clippy, tests, docs, architecture, smoke
```

The turn is efficient when the result appears in the first lines, every loaded file was
needed, no call repeats an earlier one, every claim names its evidence, and none of the
required documents, tests or checks is missing. The failure mode this skill guards against
is a shorter answer that is wrong or unproven, not a longer one that is right.
