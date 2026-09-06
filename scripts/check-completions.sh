#!/bin/sh
set -eu

completion_tmp=$(mktemp -d "${TMPDIR:-/tmp}/tmpmail-completions.XXXXXX")
trap 'rm -rf "$completion_tmp"' EXIT HUP INT TERM
cargo run --quiet --locked --example generate-completions -- "$completion_tmp"
diff -u completions/tmpmail.bash "$completion_tmp/tmpmail.bash"
diff -u completions/_tmpmail "$completion_tmp/_tmpmail"
diff -u completions/tmpmail.fish "$completion_tmp/tmpmail.fish"
