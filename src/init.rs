//! Initialize a new corrkit data directory with config and folder structure.

use anyhow::Result;
use std::path::Path;

use crate::accounts::provider_presets;
use crate::app_config;

/// Create the data directory structure with .gitkeep files.
fn create_dirs(data_dir: &Path) -> Result<()> {
    for sub in &["conversations", "drafts", "contacts"] {
        let d = data_dir.join(sub);
        std::fs::create_dir_all(&d)?;
        let gitkeep = d.join(".gitkeep");
        if !gitkeep.exists() {
            std::fs::write(&gitkeep, "")?;
        }
    }
    Ok(())
}

/// Generate accounts.toml content.
fn generate_accounts_toml(
    user: &str,
    provider: &str,
    password_cmd: &str,
    labels: &[String],
    github_user: &str,
    name: &str,
) -> String {
    let mut doc = toml_edit::DocumentMut::new();

    // Owner section
    if !github_user.is_empty() || !name.is_empty() {
        let mut owner = toml_edit::Table::new();
        if !github_user.is_empty() {
            owner.insert("github_user", toml_edit::value(github_user));
        }
        if !name.is_empty() {
            owner.insert("name", toml_edit::value(name));
        }
        doc.insert("owner", toml_edit::Item::Table(owner));
    }

    // Account section
    let mut accounts = toml_edit::Table::new();
    let mut default_acct = toml_edit::Table::new();
    default_acct.insert("provider", toml_edit::value(provider));
    default_acct.insert("user", toml_edit::value(user));
    let mut labels_arr = toml_edit::Array::new();
    for label in labels {
        labels_arr.push(label.as_str());
    }
    default_acct.insert("labels", toml_edit::value(labels_arr));
    default_acct.insert("default", toml_edit::value(true));
    if !password_cmd.is_empty() {
        default_acct.insert("password_cmd", toml_edit::value(password_cmd));
    }
    accounts.insert("default", toml_edit::Item::Table(default_acct));
    doc.insert("accounts", toml_edit::Item::Table(accounts));

    doc.to_string()
}

#[allow(clippy::too_many_arguments)]
pub fn run(
    user: &str,
    data_dir: &Path,
    provider: &str,
    password_cmd: &str,
    labels_str: &str,
    github_user: &str,
    name: &str,
    sync: bool,
    space: &str,
    force: bool,
) -> Result<()> {
    let data_dir = if data_dir.starts_with("~") {
        crate::resolve::expand_tilde(&data_dir.to_string_lossy())
    } else {
        data_dir.to_path_buf()
    };

    let labels: Vec<String> = labels_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let accounts_path = data_dir.join("accounts.toml");
    if accounts_path.exists() && !force {
        eprintln!("accounts.toml already exists at {}", accounts_path.display());
        eprintln!("Use --force to overwrite.");
        std::process::exit(1);
    }

    // 1. Create directory structure
    create_dirs(&data_dir)?;
    println!("Created {}/{{conversations,drafts,contacts}}/", data_dir.display());

    // 2. Generate accounts.toml
    let content = generate_accounts_toml(user, provider, password_cmd, &labels, github_user, name);
    std::fs::write(&accounts_path, &content)?;
    println!("Created {}", accounts_path.display());

    // 3. Create empty collaborators.toml and contacts.toml
    for filename in &["collaborators.toml", "contacts.toml"] {
        let p = data_dir.join(filename);
        if !p.exists() {
            std::fs::write(&p, "")?;
            println!("Created {}", p.display());
        }
    }

    // 4. Register space in app config
    app_config::add_space(space, &data_dir.to_string_lossy())?;
    println!("Registered space '{}' \u{2192} {}", space, data_dir.display());

    // 5. Provider-specific guidance
    let presets = provider_presets();
    if provider == "gmail" && password_cmd.is_empty() {
        println!();
        println!("Gmail setup:");
        println!("  Option A: App password \u{2014} https://myaccount.google.com/apppasswords");
        println!("    Add password_cmd = \"pass email/personal\" to accounts.toml");
        println!("  Option B: OAuth \u{2014} run 'corrkit sync-auth' after placing credentials.json");
    }

    // 6. Optional first sync
    if sync {
        std::env::set_var("CORRKIT_DATA", data_dir.to_string_lossy().as_ref());
        println!();
        crate::sync::run(false, None)?;
    }

    if !sync {
        println!();
        println!("Done! Next steps:");
        println!("  - Edit {} with your credentials", accounts_path.display());
        if provider == "gmail" && password_cmd.is_empty() {
            println!("  - Set up app password or OAuth (see above)");
        }
        if !presets.contains_key(provider) && provider == "imap" {
            println!("  - Add imap_host, smtp_host to accounts.toml");
        }
        println!(
            "  - Run: CORRKIT_DATA={} corrkit sync",
            data_dir.display()
        );
    }

    Ok(())
}
