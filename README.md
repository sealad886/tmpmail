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
```

`tmpmail latest` returns exactly the newest message in one inbox. `tmpmail
message new` returns all locally unseen messages. Reading acknowledges only
after output succeeds; `--peek` never acknowledges. Review mode accepts `r`,
`a`, `s`, and `q` actions followed by a message number where applicable.

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

The complete architecture, provider contracts, security choices, and delivery
evidence live under [`docs/`](docs/).

## License

[MIT](LICENSE)
