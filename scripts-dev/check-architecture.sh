#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 PunchiSoft
# SPDX-License-Identifier: GPL-3.0-or-later
#
# The workspace contract that `cargo` cannot enforce: it prevents cycles, not layering.
# A `solver` depending on `draw` compiles; it is still wrong. So the graph is written down
# here, `unsafe` has one home, and a file that grows past the budget has to be justified
# in this file rather than discovered in review.
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT_DIR"

# Allowed edges, one per line: <dependent> <dependency>. Everything else is a finding.
# The reasoning for each is in AGENTS.md and .agents/skills/rust-workspace-architecture.
ALLOWED_EDGES=(
    "punchi-fluid-file punchi-fluid-core"
    "punchi-fluid-solver punchi-fluid-core"
    "punchi-fluid-solver punchi-fluid-catalog"
    "punchi-fluid-draw punchi-fluid-core"
    "punchi-fluid-draw punchi-fluid-catalog"
    "punchi-fluid-draw punchi-fluid-solver"
    "punchi-fluid-doc punchi-fluid-core"
    "punchi-fluid-doc punchi-fluid-draw"
    "punchi-fluid-check punchi-fluid-catalog"
    "punchi-fluid-check punchi-fluid-doc"
    "punchi-fluid-check punchi-fluid-draw"
    "punchi-fluid-export punchi-fluid-catalog"
    "punchi-fluid-export punchi-fluid-core"
    "punchi-fluid-export punchi-fluid-draw"
    "punchi-fluid-export punchi-fluid-solver"
    "punchi-fluid-ffi punchi-fluid-catalog"
    "punchi-fluid-ffi punchi-fluid-check"
    "punchi-fluid-ffi punchi-fluid-core"
    "punchi-fluid-ffi punchi-fluid-doc"
    "punchi-fluid-ffi punchi-fluid-draw"
    "punchi-fluid-ffi punchi-fluid-export"
    "punchi-fluid-ffi punchi-fluid-file"
    "punchi-fluid-ffi punchi-fluid-solver"
    "punchi-fluid-cli punchi-fluid-catalog"
    "punchi-fluid-cli punchi-fluid-check"
    "punchi-fluid-cli punchi-fluid-core"
    "punchi-fluid-cli punchi-fluid-doc"
    "punchi-fluid-cli punchi-fluid-draw"
    "punchi-fluid-cli punchi-fluid-file"
    "punchi-fluid-cli punchi-fluid-solver"
)

# `unsafe` lives in the ABI and nowhere else. The one file listed here is where it lives
# today; the split planned for it removes this exemption.
UNSAFE_ALLOWED=(
    "crates/punchi-fluid-ffi/src/lib.rs"
)

# Files over the budget. Every entry is a debt with a plan; adding a line here is a
# decision, not a formality.
BIG_ALLOWED=(
    "crates/punchi-fluid-ffi/src/lib.rs"
    "crates/punchi-fluid-ffi/tests/abi.rs"
    "crates/punchi-fluid-draw/src/symbol.rs"
    "crates/punchi-fluid-solver/src/network.rs"
    "crates/punchi-fluid-solver/src/pneumatic.rs"
    "crates/punchi-fluid-doc/src/lib.rs"
    "apps/punchi-fluid-qt/src/main.cpp"
    "apps/punchi-fluid-qt/src/session.cpp"
)
BUDGET=800

status=0

contains() {
    local needle=$1
    shift
    local item
    for item in "$@"; do
        [[ $item == "$needle" ]] && return 0
    done
    return 1
}

# --- 1. the dependency graph -------------------------------------------------------
edges=$(cargo metadata --no-deps --format-version 1 | python3 -c "
import json, sys

meta = json.load(sys.stdin)
ours = {package['name'] for package in meta['packages']}
for package in sorted(meta['packages'], key=lambda p: p['name']):
    for dependency in package['dependencies']:
        if dependency['name'] in ours:
            print(package['name'], dependency['name'])
")

while read -r from to; do
    [[ -z $from ]] && continue
    if ! contains "$from $to" "${ALLOWED_EDGES[@]}"; then
        echo "check-architecture: $from depends on $to, which is not in the allowed graph" >&2
        status=1
    fi
done <<<"$edges"

# --- 2. unsafe ---------------------------------------------------------------------
while IFS= read -r file; do
    [[ -z $file ]] && continue
    if ! contains "$file" "${UNSAFE_ALLOWED[@]}"; then
        echo "check-architecture: unsafe code outside the ABI in $file" >&2
        status=1
    fi
done < <(grep -rl --include=*.rs -E 'unsafe (fn|impl|extern|\{|\()' crates | grep -v '/tests/' || true)

# --- 3. file size ------------------------------------------------------------------
while read -r lines file; do
    [[ -z ${file:-} ]] && continue
    (( lines > BUDGET )) || continue
    if ! contains "$file" "${BIG_ALLOWED[@]}"; then
        echo "check-architecture: $file has $lines lines, over the $BUDGET-line budget" >&2
        status=1
    fi
done < <(find crates apps \
    \( -name build -o -name target -o -name .git \) -prune -o \
    -type f \( -name '*.rs' -o -name '*.cpp' -o -name '*.h' \) -print |
    xargs wc -l | grep -v 'total$')

if ((status == 0)); then
    echo "check-architecture: graph, unsafe and file sizes are within the contract"
fi
exit "$status"
