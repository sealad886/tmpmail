# tmpmail

Fast, friendly temporary email from your terminal. `tmpmail` 2 is a native Rust
CLI for macOS and Linux with named inboxes, stable JSON, safe message rendering,
attachments, and curated Mail.tm and DropMail backends.

> Version 2 is under active development. The repository is not yet published as
> a v2 release.

Mail.tm functionality is powered by [Mail.tm](https://mail.tm). DropMail use
requires your own current `af_…` API token.

## Build

Rust 1.88 or newer is required.

```console
cargo build --release
./target/release/tmpmail --help
```

The ordinary CLI does not require `curl`, `jq`, or a terminal web browser.
Clipboard and explicit browser-opening actions use the platform tools `pbcopy`
and `open` on macOS, or `xclip` and `xdg-open` on Linux.

## Quick start

```console
tmpmail new
tmpmail new work
tmpmail inboxes
tmpmail use work
tmpmail
tmpmail latest
tmpmail read MESSAGE
tmpmail message read MESSAGE --peek
tmpmail new-mail
tmpmail message new --all-inboxes
tmpmail message new --review
tmpmail copy
tmpmail watch work
tmpmail watch --all
tmpmail config set inbox.work.watch true
```

`tmpmail latest` returns exactly the newest message in one inbox. `tmpmail
message new` returns all locally unseen messages. Reading acknowledges only
after output succeeds; `--peek` never acknowledges. Review mode accepts `r`,
`a`, `s`, and `q` actions followed by a message number where applicable.
Use `tmpmail read MESSAGE --raw` when MIME source is needed; terminal controls
remain stripped in human output and JSON remains structurally escaped.

## Providers and fallback

Mail.tm is the default. Configure DropMail without exposing the token in process
arguments:

```console
tmpmail provider auth dropmail
printf '%s\n' "$DROPMAIL_TOKEN" | tmpmail provider auth dropmail --token-stdin
tmpmail provider list
tmpmail provider domains
```

Creation can fall back to another configured provider after a transient outage,
timeout, or rate limit. Existing inboxes never change provider. Pin or exclude a
provider with `--provider` and `--no-provider`. Fallback attempts share the
command timeout and are disclosed on stderr and in JSON metadata.

## Safe content and attachments

Message content is untrusted. Terminal output strips control and bidi-override
characters. Plain text is preferred; HTML-only messages are converted to text
in-process. `read --html` writes an inert escaped representation to the cache;
`--html --open` explicitly opens it in the platform browser.
Setting `config set browser true` makes future `--html` reads open the cached
document through the platform browser without shell interpolation.

```console
tmpmail attachment list MESSAGE
tmpmail attachment save MESSAGE ATTACHMENT
tmpmail attachment save MESSAGE ATTACHMENT --force
```

Downloads are streamed, size-limited, written through an incomplete sibling
file, and never opened or executed. Path traversal, reserved names, symbolic
link replacement, and unrequested overwrite are refused. The default limit is
25 MiB and can be changed with `config set max_attachment_bytes BYTES`.

## Automation

Use `--json` for the versioned `tmpmail/v1` envelope. Data is emitted on stdout;
warnings and fallback notices use stderr. JSON mode never prompts or emits ANSI
sequences. Stable exit codes are documented in
[the v2 design](docs/design/tmpmail-v2.md).

```console
tmpmail --json inbox list
tmpmail --json message new --all-inboxes
tmpmail config show
tmpmail doctor
```

## State and privacy

Configuration and state use platform-standard directories. Credentials are
stored in macOS Keychain or Linux Secret Service; secret values never appear in
normal configuration, JSON, or diagnostics. The old `/tmp/tmpmail/email_address`
file is deliberately not imported because it cannot restore authenticated v2
inboxes.

If no supported credential service is available, `tmpmail config set
secrets.file_fallback true` explicitly opts into a `0600` `secrets.json` file
beside local state. This is less protected than the OS credential store and is
disabled by default.

The complete architecture, provider contracts, security choices, and delivery
evidence live under [`docs/`](docs/).

Prebuilt archives are planned for macOS arm64/x86_64 and Linux arm64/x86_64.
On Linux, unpack the matching archive, copy `tmpmail` into a directory on
`PATH`, and optionally install `tmpmail.1` and shell completions. Release
archives include SHA-256 checksums, a CycloneDX SBOM, and a local provenance
record. The Homebrew formula remains a template until immutable release URLs
exist; no formula or package has been published.

`tmpmail watch` runs only in the foreground. It establishes an initial baseline
unless `--existing` is supplied, polls providers concurrently in bounded
batches, persists a cursor after each successfully printed event, and exits
cleanly on Ctrl-C. It does not install a daemon or send desktop notifications.
Named arguments temporarily include those inboxes. `--all` includes only
inboxes opted in through `config set inbox.NAME.watch true`.

## License

[MIT](LICENSE)
