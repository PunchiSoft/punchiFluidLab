# 0042 — The appearance is a setting, and the desktop is its default

- **Status:** proposed
- **Date:** 2026-11-23
- **Deciders:** project owner, implementation session
- **Context crates:** `apps/punchi-fluid` (the window)

## Context

Every colour the window paints comes from the desktop: `theme.rs` reads `kdeglobals` once at
start and derives the tokens (ADR 0036, [ui-ux](../ui-ux.md)). That default has a property the
project needs and a limit the owner has now hit: it makes the program belong to the machine,
but the desktop cannot always give the interface the engineering look the owner wants, and
Breeze dark makes `[Colors:View]` equal `[Colors:Window]`, so the sheet's planes had to be
derived rather than read (#219).

The owner asked for selectable themes and one place to choose them. A theme is a palette, not
a drawing: the same recording, the same symbols and the same print path serve every
appearance (ADR 0003). A theme file would be a new untrusted input with its own parser and
bounds; the looks the owner wants are compiled values, not user files.

## Decision

1. **The appearance is view state.** It lives in `settings.state`, never in the document, the
   history or the modified mark. The key is `appearance=desktop|system|<theme>`; `desktop` is
   the default and keeps today's behaviour; `system` chooses the theme assigned to the light
   or dark scheme (`theme_day`, `theme_night`); a theme id fixes one theme. An unknown value
   falls back to `desktop`.
2. **A built-in theme is a `Palette` seed**, not a set of literals in a widget. The seven
   seed colours go through the same `tokens_from` derivation and the same contrast tests as
   the desktop; `paper`, `line`, `ink_dim`, `shadow` and the rest keep their rules. A seed
   that misses a criterion is shifted by `ensure_contrast`, so the shipped look is the
   measured one and may differ from the reference that inspired it.
3. **The selector is one window, Configuración**, reached from **Ver ▸ Tema**, `Ctrl+,`
   and the Inicio rail. It applies live, closes with `Esc` or **Cerrar**, returns the
   focus and never touches the document. Its first two pages are Apariencia and Paneles; a
   page appears only when its settings exist.
4. **One theme per change.** A theme arrives as its seed, its card, its row in the contrast
   table and its screenshots. Several themes in one diff are not reviewed together.
5. **No theme files, import, export or follow-a-file.** That would be a new format and an
   untrusted parser for a need the compiled themes already cover.

## Consequences

- The program still opens with the desktop's colours; a chosen theme overrides them until the
  owner returns to `Escritorio`.
- `Sistema` needs a light and a dark theme to be offered (CAD claro and Pizarra); until then
  the modes are `Escritorio` and `Un tema`.
- The settings file gains optional keys without a version bump; an older build ignores them
  and keeps its panels.
- The theme name is not a document field: a file opened on another machine keeps its circuit
  and takes the local appearance.
- Every theme adds a contrast row and a screenshot matrix to the evidence; that is the cost
  of the setting, paid one theme at a time.
- The theme adapter remains the only colour source. A widget that reads a literal is still a
  bug.

## Alternatives considered

- **No themes.** Rejected by the owner: the desktop palette cannot supply the engineering
  look, and the window should not depend on the machine's taste.
- **Theme files with import and a follow-a-file mode.** Rejected for this decision: a new
  untrusted format needs its own parser, limits and security review, and the designs the
  owner asked for are compiled values.
- **Every theme in one change.** Rejected: review size, and a contrast table that nobody
  reads in one sitting.
- **A theme row inline on Inicio.** Rejected: two selectors drift; Inicio gets the entry and
  Configuración owns the choice.

## References

- [ADR 0036](0036-the-toolkit-is-egui.md), the toolkit and the theme adapter.
- [ADR 0003](0003-one-drawing-two-devices.md), one drawing behind a `Surface`.
- [ui-ux.md](../ui-ux.md), the token table, *Following the desktop* and *Configuración*.
- #219, the sheet's planes that the derived palette had to separate.
