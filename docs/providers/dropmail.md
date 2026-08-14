# DropMail provider contract

Refreshed against official DropMail documentation and `schema.graphql` on
2026-08-14.

- GraphQL endpoint: `https://dropmail.me/api/graphql/${AUTH_TOKEN}`.
- Current free tokens begin with `af_`. Legacy arbitrary tokens are disabled.
- Tokens issued before 5 April 2026 were affected by a provider defect and must
  be replaced, not merely renewed.
- Sessions live for at least ten minutes and extend when accessed or when mail
  arrives. Access through `message` commands or foreground watch is explicit.
- `introduceSession` creates a server-generated address. DropMail cannot choose
  a custom local part, so tmpmail rejects that request before network access.
- Address ID, session ID, address, restore key, expiry, and token are stored in
  the OS credential store. Normal config exposes only `dropmail_ready`.
- `domains` filters out non-API and broken domains. Rotating-domain expiry is
  preserved by the provider but never presented as guaranteed deliverability.
- Every GraphQL `errors` array is treated as failure even when partial `data`
  exists. Session expiry, token expiry, retired domains, and rate limits are
  normalized. CAPTCHA metadata is redacted and automated solving is disabled.
- Attachment fields follow the current schema: `name`, `mime`, `rawSize`, and
  `downloadUrl`.

Official references:

- <https://dropmail.me/api/>
- <https://dropmail.me/api/schema.graphql>

No live token was stored and no live session was created during deterministic
implementation. Live canary remains a separately authorized external-write
acceptance gate.
