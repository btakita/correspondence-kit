# Installation

## pip / pipx (all platforms)

```sh
pip install corky
# or
pipx install corky
```

This installs a prebuilt wheel with the compiled binary — no Rust toolchain needed.

## Shell installer (Linux & macOS)

```sh
curl -sSf https://raw.githubusercontent.com/btakita/corky/main/install.sh | sh
```

This downloads a prebuilt binary to `~/.local/bin/corky`. Use `--system` to install to `/usr/local/bin` instead (requires sudo).

## From source

```sh
cargo install --path .
```

## Windows

`pip install corky` is the easiest option. Alternatively, download `.zip` from [GitHub Releases](https://github.com/btakita/corky/releases) or build from source with `cargo install --path .`.
