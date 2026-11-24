#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 PunchiSoft
# SPDX-License-Identifier: GPL-3.0-or-later
#
# The skill catalog is data the model loads at runtime: a malformed frontmatter makes the
# skill disappear without failing a build. This check parses every SKILL.md, matches it with
# its catalog row and resolves every SKILL.md link, so the failure is loud here.
#
# It uses only the Python standard library: the CI image has no YAML package, and the
# frontmatter subset this repository writes is three scalar fields.
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
ROOT_DIR=$(cd -- "${ROOT:-$ROOT_DIR}" && pwd)
python3 - "$ROOT_DIR" <<'PY'
import json
import re
import sys
from pathlib import Path

root = Path(sys.argv[1]).resolve()
skills_dir = root / ".agents" / "skills"
problems = []

if not skills_dir.is_dir():
    print("check-skills: no .agents/skills in the tree; skipped")
    sys.exit(0)

readme = skills_dir / "README.md"
rows = []
if not readme.is_file():
    problems.append(".agents/skills/README.md is missing")
else:
    for line in readme.read_text(encoding="utf-8").splitlines():
        match = re.match(r"^\|\s*\[([a-z0-9-]+)\]\(([a-z0-9-]+)/SKILL\.md\)", line)
        if match:
            rows.append((match.group(1), match.group(2)))

names = {}
folders = sorted(path for path in skills_dir.iterdir() if path.is_dir())
for folder in folders:
    skill = folder / "SKILL.md"
    if not skill.is_file():
        problems.append(f"{folder.relative_to(root)}: no SKILL.md")
        continue

    text = skill.read_text(encoding="utf-8")
    front = re.match(r"^---\s*\n(.*?)\n---\s*\n", text, re.S)
    if not front:
        problems.append(f"{skill.relative_to(root)}: no frontmatter")
        continue
    block = front.group(1)

    name_match = re.search(r"^name:\s*(\S+)\s*$", block, re.M)
    description_match = re.search(r"^description:\s*(.*)$", block, re.M)
    license_match = re.search(r"^license:\s*(.+?)\s*$", block, re.M)

    name = name_match.group(1) if name_match else ""
    if not re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*", name):
        problems.append(f"{skill.relative_to(root)}: name is missing or not kebab-case")
    elif name != folder.name:
        problems.append(f"{skill.relative_to(root)}: name '{name}' does not match the folder")
    elif name in names:
        problems.append(f"{skill.relative_to(root)}: duplicate skill name '{name}'")
    else:
        names[name] = folder.name

    if not license_match or license_match.group(1) != "GPL-3.0-or-later":
        problems.append(f"{skill.relative_to(root)}: license is not GPL-3.0-or-later")

    if not description_match:
        problems.append(f"{skill.relative_to(root)}: description is missing")
    else:
        raw = description_match.group(1).strip()
        if len(raw) < 2 or raw[0] != raw[-1] or raw[0] not in ('"', "'"):
            problems.append(
                f"{skill.relative_to(root)}: description must be quoted; an unquoted ': ' breaks the YAML"
            )
        else:
            try:
                description = json.loads(raw) if raw[0] == '"' else raw[1:-1].replace("''", "'")
            except Exception:
                problems.append(f"{skill.relative_to(root)}: description is not a valid quoted string")
                description = ""
            if len(description) >= 300:
                problems.append(
                    f"{skill.relative_to(root)}: description is {len(description)} characters, the limit is 300"
                )
            if "Use when" not in description:
                problems.append(f"{skill.relative_to(root)}: description has no 'Use when' clause")

row_names = [row[0] for row in rows]
for name, path in rows:
    if name != path:
        problems.append(f".agents/skills/README.md: row '{name}' links to '{path}/SKILL.md'")
    if row_names.count(name) > 1:
        problems.append(f".agents/skills/README.md: duplicate row for '{name}'")
    if name not in names:
        problems.append(f".agents/skills/README.md: row '{name}' has no folder")
for name in names:
    if name not in row_names:
        problems.append(f".agents/skills/README.md: no row for skill '{name}'")

for markdown in sorted(skills_dir.rglob("*.md")):
    fence = None
    for line in markdown.read_text(encoding="utf-8").splitlines():
        stripped = line.lstrip()
        if fence is not None:
            if stripped.startswith(fence):
                fence = None
            continue
        if stripped.startswith(("```", "~~~")):
            fence = stripped[:3]
            continue
        line = re.sub(r"`[^`]*`", "", line)
        for match in re.finditer(r"!?\[[^\]]*\]\(([^)\n]+)\)", line):
            target = match.group(1).strip()
            if target.startswith("<") and target.endswith(">"):
                target = target[1:-1]
            if target.startswith(("http://", "https://", "mailto:", "#", "/")):
                continue
            path = target.split("#", 1)[0]
            if not path:
                continue
            candidate = (markdown.parent / path).resolve()
            if not candidate.exists():
                problems.append(
                    f"{markdown.relative_to(root)}: broken skill link {target}"
                )

if problems:
    print("check-skills: skill catalog problems:", file=sys.stderr)
    for problem in problems:
        print(f"  {problem}", file=sys.stderr)
    sys.exit(1)

print(f"check-skills: {len(names)} skills and {len(rows)} catalog rows are coherent")
PY
