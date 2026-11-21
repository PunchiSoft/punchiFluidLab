#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 PunchiSoft
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Tests scripts-dev/check-docs.sh against a temporary tree. The check is the only thing that
# turns a skill-authored link into a resolved path, so the fixture proves the three failures
# the extension exists for (a broken relative link, a missing catalog row and a wrong anchor)
# plus the path containment that keeps a link inside the tree.
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
CHECK="$ROOT_DIR/scripts-dev/check-docs.sh"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
fail=0

make_tree() {
    local root=$1
    mkdir -p "$root/.github/workflows" "$root/.github/ISSUE_TEMPLATE" \
        "$root/docs/adr" "$root/.agents/skills/capability-plan" \
        "$root/.agents/skills/rust-workspace-architecture" \
        "$root/crates/punchi-fluid-core"

    cat >"$root/Cargo.toml" <<'EOF'
[workspace]
members = [
    "crates/punchi-fluid-core",
]
EOF

    cat >"$root/AGENTS.md" <<'EOF'
# Agent rules

The workspace holds `crates/punchi-fluid-core`.
EOF
    cat >"$root/README.md" <<'EOF'
# Fixture

The crate is `punchi-fluid-core`.
EOF
    cat >"$root/README.es.md" <<'EOF'
# Fixture

El crate es `punchi-fluid-core`.
EOF
    cat >"$root/CHANGELOG.md" <<'EOF'
# Changelog
EOF
    for file in CONTRIBUTING.md SECURITY.md CODE_OF_CONDUCT.md SUPPORT.md LICENSE; do
        printf '# %s\n' "$file" >"$root/$file"
    done
    for file in .github/PULL_REQUEST_TEMPLATE.md .github/CODEOWNERS .github/FUNDING.yml \
        .github/dependabot.yml .github/workflows/ci.yml .github/workflows/audit.yml \
        .github/workflows/release.yml; do
        printf '# %s\n' "$file" >"$root/$file"
    done
    printf 'bug\n' >"$root/.github/labels.txt"
    printf 'blank_issues_enabled: false\n' >"$root/.github/ISSUE_TEMPLATE/config.yml"

    cat >"$root/docs/README.md" <<'EOF'
# Documentation

- [capabilities](capabilities.md)
- [ADRs](adr/README.md)
EOF
    cat >"$root/docs/capabilities.md" <<'EOF'
# Capabilities

The fixture has one capability.
EOF
    cat >"$root/docs/adr/README.md" <<'EOF'
# ADRs

No decisions yet.
EOF

    cat >"$root/.agents/skills/README.md" <<'EOF'
# Skills

| Skill | Purpose |
|---|---|
| [capability-plan](capability-plan/SKILL.md) | the fixture's first skill |
| [rust-workspace-architecture](rust-workspace-architecture/SKILL.md) | the fixture's second skill |
EOF
    cat >"$root/.agents/skills/capability-plan/SKILL.md" <<'EOF'
---
name: capability-plan
description: A fixture skill. Use when testing the documentation check.
license: GPL-3.0-or-later
---

# Capability plan

Read [capabilities](../../../docs/capabilities.md#capabilities).
EOF
    cat >"$root/.agents/skills/rust-workspace-architecture/SKILL.md" <<'EOF'
---
name: rust-workspace-architecture
description: A fixture skill. Use when testing the documentation check.
license: GPL-3.0-or-later
---

# Rust workspace architecture

The workspace holds `crates/punchi-fluid-core`.
EOF
}

# expect <tree> <pass|fail> <needle> <label>
expect() {
    local root=$1 expected=$2 needle=$3 label=$4
    local status out err
    out="$tmp/stdout"
    err="$tmp/stderr"
    if ROOT="$root" "$CHECK" >"$out" 2>"$err"; then
        status=0
    else
        status=$?
    fi
    if [[ $expected == pass ]]; then
        if ((status != 0)); then
            printf 'test-check-docs: %s should pass, exited %s\n' "$label" "$status" >&2
            sed 's/^/  /' "$err" >&2
            fail=1
            return
        fi
    else
        if ((status == 0)); then
            printf 'test-check-docs: %s should fail, exited 0\n' "$label" >&2
            fail=1
            return
        fi
        if ! grep -qF -- "$needle" "$err"; then
            printf 'test-check-docs: %s failed without the expected message %q\n' "$label" "$needle" >&2
            sed 's/^/  /' "$err" >&2
            fail=1
            return
        fi
    fi
    printf 'test-check-docs: %s ok\n' "$label"
}

# The real tree must pass: the fixture is only useful if the check still accepts the docs.
expect "$ROOT_DIR" pass "" "the real tree"

make_tree "$tmp/base"
expect "$tmp/base" pass "" "a valid fixture"

make_tree "$tmp/broken"
sed -i 's|\.\./\.\./\.\./docs/capabilities\.md#capabilities|../../../docs/not-there.md|' \
    "$tmp/broken/.agents/skills/capability-plan/SKILL.md"
expect "$tmp/broken" fail "broken link" "a broken relative link"

make_tree "$tmp/no-row"
sed -i '/capability-plan\/SKILL\.md/d' "$tmp/no-row/.agents/skills/README.md"
expect "$tmp/no-row" fail "no row in .agents/skills/README.md" "a missing catalog row"

make_tree "$tmp/wrong-anchor"
sed -i 's|#capabilities|#not-a-heading|' \
    "$tmp/wrong-anchor/.agents/skills/capability-plan/SKILL.md"
expect "$tmp/wrong-anchor" fail "unknown anchor" "a wrong anchor"

make_tree "$tmp/outside"
printf '# Outside\n' >"$tmp/outside.md"
sed -i 's|\.\./\.\./\.\./docs/capabilities\.md#capabilities|../../../../outside.md|' \
    "$tmp/outside/.agents/skills/capability-plan/SKILL.md"
expect "$tmp/outside" fail "outside the tree" "a link that escapes the tree"

make_tree "$tmp/absolute"
sed -i 's|\.\./\.\./\.\./docs/capabilities\.md#capabilities|/etc/passwd|' \
    "$tmp/absolute/.agents/skills/capability-plan/SKILL.md"
expect "$tmp/absolute" fail "is absolute" "an absolute link"

make_tree "$tmp/symlink"
printf '# Outside\n' >"$tmp/outside.md"
ln -s "$tmp/outside.md" "$tmp/symlink/docs/outside.md"
sed -i 's|\.\./\.\./\.\./docs/capabilities\.md#capabilities|../../../docs/outside.md|' \
    "$tmp/symlink/.agents/skills/capability-plan/SKILL.md"
expect "$tmp/symlink" fail "outside the tree" "a symlink that leaves the tree"

if ((fail)); then
    exit 1
fi
printf 'test-check-docs: ok\n'
