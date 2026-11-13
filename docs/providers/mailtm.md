# Mail.tm provider contract

Refreshed against official Mail.tm documentation on 2026-11-13.

- API base: `https://api.mail.tm`.
- Public domain discovery and account creation require no bearer token.
- API access is free, with no API key or paid tier.
- Account access uses a JWT obtained from `/token` with mailbox address and
  password. tmpmail stores both in the OS credential store and retries once
  with a fresh JWT after an authentication failure.
- General quota: 8 queries per second per IP address.
- Message collections are paginated with at most 30 messages per page.
- Full message resources contain text, HTML, and attachment metadata.
- Attachment `size` is not a reliable decoded byte count: the live canary
  reported 1 for a 67-byte download. Preserve it as provider-reported metadata;
  use HTTP Content-Length and actual streamed bytes for transfer validation.
- Message acknowledgement uses `PATCH /messages/{id}` with
  `Content-Type: application/merge-patch+json`. The live API rejects ordinary
  `application/json` with HTTP 415.
- Account removal uses `DELETE /accounts/{id}`.
- Mail.tm requires a visible link back. Attribution appears in README, generated
  help, and `tmpmail provider info mailtm`.

Official references:

- <https://docs.mail.tm/>
- <https://docs.mail.tm/getting-started/authentication>
- <https://docs.mail.tm/api/messages>
- <https://docs.mail.tm/api/webhooks>

The authorized November 13, 2026 live canary passed account creation, Gmail
text/HTML delivery, raw reads, exact attachment bytes, acknowledgement,
invalid-token renewal, and restart checks. The test account, local inbox, and
credential were removed. Rate limits were observed and respected with backoff.
