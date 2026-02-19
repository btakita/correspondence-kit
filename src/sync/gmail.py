"""
Syncs Gmail threads (by label) to local Markdown files under conversations/.
Uses IMAP with an App Password — no OAuth required.

Setup:
  1. Enable 2FA on your Google account
  2. Go to myaccount.google.com/apppasswords
  3. Create an app password named "correspondence"
  4. Add to .env: GMAIL_APP_PASSWORD=xxxx xxxx xxxx xxxx

Usage: uv run sync-gmail
"""
import email
import email.message
import os
import re
from datetime import datetime, timedelta
from email.header import decode_header as _decode_header
from pathlib import Path

from dotenv import load_dotenv
from imapclient import IMAPClient

from .types import Message, Thread

load_dotenv()

GMAIL_USER = os.environ["GMAIL_USER_EMAIL"]
GMAIL_APP_PASSWORD = os.environ["GMAIL_APP_PASSWORD"].replace(" ", "")
SYNC_LABELS = [l.strip() for l in os.environ["GMAIL_SYNC_LABELS"].split(",") if l.strip()]
SYNC_DAYS = int(os.getenv("GMAIL_SYNC_DAYS", "90"))

if not SYNC_LABELS:
    raise SystemExit("GMAIL_SYNC_LABELS must list at least one label")


def decode_header(value: str) -> str:
    parts = _decode_header(value)
    return "".join(
        part.decode(enc or "utf-8") if isinstance(part, bytes) else part
        for part, enc in parts
    )


def extract_body(msg: email.message.Message) -> str:
    if msg.is_multipart():
        for part in msg.walk():
            if part.get_content_type() == "text/plain" and not part.get("Content-Disposition"):
                payload = part.get_payload(decode=True)
                if payload:
                    return payload.decode(part.get_content_charset() or "utf-8", errors="replace")
    else:
        payload = msg.get_payload(decode=True)
        if payload:
            return payload.decode(msg.get_content_charset() or "utf-8", errors="replace")
    return ""


def slugify(text: str) -> str:
    text = text.lower()
    text = re.sub(r"[^a-z0-9]+", "-", text)
    return text.strip("-")[:60]


def thread_to_markdown(thread: Thread) -> str:
    lines = [
        f"# {thread.subject}",
        "",
        f"**Label**: {thread.label}",
        f"**Thread ID**: {thread.id}",
        f"**Last updated**: {thread.last_date}",
        "",
    ]
    for msg in thread.messages:
        lines += ["---", "", f"## {msg.from_} — {msg.date}", "", msg.body.strip(), ""]
    return "\n".join(lines)


def sync_label(imap: IMAPClient, label_name: str) -> None:
    print(f"Syncing label: {label_name}")

    try:
        imap.select_folder(label_name, readonly=True)
    except Exception:
        print(f'  Label "{label_name}" not found — skipping')
        return

    since = (datetime.utcnow().replace(hour=0, minute=0, second=0) -
             timedelta(days=SYNC_DAYS)).strftime("%d-%b-%Y")
    msg_ids = imap.search(["SINCE", since])
    print(f"  Found {len(msg_ids)} messages in last {SYNC_DAYS} days")

    # Group by subject into threads (simple approach)
    threads: dict[str, Thread] = {}

    for msg_id, msg_data in imap.fetch(msg_ids, "RFC822").items():
        raw = msg_data[b"RFC822"]
        msg = email.message_from_bytes(raw)

        subject = decode_header(msg.get("Subject", "(no subject)"))
        from_ = decode_header(msg.get("From", ""))
        date = msg.get("Date", "")
        thread_key = re.sub(r"^(re|fwd?):\s*", "", subject.lower().strip())
        body = extract_body(msg)

        message = Message(
            id=str(msg_id),
            thread_id=thread_key,
            from_=from_,
            date=date,
            subject=subject,
            body=body,
        )

        if thread_key not in threads:
            threads[thread_key] = Thread(
                id=thread_key,
                label=label_name,
                subject=subject,
                messages=[],
                last_date=date,
            )
        threads[thread_key].messages.append(message)
        threads[thread_key].last_date = date

    out_dir = Path("conversations") / label_name
    out_dir.mkdir(parents=True, exist_ok=True)

    for thread in threads.values():
        try:
            date_prefix = datetime.strptime(thread.last_date[:16].strip(), "%a, %d %b %Y").strftime("%Y-%m-%d")
        except ValueError:
            date_prefix = datetime.utcnow().strftime("%Y-%m-%d")
        filename = f"{date_prefix}-{slugify(thread.subject)}.md"
        filepath = out_dir / filename
        filepath.write_text(thread_to_markdown(thread), encoding="utf-8")
        print(f"  Wrote: conversations/{label_name}/{filename}")


def main() -> None:
    with IMAPClient("imap.gmail.com", ssl=True) as imap:
        imap.login(GMAIL_USER, GMAIL_APP_PASSWORD)
        for label in SYNC_LABELS:
            sync_label(imap, label)
    print("Sync complete.")


if __name__ == "__main__":
    main()
