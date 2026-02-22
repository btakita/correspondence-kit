# Quick Start

## Initialize

```sh
corky init --user you@gmail.com
```

This creates `~/Documents/mail` with directory structure, `.corky.toml`, and empty config files inside it.

## Configure

Edit `mail/.corky.toml` with your email credentials:

```toml
[accounts.personal]
provider = "gmail"
user = "you@gmail.com"
password_cmd = "pass email/personal"
labels = ["correspondence"]
default = true
```

## Sync

```sh
corky sync
```

Threads are written to `mail/conversations/[slug].md` — one file per thread. Labels and accounts are metadata inside each file. A `manifest.toml` index is generated after each sync.

## Basic workflow

```sh
corky sync                          # Pull new threads
corky unanswered                    # See what needs a reply
# Draft a reply in mail/drafts/
corky draft validate mail/drafts/FILE.md   # Check format
corky draft push mail/drafts/FILE.md       # Save as email draft
```
