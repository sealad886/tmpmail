#!/bin/sh
set -eu

grep -q 'Mail.tm functionality is powered by' README.md
grep -q 'tmpmail/v1' README.md
grep -q 'tmpmail message new' tmpmail.1
grep -q 'message.*new-mail' src/help.txt
test -s completions/tmpmail.bash
test -s completions/_tmpmail
test -s completions/tmpmail.fish
test -s docs/migration-v1-to-v2.md
test -s docs/release-notes-2.0.0.md
test -s docs/troubleshooting.md
grep -q '@AARCH64_MACOS_SHA256@' packaging/homebrew/tmpmail.rb.in
jq -e '.title == "tmpmail/v1 envelope"' docs/schema/tmpmail-v1.schema.json >/dev/null
