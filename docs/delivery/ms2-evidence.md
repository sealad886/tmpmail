# MS-2 — Mail.tm vertical slice evidence

Deterministic implementation evidence, recorded 2026-08-14 on macOS arm64:

- Official API, authentication, message, event, quota, and attribution
  documentation refreshed and recorded in `docs/providers/mailtm.md`.
- Loopback provider contract tests cover active domain filtering, account and
  token creation, list/detail/attachment parsing, one-time JWT renewal,
  malformed JSON, incorrect content type, rate limiting, redaction, and
  cross-origin attachment URL refusal.
- Full suite: 16 tests passed.
- Formatting and Clippy with warnings denied: passed.
- Offline release binary: 2,639,920 bytes.
- Quiet repeat, 400 runs: `--version` p95 9.22 ms, p99 11.73 ms.
- Security/CoreFoundation are absent from eager dynamic linkage. macOS Keychain
  frameworks load only when credential access is required.

Live provider canary was not run because it creates an external mailbox and
requires separately authorized controlled-message delivery. MS-2 provider
acceptance remains conditional on that canary; deterministic implementation is
complete and independently reviewable.
