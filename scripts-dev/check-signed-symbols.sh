#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 PunchiSoft
# SPDX-License-Identifier: GPL-3.0-or-later
#
# The signed symbols are the ones the owner accepted, and this check is what makes that
# word mean something: the manifest pins the SHA-256 of the definition, the review and the
# three captures, and the owner's SSH signature over the manifest is what stops the pins
# from being rewritten. A symbol whose files no longer match its entry fails here, which is
# what "a signed symbol is immutable" costs.
#
# It reads the real tree by default. `ROOT` exists so the fixture driver can prove the
# check on a temporary tree; every resolved path is contained to it.
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
ROOT_DIR=$(cd -- "${ROOT:-$ROOT_DIR}" && pwd)
python3 - "$ROOT_DIR" <<'PY'
import hashlib
import re
import shutil
import subprocess
import sys
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:
    sys.exit("check-signed-symbols: Python 3.11 or newer is required (tomllib)")

root = Path(sys.argv[1]).resolve()
symbols = root / "assets" / "symbols"
manifest_path = symbols / "signed.toml"
signers_path = symbols / "owner_signers"
signature_path = symbols / "signed.toml.sig"
namespace = "punchi-fluid-signed-symbols"
fields = {
    "id", "revision", "previous_revision", "review", "toml", "captures",
    "sha256_toml", "sha256_review", "sha256_300", "sha256_light", "sha256_dark",
    "commit", "signed_by", "signed_at",
}
problems = []


def problem(message):
    problems.append(message)


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def review_status(path):
    match = re.search(r"^\s{4}status:\s*(\S+)\s*$", path.read_text(encoding="utf-8"), re.M)
    return match.group(1) if match else ""


def review_id(path):
    text = path.read_text(encoding="utf-8")
    match = re.search(r"^# (?:Review record: `([^`]+)`|(.+?) — conformance review)$", text, re.M)
    if not match:
        return ""
    return match.group(1) or match.group(2)


if not manifest_path.is_file():
    sys.exit("check-signed-symbols: assets/symbols/signed.toml is missing")
try:
    document = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
except Exception as error:  # tomllib.TOMLDecodeError, but the message is what matters
    sys.exit(f"check-signed-symbols: signed.toml is not valid TOML: {error}")

if document.get("schema") != 1:
    problem("signed.toml: schema must be 1")
if document.get("namespace") != namespace:
    problem(f"signed.toml: namespace must be {namespace!r}")
base_open = document.get("base_open")
if not isinstance(base_open, bool):
    problem("signed.toml: base_open must be true or false")

entries = document.get("symbol")
if entries is None:
    entries = []
if not isinstance(entries, list):
    sys.exit("check-signed-symbols: signed.toml: symbol must be an array of tables")

seen = {}
signed = []
for index, entry in enumerate(entries):
    label = f"signed.toml: symbol[{index}]"
    if not isinstance(entry, dict):
        problem(f"{label} is not a table")
        continue
    unknown = set(entry) - fields
    if unknown:
        problem(f"{label}: unknown keys {sorted(unknown)}")
    cid = entry.get("id")
    if not isinstance(cid, str) or not re.fullmatch(r"[a-z][a-z0-9_.]*", cid):
        problem(f"{label}: id is missing or not a component id")
        continue
    label = f"signed.toml: {cid}"

    if cid in seen:
        problem(f"{label}: duplicate id, first seen in symbol[{seen[cid]}]")
    else:
        seen[cid] = index

    revision = entry.get("revision")
    previous = entry.get("previous_revision")
    if not isinstance(revision, int) or isinstance(revision, bool) or revision < 1:
        problem(f"{label}: revision must be an integer >= 1")
    elif not isinstance(previous, int) or isinstance(previous, bool) or previous != revision - 1:
        problem(f"{label}: previous_revision must be revision - 1 ({revision - 1}), got {previous!r}")

    expected_review = f"assets/symbols/review/{cid}.md"
    expected_toml = f"assets/catalog/{cid}.toml"
    expected_captures = [
        f"assets/symbols/review/{cid}-300.png",
        f"assets/symbols/review/{cid}-light.png",
        f"assets/symbols/review/{cid}-dark.png",
    ]
    if entry.get("review") != expected_review:
        problem(f"{label}: review must be {expected_review!r}")
    if entry.get("toml") != expected_toml:
        problem(f"{label}: toml must be {expected_toml!r}")
    if entry.get("captures") != expected_captures:
        problem(f"{label}: captures must be {expected_captures!r}")

    linked = [(entry.get("review"), "review"), (entry.get("toml"), "toml")]
    linked += list(zip(entry.get("captures") or [], ["captures[0]", "captures[1]", "captures[2]"]))
    for relative, field in linked:
        if not isinstance(relative, str):
            continue
        candidate = (root / relative).resolve()
        if not candidate.is_relative_to(root):
            problem(f"{label}: {field} escapes the repository: {relative}")
        elif not candidate.is_file():
            problem(f"{label}: {field} does not exist: {relative}")

    catalog = root / expected_toml
    review = root / expected_review
    if catalog.is_file():
        if entry.get("sha256_toml") != sha256(catalog):
            problem(f"{label}: sha256_toml does not match {expected_toml}")
        declared = re.search(r'^id\s*=\s*"([^"]+)"', catalog.read_text(encoding="utf-8"), re.M)
        if not declared or declared.group(1) != cid:
            problem(f"{label}: {expected_toml} declares a different id")
    if review.is_file():
        if entry.get("sha256_review") != sha256(review):
            problem(f"{label}: sha256_review does not match {expected_review}")
        if review_status(review) != "complete":
            problem(f"{label}: the review is not `complete`; the entry must be removed or the review restored")
        if review_id(review) != cid:
            problem(f"{label}: the review names a different id")
    for field, name in (("sha256_300", expected_captures[0]), ("sha256_light", expected_captures[1]), ("sha256_dark", expected_captures[2])):
        capture = root / name
        if capture.is_file() and entry.get(field) != sha256(capture):
            problem(f"{label}: {field} does not match {name}")

    commit = entry.get("commit")
    if commit is not None and (not isinstance(commit, str) or not re.fullmatch(r"[0-9a-f]{40}", commit)):
        problem(f"{label}: commit must be a full 40-character lowercase SHA, or absent")

    has_by = "signed_by" in entry
    has_at = "signed_at" in entry
    if has_by != has_at:
        problem(f"{label}: signed_by and signed_at are both present or both absent")
    if has_by:
        by = entry.get("signed_by")
        at = entry.get("signed_at")
        if not isinstance(by, str) or not by.strip():
            problem(f"{label}: signed_by must be a non-empty principal")
        if not isinstance(at, str) or not re.fullmatch(r"\d{4}-\d{2}-\d{2}", at):
            problem(f"{label}: signed_at must be a date, YYYY-MM-DD")
        signed.append(entry)

pending = [entry for entry in entries if entry not in signed]

complete_reviews = []
review_dir = symbols / "review"
if review_dir.is_dir():
    for path in sorted(review_dir.glob("*.md")):
        if path.name == "TEMPLATE.md":
            continue
        if review_status(path) == "complete":
            complete_reviews.append(path.stem)
for cid in complete_reviews:
    if cid not in seen:
        problem(f"assets/symbols/review/{cid}.md is `complete` but has no entry in signed.toml")

if signed:
    principals = {entry.get("signed_by") for entry in signed}
    if len(principals) != 1:
        problem("signed.toml: signed entries name more than one principal; the file carries one signature")
    if not signature_path.is_file():
        problem("signed.toml.sig is missing while entries are signed")
    keys = []
    if signers_path.is_file():
        keys = [
            line for line in signers_path.read_text(encoding="utf-8").splitlines()
            if line.strip() and not line.lstrip().startswith("#")
        ]
    if not keys:
        problem("assets/symbols/owner_signers holds no public key")
    if not shutil.which("ssh-keygen"):
        problem("ssh-keygen is not installed; the signature cannot be verified")
    if not problems:
        principal = sorted(principals)[0]
        verified = subprocess.run(
            [
                "ssh-keygen", "-Y", "verify",
                "-f", str(signers_path),
                "-I", principal,
                "-n", namespace,
                "-s", str(signature_path),
            ],
            input=manifest_path.read_bytes(),
            capture_output=True,
        )
        if verified.returncode != 0:
            detail = (verified.stderr or verified.stdout).decode(errors="replace").strip()
            problem(f"the signature does not verify for {principal!r}: {detail or 'ssh-keygen failed'}")

if not base_open and pending:
    problem(f"signed.toml: base_open is false but {len(pending)} entries are still pending")

if problems:
    print("check-signed-symbols: signed symbol problems:", file=sys.stderr)
    for item in problems:
        print(f"  {item}", file=sys.stderr)
    sys.exit(1)

state = "open" if base_open else "closed"
print(
    f"check-signed-symbols: {len(signed)} signed, {len(pending)} pending, "
    f"{len(complete_reviews)} complete reviews, base {state}"
)
PY
