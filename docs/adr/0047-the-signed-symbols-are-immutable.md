# 0047 — The signed symbols are immutable

- **Status:** accepted
- **Date:** 2026-11-24
- **Deciders:** project owner, implementation session
- **Context crates:** none (the symbol catalogue and the gate)

## Context

A symbol is reviewed once and then trusted: the TOML definition, the review record and the
three captures in `assets/symbols/review/` are the evidence a teacher points at when a
student asks whether a symbol follows ISO 1219. Today `status: complete` is the only mark of
that trust, and it is a line of text. Nothing detects a definition edited after the review,
a review rewritten after the sign-off or a capture swapped for a nicer one; nothing
distinguishes "reviewed" from "signed".

The owner has decided that the reviewed symbols are the base, that a review is accepted one
symbol at a time, and that once accepted a symbol is not edited again except through a new
review that the owner authorises. That decision needs a mechanism, not a convention: the
repository must be able to *prove* that the files it ships are the files the owner accepted.

The constraint that shapes the mechanism: the repository is public and the agent works in
it, so the thing that proves acceptance must be held by the owner alone.

## Decision

A symbol is **signed** when all of the following hold:

1. `assets/symbols/signed.toml` has one entry for it with the SHA-256 of its definition,
   its review and its three captures.
2. The entry carries `signed_by` and `signed_at` naming the owner's principal and date.
3. `assets/symbols/signed.toml.sig` is an SSH detached signature over the manifest, made by
   the private key of a public key listed in `assets/symbols/owner_signers`.
4. `scripts-dev/check-signed-symbols.sh` verifies the signature and the hashes; it runs from
   `check-docs.sh`, so it runs on every pull request and on `main`.

`status: complete` stays as the condition for an entry, not as the proof. The proof is the
signature over the hashes. A **signed symbol is immutable**: changing its definition, review
or captures makes the hashes disagree with the manifest and the gate fails; a change needs a
new review, a new revision and a new signature from the owner. The previous revision stays in
Git, and the manifest keeps only the current one.

The namespace is `punchi-fluid-signed-symbols`; the signature mechanism is OpenSSH's
(`ssh-keygen -Y sign`/`-Y verify`), so no new dependency enters the build.

While the base is being signed, `base_open = true` allows entries whose hashes are pinned but
which carry no signature yet. The last accepted batch sets it to `false`; from then on the
gate also fails when any entry is pending. The owner signs each batch, never the agent: the
private key is the owner's, and it never enters the repository or a session.

## Consequences

- The gate now fails on the only change it was blind to: an edit to a reviewed symbol.
- A new symbol, or a correction to a signed one, costs an owner signature; the review record
  and the batches have to be prepared before asking for it.
- The renderer is not signed. The captures cover the rendered result, so a renderer change
  requires regenerating the captures and re-signing, and a reviewer who forgets is the one
  hole this decision cannot close by itself.
- Rotating the owner's key means adding the new public key, re-signing the manifest, and only
  then removing the old line; removing it first invalidates the existing signature.
- `commit` is recorded for provenance but cannot be verified: the repository squash-merges,
  so the commit that was HEAD when the owner signed does not survive on `main`. The hashes
  are the evidence.
- The check depends on `ssh-keygen` being present. A missing binary is a failure, not a
  warning: a gate that silently skips the signature is the bug this decision exists to
  prevent.

## Alternatives considered

- **GPG signatures.** Rejected: a second trust chain the owner and CI would both have to
  keep, for the same guarantee OpenSSH already provides.
- **A Git tag per symbol.** Rejected: a tag names a commit, not the five files, and an edit
  inside a reviewed symbol does not change the tag.
- **Signing in CI with a secret.** Rejected: it proves that the CI holds a key, not that the
  owner accepted the symbol, and it puts a private key in the repository's secrets.
- **Trusting `status: complete` plus review.** Rejected: it is the state this ADR replaces;
  nothing detects the edit.
