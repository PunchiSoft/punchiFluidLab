# 0045 — The interface speaks English through embedded TOML catalogs

- **Status:** accepted
- **Date:** 2026-11-24
- **Deciders:** project owner, implementation session
- **Context crates:** `punchi-fluid-window`, `punchi-fluid-check`, `punchi-fluid-export`,
  `punchi-fluid-cli`, `punchi-fluid-catalog`; new `punchi-fluid-i18n`
- **Supersedes:** [0027](0027-translation-catalogs-are-qts.md)

## Context

The interface is Spanish today and its copy lives where it is drawn: Spanish literals in the
Rust window, `message_es` sentences in the check crate, Spanish labels in the exporter and in
the CLI, and `name_es` in the catalog files. There is no catalog at runtime and no language
setting. The product promises a student with no licence and no connection a program they can
read, and the project has decided that **English is the source and the default**, with Spanish
as a selectable translation.

[ADR 0027](0027-translation-catalogs-are-qts.md) chose Qt's `.ts`/`.qm` catalogs. That
mechanism belonged to the Qt window, which is retired (ADR 0035 and ADR 0036): there is no QML,
no `QTranslator` and no CMake extraction step left. A new mechanism is needed before any
catalog is written, and a string-by-string replacement with English literals would double the
debt instead of paying it.

The audit before this decision found the copy spread across the ribbon, the catalog, the
inspector, the dialogs, the start page, the sheet menu, the check, the exporter and the CLI;
it also found tests that assert Spanish labels and a CLI JSON contract that scripts may read.
The migration cannot be a single replacement: it needs keys, a catalog per language, a
language setting and a migration order.

## Decision

1. **A new crate, `punchi-fluid-i18n`, owns the mechanism.** It holds the `Language` enum, the
   catalog type and the key lookup. It depends on no crate of ours and on no UI, physics or
   file crate; it may use the `serde` and `toml` dependencies the workspace already has. The
   window, the CLI and the exporter may depend on it; the drawing and the domain do not.
2. **Catalogs are embedded TOML files**: `i18n/en.toml` is the source and the default,
   `i18n/es.toml` the Spanish translation. They are compiled in with `include_str!`, parsed
   once at start, and never read from disk or the network at runtime. A key is a stable
   dotted path (`menu.move`, `dialog.page.title`, `notice.moved`); a value is one sentence or
   label, with named parameters (`{count}`, `{name}`), never a concatenation of fragments.
3. **English first.** English is the visible default, not a catalog that appears only when the
   desktop is English. Spanish is selectable in **Configuración**; the choice lives in
   `settings.state`, never in the document or the sheet format. A change applies without
   restarting the window. A missing Spanish key falls back to English and fails the
   completeness test.
4. **No visible literal on a migrated surface.** A migrated surface reads keys from the
   catalog; a test fails on a user-visible literal outside the catalog, with a documented
   allowlist for proper names and units. A key exists in every catalog or the test fails.
5. **The crates speak keys, not sentences.** `punchi-fluid-check` retires `message_es` and
   returns a key with its parameters; the window maps it. The exporter receives its label set
   from the caller and stays locale-free. The CLI's JSON keys and exit codes are a fixed
   English contract; its human messages stay English in this phase, and a `--lang` flag is a
   later decision, not an implicit translation.
6. **Catalog data is data.** `name`, `name_es` and `summary` remain fields of the component
   definition for now: a component is a file, and its name is part of its definition. The
   language selects `name` or `name_es`; `summary` is English until a translated summary field
   is decided with the catalog phase. The card uses the English `name` and `summary`; it does
   not introduce a second description.
7. **Migration order.** Pilot: the sheet menu and the description card. Then the window shell
   (ribbon, catalog, panels, dialogs, status, notices). Then the crates, the catalog data and
   the export; the CLI contract closes the plan. The final phase audits the coverage and the
   no-literal test.
8. **This record supersedes ADR 0027.** The Qt catalogs are retired; no `.ts`/`.qm` file is
   written.

## Consequences

- **A new crate and a dependency edge.** `punchi-fluid-i18n` joins the workspace; `Cargo.toml`,
  the crate table in [AGENTS.md](../../AGENTS.md), the architecture contract and
  `scripts-dev/check-architecture.sh` travel in the implementation change, not in this ADR.
- **The existing Spanish literals become migration debt**, surface by surface. Until a surface
  is migrated it keeps its Spanish text; the window can be mixed during the migration, and the
  plan records which surfaces are done. The pilot is the first complete surface.
- **Tests that assert Spanish labels change** to keys or to structure; no behavioural test is
  deleted or weakened. The audit lists them before the migration.
- **`message_es` is retired**; the check crate returns keys. The exporter's labels move to the
  caller. The CLI contract keeps its English keys and exit codes.
- **No per-frame parse or file read.** The catalogs are parsed once; a language change builds
  the current string view, and the draw loop reads it. A cache is bounded to the two catalogs.
- **The Spanish translation is not a second source.** A sentence is written in English first;
  the Spanish catalog translates it. A Spanish-only sentence is not added to the code.
- **A malicious catalog is not a runtime input**: the catalogs are compiled in, so the parser
  sees only project data; a user catalog path is not part of this decision.
- **The card and the menu already follow the rule**: new product text is English, or comes
  from the definition; the old Spanish labels are the debt this decision retires.

## Alternatives considered

- **Fluent (`fluent-bundle` with `.ftl`)**: stronger plural and gender support, but it adds a
  parser and a dependency for a catalog that today has no complex plural; the workspace already
  uses TOML and `serde`. Fluent remains the option if the translation grows into gender and
  plural rules.
- **English literals in place, no catalog**: no language switch, no Spanish translation, and a
  second replacement later; it contradicts the English-first/selectable-language decision.
- **A window-only module instead of a crate**: the CLI and the exporter would duplicate keys or
  own translations; the crate keeps one catalog and one lookup.
- **Qt's `.ts`/`.qm` catalogs (ADR 0027)**: the Qt window and its toolchain are retired; there
  is no QML or `QTranslator` to load them.
- **A runtime catalog file beside the binary**: an untrusted input, a path to resolve and an
  installation asset; the catalogs are compiled in until a packaged override is a real need.

## References

- [i18n/README.md](../../i18n/README.md) — the state of the copy and the migration.
- [docs/ui-ux.md](../ui-ux.md) — the language section and the interface contract.
- [docs/capabilities.md](../capabilities.md) — the capability ledger.
- [ADR 0027](0027-translation-catalogs-are-qts.md) — the retired Qt catalogs.
- [ADR 0035](0035-the-window-is-rust.md) and [ADR 0036](0036-the-toolkit-is-egui.md) — the
  Rust window and its toolkit.
