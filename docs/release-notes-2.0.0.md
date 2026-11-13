# tmpmail 2.0.0 release notes

Status: release candidate with live-provider acceptance completed; not tagged
or published. Native target CI and final provenance identify the release input.

## Highlights

- Native Rust executable for macOS and Linux.
- Authenticated Mail.tm and DropMail provider adapters.
- Named inboxes with provider-bound credentials and creation-only fallback.
- Stable `tmpmail/v1` JSON envelopes and documented exit codes.
- Explicit unseen-message acknowledgement and multi-inbox foreground watch.
- Exact latest-message, all-new-message, and opt-in raw MIME source reads.
- Text-first rendering, inert cached HTML, and streamed safe attachments.
- Keyring-first secrets with explicit protected-file fallback.
- Current DropMail bearer authentication, paginated inboxes, and nullable
  attachment handling.
- Successful nested help/version requests and explicit watch inbox selection.
- Enabled network I/O in the shared CLI runtime, fixing panics on real provider
  requests that adapter-only contract tests did not exercise.
- Mail.tm acknowledgement uses the current merge-patch media type. Attachment
  validation uses decoded download length rather than unreliable size metadata.
- Read completion failures preserve a single JSON payload and exit 10, with a
  diagnostic on stderr. HTML cache preparation happens before successful output.
- DropMail restoration avoids creating an extra unused random address.
- Networking/TLS dependency security patches within existing version ranges.
- Packaging now selects Rust 1.88 explicitly, propagates archive/metadata
  failures, derives the SBOM version from Cargo, and normalizes timestamps in UTC.
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

## Acceptance and publication

- Authorized live canaries passed for both providers, including Gmail delivery,
  attachments, Mail.tm token renewal, and genuine DropMail expiry/restoration.
  Disposable inboxes, addresses, and test credentials were cleaned up.
- The native CI matrix covers all four supported targets. See the
  [qualification evidence](delivery/2026-11-13-modernization.md) and
  [branch CI](https://github.com/sealad886/tmpmail/actions?query=branch%3Acodex%2Frelease-modernization)
  for exact-revision results.
- Final checksums, artifact identities, and provenance must be produced from the
  exact candidate commit.
- The Homebrew template placeholders must be replaced only after immutable
  release URLs and archive checksums exist.
- Tagging, release publication, and Homebrew publication require separate
  authorization.
