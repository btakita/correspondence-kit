# Email Skill

Claude Code skill for managing email correspondence using locally synced threads.

## Prerequisites

- `corky` installed and on PATH
- `mail/.corky.toml` configured with at least one email account
- `corky sync` run at least once to populate `mail/conversations/`
- `mail/voice.md` for writing style

## Data paths

| Path | Purpose |
|---|---|
| `mail/conversations/*.md` | Synced email threads (one file per thread) |
| `mail/drafts/*.md` | Outgoing drafts being worked on |
| `mail/contacts/{name}/AGENTS.md` | Per-contact context for tone and topics |
| `mail/manifest.toml` | Thread index by labels, accounts, contacts |

## Commands

```sh
corky find-unanswered             # List threads awaiting a reply
corky validate-draft FILE         # Validate draft markdown format
corky sync                        # Re-sync threads from all accounts
corky list-folders ACCOUNT        # List IMAP folders for an account
corky push-draft FILE             # Save draft to email provider
corky push-draft FILE --send      # Send via SMTP (owner only)
```

## Draft format

See the main [README.md](../../../README.md#draft-format) for the draft markdown format
and status values (`draft` -> `review` -> `approved` -> `sent`).

## Legacy files

- `find_unanswered.py` — Python predecessor of `corky find-unanswered`. Requires
  `.env` with `GMAIL_USER_EMAIL`. Superseded by the Rust CLI command.