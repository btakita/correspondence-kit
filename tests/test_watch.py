"""Tests for the watch polling daemon."""

from pathlib import Path
from unittest.mock import patch

from accounts import WatchConfig, load_watch_config
from sync.types import AccountSyncState, LabelState, SyncState
from watch import (
    _count_new_messages,
    _notify,
    _poll_once,
    _shutdown,
    _snapshot_uids,
    main,
)

WATCH_TOML = """\
[accounts.personal]
provider = "gmail"
user = "test@gmail.com"
password = "secret"
labels = ["inbox"]
default = true

[watch]
poll_interval = 60
notify = true
"""

WATCH_TOML_NO_SECTION = """\
[accounts.personal]
provider = "gmail"
user = "test@gmail.com"
password = "secret"
labels = ["inbox"]
"""


def _write_toml(tmp_path: Path, content: str = WATCH_TOML) -> Path:
    p = tmp_path / "accounts.toml"
    p.write_text(content, encoding="utf-8")
    return p


# ---------------------------------------------------------------------------
# WatchConfig loading
# ---------------------------------------------------------------------------


def test_load_watch_config(tmp_path):
    path = _write_toml(tmp_path)
    config = load_watch_config(path)
    assert config.poll_interval == 60
    assert config.notify is True


def test_load_watch_config_defaults(tmp_path):
    path = _write_toml(tmp_path, WATCH_TOML_NO_SECTION)
    config = load_watch_config(path)
    assert config.poll_interval == 300
    assert config.notify is False


def test_load_watch_config_missing_file(tmp_path):
    config = load_watch_config(tmp_path / "nope.toml")
    assert config.poll_interval == 300
    assert config.notify is False


def test_watch_section_does_not_leak_into_accounts(tmp_path):
    """The [watch] section must not be parsed as an account."""
    from accounts import load_accounts

    path = _write_toml(tmp_path)
    accounts = load_accounts(path)
    assert "watch" not in accounts
    assert "personal" in accounts


# ---------------------------------------------------------------------------
# UID snapshot / change detection
# ---------------------------------------------------------------------------


def test_snapshot_uids():
    state = SyncState(
        accounts={
            "a": AccountSyncState(
                labels={"inbox": LabelState(uidvalidity=1, last_uid=100)}
            ),
        }
    )
    snap = _snapshot_uids(state)
    assert snap == {"a": {"inbox": 100}}


def test_count_new_messages_detects_increase():
    before = {"a": {"inbox": 100, "sent": 50}}
    after = {"a": {"inbox": 105, "sent": 50}}
    assert _count_new_messages(before, after) == 1


def test_count_new_messages_no_change():
    snap = {"a": {"inbox": 100}}
    assert _count_new_messages(snap, snap) == 0


def test_count_new_messages_new_account():
    before: dict[str, dict[str, int]] = {}
    after = {"a": {"inbox": 10}}
    assert _count_new_messages(before, after) == 1


def test_count_new_messages_new_label():
    before = {"a": {"inbox": 100}}
    after = {"a": {"inbox": 100, "sent": 50}}
    assert _count_new_messages(before, after) == 1


# ---------------------------------------------------------------------------
# Notifications
# ---------------------------------------------------------------------------


@patch("watch.subprocess.run")
@patch("watch.platform.system", return_value="Linux")
def test_notify_linux(mock_system, mock_run):
    _notify("title", "body")
    mock_run.assert_called_once()
    cmd = mock_run.call_args[0][0]
    assert cmd[0] == "notify-send"


@patch("watch.subprocess.run")
@patch("watch.platform.system", return_value="Darwin")
def test_notify_macos(mock_system, mock_run):
    _notify("title", "body")
    mock_run.assert_called_once()
    cmd = mock_run.call_args[0][0]
    assert cmd[0] == "osascript"


@patch("watch.subprocess.run", side_effect=FileNotFoundError)
@patch("watch.platform.system", return_value="Linux")
def test_notify_missing_tool(mock_system, mock_run):
    """Notification silently degrades when tool not installed."""
    _notify("title", "body")  # Should not raise


@patch("watch.platform.system", return_value="Windows")
def test_notify_unsupported_platform(mock_system):
    """No-op on unsupported platforms."""
    _notify("title", "body")  # Should not raise


# ---------------------------------------------------------------------------
# Poll cycle
# ---------------------------------------------------------------------------


@patch("watch._sync_collaborators")
@patch("watch._save_state")
@patch("watch.sync_account")
@patch("watch.resolve_password", return_value="secret")
@patch("watch.load_accounts_or_env")
@patch("watch._load_state")
def test_poll_once_no_new_messages(
    mock_load_state, mock_accounts, mock_password, mock_sync, mock_save, mock_collab
):
    """No new messages: collab sync should NOT run."""
    from accounts import Account

    mock_accounts.return_value = {
        "test": Account(
            provider="gmail",
            user="t@t.com",
            password="s",
            labels=["inbox"],
            imap_host="imap.gmail.com",
        )
    }
    state = SyncState(
        accounts={"test": AccountSyncState(labels={"inbox": LabelState(1, 100)})}
    )
    mock_load_state.return_value = state

    count = _poll_once(notify_enabled=False)
    assert count == 0
    mock_collab.assert_not_called()


@patch("watch._notify")
@patch("watch._sync_collaborators")
@patch("watch._save_state")
@patch("watch.sync_account")
@patch("watch.resolve_password", return_value="secret")
@patch("watch.load_accounts_or_env")
@patch("watch._load_state")
def test_poll_once_with_new_messages(
    mock_load_state,
    mock_accounts,
    mock_password,
    mock_sync,
    mock_save,
    mock_collab,
    mock_notify,
):
    """New messages: collab sync should run, notify if enabled."""
    from accounts import Account

    mock_accounts.return_value = {
        "test": Account(
            provider="gmail",
            user="t@t.com",
            password="s",
            labels=["inbox"],
            imap_host="imap.gmail.com",
        )
    }
    state = SyncState(
        accounts={"test": AccountSyncState(labels={"inbox": LabelState(1, 100)})}
    )
    mock_load_state.return_value = state

    # Simulate sync_account advancing the UID
    def advance_uid(*args, **kwargs):
        state.accounts["test"].labels["inbox"] = LabelState(uidvalidity=1, last_uid=110)

    mock_sync.side_effect = advance_uid

    count = _poll_once(notify_enabled=True)
    assert count == 1
    mock_collab.assert_called_once()
    mock_notify.assert_called_once()


@patch("watch._save_state")
@patch("watch.sync_account", side_effect=Exception("connection failed"))
@patch("watch.resolve_password", return_value="secret")
@patch("watch.load_accounts_or_env")
@patch("watch._load_state")
def test_poll_once_sync_error_continues(
    mock_load_state, mock_accounts, mock_password, mock_sync, mock_save
):
    """Sync errors for one account should not crash the poll cycle."""
    from accounts import Account

    mock_accounts.return_value = {
        "test": Account(
            provider="gmail",
            user="t@t.com",
            password="s",
            labels=["inbox"],
            imap_host="imap.gmail.com",
        )
    }
    mock_load_state.return_value = SyncState()

    count = _poll_once(notify_enabled=False)
    assert count == 0  # No crash, returns 0


# ---------------------------------------------------------------------------
# Signal handling / main loop
# ---------------------------------------------------------------------------


@patch("watch.load_watch_config")
@patch("watch._poll_once", return_value=0)
def test_main_exits_on_shutdown(mock_poll, mock_config):
    """Main loop exits when _shutdown event is set."""
    mock_config.return_value = WatchConfig(poll_interval=1)

    def poll_then_stop(**kwargs):
        _shutdown.set()
        return 0

    mock_poll.side_effect = poll_then_stop

    with patch("sys.argv", ["watch"]):
        main()

    mock_poll.assert_called_once()
    # Reset for other tests
    _shutdown.clear()


@patch("watch.load_watch_config")
@patch("watch._poll_once", return_value=0)
def test_main_interval_override(mock_poll, mock_config):
    """--interval flag overrides config."""
    mock_config.return_value = WatchConfig(poll_interval=300)

    def poll_then_stop(**kwargs):
        _shutdown.set()
        return 0

    mock_poll.side_effect = poll_then_stop

    with patch("sys.argv", ["watch", "--interval", "5"]):
        main()

    # Verify it ran (interval applied internally to _shutdown.wait)
    mock_poll.assert_called_once()
    _shutdown.clear()
