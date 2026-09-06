# Migrating from tmpmail v1

tmpmail 2 replaces the 1secmail shell script with an authenticated Rust CLI.
Command syntax and saved addresses are intentionally not migrated.

1. Build or install the v2 binary and confirm `tmpmail --version` reports
   `2.0.0`.
2. Run `tmpmail new` to create a new authenticated Mail.tm inbox.
3. Recreate additional named inboxes with `tmpmail new NAME`.
4. If desired, authenticate DropMail with `tmpmail provider auth dropmail`.
5. Update scripts to use canonical resource commands and `--json` envelopes.
6. Remove the historical `/tmp/tmpmail/email_address` only when no old tmpmail
   process or script still needs it. v2 never reads or deletes that file.

The old address alone cannot restore Mail.tm or DropMail authentication. A
malformed or provider-generated 1secmail response may also have been stored in
that file, so importing it would create misleading state.

## Automation changes

- Use `tmpmail message list`, `message read`, and `message latest`.
- `tmpmail latest` means one newest message; `message new` means every locally
  unseen message.
- Use `--json`; do not parse human tables.
- Treat exit `0` with `meta.fallback_used: true` as successful creation through
  another configured provider.
- No v1 command-compatibility layer is provided.

## Rollback

The shell script and v2 binary can coexist under different executable names.
Rolling back the executable does not make v2 inbox state usable by v1. Keep a
copy of the v2 config/state directory before experimenting with another build.
An older v2 binary refuses a newer unsupported state schema instead of changing
it.
