# Correspondence

A personal workspace for drafting emails and syncing conversation threads from Gmail (and eventually Protonmail).

## Purpose

- Sync email threads from Gmail by label into local Markdown files
- Draft and refine outgoing emails with Claude's assistance
- Maintain a readable, version-controlled record of correspondence
- Push distilled intelligence (tags, routing rules, contact metadata) to Cloudflare for email routing

## Tech Stack

- **Runtime**: Python 3.12+ via `uv`
- **Linter/formatter**: `ruff`
- **Type checker**: `ty`
- **Types/serialization**: `msgspec` (Struct instead of dataclasses)
- **Storage**: Markdown files (one file per conversation thread)
- **Email sources**: Gmail (via Gmail API), Protonmail (planned)
- **Cloudflare** (routing layer): TypeScript Workers reading from D1/KV populated by Python

## Project Structure

```
correspondence/
  AGENTS.md                      # Project instructions (CLAUDE.md symlinks here)
  pyproject.toml
  .env                          # OAuth credentials and config (gitignored)
  .gitignore
  .claude/
    skills/
      email/
        SKILL.md                # Email drafting & management skill
        find_unanswered.py      # Find threads needing a reply
  src/
    sync/
      __init__.py
      gmail.py                  # Gmail API sync logic
      types.py                  # msgspec Structs (Thread, Message, etc.)
      auth.py                   # One-time OAuth flow
    draft/
      __init__.py
      helpers.py                # Utilities for composing drafts
    cloudflare/
      __init__.py
      push.py                   # Push intelligence to Cloudflare D1/KV
  conversations/                # Synced threads (gitignored — private)
    [label]/
      [YYYY-MM-DD]-[subject].md
  drafts/                       # Outgoing email drafts
    [YYYY-MM-DD]-[subject].md
```

## Writing Voice

See `CLAUDE.local.md` / `AGENTS.local.md` for personal voice guidelines (gitignored).

## Safety Rules

- **Never send email directly.** Always save as a Gmail draft for review first.
- **Never guess at intent.** If the right response is unclear, ask rather than assume.
- **Never share conversation content** outside this local environment (no third-party APIs) unless explicitly instructed.

## Environment Setup

Copy `.env.example` to `.env` and fill in credentials:

```sh
cp .env.example .env
uv sync
```

Required variables in `.env`:

```
GMAIL_CLIENT_ID=
GMAIL_CLIENT_SECRET=
GMAIL_REDIRECT_URI=http://localhost:3000/oauth/callback
GMAIL_REFRESH_TOKEN=
GMAIL_USER_EMAIL=                # Your Gmail address (used to detect unanswered threads)
GMAIL_SYNC_LABELS=correspondence,follow-up   # comma-separated Gmail labels to sync

# Cloudflare (optional — for routing intelligence)
CLOUDFLARE_ACCOUNT_ID=
CLOUDFLARE_API_TOKEN=
CLOUDFLARE_D1_DATABASE_ID=
```

### Gmail OAuth Setup

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a project → Enable the **Gmail API**
3. Create OAuth 2.0 credentials (Desktop app type)
4. Download the credentials JSON and extract `client_id` and `client_secret` into `.env`
5. Run the auth flow once to obtain a refresh token:
   ```sh
   uv run sync-auth
   ```

## Commands

```sh
uv sync                                         # Install dependencies
uv run sync-auth                                # One-time Gmail OAuth setup
uv run sync-gmail                               # Sync labeled Gmail threads to conversations/
uv run .claude/skills/email/find_unanswered.py  # List threads needing a reply
uv run src/cloudflare/push.py                   # Push intelligence to Cloudflare D1/KV
uv run ruff check .                             # Lint
uv run ruff format .                            # Format
uv run ty check                                 # Type check
```

## Workflows

### Daily email review

1. Run `uv run src/sync/gmail.py` to pull latest threads
2. Ask Claude: *"Review conversations/ and identify threads that need a response, ordered by priority"*
3. For each thread, ask Claude to draft a reply matching the voice guidelines above
4. Review and edit the draft in `drafts/`
5. When satisfied, ask Claude to save it as a Gmail draft (never send directly)

### Finding unanswered threads

```sh
uv run .claude/skills/email/find_unanswered.py
```

Lists all synced threads where the last message is not from you — i.e. threads awaiting your reply.

### Drafting a new email

Ask Claude: *"Draft an email to [person] about [topic]"* — point it at any relevant thread in `conversations/` for context.

## Gmail Sync Behavior

- Threads are fetched for each label listed in `GMAIL_SYNC_LABELS`
- Threads are written to `conversations/[label]/[YYYY-MM-DD]-[slug].md`
  - Date is derived from the most recent message in the thread
  - Slug is derived from the subject line
- Existing files are overwritten on re-sync (idempotent)
- Attachments are noted inline but not downloaded

## Cloudflare Architecture

Python handles the heavy lifting locally. Distilled intelligence is pushed to Cloudflare storage for use by a lightweight TypeScript Worker that handles email routing.

```
Gmail/Protonmail
      ↓
Python (local, uv)
  - sync threads → markdown
  - extract intelligence (tags, contact metadata, routing rules)
  - push to Cloudflare
      ↓
Cloudflare D1 / KV
  - contact importance scores
  - thread tags / inferred topics
  - routing rules
      ↓
Cloudflare Worker (TypeScript)
  - email routing decisions using intelligence from Python
```

Full conversation threads stay local. Cloudflare only receives the minimal distilled signal needed for routing.

## Conversation Markdown Format

Each synced thread is written in this format:

```markdown
# [Subject]

**Label**: [label]
**Thread ID**: [Gmail thread ID]
**Last updated**: [ISO date]

---

## [Sender Name] — [Date]

[Body text]

---

## [Reply sender] — [Date]

[Body text]
```

## Draft Format

Drafts live in `drafts/`. Filename convention: `[YYYY-MM-DD]-[slug].md`.

```markdown
# [Subject]

**To**: [recipient]
**CC**: (optional)
**Re**: (optional — link to conversation file)

---

[Draft body]
```

When asking Claude to help draft or refine an email:
- Point it at the relevant thread in `conversations/` for context
- Specify tone if it differs from the voice guidelines (formal, concise, etc.)
- Indicate any constraints (length, what to avoid, etc.)

## MCP Alternative

Instead of pre-syncing to markdown files, Claude can access Gmail live via an MCP server during a session. Options:

- **Pipedream** — hosted MCP with Gmail, Calendar, Contacts (note: data passes through Pipedream)
- **Local Python MCP server** — run a Gmail MCP server locally for fully private live access (future)

Current approach (file sync) is preferred for privacy and offline use. MCP is worth revisiting for real-time workflows.

## Conventions

- Use `uv run` for script execution, never bare `python`
- Use `msgspec.Struct` for all data types — not dataclasses or TypedDict
- Use `ruff` for linting and formatting
- Use `ty` for type checking
- Keep sync, draft, and cloudflare logic in separate subpackages
- Do not commit `.env`, `CLAUDE.local.md` / `AGENTS.local.md`, or `conversations/` (private data)
- Scripts must be runnable directly: `uv run src/sync/gmail.py`

## Future Work

- **Voice guidelines**: Analyze sent mail and fill in the Writing Voice section above
- **Protonmail sync**: Protonmail Bridge (IMAP) or Protonmail API
- **Cloudflare routing**: TypeScript Worker consuming D1/KV data pushed from Python
- **Local Gmail MCP server**: Live Gmail access during Claude sessions without Pipedream
- **Send integration**: Push approved drafts back to Gmail as drafts or send directly
- **Multi-user**: Per-user OAuth credential flow documented here when shared with another developer

## .gitignore

```
.env
CLAUDE.local.md
AGENTS.local.md
conversations/
*.credentials.json
.venv/
__pycache__/
```
