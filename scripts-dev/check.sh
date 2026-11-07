#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 PunchiSoft
# SPDX-License-Identifier: GPL-3.0-or-later
#
# The whole automated gate: formatting, lints, tests and a headless smoke test of
# the CLI. No window, no network.
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT_DIR"
# shellcheck source=limits.sh
source "$ROOT_DIR/scripts-dev/limits.sh"

cargo fmt --all --check
# The documents that declare the crate layout are read instead of the manifest, so they
# are checked against it rather than trusted.
./scripts-dev/check-docs.sh
cargo clippy --workspace --all-targets -- -D warnings
# The tests run through test.sh rather than `cargo test` so each binary is capped: a
# runaway allocation in one test must not take the editor with it.
./scripts-dev/test.sh

# A circuit file that exercises the format, the domain and the solver without a UI.
example=$(mktemp --suffix=.pfluid)
trap 'rm -f "$example"' EXIT
cat >"$example" <<'JSON'
{
  "format": 1,
  "title": "Smoke test",
  "components": [
    { "id": "supply", "name": "Air supply", "kind": { "type": "supply", "pressure_bar": 6.0 } }
  ],
  "connections": []
}
JSON

cargo run --quiet --bin punchi-fluid -- validate "$example"
cargo run --quiet --bin punchi-fluid -- simulate "$example"

# The window is where the drawing and the gestures live, and nothing above touches it: the
# sheet once answered no click at all with every Rust test green. The app is built and
# driven headlessly here, and the step is skipped when Qt was never configured so a machine
# without it still runs the gate.
if [[ -d apps/punchi-fluid-qt/build ]] && command -v cmake >/dev/null; then
    cmake --build apps/punchi-fluid-qt/build >/dev/null
    app=apps/punchi-fluid-qt/build/punchi-fluid
    export QT_QPA_PLATFORM=offscreen
    "$app" --check-qml
    "$app" --check-theme
    "$app" --check-icons
    "$app" --selftest
else
    if [[ ${PUNCHI_REQUIRE_UI:-0} == 1 ]]; then
        echo "ui gate: the window is required but apps/punchi-fluid-qt/build is not configured" >&2
        exit 1
    fi
    echo "ui gate skipped: configure apps/punchi-fluid-qt to run it (PUNCHI_REQUIRE_UI=1 makes this fatal)" >&2
fi
