#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 PunchiSoft
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Live development loop: save, rebuild what changed, relaunch.
#
# The window links every crate statically, so a source change needs a build and a new
# process; the incremental build is short. This script watches the sources, builds with the
# machine's limits, kills the previous window and starts the new one. There is no in-place
# reload: the relaunch is the loop, and the state of a run is lost on a rebuild, which is
# honest for a simulator.
#
# Usage:
#   ./scripts-dev/dev.sh                  # watch and relaunch
#   ./scripts-dev/dev.sh -- <file>        # open a sheet on every launch
#
# Stop with Ctrl+C; the window and the watcher are closed together.
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT_DIR"
# shellcheck source=limits.sh
source "$ROOT_DIR/scripts-dev/limits.sh"

binary=target/debug/punchi-fluid
app_args=()
running=

while (($# > 0)); do
    case "$1" in
        --help | -h)
            sed -n '5,17p' "$0" | sed 's/^# \{0,1\}//'
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

stamp=$(mktemp)
touch "$stamp"

stop_window() {
    if [[ -n "$running" ]] && kill -0 "$running" 2>/dev/null; then
        kill "$running" 2>/dev/null || true
        wait "$running" 2>/dev/null || true
    fi
    running=
    local autosave="${XDG_DATA_HOME:-$HOME/.local/share}/PunchiSoft/Punchi Fluid Lab/autosave.pfluid"
    rm -f "$autosave"
}

cleanup() {
    stop_window
    rm -f "$stamp"
}

trap cleanup EXIT INT TERM

rebuild_and_run() {
    printf '\n[dev] building at %s\n' "$(date +%H:%M:%S)"
    if ! cargo build -p punchi-fluid-window --bin punchi-fluid; then
        printf '[dev] build failed; waiting for the next save\n'
        touch "$stamp"
        return 0
    fi
    stop_window
    "$binary" "${app_args[@]}" &
    running=$!
    touch "$stamp"
    printf '[dev] running pid %s; save a source file to rebuild and relaunch\n' "$running"
}

rebuild_and_run

while true; do
    sleep 0.4
    changed=$(find crates apps/punchi-fluid assets/catalog assets/symbols \
        -type f \( -name '*.rs' -o -name '*.toml' -o -name '*.png' \) \
        -newer "$stamp" -not -path '*/target/*' 2>/dev/null | head -5)
    if [[ -n "$changed" ]]; then
        printf '\n[dev] changed:\n%s\n' "$changed"
        rebuild_and_run
    fi
done
