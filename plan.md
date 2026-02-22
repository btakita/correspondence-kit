# Contact Enrichment — Plan

## Overview

Two new features plus an email skill update:
1. `corky contact add --from SLUG` — create a contact from a conversation
2. `corky contact info NAME` — aggregate and display everything known about a contact
3. Email skill update — teach the agent to use these commands + web search

The `contact` command becomes a subcommand group (like `draft`, `mailbox`).
The existing `contact-add` stays as a hidden backward-compatible alias.

## CLI Design

```
# New subcommand group
corky contact add NAME --email EMAIL [--label LABEL] [--account ACCT]
corky contact add --from SLUG [--name NAME] [--account ACCT]
corky contact info NAME

# Hidden backward-compatible alias (unchanged behavior)
corky contact-add NAME --email EMAIL [--label LABEL] [--account ACCT]
```

### `contact add` arguments

```rust
#[derive(Subcommand)]
pub enum ContactCommands {
    /// Add a new contact
    Add {
        /// Contact name (optional with --from)
        name: Option<String>,

        /// Email address(es) — conflicts with --from
        #[arg(long = "email")]
        emails: Vec<String>,

        /// Create from a conversation slug
        #[arg(long, conflicts_with = "emails")]
        from: Option<String>,

        /// Conversation label(s)
        #[arg(long = "label")]
        labels: Vec<String>,

        /// Bind to a specific account
        #[arg(long, default_value = "")]
        account: String,
    },

    /// Show contact info
    Info {
        /// Contact name
        name: String,
    },
}
```

Validation (in the handler, not clap):
- `--from` without `--email`: OK (derive from conversation)
- `--email` without `--from`: requires `name` positional
- Both `--from` and `--email`: clap `conflicts_with` prevents this
- `--from` with multiple senders and no `--name`: print candidates, bail

### `contact info` output

```
Contact: alice

  Emails: alice@example.com, alice@work.com
  Labels: correspondence
  Account: personal

--- AGENTS.md ---
# Contact: alice
(full content)

--- Threads (3) ---
  2025-02-15  re-project-update       Re: Project Update
  2025-02-10  meeting-follow-up       Meeting Follow-Up
  2025-01-28  intro                   Introduction

Last activity: Sat, 15 Feb 2025
```

## Implementation

### 1. `src/cli.rs` — add `ContactCommands` enum and `Contact` subcommand

Add `ContactCommands` enum (above). Add to `Commands`:

```rust
/// Contact commands
#[command(subcommand)]
Contact(ContactCommands),
```

Keep existing `ContactAdd` variant but mark hidden:
```rust
#[command(hide = true)]
ContactAdd { ... }  // unchanged
```

### 2. `src/contact/mod.rs` — add modules

```rust
pub mod add;
pub mod from_conversation;
pub mod info;
```

### 3. `src/contact/add.rs` — extract `generate_agents_md` + add enriched variant

Make `generate_agents_md(name)` public (rename to `pub fn default_agents_md`).

Add a new function for enriched AGENTS.md:

```rust
pub fn enriched_agents_md(name: &str, topics: &[String]) -> String
```

This generates the same template but pre-fills the Topics section with
subjects from conversations. Format:

```markdown
## Topics

- Project Update (from conversation)
- Meeting Follow-Up (from conversation)
```

### 4. `src/contact/from_conversation.rs` — new module

Core function:

```rust
pub fn run(slug: &str, name: Option<&str>, labels: &[String], account: &str) -> Result<()>
```

Algorithm:

1. **Find conversation file** — search `conversations/{slug}.md`, then
   `mailboxes/*/conversations/{slug}.md`. Bail with helpful error if not found.

2. **Parse thread** — use `parse_thread_markdown()` from `src/sync/markdown.rs`.

3. **Load config** — `CorkyConfig` for owner account emails.

4. **Extract non-owner senders** — for each message, extract `<email>` from
   `from` field. Skip if email matches any `accounts.*.user`. Collect unique
   `(display_name, email)` pairs.

5. **Handle sender count**:
   - 0 senders: bail "No non-owner senders found in this conversation"
   - 1 sender: auto-derive name from display name (slugify), use `--name` override if given
   - 2+ senders: if `--name` given, find matching sender; otherwise print
     candidates and bail "Multiple senders found. Use --name to select one."

6. **Build contact** — `Contact { emails, labels, account }`. Labels come from
   thread metadata if `--label` not given. Account from thread metadata if
   `--account` not given (use first account).

7. **Generate enriched AGENTS.md** — call `enriched_agents_md(name, &[subject])`.

8. **Delegate to `add::run_with_agents_md()`** — shared creation logic
   (directory, symlink, save_contact, print).

### 5. `src/contact/info.rs` — new module

```rust
pub fn run(name: &str) -> Result<()>
```

Algorithm:

1. **Load contact from config** — `load_contacts(None)?`. Bail if not found.

2. **Print config section** — emails, labels, account.

3. **Print AGENTS.md** — read `contacts/{name}/AGENTS.md` if it exists.

4. **Scan manifest for threads** — load `manifest.toml`, iterate threads where
   `contacts` array contains `name`. Also scan `mailboxes/*/manifest.toml`.
   Sort by `last_updated` descending.

5. **Print thread list** — date, slug, subject per line.

6. **Print summary** — thread count, last activity date.

### 6. `src/main.rs` — dispatch

Add match arm:
```rust
Commands::Contact(cmd) => match cmd {
    ContactCommands::Add { name, emails, from, labels, account } => {
        if let Some(slug) = from {
            corky::contact::from_conversation::run(&slug, name.as_deref(), &labels, &account)
        } else {
            let name = name.ok_or_else(|| anyhow::anyhow!("NAME required when not using --from"))?;
            corky::contact::add::run(&name, &emails, &labels, &account)
        }
    }
    ContactCommands::Info { name } => corky::contact::info::run(&name),
},
```

### 7. `src/help.rs` — update command reference

Replace `contact-add` line with:
```rust
("contact add NAME --email EMAIL", "Add a contact with context docs"),
("contact add --from SLUG", "Create contact from a conversation"),
("contact info NAME", "Show contact info and thread history"),
```

### 8. `.claude/skills/email/SKILL.md` — add contact workflow

Add to "Use These Paths and Commands":
```
- `corky contact add --from SLUG` — create a contact from a conversation
- `corky contact info NAME` — show contact details and thread history
```

Add new workflow section:

```markdown
### Enrich contact context
1. After reviewing a thread, create a contact: `corky contact add --from SLUG`
2. Edit `contacts/{name}/AGENTS.md` with relationship details
3. Use web search to find the contact's role, company, and interests
4. Add findings to the Topics and Notes sections of AGENTS.md
```

### 9. `.claude/skills/email/README.md` — add commands

Add to Commands section:
```sh
corky contact add --from SLUG        # Create contact from conversation
corky contact info NAME              # Show contact details + threads
```

### 10. `SPECS.md` — add sections

**Section 5.22 — contact add**
```
corky contact add NAME --email EMAIL [--label LABEL] [--account ACCT]
corky contact add --from SLUG [--name NAME] [--account ACCT]
```

Document the `--from` flow: find conversation, extract senders, filter owner,
handle single/multiple, create enriched AGENTS.md.

Hidden alias: `corky contact-add` (backward-compatible, unchanged).

**Section 5.23 — contact info**
```
corky contact info NAME
```

Print contact config, AGENTS.md content, matching threads from manifest.toml,
summary with thread count and last activity.

### 11. `docs/guide/commands.md` — update Contacts section

Replace single `contact-add` line with full section covering both commands.

### 12. `docs/reference/specs.md` — mirror SPECS.md changes

### 13. Tests

**`tests/test_contact_from_conversation.rs`** — integration tests:

- Single sender conversation: creates contact with correct name/email
- Multiple sender conversation without `--name`: prints candidates, exits 1
- Multiple sender conversation with `--name`: creates correct contact
- Conversation not found: helpful error message
- No non-owner senders: appropriate error
- Contact already exists: bail
- AGENTS.md Topics section pre-filled from subject

**`tests/test_contact_info.rs`** — integration tests:

- Contact exists with threads in manifest: prints all sections
- Contact exists, no manifest: prints config + AGENTS.md, skip threads
- Contact not found: error message

**`tests/test_cli.rs`** — CLI parsing:

- `contact add NAME --email EMAIL` parses correctly
- `contact add --from SLUG` parses correctly
- `contact add --from SLUG --email EMAIL` fails (conflicts_with)
- `contact-add NAME --email EMAIL` still works (hidden alias)
- `contact info NAME` parses correctly

## New/Modified Files

| File | Change |
|------|--------|
| `src/contact/from_conversation.rs` | **New** — find conversation, extract senders, create enriched contact |
| `src/contact/info.rs` | **New** — aggregate and display contact info |
| `src/contact/mod.rs` | Add new modules |
| `src/contact/add.rs` | Make `generate_agents_md` public, add `enriched_agents_md` + `run_with_agents_md` |
| `src/cli.rs` | Add `ContactCommands` enum, `Contact` subcommand, keep hidden `ContactAdd` |
| `src/main.rs` | Dispatch for `Contact(cmd)` |
| `src/help.rs` | Update command reference |
| `.claude/skills/email/SKILL.md` | Add contact commands and workflow |
| `.claude/skills/email/README.md` | Add commands to table |
| `SPECS.md` | Add sections 5.22 and 5.23 |
| `docs/guide/commands.md` | Update contacts section |
| `docs/reference/specs.md` | Mirror SPECS.md |
| `tests/test_contact_from_conversation.rs` | Integration tests |
| `tests/test_contact_info.rs` | Integration tests |
| `tests/test_cli.rs` | CLI parsing tests |

## Edge Cases

- **Contact already exists** — bail with message (existing behavior)
- **Conversation not found** — search root + all mailboxes, bail with "not found" listing searched paths
- **No non-owner senders** — bail "No non-owner senders found"
- **Multiple senders** — print list with index, require `--name`
- **`--from` and `--email` both given** — clap `conflicts_with` rejects
- **No manifest.toml** — `contact info` prints config + AGENTS.md, shows "No manifest.toml found" note
- **Sender has no `<email>`** — skip that sender (some messages may have bare names)
- **Display name is just an email** — use email local part as name (e.g. `alice` from `alice@example.com`)

## Verification

1. `make check` passes
2. `corky contact add --from SLUG` creates contact with pre-filled AGENTS.md
3. `corky contact info NAME` shows config + threads + summary
4. `corky contact-add` (hidden alias) still works
5. `corky audit-docs` clean
