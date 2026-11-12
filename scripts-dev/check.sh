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
# `cargo` prevents cycles, not layering: the graph, the one home of `unsafe` and the file
# budget are enforced here.
./scripts-dev/check-architecture.sh
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

cargo run --quiet --bin punchi-fluid-cli -- validate "$example"
cargo run --quiet --bin punchi-fluid-cli -- simulate "$example"

# The check reads the sheet the editor saves and speaks a versioned contract (ADR 0011):
# a clean sheet has to pass in strict mode, and a faulty one has to fail it.
sheet=$(mktemp --suffix=.pfluid)
faulty=$(mktemp --suffix=.pfluid)
trap 'rm -f "$example" "$sheet" "$faulty"' EXIT
cat >"$sheet" <<'JSON'
{
  "format": 3,
  "title": "Hoja limpia",
  "placement": [
    { "id": 1, "catalog": "supply.compressor", "x": 34.0, "y": 39.0, "width": 33.8, "height": 33.8 },
    { "id": 2, "catalog": "valve.3_2.spring_return", "x": 94.0, "y": 93.6, "width": 33.8, "height": 33.8 },
    { "id": 3, "catalog": "cylinder.single_acting_20_100", "x": 144.0, "y": 33.8, "width": 74.4, "height": 33.8 }
  ],
  "pipe": [
    { "from": 1, "from_port": "1", "to": 2, "to_port": "1" },
    { "from": 2, "from_port": "2", "to": 3, "to_port": "1" }
  ]
}
JSON
cat >"$faulty" <<'JSON'
{
  "format": 3,
  "title": "Fuera de la hoja",
  "placement": [
    { "id": 1, "catalog": "cylinder.single_acting_20_100", "x": 250.0, "y": 60.0, "width": 74.4, "height": 33.8 }
  ],
  "pipe": []
}
JSON
cargo run --quiet --bin punchi-fluid-cli -- check "$sheet" --strict
cargo run --quiet --bin punchi-fluid-cli -- check "$faulty" --json --strict && {
    echo "check: a sheet with faults passed in strict mode" >&2
    exit 1
} || test $? -eq 3

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

    # The export prints the sheet the session holds, so it is proved on a real file: a PDF that
    # starts with the header and carries the paper it was asked for, and a PNG with its own.
    export_dir="$(mktemp -d)"
    if ! "$app" --export "$export_dir/sheet.pdf" >/dev/null 2>&1; then
        echo "ui gate: the sheet did not export as a PDF" >&2
        exit 1
    fi
    head -c 5 "$export_dir/sheet.pdf" | grep -q '%PDF-' || {
        echo "ui gate: the exported PDF has no header" >&2
        exit 1
    }
    if ! "$app" --export "$export_dir/sheet-oficio.png" --paper 8 --landscape >/dev/null 2>&1; then
        echo "ui gate: the sheet did not export as a raster on Oficio" >&2
        exit 1
    fi
    head -c 8 "$export_dir/sheet-oficio.png" | od -An -c | grep -q '211   P   N   G' || {
        echo "ui gate: the exported PNG has no header" >&2
        exit 1
    }
    rm -rf "$export_dir"
else
    if [[ ${PUNCHI_REQUIRE_UI:-0} == 1 ]]; then
        echo "ui gate: the window is required but apps/punchi-fluid-qt/build is not configured" >&2
        exit 1
    fi
    echo "ui gate skipped: configure apps/punchi-fluid-qt to run it (PUNCHI_REQUIRE_UI=1 makes this fatal)" >&2
fi
