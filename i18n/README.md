# Translations

**Status: target.** Nothing here is loaded at runtime yet. The interface is Spanish and
its copy lives in the QML and C++ that draws it.

## Where the copy lives today

- **QML:** `qsTr()` in `apps/punchi-fluid-qt/*.qml` — the command labels, hints, panels
  and the About and Licence pages. It is already the marker `lupdate` reads.
- **C++:** `QStringLiteral` in `apps/punchi-fluid-qt/src/*.cpp` — the state messages and
  the field labels of the title-block editor. Not wrapped in `tr()` yet, so extraction
  does not see them.
- **Rust:** sentences reach the window through the ABI — the check's `message_es`, the
  title-block headers and the inspector hints. They are not catalogs either.
- **Catalog data:** `name` and `name_es` in `assets/catalog/*.toml`, already documented
  in [assets/catalog/README.md](../assets/catalog/README.md).

## The decision

[ADR 0027](../docs/adr/0027-translation-catalogs-are-qts.md) settles the mechanism: the
catalogs are Qt's.

- `i18n/punchi-fluid_es.ts` is the Spanish catalog; English is the source language and
  needs no file. `lupdate` extracts `qsTr()` and `tr()`, `lrelease` compiles the `.qm`,
  and the window installs a `QTranslator` on start and when the desktop locale changes.
- The Rust crates return stable English keys, never sentences. The window turns a key into
  the catalog message.
- The exporter has no locale: the caller passes the label set for the title block and the
  parts list, and the default is English.
- No concatenated sentences and no `(s)` plural: placeholders are `%1` and `%n`.

## The pipeline (M6)

```bash
lupdate apps/punchi-fluid-qt -ts i18n/punchi-fluid_es.ts   # after wrapping the C++ copy
lrelease i18n/punchi-fluid_es.ts -qm i18n/punchi-fluid_es.qm
```

1. Wrap the remaining C++ `QStringLiteral` copy in `tr()` and give the ABI keys instead
   of sentences.
2. Extract with `lupdate`, translate the `.ts`, and compile it with `lrelease` in CMake so
   the catalog travels inside the binary.
3. Load the built-in catalog at start; a `.qm` of the same name under
   `$XDG_DATA_HOME/punchi-fluid-lab/i18n/` replaces it, so a packaged build ships another
   language without a new binary.
4. Add the test that fails on a user-visible string that does not come from a catalog.

## Adding a language

Same steps, one `.ts` per language. A language is complete when the interface, the check
messages, the title block and the parts list all resolve to it; a missing message falls
back to English, never to an empty label.

## What is not done

M6 pays the debt: extraction, the Spanish catalog, the C++ `tr()` pass, keys instead of
sentences in the ABI, the exporter label set and the hardcoded-string test. The branch
that renamed the content-model tokens to English did not translate any copy.
