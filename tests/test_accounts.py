"""Tests for account loading, presets, password resolution, env fallback."""

from pathlib import Path
from unittest.mock import patch

import pytest

from accounts import (
    Account,
    _apply_preset,
    get_account_for_email,
    get_default_account,
    load_accounts,
    load_accounts_or_env,
    resolve_password,
)

SAMPLE_TOML = """\
[accounts.personal]
provider = "gmail"
user = "brian@gmail.com"
password_cmd = "pass email/personal"
labels = ["correspondence"]
default = true

[accounts.proton]
provider = "protonmail-bridge"
user = "brian@proton.me"
password_cmd = "pass email/proton"
labels = ["private"]

[accounts.selfhosted]
provider = "imap"
imap_host = "mail.example.com"
smtp_host = "mail.example.com"
user = "user@example.com"
password = "secret123"
labels = ["important"]
"""


def _write_toml(tmp_path: Path, content: str = SAMPLE_TOML) -> Path:
    p = tmp_path / "accounts.toml"
    p.write_text(content, encoding="utf-8")
    return p


# ---------------------------------------------------------------------------
# Loading / parsing
# ---------------------------------------------------------------------------


def test_load_accounts_parses_all(tmp_path):
    path = _write_toml(tmp_path)
    accounts = load_accounts(path)
    assert set(accounts.keys()) == {"personal", "proton", "selfhosted"}


def test_load_accounts_missing_file(tmp_path):
    assert load_accounts(tmp_path / "nope.toml") == {}


# ---------------------------------------------------------------------------
# Provider presets
# ---------------------------------------------------------------------------


def test_gmail_preset_applied(tmp_path):
    path = _write_toml(tmp_path)
    accounts = load_accounts(path)
    acct = accounts["personal"]
    assert acct.imap_host == "imap.gmail.com"
    assert acct.imap_port == 993
    assert acct.smtp_host == "smtp.gmail.com"
    assert acct.smtp_port == 465
    assert acct.drafts_folder == "[Gmail]/Drafts"
    assert acct.imap_starttls is False


def test_protonmail_preset_applied(tmp_path):
    path = _write_toml(tmp_path)
    accounts = load_accounts(path)
    acct = accounts["proton"]
    assert acct.imap_host == "127.0.0.1"
    assert acct.imap_port == 1143
    assert acct.imap_starttls is True
    assert acct.smtp_host == "127.0.0.1"
    assert acct.smtp_port == 1025
    assert acct.drafts_folder == "Drafts"


def test_generic_imap_no_preset(tmp_path):
    path = _write_toml(tmp_path)
    accounts = load_accounts(path)
    acct = accounts["selfhosted"]
    assert acct.imap_host == "mail.example.com"
    assert acct.smtp_host == "mail.example.com"
    assert acct.imap_port == 993  # default, no preset override


def test_account_override_wins_over_preset():
    acct = Account(provider="gmail", imap_port=1234)
    result = _apply_preset(acct)
    assert result.imap_port == 1234  # override kept
    assert result.imap_host == "imap.gmail.com"  # preset applied


# ---------------------------------------------------------------------------
# Password resolution
# ---------------------------------------------------------------------------


def test_resolve_password_inline():
    acct = Account(user="a@b.com", password="inline-pass")
    assert resolve_password(acct) == "inline-pass"


def test_resolve_password_cmd():
    acct = Account(user="a@b.com", password_cmd="echo hunter2")
    with patch("accounts.subprocess.run") as mock_run:
        mock_run.return_value.stdout = "hunter2\n"
        mock_run.return_value.returncode = 0
        result = resolve_password(acct)
    assert result == "hunter2"
    mock_run.assert_called_once()


def test_resolve_password_none_raises():
    acct = Account(user="a@b.com")
    with pytest.raises(ValueError, match="no password"):
        resolve_password(acct)


# ---------------------------------------------------------------------------
# Env fallback
# ---------------------------------------------------------------------------


def test_load_accounts_or_env_prefers_toml(tmp_path):
    path = _write_toml(tmp_path)
    accounts = load_accounts_or_env(path)
    assert "personal" in accounts
    assert "_legacy" not in accounts


def test_load_accounts_or_env_falls_back(tmp_path, monkeypatch):
    monkeypatch.setenv("GMAIL_USER_EMAIL", "test@gmail.com")
    monkeypatch.setenv("GMAIL_APP_PASSWORD", "pass word")
    monkeypatch.setenv("GMAIL_SYNC_LABELS", "inbox,sent")
    path = tmp_path / "nope.toml"
    accounts = load_accounts_or_env(path)
    assert "_legacy" in accounts
    acct = accounts["_legacy"]
    assert acct.user == "test@gmail.com"
    assert acct.password == "password"  # spaces stripped
    assert acct.labels == ["inbox", "sent"]
    assert acct.provider == "gmail"
    assert acct.default is True


# ---------------------------------------------------------------------------
# Default / lookup
# ---------------------------------------------------------------------------


def test_get_default_account_explicit(tmp_path):
    path = _write_toml(tmp_path)
    accounts = load_accounts(path)
    name, acct = get_default_account(accounts)
    assert name == "personal"
    assert acct.default is True


def test_get_default_account_first_fallback():
    accounts = {
        "a": Account(user="a@a.com"),
        "b": Account(user="b@b.com"),
    }
    name, _ = get_default_account(accounts)
    assert name == "a"


def test_get_account_for_email(tmp_path):
    path = _write_toml(tmp_path)
    accounts = load_accounts(path)
    result = get_account_for_email(accounts, "brian@proton.me")
    assert result is not None
    name, acct = result
    assert name == "proton"


def test_get_account_for_email_case_insensitive(tmp_path):
    path = _write_toml(tmp_path)
    accounts = load_accounts(path)
    result = get_account_for_email(accounts, "Brian@Gmail.com")
    assert result is not None
    assert result[0] == "personal"


def test_get_account_for_email_not_found(tmp_path):
    path = _write_toml(tmp_path)
    accounts = load_accounts(path)
    assert get_account_for_email(accounts, "unknown@example.com") is None


# ---------------------------------------------------------------------------
# Flat TOML (no [accounts.*] nesting)
# ---------------------------------------------------------------------------


def test_load_flat_toml(tmp_path):
    flat = """\
[personal]
provider = "gmail"
user = "brian@gmail.com"
password = "secret"
labels = ["inbox"]
"""
    path = _write_toml(tmp_path, flat)
    accounts = load_accounts(path)
    assert "personal" in accounts
    assert accounts["personal"].user == "brian@gmail.com"
