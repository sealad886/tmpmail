#!/bin/sh
set -eu

binary=${1:-target/release/tmpmail}
output=${2:-target/tmpmail-offline-benchmark.json}
fixture=$(mktemp -d "${TMPDIR:-/tmp}/tmpmail-benchmark.XXXXXX")
trap 'rm -rf "$fixture"' EXIT HUP INT TERM

jq -n '
  def inbox($n): {
    name: ("inbox-" + ($n|tostring)),
    address: ("inbox-" + ($n|tostring) + "@example.test"),
    provider: "mailtm",
    secret_ref: ("fixture:" + ($n|tostring)),
    capabilities: {
      custom_address: true,
      domain_discovery: true,
      remote_delete: true,
      attachments: true,
      recoverable: true,
      retention_hint: "fixture",
      minimum_poll_seconds: 8
    },
    watch_enabled: false
  };
  {
    schema_version: 2,
    active_inbox: "inbox-0",
    inboxes: (reduce range(0; 100) as $n ({}; .[("inbox-" + ($n|tostring))] = inbox($n))),
    acknowledgements: {},
    acknowledged_ids: {},
    watch_cursors: {},
    watch_baselined: []
  }
' > "$fixture/state.json"

TMPMAIL_CONFIG_DIR="$fixture/config"
TMPMAIL_STATE_DIR="$fixture"
export TMPMAIL_CONFIG_DIR TMPMAIL_STATE_DIR

hyperfine --shell=none --warmup 20 --runs 200 --export-json "$output" \
  "$binary --help" \
  "$binary --version" \
  "$binary config show" \
  "$binary inbox list"

jq -r '
  .results[] |
  (.times | sort) as $times |
  [$times | length] as $count |
  [
    .command,
    ($times[(($count[0] * 0.95) | floor) - 1] * 1000),
    ($times[(($count[0] * 0.99) | floor) - 1] * 1000),
    (.mean * 1000)
  ] | @tsv
' "$output"
