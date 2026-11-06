#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 PunchiSoft
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Runs the workspace tests.
#
# The build step is `cargo test --no-run`, which recompiles nothing when the sources
# did not change; the test binaries are then executed directly. `--no-build` skips
# even that check and runs the binaries recorded by the previous run.
#
# Usage:
#   ./scripts-dev/test.sh                 # check freshness, then run every test
#   ./scripts-dev/test.sh ribbon          # filter by name across all binaries
#   ./scripts-dev/test.sh -p punchi-fluid-solver
#   ./scripts-dev/test.sh --release
#   ./scripts-dev/test.sh --no-build ribbon   # zero compilation
#
# Every binary runs under a virtual-memory ceiling (a quarter of RAM, see limits.sh), so
# a test that allocates without a bound is stopped with a backtrace instead of taking the
# whole session with it.
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT_DIR"
# shellcheck source=limits.sh
source "$ROOT_DIR/scripts-dev/limits.sh"

profile=debug
profile_flag=()
build=1
test_args=()
cargo_args=()

while (($# > 0)); do
    case "$1" in
        --release)
            profile=release
            profile_flag=(--release)
            ;;
        --no-build | -n)
            build=0
            ;;
        --help | -h)
            sed -n '5,19p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        -p | --package)
            cargo_args+=("$1" "${2:?test.sh: $1 needs a package name}")
            shift
            ;;
        -*)
            cargo_args+=("$1")
            ;;
        *)
            test_args+=("$1")
            ;;
    esac
    shift
done

manifest="target/$profile/test-binaries.txt"

if ((build)); then
    # `--message-format=json` lists one compiler-artifact per test binary; the
    # executable field is empty for targets that are not tests.
    if ((${#cargo_args[@]} > 0)); then
        cargo test "${profile_flag[@]}" --no-run --message-format=json "${cargo_args[@]}" |
            grep -o '"executable":"[^"]*"' |
            cut -d'"' -f4 |
            sort -u >"$manifest"
    else
        cargo test "${profile_flag[@]}" --workspace --no-run --message-format=json |
            grep -o '"executable":"[^"]*"' |
            cut -d'"' -f4 |
            sort -u >"$manifest"
    fi
elif [[ ! -s "$manifest" ]]; then
    printf 'test.sh: %s is missing; run without --no-build once.\n' "$manifest" >&2
    exit 1
fi

status=0
while IFS= read -r binary; do
    [[ -x "$binary" ]] || {
        printf 'test.sh: %s is gone; run without --no-build.\n' "$binary" >&2
        status=1
        continue
    }
    printf '\n==> %s\n' "${binary#"$ROOT_DIR"/}"
    run_capped "$binary" "${test_args[@]}" || status=1
done <"$manifest"

exit "$status"
