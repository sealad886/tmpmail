# tmpmail 2.0.0 release notes

Status: local release candidate preparation; not tagged or published.

## Highlights

- Native Rust executable for macOS and Linux.
- Authenticated Mail.tm and DropMail provider adapters.
- Named inboxes with provider-bound credentials and creation-only fallback.
- Stable `tmpmail/v1` JSON envelopes and documented exit codes.
- Explicit unseen-message acknowledgement and multi-inbox foreground watch.
- Exact latest-message, all-new-message, and opt-in raw MIME source reads.
- Text-first rendering, inert cached HTML, and streamed safe attachments.
- Keyring-first secrets with explicit protected-file fallback.
- Platform-standard config, state, cache, locking, migration backup, and atomic
  replacement.

## Performance evidence

On the identified macOS arm64 reference host, all warm offline commands met the
10 ms p95 and 20 ms p99 contract. Help measured 7.00/10.02 ms, version
5.31/5.78 ms, config show 5.34/6.16 ms, and listing 100 inboxes 6.34/7.26 ms.
Peak help RSS was 2.16 MiB and Mail.tm loopback adapter overhead measured
0.276 ms p95. See [the MS-6 evidence](delivery/ms6-evidence.md) for method and
boundaries.

## Breaking changes

This is a full CLI replacement. Historical single-letter shell options, numeric
positional message commands, the 1secmail backend, and automatic use of
`/tmp/tmpmail/email_address` are removed. See
[the migration guide](migration-v1-to-v2.md).

## Known release gates

- Fresh authorized provider canaries remain required before publication.
- Native runtime/package checks must pass for all four supported targets.
- Final checksums, artifact identities, and provenance must be produced from the
  exact candidate commit.
- The Homebrew template placeholders must be replaced only after immutable
  release URLs and archive checksums exist.
