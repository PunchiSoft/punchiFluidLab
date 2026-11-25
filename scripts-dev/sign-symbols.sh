#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 PunchiSoft
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Sign the approved symbols of one batch in `assets/symbols/signed.toml`.
#
# The owner runs this script with their private SSH key: the key never enters the
# repository or a session. The script pins the hashes of the batch's definition, review and
# the three captures, writes the batch's `signed_by`/`signed_at`, and signs the **whole**
# manifest with the owner's key. The other entries are kept: a pending entry stays pending
# (allowed while `base_open = true`) and a signed entry keeps its principal and date.
#
# Usage:
#   ./scripts-dev/sign-symbols.sh --batch assets/symbols/signing-batches/2026-11-25.toml \
#       --key ~/.ssh/id_ed25519
#   ./scripts-dev/sign-symbols.sh --batch ... --dry-run
#
# The batch file is TOML:
#   name = "2026-11-25"
#   symbols = ["cylinder.double_acting_32_200", ...]
#
# The date is UTC; `signed_at` is a date, as the checker requires. `--close-base` is for
# the last batch: it refuses to run while any entry is still pending and sets
# `base_open = false`.
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
MANIFEST="$ROOT_DIR/assets/symbols/signed.toml"
SIGNERS="$ROOT_DIR/assets/symbols/owner_signers"
SIGNATURE="$MANIFEST.sig"
NAMESPACE="punchi-fluid-signed-symbols"

BATCH=""
KEY=""
SIGNER=""
DRY_RUN=0
CLOSE_BASE=0

usage() {
    sed -n '7,31p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
}

while [ $# -gt 0 ]; do
    case "$1" in
        --batch)
            BATCH=${2:-}
            shift 2
            ;;
        --key)
            KEY=${2:-}
            shift 2
            ;;
        --signer)
            SIGNER=${2:-}
            shift 2
            ;;
        --dry-run)
            DRY_RUN=1
            shift
            ;;
        --close-base)
            CLOSE_BASE=1
            shift
            ;;
        --help | -h)
            usage
            exit 0
            ;;
        *)
            echo "sign-symbols: unknown argument: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

if [ -z "$BATCH" ]; then
    echo "sign-symbols: --batch is required" >&2
    exit 2
fi
if [ ! -f "$BATCH" ]; then
    echo "sign-symbols: batch file not found: $BATCH" >&2
    exit 2
fi
if [ "$DRY_RUN" -eq 0 ] && [ -z "$KEY" ]; then
    echo "sign-symbols: --key is required to sign; use --dry-run to preview" >&2
    exit 2
fi
if [ "$DRY_RUN" -eq 0 ] && [ ! -f "$KEY" ]; then
    echo "sign-symbols: private key not found: $KEY" >&2
    exit 2
fi
if [ -z "$SIGNER" ]; then
    SIGNER=$(awk '!/^[[:space:]]*#/ && NF { print $1; exit }' "$SIGNERS")
fi
if [ -z "$SIGNER" ]; then
    echo "sign-symbols: no principal in $SIGNERS; pass --signer" >&2
    exit 2
fi

TMP_MANIFEST=""
if [ "$DRY_RUN" -eq 1 ]; then
    TMP_MANIFEST=$(mktemp)
    trap 'rm -f "$TMP_MANIFEST"' EXIT
fi

cd "$ROOT_DIR"
python3 - "$ROOT_DIR" "$BATCH" "$SIGNER" "$DRY_RUN" "$CLOSE_BASE" "$TMP_MANIFEST" <<'PY'
import hashlib
import json
import re
import subprocess
import sys
import tomllib
from datetime import datetime, timezone
from pathlib import Path

root = Path(sys.argv[1]).resolve()
batch_path = Path(sys.argv[2]).resolve()
signer = sys.argv[3]
dry_run = sys.argv[4] == "1"
close_base = sys.argv[5] == "1"
target = Path(sys.argv[6]).resolve() if sys.argv[6] else None
manifest_path = root / "assets" / "symbols" / "signed.toml"
namespace = "punchi-fluid-signed-symbols"
status_re = re.compile(r"^\s{4}status:\s*(\S+)\s*$", re.M)
id_re = re.compile(r'^id\s*=\s*"([^"]+)"', re.M)


def fail(message):
    print(f"sign-symbols: {message}", file=sys.stderr)
    sys.exit(1)


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def quote(value):
    return json.dumps(value, ensure_ascii=False)


try:
    batch = tomllib.loads(batch_path.read_text(encoding="utf-8"))
except Exception as error:
    fail(f"the batch file is not valid TOML: {error}")

batch_name = batch.get("name")
if not isinstance(batch_name, str) or not batch_name:
    fail("the batch file must declare a non-empty name")
ids = batch.get("symbols")
if not isinstance(ids, list) or not ids:
    fail("the batch file must declare a non-empty symbols list")
if any(not isinstance(item, str) or not re.fullmatch(r"[a-z][a-z0-9_.]*", item) for item in ids):
    fail("every symbol in the batch must be a component id")
if len(set(ids)) != len(ids):
    fail("the batch lists a symbol twice")

try:
    manifest_text = manifest_path.read_text(encoding="utf-8")
    manifest = tomllib.loads(manifest_text)
except Exception as error:
    fail(f"signed.toml is not valid TOML: {error}")
if manifest.get("schema") != 1 or manifest.get("namespace") != namespace:
    fail("signed.toml has the wrong schema or namespace")
entries = manifest.get("symbol") or []
if not isinstance(entries, list):
    fail("signed.toml: symbol must be an array of tables")
by_id = {}
for entry in entries:
    cid = entry.get("id")
    if cid in by_id:
        fail(f"signed.toml lists {cid} twice")
    by_id[cid] = entry

signed_principals = {entry.get("signed_by") for entry in entries if "signed_by" in entry}
if signed_principals and signed_principals != {signer}:
    fail(
        "signed.toml already carries signatures from "
        f"{sorted(signed_principals)}; this batch names {signer!r}"
    )

head = subprocess.run(
    ["git", "rev-parse", "HEAD"],
    cwd=root,
    check=True,
    capture_output=True,
    text=True,
).stdout.strip()
if not re.fullmatch(r"[0-9a-f]{40}", head):
    fail(f"git rev-parse HEAD returned {head!r}")
today = datetime.now(timezone.utc).date().isoformat()

added = 0
updated = 0
for cid in ids:
    review = root / "assets" / "symbols" / "review" / f"{cid}.md"
    toml = root / "assets" / "catalog" / f"{cid}.toml"
    captures = [
        root / "assets" / "symbols" / "review" / f"{cid}-300.png",
        root / "assets" / "symbols" / "review" / f"{cid}-light.png",
        root / "assets" / "symbols" / "review" / f"{cid}-dark.png",
    ]
    for path in [review, toml, *captures]:
        if not path.is_file():
            fail(f"{cid}: missing {path.relative_to(root)}")
    review_text = review.read_text(encoding="utf-8")
    status = status_re.search(review_text)
    if not status or status.group(1) != "complete":
        fail(f"{cid}: the review is not `complete`")
    if not id_re.search(toml.read_text(encoding="utf-8")):
        fail(f"{cid}: the definition declares no id")

    entry = by_id.get(cid)
    if entry is None:
        entry = {
            "id": cid,
            "revision": 1,
            "previous_revision": 0,
            "review": f"assets/symbols/review/{cid}.md",
            "toml": f"assets/catalog/{cid}.toml",
            "captures": [
                f"assets/symbols/review/{cid}-300.png",
                f"assets/symbols/review/{cid}-light.png",
                f"assets/symbols/review/{cid}-dark.png",
            ],
        }
        entries.append(entry)
        by_id[cid] = entry
        added += 1
    else:
        updated += 1
    entry["sha256_toml"] = sha256(toml)
    entry["sha256_review"] = sha256(review)
    entry["sha256_300"] = sha256(captures[0])
    entry["sha256_light"] = sha256(captures[1])
    entry["sha256_dark"] = sha256(captures[2])
    entry["commit"] = head
    entry["signed_by"] = signer
    entry["signed_at"] = today

if close_base:
    pending = [entry.get("id") for entry in entries if "signed_by" not in entry]
    if pending:
        fail(f"--close-base refuses to run while entries are pending: {pending}")

prefix_end = manifest_text.find("[[symbol]]")
if prefix_end < 0:
    fail("signed.toml has no symbol entries")
prefix = manifest_text[:prefix_end].rstrip()
if close_base:
    prefix = re.sub(r"^base_open\s*=\s*(?:true|false)\s*$", "base_open = false", prefix, flags=re.M)
    if "base_open = false" not in prefix:
        fail("signed.toml has no base_open line to close")

field_order = [
    "id",
    "revision",
    "previous_revision",
    "review",
    "toml",
    "captures",
    "sha256_toml",
    "sha256_review",
    "sha256_300",
    "sha256_light",
    "sha256_dark",
    "commit",
    "signed_by",
    "signed_at",
]
out = [prefix, ""]
for entry in entries:
    out.append("[[symbol]]")
    for field in field_order:
        if field not in entry:
            continue
        value = entry[field]
        if field == "captures":
            rendered = "[" + ", ".join(quote(item) for item in value) + "]"
        elif isinstance(value, int):
            rendered = str(value)
        else:
            rendered = quote(value)
        out.append(f"{field} = {rendered}")
    out.append("")
new_manifest = "\n".join(out).rstrip() + "\n"

if dry_run:
    target.write_text(new_manifest, encoding="utf-8")
    print(f"sign-symbols: dry run for batch {batch_name}: {added} added, {updated} updated")
    print(f"sign-symbols: preview {target}")
else:
    manifest_path.write_text(new_manifest, encoding="utf-8")
    print(f"sign-symbols: batch {batch_name}: {added} added, {updated} updated")
PY

if [ "$DRY_RUN" -eq 1 ]; then
    diff -u "$MANIFEST" "$TMP_MANIFEST" || true
    echo "sign-symbols: dry run only; nothing signed"
    exit 0
fi

rm -f "$SIGNATURE"
ssh-keygen -Y sign -f "$KEY" -n "$NAMESPACE" "$MANIFEST"
ssh-keygen -Y verify -f "$SIGNERS" -I "$SIGNER" -n "$NAMESPACE" -s "$SIGNATURE" < "$MANIFEST"
./scripts-dev/check-signed-symbols.sh

echo "sign-symbols: signed batch with $SIGNER; commit assets/symbols/signed.toml and"
echo "assets/symbols/signed.toml.sig, then open or update the pull request."
