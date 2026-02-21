//! Command reference for corrkit.

use anyhow::Result;

const COMMANDS: &[(&str, &str)] = &[
    ("init --user EMAIL [PATH]", "Initialize a new project directory"),
    ("install-skill NAME", "Install an agent skill (e.g. email)"),
    ("sync [--full] [--account NAME]", "Sync email threads to markdown"),
    ("sync-auth", "Gmail OAuth setup"),
    ("list-folders [ACCOUNT]", "List IMAP folders for an account"),
    ("push-draft FILE [--send]", "Save draft to email"),
    ("add-label LABEL --account NAME", "Add a label to an account's sync config"),
    ("contact-add NAME --email EMAIL", "Add a contact with context docs"),
    ("watch [--interval N]", "Poll IMAP and sync on an interval"),
    ("spaces", "List configured spaces"),
    ("find-unanswered [--from NAME]", "Find threads awaiting a reply"),
    ("validate-draft FILE [FILE...]", "Validate draft markdown files"),
    ("audit-docs", "Audit instruction files"),
    ("help", "Show this reference"),
];

const COLLAB_COMMANDS: &[(&str, &str)] = &[
    ("collab add NAME --label LABEL", "Add a collaborator"),
    ("collab sync [NAME]", "Push/pull shared submodules"),
    ("collab status", "Check for pending changes"),
    ("collab remove NAME [--delete-repo]", "Remove a collaborator"),
    ("collab rename OLD NEW", "Rename a collaborator directory"),
    ("collab reset [NAME] [--no-sync]", "Pull, regenerate templates, commit & push"),
];

const DEV_COMMANDS: &[(&str, &str)] = &[
    ("cargo test", "Run tests"),
    ("cargo clippy", "Lint"),
    ("cargo fmt", "Format"),
];

pub fn run(filter: Option<&str>) -> Result<()> {
    if let Some(filter) = filter {
        if filter != "--dev" {
            let all_cmds: Vec<(&str, &str)> = COMMANDS
                .iter()
                .chain(COLLAB_COMMANDS.iter())
                .chain(DEV_COMMANDS.iter())
                .copied()
                .collect();
            let matches: Vec<_> = all_cmds
                .iter()
                .filter(|(name, _)| name.contains(filter))
                .collect();
            if matches.is_empty() {
                println!("No command matching '{}'", filter);
                std::process::exit(1);
            }
            print_table(&matches.iter().map(|&&(a, b)| (a, b)).collect::<Vec<_>>());
            return Ok(());
        }
    }

    println!("corrkit commands\n");
    print_table(COMMANDS);

    println!("\ncollaborator commands\n");
    print_table(COLLAB_COMMANDS);

    if filter == Some("--dev") || filter.is_none() {
        println!("\ndev commands\n");
        print_table(DEV_COMMANDS);
    }

    Ok(())
}

fn print_table(rows: &[(&str, &str)]) {
    let name_w = rows.iter().map(|(n, _)| n.len()).max().unwrap_or(0);
    for (name, desc) in rows {
        println!("  {:<width$}  {}", name, desc, width = name_w);
    }
}
