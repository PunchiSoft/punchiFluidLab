---
name: punchi-visual-language
description: "Route a visual change in the Rust window to its canonical document, its code and its evidence: controls, tokens, icons, the ribbon, layout and language. Use when adding or changing a control, a theme token, an icon, the ribbon, a layout, or reviewing the visual language."
license: GPL-3.0-or-later
---

# Punchi visual language: route, prove, record

The visual language is specified once, in [docs/ui-ux.md](../../../docs/ui-ux.md), and
implemented once in the Rust window: the theme adapter is
[theme.rs](../../../apps/punchi-fluid/src/theme.rs), the ribbon is `ribbon/`, the catalog
panel is `sidebar.rs`, the sheet is `scene.rs` and `surface.rs`, and the layout is `layout.rs`
and `shell.rs`. This skill is the procedure between a request and that contract; a colour, a
size, a radius or a motion written here would be a second source, and a second source is a
bug.

The window is egui (ADR 0035, ADR 0036); the checks are the window test suite and the gate.

Symbols are not a visual-language change: [fluid-power-symbology](../fluid-power-symbology/SKILL.md)
owns them. The documentation diagrams are not this skill's either.

## The rule

Read the canonical home, run the test, attach the evidence. A task that cannot name its
document has not been understood; ask before inventing a rule.

## Routing by task

| Task | Read (canonical) | Run | Evidence |
|---|---|---|---|
| A new control | [Visual language](../../../docs/ui-ux.md#visual-language); [The window adapts](../../../docs/ui-ux.md#22-the-window-adapts) | `cargo test -p punchi-fluid-window`; `./scripts-dev/check.sh` | light/dark capture; the manual checklist |
| A token or theme change | [Visual language](../../../docs/ui-ux.md#visual-language); [theme.rs](../../../apps/punchi-fluid/src/theme.rs) | `cargo test -p punchi-fluid-window` (the contrast tests) | the contrast test line |
| A motion change | [Motion](../../../docs/ui-ux.md#motion); [theme.rs](../../../apps/punchi-fluid/src/theme.rs) (`motion_ms`) | `cargo test -p punchi-fluid-window`; the frame measurement | the capture at rest in both themes; the reduced-motion test |
| A new icon | [ADR 0028](../../../docs/adr/0028-the-ribbon-draws-its-own-icons.md); [icons.rs](../../../apps/punchi-fluid/src/ribbon/icons.rs); `assets/icons/actions/` | `cargo test -p punchi-fluid-window` | the rendered icon; the small-size reading |
| A ribbon change | [The ribbon](../../../docs/ui-ux.md#1-ribbon-top); `ribbon/model.rs`, `ribbon/mod.rs` | `cargo test -p punchi-fluid-window` | the width captures |
| A layout change | [The window adapts](../../../docs/ui-ux.md#22-the-window-adapts); `layout.rs`, `shell.rs` | `cargo test -p punchi-fluid-window` | the size captures |
| A language key or string | [ADR 0045](../../../docs/adr/0045-the-interface-speaks-english-through-embedded-toml-catalogs.md); `i18n/en.toml`, `i18n/es.toml` | `cargo test -p punchi-fluid-i18n`; `cargo test -p punchi-fluid-window` | both languages; the missing-key fallback test |
| A review | the task's row | `./scripts-dev/check.sh` | the checklist marked and the evidence linked |

## The checklist

**Machine-checked** — what actually fails:

- **contrast:** `theme/tests.rs` (`the_tokens_meet_the_contrast_criteria`, plus each theme's
  test) fails when a pairing misses the criterion. It cannot see a literal in a widget; that
  is the manual check.
- **window:** `cargo test -p punchi-fluid-window` covers the ribbon commands, the language,
  the icons and the interaction; CI's `window` job runs it.
- **documentation:** `./scripts-dev/check-docs.sh` resolves the links and the skill row.

**Manual, with evidence** — record the capture or the measurement:

- no widget holds a literal colour, size, radius or motion value; it uses `Tokens`;
- unit and motion come from the theme, not a literal;
- the motion follows [Motion](../../../docs/ui-ux.md#motion); a document command changes in
  the frame it was issued;
- every state exists (normal, hover, pressed, disabled, checked) and none is colour-only;
- the control is keyboard reachable and exposes an accessible name;
- the icon reads at the small size and stands for one concept;
- the chrome uses the spacing unit and the drawing uses millimetres;
- every user-facing string is a key in `i18n/en.toml` with its `es.toml` translation; English
  is the source and default (ADR 0045);
- nothing is copied from another product.

The motion detail is in [a motion change](references/motion.md).

## Gotchas

- **A value copied into this skill is a bug.** It lives in `docs/ui-ux.md` or `theme.rs`; link
  it, never restate it.
- **The theme is `theme.rs`.** A widget that names a colour or a pixel is a bug; only the
  forced test palettes may hold literals.
- **A command from the retired toolkit is stale.** The window is egui; the checks are the
  window tests and the gate.
- **The layout is `docs/ui-ux.md` §2.2, `layout.rs` and `shell.rs`.**
- **UI strings are keys in `i18n/` (ADR 0045).** A Spanish literal in a widget is migration
  debt, not a second source.
- **A headless test does not prove how it looks.** Attach the light/dark capture when the
  change is visual.
- **A heading rename breaks links silently.** `check-docs.sh` validates the fragment; carry
  the rename into every link in the same change.

## Validation

```bash
./scripts-dev/check-docs.sh
cargo test -p punchi-fluid-window
./scripts-dev/check.sh
```

A skill-only change is documentation: `check-docs.sh` is the proof. A change that touches the
window also runs the tests and keeps the light/dark capture with the pull request.
