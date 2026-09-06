#!/bin/sh
set -eu

target=${1:?usage: scripts/package.sh TARGET}
version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)
binary="target/$target/release/tmpmail"
name="tmpmail-$version-$target"
archive="dist/$name.tar.gz"

if test -n "$(git status --porcelain --untracked-files=normal)"; then
  echo "refusing to package a dirty or untracked working tree" >&2
  exit 1
fi
cargo +1.88.0 build --release --locked --target "$target"
test -x "$binary"
mkdir -p dist
package_tmp=$(mktemp -d "dist/.tmpmail-package.XXXXXX")
trap 'rm -rf "$package_tmp"' EXIT HUP INT TERM
stage="$package_tmp/$name"
mkdir -p "$stage"
cp "$binary" "$stage/tmpmail"
cp README.md LICENSE tmpmail.1 "$stage/"
cp -R completions "$stage/"
./scripts/sbom.sh "$stage/tmpmail.cdx.json"
find "$stage" -type d -exec chmod 0755 {} +
find "$stage" -type f -exec chmod 0644 {} +
chmod 0755 "$stage/tmpmail"
find "$stage" -exec touch -t 200001010000.00 {} +
if tar --version 2>/dev/null | grep -q bsdtar; then
  (
    cd "$package_tmp"
    find "$name" -print | LC_ALL=C sort |
      tar --no-recursion --format ustar --uid 0 --gid 0 --uname root --gname root -cf - -T -
  ) | gzip -n > "$archive"
else
  (
    cd "$package_tmp"
    find "$name" -print | LC_ALL=C sort |
      tar --no-recursion --format=ustar --owner=0 --group=0 --numeric-owner -cf - -T -
  ) | gzip -n > "$archive"
fi
if command -v shasum >/dev/null 2>&1; then
  (cd "$(dirname "$archive")" && shasum -a 256 "$(basename "$archive")") > "$archive.sha256"
else
  (cd "$(dirname "$archive")" && sha256sum "$(basename "$archive")") > "$archive.sha256"
fi
{
  printf 'source_commit=%s\n' "$(git rev-parse HEAD)"
  printf 'rustc=%s\n' "$(rustc +1.88.0 --version)"
  printf 'target=%s\n' "$target"
  printf 'archive=%s\n' "$(basename "$archive")"
  if command -v shasum >/dev/null 2>&1; then
    printf 'binary_sha256=%s\n' "$(shasum -a 256 "$binary" | awk '{print $1}')"
  else
    printf 'binary_sha256=%s\n' "$(sha256sum "$binary" | awk '{print $1}')"
  fi
} > "$archive.provenance"
