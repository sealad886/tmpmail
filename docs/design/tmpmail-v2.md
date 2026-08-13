# tmpmail v2 product and solution design

Status: Accepted for implementation

Version target: 2.0.0

Supported platforms: macOS and Linux

Authority: local implementation and local commits; no push, tag, package publication, or release

## Problem and outcome

tmpmail 1.2.3 is a single POSIX shell script coupled to the unavailable
1secmail action API. A provider HTML error is currently accepted as mailbox
data, and the project has no provider boundary, durable authenticated state,
contract tests, or measurable performance gates.

tmpmail 2 replaces the script with one native Rust binary. It preserves the
original capabilities while adding named inboxes, Mail.tm and DropMail
backends, explicit creation fallback, stable JSON automation, secure local
state, and foreground multi-inbox monitoring.

## Requirements and acceptance

| ID | Requirement | Acceptance |
|---|---|---|
| REQ-001 | Preserve original product capabilities without preserving v1 syntax | Random/custom creation where supported, domains, list/read/latest, text/HTML, attachments, clipboard, browser, macOS, and Linux have tested v2 paths |
| REQ-002 | Support multiple provider-bound named inboxes | Users can create, list, select, use, and remove Mail.tm and DropMail inboxes; an existing inbox never silently changes provider |
| REQ-003 | Provide controlled provider fallback | Creation alone may use configured order; deny-list and explicit provider are honored; every attempt is reported without secrets |
| REQ-004 | Support humans and automation | TTY output is concise; `--json` uses `tmpmail/v1`, contains no ANSI or prompts, and keeps diagnostics off stdout |
| REQ-005 | Track locally unseen mail consistently | `latest` is singular; `new` lists without acknowledging; successful `read` acknowledges unless `--peek`; `ack` is explicit |
| REQ-006 | Monitor several inboxes in foreground | `watch` polls opted-in/named inboxes with bounded concurrency, provider limits, durable cursors, and clean Ctrl-C |
| REQ-007 | Protect secrets and untrusted content | Secrets use keyring-first storage, never argv/logs; terminal, HTML, URLs, and attachment paths are sanitized |
| REQ-008 | Be fast by contract | Offline p95 is at most 10 ms, p99 at most 20 ms, RSS at most 20 MiB, and mock-provider client overhead p95 at most 5 ms |
| REQ-009 | Fail predictably | Stable error codes and exit statuses distinguish usage, state, auth, provider, rate-limit, timeout, not-found, unsafe, and partial outcomes |
| REQ-010 | Be releasable without network-dependent PR tests | Deterministic provider contracts gate PRs; separately authorized live canaries gate provider/release acceptance |

### Public command contract

Canonical resource commands and first-class aliases:

```text
tmpmail                                  # message list for active inbox
tmpmail message list                    # aliases: tmpmail, tmpmail ls
tmpmail message read MESSAGE            # alias: tmpmail read MESSAGE
tmpmail message latest                  # alias: tmpmail latest
tmpmail message new [--all-inboxes]     # aliases: tmpmail new-mail, tmpmail unread
tmpmail message new --review
tmpmail message ack MESSAGE...
tmpmail inbox create [NAME]             # alias: tmpmail new [NAME]
tmpmail inbox list                      # alias: tmpmail inboxes
tmpmail inbox use NAME                  # alias: tmpmail use NAME
tmpmail inbox remove NAME
tmpmail inbox watch [NAME...]           # alias: tmpmail watch [NAME...]
tmpmail attachment list MESSAGE
tmpmail attachment save MESSAGE ATTACHMENT
tmpmail provider list
tmpmail provider domains [PROVIDER]
tmpmail provider auth PROVIDER
tmpmail provider info PROVIDER
tmpmail config show|get|set|unset
tmpmail copy [NAME]
tmpmail doctor
```

Common options are `--inbox`, `--provider`, repeatable `--no-provider`,
`--json`, `--no-color`, `--timeout`, `--quiet`, and `--verbose`. Secrets are
accepted only through a protected prompt or `--token-stdin`, never an ordinary
argument.

### JSON and exit contract

Successful output uses:

```json
{
  "schema": "tmpmail/v1",
  "ok": true,
  "data": {},
  "meta": {
    "provider": "mailtm",
    "fallback_used": false,
    "attempts": []
  }
}
```

Errors use the same envelope with `ok: false`, stable code, safe message,
retryability, and redacted provider metadata. Exit codes: `0` success, `2`
usage, `3` config/state, `4` auth/secrets, `5` provider unavailable, `6` rate
limit, `7` timeout, `8` not found, `9` unsafe operation, and `10` partial.
Successful fallback exits `0` and sets `fallback_used`.

### New-message contract

`message latest` returns one message. `message new` never acknowledges.
`message read` acknowledges only after successful output; `--peek` prevents
that. `message ack` supports noninteractive acknowledgement. JSON never
prompts.

`message new --review` shows a numbered table. For the selected message, `r`
reads then acknowledges after successful render, `a` acknowledges without
reading, `s` leaves it unseen, and `q` exits without changing remaining state.
Acknowledgements use a compact high-water mark plus gap set.

## Architecture

One Rust workspace produces one `tmpmail` binary. Modules have narrow roles:

- `cli`: parsing, aliases, validation, and output-mode selection.
- `core`: provider-neutral models, use cases, acknowledgement, fallback, errors.
- `providers`: sealed contract with Mail.tm and DropMail adapters.
- `state`: configuration, metadata, schema migration, locking, atomic writes.
- `secrets`: OS keyring and explicit protected-file fallback.
- `render`: human output, JSON v1, safe text and HTML.
- `watch`: foreground scheduler, backoff, and cursors.
- `platform`: paths, clipboard, browser, and TTY behavior.

Offline commands stay synchronous. Network commands initialize one async
runtime and reusable HTTP client. Provider selection uses static enum dispatch;
v2 has no dynamic provider ABI or external provider subprocesses.

### Provider contract

Adapters expose identity, attribution, capabilities, domains/health, inbox
create/restore/delete, message list/read, attachments, renewal, rate hints, and
normalized errors. Capabilities include custom-address support, remote delete,
attachments, recoverability, retention hints, and polling floor.

Mail.tm uses domain discovery, account/password creation, JWT authentication,
one password-based token refresh, messages, attachments, and account deletion.
DropMail uses a securely stored `af_` token, session/address/restore state,
GraphQL messages and attachments, and treats any `errors` array as failure even
when `data` exists. DropMail rejects custom local parts before network access.

### State and secrets

Linux honors XDG config/state/cache directories. macOS uses Application
Support and Caches. Config is TOML; mutable metadata is versioned JSON. Message
bodies are not cached by default. Mutations take an inter-process lock and use
write, flush, fsync, and atomic rename. Migration creates a backup and restores
it on validation failure. v1 `/tmp/tmpmail/email_address` is never imported.

macOS Keychain and Linux Secret Service are preferred. If unavailable, the CLI
must explain the risk and obtain explicit opt-in before creating a `0600`
`secrets.json`. Config, doctor, JSON, and logs expose only secret presence or an
opaque reference.

### Creation fallback

Selection precedence: explicit provider, command exclusions, config deny-list,
config order, then Mail.tm followed by configured DropMail. Only new-inbox
creation can fallback. Eligible failures are DNS/connectivity, timeout, 5xx,
documented overload/outage, rate limit with another candidate, and temporary
domain unavailability. Validation, unsupported capability, pinned provider,
missing token, keyring/state, auth/config, malformed response, and cancellation
never fallback.

Attempts share one deadline. Credentials persist before inbox metadata. If the
metadata commit fails, remote deletion is best effort and possible orphan state
is reported. Human fallback notice goes to stderr; JSON returns the attempt
ledger.

### Monitoring and content safety

`watch` is foreground-only. Initial polling establishes a baseline unless
`--existing` is requested. It bounds concurrency, respects provider limits and
`Retry-After`, uses jittered exponential backoff, isolates inbox failures, and
flushes confirmed cursors on Ctrl-C.

Mail content is untrusted. Terminal controls are escaped. Plain text is the
default; HTML is sanitized, disables active content and remote images, and is
opened only through explicit action. Attachments use sanitized basenames,
exclusive creation, overwrite opt-in, streaming size limits, advertised-size
checks, and incomplete-file cleanup. Subprocesses are limited to explicit
clipboard/browser actions.

## Performance contract

On reference macOS arm64 and Linux x86_64 optimized builds:

- Help, version, config read, and listing 100 inboxes: warm p95 <= 10 ms and
  p99 <= 20 ms.
- Offline peak RSS <= 20 MiB.
- Local mock-provider client overhead p95 <= 5 ms, excluding injected delay.
- No network/keyring access for help/version and no subprocess on normal paths.
- Each milestone records stripped binary size, startup, RSS, and relevant
  provider/mock latency.
- A measured regression over 10% blocks the milestone unless explicitly
  accepted with evidence.

`hyperfine` measures process startup. A deterministic loopback server measures
adapter overhead. Shared CI trends are informational unless runner stability is
proven; release gates run on identified reference machines.

## Scope boundaries

Out of v2: daemon/service installation, desktop notifications, Windows, a
full-screen TUI, dynamic provider plugins, mail sending, v1 syntax aliases,
automatic v1 state import, publishing, and release execution.

## Definition of done

All requirements have deterministic evidence; both providers work through the
same core; performance and safety gates pass; macOS/Linux artifacts install and
run; docs match behavior; milestone commits are coherent and local; blocking
review findings are fixed; residual risks and publication state are explicit.
