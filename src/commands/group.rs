// gitb group add/list/remove/show: manage repo groups in gitb.toml

use crate::cli::GroupAction;
use crate::config::WorkspaceConfig;
use crate::core::GlobalOpts;
use colored::*;

pub fn run(
    action: GroupAction,
    config: &WorkspaceConfig,
    _opts: &GlobalOpts,
) -> anyhow::Result<()> {
    match action {
        GroupAction::Add { name, repos } => run_add(config, &name, &repos),
        GroupAction::List => run_list(config),
        GroupAction::Remove { name } => run_remove(config, &name),
        GroupAction::Show { name } => run_show(config, &name),
    }
}

fn run_add(config: &WorkspaceConfig, name: &str, repos: &[String]) -> anyhow::Result<()> {
    let mut config = config.clone();
    config.add_group(name, repos.to_vec());
    config.save()?;
    println!(
        "{} Added group '{}' with {} repo(s)",
        "✓".green(),
        name.cyan(),
        repos.len()
    );
    for r in repos {
        println!("  - {}", r);
    }
    Ok(())
}

fn run_list(config: &WorkspaceConfig) -> anyhow::Result<()> {
    if config.groups.is_empty() {
        println!("No groups defined. Use 'gitb group add <name> <repo1,repo2,...>' to create one.");
        return Ok(());
    }

    println!("\n{}", "Groups".bold());
    println!("------");
    for (name, group) in &config.groups {
        println!("\n  {} ({} repos)", name.cyan(), group.repos.len());
        for repo in &group.repos {
            println!("    - {}", repo);
        }
    }
    println!();
    Ok(())
}

fn run_remove(config: &WorkspaceConfig, name: &str) -> anyhow::Result<()> {
    let mut config = config.clone();
    if config.remove_group(name) {
        config.save()?;
        println!("{} Removed group '{}'", "✓".green(), name.cyan());
    } else {
        eprintln!("{} Group '{}' not found", "✗".red(), name);
        std::process::exit(1);
    }
    Ok(())
}

fn run_show(config: &WorkspaceConfig, name: &str) -> anyhow::Result<()> {
    match config.groups.get(name) {
        Some(group) => {
            println!("\n{} ({} repos)", name.cyan(), group.repos.len());
            for repo in &group.repos {
                println!("  - {}", repo);
            }
            println!();
        }
        None => {
            eprintln!("{} Group '{}' not found", "✗".red(), name);
            std::process::exit(1);
        }
    }
    Ok(())
}
