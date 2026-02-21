//! List configured corrkit spaces.

use anyhow::Result;

use crate::app_config;

pub fn run() -> Result<()> {
    let spaces = app_config::list_spaces()?;

    if spaces.is_empty() {
        println!("No spaces configured.");
        println!("Run 'corrkit init --user EMAIL' to create one.");
        return Ok(());
    }

    println!("corrkit spaces\n");
    let name_w = spaces.iter().map(|(n, _, _)| n.len()).max().unwrap_or(0);
    for (name, path, is_default) in &spaces {
        let marker = if *is_default { " (default)" } else { "" };
        println!("  {:<width$}  {}{}", name, path, marker, width = name_w);
    }
    Ok(())
}
