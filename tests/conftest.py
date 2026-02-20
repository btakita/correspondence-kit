"""Shared fixtures for correspondence-kit tests."""

import pytest

from accounts import Account


@pytest.fixture
def gmail_account() -> Account:
    """A sample Gmail Account for tests that need one."""
    return Account(
        provider="gmail",
        user="test@example.com",
        password="test-password",
        labels=["correspondence"],
        imap_host="imap.gmail.com",
        imap_port=993,
        smtp_host="smtp.gmail.com",
        smtp_port=465,
        drafts_folder="[Gmail]/Drafts",
        default=True,
    )
