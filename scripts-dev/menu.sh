#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 PunchiSoft
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Interactive development menu, in the spirit of the Quickshell assistant: it shows the
# state of the checkout, checks the tools, and runs the common tasks without remembering the
# exact script name. It is a developer tool, not a user installer; packaging is a later
# phase and has its own entry in docs/capabilities.md.
#
# Usage:
#   ./scripts-dev/menu.sh                 # interactive menu
#   ./scripts-dev/menu.sh --check-deps    # report tools and exit
#   ./scripts-dev/menu.sh --build         # one action, then exit
#   ./scripts-dev/menu.sh --run | --try | --dev | --test | --gate | --docs
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT_DIR"
# shellcheck source=limits.sh
source "$ROOT_DIR/scripts-dev/limits.sh"

binary=target/debug/punchi-fluid
no_color=0
action=""

if [[ -t 1 ]]; then
    bold=$'\033[1m'; dim=$'\033[2m'; green=$'\033[32m'; yellow=$'\033[33m'; red=$'\033[31m'; reset=$'\033[0m'
else
    bold=""; dim=""; green=""; yellow=""; red=""; reset=""
fi

say() { printf '%s\n' "$*"; }
header() { printf '%s\n' "${bold}$*${reset}"; }
ok() { printf '%s%s%s\n' "$green" "$*" "$reset"; }
warn() { printf '%s%s%s\n' "$yellow" "$*" "$reset"; }
bad() { printf '%s%s%s\n' "$red" "$*" "$reset"; }

command -v cargo >/dev/null 2>&1 || { bad "cargo is missing"; exit 1; }

project_version() {
    sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1
}

branch() {
    git rev-parse --abbrev-ref HEAD 2>/dev/null || printf 'not a git checkout'
}

binary_state() {
    if [[ -x "$binary" ]]; then
        printf 'built %s' "$(date -r "$binary" '+%Y-%m-%d %H:%M')"
    else
        printf 'not built'
    fi
}

check_deps() {
    local missing=0
    local tool
    for tool in cargo rustc git; do
        if command -v "$tool" >/dev/null 2>&1; then
            ok "  $tool  $(command -v "$tool")"
        else
            bad "  $tool  MISSING"
            missing=1
        fi
    done
    if cargo fmt --version >/dev/null 2>&1; then
        ok "  rustfmt $(cargo fmt --version | awk '{print $2}')"
    else
        warn "  rustfmt missing (cargo fmt --check will fail)"
    fi
    if cargo clippy --version >/dev/null 2>&1; then
        ok "  clippy  $(cargo clippy --version | awk '{print $2}')"
    else
        warn "  clippy missing (the gate will fail)"
    fi
    return "$missing"
}

show_header() {
    printf '\n%s\n' "${bold}Punchi Fluid Lab — development menu${reset}"
    printf '%s\n' "${dim}version $(project_version) · branch $(branch) · $(binary_state)${reset}"
    printf '%s\n\n' "${dim}$(uname -srm) · $(rustc --version 2>/dev/null || printf 'rustc ?')${reset}"
}

build_window() {
    header "Build the window"
    cargo build -p punchi-fluid-window --bin punchi-fluid
}

run_window() {
    header "Run the window"
    ./scripts-dev/run.sh --no-build
}

build_and_run() {
    header "Build and run the window"
    cargo build -p punchi-fluid-window --bin punchi-fluid
    ./scripts-dev/run.sh --no-build
}

run_dev() {
    header "Live loop: save, rebuild and relaunch"
    ./scripts-dev/dev.sh
}

clean_build() {
    header "Remove the window build"
    warn "This deletes the window crate's artifacts under target/ (including $binary); cached dependencies stay, and the next build recompiles the window."
    read -r -p "Type yes to remove it: " answer
    if [[ "$answer" == "yes" ]]; then
        cargo clean -p punchi-fluid-window
        ok "removed the window build"
    else
        say "kept"
    fi
}

cli_check() {
    header "Check a sheet with the CLI"
    local path=""
    read -r -p "Path to a .pfluid sheet (empty to cancel): " path
    if [[ -n "$path" ]]; then
        cargo run -q -p punchi-fluid-cli -- check "$path"
    fi
}

pause() {
    [[ -t 0 ]] && read -r -p "Enter to return to the menu… " _
}

menu() {
    while true; do
        show_header
        cat <<'EOF'
  1) Live loop: save, rebuild and relaunch  <- the one to keep open while editing
  2) Build and run the window once
  3) Check dependencies
  4) Build the window
  5) Run the window
  6) Tests (scripts-dev/test.sh)
  7) Full gate (scripts-dev/check.sh)
  8) Documentation check
  9) Check a sheet with the CLI
 10) Remove the window build
  q) Quit
EOF
        read -r -p "Choice [1]: " choice
        choice=${choice:-1}
        case "$choice" in
            1) run_dev ;;
            2) build_and_run ;;
            3) check_deps || true; pause ;;
            4) build_window; pause ;;
            5) run_window ;;
            6) ./scripts-dev/test.sh; pause ;;
            7) ./scripts-dev/check.sh; pause ;;
            8) ./scripts-dev/check-docs.sh; pause ;;
            9) cli_check; pause ;;
           10) clean_build ;;
            q|Q) exit 0 ;;
            *) warn "No such choice: $choice" ;;
        esac
    done
}

while (($# > 0)); do
    case "$1" in
        --check-deps) action=deps ;;
        --build) action=build ;;
        --run) action=run ;;
        --try) action=try ;;
        --dev) action=dev ;;
        --test) action=test ;;
        --gate) action=gate ;;
        --docs) action=docs ;;
        --clean) action=clean ;;
        --no-color) no_color=1 ;;
        --help | -h)
            awk 'NR > 4 && /^set -euo/{exit} NR > 4{print}' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *) bad "Unknown option: $1"; exit 2 ;;
    esac
    shift
done

if ((no_color)); then
    bold=""; dim=""; green=""; yellow=""; red=""; reset=""
fi

case "$action" in
    deps) check_deps ;;
    build) build_window ;;
    run) run_window ;;
    try) build_and_run ;;
    dev) run_dev ;;
    test) ./scripts-dev/test.sh ;;
    gate) ./scripts-dev/check.sh ;;
    docs) ./scripts-dev/check-docs.sh ;;
    clean) clean_build ;;
    "")
        if [[ -t 0 ]]; then
            menu
        else
            bad "No terminal for the interactive menu; pass --check-deps, --build, --run, --try, --dev, --test, --gate or --docs."
            exit 2
        fi
        ;;
esac
