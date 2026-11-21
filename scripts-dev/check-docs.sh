#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 PunchiSoft
# SPDX-License-Identifier: GPL-3.0-or-later
#
# The workspace layout is declared in Cargo.toml and described in two documents that
# people and agents read instead of the manifest. Four retired crate names survived a
# whole toolkit change because nothing compared the two, so this compares them.
set -euo pipefail

# The real tree is the default. `ROOT` exists so the fixture driver can prove the
# documentation checks on a temporary tree without touching this one; every resolve is
# bounded to it.
DEFAULT_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
ROOT_DIR=$(cd -- "${ROOT:-$DEFAULT_ROOT}" && pwd)
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
    if hits=$(git -C "$ROOT_DIR" grep -l -i --fixed-strings -- "$term" -- . ':!scripts-dev/check-docs.sh' 2>/dev/null); then
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

# The documentation architecture: an index that reaches every page, links that resolve,
# and ADRs with a number, a status, a date and a line in their own index. A page nobody
# links to is a page nobody reads, and a link to a file that does not exist is a promise
# the repository does not keep.
if ! python3 - <<'PY'
from pathlib import Path
from urllib.parse import unquote
import os
import re
import sys

root = Path(".").resolve()
docs = sorted(Path("docs").rglob("*.md"))
index = Path("docs/README.md").read_text(encoding="utf-8")
adr_index = Path("docs/adr/README.md").read_text(encoding="utf-8")
problems = []


def anchors(path):
    """The anchors GitHub derives from a document's ATX headings."""
    found = set()
    seen = {}
    fence = None
    for line in path.read_text(encoding="utf-8").splitlines():
        stripped = line.lstrip()
        if fence is not None:
            if stripped.startswith(fence):
                fence = None
            continue
        if stripped.startswith(("```", "~~~")):
            fence = stripped[:3]
            continue
        match = re.match(r"^ {0,3}(#{1,6})\s+(.*?)\s*#*\s*$", line)
        if not match:
            continue
        text = re.sub(r"!\[([^\]]*)\]\([^)]*\)", r"\1", match.group(2))
        text = re.sub(r"\[([^\]]*)\]\([^)]*\)", r"\1", text)
        text = re.sub(r"<[^>]+>", "", text).replace("`", "")
        text = re.sub(r"[*_~]", "", text).strip().lower()
        slug = "".join(ch for ch in text if ch.isalnum() or ch in " -_").replace(" ", "-")
        if slug in seen:
            seen[slug] += 1
            found.add(f"{slug}-{seen[slug]}")
        else:
            seen[slug] = 0
            found.add(slug)
    return found


def check_links(file):
    text = file.read_text(encoding="utf-8")
    fence = None
    for line in text.splitlines():
        stripped = line.lstrip()
        if fence is not None:
            if stripped.startswith(fence):
                fence = None
            continue
        if stripped.startswith(("```", "~~~")):
            fence = stripped[:3]
            continue
        # A link inside an inline code span is an example, not a link.
        line = re.sub(r"`[^`]*`", "", line)
        for match in re.finditer(r"!?\[[^\]]*\]\(([^)\n]+)\)", line):
            raw = match.group(1).strip()
            if not raw:
                continue
            target = raw.split()[0]
            if target.startswith("<") and target.endswith(">"):
                target = target[1:-1]
            if target.startswith(("http://", "https://", "mailto:", "//")):
                continue
            path, separator, fragment = target.partition("#")
            path = unquote(path)
            fragment = unquote(fragment)
            if re.match(r"^[A-Za-z]:[\\/]", path) or path.startswith(("/", "\\")):
                problems.append(f"{file}: link {target} is absolute; links stay inside the tree")
                continue
            candidate = (file.parent / path).resolve() if path else file.resolve()
            try:
                candidate.relative_to(root)
            except ValueError:
                problems.append(f"{file}: link {target} points outside the tree")
                continue
            if not candidate.exists():
                problems.append(f"{file}: broken link {target}")
                continue
            if separator and fragment and candidate.is_file() and candidate.suffix.lower() == ".md":
                if fragment not in anchors(candidate):
                    problems.append(f"{file}: link {target} has an unknown anchor #{fragment}")


for doc in docs:
    if doc == Path("docs/README.md"):
        continue
    if doc.parent == Path("docs/adr") and doc.name[:4].isdigit():
        target = adr_index
        label = "docs/adr/README.md"
        relative = doc.name
    else:
        target = index
        label = "docs/README.md"
        relative = doc.relative_to("docs").as_posix()
    if f"]({relative})" not in target:
        problems.append(f"{doc} is not linked from {label}")

files = [Path("README.md"), Path("README.es.md"), Path("AGENTS.md"), *docs]
skills_dir = Path(".agents/skills")
skills_readme = skills_dir / "README.md"
skill_files = sorted(skills_dir.rglob("*.md"))

# A skill that no catalog row reaches is invisible, and a frontmatter name that differs from
# its folder breaks every dispatch by name.
if not skills_readme.is_file():
    problems.append(".agents/skills/README.md is missing")
else:
    catalog = skills_readme.read_text(encoding="utf-8")
    for folder in sorted(path for path in skills_dir.iterdir() if path.is_dir()):
        skill = folder / "SKILL.md"
        if not skill.is_file():
            problems.append(f"{folder}: no SKILL.md")
            continue
        if f"]({folder.name}/SKILL.md)" not in catalog:
            problems.append(f"{folder}: no row in .agents/skills/README.md")
        frontmatter = re.match(r"^---\s*\n(.*?)\n---\s*\n", skill.read_text(encoding="utf-8"), re.S)
        name = re.search(r"^name:\s*(\S+)\s*$", frontmatter.group(1), re.M) if frontmatter else None
        if name is None:
            problems.append(f"{skill}: no name in the frontmatter")
        elif name.group(1) != folder.name:
            problems.append(f"{skill}: name '{name.group(1)}' does not match the folder '{folder.name}'")

for file in [*files, *skill_files]:
    check_links(file)

numbers = []
for adr in sorted(Path("docs/adr").glob("*.md")):
    if adr.name in {"README.md", "template.md"}:
        continue
    match = re.match(r"^(\d{4})-", adr.name)
    if not match:
        problems.append(f"{adr}: name is not NNNN-kebab-case.md")
        continue
    numbers.append(int(match.group(1)))
    head = "\n".join(adr.read_text(encoding="utf-8").splitlines()[:12])
    if "Status" not in head:
        problems.append(f"{adr}: no Status in the first lines")
    if "Date" not in head:
        problems.append(f"{adr}: no Date in the first lines")

expected = list(range(1, len(numbers) + 1))
if numbers != expected:
    problems.append(f"ADR numbers are {numbers}, expected {expected}")

for file in files:
    first = next((line for line in file.read_text(encoding="utf-8").splitlines() if line.strip()), "")
    if not first.startswith("# "):
        problems.append(f"{file}: first non-empty line is not an H1")

if problems:
    print("check-docs: documentation structure problems:", file=sys.stderr)
    for problem in problems:
        print(f"  {problem}", file=sys.stderr)
    sys.exit(1)
PY
then
    status=1
fi

if ((status == 0)); then
    echo "check-docs: the workspace documents match Cargo.toml"
    echo "check-docs: the documentation index, links and ADRs are complete"
fi
exit "$status"
