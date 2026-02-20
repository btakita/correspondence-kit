"""Tests for contact config parser (load/save contacts.toml)."""

from contact import Contact, load_contacts, save_contacts


def test_load_missing_file(tmp_path):
    result = load_contacts(tmp_path / "nonexistent.toml")
    assert result == {}


def test_load_empty_file(tmp_path):
    p = tmp_path / "contacts.toml"
    p.write_text("", encoding="utf-8")
    result = load_contacts(p)
    assert result == {}


def test_load_single_contact(tmp_path):
    p = tmp_path / "contacts.toml"
    p.write_text(
        '[alex]\nemails = ["alex@example.com", "alex@work.com"]\n'
        'labels = ["correspondence"]\n'
        'account = "personal"\n',
        encoding="utf-8",
    )
    result = load_contacts(p)
    assert "alex" in result
    c = result["alex"]
    assert c.emails == ["alex@example.com", "alex@work.com"]
    assert c.labels == ["correspondence"]
    assert c.account == "personal"


def test_load_multiple_contacts(tmp_path):
    p = tmp_path / "contacts.toml"
    p.write_text(
        '[alex]\nemails = ["alex@example.com"]\n'
        'labels = ["correspondence"]\n\n'
        '[dana]\nemails = ["dana@example.com"]\n'
        'labels = ["project-x", "triage"]\n',
        encoding="utf-8",
    )
    result = load_contacts(p)
    assert len(result) == 2
    assert result["dana"].labels == ["project-x", "triage"]
    assert result["dana"].account == ""


def test_load_minimal_contact(tmp_path):
    """Contact with no optional fields."""
    p = tmp_path / "contacts.toml"
    p.write_text('[alex]\nemails = ["alex@example.com"]\n', encoding="utf-8")
    result = load_contacts(p)
    assert result["alex"].emails == ["alex@example.com"]
    assert result["alex"].labels == []
    assert result["alex"].account == ""


def test_save_round_trip(tmp_path):
    p = tmp_path / "contacts.toml"
    contacts = {
        "alex": Contact(
            emails=["alex@example.com", "alex@work.com"],
            labels=["correspondence"],
            account="personal",
        ),
        "dana": Contact(
            emails=["dana@example.com"],
            labels=["project-x", "triage"],
        ),
    }
    save_contacts(contacts, p)

    reloaded = load_contacts(p)
    assert len(reloaded) == 2
    assert reloaded["alex"].emails == ["alex@example.com", "alex@work.com"]
    assert reloaded["alex"].labels == ["correspondence"]
    assert reloaded["alex"].account == "personal"
    assert reloaded["dana"].labels == ["project-x", "triage"]
    assert reloaded["dana"].account == ""


def test_save_overwrites(tmp_path):
    p = tmp_path / "contacts.toml"
    save_contacts({"a": Contact(emails=["a@test.com"])}, p)
    save_contacts({"b": Contact(emails=["b@test.com"])}, p)
    result = load_contacts(p)
    assert "a" not in result
    assert "b" in result


def test_save_empty_optional_fields(tmp_path):
    """Contacts with empty lists/strings omit those fields from TOML."""
    p = tmp_path / "contacts.toml"
    save_contacts({"alex": Contact(emails=["alex@test.com"])}, p)
    content = p.read_text(encoding="utf-8")
    assert "labels" not in content
    assert "account" not in content
