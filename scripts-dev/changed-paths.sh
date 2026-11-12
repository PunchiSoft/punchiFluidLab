#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 PunchiSoft
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Classifies a change as documentation-only or code. The CI uses it to run the full
# Rust and window gate only when a file the gate can observe has changed.
#
# Usage:
#   ./scripts-dev/changed-paths.sh --stdin < files
#   ./scripts-dev/changed-paths.sh --git <base> <head>
#   ./scripts-dev/changed-paths.sh <file>...
#
# Prints `docs` or `code` and exits 0. A usage error exits 2; a git failure exits 1.
# The allow-list is deliberately narrow: an unknown path is code. The assets that are
# compiled into the binary or checked by a test are code, even when they are Markdown.
set -euo pipefail

is_docs() {
    case "$1" in
        # The documentation tree and the community files.
        docs/*)
            return 0
            ;;
        README.md|README.es.md|AGENTS.md|CHANGELOG.md|CONTRIBUTING.md|SECURITY.md|SUPPORT.md|CODE_OF_CONDUCT.md|LICENSE)
            return 0
            ;;
        # Captures are referenced by the READMEs, not compiled.
        assets/screenshots/*)
            return 0
            ;;
        # Authoring and provenance guides. The .toml/.wav files and the symbol reviews
        # next to them are compiled in or observed by tests, so they stay code. The
        # patterns are explicit because a `case` glob matches `/` too.
        assets/catalog/README.md|assets/symbols/README.md|assets/sounds/README.md)
            return 0
            ;;
        # Planning documents. A translation catalogue or a packaging recipe is code.
        i18n/*.md|packaging/*.md)
            return 0
            ;;
        # Procedures for agents. A script beside a skill is code and falls through.
        .agents/skills/*.md|.agents/skills/*/*.md)
            return 0
            ;;
        # The issue forms and the label manifest are checked by check-docs.sh.
        .github/ISSUE_TEMPLATE/*|.github/PULL_REQUEST_TEMPLATE.md|.github/labels.txt)
            return 0
            ;;
        # Anything else, including Cargo files, crates, apps, scripts, workflows and
        # the compiled assets, runs the full gate.
        *)
            return 1
            ;;
    esac
}

usage() {
    printf 'usage: %s --stdin | --git <base> <head> | <file>...\n' "$0" >&2
    exit 2
}

files=()
case "${1:-}" in
    --stdin)
        (($# == 1)) || usage
        mapfile -t files
        ;;
    --git)
        (($# == 3)) || usage
        if ! diff=$(git diff --name-only "$2...$3"); then
            printf 'changed-paths: git diff %s...%s failed\n' "$2" "$3" >&2
            exit 1
        fi
        mapfile -t files <<<"$diff"
        ;;
    '')
        usage
        ;;
    *)
        files=("$@")
        ;;
esac

for file in "${files[@]}"; do
    [[ -z $file ]] && continue
    if ! is_docs "$file"; then
        printf 'code\n'
        exit 0
    fi
done
printf 'docs\n'
