# MS-6 Evidence — Performance, Resilience, and Review

Status: accepted locally on 2026-09-06. No live mailbox, remote branch, tag,
package registry, formula, or release was changed.

## Corrected behavior

- State schema migration, concurrent mutation, atomic replacement, and
  corrupted/newer-state failure paths are covered by deterministic tests.
- Inbox creation uses unique credential references and reports possible remote
  or credential orphans as exit-10 partial operations.
- Inbox removal compares the captured provider and credential reference while
  holding the state lock, so a concurrent same-name replacement is preserved.
- DropMail inbox credentials bind the current global token at access time;
  serialized inbox records no longer duplicate a rotatable provider token.
- Native credential rotation removes stale protected-file copies. A cleanup
  failure after the native mutation is disclosed as an exit-10 partial
  operation rather than an ordinary all-or-nothing credential failure.
- Mail.tm collection pagination is bounded, same-origin, and governed by one
  operation deadline. DropMail raw source reads use the selected message's
  same-origin download URL and a streamed 25 MiB limit rather than selecting
  every raw message in the session.
- Local unseen state is authoritative over provider read flags, acknowledgement
  gaps compact to the current fully fetched provider view, and newest-message
  ties are deterministic.
- Invalid JSON-mode invocations preserve the `tmpmail/v1` envelope, lifecycle
  failures use partial-operation semantics, and human output sanitizes
  provider-controlled fields at the final output boundary.

## Verification

- Rust `1.88.0`; `cargo fmt --check`: passed.
- `cargo clippy --all-targets --all-features --locked -- -D warnings`: passed.
- `cargo test --all-targets --all-features --locked`: passed after final review
  with 68 tests and one intentionally ignored reference-machine benchmark.
- `cargo deny check advisories bans licenses sources`: passed. Transitive
  duplicate-version warnings remain informational under the accepted policy.
- Documentation, generated completion, and CycloneDX generation checks: passed.
- Release binary on the identified macOS arm64 host: 2,271,008 bytes. Peak help
  RSS: 2,162,688 bytes.
- Warm offline benchmark, 200 runs with 20 warmups and `--shell=none`:
  - help: p95 7.00 ms; p99 10.02 ms;
  - version: p95 5.31 ms; p99 5.78 ms;
  - config show: p95 5.34 ms; p99 6.16 ms;
  - 100-inbox list: p95 6.34 ms; p99 7.26 ms.
- The benchmark host load averages were 10.99, 11.66, and 11.26. All budgets
  passed despite the noisy host; the raw benchmark JSON remains outside the
  repository under `/private/tmp`.
- Deterministic Mail.tm loopback adapter benchmark: p95 0.276 ms, excluding
  injected network delay.
- A fresh independent read-only review found and then verified corrections for
  the same-name removal race, partial native/fallback credential mutation, and
  unbounded DropMail raw selection. The remediation re-review found no blocker.
- Repository secret scan found only explicit fake fixture credentials.

## Boundaries

- No live Mail.tm or DropMail canary was run because mailbox creation and
  controlled message delivery require separate external-write authority.
- Linux and macOS x86_64 native runtime/package checks remain release-candidate
  gates for the prepared CI matrix; this host verified macOS arm64 only.
- Performance numbers are local reference-host evidence, not shared-runner CI
  gates. No profile-driven optimization beyond the measured state-read path was
  necessary.
