# Building

## Developer setup

```sh
git clone https://github.com/btakita/corky.git
cd corky
cp .corky.toml.example mail/.corky.toml   # configure your email accounts
make release                               # build + symlink to .bin/corky
```

## Make targets

```sh
make build        # Debug build
make release      # Release build + symlink to .bin/corky
make test         # Run tests
make clippy       # Lint
make check        # Lint + test
make install      # Install to ~/.cargo/bin
make precommit    # Full pre-commit checks
```

## .gitignore

The following are gitignored:

```
.env
.corky.toml
credentials.json
*.credentials.json
CLAUDE.local.md
AGENTS.local.md
mail
.idea/
tmp/
target/
.bin/
```

Config files (`.corky.toml`, `voice.md`) live inside `mail/` which is already gitignored. `credentials.json` is also gitignored in `mail/.gitignore`.
