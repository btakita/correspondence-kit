# Correspondence Kit

Sync Gmail threads to local Markdown files, draft replies with AI assistance, and push routing intelligence to Cloudflare.

## Install

Requires Python 3.12+ and [uv](https://docs.astral.sh/uv/).

```sh
cp .env.example .env   # fill in credentials
uv sync
```

### `.env` configuration

| Variable | Required | Description |
|---|---|---|
| `GMAIL_USER_EMAIL` | yes | Your Gmail address |
| `GMAIL_APP_PASSWORD` | yes | [App password](https://myaccount.google.com/apppasswords) |
| `GMAIL_SYNC_LABELS` | yes | Comma-separated Gmail labels to sync |
| `GMAIL_SYNC_DAYS` | no | How far back to sync (default: 3650) |
| `CLOUDFLARE_ACCOUNT_ID` | no | For routing intelligence push |
| `CLOUDFLARE_API_TOKEN` | no | For routing intelligence push |
| `CLOUDFLARE_D1_DATABASE_ID` | no | For routing intelligence push |

## Usage

All commands are available through the `corrkit` CLI:

```sh
corrkit --help                    # Show all commands
corrkit sync-gmail                # Sync labeled threads to conversations/
corrkit sync-gmail --full         # Full re-sync (ignore saved state)
corrkit push-draft drafts/FILE.md # Save a draft to Gmail
corrkit push-draft drafts/FILE.md --send  # Send email
corrkit collab-add NAME --label LABEL     # Add a collaborator
corrkit collab-sync [NAME]        # Push/pull shared submodules
corrkit collab-status             # Check for pending changes
corrkit collab-remove NAME        # Remove a collaborator
corrkit audit-docs                # Audit instruction files for staleness
corrkit help                      # Show command reference
```

Run with `uv run corrkit <subcommand>` if the package isn't installed globally.

Synced threads are written to `conversations/[label]/[YYYY-MM-DD]-[slug].md`.

## Development

```sh
uv run pytest             # Run tests
uv run ruff check .       # Lint
uv run ruff format .      # Format
uv run ty check           # Type check
uv run poe precommit      # Run ty + ruff + tests
```

## AI agent instructions

Project instructions live in `AGENTS.md` (symlinked as `CLAUDE.md`). Personal overrides go in `AGENTS.override.md` / `CLAUDE.local.md` (gitignored).
