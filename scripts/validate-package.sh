#!/bin/sh
set -eu

archive=${1:?usage: scripts/validate-package.sh ARCHIVE}
checksum="$archive.sha256"
test -s "$checksum"
if command -v shasum >/dev/null 2>&1; then
  (cd "$(dirname "$archive")" && shasum -a 256 -c "$(basename "$checksum")")
else
  (cd "$(dirname "$archive")" && sha256sum -c "$(basename "$checksum")")
fi
work=$(mktemp -d "${TMPDIR:-/tmp}/tmpmail-package.XXXXXX")
trap 'rm -rf "$work"' EXIT HUP INT TERM

tar -xzf "$archive" -C "$work"
root=$(find "$work" -mindepth 1 -maxdepth 1 -type d | head -n 1)
test -n "$root"
test -x "$root/tmpmail"
test -s "$root/README.md"
test -s "$root/LICENSE"
test -s "$root/tmpmail.1"
test -s "$root/completions/tmpmail.bash"
test -s "$root/completions/_tmpmail"
test -s "$root/completions/tmpmail.fish"
test -s "$root/tmpmail.cdx.json"
jq -e '.bomFormat == "CycloneDX"' "$root/tmpmail.cdx.json" >/dev/null

install_root="$work/install"
mkdir -p "$install_root/bin"
cp "$root/tmpmail" "$install_root/bin/tmpmail"
"$install_root/bin/tmpmail" --version | grep -Eq '^tmpmail 2\.0\.0$'
rm -f "$install_root/bin/tmpmail"
test ! -e "$install_root/bin/tmpmail"
