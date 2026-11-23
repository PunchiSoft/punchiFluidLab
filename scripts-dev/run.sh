#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 PunchiSoft
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Builds and runs the Rust window in apps/punchi-fluid (ADR 0035, ADR 0036).
#
# Usage:
#   ./scripts-dev/run.sh                  # build what changed, then run
#   ./scripts-dev/run.sh --no-build       # run the existing binary only
#   ./scripts-dev/run.sh --release        # build the release profile
#   ./scripts-dev/run.sh -- <file>        # arguments after -- go to the app
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT_DIR"
# shellcheck source=limits.sh
source "$ROOT_DIR/scripts-dev/limits.sh"

build=1
release=0
app_args=()

while (($# > 0)); do
    case "$1" in
        --no-build | -n)
            build=0
            ;;
        --release)
            release=1
            ;;
        --help | -h)
            sed -n '5,13p' "$0" | sed 's/^# \{0,1\}//'
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
    if ((release)); then
        cargo build -p punchi-fluid-window --bin punchi-fluid --release
    else
        cargo build -p punchi-fluid-window --bin punchi-fluid
    fi
fi

profile=debug
((release)) && profile=release
binary="target/$profile/punchi-fluid"

if [[ ! -x "$binary" ]]; then
    printf 'run.sh: %s is missing; run without --no-build once.\n' "$binary" >&2
    exit 1
fi

exec "$binary" "${app_args[@]}"
