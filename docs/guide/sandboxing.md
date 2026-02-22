# Sandboxing

Most AI email tools (OpenClaw, etc.) require OAuth access to your entire account. Once authorized, the agent can read every message, every contact, every thread — and you're trusting the service not to overreach.

Corky inverts this. You control what any agent or collaborator can see:

1. **You label threads in your email client.** Only threads you explicitly label get synced locally.
2. **Labels route to scoped views.** Each mailbox gives the collaborator/agent a directory containing only the threads labeled for them — nothing else.
3. **Credentials never leave your machine.** Config lives inside `mail/` (your private data repo). Agents draft replies in markdown; only you can push to your email.

## Blast radius

An agent added with `corky mailbox add assistant --label for-assistant` can only see threads you've tagged `for-assistant`. It can't see your other conversations, your contacts, or other collaborators' repos. If the agent is compromised, the blast radius is limited to the threads you chose to share.

## Multi-account

This works across multiple email accounts — Gmail, Protonmail, self-hosted — each with its own labels and routing rules, all funneling through the same scoped mailbox model.

## Collaborator workflow

Each collaborator — human or agent — gets a scoped directory with:

```
mailboxes/{name}/
  AGENTS.md          # Full instructions: formats, commands, status flow
  CLAUDE.md          # Symlink for Claude Code auto-discovery
  README.md          # Quick-start guide
  voice.md           # Writing style guidelines
  contacts/          # Per-contact context for drafting
  conversations/     # Synced threads (read-only for the collaborator)
  drafts/            # Where the collaborator writes replies
```

The collaborator reads conversations, drafts replies following the documented format, validates with `corky validate-draft`, and pushes. The owner reviews and sends.
