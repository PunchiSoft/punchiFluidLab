# Translations

User-facing strings are English in the source. Translation will use Fluent
(`fluent-bundle` + `fluent-templates`), one directory per language here, loaded at
runtime so a packaged build can ship extra languages without recompiling.

Not implemented yet: the interface still has three strings, and adding a
translation pipeline before the UI stabilises would lock in copy that is about to
change.
