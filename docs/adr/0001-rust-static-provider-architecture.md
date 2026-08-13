# ADR-0001: Rust single binary with static provider adapters

Status: Accepted

Date: 2026-08-13

## Context

The v1 POSIX shell script depends on several subprocesses, has no typed provider
boundary, and cannot safely represent the different Mail.tm and DropMail
credential and lifecycle models. tmpmail v2 has strict startup, memory,
security, portability, and multi-backend requirements.

## Decision

Rewrite tmpmail as one Rust binary. Use a provider-neutral core and sealed,
statically compiled Mail.tm and DropMail adapters selected through enum/static
dispatch. Keep offline commands synchronous and initialize asynchronous HTTP
support only for network operations.

Use versioned TOML/JSON local storage, platform keyrings with an explicitly
approved protected-file fallback, a stable JSON CLI envelope, and no dynamic
provider ABI in v2.

## Consequences

Positive:

- One distributable artifact with no normal-path shell dependencies.
- Typed provider state, capabilities, errors, and migrations.
- Fast offline startup and bounded memory are measurable and enforceable.
- Provider implementations remain independently testable behind one contract.
- Secrets and untrusted content can be handled centrally.

Negative:

- This is a deliberate breaking rewrite rather than an incremental shell patch.
- Contributors need a Rust toolchain.
- New providers require a source change and release.
- macOS/Linux keyring and packaging behavior need platform-specific validation.

## Alternatives rejected

- Modernize POSIX shell: preserves source portability but retains subprocess
  cost, weak state modeling, fragile JSON/error handling, and limited tests.
- Go binary: viable, but Rust better matches strict modeling and memory/startup
  control selected for this project.
- External plugins: adds startup, protocol, compatibility, trust, and packaging
  cost before third-party extensibility is proven necessary.
- Declarative JSON API mappings: cannot faithfully model JWT account renewal,
  DropMail GraphQL partial errors, session restore, or provider-specific limits.

## Revisit triggers

Revisit dynamic providers only after at least three maintained adapters expose a
stable shared contract and third-party extension demand is demonstrated. Revisit
runtime choice only if reference benchmarks or platform distribution fail the
accepted gates despite measured optimization.
