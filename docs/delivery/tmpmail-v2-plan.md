# tmpmail v2 delivery, quality, and risk plan

Status: Approved

Authority: local changes and commits only

## Milestones

| ID | Demonstrable outcome | Evidence gate | Expected commit |
|---|---|---|---|
| MS-0 | Accepted design, requirements, ADR, risks, and test plan | Placeholder/consistency/traceability review | `docs(design): define tmpmail v2 architecture` |
| MS-1 | Fast Rust CLI/state/config/secrets foundation | Format, Clippy, tests, schemas, offline benchmarks | `feat(core): establish tmpmail v2 command foundation` |
| MS-2 | Complete Mail.tm vertical slice | Contract tests, redaction, performance, authorized live canary | `feat(mailtm): add authenticated mailbox workflow` |
| MS-3 | DropMail plus creation fallback | GraphQL/error/fallback tests and authorized live canary | `feat(providers): add DropMail fallback` |
| MS-4 | Safe polished messages and attachments | Injection/path/platform/output tests | `feat(cli): complete safe message experience` |
| MS-5 | Multi-inbox foreground watch | Clock, concurrency, backoff, cursor, interruption tests | `feat(watch): monitor multiple inboxes` |
| MS-6 | Performance/resilience/security hardening | Full matrix, reference benchmarks, review, canaries | `perf(...)` and `fix(...)` by coherent cause |
| MS-7 | Local release-ready artifacts | Clean builds, package/install checks, checksums, readiness review | `build(release): prepare tmpmail v2 artifacts` |

Critical path is MS-0 through MS-7 in order. Ownership is unassigned. Relative
sizes: MS-0 M/high confidence; MS-1 L/medium; MS-2 L/medium; MS-3 L/medium;
MS-4 M/medium; MS-5 M/medium; MS-6 M/medium; MS-7 M/medium. These are planning
ranges, not calendar commitments.

## Conventional Commit gate

At every milestone completion, invoke the repository `conventional-commit`
workflow: inspect all staged/unstaged/untracked changes, classify secrets and
generated/local data, run milestone evidence, stage only coherent paths or
hunks, inspect staged diff and `git diff --cached --check`, create local
Conventional Commits, rerun focused smoke checks, and report hashes, exclusions,
verification, and residual risk. Never enable clean mode. Never push, tag, or
publish without separate authority.

## Test strategy

Unit tests cover commands/aliases, precedence, capabilities, fallback,
deadlines, output envelopes, exit codes, state migration/locking/recovery,
keyring failure, redaction, acknowledgement compaction, watch cursors/backoff,
and content/path sanitization.

Local provider-contract servers cover domains, creation, auth renewal, sessions,
pagination, empty and populated mailboxes, text/HTML, attachments, GraphQL
partial errors, malformed responses, status/content-type validation, rate
limits, timeouts, disconnects, truncation, and deletion.

Integration tests isolate config/state roots and substitute the keyring. They
cover first run, named inboxes, active selection, concurrent mutation,
interrupted writes, human/pipe/JSON output, acknowledgement/review, multi-inbox
aggregation, watch interruption, and platform command construction without
launching apps.

Live provider canaries are separate external mutations requiring explicit
authority. Each canary creates a mailbox, receives controlled text/HTML and a
harmless attachment, measures delivery, reads/downloads, restarts/restores,
exercises safe expiry/renewal cases, cleans up where supported, and checks that
credentials never appear in commands or artifacts. Live providers never gate
ordinary pull-request CI.

## RAID

| ID | Risk | Response | Trigger/review point |
|---|---|---|---|
| RISK-001 | Provider outage or contract drift | Typed adapters, status/content validation, deterministic contracts, explicit fallback, fresh canaries | Every provider milestone and release gate |
| RISK-002 | Credential disclosure | Keyring first, no argv secrets, opaque references, redaction tests, protected fallback opt-in | State/secrets work and security review |
| RISK-003 | Malicious mail content | Text first, control escaping, HTML sanitizer, safe attachments, no auto-open/execute | MS-4 and review |
| RISK-004 | Startup/memory regression | Fixed budgets, baseline each milestone, profile before optimize | Every milestone |
| RISK-005 | Noisy CI benchmarks | Separate trend artifacts from identified reference-machine gates | MS-1, MS-6, MS-7 |
| RISK-006 | Domain rejection/churn | Expose provider/domain, configurable order/deny-list, never promise deliverability | Provider canaries |
| RISK-007 | State corruption/concurrency | Locking, atomic replace, backup, validated migration, fault injection | MS-1 and MS-6 |
| RISK-008 | Scope growth | Daemon, desktop notifications, Windows, TUI, plugins, sending remain explicit non-scope | Every milestone review |
| RISK-009 | Provider terms change | Recheck authoritative docs, terms, attribution, and limits | Before provider implementation and release |

## CI and release evidence

Run formatting, Clippy with warnings denied, unit/integration/contract tests,
documentation and completion generation checks, JSON compatibility, dependency
advisory/license/provenance checks, and macOS/Linux arm64/x86_64 release builds.
Produce checksums and provenance locally. Artifact publication, tag creation,
Homebrew publication, remote push, and GitHub release creation are not
authorized by this plan.

## Change control

Changes to public commands, JSON v1, state schema, provider contract,
performance budgets, security defaults, supported platforms, or milestone
boundaries require requirements/design re-entry before implementation.
