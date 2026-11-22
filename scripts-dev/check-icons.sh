#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 PunchiSoft
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Validates the action-icon files and their coverage.
#
# The family is one SVG per drawing of the window's own table. The files must follow the
# box, stroke and ink ADR 0035 fixes, and the table and the files must name the same ids:
# a drawing without a file is a command that would fall back to the desktop theme, and a
# file without a drawing is an orphan nobody asks for.
#
# Usage: ./scripts-dev/check-icons.sh

set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT_DIR"

ICONS=assets/icons/actions
TABLE=apps/punchi-fluid-qt/src/actionicons.cpp
MAX_BYTES=4096
problems=0

fail() {
    printf 'check-icons: %s\n' "$*" >&2
    problems=$((problems + 1))
}

if [[ ! -d "$ICONS" ]]; then
    printf 'check-icons: %s is missing\n' "$ICONS" >&2
    exit 1
fi

for file in "$ICONS"/*.svg; do
    name=$(basename "$file" .svg)
    if [[ ! "$name" =~ ^[a-z0-9][a-z0-9.-]*$ ]]; then
        fail "$name: the id is not kebab-case"
    fi
    if [[ $(stat -c %s "$file") -gt $MAX_BYTES ]]; then
        fail "$name: the file is larger than $MAX_BYTES bytes"
    fi
    if ! grep -q 'viewBox="0 0 24 24"' "$file"; then
        fail "$name: the box is not 24 x 24"
    fi
    if ! grep -q 'stroke-width="2"' "$file"; then
        fail "$name: the stroke is not 2 units"
    fi
    if ! grep -q 'currentColor' "$file"; then
        fail "$name: the ink is not currentColor"
    fi
    if grep -qE '<text|<image|href=|<script|<style' "$file"; then
        fail "$name: a text, image, link, script or style element is forbidden"
    fi
done

if [[ ! -f "$TABLE" ]]; then
    printf 'check-icons: %s is missing\n' "$TABLE" >&2
    exit 1
fi

names=$(sed -n '/^const QStringList &allNames()/,/^}$/p' "$TABLE" \
    | grep -o 'QStringLiteral("[^"]*")' \
    | sed 's/QStringLiteral("//; s/")//' \
    | sort)
have=$(for file in "$ICONS"/*.svg; do basename "$file" .svg; done | sort)

while read -r id; do
    [[ -n "$id" ]] && fail "$id: the table draws it and there is no file"
done < <(comm -23 <(printf '%s\n' "$names") <(printf '%s\n' "$have"))

while read -r id; do
    [[ -n "$id" ]] && fail "$id: the file exists and the table does not draw it"
done < <(comm -13 <(printf '%s\n' "$names") <(printf '%s\n' "$have"))

count=$(printf '%s\n' "$have" | wc -l)
if ((problems > 0)); then
    printf 'check-icons: %d problem(s) in %d file(s)\n' "$problems" "$count" >&2
    exit 1
fi
printf 'check-icons: %d files, the table and the family agree\n' "$count"
