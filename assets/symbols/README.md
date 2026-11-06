# Symbol reference and review material

Symbols are **not** stored here. Every symbol is a definition file in
[`assets/catalog/`](../catalog/README.md), so that geometry, behaviour and ports
travel together and a user can override any of them.

This folder holds what a conformance review needs:

| Content | Why |
|---|---|
| `review/<id>.md` | The signed-off review of one symbol: which function it draws, the Part 1 reference it was checked against, the reviewer, the date |
| `review/<id>-light.png`, `-dark.png` | The symbol at palette size and at 300 dpi, in both themes |
| `notes/` | Our own engineering notes on ambiguous cases |

Rules:

- **No third-party asset.** No screenshot, scan or extract from another program, and
  no page of the standard. Shapes are drawn by us to conform, and the review records
  which requirement was checked.
- The requirements and the checklist live in
  [docs/symbology.md](../../docs/symbology.md).
- A symbol without a review file is *unverified* and is labelled as such in the
  exported document until the review exists.
