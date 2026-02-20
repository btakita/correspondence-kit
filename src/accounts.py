"""Account configuration — parse accounts.toml with provider presets."""

import os
import subprocess
import tomllib
from pathlib import Path

import msgspec

CONFIG_PATH = Path("accounts.toml")

PROVIDER_PRESETS: dict[str, dict[str, object]] = {
    "gmail": {
        "imap_host": "imap.gmail.com",
        "imap_port": 993,
        "imap_starttls": False,
        "smtp_host": "smtp.gmail.com",
        "smtp_port": 465,
        "drafts_folder": "[Gmail]/Drafts",
    },
    "protonmail-bridge": {
        "imap_host": "127.0.0.1",
        "imap_port": 1143,
        "imap_starttls": True,
        "smtp_host": "127.0.0.1",
        "smtp_port": 1025,
        "drafts_folder": "Drafts",
    },
}


_NON_ACCOUNT_KEYS = frozenset({"watch"})


class WatchConfig(msgspec.Struct):
    poll_interval: int = 300
    notify: bool = False


class Account(msgspec.Struct):
    provider: str = "imap"
    user: str = ""
    password: str = ""
    password_cmd: str = ""
    labels: list[str] = []
    imap_host: str = ""
    imap_port: int = 993
    imap_starttls: bool = False
    smtp_host: str = ""
    smtp_port: int = 465
    drafts_folder: str = "Drafts"
    sync_days: int = 3650
    default: bool = False


def _apply_preset(account: Account) -> Account:
    """Merge provider preset defaults — account values win over preset."""
    preset = PROVIDER_PRESETS.get(account.provider)
    if preset is None:
        return account
    # Only apply preset values for fields still at their Struct defaults
    defaults = Account()
    updates: dict[str, object] = {}
    for key, preset_val in preset.items():
        if getattr(account, key) == getattr(defaults, key):
            updates[key] = preset_val
    if updates:
        return msgspec.structs.replace(account, **updates)
    return account


def resolve_password(account: Account) -> str:
    """Return password: inline value if set, else run password_cmd."""
    if account.password:
        return account.password
    if account.password_cmd:
        result = subprocess.run(
            account.password_cmd,
            shell=True,
            capture_output=True,
            text=True,
            check=True,
        )
        return result.stdout.strip()
    raise ValueError(f"Account {account.user!r} has no password or password_cmd")


def load_accounts(path: Path | None = None) -> dict[str, Account]:
    """Parse accounts.toml → {name: Account} mapping."""
    if path is None:
        path = CONFIG_PATH
    if not path.exists():
        return {}
    with open(path, "rb") as f:
        raw = tomllib.load(f)
    accounts_section = raw.get("accounts", raw)
    result: dict[str, Account] = {}
    for name, data in accounts_section.items():
        if name in _NON_ACCOUNT_KEYS:
            continue
        account = msgspec.convert(data, Account)
        account = _apply_preset(account)
        result[name] = account
    return result


def _legacy_account_from_env() -> Account:
    """Build a synthetic Account from legacy .env GMAIL_* vars."""
    from dotenv import load_dotenv

    load_dotenv()

    user = os.environ.get("GMAIL_USER_EMAIL", "")
    password = os.environ.get("GMAIL_APP_PASSWORD", "")
    labels_str = os.environ.get("GMAIL_SYNC_LABELS", "")
    sync_days = int(os.environ.get("GMAIL_SYNC_DAYS", "3650"))

    if not user:
        raise SystemExit("No accounts.toml found and GMAIL_USER_EMAIL not set in .env")

    return Account(
        provider="gmail",
        user=user,
        password=password.replace(" ", ""),
        labels=[s.strip() for s in labels_str.split(",") if s.strip()],
        imap_host="imap.gmail.com",
        imap_port=993,
        smtp_host="smtp.gmail.com",
        smtp_port=465,
        drafts_folder="[Gmail]/Drafts",
        sync_days=sync_days,
        default=True,
    )


def load_accounts_or_env(path: Path | None = None) -> dict[str, Account]:
    """Load accounts.toml, falling back to .env GMAIL_* vars."""
    accounts = load_accounts(path)
    if accounts:
        return accounts
    return {"_legacy": _legacy_account_from_env()}


def get_default_account(accounts: dict[str, Account]) -> tuple[str, Account]:
    """Return (name, account) for the default account."""
    for name, acct in accounts.items():
        if acct.default:
            return name, acct
    # Fall back to first account
    name = next(iter(accounts))
    return name, accounts[name]


def get_account_for_email(
    accounts: dict[str, Account], email_addr: str
) -> tuple[str, Account] | None:
    """Lookup account by email address."""
    email_lower = email_addr.lower()
    for name, acct in accounts.items():
        if acct.user.lower() == email_lower:
            return name, acct
    return None


def load_watch_config(path: Path | None = None) -> WatchConfig:
    """Load [watch] section from accounts.toml. Returns defaults if missing."""
    if path is None:
        path = CONFIG_PATH
    if not path.exists():
        return WatchConfig()
    with open(path, "rb") as f:
        raw = tomllib.load(f)
    watch_data = raw.get("watch")
    if watch_data is None:
        return WatchConfig()
    return msgspec.convert(watch_data, WatchConfig)
