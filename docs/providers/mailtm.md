# Mail.tm provider contract

Refreshed against official Mail.tm documentation on 2026-08-14.

- API base: `https://api.mail.tm`.
- Public domain discovery and account creation require no bearer token.
- Account access uses a JWT obtained from `/token` with mailbox address and
  password. tmpmail stores both in the OS credential store and retries once
  with a fresh JWT after an authentication failure.
- General quota: 8 queries per second per IP address.
- Message collections are paginated with at most 30 messages per page.
- Full message resources contain text, HTML, and attachment metadata.
- Message acknowledgement uses `PATCH /messages/{id}`.
- Account removal uses `DELETE /accounts/{id}`.
- Mail.tm requires a visible link back. Attribution appears in README, generated
  help, and `tmpmail provider info mailtm`.

Official references:

- <https://docs.mail.tm/>
- <https://docs.mail.tm/getting-started/authentication>
- <https://docs.mail.tm/api/messages>
- <https://docs.mail.tm/api/webhooks>

No live account was created during contract refresh or deterministic testing.
Live create-deliver-read-attachment-delete canary remains explicit external-
write acceptance gate.
