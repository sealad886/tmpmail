# November 2026 release qualification

Status: local candidate preparation. Publication remains blocked on fresh live
provider canaries and the other three native target jobs. No release, push, tag,
or external account/session creation is authorized by this document.

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
- Four dependency patch updates clear seven RustSec advisories: rustls
  0.23.43 to 0.23.46, rustls-webpki 0.103.14 to 0.103.16, Tokio 1.53.1 to 1.53.3,
  and webpki-roots 1.0.9 to 1.0.10. No new dependencies were added.
- Packaging explicitly selects the pinned toolchain on PATH, propagates tar and
  metadata failures, derives the SBOM application version, and uses UTC archive
  timestamps.

## Verification

- On this macOS arm64 host, Rust 1.88 all-target tests: 76 passed; one reference
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
- Read-only Mail.tm domain discovery succeeded. No authenticated live canary
  was run; deterministic contract servers do not prove external acceptance.

## Candidate artifacts and remaining gates

After committing the candidate, run from a clean checkout:

```sh
./scripts/package.sh aarch64-apple-darwin
./scripts/validate-package.sh dist/tmpmail-2.0.0-aarch64-apple-darwin.tar.gz
./scripts/check-reproducible-package.sh aarch64-apple-darwin
```

Ignored `dist/` output holds the archive, checksum, and provenance naming the
exact source revision. A local build is not a published release.

Before shipping:

1. Authorize isolated live canaries for Mail.tm and DropMail: create a test
   inbox, receive controlled text/HTML plus a harmless attachment, list/read/raw
   read/download, restart and restore, then remove only those test resources
   where supported. DropMail requires a current free token supplied through
   protected input. A controlled sender must also be authorized separately.
2. Run the existing native matrix for Linux arm64/x86_64 and macOS x86_64, plus
   macOS arm64, from the exact candidate revision. Pushing the branch or running
   remote CI requires explicit authorization.
3. Once all gates pass, obtain publication authorization, produce final assets
   and checksums, and fill the Homebrew template from immutable release URLs.

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
