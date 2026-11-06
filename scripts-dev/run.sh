#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 PunchiSoft
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Builds and runs the window.
#
# The front end is the Qt application in apps/punchi-fluid-qt, which links the Rust core
# as a static library. There is no cargo binary to run any more: `cargo run -p
# punchi-fluid-desktop` stopped existing when ADR 0005 retired that crate.
#
# Usage:
#   ./scripts-dev/run.sh                  # build what changed, then run
#   ./scripts-dev/run.sh --toggle         # start with the valve actuated
#   ./scripts-dev/run.sh --theme dark     # pin a palette (light or dark)
#   ./scripts-dev/run.sh --no-build       # run the existing binary only
#   ./scripts-dev/run.sh -- --report-paints 4   # arguments after -- go to the app
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT_DIR"
# shellcheck source=limits.sh
source "$ROOT_DIR/scripts-dev/limits.sh"

build_dir=apps/punchi-fluid-qt/build
binary="$build_dir/punchi-fluid"
build=1
app_args=()

while (($# > 0)); do
    case "$1" in
        --no-build | -n)
            build=0
            ;;
        --release)
            # The Rust core is always linked in release: the drawing budget is measured
            # there. Accepted so the flag keeps meaning something for anyone used to it.
            ;;
        --toggle | --theme | --grab | --report-paints)
            app_args+=("$1")
            if (($# > 1)); then
                app_args+=("$2")
                shift
            fi
            ;;
        --help | -h)
            sed -n '5,16p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        --)
            shift
            app_args+=("$@")
            break
            ;;
        *)
            app_args+=("$1")
            ;;
    esac
    shift
done

if ((build)); then
    if [[ ! -f "$build_dir/build.ninja" && ! -f "$build_dir/Makefile" ]]; then
        cmake -S apps/punchi-fluid-qt -B "$build_dir" -G Ninja -DCMAKE_BUILD_TYPE=Release -Wno-dev
    fi
    # CMake drives `cargo build --release -p punchi-fluid-ffi` through its rust_core
    # target, so the core is never stale relative to the window that links it.
    cmake --build "$build_dir"
fi

if [[ ! -x "$binary" ]]; then
    printf 'run.sh: %s is missing; run without --no-build once.\n' "$binary" >&2
    exit 1
fi

exec "$binary" "${app_args[@]}"
