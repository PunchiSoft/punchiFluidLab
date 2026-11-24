# Translations

**Status: target.** The mechanism is decided and its first slice is implemented:
`punchi-fluid-i18n` reads the two embedded catalogs, **Configuración** offers the language,
and the whole Rust window is translated: the ribbon, the catalog, the inspector, Configuración,
the check panel and its finding sentences, the start page, the status bar and its notices, the
lifecycle panels and every dialog. The exporter labels and the catalog summary are the
remaining phase-4 debt, and the CLI is a fixed English contract. The technical error causes
from `punchi-fluid-file` and `punchi-fluid-session` stay in English as a diagnostic inside the
translated frame: they are not UI copy, and a typed-error translation would be a separate
decision.

## The decision

[ADR 0045](../docs/adr/0045-the-interface-speaks-english-through-embedded-toml-catalogs.md)
supersedes [ADR 0027](../docs/adr/0027-translation-catalogs-are-qts.md). The Qt `.ts`/`.qm`
catalogs retired with the Qt window; the Rust window uses a new crate and embedded TOML.

- **A crate owns the mechanism.** `punchi-fluid-i18n` holds `Language`, the catalog and the
  lookup. It depends on no crate of ours and on no UI; the window, the CLI and the exporter
  may depend on it. The crate, its dependency edge and the architecture table travel in the
  implementation change.
- **Catalogs are embedded TOML.** `i18n/en.toml` is the source and the default;
  `i18n/es.toml` is the Spanish translation. Both are compiled in with `include_str!`,
  parsed once at start and never read from disk or the network. A key is a stable dotted
  path (`menu.move`, `notice.moved`); a value is one sentence with named parameters, never a
  concatenation.
- **English first.** English is the visible default, not a catalog that appears only when the
  desktop is English. Spanish is selected in **Configuración**, stored in `settings.state`,
  applied without restarting the window. A missing Spanish key falls back to English and
  fails the completeness test.
- **Keys in the crates.** `punchi-fluid-check` retires `message_es` and returns a key with its
  parameters; the window translates it. The exporter receives its labels from the caller and
  stays locale-free. The CLI's JSON keys and exit codes are a fixed English contract; its
  human messages stay English in this phase.
- **No visible literal on a migrated surface.** A test fails on a user-visible literal outside
  the catalog, with a documented allowlist for proper names and units. Every key exists in
  every catalog or the test fails.
- **Catalog data stays data.** `name`, `name_es` and `summary` remain fields of the component
  definition; the language selects `name` or `name_es`; `summary` is English until a
  translated summary is decided with the catalog phase. The description card uses the English
  `name` and `summary` and does not introduce a second description.
- **No per-frame parse or file read.** The catalogs are parsed once; a language change builds
  the string view the widgets read. A cache is bounded to the two catalogs.

## Migration order

1. **Pilot:** the sheet menu and the description card.
2. **Window shell:** ribbon, catalog, panels, dialogs, status bar and notices.
3. **Crates and data:** check keys, catalog names and summaries, exporter labels.
4. **CLI contract:** English keys and exit codes; a `--lang` flag is a later decision.
5. **Final audit:** coverage, the no-literal test and the light/dark captures.

The audit before the pilot classifies every string, test and contract. The audit and the
migration plan are the owner's working notes, outside the repository.

## Where the copy lives today

- **The Rust window** (`apps/punchi-fluid`): every surface and every notice reads the catalog,
  and the check crate returns only a stable `kind`; the panel maps it to `check.finding.*`. The
  exporter labels and the catalog summary are phase-4 debt; the crate error causes are
  technical English diagnostics inside the translated frame.
- **The crates:** `punchi-fluid-check::FindingKind::message_es`, the title-block headers in
  `punchi-fluid-export`, the parts-list words and the CLI output are Spanish sentences, not
  keys.
- **Catalog data:** `name` and `name_es` in `assets/catalog/*.toml`, documented in
  [assets/catalog/README.md](../assets/catalog/README.md). The description card uses `name`
  and `summary`, which are English.
- **The new surfaces** already follow the rule: the card's text is the English definition,
  and no new Spanish literal was added for the card.

## What is not done

- The exporter labels and the catalog summary are still pending; the whole Rust window and
  the check findings are on the catalog today, and the CLI speaks English.
- The manual classification of every string, test and contract is still pending for the
  surfaces after the pilot; the pilot's key map is the first classified slice.
- The catalog data (`name_es`, `summary`) and the Spanish translation of the rest of the
  window are a later slice; the English copy of the remaining surfaces is not written.
- No light/dark capture of the language selector or the translated menu is attached yet; the
  headless tests cover the switch.
