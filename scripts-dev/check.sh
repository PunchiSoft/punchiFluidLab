#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 PunchiSoft
# SPDX-License-Identifier: GPL-3.0-or-later
#
# The whole automated gate: formatting, lints, tests and a headless smoke test of
# the CLI. No window, no network.
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT_DIR"

cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

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
