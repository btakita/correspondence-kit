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
corky unanswered                  # List threads awaiting a reply
corky draft validate FILE         # Validate draft markdown format
corky draft validate              # Validate all drafts (root + mailboxes)
corky sync                        # Re-sync threads from all accounts
corky list-folders ACCOUNT        # List IMAP folders for an account
corky draft push FILE             # Save draft to email provider
corky draft push FILE --send      # Send via SMTP (owner only)
```

## Draft format

See the main [README.md](../../../README.md#draft-format) for the draft markdown format
and status values (`draft` -> `review` -> `approved` -> `sent`).

## Legacy files

- `find_unanswered.py` — Python predecessor of `corky unanswered`. Requires
  `.env` with `GMAIL_USER_EMAIL`. Superseded by the Rust CLI command.