# Symbol reference and review material

Symbols are **not** stored here. Every symbol is a definition file in
[`assets/catalog/`](../catalog/README.md), so that geometry, behaviour and ports
travel together and a user can override any of them.

This folder holds what a conformance review needs:

| Content | Why |
|---|---|
| `review/<id>.md` | The reviewed record of one symbol: which function it draws, the Part 1 reference it was checked against, the reviewer, the date; `status: complete` means reviewed, not signed |
| `review/<id>-light.png`, `-dark.png` | The symbol at palette size and at 300 dpi, in both themes |
| `conformance.md` | The work list: one row per core symbol, the figure it must match, its ports and the state of its review and its signature |
| `signed.toml`, `signed.toml.sig`, `owner_signers` | The signed symbols, the owner's SSH signature over the manifest and the public key that verifies it ([ADR 0047](../../docs/adr/0047-the-signed-symbols-are-immutable.md)) |
| `notes/` | Our own engineering notes on ambiguous cases |

Rules:

- **No third-party asset.** No screenshot, scan or extract from another program, and
  no page of the standard. Shapes are drawn by us to conform, and the review records
  which requirement was checked.
- The requirements and the checklist live in
  [docs/symbology.md](../../docs/symbology.md).
- A symbol without a review file is *unverified* and is labelled as such in the
  exported document until the review exists.
- **Reviewed is not signed.** `status: complete` says the second reader accepted the record.
  A symbol is **signed** only when its entry in `signed.toml` matches the hashes of its TOML,
  review and three captures and the owner's SSH signature over the manifest verifies against
  `owner_signers`; `scripts-dev/check-signed-symbols.sh` is the gate. A signed symbol is
  immutable: editing any of its files fails the gate and needs a new review and signature.
  The agent never signs and never edits the manifest or the signature on its own.
