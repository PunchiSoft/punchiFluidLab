#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 PunchiSoft
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Live development loop: save, rebuild what changed, relaunch.
#
# The window links the Rust core statically, so a Rust change needs a build and a new
# process; a QML or catalog change needs the same, but the incremental build is short. This
# script watches the sources, builds with the machine's limits, kills the previous window
# and starts the new one. It is the closest safe equivalent of the QML shells that reload
# in place: the state of a run is lost on a rebuild, which is honest for a simulator.
#
# Usage:
#   ./scripts-dev/dev.sh                  # watch and relaunch
#   ./scripts-dev/dev.sh --theme dark     # pass a run.sh option to every launch
#   ./scripts-dev/dev.sh -- --report-paints 4
#
# Stop with Ctrl+C; the window and the watcher are closed together.
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT_DIR"
# shellcheck source=limits.sh
source "$ROOT_DIR/scripts-dev/limits.sh"

build_dir=apps/punchi-fluid-qt/build
binary="$build_dir/punchi-fluid"
app_args=()
running=

while (($# > 0)); do
    case "$1" in
        --toggle | --theme | --grab | --report-paints)
            app_args+=("$1")
            if (($# > 1)); then
                app_args+=("$2")
                shift
            fi
            ;;
        --help | -h)
            sed -n '5,18p' "$0" | sed 's/^# \{0,1\}//'
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

if [[ ! -f "$build_dir/build.ninja" && ! -f "$build_dir/Makefile" ]]; then
    cmake -S apps/punchi-fluid-qt -B "$build_dir" -G Ninja -DCMAKE_BUILD_TYPE=Release -Wno-dev
fi

stamp=$(mktemp)
touch "$stamp"

stop_window() {
    if [[ -n "$running" ]] && kill -0 "$running" 2>/dev/null; then
        kill "$running" 2>/dev/null || true
        wait "$running" 2>/dev/null || true
    fi
    running=
}

cleanup() {
    stop_window
    rm -f "$stamp"
}
trap cleanup EXIT INT TERM

rebuild_and_run() {
    printf '\n[dev] building at %s\n' "$(date +%H:%M:%S)"
    if ! cmake --build "$build_dir"; then
        printf '[dev] build failed; waiting for the next save\n'
        touch "$stamp"
        return 0
    fi
    stop_window
    "$binary" "${app_args[@]}" &
    running=$!
    touch "$stamp"
    printf '[dev] running pid %s; save a source file to reload\n' "$running"
}

rebuild_and_run

while true; do
    sleep 0.4
    changed=$(find crates apps/punchi-fluid-qt assets/catalog \
        -type f \( -name '*.rs' -o -name '*.toml' -o -name '*.qml' -o -name '*.cpp' \
        -o -name '*.h' -o -name '*.png' \) \
        -newer "$stamp" -not -path '*/build/*' -not -path '*/target/*' 2>/dev/null | head -5)
    if [[ -n "$changed" ]]; then
        printf '\n[dev] changed:\n%s\n' "$changed"
        rebuild_and_run
    fi
done
