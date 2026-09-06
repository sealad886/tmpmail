# MS-7 Evidence — Release-Ready v2 Candidate

Status: accepted as a local release candidate on 2026-09-06. The candidate is
the signed MS-7 Conventional Commit containing this evidence; its exact hash is
reported from Git after commit. Nothing was pushed, tagged, published, or
released.

## Release preparation

- Removed the defunct 1secmail POSIX executable, Alpine/curl container, and
  shellcheck-only workflow. The Rust binary is now the sole implementation.
- Added read-only GitHub Actions jobs for Rust 1.88 quality checks, dependency
  policy, and native archives on macOS/Linux arm64/x86_64.
- Added weekly Cargo and GitHub Actions dependency update configuration.
- Added generated Bash, Zsh, and Fish completions plus a deterministic drift
  check.
- Added migration, troubleshooting, release notes, JSON schema, Homebrew formula
  template, Linux archive guidance, packaging, checksum, SBOM, provenance, and
  archive validation material.
- Packaging stages each archive in a fresh temporary directory so stale files
  cannot leak from an earlier build.

## Verification

- Current GitHub-hosted labels were checked against the official runner table:
  `ubuntu-24.04`, `ubuntu-24.04-arm`, `macos-15-intel`, and `macos-15` cover the
  four supported target architectures.
- Official current action majors were checked for `actions/checkout@v6`,
  `actions/upload-artifact@v7`, and
  `EmbarkStudios/cargo-deny-action@v2.1.1`.
- `actionlint .github/workflows/ci.yml`: passed.
- `shellcheck scripts/*.sh` and POSIX `sh -n`: passed.
- Documentation and completion drift checks: passed.
- Rust 1.88 formatting, Clippy, all-target tests, dependency policy, and MS-6
  performance gates passed immediately before release preparation.
- macOS arm64 target build and archive validation passed: checksum verification,
  expected content inventory, temporary installation, `tmpmail 2.0.0` execution,
  and uninstall.
- After the MS-7 commit, the macOS arm64 archive is rebuilt so its provenance
  names the exact candidate revision, then the same validation is repeated.

## Publication and external gates

- `dist/` remains ignored local output; archives and checksums are not committed.
- Homebrew URL/checksum placeholders intentionally remain until immutable
  release assets exist. The template is not a published formula.
- Fresh Mail.tm and DropMail canaries require separate external-write authority
  and were not run.
- Linux and macOS x86_64 native jobs are prepared but cannot be evidenced from
  this macOS arm64 checkout without executing the remote CI matrix.
- Tags, GitHub releases, pushes, formula updates, and package publication remain
  outside this task's authority.
