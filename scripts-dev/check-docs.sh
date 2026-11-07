#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 PunchiSoft
# SPDX-License-Identifier: GPL-3.0-or-later
#
# The workspace layout is declared in Cargo.toml and described in two documents that
# people and agents read instead of the manifest. Four retired crate names survived a
# whole toolkit change because nothing compared the two, so this compares them.
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT_DIR"

DOCS=(
    "AGENTS.md"
    "README.md"
    "README.es.md"
    ".agents/skills/rust-workspace-architecture/SKILL.md"
)
RETIRED=(
    "punchi-fluid-theme"
    "punchi-fluid-ui"
    "punchi-fluid-desktop"
    "punchi-fluid-icons"
)

members=$(sed -n '/^members = \[/,/^\]/p' Cargo.toml | grep -oE '"[^"]+"' | tr -d '"')
if [[ -z $members ]]; then
    echo "check-docs: no workspace members found in Cargo.toml" >&2
    exit 1
fi

status=0
for doc in "${DOCS[@]}"; do
    if [[ ! -f $doc ]]; then
        echo "check-docs: $doc is missing" >&2
        status=1
        continue
    fi
    while read -r member; do
        name=$(basename -- "$member")
        if ! grep -qF -- "$name" "$doc"; then
            echo "check-docs: $doc does not list the crate $name" >&2
            status=1
        fi
    done <<<"$members"
    for retired in "${RETIRED[@]}"; do
        if grep -qF -- "$retired" "$doc"; then
            echo "check-docs: $doc still names $retired, retired by ADR 0005" >&2
            status=1
        fi
    done
done

if ((status == 0)); then
    echo "check-docs: the workspace documents match Cargo.toml"
fi
exit "$status"
