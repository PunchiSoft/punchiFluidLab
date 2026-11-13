#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 PunchiSoft
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Entry point for a person opening the checkout: it opens the development menu.
#
# This is not a system installer. The project is unreleased and has no package yet, so
# `setup` means "check the tools, build, run and try the live loop", not "install into
# /usr". Packaging is a later phase and has its own entry in docs/capabilities.md.
#
# Usage:
#   ./setup.sh              # open the interactive menu
#   ./setup.sh --try        # go straight to the live rebuild loop
#   ./setup.sh --build      # build the window and exit
#   ./setup.sh --help       # the menu's options
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
exec "$ROOT_DIR/scripts-dev/menu.sh" "$@"
