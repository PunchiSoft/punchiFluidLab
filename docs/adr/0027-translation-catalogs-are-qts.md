# 0027 — The translation catalogs are Qt's, and the core speaks keys

- **Status:** accepted
- **Date:** 2026-11-14
- **Deciders:** project owner, implementation session
- **Context crates:** `punchi-fluid-check`, `punchi-fluid-export`, `punchi-fluid-ffi`, `punchi-fluid-qt`
- **Amends:** the Fluent sentence in `docs/plan.md` (M6) and `docs/ui-ux.md` (Language)

## Context

The interface is Spanish today and its copy lives where it is drawn: `qsTr()` in the QML
files, `QStringLiteral` in the C++ that drives the session, and Spanish sentences that the
Rust crates hand to the window through the ABI (`FindingKind::message_es`, the title-block
headers, the inspector hints). There is no catalog, no extraction step and no translator:
`i18n/` holds a plan, not a language.

The earlier text planned Fluent (`fluent-bundle` + `fluent-templates`) with
`i18n/en.ftl` and `i18n/es.ftl`. That plan predates the Qt Quick front end of
[ADR 0005](0005-qt-quick-over-a-c-abi.md). Fluent is a Rust library; the strings are in
QML and C++, so the plan would need a bridge that carries every message across the ABI in
both directions. A student who reads Spanish and a course that wants the interface in
another language both need the copy in one place, and the project must not own a
translation runtime it does not need.

Qt already extracts and loads catalogs for the language the strings are written in:
`lupdate` reads `qsTr()` and `tr()`, `lrelease` writes the `.qm`, and `QTranslator`
installs it. The front end uses Qt Quick by decision, and its translation tooling is part
of that decision, not a second mechanism beside it.

## Decision

- **The catalog is Qt's.** `i18n/punchi-fluid_es.ts` is the Spanish catalog, extracted from
  the QML and C++ sources and compiled to `.qm`; English is the source language and needs
  no file. The window installs a `QTranslator` on start and when the desktop locale
  changes, never per frame. A `.qm` of the same name under
  `$XDG_DATA_HOME/punchi-fluid-lab/i18n/` replaces the built-in one, so a packaged build
  ships another language without a new binary.
- **The core returns keys, never sentences.** A Rust crate that reports a fault returns
  its stable English name (`outside_sheet`), not the text the student reads; the window
  owns the mapping from key to catalog message. `FindingKind::message_es`, the category
  and actuation helpers of the ABI and the Spanish title-block headers are retired in M6,
  not translated in place.
- **The exporter receives its labels.** `punchi-fluid-export` has no locale: the caller
  passes the label set for the title block and the parts list, and the default is
  English. A printed page and the window that asked for it then agree on the language.
- **One sentence, one message.** No string is built by concatenation and no plural is
  written as `(s)`: placeholders are `%1` and Qt's `%n`, so a translator can reorder the
  sentence and a language with three plural forms is not forced into two.
- **Identifiers are not copy.** Enum variants, file-format tokens, ABI names and test
  names stay English; only the catalog and the catalog-backed labels are Spanish. The
  content model of format 11 already broke this rule with `escolar`/`tecnico`; format 12
  renames them to `school`/`technical` and reads the old file as the model it named.

## Consequences

- **Easier:** one extraction command, one catalog per language, no bridge and no new Rust
  dependency. The translator edits text, never code. The locale is the desktop's, which is
  the same source as the palette and the icon theme.
- **Harder:** the copy that is a bare `QStringLiteral` in C++ must first be wrapped in
  `tr()`, or `lupdate` cannot see it. The Rust strings need ABI changes: the check already
  exposes its stable key, while the title-block headers and the parts list need the label
  set the exporter receives.
- **Forbidden:** a Spanish sentence in a Rust crate, a translated string in the ABI, or a
  test that asserts the Spanish copy instead of the key. The tests that do assert copy
  today (`"se sale de la hoja"`, `"ok: sin faltas"`) move to the key in M6.
- **Debt, paid in M6:** the Spanish copy stays inside the widgets until the catalogs land;
  the branch that renamed the content models translates nothing. `i18n/README.md` states
  the real state, and the translation test ("every user-visible string comes from a file")
  is part of the M6 acceptance criteria.
- **Not affected:** circuit files, the catalog definitions and the model. A `.pfluid`
  stores data, not interface copy.

## Alternatives considered

- **Fluent in Rust, as the earlier plan said.** Lost because the strings are in QML and
  C++; Fluent has no Qt bridge, so the project would either duplicate every message in two
  systems or write and own the bridge. No Fluent code exists to preserve.
- **GNU gettext.** Lost for the same reason: `.po` needs an extraction and lookup layer for
  QML and C++, while Qt's extraction, plural handling and translator tooling already
  exist and are maintained with the toolkit the window uses.
- **English only.** Lost because the project's first reader is a Spanish-speaking student
  and the owner teaches in Spanish; the interface being readable in the classroom is part
  of the mission, not a preference.

## References

- [docs/ui-ux.md](../ui-ux.md), the Language section and what is still open.
- [docs/plan.md](../plan.md), M6 and its acceptance criteria.
- [i18n/README.md](../../i18n/README.md), the current state and the pipeline.
- [ADR 0005](0005-qt-quick-over-a-c-abi.md), the Qt Quick front end over a C ABI.
