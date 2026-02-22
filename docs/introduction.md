# Corky

> **Alpha software.** Expect breaking changes between minor versions. See the [changelog](reference/changelog.md) for migration notes.

Corky consolidates conversations from multiple email accounts into a single flat directory of Markdown files. Draft replies with AI assistance. Share scoped threads with collaborators via git.

Corky syncs threads from any IMAP provider (Gmail, Protonmail Bridge, self-hosted) into `mail/conversations/` — one file per thread, regardless of source. A thread that arrives via both Gmail and Protonmail merges into one file. Labels, accounts, and contacts are metadata, not directory structure. Slack and social media sources are planned.

## Why Corky?

Most AI email tools require OAuth access to your entire account. Once authorized, the agent can read every message, every contact, every thread — and you're trusting the service not to overreach.

Corky inverts this:

1. **You label threads in your email client.** Only threads you explicitly label get synced locally.
2. **Labels route to scoped views.** Each mailbox gives a collaborator or agent a directory containing only the threads labeled for them — nothing else.
3. **Credentials never leave your machine.** Config lives inside `mail/` (your private data repo). Agents draft replies in markdown; only you can push to your email.
4. **Reduced context poisoning.** Agents only see the threads you route to them — not your entire inbox. A focused context means fewer irrelevant details leaking into prompts, better signal-to-noise, and more accurate replies.
5. **Per-contact context.** Each mailbox ships with `AGENTS.md` (or `CLAUDE.md`), `voice.md`, and relationship-specific instructions — so agents adapt their tone and knowledge to each collaborator automatically.

## Designed for humans and agents

Corky is built around files, CLI commands, and git — interfaces that work equally well for humans and AI agents. No GUIs, no OAuth popups, no interactive prompts.

- **Everything is files.** Threads are Markdown. Config is TOML. Drafts are Markdown.
- **CLI is the interface.** Every operation is a single `corky` command. Scriptable and composable.
- **Single-binary install.** One `curl | sh` gives collaborators `corky find-unanswered` and `corky validate-draft`.
- **Self-documenting repos.** Each shared repo ships with `AGENTS.md`, `voice.md`, and a `README.md`.

## Tech Stack

- **Language**: Rust (2021 edition)
- **CLI**: `clap` (derive macros)
- **Serialization**: `serde` + `toml` / `toml_edit` / `serde_json`
- **IMAP**: `imap` + `native-tls`
- **Email parsing**: `mailparse`
- **SMTP**: `lettre`
- **Dates**: `chrono`
- **Storage**: Markdown files (one flat directory, one file per conversation thread)
- **Sources**: Any IMAP provider (Gmail, Protonmail Bridge, generic IMAP); Slack and social media planned
