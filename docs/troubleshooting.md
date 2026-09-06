# Troubleshooting and privacy

## No active inbox

Create one with `tmpmail new`, or select a saved inbox with `tmpmail use NAME`.

## Credential-store unavailable

tmpmail prefers macOS Keychain or Linux Secret Service. Fix the platform
credential service where possible. To accept weaker local protection explicitly,
run `tmpmail config set secrets.file_fallback true`; future failed keyring writes
then use a `0600` `secrets.json` beside state. Back up and protect that file like
any password store. `config show` never includes its values.

## Provider failure or rate limit

Creation may fall back after transient failures unless `--provider` pins one
backend. Existing inboxes never move between providers. Use `--verbose` for
safe context and `--json` for structured error codes. CAPTCHA metadata causes
backoff; tmpmail never attempts CAPTCHA solving.

## HTML and attachments

Message content is untrusted. Normal reads use sanitized terminal text.
`--html` creates an escaped inert cache document; `--open` is an explicit
per-read action, while `config set browser true` is the persistent opt-in.
Attachments are never opened. Existing paths require `--force`, symbolic links
are refused, and downloads exceeding `max_attachment_bytes` fail safely.

## State recovery

Before a schema migration, tmpmail writes a sibling `state.json.v1.bak` backup.
Unsupported newer schemas fail with exit 3. Do not hand-edit live state while a
tmpmail process is running. Preserve config, state, backup, and secret storage
together when diagnosing recovery.

## Privacy boundaries

Message bodies are not cached by default. Cached HTML remains on local disk
until the user removes it. Provider retention and deletion semantics differ;
`provider info PROVIDER` reports capabilities. Diagnostics redact passwords,
JWTs, API tokens, restore keys, credential-bearing URLs, and message bodies.
