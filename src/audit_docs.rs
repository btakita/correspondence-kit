//! Audit instruction files against the codebase.

use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::{Path, PathBuf};

const LINE_BUDGET: usize = 1000;
static SKIP_PATHS: Lazy<std::collections::HashSet<&str>> =
    Lazy::new(|| [".env"].iter().copied().collect());

struct Issue {
    file: String,
    line: usize,
    message: String,
}

/// Find the project root by walking up from CWD looking for Cargo.toml.
fn find_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut dir = cwd.as_path();
    loop {
        if dir.join("Cargo.toml").exists() {
            return dir.to_path_buf();
        }
        match dir.parent() {
            Some(p) => dir = p,
            None => {
                eprintln!("Error: could not find Cargo.toml");
                std::process::exit(2);
            }
        }
    }
}

fn find_instruction_files(root: &Path) -> Vec<PathBuf> {
    let patterns = ["AGENTS.md", "README.md"];
    let mut found = std::collections::HashSet::new();

    for pattern in &patterns {
        let path = root.join(pattern);
        if path.exists() {
            found.insert(path);
        }
    }

    // .claude/**/SKILL.md
    if let Ok(entries) = glob::glob(&root.join(".claude/**/SKILL.md").to_string_lossy()) {
        for entry in entries.flatten() {
            found.insert(entry);
        }
    }

    // src/**/AGENTS.md
    if let Ok(entries) = glob::glob(&root.join("src/**/AGENTS.md").to_string_lossy()) {
        for entry in entries.flatten() {
            found.insert(entry);
        }
    }

    let mut result: Vec<PathBuf> = found.into_iter().collect();
    result.sort();
    result
}

/// Parse file paths from the Project Structure tree block.
fn extract_tree_paths(content: &str) -> Vec<(usize, String)> {
    let mut results = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut in_section = false;
    let mut in_block = false;
    let mut stack: Vec<(usize, String)> = Vec::new(); // (indent, dirname_with_slash)

    for (i, line) in lines.iter().enumerate() {
        let line_no = i + 1;
        if line.starts_with("## Project Structure") {
            in_section = true;
            continue;
        }
        if in_section && !in_block {
            if line.trim().starts_with("```") {
                in_block = true;
                continue;
            }
            if line.starts_with("## ") {
                break;
            }
            continue;
        }
        if !in_block {
            continue;
        }
        if line.trim().starts_with("```") {
            break;
        }

        let stripped = line.trim_end();
        let trimmed = stripped.trim();
        if trimmed.is_empty() {
            continue;
        }
        let indent = stripped.len() - stripped.trim_start().len();
        let mut name = trimmed.split('#').next().unwrap_or("").trim().to_string();
        if name.is_empty() {
            continue;
        }

        // Strip symlink arrow notation
        if name.contains(" -> ") {
            name = format!("{}/", name.split(" -> ").next().unwrap_or("").trim());
        }

        // Pop deeper/equal entries from stack
        while stack.last().map(|(ind, _)| *ind >= indent).unwrap_or(false) {
            stack.pop();
        }

        if name.ends_with('/') {
            stack.push((indent, name));
        } else {
            let mut parts: Vec<String> = stack.iter().map(|(_, d)| d.clone()).collect();
            parts.push(name);
            let full = parts.join("");
            results.push((line_no, full));
        }
    }

    results
}

fn check_tree_paths(rel: &str, content: &str, root: &Path) -> Vec<Issue> {
    let mut issues = Vec::new();
    let bracket_re = Regex::new(r"\[.*?]").unwrap();
    for (line_no, path) in extract_tree_paths(content) {
        if bracket_re.is_match(&path) {
            continue;
        }
        if SKIP_PATHS.contains(path.as_str()) {
            continue;
        }
        if !root.join(&path).exists() {
            issues.push(Issue {
                file: rel.to_string(),
                line: line_no,
                message: format!("Referenced path does not exist: {}", path),
            });
        }
    }
    issues
}

fn check_line_budget(files: &[PathBuf], root: &Path) -> (Vec<Issue>, Vec<(String, usize)>, usize) {
    let mut counts = Vec::new();
    let mut total = 0;
    for f in files {
        if let Ok(content) = std::fs::read_to_string(f) {
            let n = content.lines().count();
            let rel = f.strip_prefix(root).unwrap_or(f).to_string_lossy().to_string();
            counts.push((rel, n));
            total += n;
        }
    }
    let mut issues = Vec::new();
    if total > LINE_BUDGET {
        issues.push(Issue {
            file: "(all)".to_string(),
            line: 0,
            message: format!("Over line budget: {} lines (max {})", total, LINE_BUDGET),
        });
    }
    (issues, counts, total)
}

fn check_staleness(files: &[PathBuf], root: &Path) -> Vec<Issue> {
    let src_dir = root.join("src");
    if !src_dir.exists() {
        return vec![];
    }

    let mut newest_mtime = std::time::SystemTime::UNIX_EPOCH;
    let mut newest_src = PathBuf::new();

    fn scan_rs(dir: &Path, newest: &mut std::time::SystemTime, newest_path: &mut PathBuf) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    scan_rs(&path, newest, newest_path);
                } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                    if let Ok(meta) = path.metadata() {
                        if let Ok(mtime) = meta.modified() {
                            if mtime > *newest {
                                *newest = mtime;
                                *newest_path = path;
                            }
                        }
                    }
                }
            }
        }
    }

    scan_rs(&src_dir, &mut newest_mtime, &mut newest_src);

    let mut issues = Vec::new();
    for doc in files {
        if let Ok(meta) = doc.metadata() {
            if let Ok(doc_mtime) = meta.modified() {
                if doc_mtime < newest_mtime {
                    let rel = doc.strip_prefix(root).unwrap_or(doc).to_string_lossy().to_string();
                    let src_rel = newest_src
                        .strip_prefix(root)
                        .unwrap_or(&newest_src)
                        .to_string_lossy()
                        .to_string();
                    issues.push(Issue {
                        file: rel,
                        line: 0,
                        message: format!("Older than {} \u{2014} may be stale", src_rel),
                    });
                }
            }
        }
    }
    issues
}

pub fn run() -> Result<()> {
    println!("Auditing docs...\n");

    let root = find_root();
    let files = find_instruction_files(&root);
    let mut issues: Vec<Issue> = Vec::new();

    for doc in &files {
        let rel = doc
            .strip_prefix(&root)
            .unwrap_or(doc)
            .to_string_lossy()
            .to_string();
        if let Ok(content) = std::fs::read_to_string(doc) {
            issues.extend(check_tree_paths(&rel, &content, &root));
        }
    }

    let (budget_issues, counts, total) = check_line_budget(&files, &root);
    issues.extend(budget_issues);
    issues.extend(check_staleness(&files, &root));

    for issue in &issues {
        let mut loc = format!("  {}", issue.file);
        if issue.line > 0 {
            loc.push_str(&format!(":{}", issue.line));
        }
        println!("{:<35} \u{2717} {}", loc, issue.message);
    }

    let mark = if total <= LINE_BUDGET { "\u{2713}" } else { "\u{2717}" };
    println!(
        "\nCombined instruction files: {} lines (budget: {}) {}",
        total, LINE_BUDGET, mark
    );
    for (name, n) in &counts {
        println!("  {}: {}", name, n);
    }

    let n = issues.len();
    if n > 0 {
        println!("\nFound {} issue(s)", n);
        std::process::exit(1);
    } else {
        println!("\nNo issues found \u{2713}");
    }

    Ok(())
}
