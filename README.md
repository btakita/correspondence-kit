# Correspondence

Sync Gmail threads to local Markdown files, draft replies with AI assistance, and push routing intelligence to Cloudflare.

## Setup

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

```sh
uv run sync-gmail                               # Sync labeled threads to conversations/
uv run .claude/skills/email/find_unanswered.py  # List threads needing a reply
```

Synced threads are written to `conversations/[label]/[YYYY-MM-DD]-[slug].md`.

## Development

```sh
uv run ruff check .   # Lint
uv run ruff format .  # Format
uv run ty check       # Type check
```

## AI agent instructions

Project instructions live in `AGENTS.md` (symlinked as `CLAUDE.md`). Personal overrides go in `CLAUDE.local.md` / `AGENTS.local.md` (gitignored).
