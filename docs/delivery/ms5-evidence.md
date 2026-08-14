# MS-5 Evidence — Multi-Inbox Foreground Watch

Status: accepted locally on 2026-08-14. No provider mailbox, remote branch, tag,
package registry, or release was changed.

## Delivered behavior

- `inbox watch [NAME...]` temporarily includes named inboxes.
- `inbox watch --all` selects only inboxes explicitly enabled through
  `config set inbox.NAME.watch true`.
- First successful poll establishes a baseline per inbox unless `--existing`
  is requested. A failure in one inbox cannot advance another inbox's baseline.
- Durable ordered cursors prevent replay when an old provider page no longer
  contains the prior message ID.
- Polling is bounded in batches of eight, reuses provider clients, respects
  provider floors and `Retry-After`, and applies deterministic jittered backoff.
- Cursor state advances only after event output is flushed. Ctrl-C can cancel an
  active polling batch or the inter-poll wait.

## Verification

- `cargo fmt --check`: passed.
- `cargo clippy --all-targets --locked -- -D warnings`: passed.
- `cargo test --all-targets`: passed outside the filesystem sandbox because
  provider contract fixtures bind loopback sockets.
- Watch contracts cover opt-in selection, named temporary inclusion, per-inbox
  delayed baseline establishment, ordered high-water cursor rollover, polling
  floor, retry hints, backoff, and jitter.
- Release binary: 2.1 MiB stripped on macOS arm64.
- Warm process benchmarks on identified macOS arm64 host, 200 runs with 20
  warmups: top-level help p95 6.94 ms and p99 8.66 ms; version p95 5.79 ms and
  p99 9.75 ms. Peak help RSS: 2,179,072 bytes. Host load averaged 6.61, 7.97,
  and 8.43 during final measurement; all contractual limits passed.

Raw benchmark JSON remains an untracked temporary artifact under `/tmp`; it is
not part of repository or release content.

## Boundaries

No live provider canary was run. Live mailbox creation and controlled message
delivery require separate external-write authority and remain an MS-6/release
acceptance gate.
