# Translations

**Status: target.** Nothing here is loaded at runtime yet. The interface is Spanish and its
copy is hardcoded in the Rust window and in the crates that hand it sentences.

## Where the copy lives today

- **The Rust window** (`apps/punchi-fluid`): ribbon labels, sidebar categories, status bar,
  notices and dialog text are Spanish string literals.
- **The crates:** `punchi-fluid-check::FindingKind::message_es`, the title-block headers in
  `punchi-fluid-export`, the parts-list words and the CLI output are Spanish sentences, not
  keys.
- **Catalog data:** `name` and `name_es` in `assets/catalog/*.toml`, documented in
  [assets/catalog/README.md](../assets/catalog/README.md); the window currently chooses
  `name_es` first.

## The decision that no longer applies

[ADR 0027](../docs/adr/0027-translation-catalogs-are-qts.md) chose Qt's `.ts`/`.qm`
catalogs. That mechanism belonged to the Qt window, which is retired (ADR 0035, ADR 0036):
there is no QML, no `QTranslator` and no CMake step to extract from. The plan must be
re-decided for the Rust/egui window before any catalog is written.

## What the new decision must settle

1. **English first.** English is the default visible language, not a catalogue that appears
   only when the desktop is English. Spanish remains a selectable translation.
2. **A translation package, not string replacement.** The candidates are Fluent
   (`fluent-bundle` with embedded `.ftl` catalogs) and a small project catalog crate with
   TOML. The mechanism is an ADR; no Spanish sentence is replaced by an English literal in
   place.
3. **Keys in the crates.** `FindingKind::as_str` is already a stable key; `message_es` is
   retired, and the window maps the key to a catalog message. The exporter receives its
   label set from the caller and stays locale-free.
4. **No per-frame lookup.** Labels resolve once on language change, not in the draw loop.
5. **A test that fails on a visible literal** outside a catalog, and a completeness test for
   every message in every catalog.

## What is not done

No catalog file exists, no runtime loads one, no language switch exists, and the English
copy has not been written. The current state is the debt the audit records.
