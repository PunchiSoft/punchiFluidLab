#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 PunchiSoft
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Runs the desktop application.
#
# By default Cargo checks whether the binary is up to date (a no-op when nothing
# changed) and only then starts it. With --no-build the compiled binary is executed
# directly: no Cargo, no compiler, no waiting.
#
# Usage:
#   ./scripts-dev/run.sh                  # build if stale, then run
#   ./scripts-dev/run.sh --demo           # start actuated and playing
#   ./scripts-dev/run.sh --release        # use the release build
#   ./scripts-dev/run.sh --no-build       # run the existing binary only
#   ./scripts-dev/run.sh --light          # pin the light palette
#   ./scripts-dev/run.sh --dark           # pin the dark palette
#   ./scripts-dev/run.sh -- --help        # arguments after -- go to the app
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT_DIR"

profile=debug
profile_flag=()
build=1
demo=0
app_args=()

while (($# > 0)); do
    case "$1" in
        --release)
            profile=release
            profile_flag=(--release)
            ;;
        --no-build | -n)
            build=0
            ;;
        --demo)
            demo=1
            ;;
        --light | --dark)
            export PUNCHI_FLUID_THEME="${1#--}"
            ;;
        --help | -h)
            sed -n '5,19p' "$0" | sed 's/^# \{0,1\}//'
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

binary="target/$profile/punchi-fluid-lab"

if ((build)); then
    cargo build "${profile_flag[@]}" -p punchi-fluid-desktop --bin punchi-fluid-lab
elif [[ ! -x "$binary" ]]; then
    printf 'run.sh: %s does not exist; run without --no-build once.\n' "$binary" >&2
    exit 1
fi

if [[ ! -x "$binary" ]]; then
    printf 'run.sh: the build finished but %s is still missing.\n' "$binary" >&2
    exit 1
fi

if ((demo)); then
    export PUNCHI_FLUID_DEMO=1
fi

exec "$binary" "${app_args[@]}"
