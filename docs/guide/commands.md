# Commands

All commands are available through the `corky` CLI.

## General

```sh
corky --help                    # Show all commands
corky help [FILTER]             # Show command reference (optional filter)
corky init --user EMAIL        # Initialize in current directory
corky init --user EMAIL /path # Initialize at specific path
corky install-skill email     # Install the email agent skill
```

### init

```
corky init --user EMAIL [PATH] [--with-skill] [--provider PROVIDER]
           [--password-cmd CMD] [--labels LABEL,...] [--github-user USER]
           [--name NAME] [--mailbox-name NAME] [--sync] [--force]
```

Creates `{path}/mail/` with directory structure, `.corky.toml`, and `voice.md`. If inside a git repo, adds `mail` to `.gitignore`. Registers the project dir as a named mailbox in app config.

- `--provider`: `gmail` (default), `protonmail-bridge`, `imap`
- `--labels`: default `correspondence` (comma-separated)
- `--with-skill`: install the email skill to `.claude/skills/email/`
- `--force`: overwrite existing config
- `--sync`: run sync after init

### install-skill

```
corky install-skill NAME
```

Install an agent skill into the current directory. Currently supported: `email`.

## Sync

```sh
corky sync                     # Incremental IMAP sync (all accounts)
corky sync full                # Full re-sync (ignore saved state)
corky sync account personal    # Sync one account
corky sync routes              # Apply routing rules to existing conversations
corky sync mailbox [NAME]      # Push/pull shared mailboxes
```

### sync-auth

```sh
corky sync-auth
```

Gmail OAuth setup. Requires `credentials.json` from Google Cloud Console.

## Email

```sh
corky list-folders [ACCOUNT]   # List IMAP folders for an account
corky push-draft mail/drafts/FILE.md         # Save a draft via IMAP
corky push-draft mail/drafts/FILE.md --send  # Send via SMTP
corky add-label LABEL --account NAME         # Add a label to sync config
corky unanswered                             # Find threads awaiting a reply (all scopes)
corky unanswered .                           # Root conversations only
corky unanswered NAME                        # Specific mailbox only
corky validate-draft FILE                    # Validate draft markdown files
```

### push-draft

Default: creates a draft via IMAP APPEND to the drafts folder.
`--send`: sends via SMTP. Requires Status to be `review` or `approved`. After sending, updates Status to `sent`.

Account resolution:
1. `**Account**` field → match by name in `.corky.toml`
2. `**From**` field → match by email address
3. Fall back to default account

## Contacts

```sh
corky contact-add NAME --email EMAIL [--email EMAIL2] [--label LABEL] [--account ACCT]
```

Creates `mail/contacts/{name}/` with `AGENTS.md` template and `CLAUDE.md` symlink. Updates `.corky.toml`.

## Mailboxes

```sh
corky mailbox add NAME --label LABEL [--name NAME] [--github] [--pat]
corky mailbox sync [NAME]                   # Push/pull shared mailboxes
corky mailbox status                        # Check mailbox status
corky mailbox list                          # List registered mailboxes
corky mailbox remove NAME [--delete-repo]   # Remove a mailbox
corky mailbox rename OLD NEW [--rename-repo] # Rename a mailbox
corky mailbox reset [NAME] [--no-sync]      # Regenerate templates
```

All mailbox commands accept the `mb` alias (e.g. `corky mb add`).

## Watch

```sh
corky watch                    # Poll IMAP and sync on an interval
corky watch --interval 60      # Override poll interval (seconds)
```

## Global flags

```sh
corky --mailbox NAME <command>  # Use a specific mailbox for any command
```

## Development

```sh
corky audit-docs               # Audit instruction files for staleness
```
