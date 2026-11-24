#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 PunchiSoft
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Tests scripts-dev/check-skills.sh against temporary trees. The lint exists because a
# malformed frontmatter makes a skill disappear from the model catalog without failing the
# build; each case here is one of the ways that happened (an unquoted ': ', a long
# description, a missing licence, a name that does not match the folder, a missing catalog
# row and a broken SKILL.md link).
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
CHECK="$ROOT_DIR/scripts-dev/check-skills.sh"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
fail=0

make_tree() {
    local root=$1
    mkdir -p "$root/.agents/skills/good"
    cat >"$root/.agents/skills/README.md" <<'MD'
| Skill | Purpose | Use when |
|---|---|---|
| [good](good/SKILL.md) | a fixture | testing the lint |
MD
    cat >"$root/.agents/skills/good/SKILL.md" <<'MD'
---
name: good
description: "A valid fixture. Use when testing the lint."
license: GPL-3.0-or-later
---

# Good
MD
}

expect_ok() {
    local name=$1 root=$2
    if ! ROOT="$root" "$CHECK" >"$tmp/out" 2>&1; then
        echo "check-skills fixture '$name' failed but should pass:" >&2
        cat "$tmp/out" >&2
        fail=1
    fi
}

expect_fail() {
    local name=$1 root=$2 pattern=$3
    if ROOT="$root" "$CHECK" >"$tmp/out" 2>&1; then
        echo "check-skills fixture '$name' passed but should fail" >&2
        fail=1
    elif ! grep -q -- "$pattern" "$tmp/out"; then
        echo "check-skills fixture '$name' failed without '$pattern':" >&2
        cat "$tmp/out" >&2
        fail=1
    fi
}

# The baseline must pass.
root="$tmp/baseline"
make_tree "$root"
expect_ok baseline "$root"

# An unquoted colon is the original failure.
root="$tmp/unquoted"
make_tree "$root"
sed -i 's/^description: .*/description: A valid fixture: with a colon. Use when testing./' "$root/.agents/skills/good/SKILL.md"
expect_fail unquoted "$root" "must be quoted"

# A description at or over 300 characters.
root="$tmp/long"
make_tree "$root"
long="$(printf 'A%.0s' {1..310})"
sed -i "s/^description: .*/description: \"$long Use when testing.\"/" "$root/.agents/skills/good/SKILL.md"
expect_fail long "$root" "300"

# A missing licence.
root="$tmp/license"
make_tree "$root"
sed -i '/^license:/d' "$root/.agents/skills/good/SKILL.md"
expect_fail license "$root" "license"

# A name that does not match the folder.
root="$tmp/name"
make_tree "$root"
sed -i 's/^name: good/name: other/' "$root/.agents/skills/good/SKILL.md"
expect_fail name "$root" "does not match"

# A folder without its catalog row.
root="$tmp/row"
make_tree "$root"
sed -i '/good\/SKILL\.md/d' "$root/.agents/skills/README.md"
expect_fail row "$root" "no row"

# A SKILL.md link that resolves to nothing.
root="$tmp/link"
make_tree "$root"
printf '\n[missing](../missing/SKILL.md)\n' >>"$root/.agents/skills/good/SKILL.md"
expect_fail link "$root" "broken skill link"

if ((fail != 0)); then
    exit 1
fi
echo "check-skills: the fixtures pass"
