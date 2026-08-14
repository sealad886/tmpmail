# MS-3 — DropMail and creation fallback evidence

Deterministic implementation evidence, recorded 2026-08-14:

- Current official DropMail API and GraphQL schema refreshed.
- Secure interactive or `--token-stdin` onboarding; no token argv option.
- Session creation, expiry refresh, restore metadata, message retrieval,
  attachments, domain discovery, and remote address removal implemented.
- Partial GraphQL data with errors is rejected. Token/session/rate-limit and
  CAPTCHA cases are normalized and redacted.
- Creation preference, command exclusion, configured deny-list, provider pin,
  custom-address capability, fallback eligibility, total-deadline accounting,
  attempt ledger, and visible fallback notice implemented.
- Existing inboxes remain bound to their stored provider.
- `message new --all-inboxes` fetches inboxes concurrently and includes inbox
  and provider identity. Partial failures return exit 10 with structured data.
- DropMail contract tests use loopback-only fixtures; no provider mutation.
- Full deterministic suite: 30 tests passed before final evidence update.
- Size-optimized fat-LTO binary: 2,002,464 bytes, down from an initial
  2,805,664-byte MS-3 build.
- Last valid isolated help sample: p95 9.51 ms, p99 11.80 ms (500 runs).
- Later version samples are invalid as release evidence: host load averages
  reached 65.64/95.76/90.71 and `/usr/bin/true` itself measured 14.11 ms p95.
  Stable-reference version/RSS confirmation remains release-blocking MS-6 work;
  no performance regression is accepted or waived here.

Live DropMail and forced live-provider fallback canaries were not run because
they require external-write authority. Local deterministic gates and final
performance measurements are recorded before milestone commit.
