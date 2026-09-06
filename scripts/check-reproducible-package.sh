#!/bin/sh
set -eu

target=${1:?usage: scripts/check-reproducible-package.sh TARGET}
version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)
archive="dist/tmpmail-$version-$target.tar.gz"

./scripts/package.sh "$target"
if command -v shasum >/dev/null 2>&1; then
  first=$(shasum -a 256 "$archive" | awk '{print $1}')
else
  first=$(sha256sum "$archive" | awk '{print $1}')
fi
./scripts/package.sh "$target"
if command -v shasum >/dev/null 2>&1; then
  second=$(shasum -a 256 "$archive" | awk '{print $1}')
else
  second=$(sha256sum "$archive" | awk '{print $1}')
fi

test "$first" = "$second"
printf 'reproducible_archive_sha256=%s\n' "$second"
