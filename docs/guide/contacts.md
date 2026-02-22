# Contacts

Per-contact directories give Claude context when drafting emails — relationship history, tone preferences, recurring topics.

## Adding a contact

```sh
corky contact-add alex --email alex@example.com --email alex@work.com \
  --label correspondence --account personal
```

This creates `mail/contacts/alex/` with an AGENTS.md template (+ CLAUDE.md symlink) and updates `.corky.toml`.

## Contact context

Edit `mail/contacts/{name}/AGENTS.md` with:
- **Relationship**: How you know this person, shared history
- **Tone**: Communication style overrides (defaults to voice.md)
- **Topics**: Recurring subjects, current projects
- **Notes**: Freeform context — preferences, pending items, important dates

## Contact metadata

Contact metadata in `.corky.toml` maps names to email addresses (for manifest tagging, not sync routing):

```toml
[contacts.alex]
emails = ["alex@example.com", "alex@work.com"]
labels = ["correspondence"]
account = "personal"
```
