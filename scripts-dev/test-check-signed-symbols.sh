#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 PunchiSoft
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Tests scripts-dev/check-signed-symbols.sh against temporary trees. The check exists to
# make "signed" mean something, so each case here is one of the ways it could stop being
# true: a signature that does not verify, a hash edited after the signature, a `complete`
# review with no entry, a duplicate id, a revision that jumps, and a closed base that still
# has pending entries. The key pair is generated in the temporary directory and never
# leaves it.
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
CHECK="$ROOT_DIR/scripts-dev/check-signed-symbols.sh"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
fail=0

ssh-keygen -q -t ed25519 -N '' -C "fixture" -f "$tmp/key"

make_tree() {
    local root=$1
    mkdir -p "$root/assets/catalog" "$root/assets/symbols/review"
    printf 'id = "valve.test"\n' >"$root/assets/catalog/valve.test.toml"
    cat >"$root/assets/symbols/review/valve.test.md" <<'MD'
# valve.test — conformance review

    status: complete
MD
    local suffix
    for suffix in 300 light dark; do
        printf 'capture:%s\n' "$suffix" >"$root/assets/symbols/review/valve.test-$suffix.png"
    done
    printf 'test@punchisoft %s\n' "$(cat "$tmp/key.pub")" >"$root/assets/symbols/owner_signers"
}

# write_manifest <root> <revision> <previous> <signed: yes|no>
write_manifest() {
    local root=$1 revision=$2 previous=$3 signed=$4
    python3 - "$root" "$revision" "$previous" "$signed" <<'PY'
import hashlib
import sys
from pathlib import Path

root = Path(sys.argv[1])
revision = int(sys.argv[2])
previous = int(sys.argv[3])
signed = sys.argv[4] == "yes"
review = root / "assets/symbols/review"
catalog = root / "assets/catalog/valve.test.toml"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


captures = [f"assets/symbols/review/valve.test-{suffix}.png" for suffix in ("300", "light", "dark")]
lines = [
    "schema = 1",
    'namespace = "punchi-fluid-signed-symbols"',
    "base_open = true",
    "",
    "[[symbol]]",
    'id = "valve.test"',
    f"revision = {revision}",
    f"previous_revision = {previous}",
    'review = "assets/symbols/review/valve.test.md"',
    'toml = "assets/catalog/valve.test.toml"',
    f'captures = ["{captures[0]}", "{captures[1]}", "{captures[2]}"]',
    f'sha256_toml = "{sha(catalog)}"',
    f'sha256_review = "{sha(review / "valve.test.md")}"',
    f'sha256_300 = "{sha(root / captures[0])}"',
    f'sha256_light = "{sha(root / captures[1])}"',
    f'sha256_dark = "{sha(root / captures[2])}"',
]
if signed:
    lines += ['signed_by = "test@punchisoft"', 'signed_at = "2026-11-24"']
(root / "assets/symbols/signed.toml").write_text("\n".join(lines) + "\n", encoding="utf-8")
PY
}

sign_manifest() {
    local root=$1
    ssh-keygen -Y sign -f "$tmp/key" -n punchi-fluid-signed-symbols \
        "$root/assets/symbols/signed.toml" >/dev/null 2>&1
}

expect_ok() {
    local name=$1 root=$2
    if ! ROOT="$root" "$CHECK" >"$tmp/out" 2>&1; then
        echo "check-signed-symbols fixture '$name' failed but should pass:" >&2
        cat "$tmp/out" >&2
        fail=1
    fi
}

expect_fail() {
    local name=$1 root=$2 pattern=$3
    if ROOT="$root" "$CHECK" >"$tmp/out" 2>&1; then
        echo "check-signed-symbols fixture '$name' passed but should fail" >&2
        fail=1
    elif ! grep -q -- "$pattern" "$tmp/out"; then
        echo "check-signed-symbols fixture '$name' failed without '$pattern':" >&2
        cat "$tmp/out" >&2
        fail=1
    fi
}

# The base as it lands in Phase 1: hashes pinned, no signature yet.
root="$tmp/pending"
make_tree "$root"
write_manifest "$root" 1 0 no
expect_ok pending "$root"

# A signed entry whose signature verifies.
root="$tmp/signed"
make_tree "$root"
write_manifest "$root" 1 0 yes
sign_manifest "$root"
expect_ok signed "$root"

# A signature that does not verify.
root="$tmp/badsig"
make_tree "$root"
write_manifest "$root" 1 0 yes
printf 'not a signature\n' >"$root/assets/symbols/signed.toml.sig"
expect_fail badsig "$root" "signature does not verify"

# A hash edited after the owner signed.
root="$tmp/hash"
make_tree "$root"
write_manifest "$root" 1 0 yes
sign_manifest "$root"
printf 'changed\n' >>"$root/assets/catalog/valve.test.toml"
expect_fail hash "$root" "sha256_toml does not match"

# A complete review with no entry in the manifest.
root="$tmp/complete"
make_tree "$root"
write_manifest "$root" 1 0 no
cat >"$root/assets/symbols/review/valve.other.md" <<'MD'
# valve.other — conformance review

    status: complete
MD
expect_fail complete "$root" "has no entry in signed.toml"

# The same id twice.
root="$tmp/duplicate"
make_tree "$root"
write_manifest "$root" 1 0 no
python3 - "$root" <<'PY'
import sys
from pathlib import Path

path = Path(sys.argv[1]) / "assets/symbols/signed.toml"
text = path.read_text(encoding="utf-8")
path.write_text(text + "\n" + text[text.index("[[symbol]]"):], encoding="utf-8")
PY
expect_fail duplicate "$root" "duplicate id"

# A revision that jumps instead of incrementing.
root="$tmp/revision"
make_tree "$root"
write_manifest "$root" 3 5 no
expect_fail revision "$root" "previous_revision must be revision - 1"

# A closed base that still lists pending entries.
root="$tmp/closed"
make_tree "$root"
write_manifest "$root" 1 0 no
sed -i 's/^base_open = true/base_open = false/' "$root/assets/symbols/signed.toml"
expect_fail closed "$root" "base_open is false but"

if ((fail != 0)); then
    exit 1
fi
echo "check-signed-symbols: the fixtures pass"
