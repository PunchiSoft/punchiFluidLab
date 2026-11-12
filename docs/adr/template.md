# NNNN — A sentence that names the decision

- **Status:** proposed
- **Date:** YYYY-MM-DD
- **Deciders:** project owner, implementation session
- **Context crates:** `punchi-fluid-…`

## Context

What is true today, what cannot be done, and why it matters to a student or to the
project. Name the constraint that forces a choice, not the solution you prefer. If a
standard settles the question, name the standard and the clause; if it does not, say so.

## Decision

The decision in one or two sentences, then the rules that follow from it. A reader must be
able to tell what is allowed and what is not without reading the implementation.

## Consequences

What becomes easier, what becomes harder, what is now forbidden, and which file, crate or
test has to change. A consequence that is a debt is named as a debt, with the phase that
pays it.

## Alternatives considered

The options that were not chosen, and the concrete reason each one lost. An alternative
that lost on taste is not recorded; a decision that only has taste behind it is not an
ADR.

## References

- The standard, the issue, the pull request or the measurement that supports the
  decision.
- The ADR this one supersedes, if any.

---

## Rules for this folder

- One decision per file, named `NNNN-kebab-case.md`, numbered in creation order.
- `Status` is `proposed`, `accepted`, `superseded by NNNN` or `rejected`; when it is
  superseded, the NNNN is a link to the new record.
- `Date` is the day the decision was accepted, in ISO 8601.
- An accepted ADR is not rewritten. A later decision that changes it is a new ADR that
  names the one it supersedes; the old file gains one line in its status.
- Every ADR is listed in [README.md](README.md) in the same change that adds it.
- No ADR names another product, copies its material or describes it as the reference
  (ADR 0009).
