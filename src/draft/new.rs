//! Scaffold a new draft markdown file.

use anyhow::Result;
use chrono::Local;
use std::path::PathBuf;

use crate::config::corky_config;
use crate::resolve;
use crate::util;

/// Create a new draft file with the given metadata fields.
pub fn run(
    subject: &str,
    to: &str,
    cc: Option<&str>,
    account: Option<&str>,
    from: Option<&str>,
    in_reply_to: Option<&str>,
    mailbox: Option<&str>,
) -> Result<()> {
    let drafts_dir = match mailbox {
        Some(name) => resolve::mailbox_dir(name).join("drafts"),
        None => resolve::drafts_dir(),
    };
    std::fs::create_dir_all(&drafts_dir)?;

    // Resolve author name from [owner] in .corky.toml
    let author = corky_config::try_load_config(None)
        .and_then(|cfg| cfg.owner)
        .map(|o| o.name)
        .unwrap_or_default();

    let date = Local::now().format("%Y-%m-%d").to_string();
    let slug = util::slugify(subject);
    let path = unique_path(&drafts_dir, &date, &slug);

    let content = render(subject, to, cc, account, from, in_reply_to, &author);
    std::fs::write(&path, content)?;
    println!("{}", path.display());
    Ok(())
}

/// Find a unique filename, appending -2, -3, etc. on collision.
fn unique_path(dir: &std::path::Path, date: &str, slug: &str) -> PathBuf {
    let base = dir.join(format!("{}-{}.md", date, slug));
    if !base.exists() {
        return base;
    }
    let mut n = 2u32;
    loop {
        let candidate = dir.join(format!("{}-{}-{}.md", date, slug, n));
        if !candidate.exists() {
            return candidate;
        }
        n += 1;
    }
}

/// Render the draft markdown content.
fn render(
    subject: &str,
    to: &str,
    cc: Option<&str>,
    account: Option<&str>,
    from: Option<&str>,
    in_reply_to: Option<&str>,
    author: &str,
) -> String {
    let mut lines = Vec::new();
    lines.push(format!("# {}", subject));
    lines.push(String::new());
    lines.push(format!("**To**: {}", to));
    if let Some(cc) = cc {
        lines.push(format!("**CC**: {}", cc));
    }
    lines.push("**Status**: draft".to_string());
    if !author.is_empty() {
        lines.push(format!("**Author**: {}", author));
    }
    if let Some(account) = account {
        lines.push(format!("**Account**: {}", account));
    }
    if let Some(from) = from {
        lines.push(format!("**From**: {}", from));
    }
    if let Some(in_reply_to) = in_reply_to {
        lines.push(format!("**In-Reply-To**: {}", in_reply_to));
    }
    lines.push(String::new());
    lines.push("---".to_string());
    lines.push(String::new());
    lines.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_minimal() {
        let out = render("Hello", "a@b.com", None, None, None, None, "");
        assert!(out.starts_with("# Hello\n"));
        assert!(out.contains("**To**: a@b.com\n"));
        assert!(out.contains("**Status**: draft\n"));
        assert!(!out.contains("**Author**"));
        assert!(!out.contains("**CC**"));
        assert!(out.ends_with("---\n\n"));
    }

    #[test]
    fn test_render_all_fields() {
        let out = render(
            "Test",
            "a@b.com",
            Some("c@d.com"),
            Some("personal"),
            Some("me@x.com"),
            Some("<msg-1>"),
            "Alice",
        );
        assert!(out.contains("**CC**: c@d.com\n"));
        assert!(out.contains("**Author**: Alice\n"));
        assert!(out.contains("**Account**: personal\n"));
        assert!(out.contains("**From**: me@x.com\n"));
        assert!(out.contains("**In-Reply-To**: <msg-1>\n"));
    }

    #[test]
    fn test_unique_path_no_collision() {
        let dir = std::env::temp_dir().join("corky-test-unique-no-collision");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let p = unique_path(&dir, "2026-02-22", "hello");
        assert_eq!(p.file_name().unwrap().to_str().unwrap(), "2026-02-22-hello.md");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_unique_path_collision() {
        let dir = std::env::temp_dir().join("corky-test-unique-collision");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("2026-02-22-hello.md"), "x").unwrap();
        let p = unique_path(&dir, "2026-02-22", "hello");
        assert_eq!(p.file_name().unwrap().to_str().unwrap(), "2026-02-22-hello-2.md");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
