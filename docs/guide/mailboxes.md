# Mailboxes

Share specific email threads with people or AI agents via scoped directories or GitHub repos.

## Adding a mailbox

```sh
# Plain directory (default)
corky mailbox add alex --label for-alex --name "Alex"

# With GitHub submodule
corky mailbox add alex --label for-alex --name "Alex" --github

# AI agent (uses a PAT instead of GitHub invite)
corky mailbox add assistant-bot --label for-assistant --pat

# Bind all labels to one account
corky mailbox add alex --label for-alex --account personal
```

This creates a scoped directory under `mailboxes/{name}/`. With `--github`, it also creates a private GitHub repo and adds it as a submodule.

## Multiple mailboxes

Manage multiple correspondence directories (personal, work, etc.) with named mailboxes:

```sh
# Init registers a mailbox automatically
corky init --user you@gmail.com
corky init --user work@company.com ~/work/project --mailbox-name work

# List registered mailboxes
corky mailbox list

# Use a specific mailbox for any command
corky --mailbox work sync
corky --mailbox personal mailbox status
```

Mailboxes are stored in platform-specific app config. The first mailbox added becomes the default.

## Routing

Labels in `.corky.toml` route to mailbox directories:

```toml
[routing]
for-alex = ["mailboxes/alex"]
shared = ["mailboxes/alice", "mailboxes/bob"]
```

One label can fan-out to multiple mailboxes. Use `account:label` syntax for per-account scoping.

## Daily workflow

```sh
# 1. Sync emails — shared labels route to mailboxes/{name}/conversations/
corky sync

# 2. Push synced threads to mailbox repos & pull their drafts
corky mailbox sync

# 3. Check what's pending without pushing
corky mailbox status

# 4. Review a collaborator's draft and push it as an email draft
corky push-draft mailboxes/alex/drafts/2026-02-19-reply.md
```

## What collaborators can do

- Read conversations labeled for them
- Draft replies in `mailboxes/{name}/drafts/` following the format in AGENTS.md
- Run `corky find-unanswered` and `corky validate-draft` in their repo
- Push changes to their shared repo

## What only you can do

- Sync new emails (`corky sync`)
- Push synced threads to mailbox repos (`corky mailbox sync`)
- Send emails (`corky push-draft --send`)
- Change draft Status to `sent`

## Managing mailboxes

```sh
corky mailbox remove alex                    # Remove a mailbox
corky mailbox remove alex --delete-repo      # Also delete GitHub repo
corky mailbox rename old-name new-name       # Rename a mailbox
corky mailbox reset [NAME]                   # Regenerate template files
```
