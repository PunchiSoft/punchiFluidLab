#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 PunchiSoft
# SPDX-License-Identifier: GPL-3.0-or-later
#
# The workspace layout is declared in Cargo.toml and described in two documents that
# people and agents read instead of the manifest. Four retired crate names survived a
# whole toolkit change because nothing compared the two, so this compares them.
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT_DIR"

DOCS=(
    "AGENTS.md"
    "README.md"
    "README.es.md"
    ".agents/skills/rust-workspace-architecture/SKILL.md"
)
RETIRED=(
    "punchi-fluid-theme"
    "punchi-fluid-ui"
    "punchi-fluid-desktop"
    "punchi-fluid-icons"
)

members=$(sed -n '/^members = \[/,/^\]/p' Cargo.toml | grep -oE '"[^"]+"' | tr -d '"')
if [[ -z $members ]]; then
    echo "check-docs: no workspace members found in Cargo.toml" >&2
    exit 1
fi

# The identity of the project is its mission and the standards (ADR 0009). These terms
# brought a third-party product, its manual and its file inventory into a public
# repository once; the check is here so it cannot come back quietly.
FORBIDDEN=(
    "FluidSIM"
    "Festo"
    "FLUID SIM NEUMATICO"
    "fl_sim_p.pdf"
    "PUNCHI_REFERENCE_DIR:-"
)

status=0

for term in "${FORBIDDEN[@]}"; do
    if hits=$(git grep -l -i --fixed-strings -- "$term" -- . ':!scripts-dev/check-docs.sh'); then
        echo "check-docs: '$term' still appears in:" >&2
        echo "$hits" | sed 's/^/  /' >&2
        status=1
    fi
done

for f in docs/capabilities.md .agents/skills/capability-plan/SKILL.md; do
    if [[ ! -f $f ]]; then
        echo "check-docs: $f is missing" >&2
        status=1
    fi
done

# The community files of docs/engineering.md, "Files that must exist". GitHub reads them
# from a fixed path, so a rename is a silent breakage: this makes it loud.
COMMUNITY=(
    "README.md"
    "README.es.md"
    "CONTRIBUTING.md"
    "SECURITY.md"
    "CODE_OF_CONDUCT.md"
    "SUPPORT.md"
    "LICENSE"
    "CHANGELOG.md"
    "AGENTS.md"
    ".github/PULL_REQUEST_TEMPLATE.md"
    ".github/CODEOWNERS"
    ".github/FUNDING.yml"
    ".github/dependabot.yml"
    ".github/workflows/ci.yml"
    ".github/workflows/audit.yml"
    ".github/workflows/release.yml"
)
for f in "${COMMUNITY[@]}"; do
    if [[ ! -f $f ]]; then
        echo "check-docs: the community file $f is missing" >&2
        status=1
    fi
done

# A template that names a label the repository does not have creates an unlabelled issue
# and a silent triage cost. The known labels are listed once, in the label manifest.
LABELS_FILE=".github/labels.txt"
if [[ ! -f $LABELS_FILE ]]; then
    echo "check-docs: $LABELS_FILE is missing" >&2
    status=1
else
    while IFS= read -r label; do
        [[ -z $label ]] && continue
        if ! grep -qxF -- "$label" "$LABELS_FILE"; then
            echo "check-docs: $LABELS_FILE does not declare the label '$label'" >&2
            status=1
        fi
    done < <(sed -n 's/^labels: \[\(.*\)\]$/\1/p' .github/ISSUE_TEMPLATE/*.yml |
        tr ',' '\n' | tr -d ' "')
fi

# A contact link to something the repository does not have is a dead end. Discussions is
# a repository setting, so this needs the network: it runs in CI, and warns locally.
if grep -q '/discussions' .github/ISSUE_TEMPLATE/config.yml; then
    if [[ -n ${GH_TOKEN:-}${GITHUB_TOKEN:-} ]] && command -v gh >/dev/null; then
        enabled=$(gh api "repos/${PUNCHI_REPO:-PunchiSoft/punchiFluidLab}" --jq .has_discussions 2>/dev/null || echo unknown)
        case $enabled in
            false)
                echo "check-docs: config.yml links to Discussions, which this repository has disabled" >&2
                status=1
                ;;
            unknown)
                echo "check-docs: could not read has_discussions; skipped" >&2
                ;;
        esac
    else
        echo "check-docs: Discussions link not verified (no GH_TOKEN, or no gh)" >&2
    fi
fi

for doc in "${DOCS[@]}"; do
    if [[ ! -f $doc ]]; then
        echo "check-docs: $doc is missing" >&2
        status=1
        continue
    fi
    while read -r member; do
        name=$(basename -- "$member")
        if ! grep -qF -- "$name" "$doc"; then
            echo "check-docs: $doc does not list the crate $name" >&2
            status=1
        fi
    done <<<"$members"
    for retired in "${RETIRED[@]}"; do
        if grep -qF -- "$retired" "$doc"; then
            echo "check-docs: $doc still names $retired, retired by ADR 0005" >&2
            status=1
        fi
    done
done

if ((status == 0)); then
    echo "check-docs: the workspace documents match Cargo.toml"
fi
exit "$status"
