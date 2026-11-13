# November 2026 release qualification

Status: live-provider acceptance complete; release candidate prepared for
publication review. User subsequently authorized provider canaries, Gmail test
messages, test-resource cleanup, and pushing the branch for CI. Tags, releases,
and package publication remain unauthorized.

## Completed changes

- Retained the existing Rust v2 implementation and free, keyless Mail.tm default.
  Optional DropMail uses a user-supplied free token.
- DropMail now uses bearer authentication, `mailsConnection` pagination (at most
  100 pages of 100 messages, bounded by the configured timeout), nullable
  attachments, and errors-first GraphQL parsing. Pagination fails explicitly
  rather than returning a truncated inbox when its safety limit is exceeded.
- Both adapters reject redirects to preserve same-origin download validation.
- Nested help/version requests exit successfully. Watch honors `--inbox`, gives
  positional names precedence, and rejects combining explicit selection with
  `--all`. README and man page describe these rules.
- Enabled I/O in the shared CLI runtime after a release-binary public discovery
  check exposed a panic. A subprocess test uses an isolated silent loopback
  proxy to verify network timeouts return the JSON error envelope without
  contacting a provider.
- Live canaries exposed Mail.tm's required merge-patch media type and unreliable
  attachment size metadata. Fixed acknowledgement and decoded download-length
  validation while retaining HTTP/stream size limits and DropMail raw-size
  validation. Post-output failures now preserve one message envelope and exit 10.
- DropMail restoration creates an empty replacement session so it restores only
  the requested address, without leaving an unused random address.
- Four dependency patch updates clear seven RustSec advisories: rustls
  0.23.43 to 0.23.46, rustls-webpki 0.103.14 to 0.103.16, Tokio 1.53.1 to 1.53.3,
  and webpki-roots 1.0.9 to 1.0.10. No new dependencies were added.
- Packaging explicitly selects the pinned toolchain on PATH, propagates tar and
  metadata failures, derives the SBOM application version, and uses UTC archive
  timestamps.

## Verification

- On this macOS arm64 host, Rust 1.88 all-target tests: 79 passed; one reference
  performance test intentionally ignored in the ordinary suite.
- Rust 1.88 Clippy with warnings denied: passed. The compiler, Cargo, and Clippy
  were explicitly selected together; an earlier mixed Homebrew/rustup attempt
  was rejected as invalid compatibility evidence.
- Formatting, documentation checks, generated completion drift, actionlint,
  shellcheck, shell syntax, and whitespace checks: passed.
- cargo-deny advisories, bans, licenses, and sources: passed; duplicate-version
  warnings remain nonblocking.
- Assert-based packaging checks verified metadata failure produces no SBOM,
  the Cargo version appears in the SBOM, and tar failure produces no archive.
- Independent provider and release-tooling reviews found no remaining blocking
  issues after errors-first GraphQL handling was corrected.
- Live Mail.tm acceptance passed creation, Gmail plain/HTML delivery, Unicode,
  peek/unseen tracking, raw MIME, inert HTML caching, exact 67-byte attachment
  download, overwrite refusal, remote/local acknowledgement, process restart,
  and renewal after deliberately invalidating only the test token. A reversible
  canary-only storage failure verified one JSON payload and partial exit 10.
- Live DropMail acceptance passed the same message paths, then genuine ten-minute
  inactivity expiry, automatic same-address restoration, and a new Gmail message
  received/read after restoration. The empty-session replacement mutation was
  separately verified live to create zero unwanted addresses.
- Removed the Mail.tm test account/native credential/local inbox and all DropMail
  test addresses/local inboxes. Empty DropMail sessions expire naturally.
- All six jobs passed at the initial candidate `40a3f4c`, including native build,
  installation, archive validation, and reproducibility for macOS/Linux
  arm64/x86_64. The follow-up fixes are qualified by the latest
  [branch CI run](https://github.com/sealad886/tmpmail/actions?query=branch%3Acodex%2Frelease-modernization).

## Candidate artifacts and publication steps

After committing the candidate, run from a clean checkout:

```sh
./scripts/package.sh aarch64-apple-darwin
./scripts/validate-package.sh dist/tmpmail-2.0.0-aarch64-apple-darwin.tar.gz
./scripts/check-reproducible-package.sh aarch64-apple-darwin
```

Ignored `dist/` output holds the archive, checksum, and provenance naming the
exact source revision. A local build is not a published release.

Before publication, require green CI at the exact final branch revision and
retain the matching archive checksums/provenance. Obtain publication
authorization, produce immutable release assets, and fill the Homebrew template
from those URLs and checksums. No release tag or package has been published.

## Commit context

Git attributes the existing provider/core implementation to `7d235b461c03f44a01302a89cc9e1013aa53872f`
(`fix(core): harden provider and state workflows`, sealad886) and release tooling
to `9f1c6b9be466fbd044630c45bb2048d1d517790e`
(`build(release): prepare tmpmail v2 artifacts`, sealad886).
The commit-context skill was applied at audit and qualification milestones.
Its commit lookup tool was not exposed, and its localhost REST fallback was
unreachable. No linked agent-session intent could be established.

Provider references checked on November 13, 2026:
[Mail.tm](https://docs.mail.tm/), [DropMail API](https://dropmail.me/api/), and
[DropMail schema](https://dropmail.me/api/schema.graphql).
