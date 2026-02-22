# Configuration

All config lives inside the data directory (`mail/`).

## .corky.toml

The main configuration file at `mail/.corky.toml`:

```toml
[owner]
github_user = "username"
name = "Display Name"

[accounts.personal]
provider = "gmail"                      # gmail | protonmail-bridge | imap
user = "you@gmail.com"
password = ""                           # Inline password (not recommended)
password_cmd = ""                       # Shell command to retrieve password
labels = ["correspondence"]
imap_host = ""                          # Auto-filled by provider preset
imap_port = 993
imap_starttls = false
smtp_host = ""
smtp_port = 465
drafts_folder = "Drafts"
sync_days = 3650                        # How far back to sync
default = false                         # Mark one account as default

[contacts.alex]
emails = ["alex@example.com"]
labels = ["correspondence"]
account = "personal"

[routing]
for-alex = ["mailboxes/alex"]
shared = ["mailboxes/alice", "mailboxes/bob"]

[mailboxes.alex]
auto_send = false

[watch]
poll_interval = 300                     # Seconds between polls
notify = false                          # Desktop notifications
```

## Account providers

Provider presets fill in IMAP/SMTP connection defaults:

| Field | `gmail` | `protonmail-bridge` | `imap` (generic) |
|---|---|---|---|
| imap_host | imap.gmail.com | 127.0.0.1 | (required) |
| imap_port | 993 | 1143 | 993 |
| imap_starttls | false | true | false |
| smtp_host | smtp.gmail.com | 127.0.0.1 | (required) |
| smtp_port | 465 | 1025 | 465 |
| drafts_folder | [Gmail]/Drafts | Drafts | Drafts |

Any preset value can be overridden per-account.

## Password resolution

1. `password` field (inline string)
2. `password_cmd` (shell command, captures stdout, strips trailing whitespace)
3. Error if neither set

## Label scoping

Use `account:label` syntax to bind a label to a specific account (e.g. `"proton-dev:INBOX"`). Plain labels match all accounts.

## Data directory resolution

The data directory is resolved at runtime in this order:
1. `mail/` directory in current working directory
2. `CORKY_DATA` environment variable
3. App config mailbox (named mailboxes)
4. `~/Documents/mail` (fallback)

## App config

Platform-specific location for named mailboxes:
- Linux: `~/.config/corky/config.toml`
- macOS: `~/Library/Application Support/corky/config.toml`
- Windows: `%APPDATA%/corky/config.toml`

```toml
default_mailbox = "personal"

[mailboxes.personal]
path = "~/Documents/mail"

[mailboxes.work]
path = "~/work/mail"
```
