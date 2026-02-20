# Versions

## 0.4.1

Add-label command and audit-docs fixes.

- `corrkit add-label LABEL --account NAME`: Add a label to an account's sync config via text-level TOML edit (preserves comments).
- `contact-add` integration: `--label` + `--account` automatically adds label to account sync config.
- audit-docs: Fix tree parser for symlink-to-directory entries.
- SKILL.md: Updated to reflect flat conversation directory, contacts, manifest.

## 0.4.0

Flat conversation directory, contacts, manifest.

- **Flat conversations**: All threads in `correspondence/conversations/` as `[slug].md`. No account or label subdirectories. Consolidates correspondence across multiple email accounts into one directory.
- **Immutable filenames**: Slug derived from subject on first write, never changes. Thread identity tracked by `**Thread ID**` metadata.
- **File mtime**: Set to last message date via `os.utime()`. `ls -t` sorts by thread activity.
- **Multi-source accumulation**: Threads fetched from multiple labels or accounts accumulate all sources in `**Labels**` and `**Accounts**` metadata.
- **Orphan cleanup**: `--full` sync deletes files not touched during the run.
- **manifest.toml**: Generated after sync. Indexes threads by labels, accounts, contacts, and last-updated date.
- **Contacts**: `contacts.toml` maps contacts to email addresses. Per-contact `AGENTS.md` in `correspondence/contacts/{name}/` provides drafting context. `corrkit contact-add` scaffolds new contacts.
- **tomli-w**: Added as dependency for TOML writing.
- Backward-compatible parsing of legacy `**Label**` format.

## 0.3.0

IMAP polling daemon.

- `corrkit watch` polls IMAP on an interval and syncs automatically.
- Configurable poll interval and desktop notifications via `accounts.toml` `[watch]` section.
- systemd and launchd service templates.

## 0.2.1

Maintenance release.

- CI workflow: ty, ruff, pytest on push and PR.

## 0.2.0

Collaborator tooling and multi-account support.

- `collab-reset` command: pull, regenerate templates, commit and push.
- Reframed docs as human-and-agent friendly.
- `account:label` scoped routing for collaborators.
- `list-folders` command and self-signed cert support.
- Multi-account IMAP support via `accounts.toml` with provider presets.
- Collaborator tooling: `collab-add`, `collab-sync`, `collab-remove`, `find-unanswered`, `validate-draft`.
- Multi-collaborator architecture with submodule-based sharing.

## 0.1.0

Renamed to corrkit. Unified CLI dispatcher.

- `corrkit` CLI with subcommands.
- `push-draft` command to create drafts or send emails from markdown.
- Incremental IMAP sync with `--full` option.
- Gmail sync workspace with drafting support.
