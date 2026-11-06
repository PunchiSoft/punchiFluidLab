# ADR 0004 — Icons come from the desktop, never from the repository

- **Status:** superseded by [0005](0005-qt-quick-over-a-c-abi.md) — Qt resolves the
  desktop icons itself through `QIcon::fromTheme`, so `punchi-fluid-icons` retires; the
  rule that components are shown by their own symbol still holds.
- **Date:** 2026-11-06
- **Context crates:** `punchi-fluid-icons`, `punchi-fluid-ui`, `punchi-fluid-desktop`

## Context

The toolbar needs pictograms. Three ways to get them:

1. **Ship an icon set** — a font, a sprite sheet or a folder of SVGs in `assets/`. Every
   pictogram then has to be licensed, credited and kept in sync with the theme, and the
   toolbar stops looking like the rest of the session the moment the user changes theme
   or contrast.
2. **Draw them with lines**, which is what this project did first. Honest and
   dependency-free, but a hand-drawn toolbar is exactly what makes an application look
   foreign next to `kate`, `dolphin` or any other KDE program: the desktop ships a
   better set, already themed, already scaled, already accessible.
3. **Ask the desktop** for `document-new`, `edit-undo`, `media-playback-start` — the
   freedesktop names — and draw whatever the theme answers.

The project rule is that no asset of another program enters the repository. Option 3
respects the rule literally: nothing is copied, nothing is redistributed, and the theme's
licence stays between the theme and the user who installed it. It is also what every
native toolkit does (`QIcon::fromTheme`, `g_icon_new_for_string`).

## Decision

A new leaf crate, **`punchi-fluid-icons`**, resolves an icon name against the user's icon
theme and rasterises it:

- **The variant follows the scheme.** A desktop that names its theme in `kdeglobals`
  gets exactly that theme; a desktop that names nothing gets `breeze` on a light window
  and `breeze-dark` on a dark one, because the two themes are different files and a dark
  stroke on a dark bar is worse than no icon at all.
- **Lookup** follows the freedesktop layout: `$XDG_DATA_HOME/icons` and
  `$XDG_DATA_DIRS/icons`, the theme named in `kdeglobals` `[Icons] Theme`, the
  `Inherits` chain of its `index.theme`, ending in `hicolor`. Directories are scored by
  how close their declared size is to the one asked for; a `symbolic` variant is never
  chosen, because a toolbar wants the full-colour icon.
- **Rasterisation** is `resvg` (Apache-2.0 OR MIT) with default features off: Breeze
  action icons are pure paths, so text, system fonts and raster images stay out of the
  dependency graph.
- **The UI owns the textures.** `punchi-fluid-ui::icons` maps an `Action` to its
  freedesktop name, rasterises once per pixel size, uploads an `egui` texture and caches
  it in the ribbon state. The cache is passed in, not parked in the context: asking the
  context for its data while it is uploading a texture deadlocks.
- **The fallback is the label alone.** A machine with no icon theme, a theme missing an
  icon, or a name with no standard produces a text button. Drawing our own pictograms as
  a second source was tried first and dropped: two icon sets drift, and a hand-drawn
  toolbar is exactly the thing that made the window look foreign.

Dependency direction: `punchi-fluid-icons` is a leaf; `punchi-fluid-ui` depends on it;
nothing else does. No crate of ours depends on the desktop.

## Consequences

- The toolbar changes with the desktop theme, at the right size, without a rebuild, and
  it costs the repository nothing: the pictograms are the session's, not ours.
- `resvg` brings a parser and a renderer (~20 crates, all MIT/Apache-2.0/Zlib). That is
  the price of not shipping icons; `cargo tree -d` shows no duplicate major version.
- A screenshot of this application is not portable: on a machine with a different theme
  the toolbar looks different. The review images in `assets/symbols/review/` are about
  the schematic, which is ours and does not depend on the theme.
- Headless environments have no icons and get the drawn fallback, so a CI test can never
  assert "the Breeze icon rendered": it asserts the lookup and the rasteriser separately
  and the fallback path stays exercised.
- A theme installed only through Flatpak or a container is invisible to the lookup; that
  is a known limitation of reading `$XDG_DATA_DIRS` and is acceptable for a desktop
  build.
