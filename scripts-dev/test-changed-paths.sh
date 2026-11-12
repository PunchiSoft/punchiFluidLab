#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 PunchiSoft
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Tests the documentation-only classifier. A path that is not on the allow-list must
# be code: a wrong `docs` answer skips the Rust gate on a change it can observe.
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
CLASSIFIER="$ROOT_DIR/scripts-dev/changed-paths.sh"
fail=0

check() {
    local expected=$1
    shift
    local actual
    actual=$(printf '%s\n' "$@" | "$CLASSIFIER" --stdin)
    if [[ $actual != "$expected" ]]; then
        printf 'changed-paths: expected %s for [%s], got %s\n' "$expected" "$*" "$actual" >&2
        fail=1
    fi
}

# Documentation, community files and the pages that are not compiled.
check docs docs/README.md
check docs README.md docs/guides/first-circuit.md
check docs assets/catalog/README.md
check docs assets/symbols/README.md
check docs assets/sounds/README.md
check docs assets/screenshots/window-dark.png
check docs i18n/README.md
check docs packaging/README.md
check docs .agents/skills/rust-idioms/SKILL.md
check docs .agents/skills/README.md
check docs .github/ISSUE_TEMPLATE/bug_report.yml
check docs .github/PULL_REQUEST_TEMPLATE.md
check docs .github/labels.txt
check docs LICENSE

# The compiled assets and the files a test or a tool observes.
check code assets/catalog/valve.3_2.spring_return.toml
check code assets/sounds/air.wav
check code assets/symbols/review/valve.3_2.spring_return.md
check code crates/punchi-fluid-core/src/lib.rs
check code apps/punchi-fluid-qt/Main.qml
check code Cargo.toml
check code Cargo.lock
check code rust-toolchain.toml
check code scripts-dev/check.sh
check code scripts-dev/changed-paths.sh
check code .github/workflows/ci.yml
check code .github/CODEOWNERS
check code deny.toml

# One code file is enough to run the full gate, and an empty diff is docs.
check code docs/README.md crates/punchi-fluid-core/src/lib.rs
check docs

if ((fail)); then
    exit 1
fi
printf 'changed-paths: ok\n'
